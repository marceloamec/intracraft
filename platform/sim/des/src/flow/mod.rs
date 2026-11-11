//! The 2D flow model: the network of a layout, and its run.

pub mod gravity;
pub mod network;
pub mod parts;
pub mod sim;
pub(crate) mod station;
pub mod storage;
pub mod warehouse;
pub mod wrap;

use intracraft_library::Library;
use intracraft_library::layout::Layout;
use intracraft_library::snap::derive_connections;

pub use network::{FlowIssue, FlowNetwork, build_flow_network};
pub use sim::{CodeBlocks, FlowSimulation, PalletView, Snapshot, Statistics, describe};
pub use station::processing_time;

/// The run of a layout as a project holds it, with its code blocks (IntraScript, A17); an error when the layout doesn't
/// connect or its code has check errors (no connections stored: they come from the geometry, as in the editor).
pub fn simulate(layout: &Layout, library: &Library, seed: u64) -> Result<FlowSimulation, String> {
    let mut connected = layout.clone();
    connected.connections = derive_connections(layout, library)?;
    let mut sim = FlowSimulation::new(build_flow_network(&connected, library), seed, &library.reference);
    sim.attach_scripts(&connected, library, seed)?;
    Ok(sim)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::time::{from_seconds, to_seconds};
    use intracraft_library::layout::{Action, Condition, Project, Rule};

    const FLOW_T1: &str = include_str!("../../../../../packages/scenarios/scenarios/layouts/flow-t1.intracraft.json");
    const FLOW_T2: &str = include_str!("../../../../../packages/scenarios/scenarios/layouts/flow-t2.intracraft.json");
    const ROLLER_LINE: &str = include_str!("../../../../../packages/scenarios/scenarios/layouts/demo-roller-line.intracraft.json");

    fn run(json: &str, seconds: f64) -> FlowSimulation {
        let lib = Library::built_in().unwrap();
        let project = Project::read(json).unwrap();
        let mut sim = simulate(&project.layout, &lib, project.seed).unwrap();
        sim.run_until(from_seconds(seconds)).unwrap();
        sim
    }

    fn within(actual: f64, expected: f64) {
        assert!(((actual - expected) / expected).abs() < 0.01, "{actual} is not within 1 % of {expected}");
    }

    #[test]
    fn a_conveyor_that_runs_off_its_open_end_lets_its_pallets_fall_and_never_holds() {
        use intracraft_library::definition::Value;
        let lib = Library::built_in().unwrap();
        let mut project = Project::read(FLOW_T1).unwrap();
        project.layout.instances.retain(|i| i.definition_id != "flow.sink");
        for i in &mut project.layout.instances {
            match i.definition_id.as_str() {
                "conveyor.roller" => _ = i.params.insert("openEnd".into(), Value::Text("runOff".into())),
                "flow.source" => {
                    i.params.insert("maxPallets".into(), Value::Number(0.0));
                    i.params.insert("interval".into(), Value::Number(10.0));
                }
                _ => {}
            }
        }
        let mut sim = simulate(&project.layout, &lib, project.seed).unwrap();
        assert!(sim.issues.is_empty(), "{:?}", sim.issues);
        sim.run_until(from_seconds(65.0)).unwrap();
        // One pallet every 10 s; each falls once its rear is over the end, 3 m from rest, in 12.25 s (T1): at 12.25, 22.25
        // … 62.25 s, without the conveyor ever stopping.
        assert_eq!(sim.fallen_pallets(), 6);
        assert!(sim.statistics()["roller1"]["blockedTime"] < 1e-9);
    }

    #[test]
    fn t1_2d_a_pallet_crosses_a_3_m_conveyor_in_12_25_s() {
        let mut sim = run(FLOW_T1, 60.0);
        within(sim.statistics()["roller1"]["meanTransitTime"], 12.25);
    }

    #[test]
    fn t2_2d_a_saturated_line_carries_about_667_pallets_an_hour() {
        let mut sim = run(FLOW_T2, 28_800.0);
        within(sim.statistics()["sink1"]["throughputPerHour"], 666.6666666666666);
    }

    #[test]
    fn the_same_layout_and_seed_give_the_same_run() {
        let a = run(ROLLER_LINE, 600.0).snapshot();
        let b = run(ROLLER_LINE, 600.0).snapshot();
        assert_eq!(a, b);
        assert!(!a.pallets.is_empty() || a.status.values().any(|s| *s != "empty"));
    }

    #[test]
    fn pallets_never_overlap_or_go_backwards() {
        let lib = Library::built_in().unwrap();
        let project = Project::read(FLOW_T2).unwrap();
        let mut sim = simulate(&project.layout, &lib, project.seed).unwrap();
        let mut last: std::collections::BTreeMap<u64, (String, f64)> = Default::default();
        for step in 1..=600 {
            sim.run_until(from_seconds(f64::from(step) * 0.5)).unwrap();
            let snap = sim.snapshot();
            let mut by_carrier: std::collections::BTreeMap<&str, Vec<&PalletView>> = Default::default();
            for p in &snap.pallets {
                by_carrier.entry(p.carrier.as_str()).or_default().push(p);
                if let Some((carrier, front)) = last.get(&p.id)
                    && carrier == &p.carrier
                {
                    assert!(p.front >= front - 1e-9, "pallet {} went backwards at {} s", p.id, to_seconds(snap.time));
                }
                last.insert(p.id, (p.carrier.clone(), p.front));
            }
            for pallets in by_carrier.values() {
                for w in pallets.windows(2) {
                    assert!(w[1].front <= w[0].front - w[0].length + 1e-6, "overlap at {} s", to_seconds(snap.time));
                }
            }
        }
    }

    #[test]
    fn a_rule_stops_the_conveyor_and_the_pallet_waits() {
        // The reference's control test: when the source's pallet is on the conveyor, a rule switches it off for 10 s.
        let lib = Library::built_in().unwrap();
        let mut project = Project::read(FLOW_T1).unwrap();
        let conveyor = project.layout.instances.iter().find(|i| i.definition_id == "conveyor.roller").unwrap().id.clone();
        project.layout.rules = vec![Rule {
            id: "stop".into(),
            when: vec![Condition::Is { signal: format!("{conveyor}.occupied"), is: true }],
            then: vec![Action { signal: format!("{conveyor}.run"), value: false, delay_ms: None }],
        }];
        let mut sim = simulate(&project.layout, &lib, project.seed).unwrap();
        sim.run_until(from_seconds(30.0)).unwrap();
        let snap = sim.snapshot();
        assert!(!snap.signals[&format!("{conveyor}.run")]);
        assert_eq!(snap.status[&conveyor], "stopped");
        assert!(snap.pallets.iter().all(|p| p.speed == 0.0));
    }

    mod turntable {
        use super::super::*;
        use crate::time::from_seconds;
        use intracraft_library::definition::Value;
        use intracraft_library::layout::{Instance, Vec3};
        use std::f64::consts::FRAC_PI_2;

        /// T3 (6.5 s) plus the 0.3 s settling of the default roller turntable.
        const TURN_90: f64 = 6.8;
        const CX: f64 = 3.72;

        fn instance(id: &str, definition: &str, x: f64, z: f64, yaw: f64, params: &[(&str, Value)]) -> Instance {
            Instance {
                id: id.into(),
                definition_id: definition.into(),
                definition_version: "0.1.0".into(),
                name: id.into(),
                position: Vec3::new(x, 0.0, z),
                yaw,
                params: params.iter().map(|(k, v)| (k.to_string(), v.clone())).collect(),
                attributes: Default::default(),
            }
        }

        /// Source → roller1 → roller turntable, then straight: P0 → roller2 → sink1; corner: P90 → roller2 → sink1;
        /// fork: P0 → sink1 and P90 → roller2 → sink2.
        fn layout(shape: &str, source: &[(&str, Value)], exit_rule: &str) -> Layout {
            let mut instances = vec![
                instance("source1", "flow.source", -0.42, 0.0, 0.0, source),
                instance("roller1", "conveyor.roller", 0.0, 0.0, 0.0, &[]),
                instance("roller_turntable1", "conveyor.roller_turntable", CX, 0.0, 0.0, &[("exitRule", Value::Text(exit_rule.into()))]),
            ];
            let (ahead, side) = (CX + 0.72, -0.72);
            if shape == "straight" {
                instances.push(instance("roller2", "conveyor.roller", ahead, 0.0, 0.0, &[]));
                instances.push(instance("sink1", "flow.sink", ahead + 3.02, 0.0, 0.0, &[]));
            } else {
                instances.push(instance("roller2", "conveyor.roller", CX, side, FRAC_PI_2, &[]));
                let sink = if shape == "fork" { "sink2" } else { "sink1" };
                instances.push(instance(sink, "flow.sink", CX, side - 3.02, FRAC_PI_2, &[]));
                if shape == "fork" {
                    instances.push(instance("sink1", "flow.sink", ahead, 0.0, 0.0, &[]));
                }
            }
            Layout { instances, ..Layout::default() }
        }

        fn n(x: f64) -> Value {
            Value::Number(x)
        }

        fn run(layout: &Layout, seconds: f64) -> FlowSimulation {
            let lib = Library::built_in().unwrap();
            let mut sim = simulate(layout, &lib, 3).unwrap();
            sim.run_until(from_seconds(seconds)).unwrap();
            sim
        }

        #[test]
        fn passes_a_pallet_straight_through_without_turning() {
            let mut sim = run(&layout("straight", &[("maxPallets", n(1.0))], "alternate"), 120.0);
            let stats = sim.statistics();
            assert_eq!(stats["sink1"]["palletsReceived"], 1.0);
            assert_eq!((stats["roller_turntable1"]["turns"], stats["roller_turntable1"]["palletsProcessed"]), (0.0, 1.0));
        }

        #[test]
        fn a_corner_adds_exactly_one_turn_to_the_time_on_the_platform() {
            let straight =
                run(&layout("straight", &[("maxPallets", n(1.0))], "alternate"), 120.0).statistics()["roller_turntable1"]["meanCycleTime"];
            let corner = run(&layout("corner", &[("maxPallets", n(1.0))], "alternate"), 120.0).statistics();
            assert_eq!(corner["sink1"]["palletsReceived"], 1.0);
            assert_eq!(corner["roller_turntable1"]["turns"], 1.0);
            let added = corner["roller_turntable1"]["meanCycleTime"] - straight;
            assert!((added - TURN_90).abs() < 1e-5, "the corner added {added} s");
        }

        #[test]
        fn turns_back_to_the_entry_for_the_next_pallet() {
            let stats = run(&layout("corner", &[("maxPallets", n(2.0)), ("interval", n(5.0))], "alternate"), 300.0).statistics();
            assert_eq!(stats["sink1"]["palletsReceived"], 2.0);
            assert_eq!(stats["roller_turntable1"]["turns"], 3.0);
        }

        #[test]
        fn alternates_between_two_exits_or_always_takes_the_first() {
            let src = [("maxPallets", n(6.0)), ("interval", n(10.0))];
            let alternate = run(&layout("fork", &src, "alternate"), 900.0).statistics();
            assert_eq!((alternate["sink1"]["palletsReceived"], alternate["sink2"]["palletsReceived"]), (3.0, 3.0));
            let first = run(&layout("fork", &src, "firstExit"), 900.0).statistics();
            let (a, b) = (first["sink1"]["palletsReceived"], first["sink2"]["palletsReceived"]);
            assert_eq!(a + b, 6.0);
            assert!(a == 0.0 || b == 0.0);
        }

        #[test]
        fn chooses_randomly_but_reproducibly() {
            let counts = || {
                let s = run(&layout("fork", &[("maxPallets", n(20.0)), ("interval", n(10.0))], "random"), 3600.0).statistics();
                (s["sink1"]["palletsReceived"], s["sink2"]["palletsReceived"])
            };
            let first = counts();
            assert_eq!(first.0 + first.1, 20.0);
            assert!(first.0 > 0.0 && first.1 > 0.0);
            assert_eq!(counts(), first);
        }

        #[test]
        fn carries_the_pallet_round_with_the_platform() {
            let lib = Library::built_in().unwrap();
            let mut sim = simulate(&layout("corner", &[("maxPallets", n(1.0))], "alternate"), &lib, 1).unwrap();
            // Run until the platform turns, then to the middle of the turn.
            let mut t = 0.0;
            while sim.snapshot().status.get("roller_turntable1").copied() != Some("turning") && t < 60.0 {
                t += 0.05;
                sim.run_until(from_seconds(t)).unwrap();
            }
            sim.run_until(from_seconds(t + 3.4)).unwrap();
            let snap = sim.snapshot();
            let p = &snap.pallets[0];
            assert_eq!(p.carrier, "roller_turntable1");
            assert!((p.heading.abs() - FRAC_PI_2 / 2.0).abs() < 0.05, "heading {}", p.heading);
            assert_eq!(snap.angles["roller_turntable1"], p.heading);
            // The pallet stays centred on the platform.
            let along = p.front - p.length / 2.0;
            assert!((p.path_x + along * p.heading.cos()).abs() < 1e-6);
            assert!((p.path_z - along * p.heading.sin()).abs() < 1e-6);
        }

        #[test]
        fn warns_when_no_station_leads_anywhere() {
            let mut no_exit = layout("straight", &[("maxPallets", n(1.0))], "alternate");
            no_exit.instances.retain(|i| i.id != "roller2" && i.id != "sink1");
            let sim = run(&no_exit, 60.0);
            assert!(sim.issues.iter().any(|i| i.rule == "noExit"));
        }
    }

    mod transfer {
        use super::super::*;
        use crate::time::from_seconds;
        use intracraft_library::definition::Value;
        use intracraft_library::layout::{Instance, Vec3};
        use std::f64::consts::{FRAC_PI_2, PI};

        /// A yaw that points an object's +x to the world's +z (or −z).
        const TO_PLUS_Z: f64 = 3.0 * PI / 2.0;
        const TO_MINUS_Z: f64 = FRAC_PI_2;
        /// The transfer table's A at x = 3.02; 1.4 m long, 1.0 m wide: its centre at x = 3.72, chain ports at z = ±0.5.
        const TX: f64 = 3.02;
        const CX: f64 = TX + 0.7;
        const CHAIN_HEIGHT: f64 = 0.64;

        fn instance(id: &str, definition: &str, x: f64, z: f64, yaw: f64, params: &[(&str, Value)]) -> Instance {
            Instance {
                id: id.into(),
                definition_id: definition.into(),
                definition_version: "0.1.0".into(),
                name: id.into(),
                position: Vec3::new(x, 0.0, z),
                yaw,
                params: params.iter().map(|(k, v)| (k.to_string(), v.clone())).collect(),
                attributes: Default::default(),
            }
        }

        fn n(x: f64) -> Value {
            Value::Number(x)
        }

        fn t(x: &str) -> Value {
            Value::Text(x.into())
        }

        /// Source → roller1 → transfer (A) → C2 → chain1 → sink1 (and C1 → chain2 → sink2 when `both`).
        fn rollers_to_chains(source: &[(&str, Value)], transfer: &[(&str, Value)], both: bool) -> Layout {
            let h = [("surfaceHeight", n(CHAIN_HEIGHT))];
            let mut instances = vec![
                instance("source1", "flow.source", -0.42, 0.0, 0.0, source),
                instance("roller1", "conveyor.roller", 0.0, 0.0, 0.0, &[]),
                instance("transfer_rc1", "conveyor.transfer_rc", TX, 0.0, 0.0, transfer),
                instance("chain1", "conveyor.chain", CX, 0.52, TO_PLUS_Z, &h),
                instance("sink1", "flow.sink", CX, 3.54, TO_PLUS_Z, &h),
            ];
            if both {
                instances.push(instance("chain2", "conveyor.chain", CX, -0.52, TO_MINUS_Z, &h));
                instances.push(instance("sink2", "flow.sink", CX, -3.54, TO_MINUS_Z, &h));
            }
            Layout { instances, ..Layout::default() }
        }

        /// Source → chain1 (running +z) → transfer C1 → B → roller1 → sink1.
        fn chains_to_rollers() -> Layout {
            let h = ("surfaceHeight", n(CHAIN_HEIGHT));
            Layout {
                instances: vec![
                    instance(
                        "source1",
                        "flow.source",
                        0.7,
                        -3.94,
                        TO_PLUS_Z,
                        &[("maxPallets", n(1.0)), ("orientation", t("short")), h.clone()],
                    ),
                    instance("chain1", "conveyor.chain", 0.7, -3.52, TO_PLUS_Z, &[h]),
                    instance("transfer_rc1", "conveyor.transfer_rc", 0.0, 0.0, 0.0, &[("operationMode", t("chainsToRollers"))]),
                    instance("roller1", "conveyor.roller", 1.42, 0.0, 0.0, &[]),
                    instance("sink1", "flow.sink", 4.44, 0.0, 0.0, &[]),
                ],
                ..Layout::default()
            }
        }

        fn run(layout: &Layout, seconds: f64) -> FlowSimulation {
            let lib = Library::built_in().unwrap();
            let mut sim = simulate(layout, &lib, 5).unwrap();
            sim.run_until(from_seconds(seconds)).unwrap();
            sim
        }

        fn rules(sim: &FlowSimulation) -> Vec<&'static str> {
            sim.issues.iter().map(|i| i.rule).collect()
        }

        #[test]
        fn lifts_the_pallet_off_the_rollers_and_sends_it_sideways_short_side_first() {
            let mut sim = run(&rollers_to_chains(&[("maxPallets", n(1.0))], &[], false), 120.0);
            let s = sim.statistics();
            assert_eq!(s["sink1"]["palletsReceived"], 1.0);
            assert_eq!((s["transfer_rc1"]["transfers"], s["transfer_rc1"]["palletsProcessed"]), (1.0, 1.0));
            assert!(!rules(&sim).contains(&"palletOrientationMismatch"));
        }

        #[test]
        fn adds_the_lift_and_lowering_times_to_its_cycle() {
            let cycle = |params: &[(&str, Value)]| {
                run(&rollers_to_chains(&[("maxPallets", n(1.0))], params, false), 120.0).statistics()["transfer_rc1"]["meanCycleTime"]
            };
            let added = cycle(&[("liftTime", n(2.0)), ("lowerTime", n(2.5))]) - cycle(&[("liftTime", n(1.0)), ("lowerTime", n(1.0))]);
            assert!((added - 2.5).abs() < 1e-5, "{added}");
        }

        #[test]
        fn keeps_the_rollers_closed_while_the_chains_work() {
            let s = run(&rollers_to_chains(&[("interval", n(1.0))], &[], false), 3600.0).statistics();
            let cycle = s["transfer_rc1"]["meanCycleTime"];
            assert!(s["transfer_rc1"]["utilization"] > 0.9);
            assert!(s["sink1"]["throughputPerHour"] <= 3600.0 / cycle + 1.0);
        }

        #[test]
        fn warns_and_keeps_the_pallet_when_the_chosen_side_is_not_connected() {
            let mut sim = run(&rollers_to_chains(&[("maxPallets", n(1.0))], &[("transferDirection", t("left"))], false), 60.0);
            assert!(rules(&sim).contains(&"noExit"));
            assert_eq!(sim.snapshot().pallets[0].carrier, "transfer_rc1");
        }

        #[test]
        fn sends_pallets_to_both_sides_in_turn_when_bidirectional() {
            let layout =
                rollers_to_chains(&[("maxPallets", n(4.0)), ("interval", n(15.0))], &[("transferDirection", t("bidirectional"))], true);
            let s = run(&layout, 600.0).statistics();
            assert_eq!((s["sink1"]["palletsReceived"], s["sink2"]["palletsReceived"]), (2.0, 2.0));
        }

        #[test]
        fn takes_pallets_from_the_chains_and_sends_them_on_along_the_rollers() {
            let mut sim = run(&chains_to_rollers(), 120.0);
            let s = sim.statistics();
            assert_eq!(s["sink1"]["palletsReceived"], 1.0);
            assert_eq!(s["transfer_rc1"]["transfers"], 1.0);
            assert!(!rules(&sim).contains(&"palletOrientationMismatch"));
        }

        #[test]
        fn ref_5_the_reference_layout_with_all_five_conveyors_delivers_120_pallets_an_hour() {
            let lib = Library::built_in().unwrap();
            let project = intracraft_library::layout::Project::read(include_str!(
                "../../../../../packages/scenarios/scenarios/layouts/reference-all-five.intracraft.json"
            ))
            .unwrap();
            let mut sim = simulate(&project.layout, &lib, project.seed).unwrap();
            sim.run_until(from_seconds(28_800.0)).unwrap();
            let rate = sim.statistics()["sink1"]["throughputPerHour"];
            assert!(((rate - 120.0) / 120.0).abs() < 0.01, "{rate} pallets/h");
        }
    }

    /// The stretch wrapper (S25 §8): source → roller1 → wrapper1 → roller2 → sink1, in line along x.
    mod wrapper {
        use super::super::*;
        use crate::time::from_seconds;
        use intracraft_library::definition::Value;
        use intracraft_library::layout::{Instance, Vec3};

        /// The wrapper's table: 1.8 m across, its centre 0.9 m from its entry.
        const TABLE: f64 = 1.8;
        /// Facing ports join 20 mm apart.
        const AT: f64 = 3.02;

        fn instance(id: &str, definition: &str, x: f64, params: &[(&str, Value)]) -> Instance {
            Instance {
                id: id.into(),
                definition_id: definition.into(),
                definition_version: "0.1.0".into(),
                name: id.into(),
                position: Vec3::new(x, 0.0, 0.0),
                yaw: 0.0,
                params: params.iter().map(|(k, v)| (k.to_string(), v.clone())).collect(),
                attributes: Default::default(),
            }
        }

        /// The line with `station` (the wrapper, or a process station to compare with); without a sink the pallet stops
        /// at roller2's end, where it can be looked at.
        fn line(station: &str, source: &[(&str, Value)], params: &[(&str, Value)], sink: bool) -> Layout {
            let after = AT + TABLE + 0.02;
            let mut instances = vec![
                instance("source1", "flow.source", -0.42, source),
                instance("roller1", "conveyor.roller", 0.0, &[]),
                instance("wrapper1", station, AT, params),
                instance("roller2", "conveyor.roller", after, &[]),
            ];
            if sink {
                instances.push(instance("sink1", "flow.sink", after + 3.02, &[]));
            }
            Layout { instances, ..Layout::default() }
        }

        fn n(x: f64) -> Value {
            Value::Number(x)
        }

        fn one() -> [(&'static str, Value); 1] {
            [("maxPallets", n(1.0))]
        }

        fn run(layout: &Layout, seconds: f64) -> FlowSimulation {
            let lib = Library::built_in().unwrap();
            let mut sim = simulate(layout, &lib, 5).unwrap();
            sim.run_until(from_seconds(seconds)).unwrap();
            sim
        }

        fn within(actual: f64, expected: f64) {
            assert!(((actual - expected) / expected).abs() < 0.01, "{actual} is not within 1 % of {expected}");
        }

        #[test]
        fn w1_a_load_of_1_2_m_is_wrapped_in_10_turns_and_59_s_and_leaves_wrapped_in_line() {
            let mut sim = run(&line("flow.stretch_wrapper", &one(), &[], false), 200.0);
            let s = sim.statistics();
            let w = &s["wrapper1"];
            within(w["meanProcessingTime"], 59.0);
            assert_eq!((w["palletsProcessed"], w["meanTurns"], w["tooTallPallets"]), (1.0, 10.0, 0.0));
            // 10 turns round an EUR pallet's 4 m perimeter.
            assert!((w["filmUsed"] - 40.0).abs() < 1e-9, "{}", w["filmUsed"]);
            let snap = sim.snapshot();
            assert_eq!(snap.angles["wrapper1"], 0.0);
            let p = &snap.pallets[0];
            assert_eq!((p.carrier.as_str(), p.wrapped, p.heading), ("roller2", true, 0.0));
            assert!(!sim.issues.iter().any(|i| i.instance_id == "wrapper1"), "{:?}", sim.issues);
        }

        #[test]
        fn the_pallet_turns_with_the_table_and_stays_centred_on_it() {
            let lib = Library::built_in().unwrap();
            let mut sim = simulate(&line("flow.stretch_wrapper", &one(), &[], false), &lib, 5).unwrap();
            let mut t = 0.0;
            while sim.snapshot().status.get("wrapper1").copied() != Some("processing") && t < 60.0 {
                t += 0.05;
                sim.run_until(from_seconds(t)).unwrap();
            }
            // Before the film is attached the table stands still; 10 s into the turning it has turned.
            assert_eq!(sim.snapshot().angles["wrapper1"], 0.0);
            sim.run_until(from_seconds(t + 13.3)).unwrap();
            let snap = sim.snapshot();
            let p = &snap.pallets[0];
            assert_eq!(p.carrier, "wrapper1");
            assert!(!p.wrapped);
            assert!(p.heading > 0.1, "heading {}", p.heading);
            assert_eq!(snap.angles["wrapper1"], p.heading);
            let along = p.front - p.length / 2.0;
            assert!((p.path_x + along * p.heading.cos() - TABLE / 2.0).abs() < 1e-6);
            assert!((p.path_z - along * p.heading.sin()).abs() < 1e-6);
        }

        #[test]
        fn w2_a_saturated_line_runs_as_a_process_station_of_59_s() {
            let saturated = [("interval", n(1.0))];
            let mut wrapper = run(&line("flow.stretch_wrapper", &saturated, &[], true), 28_800.0);
            let process = [("processingTime", n(59.0)), ("length", n(TABLE))];
            let mut station = run(&line("flow.process_station", &saturated, &process, true), 28_800.0);
            let (w, s) = (wrapper.statistics(), station.statistics());
            assert!((w["wrapper1"]["meanCycleTime"] - s["wrapper1"]["meanCycleTime"]).abs() < 1e-6);
            let cycle = w["wrapper1"]["meanCycleTime"];
            assert!(cycle > 59.0);
            within(w["sink1"]["throughputPerHour"], 3600.0 / cycle);
            assert!(w["wrapper1"]["utilization"] > 0.8);
        }

        #[test]
        fn w3_up_only_takes_7_turns_and_44_s_and_a_pallet_without_a_load_runs_straight_through() {
            let s = run(&line("flow.stretch_wrapper", &one(), &[("passes", Value::Text("up".into()))], true), 200.0).statistics();
            within(s["wrapper1"]["meanProcessingTime"], 44.0);
            assert_eq!(s["wrapper1"]["meanTurns"], 7.0);
            let bare = [("maxPallets", n(1.0)), ("loadKind", Value::Text("none".into()))];
            let mut sim = run(&line("flow.stretch_wrapper", &bare, &[], false), 200.0);
            let s = sim.statistics();
            assert_eq!((s["wrapper1"]["meanProcessingTime"], s["wrapper1"]["meanTurns"], s["wrapper1"]["filmUsed"]), (0.0, 0.0, 0.0));
            assert!(!sim.snapshot().pallets[0].wrapped);
        }

        #[test]
        fn counts_a_load_above_the_mast_as_too_tall() {
            let tall = [("maxPallets", n(1.0)), ("loadHeight", n(2.2))];
            let s = run(&line("flow.stretch_wrapper", &tall, &[("maxLoadHeight", n(1.6))], true), 300.0).statistics();
            assert_eq!(s["wrapper1"]["tooTallPallets"], 1.0);
        }

        #[test]
        fn the_same_layout_gives_the_same_result() {
            let layout =
                line("flow.stretch_wrapper", &[("interval", n(30.0)), ("generationMode", Value::Text("exponential".into()))], &[], true);
            assert_eq!(run(&layout, 3600.0).statistics(), run(&layout, 3600.0).statistics());
        }
    }

    /// The reference's layout scenarios (`packages/scenarios/scenarios/layouts.json`): the same files, the same expected
    /// values, within 1 %.
    mod scenarios {
        use super::super::*;
        use crate::time::from_seconds;
        use intracraft_library::layout::Project;

        fn statistic(json: &str, seconds: f64, instance: &str, statistic: &str) -> f64 {
            let lib = Library::built_in().unwrap();
            let project = Project::read(json).unwrap();
            let mut sim = simulate(&project.layout, &lib, project.seed).unwrap();
            sim.run_until(from_seconds(seconds)).unwrap();
            let stats = sim.statistics();
            *stats.get(instance).and_then(|s| s.get(statistic)).unwrap_or_else(|| panic!("no {instance}.{statistic}: {:?}", sim.issues))
        }

        fn within(actual: f64, expected: f64) {
            assert!(((actual - expected) / expected).abs() < 0.01, "{actual} is not within 1 % of {expected}");
        }

        #[test]
        fn proc_1_the_process_line_processes_30_pallets_an_hour() {
            let json = include_str!("../../../../../packages/scenarios/scenarios/layouts/demo-process-line.intracraft.json");
            within(statistic(json, 3600.0, "process_station1", "palletsProcessed"), 30.0);
        }

        #[test]
        fn lift_c_the_c_elevator_delivers_60_pallets_an_hour() {
            let json = include_str!("../../../../../packages/scenarios/scenarios/layouts/demo-elevator-c.intracraft.json");
            within(statistic(json, 28_800.0, "sink1", "throughputPerHour"), 60.0);
        }

        #[test]
        fn t10_a_pallet_rolls_down_the_gravity_line_to_its_end_stop_in_24_s() {
            let json = include_str!("../../../../../packages/scenarios/scenarios/layouts/flow-t10.intracraft.json");
            within(statistic(json, 120.0, "gravity1", "meanTransitTime"), 24.0);
        }

        #[test]
        fn t11_the_gravity_line_delivers_60_pallets_an_hour() {
            let json = include_str!("../../../../../packages/scenarios/scenarios/layouts/demo-gravity-line.intracraft.json");
            within(statistic(json, 28_800.0, "sink1", "throughputPerHour"), 60.0);
        }

        #[test]
        fn lift_out_the_manual_output_table_delivers_55_34_pallets_an_hour() {
            let json = include_str!("../../../../../packages/scenarios/scenarios/layouts/demo-manual-output.intracraft.json");
            within(statistic(json, 28_800.0, "sink1", "throughputPerHour"), 55.34204458109147);
        }
    }

    /// IntraScript in the 2D run: the reference tests of S21 §11 (IS-1 to IS-3) and the run's guarantees.
    mod intrascript {
        use super::super::*;
        use crate::time::from_seconds;
        use intracraft_library::definition::Value;
        use intracraft_library::layout::{Instance, Project, Vec3};

        const ROLLER_LINE: &str = include_str!("../../../../../packages/scenarios/scenarios/layouts/demo-roller-line.intracraft.json");
        const SPLIT: &str = include_str!("../../../../../packages/scenarios/scenarios/layouts/demo-turntable-split.intracraft.json");
        const FLOW_T1: &str = include_str!("../../../../../packages/scenarios/scenarios/layouts/flow-t1.intracraft.json");

        fn instance(id: &str, definition: &str, name: &str, params: &[(&str, Value)]) -> Instance {
            Instance {
                id: id.into(),
                definition_id: definition.into(),
                definition_version: "0.1.0".into(),
                name: name.into(),
                position: Vec3::new(0.0, 0.0, 2.0),
                yaw: 0.0,
                params: params.iter().map(|(k, v)| (k.to_string(), v.clone())).collect(),
                attributes: Default::default(),
            }
        }

        /// A reference layout with a code block (and the photocell `Photoeye2`, 1.5 m into roller2, when asked).
        fn with_code(json: &str, code: &str, photocell: bool) -> (Layout, u64) {
            let project = Project::read(json).unwrap();
            let mut layout = project.layout;
            if photocell {
                let at = Value::Number(1.5);
                layout.instances.push(instance(
                    "pe2",
                    "sensor.photoeye",
                    "Photoeye2",
                    &[("host", Value::Text("roller2".into())), ("sensorS", at.clone()), ("reflectorS", at)],
                ));
            }
            layout.instances.push(instance("code1", "info.code", "Logic", &[("source", Value::Text(code.into()))]));
            (layout, project.seed)
        }

        fn run(layout: &Layout, seed: u64, seconds: f64) -> FlowSimulation {
            let lib = Library::built_in().unwrap();
            let mut sim = simulate(layout, &lib, seed).unwrap();
            sim.run_until(from_seconds(seconds)).unwrap();
            sim
        }

        fn variable(sim: &FlowSimulation, name: &str) -> String {
            sim.script_variables("code1").into_iter().find(|(n, _)| n == name).map(|(_, v)| v).unwrap_or_default()
        }

        #[test]
        fn is_1_code_counts_the_pallets_a_photocell_sees_and_a_sink_takes() {
            let code = "counted = 0\narrived = 0\n\n@on_rise(Photoeye2.detected)\ndef count():\n    global counted\n    counted += 1\n\n@on_enter(Outfeed_15_s)\ndef arrive(pallet):\n    global arrived\n    arrived += 1\n";
            let (layout, seed) = with_code(ROLLER_LINE, code, true);
            let mut sim = run(&layout, seed, 3600.0);
            let stats = sim.statistics();
            let pulses = stats["pe2"]["pulses"];
            assert!(pulses > 100.0, "{pulses}");
            assert_eq!(variable(&sim, "counted"), format!("{pulses}"));
            assert_eq!(variable(&sim, "arrived"), format!("{}", stats["sink1"]["palletsReceived"]));
            assert_eq!(stats["code1"]["activations"], pulses + stats["sink1"]["palletsReceived"]);
        }

        #[test]
        fn is_2_an_exit_decision_routes_by_the_attribute_given_at_creation() {
            let code = r#"
@on_create(Infeed)
def tag(pallet):
    pallet.destination = "B" if random.chance(0.3) else "A"

@choose_exit(Turntable)
def route(pallet):
    return Line_B if pallet.destination == "B" else Line_A

@on_enter(Outfeed_B)
def check(pallet):
    if pallet.destination != "B":
        error("pallet", pallet.id, "reached the wrong outfeed")
"#;
            let (layout, seed) = with_code(SPLIT, code, false);
            let counts = || {
                let mut sim = run(&layout, seed, 36_000.0);
                let s = sim.statistics();
                (s["sink1"]["palletsReceived"], s["sink2"]["palletsReceived"])
            };
            let (a, b) = counts();
            let share = b / (a + b);
            assert!(a + b > 2000.0, "{a} + {b}");
            assert!((share - 0.3).abs() < 0.03, "{share}");
            assert_eq!(counts(), (a, b), "the same seed gives the same routing");
        }

        #[test]
        fn is_3_a_two_second_stop_delays_the_pallet_by_exactly_two_seconds() {
            let code = "@on_enter(Roller_3_m)\ndef hold(pallet):\n    Roller_3_m.run = False\n    wait(2 * s)\n    Roller_3_m.run = True\n";
            let (layout, seed) = with_code(FLOW_T1, code, false);
            let mut sim = run(&layout, seed, 60.0);
            let transit = sim.statistics()["roller1"]["meanTransitTime"];
            assert!((transit - 14.25).abs() < 0.01 * 14.25, "{transit} s");
        }

        #[test]
        fn the_events_of_pallets_and_signals_arrive_in_their_order() {
            // Each pallet is created on the source, enters the roller, leaves it, and enters the sink; the photocell's
            // signal falls after it rises. Every handler records what it saw, so the order is checked by the run itself.
            let code = "log = []

@on_create(Source)
def made(pallet):
    log.append(f'created {pallet.id}')

@on_enter(Roller_3_m)
def in_roller(pallet):
    log.append(f'in {pallet.id}')

@on_leave(Roller_3_m)
def out_of_roller(pallet):
    log.append(f'out {pallet.id}')

@on_enter(Sink)
def arrived(pallet):
    log.append(f'arrived {pallet.id}')

@on_fall(Photoeye2.detected)
def cleared():
    log.append('cleared')

@on_rise(Photoeye2.detected)
def seen():
    log.append('seen')
";
            let (mut layout, seed) = with_code(FLOW_T1, code, false);
            let at = Value::Number(1.5);
            layout.instances.push(instance(
                "pe2",
                "sensor.photoeye",
                "Photoeye2",
                &[("host", Value::Text("roller1".into())), ("sensorS", at.clone()), ("reflectorS", at)],
            ));
            let sim = run(&layout, seed, 40.0);
            assert_eq!(variable(&sim, "log").matches("created 1").count(), 1);
            let log = variable(&sim, "log");
            let at = |t: &str| log.find(t).unwrap_or_else(|| panic!("no {t} in {log}"));
            assert!(at("created 1") < at("in 1"), "{log}");
            assert!(at("in 1") < at("out 1") && at("out 1") < at("arrived 1"), "{log}");
            assert!(at("seen") < at("cleared"), "{log}");
        }

        #[test]
        fn a_typo_keeps_the_run_from_starting_and_says_where() {
            let (layout, _) = with_code(FLOW_T1, "@on_start\ndef go():\n    Roler_3_m.run = False\n", false);
            let lib = Library::built_in().unwrap();
            let e = simulate(&layout, &lib, 1).err().unwrap();
            assert!(e.contains("Logic, line 3") && e.contains("Roller_3_m"), "{e}");
            // A disabled block is neither checked nor run.
            let mut off = layout.clone();
            off.instances.last_mut().unwrap().params.insert("enabled".into(), Value::Bool(false));
            assert!(simulate(&off, &lib, 1).is_ok());
        }

        #[test]
        fn a_loop_that_never_waits_stops_the_run_with_its_line() {
            let (layout, seed) = with_code(FLOW_T1, "@at(5 * s)\ndef spin():\n    while True:\n        pass\n", false);
            let lib = Library::built_in().unwrap();
            let mut sim = simulate(&layout, &lib, seed).unwrap();
            let e = sim.run_until(from_seconds(60.0)).unwrap_err();
            assert!(e.contains("stepLimit") && e.contains("0:00:05.000"), "{e}");
            assert_eq!(sim.script_error().map(|e| e.line), Some(3));
        }

        #[test]
        fn code_reads_parameters_with_units_prints_and_can_stop_the_run() {
            let code = "@on_start\ndef show():\n    print(Roller_3_m.length, Roller_3_m.nominal_speed, Roller_3_m.length / Roller_3_m.nominal_speed)\n\n@on_enter(Sink)\ndef done(pallet):\n    print(f'pallet {pallet.id} took {pallet.age:.2f}')\n    simulation.stop('one pallet is enough')\n";
            let (layout, seed) = with_code(FLOW_T1, code, false);
            let lib = Library::built_in().unwrap();
            let mut sim = simulate(&layout, &lib, seed).unwrap();
            sim.run_until(from_seconds(600.0)).unwrap();
            let printed: Vec<&str> = sim.script_console().iter().map(|l| l.text.as_str()).collect();
            assert_eq!(printed[0], "3000 mm 0.25 m/s 12 s");
            assert!(printed[1].starts_with("pallet 1 took 12."), "{}", printed[1]);
            assert_eq!(sim.stopped(), Some("one pallet is enough"));
            assert!(sim.now() < from_seconds(20.0));
        }
    }
}

