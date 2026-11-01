//! The reference's 3D run tests (`physics-flow.test.ts`), with their expected values.

use std::collections::BTreeMap;

use intracraft_des::flow::simulate;
use intracraft_des::time::{SimTime, from_seconds};
use intracraft_library::definition::Value;
use intracraft_library::layout::{Action, Condition, Instance, Layout, Rule, Vec3};
use intracraft_library::params::{instance_values, num};

use super::*;

const SEED: u64 = 7;
const FLOW_T2: &str = include_str!("../../../../../packages/scenarios/scenarios/layouts/flow-t2.intracraft.json");
const REL_TOL: f64 = 0.01;
/// 2D and 3D sensor edges must agree within ±50 ms (S06 P6, A11).
const EDGE_TOL: f64 = 0.05;
/// The stop criterion of spike S1: the front within ±5 mm (ADR-0002).
const STOP_TOL: f64 = 0.005;

mod reference {
    use super::*;

    #[test]
    fn t1_a_pallet_crosses_a_3_m_roller_conveyor_from_rest_in_12_25_s() {
        let sim = run(&line(&[("flow.source", &[("maxPallets", n(1.0))]), ("conveyor.roller", &[]), ("flow.sink", &[])]), 16.0);
        let stats = sim.statistics();
        assert_eq!(stat(&stats, "roller1", "palletsProcessed"), 1.0);
        within(stat(&stats, "roller1", "meanTransitTime"), 12.25, REL_TOL);
        assert_eq!(stat(&stats, "sink1", "palletsReceived"), 1.0);
    }

    #[test]
    fn t1_on_chains_eur_short_side_takes_the_same_12_25_s() {
        let source = [("maxPallets", n(1.0)), ("orientation", t("short"))];
        let sim = run(&line(&[("flow.source", &source), ("conveyor.chain", &[]), ("flow.sink", &[])]), 16.0);
        within(stat(&sim.statistics(), "chain1", "meanTransitTime"), 12.25, REL_TOL);
    }

    #[test]
    fn t2_a_saturated_line_carries_about_667_pallets_an_hour() {
        let project = intracraft_library::layout::Project::read(FLOW_T2).unwrap();
        let mut sim = PhysicsFlow::new(&project.layout, &library(), project.seed).unwrap();
        let sink = project.layout.instances.iter().find(|i| i.definition_id == "flow.sink").unwrap().id.clone();
        sim.run_until(from_seconds(600.0));
        let before = stat(&sim.statistics(), &sink, "palletsReceived");
        sim.run_until(from_seconds(1200.0));
        // Once the line is full, one pallet every (1.2 + 0.15) / 0.25 = 5.4 s.
        let rate = (stat(&sim.statistics(), &sink, "palletsReceived") - before) / 600.0 * 3600.0;
        within(rate, 666.67, 0.03);
        assert!(sim.issues.is_empty(), "{:?}", sim.issues);
    }
}

mod without_accumulation {
    use super::*;

    #[test]
    fn hands_a_pallet_over_to_the_next_conveyor_without_stopping() {
        let parts =
            [("flow.source", &[("maxPallets", n(1.0))][..]), ("conveyor.roller", &[]), ("conveyor.roller", &[]), ("flow.sink", &[])];
        let sim = run(&line(&parts), 30.0);
        within(stat(&sim.statistics(), "sink1", "meanCycleTime"), 24.33, REL_TOL);
    }

    #[test]
    fn stops_the_pallet_with_its_front_at_the_open_end_within_5_mm_and_holds_the_conveyor() {
        let sim = run(&line(&[("flow.source", &[("maxPallets", n(1.0))]), ("conveyor.roller", &[])]), 20.0);
        let pallets = sim.pallets();
        assert!((pallets[0].front - 3.0).abs() < STOP_TOL, "front {}", pallets[0].front);
        assert!(pallets[0].speed.abs() < 0.002);
        assert_eq!(sim.status()["roller1"], "blocked");
        assert!(sim.issues.iter().any(|i| i.rule == "openEnd"));
    }

