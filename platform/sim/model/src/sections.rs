//! Sections and addresses (ADR-0029). A section is a group of objects, like a layer: a project parameter `Section` of
//! components, normally on one level but free to span levels. Every object has an address, `Section.Object` (or `Object`
//! alone when it has no section), and the load units a conveyor carries have addresses below it: `Section.Conveyor.MU3`.
//! The address of a load unit changes as it moves; its number (`MU3`) does not.

use std::collections::BTreeMap;

use bimcraft_core::engine::Session;
use bimcraft_core::model::{Element, Id, Model};
use intracraft_library::definition::Value;
use serde_json::{Value as Json, json};

/// The project parameter that holds an object's section (a text of components).
pub const SECTION: &str = "Section";
/// The separator of an address.
pub const SEPARATOR: char = '.';

/// Makes the project parameter `Section` of components, when the project lacks it.
pub fn section_parameter(session: &mut Session) -> Result<(), String> {
    if !session.model.parameters.contains_key(SECTION) {
        session.execute("parameter", &json!({"name": SECTION, "kind": "text", "categories": ["component"]}))?;
    }
    Ok(())
}

/// Puts object `id` in section `section` (empty: in none). A section name is text without `.` at its ends or inside.
/// One undo step.
pub fn set_section(session: &mut Session, id: Id, section: &str) -> Result<(), String> {
    let section = section.trim();
    if section.contains(SEPARATOR) {
        return Err(format!("a section's name can't hold `{SEPARATOR}`, which separates the parts of an address: {section:?}"));
    }
    section_parameter(session)?;
    let value = if section.is_empty() { Json::Null } else { json!(section) };
    session.execute("modify", &json!({"id": id, "params": {SECTION: value}})).map(|_| ())
}

/// The section object `id` is in, if any.
pub fn section_of(model: &Model, id: Id) -> Option<String> {
    let Element::Component(c) = model.elements.get(id)? else { return None };
    c.params.get(SECTION).and_then(Json::as_str).map(str::trim).filter(|s| !s.is_empty()).map(str::to_string)
}

/// Every section the components of the model are in, once each, in name order.
pub fn sections(model: &Model) -> Vec<String> {
    let mut names: Vec<String> = model.elements.iter().filter_map(|(&id, _)| section_of(model, id)).collect();
    names.sort();
    names.dedup();
    names
}

/// The address of an object called `name` in section `section` (`Seccion1.Conveyor1`, or `Conveyor1` with no section).
pub fn address(section: Option<&str>, name: &str) -> String {
    match section {
        Some(s) if !s.is_empty() => format!("{s}{SEPARATOR}{name}"),
        _ => name.to_string(),
    }
}

/// The address of a load unit: its carrier's address and its own name, `MU3`. Without a carrier it is `MU3` alone.
pub fn load_unit_address(carrier: Option<&str>, unit: &str) -> String {
    address(carrier, unit)
}

/// The name a load unit gets from its pallet id: `MU` and the number, unique in the run (ADR-0029).
pub fn load_unit_name(pallet_id: u64) -> String {
    format!("MU{pallet_id}")
}

/// The names of the objects of a section that repeat (the addresses that would clash), each once.
pub fn duplicate_names(entries: &[(Option<String>, String)]) -> Vec<String> {
    let mut seen: Vec<String> = Vec::new();
    let mut out: Vec<String> = Vec::new();
    for (section, name) in entries {
        let addr = address(section.as_deref(), name);
        if seen.contains(&addr) {
            if !out.contains(&addr) {
                out.push(addr);
            }
        } else {
            seen.push(addr);
        }
    }
    out
}

/// The name an object has in the layout: its project name, else its family and id (as `layout` names it).
pub fn display_name(model: &Model, id: Id) -> String {
    crate::name_of(model, id).unwrap_or_else(|| match model.elements.get(id) {
        Some(Element::Component(c)) => format!("{} {id}", c.family),
        _ => id.to_string(),
    })
}