/// Sections and addresses in the 2D run (ADR-0029): the layout's section gives its objects an address that code uses,
/// and a load unit's name and address follow the carrier it is on.
#[cfg(test)]
mod address_tests {
    use super::*;
    use crate::time::from_seconds;
    use intracraft_library::definition::Value as LibValue;
    use intracraft_library::layout::{Instance, Project, Vec3};

    const FLOW_T1: &str = include_str!("../../../../../packages/scenarios/scenarios/layouts/flow-t1.intracraft.json");

    /// FLOW_T1 with its roller in section `Seccion1`, and code that prints the roller's length, and each pallet's name and
    /// address when it enters the roller.
    fn run_in_section(seconds: f64) -> FlowSimulation {
        let lib = Library::built_in().unwrap();
        let mut project = Project::read(FLOW_T1).unwrap();
        for i in project.layout.instances.iter_mut().filter(|i| i.id == "roller1") {
            i.attributes.insert("section".into(), LibValue::Text("Seccion1".into()));
        }
        let code = "@on_start\ndef start():\n    print(Seccion1.Roller_3_m.length)\n\n@on_enter(Roller_3_m)\ndef entered(pallet):\n    print(pallet.name, pallet.address)\n";
        project.layout.instances.push(Instance {
            id: "code1".into(),
            definition_id: intracraft_script::CODE_BLOCK.into(),
            definition_version: "0.1.0".into(),
            name: "Logic".into(),
            position: Vec3::new(0.0, 0.0, 2.0),
            yaw: 0.0,
            params: [("source".to_string(), LibValue::Text(code.into()))].into_iter().collect(),
            attributes: Default::default(),
        });
        let mut sim = simulate(&project.layout, &lib, project.seed).unwrap();
        sim.run_until(from_seconds(seconds)).unwrap();
        sim
    }