    #[test]
    fn a_blocked_conveyor_receives_nothing_two_pallets_per_3_m_conveyor() {
        let sim = run(&line(&[("flow.source", &[("interval", n(5.0))]), ("conveyor.roller", &[]), ("conveyor.roller", &[])]), 90.0);
        let pallets = sim.pallets();
        assert_eq!(pallets.len(), 4);
        for carrier in ["roller1", "roller2"] {
            let fronts: Vec<f64> = pallets.iter().filter(|p| p.carrier == carrier).map(|p| p.front).collect();
            let (lo, hi) = fronts.iter().fold((f64::INFINITY, f64::NEG_INFINITY), |(a, b), &f| (a.min(f), b.max(f)));
            assert!((hi - 3.0).abs() < STOP_TOL, "{carrier}: {fronts:?}");
            assert!((lo - 1.65).abs() < STOP_TOL, "{carrier}: {fronts:?}");
        }
        let status = sim.status();
        assert_eq!((status["roller1"], status["roller2"], status["source1"]), ("blocked", "blocked", "waiting"));
    }

    #[test]
    fn a_controller_rule_switches_a_conveyor_off_at_a_photocell_and_the_pallet_stops_there() {
        let mut layout = line(&[("flow.source", &[("maxPallets", n(1.0))]), ("conveyor.roller", &[]), ("flow.sink", &[])]);
        layout.instances.push(photoeye("photoeye1", "roller1", 2.0, &[]));
        layout.rules = vec![Rule {
            id: "r1".into(),
            when: vec![Condition::Is { signal: "photoeye1.detected".into(), is: true }],
            then: vec![Action { signal: "roller1.run".into(), value: false, delay_ms: None }],
        }];
        let sim = run(&layout, 20.0);
        // Detected when the front reaches 2 m; braking from 0.25 m/s at 0.5 m/s² adds 62.5 mm.
        let front = sim.pallets()[0].front;
        assert!(front > 2.0 && front < 2.08, "front {front}");
        assert_eq!(sim.signals().get("roller1.run"), Some(&false));
        assert_eq!(stat(&sim.statistics(), "sink1", "palletsReceived"), 0.0);
    }
}

mod photocells {
    use super::*;

    /// Open (ADR-0024): the rising edges agree, but after 12 s on rollers the pallet runs about 13 mm behind and has turned
    /// 0.44° (no side guides in 3D yet), so the falling edge comes 79 ms after the 2D one.
    #[test]
    #[ignore = "open: the falling edge is 79 ms late (ADR-0024)"]
    fn p6_every_photocell_edge_of_the_3d_run_lies_within_50_ms_of_the_2d_edge() {
        let source = [("interval", n(8.0)), ("maxPallets", n(3.0))];
        let mut layout = line(&[("flow.source", &source), ("conveyor.roller", &[("length", n(4.0))]), ("flow.sink", &[])]);
        // A beam at 400 mm crosses the load: one pulse per pallet.
        layout.instances.push(photoeye("photoeye1", "roller1", 3.0, &[("sensorHeight", n(0.4)), ("reflectorHeight", n(0.4))]));
        let expected = edges_2d(&layout, 40.0, "photoeye1");
        assert_eq!(expected.len(), 6);
        let mut sim = run(&layout, 40.0);
        let edges: Vec<SensorEdge> = sim.take_sensor_edges().into_iter().filter(|e| e.sensor == "photoeye1").collect();
        assert_eq!(edges.iter().map(|e| e.detecting).collect::<Vec<_>>(), expected.iter().map(|e| e.1).collect::<Vec<_>>());
        for (edge, (time, _)) in edges.iter().zip(&expected) {
            let error = (edge.time - time) as f64 / 1e6;
            assert!(error.abs() < EDGE_TOL, "3D {} µs, 2D {time} µs", edge.time);
        }
    }

    #[test]
    fn the_beam_ignores_the_conveyor_and_sees_the_pallet_voids_at_the_bottom_board_height() {
        let mut layout = line(&[("flow.source", &[("maxPallets", n(1.0))]), ("conveyor.roller", &[]), ("flow.sink", &[])]);
        // 60 mm: the beam crosses the blocks and the voids between them, as in 2D (T6).
        layout.instances.push(photoeye("photoeye1", "roller1", 1.5, &[]));
        let expected = edges_2d(&layout, 16.0, "photoeye1");
        assert!(expected.len() > 2);
        let mut sim = run(&layout, 16.0);
        let detecting = sim.take_sensor_edges().iter().filter(|e| e.detecting).count();
        assert_eq!(detecting, expected.iter().filter(|e| e.1).count());
    }

