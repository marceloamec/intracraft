//! An object's attribute table (A18 §4): one row per parameter of its library definition, with its value in SI, where
//! the value comes from, and whether it is inherited (✔: the type or the library gives it, so a change there reaches
//! the object) or overridden (✖: the object holds its own value).

use bimcraft_core::family::Val;
use bimcraft_core::model::{Element, Id, Model};
use intracraft_library::Library;
use intracraft_library::definition::{Definition, ParamType, Text, Value};
use serde_json::Value as Json;

use crate::family::{DEFINITION, from_bim};

/// Where an attribute's value comes from.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Source {
    /// The object's own value.
    Instance,
    /// Its type's (a preset of the library).
    Type,
    /// The library definition's default.
    Library,
    /// Worked out from the others, read-only.
    Derived,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Attribute {
    pub key: String,
    pub label: Text,
    pub kind: ParamType,
    /// The SI unit the value is in, if any.
    pub unit: Option<String>,
    /// The unit a front end shows it in (mm, °), if the library names one.
    pub display_unit: Option<String>,
    pub value: Option<Value>,
    pub description: Text,
    pub source: Source,
    pub group: String,
    pub advanced: bool,
    /// The values a choice may take.
    pub options: Vec<String>,
}

impl Attribute {
    /// ✔ when the value is not the object's own.
    pub fn inherited(&self) -> bool {
        self.source != Source::Instance
    }
}

/// The definition a component of the model was placed from, if it is an intralogistics object.
pub fn definition_of<'l>(model: &Model, library: &'l Library, id: Id) -> Option<&'l Definition> {
    let Some(Element::Component(c)) = model.elements.get(id) else { return None };
    let family = model.families.get(&c.family)?;
    let def = family.parameters.iter().find(|p| p.name == DEFINITION)?;
    let id = family.types.get(&c.type_name).and_then(|t| t.get(DEFINITION)).unwrap_or(&def.value);
    library.get(id.as_str()?)
}

/// The attribute table of a placed object, in the definition's order; an error when it isn't an intralogistics object.
pub fn attributes(model: &Model, library: &Library, id: Id) -> Result<Vec<Attribute>, String> {
    let Some(Element::Component(c)) = model.elements.get(id) else { return Err(format!("{id} is not a placed object")) };
    let definition = definition_of(model, library, id).ok_or_else(|| format!("{id} is not an object of the IntraCraft library"))?;
    let family = model.families.get(&c.family).ok_or_else(|| format!("{id}: no family {}", c.family))?;
    let values = family.values(&c.type_name, &c.params).map_err(|e| format!("{id}: {e}"))?;
    let set = family.types.get(&c.type_name);
    Ok(definition
        .parameters
        .iter()
        .map(|p| {
            let source = if p.derived {
                Source::Derived
            } else if c.params.contains_key(&p.key) {
                Source::Instance
            } else if set.is_some_and(|t| t.contains_key(&p.key)) {
                Source::Type
            } else {
                Source::Library
            };
            let raw = values.get(&p.key).map(|v| match v {
                Val::Number(x) => Json::from(*x),
                Val::Text(t) => Json::from(t.as_str()),
            });
            Attribute {
                key: p.key.clone(),
                label: p.label.clone(),
                kind: p.kind,
                unit: p.unit.clone(),
                display_unit: p.display_unit.clone(),
                value: raw.as_ref().and_then(|v| from_bim(p, v)),
                description: p.description.clone(),
                source,
                group: p.group.clone(),
                advanced: p.advanced,
                options: p.options.clone().unwrap_or_default(),
            }
        })
        .collect())
}

/// How a value reads in a table: the library's display unit where it has one (mm, °), else SI.
pub fn shown(a: &Attribute) -> String {
    match (&a.value, a.display_unit.as_deref(), unit_of_attr(a)) {
        (None, _, _) => String::new(),
        (Some(Value::Bool(b)), _, _) => if *b { "yes" } else { "no" }.into(),
        (Some(Value::Text(t)), _, _) => t.clone(),
        (Some(Value::Number(x)), Some("mm"), _) => format!("{} mm", trim(x * 1000.0)),
        (Some(Value::Number(x)), Some("°" | "deg"), _) => format!("{} °", trim(x.to_degrees())),
        (Some(Value::Number(x)), Some("°/s"), _) => format!("{} °/s", trim(x.to_degrees())),
        (Some(Value::Number(x)), Some("°/s²"), _) => format!("{} °/s²", trim(x.to_degrees())),
        (Some(Value::Number(x)), _, Some(u)) => format!("{} {u}", trim(*x)),
        (Some(Value::Number(x)), _, None) => trim(*x),
    }
}

