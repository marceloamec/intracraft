//! The 2D flow model: the network of a layout, and its run.

pub mod network;
pub mod sim;
pub(crate) mod station;

use intracraft_library::Library;
use intracraft_library::layout::Layout;
use intracraft_library::snap::derive_connections;

pub use network::{FlowIssue, FlowNetwork, build_flow_network};
pub use sim::{FlowSimulation, PalletView, Snapshot, Statistics};

/// The run of a layout as a project holds it (no connections stored: they come from the geometry, as in the editor).
pub fn simulate(layout: &Layout, library: &Library, seed: u64) -> Result<FlowSimulation, String> {
    let mut connected = layout.clone();
    connected.connections = derive_connections(layout, library)?;
    Ok(FlowSimulation::new(build_flow_network(&connected, library), seed, &library.reference))
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
}