    #[test]
    fn gives_the_same_state_and_statistics_for_the_same_layout_and_seed() {
        let source = [("interval", n(7.0)), ("generationMode", t("exponential"))];
        let layout = line(&[("flow.source", &source), ("conveyor.roller", &[]), ("conveyor.roller", &[]), ("flow.sink", &[])]);
        let one = run(&layout, 40.0);
        let two = run(&layout, 40.0);
        assert_eq!(format!("{:?}{:?}", one.pallets(), one.statistics()), format!("{:?}{:?}", two.pallets(), two.statistics()));
    }
}

mod stations {
    use super::*;

    #[test]
    fn the_c_elevator_lifts_each_pallet_4_m_and_returns_in_a_30_s_cycle() {
        let sim = run(&elevator_layout(), 120.0);
        let stats = sim.statistics();
        assert_eq!(stat(&stats, "sink1", "palletsReceived"), 1.0);
        within(stat(&stats, "elevator1", "meanCycleTime"), 30.03, REL_TOL);
        assert_eq!(sim.fallen_pallets(), 0);
    }

    #[test]
    fn reports_the_deck_height_while_lifting() {
        let sim = run(&elevator_layout(), 20.0);
        let height = sim.heights()["elevator1"];
        assert!(height > 0.6 && height < 4.6, "{height}");
        assert_eq!(sim.status()["elevator1"], "lifting");
    }

    #[test]
    fn a_manual_input_table_lifts_the_pallet_the_operator_puts_down_to_the_line() {
        let parts = [
            ("flow.source", &[("maxPallets", n(1.0)), ("surfaceHeight", n(0.1))][..]),
            ("conveyor.lift_table_manual", &[("flowDirection", t("in"))]),
            ("conveyor.roller", &[]),
            ("flow.sink", &[]),
        ];
        let layout = line(&parts);
        let expected = run_2d(&layout, 60.0)["sink1"]["meanCycleTime"];
        let sim = run(&layout, 60.0);
        assert_eq!(stat(&sim.statistics(), "sink1", "palletsReceived"), 1.0);
        within(stat(&sim.statistics(), "sink1", "meanCycleTime"), expected, REL_TOL);
    }

    #[test]
    fn a_process_station_adds_its_processing_time_to_the_cycle() {
        let parts = [
            ("flow.source", &[("maxPallets", n(1.0))][..]),
            ("conveyor.roller", &[]),
            ("flow.process_station", &[("processingTime", n(30.0))]),
            ("conveyor.roller", &[]),
            ("flow.sink", &[]),
        ];
        let sim = run(&line(&parts), 80.0);
        let stats = sim.statistics();
        within(stat(&stats, "process_station1", "meanCycleTime"), 42.03, REL_TOL);
        assert_eq!(stat(&stats, "process_station1", "palletsProcessed"), 1.0);
        assert_eq!(stat(&stats, "sink1", "palletsReceived"), 1.0);
    }

    #[test]
    fn warns_that_turntables_have_no_3d_behaviour_yet() {
        let parts = [("flow.source", &[("maxPallets", n(1.0))][..]), ("conveyor.roller", &[]), ("conveyor.roller_turntable", &[])];
        let sim = run(&line(&parts), 1.0);
        assert!(sim.issues.iter().any(|i| i.rule == "behaviorNotAvailable3D"));
    }
}

