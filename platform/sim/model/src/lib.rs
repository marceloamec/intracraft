//! IntraCraft's Design module on BIMCraft (A18, phase R1–R2 of A19 Rust Implementation Plan).
//!
//! The library's definitions become BIMCraft families; a placed object is a BIMCraft component with a unique id and a
//! GUID; its attribute table shows each value, where it comes from and whether it is inherited; the simulation tree
//! lists the objects the simulations will run; and the layout the simulation modules read is built from the BIM model.
//! Every change goes through BIMCraft's own commands (`family`, `component`, `modify`), so it can be undone, is checked
//! by `Model::check`, and is open to the CLI and to agents.

pub mod attributes;
pub mod family;
pub mod forms;
pub mod layout;
pub mod tree;

use bimcraft_core::engine::Session;
use bimcraft_core::model::Id;
use intracraft_library::Library;
use intracraft_library::definition::Value;
use intracraft_library::layout::Project;
use serde_json::{Value as Json, json};

use crate::attributes::{attributes, check_value, definition_of};
use crate::family::{DEFAULT_TYPE, family_name, family_params, to_bim};

/// Loads every library definition into the project as a family, or brings the families up to date; their names.
pub fn load_library(session: &mut Session, library: &Library) -> Result<Vec<String>, String> {
    materials(session)?;
    let mut names = Vec::new();
    for d in library.list() {
        session.execute("family", &family_params(d)).map_err(|e| format!("{}: {e}", d.id))?;
        names.push(family_name(d));
    }
    Ok(names)
}

/// Makes the materials the families' forms are made of, those the project lacks (a project may have recoloured them).
fn materials(session: &mut Session) -> Result<(), String> {
    for (name, colour) in forms::MATERIALS {
        if !session.model.materials.contains_key(name) {
            session.execute("material", &json!({"name": name, "color": colour}))?;
        }
    }
    Ok(())
}

/// Places a library object of `definition` (its type `type_name`, else its default) at `at` (mm, plan), turned `angle`
/// degrees, `offset` mm above its level (the first level by default), loading its family first if the project lacks it;
/// the new element's id.
pub fn place(
    session: &mut Session,
    library: &Library,
    definition: &str,
    type_name: Option<&str>,
    at: [f64; 2],
    angle: f64,
    offset: f64,
) -> Result<Id, String> {
    let d = library.get(definition).ok_or_else(|| format!("the library has no {definition}"))?;
    let family = family_name(d);
    if !session.model.families.contains_key(&family) {
        materials(session)?;
        session.execute("family", &family_params(d)).map_err(|e| format!("{definition}: {e}"))?;
    }
    let r = session.execute(
        "component",
        &json!({"family": family, "type": type_name.unwrap_or(DEFAULT_TYPE), "at": at, "angle": angle, "offset": offset}),
    )?;
    r["id"].as_u64().ok_or_else(|| "the component made no id".to_string())
}

/// Gives object `id` its own value of attribute `key` (SI), checked as the library says: the attribute is overridden.
pub fn set_attribute(session: &mut Session, library: &Library, id: Id, key: &str, value: &Value) -> Result<(), String> {
    let d = definition_of(&session.model, library, id).ok_or_else(|| format!("{id} is not an object of the IntraCraft library"))?;
    check_value(d, key, value)?;
    let p = d.parameter(key).ok_or_else(|| format!("{} has no attribute {key}", d.id))?;
    session.execute("modify", &json!({"id": id, "params": {key: to_bim(p, value)}})).map(|_| ())
}

/// Takes object `id`'s own value of `key` away: the attribute is inherited again, from its type or the library.
pub fn reset_attribute(session: &mut Session, library: &Library, id: Id, key: &str) -> Result<(), String> {
    let d = definition_of(&session.model, library, id).ok_or_else(|| format!("{id} is not an object of the IntraCraft library"))?;
    d.parameter(key).ok_or_else(|| format!("{} has no attribute {key}", d.id))?;
    session.execute("modify", &json!({"id": id, "params": {key: Json::Null}})).map(|_| ())
}

