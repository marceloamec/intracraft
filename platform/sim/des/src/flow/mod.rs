//! The 2D flow model: the network of a layout, and its run.

pub mod network;
pub mod sim;

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
}