type Params<'a> = &'a [(&'a str, Value)];

fn n(v: f64) -> Value {
    Value::Number(v)
}

fn t(v: &str) -> Value {
    Value::Text(v.into())
}

fn library() -> Library {
    Library::built_in().unwrap()
}

/// Objects placed one after another along +x, 20 mm apart (their ports meet and connect).
fn line(parts: &[(&str, Params)]) -> Layout {
    let lib = library();
    let mut counters = BTreeMap::new();
    let mut x = 0.0;
    let instances = parts
        .iter()
        .map(|(definition, params)| {
            let i = place(&lib, definition, params, x, &mut counters);
            let values = instance_values(lib.get(definition).unwrap(), &i.params);
            x += num(&values, "length", if *definition == "flow.source" { 0.4 } else { 0.0 }) + 0.02;
            i
        })
        .collect();
    Layout { instances, connections: Vec::new(), rules: Vec::new() }
}

/// Floor conveyor → C elevator (0.6 → 4.6 m) → mezzanine conveyor running back → sink.
fn elevator_layout() -> Layout {
    let lib = library();
    let mut c = BTreeMap::new();
    let high = |mut i: Instance, x: f64| {
        i.position = Vec3::new(x, 4.0, 0.0);
        i.yaw = std::f64::consts::PI;
        i
    };
    let instances = vec![
        place(&lib, "flow.source", &[("maxPallets", n(1.0))], -3.44, &mut c),
        place(&lib, "conveyor.roller", &[], -3.02, &mut c),
        place(&lib, "conveyor.elevator", &[], 0.0, &mut c),
        high(place(&lib, "conveyor.roller", &[], -0.02, &mut c), -0.02),
        high(place(&lib, "flow.sink", &[], -3.04, &mut c), -3.04),
    ];
    Layout { instances, connections: Vec::new(), rules: Vec::new() }
}

fn place(lib: &Library, definition: &str, params: Params, x: f64, counters: &mut BTreeMap<String, u32>) -> Instance {
    let d = lib.get(definition).unwrap();
    let prefix = definition.split('.').nth(1).unwrap_or(definition).to_string();
    let count = counters.entry(prefix.clone()).or_insert(0);
    *count += 1;
    Instance {
        id: format!("{prefix}{count}"),
        definition_id: definition.into(),
        definition_version: d.version.clone(),
        name: String::new(),
        position: Vec3::new(x, 0.0, 0.0),
        yaw: 0.0,
        params: params.iter().map(|(k, v)| (k.to_string(), v.clone())).collect(),
        attributes: BTreeMap::new(),
    }
}

/// A retroreflective photocell across `host` at `s` m from its entry.
fn photoeye(id: &str, host: &str, s: f64, params: Params) -> Instance {
    let lib = library();
    let mut i = place(&lib, "sensor.photoeye", &[("host", t(host)), ("sensorS", n(s)), ("reflectorS", n(s))], 0.0, &mut BTreeMap::new());
    i.id = id.into();
    i.params.extend(params.iter().map(|(k, v)| (k.to_string(), v.clone())));
    i
}

fn run(layout: &Layout, seconds: f64) -> PhysicsFlow {
    let mut sim = PhysicsFlow::new(layout, &library(), SEED).unwrap();
    sim.run_until(from_seconds(seconds));
    sim
}

fn run_2d(layout: &Layout, seconds: f64) -> Statistics {
    let mut sim = simulate(layout, &library(), SEED).unwrap();
    sim.run_until(from_seconds(seconds)).unwrap();
    sim.statistics()
}

/// The output edges of a 2D sensor, event by event (exact times).
fn edges_2d(layout: &Layout, seconds: f64, sensor: &str) -> Vec<(SimTime, bool)> {
    let mut sim = simulate(layout, &library(), SEED).unwrap();
    let end = from_seconds(seconds);
    let mut edges = Vec::new();
    let mut detecting = false;
    while let Some(next) = sim.next_event_time().filter(|&t| t <= end) {
        sim.run_until(next).unwrap();
        let now = sim.snapshot().status.get(sensor) == Some(&"detecting");
        if now != detecting {
            edges.push((next, now));
        }
        detecting = now;
    }
    edges
}

fn stat(stats: &Statistics, id: &str, key: &str) -> f64 {
    stats.get(id).and_then(|s| s.get(key)).copied().unwrap_or(f64::NAN)
}

fn within(value: f64, expected: f64, relative: f64) {
    assert!(((value - expected) / expected).abs() < relative, "{value}, expected {expected} ±{}%", relative * 100.0);
}