fn unit_of_attr(a: &Attribute) -> Option<&str> {
    a.unit.as_deref().filter(|u| !u.is_empty() && *u != "ratio")
}

/// A number with no more decimals than it needs (up to 4).
fn trim(x: f64) -> String {
    let s = format!("{x:.4}");
    let s = s.trim_end_matches('0').trim_end_matches('.');
    if s == "-0" { "0".into() } else { s.into() }
}

/// The parameter `key` of a definition, checked as the library says: not derived, of its type, inside its range, one of
/// its options or choices.
pub fn check_value(definition: &Definition, key: &str, value: &Value) -> Result<(), String> {
    let p = definition.parameter(key).ok_or_else(|| format!("{} has no attribute {key}", definition.id))?;
    if p.derived {
        return Err(format!("{key} is worked out from the others"));
    }
    match (p.kind, value) {
        (ParamType::Number | ParamType::Integer, Value::Number(x)) => {
            if !x.is_finite() {
                return Err(format!("{key}: {x} is not a number"));
            }
            if p.kind == ParamType::Integer && x.fract() != 0.0 {
                return Err(format!("{key} takes a whole number"));
            }
            if let Some(min) = p.min.filter(|m| x < m) {
                return Err(format!("{key}: {x} is below the minimum {min}"));
            }
            if let Some(max) = p.max.filter(|m| x > m) {
                return Err(format!("{key}: {x} is above the maximum {max}"));
            }
            if let Some(choices) = &p.choices
                && !choices.iter().any(|c| (c - x).abs() < 1e-12)
            {
                return Err(format!("{key}: {x} is not one of {choices:?}"));
            }
            Ok(())
        }
        (ParamType::Boolean, Value::Bool(_)) => Ok(()),
        (ParamType::Enum, Value::Text(t)) => {
            let options = p.options.as_deref().unwrap_or_default();
            if options.iter().any(|o| o == t) { Ok(()) } else { Err(format!("{key}: {t:?} is not one of {options:?}")) }
        }
        (ParamType::Text, Value::Text(_)) => Ok(()),
        _ => Err(format!("{key} does not hold {value:?}")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn attr(value: Value, unit: Option<&str>, display: Option<&str>) -> Attribute {
        Attribute {
            key: "k".into(),
            label: Text::en("k"),
            kind: ParamType::Number,
            unit: unit.map(Into::into),
            display_unit: display.map(Into::into),
            value: Some(value),
            description: Text::en("d"),
            source: Source::Library,
            group: "g".into(),
            advanced: false,
            options: vec![],
        }
    }

    #[test]
    fn shows_values_in_their_display_unit() {
        assert_eq!(shown(&attr(Value::Number(0.2063), Some("m"), Some("mm"))), "206.3 mm");
        assert_eq!(shown(&attr(Value::Number(0.25), Some("m/s"), None)), "0.25 m/s");
        assert_eq!(shown(&attr(Value::Number(std::f64::consts::FRAC_PI_2), Some("rad"), Some("°"))), "90 °");
        assert_eq!(shown(&attr(Value::Bool(true), None, None)), "yes");
        assert_eq!(shown(&attr(Value::Number(3.0), None, None)), "3");
    }

    #[test]
    fn checks_values_as_the_library_says() {
        let lib = Library::built_in().unwrap();
        let roller = lib.get("conveyor.roller").unwrap();
        assert!(check_value(roller, "length", &Value::Number(5.0)).is_ok());
        assert!(check_value(roller, "length", &Value::Number(-1.0)).unwrap_err().contains("minimum"));
        assert!(check_value(roller, "rollerDiameter", &Value::Number(0.05)).unwrap_err().contains("not one of"));
        assert!(check_value(roller, "accumulationMode", &Value::Text("none".into())).is_ok());
        assert!(check_value(roller, "accumulationMode", &Value::Text("lots".into())).is_err());
        assert!(check_value(roller, "rollerCount", &Value::Number(3.0)).unwrap_err().contains("worked out"));
        assert!(check_value(roller, "length", &Value::Text("long".into())).is_err());
        assert!(check_value(roller, "nope", &Value::Number(1.0)).is_err());
    }
}
