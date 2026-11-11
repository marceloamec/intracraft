//! The automatic validation rules (MVP plan §6.7) a definition lists in `validationRules`, run with an instance's
//! values. Each rule returns issues and never fails. Until a line carries its pallet type, the rules are checked
//! against the EUR pallet, as in the reference.

use crate::definition::{Definition, Value};
use crate::params::{Values, instance_values, num};
use crate::reference::Reference;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Severity {
    Warning,
    Error,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Issue {
    pub rule: &'static str,
    pub severity: Severity,
    pub message: String,
}

pub const MIN_ROLLERS_UNDER_PALLET: f64 = 3.0;
/// Room beside a transfer chain between two rollers, m.
pub const CHAIN_GAP_MARGIN: f64 = 0.01;
/// Smallest height change of a lift, m; below it a roller conveyor does the job (S13, S14).
pub const MIN_HEIGHT_CHANGE: f64 = 0.01;
/// Clearance of a station deck beyond the pallet along travel, m.
pub const MIN_DECK_MARGIN: f64 = 0.1;
/// Clearance of the gravity line's pick-up position beyond the pallet, m (S15).
pub const PICKUP_MARGIN: f64 = 0.05;
/// Lateral clearance between pallet and guides, per side, m. Indicative.
pub const GUIDE_CLEARANCE: f64 = 0.025;
/// Clearance of a stretch wrapper's round table beyond the pallet's diagonal, m (S25).
pub const TABLE_MARGIN: f64 = 0.1;
/// Standard gravity, m/s².
pub const GRAVITY: f64 = 9.81;

fn issue(rule: &'static str, severity: Severity, message: String) -> Vec<Issue> {
    vec![Issue { rule, severity, message }]
}

/// At least 3 rollers under the supported pallet side: pitch ≤ side / 3.
pub fn min3_rollers_under_pallet(pitch: f64, pallet_support_length: f64) -> Vec<Issue> {
    if pitch <= pallet_support_length / MIN_ROLLERS_UNDER_PALLET {
        return vec![];
    }
    issue(
        "min3RollersUnderPallet",
        Severity::Error,
        format!("Roller pitch {pitch} m leaves fewer than 3 rollers under a {pallet_support_length} m pallet side"),
    )
}

/// Usable width ≥ pallet width + 2 × guide clearance.
pub fn usable_width_fits_pallet(usable_width: f64, pallet_width: f64, guide_clearance: f64) -> Vec<Issue> {
    let needed = pallet_width + 2.0 * guide_clearance;
    if usable_width >= needed {
        return vec![];
    }
    issue(
        "usableWidthFitsPallet",
        Severity::Error,
        format!("Usable width {usable_width} m is smaller than pallet + clearances ({needed} m)"),
    )
}

/// Transfer table: the gap between rollers (pitch − diameter) ≥ chain width + 10 mm.
pub fn chain_fits_between_rollers(pitch: f64, roller_diameter: f64, chain_width: f64) -> Vec<Issue> {
    let gap = pitch - roller_diameter;
    if gap >= chain_width + CHAIN_GAP_MARGIN {
        return vec![];
    }
    issue("chainFitsBetweenRollers", Severity::Error, format!("Roller gap {gap:.3} m is too small for a {chain_width} m chain"))
}

/// Manual lift table: the operator side must be lower than the conveyor side (S12).
pub fn heights_reversed(low_height: f64, conveyor_height: f64) -> Vec<Issue> {
    if low_height < conveyor_height {
        return vec![];
    }
    issue("heightsReversed", Severity::Error, format!("Low height {low_height} m is not below the conveyor height {conveyor_height} m"))
}

/// Scissor table or elevator whose entry and exit heights are practically equal (S13, S14).
pub fn no_height_change(entry_height: f64, exit_height: f64) -> Vec<Issue> {
    if (exit_height - entry_height).abs() >= MIN_HEIGHT_CHANGE {
        return vec![];
    }
    issue("noHeightChange", Severity::Warning, "Entry and exit heights differ by less than 10 mm; a roller conveyor is enough".into())
}

/// A station deck must hold the pallet with some clearance along travel.
pub fn deck_too_short(length: f64, pallet_along: f64) -> Vec<Issue> {
    if length >= pallet_along + MIN_DECK_MARGIN - 1e-9 {
        return vec![];
    }
    issue("deckTooShort", Severity::Warning, format!("Deck length {length} m is shorter than the pallet ({pallet_along} m) plus 100 mm"))
}

/// Gravity line: a pallet at rest starts rolling only when the slope beats the rolling resistance (S15).
pub fn slope_too_low(slope: f64, rolling_resistance: f64) -> Vec<Issue> {
    if slope > rolling_resistance {
        return vec![];
    }
    issue(
        "slopeTooLow",
        Severity::Error,
        format!("Slope {slope} is not above the rolling resistance {rolling_resistance}: a pallet at rest does not start"),
    )
}

/// Gravity line: brake rollers so far apart that a pallet can have none under it (S15).
pub fn brake_spacing_too_long(brake_every: f64, pitch: f64, pallet_along: f64) -> Vec<Issue> {
    let spacing = brake_every * pitch;
    if spacing <= pallet_along + 1e-9 {
        return vec![];
    }
    issue(
        "brakeSpacingTooLong",
        Severity::Warning,
        format!("Brake rollers {spacing:.3} m apart: a {pallet_along} m pallet can have no brake under it"),
    )
}

/// Gravity line: the pick-up position must hold a pallet with some clearance (S15).
pub fn pickup_too_short(pickup_length: f64, pallet_along: f64) -> Vec<Issue> {
    if pickup_length >= pallet_along + PICKUP_MARGIN - 1e-9 {
        return vec![];
    }
    issue(
        "pickupTooShort",
        Severity::Error,
        format!("Pick-up length {pickup_length} m is shorter than the pallet ({pallet_along} m) plus 50 mm"),
    )
}

/// Storage aisle: a station height in use above the top level, where the crane's forks don't go (S26 §6).
pub fn station_above_rack(station_heights: &[f64], top_level: f64) -> Vec<Issue> {
    let Some(high) = station_heights.iter().copied().find(|h| *h > top_level + 1e-9) else { return vec![] };
    issue("stationAboveRack", Severity::Error, format!("Station height {high} m is above the top level at {top_level} m"))
}

/// Stretch wrapper: the round table must hold the pallet's corners with some clearance, or they sweep past it (S25 §6).
pub fn table_too_small(diameter: f64, pallet_diagonal: f64) -> Vec<Issue> {
    let needed = pallet_diagonal + TABLE_MARGIN;
    if diameter >= needed - 1e-9 {
        return vec![];
    }
    issue(
        "tableTooSmall",
        Severity::Error,
        format!("Table diameter {diameter} m is smaller than the pallet's diagonal plus 100 mm ({needed:.3} m)"),
    )
}

/// Stretch wrapper: the acceleration at the pallet's corner while the table turns (centripetal and tangential together)
/// is more than friction inside the load can hold (S25 §6). Speeds in rad/s and rad/s².
pub fn load_may_shift(turn_speed: f64, turn_accel: f64, corner_radius: f64, friction: f64) -> Vec<Issue> {
    let at_corner = (turn_speed * turn_speed * corner_radius).hypot(turn_accel * corner_radius);
    let held = friction * GRAVITY;
    if at_corner <= held + 1e-9 {
        return vec![];
    }
    issue(
        "loadMayShift",
        Severity::Warning,
        format!(
            "The pallet's corner accelerates at {at_corner:.2} m/s², more than friction {friction} holds ({held:.2} m/s²): a loose load may shift"
        ),
    )
}

/// The issues of an instance: the rules its definition lists, with its values over the defaults.
pub fn validate_instance(definition: &Definition, own: &Values, reference: &Reference) -> Vec<Issue> {
    let Some(eur) = reference.pallet_type("eur") else { return vec![] };
    let p = instance_values(definition, own);
    let long_along = p.get("palletSideAlongTravel") != Some(&Value::Text("short".into()));
    let (along, across) = if long_along { (eur.length, eur.width) } else { (eur.width, eur.length) };
    definition
        .validation_rules
        .iter()
        .flat_map(|rule| match rule.as_str() {
            // A chain deck (elevator, S14) has no rollers to count.
            "min3RollersUnderPallet" if p.get("deckType") == Some(&Value::Text("chains".into())) => vec![],
            "min3RollersUnderPallet" => min3_rollers_under_pallet(num(&p, "rollerPitch", 0.0), along),
            "usableWidthFitsPallet" => {
                usable_width_fits_pallet(num(&p, "usableWidth", num(&p, "platformWidth", 0.0)), across, GUIDE_CLEARANCE)
            }
            "chainFitsBetweenRollers" => {
                chain_fits_between_rollers(num(&p, "rollerPitch", 0.0), num(&p, "rollerDiameter", 0.0), num(&p, "transferChainWidth", 0.0))
            }
            "heightsReversed" => heights_reversed(num(&p, "lowHeight", 0.0), num(&p, "conveyorHeight", 0.0)),
            "noHeightChange" => no_height_change(num(&p, "entryHeight", 0.0), num(&p, "exitHeight", 0.0)),
            "deckTooShort" => deck_too_short(num(&p, "length", 0.0), along),
            "slopeTooLow" => slope_too_low(num(&p, "slope", 0.0), num(&p, "rollingResistance", 0.0)),
            "brakeSpacingTooLong" => brake_spacing_too_long(num(&p, "brakeEvery", 1.0), num(&p, "rollerPitch", 0.0), along),
            "pickupTooShort" => pickup_too_short(num(&p, "pickupLength", 0.0), along),
            "stationAboveRack" => {
                let used = num(&p, "pdLevels", 1.0).clamp(1.0, 3.0) as usize;
                let heights: Vec<f64> = (1..=used).map(|n| num(&p, &format!("pdHeight{n}"), 0.0)).collect();
                let top = num(&p, "firstLevelHeight", 0.0) + (num(&p, "levels", 1.0) - 1.0) * num(&p, "levelPitch", 0.0);
                station_above_rack(&heights, top)
            }
            "tableTooSmall" => table_too_small(num(&p, "length", 0.0), eur.length.hypot(eur.width)),
            "loadMayShift" => load_may_shift(
                num(&p, "turnSpeed", 0.0),
                num(&p, "turnAccel", 0.0),
                eur.length.hypot(eur.width) / 2.0,
                num(&p, "loadFriction", 0.0),
            ),
            _ => vec![],
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Library;

    fn rules_of(lib: &Library, id: &str, values: &[(&str, Value)]) -> Vec<&'static str> {
        let own: Values = values.iter().map(|(k, v)| (k.to_string(), v.clone())).collect();
        validate_instance(lib.get(id).unwrap(), &own, &lib.reference).iter().map(|i| i.rule).collect()
    }

    fn n(v: f64) -> Value {
        Value::Number(v)
    }

    #[test]
    fn flags_a_chain_conveyor_too_narrow_for_a_transverse_eur_pallet() {
        let lib = Library::built_in().unwrap();
        assert_eq!(rules_of(&lib, "conveyor.chain", &[("usableWidth", n(1.0))]), ["usableWidthFitsPallet"]);
    }

    #[test]
    fn flags_transfer_chains_that_do_not_fit_between_the_rollers() {
        let lib = Library::built_in().unwrap();
        let values = [("rollerPitch", n(0.133)), ("rollerDiameter", n(0.089)), ("transferChainWidth", n(0.08))];
        assert_eq!(rules_of(&lib, "conveyor.transfer_rc", &values), ["chainFitsBetweenRollers"]);
    }

    #[test]
    fn flags_a_manual_lift_table_with_reversed_heights() {
        let lib = Library::built_in().unwrap();
        assert_eq!(rules_of(&lib, "conveyor.lift_table_manual", &[("lowHeight", n(0.4)), ("conveyorHeight", n(0.4))]), ["heightsReversed"]);
    }

    #[test]
    fn flags_a_scissor_table_or_elevator_that_does_not_change_height() {
        let lib = Library::built_in().unwrap();
        for id in ["conveyor.scissor_lift", "conveyor.elevator"] {
            assert_eq!(rules_of(&lib, id, &[("entryHeight", n(0.6)), ("exitHeight", n(0.605))]), ["noHeightChange"], "{id}");
        }
    }

    #[test]
    fn flags_a_lift_deck_shorter_than_the_pallet_plus_100_mm() {
        let lib = Library::built_in().unwrap();
        assert_eq!(rules_of(&lib, "conveyor.scissor_lift", &[("length", n(1.25))]), ["deckTooShort"]);
        assert!(rules_of(&lib, "conveyor.scissor_lift", &[("length", n(1.3))]).is_empty());
    }

    #[test]
    fn counts_rollers_under_the_pallet_only_on_an_elevator_with_a_roller_deck() {
        let lib = Library::built_in().unwrap();
        assert_eq!(rules_of(&lib, "conveyor.elevator", &[("rollerPitch", n(0.5))]), ["min3RollersUnderPallet"]);
        let chains = [
            ("rollerPitch", n(0.5)),
            ("deckType", Value::Text("chains".into())),
            ("palletSideAlongTravel", Value::Text("short".into())),
            ("usableWidth", n(1.3)),
        ];
        assert!(rules_of(&lib, "conveyor.elevator", &chains).is_empty());
    }

    #[test]
    fn checks_the_gravity_line_slope_brakes_and_pick_up() {
        let lib = Library::built_in().unwrap();
        assert_eq!(rules_of(&lib, "conveyor.gravity", &[("slope", n(0.02)), ("rollingResistance", n(0.02))]), ["slopeTooLow"]);
        // 6 × 206.3 mm = 1.238 m > 1.2 m.
        assert_eq!(rules_of(&lib, "conveyor.gravity", &[("brakeEvery", n(6.0))]), ["brakeSpacingTooLong"]);
        assert!(rules_of(&lib, "conveyor.gravity", &[("brakeEvery", n(5.0))]).is_empty());
        assert_eq!(rules_of(&lib, "conveyor.gravity", &[("pickupLength", n(1.24))]), ["pickupTooShort"]);
    }

    #[test]
    fn flags_a_storage_aisle_station_above_its_top_level() {
        let lib = Library::built_in().unwrap();
        // Top level at 0.6 + 7 × 1.8 = 13.2 m.
        let two = [("pdLevels", n(2.0)), ("pdHeight2", n(14.0))];
        assert_eq!(rules_of(&lib, "storage.aisle", &two), ["stationAboveRack"]);
        assert!(rules_of(&lib, "storage.aisle", &[("pdLevels", n(2.0)), ("pdHeight2", n(13.2))]).is_empty());
        // A height that is not in use is not checked.
        assert!(rules_of(&lib, "storage.aisle", &[("pdHeight2", n(14.0))]).is_empty());
    }

    #[test]
    fn flags_a_stretch_wrapper_table_smaller_than_the_pallet_s_diagonal_plus_100_mm() {
        let lib = Library::built_in().unwrap();
        // The EUR pallet's diagonal: √(1.2² + 0.8²) = 1.442 m.
        assert_eq!(rules_of(&lib, "flow.stretch_wrapper", &[("length", n(1.5))]), ["tableTooSmall"]);
        assert!(rules_of(&lib, "flow.stretch_wrapper", &[("length", n(1.55))]).is_empty());
    }

    #[test]
    fn warns_that_a_load_may_shift_on_a_stretch_wrapper_at_20_rpm_and_not_at_12() {
        let lib = Library::built_in().unwrap();
        let rpm = |r: f64| n(r * 2.0 * std::f64::consts::PI / 60.0);
        // W4 of S25: at the corner, 0.721 m out, 20 rpm gives 3.19 m/s² against 0.3 × 9.81 = 2.94 m/s²; 12 rpm, 1.23 m/s².
        assert_eq!(rules_of(&lib, "flow.stretch_wrapper", &[("turnSpeed", rpm(20.0))]), ["loadMayShift"]);
        assert!(rules_of(&lib, "flow.stretch_wrapper", &[("turnSpeed", rpm(12.0))]).is_empty());
        // Cartons on cartons (0.4) hold at 20 rpm.
        assert!(rules_of(&lib, "flow.stretch_wrapper", &[("turnSpeed", rpm(20.0)), ("loadFriction", n(0.4))]).is_empty());
    }

    #[test]
    fn the_rules_themselves() {
        assert!(min3_rollers_under_pallet(0.2063, 1.2).is_empty());
        assert_eq!(min3_rollers_under_pallet(0.25, 0.7).len(), 1);
        assert!(usable_width_fits_pallet(1.0, 0.8, 0.025).is_empty());
        assert_eq!(usable_width_fits_pallet(0.82, 0.8, 0.025).len(), 1);
        // A 40 mm chain between Ø89 rollers at 206.3 mm pitch: a 117.3 mm gap.
        assert!(chain_fits_between_rollers(0.2063, 0.089, 0.04).is_empty());
        assert!(chain_fits_between_rollers(0.133, 0.076, 0.04).is_empty());
        assert_eq!(chain_fits_between_rollers(0.1, 0.076, 0.04).len(), 1);
    }
}
