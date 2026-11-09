//! The example installations (`examples/layouts`) run in both modules: every sink of each receives pallets in ten minutes
//! of the discrete run, and the physics run on Avian carries pallets without dropping any.

use intracraft_avian::AvianBench;
use intracraft_physics::deps::intracraft_des::flow::{Statistics, simulate};
use intracraft_physics::deps::intracraft_des::time::from_seconds;
use intracraft_physics::deps::intracraft_library::Library;
use intracraft_physics::deps::intracraft_library::layout::Project;
use intracraft_physics::flow::PhysicsFlow;

const EXAMPLES: [(&str, &str); 6] = [
    ("roller-accumulation-line", include_str!("../../../../examples/layouts/roller-accumulation-line.intracraft.json")),
    ("roller-chain-transfer", include_str!("../../../../examples/layouts/roller-chain-transfer.intracraft.json")),
    ("turntable-distribution", include_str!("../../../../examples/layouts/turntable-distribution.intracraft.json")),
    ("chain-line-turntable", include_str!("../../../../examples/layouts/chain-line-turntable.intracraft.json")),
    ("mezzanine-elevator-gravity", include_str!("../../../../examples/layouts/mezzanine-elevator-gravity.intracraft.json")),
    ("mixed-plant", include_str!("../../../../examples/layouts/mixed-plant.intracraft.json")),
];

#[test]
fn every_example_feeds_every_sink_in_the_discrete_run_without_warnings() {
    let library = Library::built_in().unwrap();
    for (name, text) in EXAMPLES {
        let p = Project::read(text).unwrap_or_else(|e| panic!("{name}: {e}"));
        let mut sim = simulate(&p.layout, &library, p.seed).unwrap_or_else(|e| panic!("{name}: {e}"));
        assert!(sim.issues.is_empty(), "{name}: {:?}", sim.issues);
        sim.run_until(from_seconds(600.0)).unwrap_or_else(|e| panic!("{name}: {e}"));
        let stats = sim.statistics();
        for sink in p.layout.instances.iter().filter(|i| i.definition_id == "flow.sink") {
            assert!(received(&stats, &sink.id) >= 1.0, "{name}: {} received nothing: {stats:?}", sink.id);
        }
    }
}

#[test]
fn every_example_carries_pallets_on_avian_without_dropping_one() {
    let library = Library::built_in().unwrap();
    for (name, text) in EXAMPLES {
        let p = Project::read(text).unwrap();
        let mut flow = PhysicsFlow::on(Box::new(AvianBench::new()), &p.layout, &library, p.seed).unwrap_or_else(|e| panic!("{name}: {e}"));
        assert!(flow.issues.is_empty(), "{name}: {:?}", flow.issues);
        flow.run_until(from_seconds(120.0));
        let stats = flow.statistics();
        let delivered: f64 = p.layout.instances.iter().filter(|i| i.definition_id == "flow.sink").map(|s| received(&stats, &s.id)).sum();
        assert!(delivered >= 1.0, "{name}: no pallet delivered in 2 min: {stats:?}");
        assert_eq!(flow.fallen_pallets(), 0, "{name}");
    }
}

fn received(stats: &Statistics, id: &str) -> f64 {
    stats.get(id).and_then(|s| s.get("palletsReceived")).copied().unwrap_or(0.0)
}
