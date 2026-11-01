//! The reference scenarios (A10, A11), read from the very files of the TypeScript reference
//! (`packages/scenarios/scenarios/*.json` and its layouts) and run on the Rust engines: the analytic ones on the
//! kinematics, the layout ones on the 2D flow model. The physics ones (P1–P11) run on the physics bench in phase R5; until
//! then they are listed as not run.

use intracraft_des::kinematics::{MoveProfile, impact_energy, max_throughput, profile_time, transfer_cycle_time};
use intracraft_des::time::from_seconds;
use intracraft_library::Library;
use intracraft_library::definition::Text;
use intracraft_library::layout::Project;
use serde::Deserialize;
use serde_json::Value;

const ANALYTIC: &str = include_str!("../../../../packages/scenarios/scenarios/analytic.json");
const LAYOUTS: &str = include_str!("../../../../packages/scenarios/scenarios/layouts.json");
const PHYSICS: &str = include_str!("../../../../packages/scenarios/scenarios/physics.json");

/// The layout files the scenarios name, by name.
const LAYOUT_FILES: [(&str, &str); 10] = [
    ("demo-elevator-c", include_str!("../../../../packages/scenarios/scenarios/layouts/demo-elevator-c.intracraft.json")),
    ("demo-gravity-line", include_str!("../../../../packages/scenarios/scenarios/layouts/demo-gravity-line.intracraft.json")),
    ("demo-manual-output", include_str!("../../../../packages/scenarios/scenarios/layouts/demo-manual-output.intracraft.json")),
    ("demo-process-line", include_str!("../../../../packages/scenarios/scenarios/layouts/demo-process-line.intracraft.json")),
    ("demo-roller-line", include_str!("../../../../packages/scenarios/scenarios/layouts/demo-roller-line.intracraft.json")),
    ("demo-turntable-split", include_str!("../../../../packages/scenarios/scenarios/layouts/demo-turntable-split.intracraft.json")),
    ("flow-t1", include_str!("../../../../packages/scenarios/scenarios/layouts/flow-t1.intracraft.json")),
    ("flow-t10", include_str!("../../../../packages/scenarios/scenarios/layouts/flow-t10.intracraft.json")),
    ("flow-t2", include_str!("../../../../packages/scenarios/scenarios/layouts/flow-t2.intracraft.json")),
    ("reference-all-five", include_str!("../../../../packages/scenarios/scenarios/layouts/reference-all-five.intracraft.json")),
];

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Scenario {
    pub id: String,
    pub title: Text,
    pub kind: String,
    pub inputs: Value,
    pub unit: String,
    pub expected: f64,
    #[serde(default)]
    pub relative_tolerance: Option<f64>,
    #[serde(default)]
    pub absolute_tolerance: Option<f64>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Outcome {
    /// Run: the value, and whether it is within the tolerance.
    Ran { actual: f64, passed: bool },
    /// Not run by the Rust engines yet, and why.
    NotRun(String),
    /// It failed to run, and why.
    Error(String),
}

/// Every scenario, in the reference's order: analytic, layouts, physics.
pub fn scenarios() -> Result<Vec<Scenario>, String> {
    let mut out = Vec::new();
    for (name, text) in [("analytic", ANALYTIC), ("layouts", LAYOUTS), ("physics", PHYSICS)] {
        let list: Vec<Scenario> = serde_json::from_str(text).map_err(|e| format!("{name}.json: {e}"))?;
        out.extend(list);
    }
    Ok(out)
}

pub fn find(id: &str) -> Result<Option<Scenario>, String> {
    Ok(scenarios()?.into_iter().find(|s| s.id.eq_ignore_ascii_case(id)))
}

/// Runs a scenario.
pub fn run(s: &Scenario, library: &Library) -> Outcome {
    match evaluate(s, library) {
        Ok(Some(actual)) => Outcome::Ran { actual, passed: passed(s, actual) },
        Ok(None) => Outcome::NotRun("physics scenarios run on the Rapier bench in phase R5".into()),
        Err(e) => Outcome::Error(e),
    }
}

fn passed(s: &Scenario, actual: f64) -> bool {
    let error = actual - s.expected;
    match (s.absolute_tolerance, s.relative_tolerance) {
        (Some(abs), _) => error.abs() <= abs,
        (None, Some(rel)) => (error / s.expected).abs() < rel,
        (None, None) => false,
    }
}

fn evaluate(s: &Scenario, library: &Library) -> Result<Option<f64>, String> {
    let i = &s.inputs;
    let n = |k: &str| i.get(k).and_then(Value::as_f64).ok_or_else(|| format!("{}: input {k} is missing", s.id));
    Ok(Some(match s.kind.as_str() {
        "profileTime" => profile_time(&MoveProfile {
            distance: n("distance")?,
            max_speed: n("maxSpeed")?,
            accel: n("accel")?,
            decel: i.get("decel").and_then(Value::as_f64),
        })?,
        "throughput" => max_throughput(n("speed")?, n("palletLength")?, n("gap")?),
        "transferCycle" => transfer_cycle_time(n("liftTime")?, n("lowerTime")?, n("distance")?, n("speed")?, n("accel")?, 0.0)?,
        "impactEnergy" => impact_energy(n("mass")?, n("speed")?),
        "flow" => {
            let text = |k: &str| i.get(k).and_then(Value::as_str).ok_or_else(|| format!("{}: input {k} is missing", s.id));
            let (layout, instance, statistic) = (text("layout")?, text("instanceId")?, text("statistic")?);
            let json =
                LAYOUT_FILES.iter().find(|(name, _)| *name == layout).map(|(_, j)| *j).ok_or_else(|| format!("no layout {layout}"))?;
            let project = Project::read(json)?;
            let mut sim = intracraft_des::flow::simulate(&project.layout, library, project.seed)?;
            sim.run_until(from_seconds(n("seconds")?))?;
            let stats = sim.statistics();
            *stats
                .get(instance)
                .and_then(|m| m.get(statistic))
                .ok_or_else(|| format!("layout {layout} has no statistic {instance}.{statistic}"))?
        }
        "physics" => return Ok(None),
        other => return Err(format!("{}: unknown kind {other}", s.id)),
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_analytic_and_layout_scenario_passes_on_the_rust_engines() {
        let lib = Library::built_in().unwrap();
        let all = scenarios().unwrap();
        assert_eq!(all.iter().filter(|s| s.kind != "physics").count(), 15);
        for s in all.iter().filter(|s| s.kind != "physics") {
            match run(s, &lib) {
                Outcome::Ran { actual, passed } => assert!(passed, "{} gives {actual}, expected {} {}", s.id, s.expected, s.unit),
                other => panic!("{}: {other:?}", s.id),
            }
        }
    }

    #[test]
    fn physics_scenarios_are_listed_as_not_run_and_ids_are_found_in_any_case() {
        let lib = Library::built_in().unwrap();
        let p1 = find("p1").unwrap().unwrap();
        assert!(matches!(run(&p1, &lib), Outcome::NotRun(_)));
        assert!(find("T1-2d").unwrap().is_some());
        assert!(find("nope").unwrap().is_none());
    }
}
