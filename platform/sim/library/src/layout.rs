//! A layout as the simulation reads it: placed instances (SI, y up, right-handed yaw about +Y), the connections
//! between their ports, and the controller's rules. It mirrors the TypeScript reference's project file
//! (`.intracraft.json`, schema version 1), so the reference's layouts and scenarios read as they are; in the Rust
//! platform `intracraft-model` builds it from the BIM model of the Design module.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::definition::Value;

/// The project file versions this build reads.
pub const SCHEMA_VERSION: u32 = 1;

#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Vec3 {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl Vec3 {
    pub const fn new(x: f64, y: f64, z: f64) -> Vec3 {
        Vec3 { x, y, z }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Instance {
    pub id: String,
    pub definition_id: String,
    #[serde(default)]
    pub definition_version: String,
    #[serde(default)]
    pub name: String,
    pub position: Vec3,
    #[serde(default)]
    pub yaw: f64,
    #[serde(default)]
    pub params: BTreeMap<String, Value>,
    /// Free key/value attributes the user attaches (zone = "A", line = 3).
    #[serde(default)]
    pub attributes: BTreeMap<String, Value>,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PortRef {
    pub instance_id: String,
    pub port_id: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Connection {
    pub from: PortRef,
    pub to: PortRef,
    #[serde(default = "default_gap")]
    pub gap: f64,
}

fn default_gap() -> f64 {
    crate::snap::PORT_GAP
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Edge {
    Rising,
    Falling,
}

/// A condition of a rule: a signal's level, or its change.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Condition {
    Is { signal: String, is: bool },
    Edge { signal: String, edge: Edge },
}

/// What a rule writes: a signal's value, now or after a delay.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Action {
    pub signal: String,
    pub value: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub delay_ms: Option<f64>,
}

/// An If/Then rule of the controller (`A08 Controller`): every condition holds → every action is written.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Rule {
    pub id: String,
    pub when: Vec<Condition>,
    pub then: Vec<Action>,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Layout {
    pub instances: Vec<Instance>,
    #[serde(default)]
    pub connections: Vec<Connection>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub rules: Vec<Rule>,
}

impl Layout {
    pub fn instance(&self, id: &str) -> Option<&Instance> {
        self.instances.iter().find(|i| i.id == id)
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Project {
    pub schema_version: u32,
    pub name: String,
    pub seed: u64,
    pub layout: Layout,
}

impl Project {
    /// Reads a reference project file: a newer schema is refused, saying so.
    pub fn read(json: &str) -> Result<Project, String> {
        let value: serde_json::Value = serde_json::from_str(json).map_err(|e| format!("not a project file: {e}"))?;
        let version = value.get("schemaVersion").and_then(serde_json::Value::as_u64).ok_or("the file has no schemaVersion")?;
        if version > u64::from(SCHEMA_VERSION) {
            return Err(format!("the file is schema version {version}; this build reads up to {SCHEMA_VERSION}"));
        }
        serde_json::from_value(value).map_err(|e| format!("project file: {e}"))
    }

    /// The project as the text of a reference project file.
    pub fn write(&self) -> Result<String, String> {
        serde_json::to_string_pretty(self)
            .map(|text| {
                text + "
"
            })
            .map_err(|e| format!("the project doesn't write: {e}"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const REFERENCE_ALL_FIVE: &str = include_str!("../../../../packages/scenarios/scenarios/layouts/reference-all-five.intracraft.json");

    #[test]
    fn reads_the_reference_layout_with_all_five_conveyors() {
        let project = Project::read(REFERENCE_ALL_FIVE).unwrap();
        assert_eq!(project.schema_version, 1);
        let ids: std::collections::BTreeSet<&str> = project.layout.instances.iter().map(|i| i.definition_id.as_str()).collect();
        for id in ["conveyor.roller", "conveyor.chain", "conveyor.chain_turntable", "conveyor.roller_turntable", "conveyor.transfer_rc"] {
            assert!(ids.contains(id), "{id}");
        }
    }

    #[test]
    fn refuses_a_newer_schema_and_a_file_that_is_not_a_project() {
        assert!(
            Project::read(r#"{"schemaVersion": 2, "name": "x", "seed": 1, "layout": {"instances": []}}"#)
                .unwrap_err()
                .contains("version 2")
        );
        assert!(Project::read("[1, 2]").is_err());
        assert!(Project::read("not json").is_err());
    }

    #[test]
    fn reads_both_kinds_of_rule_condition() {
        let rule: Rule = serde_json::from_str(
            r#"{"id": "r1", "when": [{"signal": "pe1.blocked", "is": true}, {"signal": "pe2.blocked", "edge": "rising"}],
                "then": [{"signal": "roller1.run", "value": false, "delayMs": 500}]}"#,
        )
        .unwrap();
        assert!(matches!(rule.when[0], Condition::Is { is: true, .. }));
        assert!(matches!(rule.when[1], Condition::Edge { edge: Edge::Rising, .. }));
        assert_eq!(rule.then[0].delay_ms, Some(500.0));
    }
}
