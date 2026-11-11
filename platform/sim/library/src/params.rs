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
        // A storage aisle (S26 §2).
        "rackLength" => Some(num(p, "bays", 0.0) * num(p, "bayPitch", 0.0)),
        "rackHeight" => Some(num(p, "firstLevelHeight", 0.0) + num(p, "levels", 0.0) * num(p, "levelPitch", 0.0)),
        "moduleWidth" => Some(num(p, "aisleWidth", 0.0) + 2.0 * deep(p) * num(p, "positionDepth", 0.0)),
        "positions" => Some(num(p, "bays", 0.0) * num(p, "levels", 0.0) * 2.0 * deep(p)),
        // A stretch wrapper (S25 §2).
        "advancePerTurn" => Some(advance_per_turn(p)),
        "carriageSpeed" => Some(advance_per_turn(p) * num(p, "turnSpeed", 0.0) / std::f64::consts::TAU),
        _ => None,
    }
}

/// How far a stretch wrapper's film carriage moves in one turn of the table, m.
fn advance_per_turn(p: &Values) -> f64 {
    num(p, "filmWidth", 0.0) * (1.0 - num(p, "overlap", 0.0))
}

/// Pallets deep on each side of a storage aisle: 1 or 2.
fn deep(p: &Values) -> f64 {
    if num(p, "deep", 1.0) >= 2.0 { 2.0 } else { 1.0 }
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
    fn derives_a_storage_aisle_s_size_and_positions_single_and_double_deep() {
        // 20 bays × 1.0 m, 0.6 m + 8 levels × 1.8 m, 1.5 m + 2 × 1.3 m, 20 × 8 × 2 positions (S26 §2).
        let lib = library();
        let aisle = lib.get("storage.aisle").unwrap();
        let mut values = defaults(aisle);
        let d = derived(aisle, &values);
        assert!((d["rackLength"] - 20.0).abs() < 1e-12 && (d["rackHeight"] - 15.0).abs() < 1e-12);
        assert!((d["moduleWidth"] - 4.1).abs() < 1e-12);
        assert_eq!(d["positions"], 320.0);
        values.insert("deep".into(), Value::Number(2.0));
        let d = derived(aisle, &values);
        assert!((d["moduleWidth"] - 6.7).abs() < 1e-12);
        assert_eq!(d["positions"], 640.0);
    }

    #[test]
    fn derives_a_stretch_wrapper_s_carriage_advance_and_speed() {
        // 500 mm of film at 50 % overlap: 250 mm a turn; at 12 rpm (0.2 turns/s): 0.05 m/s (S25 §2).
        let lib = library();
        let wrapper = lib.get("flow.stretch_wrapper").unwrap();
        let mut values = defaults(wrapper);
        let d = derived(wrapper, &values);
        assert!((d["advancePerTurn"] - 0.25).abs() < 1e-12);
        assert!((d["carriageSpeed"] - 0.05).abs() < 1e-12);
        values.insert("overlap".into(), Value::Number(0.2));
        assert!((derived(wrapper, &values)["advancePerTurn"] - 0.4).abs() < 1e-12);
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
