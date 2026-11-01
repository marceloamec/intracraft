//! A library definition as a BIMCraft family (A18 §3.1): its parameters, instance parameters with the library's
//! defaults, its presets as the family's types, its detailed parametric forms (`forms`), and a type parameter that names
//! the definition it came from. BIMCraft keeps lengths in millimetres and angles in degrees; the library keeps SI. The conversion lives
//! here, both ways, and nowhere else.

use intracraft_library::definition::{Definition, ParamType, Parameter, Value};
use serde_json::{Value as Json, json};

/// The type parameter of every intralogistics family: the id of the definition it came from (`conveyor.roller`).
pub const DEFINITION: &str = "IntraCraft definition";
/// The type every intralogistics family has: the library's defaults, nothing of its own.
pub const DEFAULT_TYPE: &str = "Default";

/// How a parameter's value is kept in the BIM model.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Unit {
    /// Metres in the library, millimetres in the model.
    Length,
    /// Radians in the library, degrees in the model.
    Angle,
    /// The same number in both (SI: m/s, s, kg…).
    Plain,
    YesNo,
    Text,
}

pub fn unit_of(p: &Parameter) -> Unit {
    match (p.kind, p.unit.as_deref()) {
        (ParamType::Boolean, _) => Unit::YesNo,
        (ParamType::Enum | ParamType::Text, _) => Unit::Text,
        (_, Some("m")) => Unit::Length,
        (_, Some("rad")) => Unit::Angle,
        _ => Unit::Plain,
    }
}

/// The family parameter's kind for a library parameter.
fn kind(p: &Parameter) -> &'static str {
    match unit_of(p) {
        Unit::Length => "length",
        Unit::Angle => "angle",
        Unit::Plain => "number",
        Unit::YesNo => "yesno",
        Unit::Text => "text",
    }
}

/// A library value (SI) as the model keeps it.
pub fn to_bim(p: &Parameter, v: &Value) -> Json {
    match (unit_of(p), v) {
        (Unit::Length, Value::Number(x)) => json!(x * 1000.0),
        (Unit::Angle, Value::Number(x)) => json!(x.to_degrees()),
        (_, Value::Number(x)) => json!(x),
        (_, Value::Bool(b)) => json!(b),
        (_, Value::Text(t)) => json!(t),
    }
}

/// A model value as the library keeps it (SI); `None` when it isn't what the parameter holds.
pub fn from_bim(p: &Parameter, v: &Json) -> Option<Value> {
    match unit_of(p) {
        Unit::Length => v.as_f64().map(|x| Value::Number(x / 1000.0)),
        Unit::Angle => v.as_f64().map(|x| Value::Number(x.to_radians())),
        Unit::Plain => v.as_f64().map(Value::Number),
        Unit::YesNo => v.as_bool().or_else(|| v.as_f64().map(|x| x != 0.0)).map(Value::Bool),
        Unit::Text => v.as_str().map(|t| Value::Text(t.to_string())),
    }
}

/// The formulas BIMCraft works the derived parameters out with, in the model's units (A05: the library's `formula`
/// text documents them; `intracraft_library::params` computes them in SI).
fn formula(definition: &Definition, key: &str) -> Option<&'static str> {
    match key {
        // A turntable counts the rollers of its platform.
        "rollerCount" if definition.parameter("length").is_none() => Some("rounddown(platformLength / rollerPitch)"),
        "rollerCount" => Some("rounddown(length / rollerPitch)"),
        "sweptRadius" => Some("0.5 * sqrt(platformLength ^ 2 + platformWidth ^ 2) + sweepClearance"),
        "entryHeight" => Some("exitHeight + slope * length"),
        _ => None,
    }
}

/// The name a definition's family has in the model.
pub fn family_name(definition: &Definition) -> String {
    definition.name.en.clone()
}

/// The `family` command's params that load a definition as a family (or bring an existing one up to date: `family`
/// replaces its parameters and forms and merges its types).
pub fn family_params(definition: &Definition) -> Json {
    let mut parameters = vec![json!({"name": DEFINITION, "kind": "text", "value": definition.id})];
    for p in &definition.parameters {
        let mut entry = json!({"name": p.key, "kind": kind(p), "instance": true});
        match (p.derived, formula(definition, &p.key)) {
            (true, Some(f)) => entry["formula"] = json!(f),
            // A derived value without a formula here is left to the library.
            (true, None) => continue,
            _ => {}
        }
        if let Some(v) = &p.default {
            entry["value"] = to_bim(p, v);
        }
        parameters.push(entry);
    }
    let mut types = serde_json::Map::new();
    types.insert(DEFAULT_TYPE.into(), json!({}));
    for preset in &definition.presets {
        let values: serde_json::Map<String, Json> = preset
            .values
            .iter()
            .filter_map(|(k, v)| definition.parameter(k).filter(|p| !p.derived).map(|p| (k.clone(), to_bim(p, v))))
            .collect();
        types.insert(preset.name.en.clone(), Json::Object(values));
    }
    json!({
        "name": family_name(definition),
        "category": "generic",
        "parameters": parameters,
        "types": types,
        "forms": crate::forms::forms(definition),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use intracraft_library::Library;

    #[test]
    fn converts_lengths_to_millimetres_and_angles_to_degrees_and_back() {
        let lib = Library::built_in().unwrap();
        let roller = lib.get("conveyor.roller").unwrap();
        let length = roller.parameter("length").unwrap();
        assert_eq!(to_bim(length, &Value::Number(3.0)), json!(3000.0));
        assert_eq!(from_bim(length, &json!(3000.0)), Some(Value::Number(3.0)));
        let turntable = lib.get("conveyor.chain_turntable").unwrap();
        let angle = turntable.parameters.iter().find(|p| unit_of(p) == Unit::Angle).unwrap();
        assert!((to_bim(angle, &Value::Number(std::f64::consts::FRAC_PI_2)).as_f64().unwrap() - 90.0).abs() < 1e-9);
        let speed = roller.parameter("nominalSpeed").unwrap();
        assert_eq!(to_bim(speed, &Value::Number(0.25)), json!(0.25));
        let mode = roller.parameter("accumulationMode").unwrap();
        assert_eq!(unit_of(mode), Unit::Text);
        assert_eq!(from_bim(mode, &json!(3)), None);
    }

    #[test]
    fn a_family_names_its_definition_and_takes_every_parameter_with_its_default() {
        let lib = Library::built_in().unwrap();
        let roller = lib.get("conveyor.roller").unwrap();
        let f = family_params(roller);
        assert_eq!(f["name"], json!("Roller conveyor"));
        let params = f["parameters"].as_array().unwrap();
        assert_eq!(params[0], json!({"name": DEFINITION, "kind": "text", "value": "conveyor.roller"}));
        let length = params.iter().find(|p| p["name"] == "length").unwrap();
        assert_eq!(
            (length["kind"].clone(), length["value"].clone(), length["instance"].clone()),
            (json!("length"), json!(3000.0), json!(true))
        );
        let count = params.iter().find(|p| p["name"] == "rollerCount").unwrap();
        assert_eq!(count["formula"], json!("rounddown(length / rollerPitch)"));
        assert!(f["types"][DEFAULT_TYPE].is_object());
        assert_eq!(f["types"]["Heavy 1000 kg, 0.25 m/s"].as_object().map(|t| !t.is_empty()), Some(true));
    }
}
