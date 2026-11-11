//! The simulation tree (A18 §3.2): every intralogistics object of the project, grouped as the simulation sees them,
//! each with its unique id and how many of its attributes it inherits and overrides.

use bimcraft_core::model::{Element, Id, Model};
use intracraft_library::Library;
use intracraft_library::definition::Definition;

use crate::attributes::{Source, attributes, definition_of};

#[derive(Clone, Debug, PartialEq)]
pub struct Leaf {
    pub id: Id,
    pub family: String,
    pub type_name: String,
    pub definition: String,
    pub inherited: usize,
    pub overridden: usize,
}

impl Leaf {
    /// `100245  Roller conveyor · Default  [inherited 21 / overridden 2]`.
    pub fn label(&self) -> String {
        format!("{}  {} · {}  [inherited {} / overridden {}]", self.id, self.family, self.type_name, self.inherited, self.overridden)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Group {
    pub name: &'static str,
    pub leaves: Vec<Leaf>,
}

/// The groups of the tree, in the order a line is read: what feeds it, what carries, what senses, what processes,
/// what stores, what takes away, the code that drives them, and the scenery.
pub const GROUPS: [&str; 9] =
    ["Sources", "Conveyors", "Turntables and transfers", "Lifts and stations", "Sensors", "Storage", "Sinks", "Code blocks", "Scenery"];

fn group_of(d: &Definition) -> &'static str {
    match d.behaviors.des.as_str() {
        "des.source" => "Sources",
        "des.sink" => "Sinks",
        "des.turntable" | "des.transfer_rc" => "Turntables and transfers",
        "des.lift_station" | "des.process_station" | "des.stretch_wrapper" => "Lifts and stations",
        "des.photoeye" => "Sensors",
        "des.storage_aisle" => "Storage",
        "des.none" => "Scenery",
        "des.script" => "Code blocks",
        _ => "Conveyors",
    }
}

/// The tree of the project's intralogistics objects; empty groups are left out, objects within a group go by id.
pub fn simulation_tree(model: &Model, library: &Library) -> Vec<Group> {
    let mut groups: Vec<Group> = GROUPS.iter().map(|name| Group { name, leaves: Vec::new() }).collect();
    for (&id, element) in model.elements.iter() {
        let Element::Component(c) = element else { continue };
        let Some(d) = definition_of(model, library, id) else { continue };
        let attrs = attributes(model, library, id).unwrap_or_default();
        let overridden = attrs.iter().filter(|a| a.source == Source::Instance).count();
        let inherited = attrs.iter().filter(|a| matches!(a.source, Source::Type | Source::Library)).count();
        let leaf = Leaf { id, family: c.family.clone(), type_name: c.type_name.clone(), definition: d.id.clone(), inherited, overridden };
        if let Some(g) = groups.iter_mut().find(|g| g.name == group_of(d)) {
            g.leaves.push(leaf);
        }
    }
    groups.retain(|g| !g.leaves.is_empty());
    groups
}
