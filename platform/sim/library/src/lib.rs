//! IntraCraft's object library in Rust (`A05 Component Library`, phase R1 of `A19 Rust Implementation Plan`).
//!
//! The built-in definitions and the reference data are the very files of the TypeScript reference, embedded at build
//! time: one source while the reference exists (`ADR-0020 Rust Platform on BIMCraft with Three Modules` §7). This
//! crate knows what each object is (parameters, ports, presets), checks an instance's values against the validation
//! rules, and places ports in the world to find the connections the geometry implies. It has no UI and never
//! panics: data that doesn't read is an error that says what to fix.

pub mod collate;
pub mod definition;
pub mod layout;
pub mod params;
pub mod ports;
pub mod reference;
pub mod rules;
pub mod snap;

use std::collections::BTreeMap;

use definition::{Definition, Text};
use reference::{ASSET_CATEGORIES, Reference};

/// The built-in definitions, in the reference's order.
const BUILT_IN: [(&str, &str); 14] = [
    ("roller-conveyor", include_str!("../../../../packages/library/src/definitions/roller-conveyor.json")),
    ("chain-conveyor", include_str!("../../../../packages/library/src/definitions/chain-conveyor.json")),
    ("chain-turntable", include_str!("../../../../packages/library/src/definitions/chain-turntable.json")),
    ("roller-turntable", include_str!("../../../../packages/library/src/definitions/roller-turntable.json")),
    ("transfer-rc", include_str!("../../../../packages/library/src/definitions/transfer-rc.json")),
    ("lift-table-manual", include_str!("../../../../packages/library/src/definitions/lift-table-manual.json")),
    ("scissor-lift", include_str!("../../../../packages/library/src/definitions/scissor-lift.json")),
    ("elevator", include_str!("../../../../packages/library/src/definitions/elevator.json")),
    ("gravity-line", include_str!("../../../../packages/library/src/definitions/gravity-line.json")),
    ("source", include_str!("../../../../packages/library/src/definitions/source.json")),
    ("sink", include_str!("../../../../packages/library/src/definitions/sink.json")),
    ("process-station", include_str!("../../../../packages/library/src/definitions/process-station.json")),
    ("photoeye", include_str!("../../../../packages/library/src/definitions/photoeye.json")),
    ("scenery-asset", include_str!("../../../../packages/library/src/definitions/scenery-asset.json")),
];

/// The definitions a project can place, and the reference data they use.
#[derive(Clone, Debug, PartialEq)]
pub struct Library {
    definitions: Vec<Definition>,
    pub reference: Reference,
}

impl Library {
    /// The built-in library: every definition read, its option sources filled in (catalogue models, contact
    /// materials) and checked.
    pub fn built_in() -> Result<Library, String> {
        let mut library = Library { definitions: Vec::new(), reference: Reference::built_in()? };
        for (file, json) in BUILT_IN {
            let definition: Definition = serde_json::from_str(json).map_err(|e| format!("{file}.json: {e}"))?;
            library.register(definition)?;
        }
        Ok(library)
    }

    /// Adds a definition, its option sources filled in; refused when its id is taken or it doesn't check.
    pub fn register(&mut self, mut definition: Definition) -> Result<&Definition, String> {
        definition.check()?;
        if self.get(&definition.id).is_some() {
            return Err(format!("definition {} is already registered", definition.id));
        }
        for p in &mut definition.parameters {
            let Some(from) = p.options_from.clone() else { continue };
            let (options, labels) = self.option_list(&from).map_err(|e| format!("{}.{}: {e}", definition.id, p.key))?;
            p.options.get_or_insert_with(Vec::new).extend(options);
            // The definition's own labels win over the source's.
            let mut merged = labels;
            merged.extend(p.option_labels.take().unwrap_or_default());
            p.option_labels = Some(merged);
        }
        self.definitions.push(definition);
        Ok(&self.definitions[self.definitions.len() - 1])
    }

    pub fn get(&self, id: &str) -> Option<&Definition> {
        self.definitions.iter().find(|d| d.id == id)
    }

    pub fn list(&self) -> &[Definition] {
        &self.definitions
    }

