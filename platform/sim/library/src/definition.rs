//! What a library component is (S01–S17, A05): its parameters, ports, signals and presets, read from the same JSON
//! definitions as the TypeScript reference (`packages/library/src/definitions`). How it behaves lives in the engines,
//! keyed by `behaviors.des` and `behaviors.physics`. Stored values are SI (`unit`); the UI shows `displayUnit`.
//!
//! A definition is read kindly, as the reference's schema did (keys it doesn't know are left out), and then checked
//! strictly by [`Definition::check`].

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

/// A text in the languages the library is written in: English always, Spanish and German when translated.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Text {
    pub en: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub es: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub de: Option<String>,
}

impl Text {
    pub fn en(text: &str) -> Text {
        Text { en: text.to_string(), es: None, de: None }
    }
}

/// A parameter's value: a number (SI), a yes/no, or a text (an option of an enum, a free text).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Value {
    Bool(bool),
    Number(f64),
    Text(String),
}

impl Value {
    pub fn number(&self) -> Option<f64> {
        match self {
            Value::Number(n) => Some(*n),
            _ => None,
        }
    }

    pub fn text(&self) -> Option<&str> {
        match self {
            Value::Text(t) => Some(t),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ParamType {
    Number,
    Integer,
    Boolean,
    Enum,
    Text,
}

/// Which run a parameter changes.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Affects {
    #[serde(rename = "2d")]
    TwoD,
    #[serde(rename = "3d")]
    ThreeD,
    #[serde(rename = "both")]
    Both,
    #[serde(rename = "geometry")]
    Geometry,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Parameter {
    pub key: String,
    pub label: Text,
    #[serde(rename = "type")]
    pub kind: ParamType,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub unit: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub display_unit: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub default: Option<Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub min: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub step: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub options: Option<Vec<String>>,
    /// Options added when the registry is built: `catalogue`, `catalogue:<category>,…` or `materials`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub options_from: Option<String>,
    /// Text shown for an option instead of its id.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub option_labels: Option<BTreeMap<String, Text>>,
    /// Allowed values of a number parameter, e.g. standard roller diameters.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub choices: Option<Vec<f64>>,
    pub group: String,
    pub description: Text,
    #[serde(default)]
    pub advanced: bool,
    pub affects: Affects,
    #[serde(default)]
    pub derived: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub formula: Option<String>,
    /// `key=value` or `key!=value`: the parameter applies only then.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub depends_on: Option<String>,
}

/// One term of a port coordinate: a constant (m), or a factor times a parameter's value.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Term {
    Number(f64),
    Param {
        param: String,
        #[serde(default = "one")]
        factor: f64,
    },
}

fn one() -> f64 {
    1.0
}

/// A port coordinate: a number (m), a parameter's key, or terms added up.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Coordinate {
    Number(f64),
    Param(String),
    Terms(Vec<Term>),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Direction {
    In,
    Out,
    Inout,
}

/// Outward direction of a port in the component's own frame (x along travel, z across).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Facing {
    #[serde(rename = "+x")]
    PlusX,
    #[serde(rename = "-x")]
    MinusX,
    #[serde(rename = "+z")]
    PlusZ,
    #[serde(rename = "-z")]
    MinusZ,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PortLocal {
    pub x: Coordinate,
    pub y: Coordinate,
    pub z: Coordinate,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Port {
    pub id: String,
    pub direction: Direction,
    pub facing: Facing,
    pub local: PortLocal,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Signals {
    pub inputs: Vec<String>,
    pub outputs: Vec<String>,
}

/// The behaviours the two simulation modules register for the component (`des.*`, `phys.*`).
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Behaviors {
    pub des: String,
    pub physics: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Preset {
    pub name: Text,
    pub values: BTreeMap<String, Value>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Category {
    Conveyors,
    Sensors,
    Mechanics,
    Flow,
    Pallets,
    Control,
    Scenery,
    Storage,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Definition {
    pub id: String,
    pub version: String,
    pub category: Category,
    pub name: Text,
    pub description: Text,
    pub parameters: Vec<Parameter>,
    pub ports: Vec<Port>,
    pub signals: Signals,
    pub statistics: Vec<String>,
    pub geometry: String,
    pub behaviors: Behaviors,
    pub validation_rules: Vec<String>,
    pub presets: Vec<Preset>,
}

impl Definition {
    pub fn parameter(&self, key: &str) -> Option<&Parameter> {
        self.parameters.iter().find(|p| p.key == key)
    }

    /// What the reference's schema refused: an id `family.name`, a version `x.y.z`, English texts, a group for
    /// every parameter, and no key or port id twice.
    pub fn check(&self) -> Result<(), String> {
        let id_ok = self.id.split_once('.').is_some_and(|(family, name)| {
            !family.is_empty()
                && family.bytes().all(|b| b.is_ascii_lowercase())
                && !name.is_empty()
                && name.bytes().all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'_')
        });
        if !id_ok {
            return Err(format!("definition id {:?} is not of the form family.name", self.id));
        }
        let parts: Vec<&str> = self.version.split('.').collect();
        if parts.len() != 3 || parts.iter().any(|p| p.is_empty() || !p.bytes().all(|b| b.is_ascii_digit())) {
            return Err(format!("{}: version {:?} is not x.y.z", self.id, self.version));
        }
        if self.name.en.is_empty() || self.description.en.is_empty() {
            return Err(format!("{}: the name and description need an English text", self.id));
        }
        let mut keys = std::collections::BTreeSet::new();
        for p in &self.parameters {
            if p.key.is_empty() || p.group.is_empty() || p.label.en.is_empty() {
                return Err(format!("{}: parameter {:?} needs a key, a group and an English label", self.id, p.key));
            }
            if !keys.insert(p.key.as_str()) {
                return Err(format!("{}: parameter {:?} is declared twice", self.id, p.key));
            }
        }
        let mut ports = std::collections::BTreeSet::new();
        for port in &self.ports {
            if port.id.is_empty() || !ports.insert(port.id.as_str()) {
                return Err(format!("{}: port id {:?} is empty or declared twice", self.id, port.id));
            }
        }
        Ok(())
    }
}