/// The address of object `id` in the model, from its section and display name.
pub fn object_address(model: &Model, id: Id) -> String {
    address(section_of(model, id).as_deref(), &display_name(model, id))
}

/// The free attributes an instance of the layout carries for its section and address (`section`, `address`), so the
/// simulations and IntraScript read them as the layout gives them.
pub fn attributes_of(model: &Model, id: Id) -> BTreeMap<String, Value> {
    let mut out = BTreeMap::new();
    if let Some(section) = section_of(model, id) {
        out.insert("section".to_string(), Value::Text(section));
    }
    out.insert("address".to_string(), Value::Text(object_address(model, id)));
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_address_joins_the_section_and_the_name() {
        assert_eq!(address(Some("Seccion1"), "Conveyor1"), "Seccion1.Conveyor1");
        assert_eq!(address(None, "Conveyor1"), "Conveyor1");
        assert_eq!(address(Some(""), "Conveyor1"), "Conveyor1");
    }

    #[test]
    fn a_load_unit_sits_under_its_carrier_and_keeps_its_number() {
        assert_eq!(load_unit_name(3), "MU3");
        assert_eq!(load_unit_address(Some("Seccion1.Conveyor1"), &load_unit_name(3)), "Seccion1.Conveyor1.MU3");
        assert_eq!(load_unit_address(None, &load_unit_name(3)), "MU3");
    }

    #[test]
    fn two_objects_at_one_address_are_reported_by_the_layout() {
        let library = intracraft_library::Library::built_in().unwrap();
        let mut s = Session::new();
        crate::load_library(&mut s, &library).unwrap();
        let a = crate::place(&mut s, &library, "conveyor.roller", None, [0.0, 0.0], 0.0, 0.0).unwrap();
        let b = crate::place(&mut s, &library, "conveyor.roller", None, [4000.0, 0.0], 0.0, 0.0).unwrap();
        crate::set_name(&mut s, a, "Line A").unwrap();
        crate::set_name(&mut s, b, "Line A").unwrap();
        set_section(&mut s, a, "Seccion1").unwrap();
        set_section(&mut s, b, "Seccion1").unwrap();
        let (_, problems) = crate::layout::layout(&s.model, &library);
        assert!(problems.iter().any(|p| p.contains("Seccion1.Line A")), "{problems:?}");
        // The same name in another section is fine.
        set_section(&mut s, b, "Seccion2").unwrap();
        let (_, problems) = crate::layout::layout(&s.model, &library);
        assert!(!problems.iter().any(|p| p.contains("two objects")), "{problems:?}");
    }

    #[test]
    fn repeated_addresses_are_found_once() {
        let entries = [
            (Some("S1".to_string()), "Roller".to_string()),
            (Some("S1".to_string()), "Roller".to_string()),
            (Some("S2".to_string()), "Roller".to_string()),
            (None, "Roller".to_string()),
            (None, "Roller".to_string()),
        ];
        assert_eq!(duplicate_names(&entries), ["S1.Roller", "Roller"]);
    }

    #[test]
    fn a_section_name_cannot_hold_the_separator() {
        let mut s = Session::new();
        let id =
            crate::place(&mut s, &intracraft_library::Library::built_in().unwrap(), "conveyor.roller", None, [0.0, 0.0], 0.0, 0.0).unwrap();
        assert!(set_section(&mut s, id, "Seccion.1").is_err());
        set_section(&mut s, id, "Seccion1").unwrap();
        assert_eq!(section_of(&s.model, id).as_deref(), Some("Seccion1"));
        assert_eq!(sections(&s.model), ["Seccion1"]);
        assert_eq!(object_address(&s.model, id), format!("Seccion1.Roller conveyor {id}"));
        s.execute("undo", &json!({})).unwrap();
        assert_eq!(section_of(&s.model, id), None);
    }
}