    /// The options a parameter's `optionsFrom` names: `materials`, `catalogue`, or `catalogue:<category>,…`.
    fn option_list(&self, from: &str) -> Result<(Vec<String>, BTreeMap<String, Text>), String> {
        if from == "materials" {
            let m = &self.reference.contact_materials;
            return Ok((m.iter().map(|m| m.id.clone()).collect(), m.iter().map(|m| (m.id.clone(), m.name.clone())).collect()));
        }
        let (kind, list) = match from.split_once(':') {
            Some((kind, list)) => (kind, Some(list)),
            None => (from, None),
        };
        if kind != "catalogue" {
            return Err(format!("unknown option source {from:?}"));
        }
        let categories: Vec<&str> = match list {
            None => ASSET_CATEGORIES.to_vec(),
            Some(list) => list.split(',').map(str::trim).collect(),
        };
        if let Some(bad) = categories.iter().find(|c| !ASSET_CATEGORIES.contains(c)) {
            return Err(format!("unknown catalogue category {bad:?}"));
        }
        let assets: Vec<_> = categories.iter().flat_map(|c| self.reference.assets_of(c)).collect();
        Ok((assets.iter().map(|a| a.id.clone()).collect(), assets.iter().map(|a| (a.id.clone(), a.name.clone())).collect()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::definition::{ParamType, Value};
    use crate::params::defaults;
    use crate::rules::validate_instance;

    fn library() -> Library {
        Library::built_in().unwrap()
    }

    #[test]
    fn loads_the_roller_conveyor() {
        let lib = library();
        let roller = lib.get("conveyor.roller").unwrap();
        assert_eq!(roller.name.de.as_deref(), Some("Rollenförderer"));
        assert_eq!(roller.ports.iter().map(|p| p.id.as_str()).collect::<Vec<_>>(), ["A", "B"]);
    }

    #[test]
    fn holds_the_fourteen_built_in_objects_in_the_reference_order() {
        let lib = library();
        let ids: Vec<&str> = lib.list().iter().map(|d| d.id.as_str()).collect();
        assert_eq!(
            ids,
            [
                "conveyor.roller",
                "conveyor.chain",
                "conveyor.chain_turntable",
                "conveyor.roller_turntable",
                "conveyor.transfer_rc",
                "conveyor.lift_table_manual",
                "conveyor.scissor_lift",
                "conveyor.elevator",
                "conveyor.gravity",
                "flow.source",
                "flow.sink",
                "flow.process_station",
                "sensor.photoeye",
                "scenery.asset",
            ]
        );
    }

    #[test]
    fn offers_every_catalogue_model_in_the_scenery_object_named_as_in_the_catalogue() {
        let lib = library();
        let asset = lib.get("scenery.asset").unwrap().parameter("asset").unwrap();
        let ids: Vec<&str> = lib.reference.assets.iter().map(|a| a.id.as_str()).collect();
        assert_eq!(asset.options.as_deref().unwrap_or_default().iter().map(String::as_str).collect::<Vec<_>>(), ids);
        let labels = asset.option_labels.as_ref().unwrap();
        for a in &lib.reference.assets {
            assert_eq!(labels[&a.id].en, a.name.en);
        }
    }

    #[test]
    fn offers_auto_and_every_contact_material_in_the_scenery_object() {
        let lib = library();
        let material = lib.get("scenery.asset").unwrap().parameter("material").unwrap();
        let mut expected = vec!["auto".to_string()];
        expected.extend(lib.reference.contact_materials.iter().map(|m| m.id.clone()));
        assert_eq!(material.options.clone().unwrap(), expected);
    }

    #[test]
    fn offers_catalogue_pallets_as_pallet_models_and_boxes_containers_drums_as_loads() {
        let lib = library();
        let source = lib.get("flow.source").unwrap();
        let mut pallets = vec!["auto".to_string(), "none".to_string()];
        pallets.extend(lib.reference.assets_of("pallets").map(|a| a.id.clone()));
        assert_eq!(source.parameter("palletModel").unwrap().options.clone().unwrap(), pallets);
        let loads: Vec<String> =
            ["boxes", "containers", "drums"].iter().flat_map(|c| lib.reference.assets_of(c)).map(|a| a.id.clone()).collect();
        assert_eq!(source.parameter("loadAsset").unwrap().options.clone().unwrap(), loads);
        assert_eq!(source.parameter("loadKind").unwrap().options.clone().unwrap(), ["block", "none", "asset"]);
        let values = defaults(source);
        assert_eq!(values["palletModel"], Value::Text("auto".into()));
        assert_eq!(values["loadKind"], Value::Text("block".into()));
        let types: Vec<String> = lib.reference.pallet_types.iter().map(|p| p.id.clone()).collect();
        assert_eq!(source.parameter("palletType").unwrap().options.clone().unwrap(), types);
    }

    #[test]
    fn every_definition_is_complete_and_passes_its_own_rules_with_its_defaults() {
        let lib = library();
        for d in lib.list() {
            for p in &d.parameters {
                assert!(!p.label.en.is_empty() && !p.description.en.is_empty(), "{} {}", d.id, p.key);
                if let Some(Value::Number(v)) = &p.default {
                    assert!(p.min.is_none_or(|m| *v >= m), "{} {} below min", d.id, p.key);
                    assert!(p.max.is_none_or(|m| *v <= m), "{} {} above max", d.id, p.key);
                    if let Some(choices) = &p.choices {
                        assert!(choices.contains(v), "{} {} not a choice", d.id, p.key);
                    }
                }
                if p.kind == ParamType::Enum {
                    let default = p.default.as_ref().and_then(Value::text).unwrap_or_default();
                    assert!(p.options.as_deref().unwrap_or_default().iter().any(|o| o == default), "{} {}", d.id, p.key);
                }
                if let Some(labels) = &p.option_labels {
                    for o in p.options.as_deref().unwrap_or_default() {
                        assert!(labels.get(o).is_some_and(|l| !l.en.is_empty()), "{} {} {o}", d.id, p.key);
                    }
                }
            }
            assert_eq!(validate_instance(d, &Default::default(), &lib.reference), [], "{}", d.id);
        }
    }

    #[test]
    fn lets_the_operator_source_and_sink_go_down_to_50_mm() {
        let lib = library();
        for id in ["flow.source", "flow.sink"] {
            assert_eq!(lib.get(id).unwrap().parameter("surfaceHeight").unwrap().min, Some(0.05), "{id}");
        }
    }

    #[test]
    fn refuses_a_definition_twice_and_a_malformed_one() {
        let mut lib = library();
        let roller = lib.get("conveyor.roller").unwrap().clone();
        assert!(lib.register(roller.clone()).is_err());
        let mut bad = roller;
        bad.id = "Conveyor roller".into();
        assert!(lib.register(bad).is_err());
    }
}