    #[test]
    fn code_reads_an_object_by_its_address_and_a_load_unit_by_its_name_and_address() {
        let sim = run_in_section(20.0);
        let printed: Vec<&str> = sim.script_console().iter().map(|l| l.text.as_str()).collect();
        assert_eq!(printed[0], "3000 mm");
        assert_eq!(printed[1], "MU1 Seccion1.Roller_3_m.MU1");
    }

    #[test]
    fn an_object_outside_a_section_keeps_its_plain_name() {
        let lib = Library::built_in().unwrap();
        let project = Project::read(FLOW_T1).unwrap();
        let mut layout = project.layout;
        layout.instances.push(Instance {
            id: "code1".into(),
            definition_id: intracraft_script::CODE_BLOCK.into(),
            definition_version: "0.1.0".into(),
            name: "Logic".into(),
            position: Vec3::new(0.0, 0.0, 2.0),
            yaw: 0.0,
            params: [("source".to_string(), LibValue::Text("@on_start\ndef go():\n    print(Roller_3_m.length)\n".into()))]
                .into_iter()
                .collect(),
            attributes: Default::default(),
        });
        let mut sim = simulate(&layout, &lib, project.seed).unwrap();
        sim.run_until(from_seconds(1.0)).unwrap();
        assert_eq!(sim.script_console()[0].text, "3000 mm");
    }
}