/// Detaches object `id` from its library type: every value it inherits becomes its own, so later changes to the type or
/// the library no longer reach it. One undo step.
pub fn detach(session: &mut Session, library: &Library, id: Id) -> Result<usize, String> {
    let d = definition_of(&session.model, library, id).ok_or_else(|| format!("{id} is not an object of the IntraCraft library"))?;
    let attrs = attributes(&session.model, library, id)?;
    let mut params = serde_json::Map::new();
    for a in attrs.iter().filter(|a| matches!(a.source, attributes::Source::Type | attributes::Source::Library)) {
        if let (Some(p), Some(v)) = (d.parameter(&a.key), &a.value) {
            params.insert(a.key.clone(), to_bim(p, v));
        }
    }
    let n = params.len();
    if n > 0 {
        session.execute("modify", &json!({"id": id, "params": params}))?;
    }
    Ok(n)
}

/// Places every instance of a reference project (`.intracraft.json`) in the model, with its own values: the reference's
/// layouts open in the Design module. The new elements' ids, in the file's order.
pub fn import_project(session: &mut Session, library: &Library, project: &Project) -> Result<Vec<Id>, String> {
    session.transaction("Import IntraCraft layout", |s| {
        let mut ids = Vec::new();
        for i in &project.layout.instances {
            let d = library.get(&i.definition_id).ok_or_else(|| format!("{}: the library has no {}", i.id, i.definition_id))?;
            let (at, height) = layout::to_model(i.position);
            let id = place(s, library, &d.id, None, [at.0, at.1], i.yaw.to_degrees(), height)?;
            let mut params = serde_json::Map::new();
            for (k, v) in &i.params {
                let p = d.parameter(k).ok_or_else(|| format!("{}: {} has no attribute {k}", i.id, d.id))?;
                if !p.derived {
                    params.insert(k.clone(), to_bim(p, v));
                }
            }
            if !params.is_empty() {
                s.execute("modify", &json!({"id": id, "params": params}))?;
            }
            ids.push(id);
        }
        Ok(ids)
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::attributes::Source;
    use crate::tree::simulation_tree;
    use intracraft_library::snap::derive_connections;

    fn session() -> (Session, Library) {
        (Session::new(), Library::built_in().unwrap())
    }

    fn attr(s: &Session, lib: &Library, id: Id, key: &str) -> attributes::Attribute {
        attributes(&s.model, lib, id).unwrap().into_iter().find(|a| a.key == key).unwrap()
    }

    #[test]
    fn loads_every_library_object_as_a_family() {
        let (mut s, lib) = session();
        let names = load_library(&mut s, &lib).unwrap();
        assert_eq!(names.len(), 14);
        for n in &names {
            assert!(s.model.families.contains_key(n), "{n}");
        }
        // Loading again brings them up to date and changes nothing else.
        assert_eq!(load_library(&mut s, &lib).unwrap(), names);
    }

    #[test]
    fn a_placed_conveyor_inherits_every_attribute_until_it_sets_its_own() {
        let (mut s, lib) = session();
        let id = place(&mut s, &lib, "conveyor.roller", None, [0.0, 0.0], 0.0, 0.0).unwrap();
        let length = attr(&s, &lib, id, "length");
        assert_eq!((length.value.clone(), length.source, length.inherited()), (Some(Value::Number(3.0)), Source::Library, true));
        set_attribute(&mut s, &lib, id, "length", &Value::Number(5.0)).unwrap();
        let length = attr(&s, &lib, id, "length");
        assert_eq!((length.value.clone(), length.source, length.inherited()), (Some(Value::Number(5.0)), Source::Instance, false));
        // The derived roller count follows: 5 m at 206.3 mm pitch is 24 rollers.
        assert_eq!(attr(&s, &lib, id, "rollerCount").value, Some(Value::Number(24.0)));
        assert_eq!(attr(&s, &lib, id, "rollerCount").source, Source::Derived);
        reset_attribute(&mut s, &lib, id, "length").unwrap();
        assert_eq!(attr(&s, &lib, id, "length").source, Source::Library);
        // Undo brings the override back: every change is a command.
        s.execute("undo", &json!({})).unwrap();
        assert_eq!(attr(&s, &lib, id, "length").value, Some(Value::Number(5.0)));
    }

    #[test]
    fn a_preset_is_a_type_its_objects_inherit_from() {
        let (mut s, lib) = session();
        let id = place(&mut s, &lib, "conveyor.elevator", Some("Chains, EUR transverse"), [0.0, 0.0], 0.0, 0.0).unwrap();
        let deck = attr(&s, &lib, id, "deckType");
        assert_eq!((deck.value, deck.source), (Some(Value::Text("chains".into())), Source::Type));
    }

    #[test]
    fn values_outside_the_library_rules_are_refused_and_change_nothing() {
        let (mut s, lib) = session();
        let id = place(&mut s, &lib, "conveyor.roller", None, [0.0, 0.0], 0.0, 0.0).unwrap();
        for (key, v) in
            [("length", Value::Number(-1.0)), ("accumulationMode", Value::Text("lots".into())), ("rollerCount", Value::Number(3.0))]
        {
            assert!(set_attribute(&mut s, &lib, id, key, &v).is_err(), "{key}");
        }
        assert!(attributes(&s.model, &lib, id).unwrap().iter().all(|a| a.source != Source::Instance));
    }

    #[test]
    fn detaching_makes_every_inherited_value_the_objects_own() {
        let (mut s, lib) = session();
        let id = place(&mut s, &lib, "conveyor.roller", None, [0.0, 0.0], 0.0, 0.0).unwrap();
        let n = detach(&mut s, &lib, id).unwrap();
        let attrs = attributes(&s.model, &lib, id).unwrap();
        assert_eq!(n, attrs.iter().filter(|a| a.source == Source::Instance).count());
        assert!(attrs.iter().all(|a| matches!(a.source, Source::Instance | Source::Derived)));
        // Later changes to the family no longer reach it.
        s.execute("family", &json!({"name": "Roller conveyor", "parameters": []})).ok();
        assert_eq!(detach(&mut s, &lib, id).unwrap(), 0);
    }

    #[test]
    fn the_reference_layouts_open_in_the_design_module_with_the_same_connections() {
        let (mut s, lib) = session();
        let project =
            Project::read(include_str!("../../../../packages/scenarios/scenarios/layouts/reference-all-five.intracraft.json")).unwrap();
        let ids = import_project(&mut s, &lib, &project).unwrap();
        assert_eq!(ids.len(), project.layout.instances.len());
        let (from_model, problems) = layout::layout(&s.model, &lib);
        assert!(problems.is_empty(), "{problems:?}");
        assert_eq!(from_model.instances.len(), project.layout.instances.len());
        // The same plant: the same connections, port to port, once each id is mapped.
        let name = |i: &str| {
            let k = from_model.instances.iter().position(|x| x.id == i).unwrap();
            project.layout.instances[k].id.clone()
        };
        let mut got: Vec<String> = from_model
            .connections
            .iter()
            .map(|c| format!("{}.{}→{}.{}", name(&c.from.instance_id), c.from.port_id, name(&c.to.instance_id), c.to.port_id))
            .collect();
        let mut want: Vec<String> = derive_connections(&project.layout, &lib)
            .unwrap()
            .iter()
            .map(|c| format!("{}.{}→{}.{}", c.from.instance_id, c.from.port_id, c.to.instance_id, c.to.port_id))
            .collect();
        got.sort();
        want.sort();
        assert!(!want.is_empty());
        assert_eq!(got, want);
        // One undo step takes the whole import back.
        s.execute("undo", &json!({})).unwrap();
        assert_eq!(layout::layout(&s.model, &lib).0.instances.len(), 0);
    }

    #[test]
    fn the_simulation_tree_lists_every_object_by_group_with_its_unique_id() {
        let (mut s, lib) = session();
        let source = place(&mut s, &lib, "flow.source", None, [0.0, 0.0], 0.0, 0.0).unwrap();
        let roller = place(&mut s, &lib, "conveyor.roller", None, [420.0, 0.0], 0.0, 0.0).unwrap();
        let sink = place(&mut s, &lib, "flow.sink", None, [3440.0, 0.0], 0.0, 0.0).unwrap();
        set_attribute(&mut s, &lib, roller, "nominalSpeed", &Value::Number(0.3)).unwrap();
        let tree = simulation_tree(&s.model, &lib);
        let names: Vec<&str> = tree.iter().map(|g| g.name).collect();
        assert_eq!(names, ["Sources", "Conveyors", "Sinks"]);
        assert_eq!(tree[0].leaves[0].id, source);
        assert_eq!(tree[2].leaves[0].id, sink);
        let leaf = &tree[1].leaves[0];
        assert_eq!((leaf.id, leaf.overridden), (roller, 1));
        assert!(leaf.label().starts_with(&format!("{roller}  Roller conveyor · Default")), "{}", leaf.label());
        // Source → conveyor → sink connect from the model alone.
        let (l, _) = layout::layout(&s.model, &lib);
        assert_eq!(l.connections.len(), 2, "{:?}", l.connections);
    }
}
