//! An instance's parameter values: the definition's defaults overridden by the instance's own values, and the
//! derived values computed from them. The `formula` text of a derived parameter documents it; the computation lives
//! here, keyed by the parameter's key, as in the reference.

use std::collections::BTreeMap;

use crate::definition::{Definition, Value};

/// Parameter values by key.
pub type Values = BTreeMap<String, Value>;

/// The definition's defaults, derived parameters left out.
pub fn defaults(definition: &Definition) -> Values {
    definition.parameters.iter().filter(|p| !p.derived).filter_map(|p| p.default.clone().map(|v| (p.key.clone(), v))).collect()
}

/// The defaults overridden by the instance's stored values.
pub fn instance_values(definition: &Definition, own: &Values) -> Values {
    let mut values = defaults(definition);
    values.extend(own.iter().map(|(k, v)| (k.clone(), v.clone())));
    values
}

/// The values of the definition's derived parameters that have a known computation.
pub fn derived(definition: &Definition, values: &Values) -> BTreeMap<String, f64> {
    definition.parameters.iter().filter(|p| p.derived).filter_map(|p| compute(&p.key, values).map(|v| (p.key.clone(), v))).collect()
}

fn compute(key: &str, p: &Values) -> Option<f64> {
    match key {
        "rollerCount" => Some((num(p, "length", num(p, "platformLength", 0.0)) / num(p, "rollerPitch", f64::INFINITY)).floor()),
        "sweptRadius" => Some(0.5 * num(p, "platformLength", 0.0).hypot(num(p, "platformWidth", 0.0)) + num(p, "sweepClearance", 0.0)),
        "entryHeight" => Some(num(p, "exitHeight", 0.0) + num(p, "slope", 0.0) * num(p, "length", 0.0)),
        _ => None,
    }
}

/// A number parameter's value, or `fallback` when it is missing or not a number.
pub fn num(values: &Values, key: &str, fallback: f64) -> f64 {
    values.get(key).and_then(Value::number).unwrap_or(fallback)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Library;

    fn library() -> Library {
        Library::built_in().unwrap()
    }

    #[test]
    fn derives_the_gravity_line_entry_height() {
        // 0.1 m + 4 % × 6 m = 0.34 m (S15).
        let lib = library();
        let gravity = lib.get("conveyor.gravity").unwrap();
        assert!((derived(gravity, &defaults(gravity))["entryHeight"] - 0.34).abs() < 1e-12);
    }

    #[test]
    fn counts_14_rollers_on_3_m_at_206_3_mm_and_22_at_133_mm() {
        let lib = library();
        let roller = lib.get("conveyor.roller").unwrap();
        assert_eq!(derived(roller, &defaults(roller)), BTreeMap::from([("rollerCount".to_string(), 14.0)]));
        let mut values = defaults(roller);
        values.insert("rollerPitch".into(), Value::Number(0.133));
        assert_eq!(derived(roller, &values)["rollerCount"], 22.0);
    }

    #[test]
    fn gives_the_turntable_a_swept_radius_of_972_mm() {
        let lib = library();
        let turntable = lib.get("conveyor.chain_turntable").unwrap();
        assert!((derived(turntable, &defaults(turntable))["sweptRadius"] - 0.972).abs() < 5e-4);
    }

    #[test]
    fn defaults_to_89_mm_rollers_at_206_3_mm_pitch_without_accumulation_and_leaves_derived_values_out() {
        let lib = library();
        let values = defaults(lib.get("conveyor.roller").unwrap());
        assert_eq!(values["rollerDiameter"], Value::Number(0.089));
        assert_eq!(values["rollerPitch"], Value::Number(0.2063));
        assert_eq!(values["accumulationMode"], Value::Text("none".into()));
        assert_eq!(values["length"], Value::Number(3.0));
        assert!(!values.contains_key("rollerCount"));
    }
}
