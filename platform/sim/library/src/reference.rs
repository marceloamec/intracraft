//! Reference data (S10, S16, A16): pallet types, contact materials and their pairs, and the models of the asset
//! catalogue. The same files as the TypeScript reference; indicative starting values, not manufacturer data.

use serde::{Deserialize, Serialize};

use crate::definition::Text;

const PALLET_TYPES: &str = include_str!("../../../../packages/library/src/definitions/pallet-types.json");
const CONTACT_MATERIALS: &str = include_str!("../../../../packages/library/src/definitions/contact-materials.json");
const CONTACT_PAIRS: &str = include_str!("../../../../packages/library/src/definitions/contact-pairs.json");
const CATALOGUE: &str = include_str!("../../../../packages/asset-catalog/src/catalog.json");

/// Heights above the conveying surface (m) where a horizontal beam across the travel meets solid parts, and the
/// intervals along the pallet (from its front, m) that are solid there.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OccupancyBand {
    pub from_height: f64,
    pub to_height: f64,
    pub solid: Vec<[f64; 2]>,
}

/// `along_length`: the pallet travels long side first (rollers); `along_width`: short side first (chains).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Occupancy {
    pub along_length: Vec<OccupancyBand>,
    pub along_width: Vec<OccupancyBand>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ProfileStatus {
    Indicative,
    Approximate,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PalletType {
    pub id: String,
    pub name: Text,
    pub length: f64,
    pub width: f64,
    pub height: f64,
    pub tare_mass: f64,
    pub bottom_boards: u32,
    pub profile_status: ProfileStatus,
    pub occupancy: Occupancy,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ContactMaterial {
    pub id: String,
    pub name: Text,
    pub description: Text,
}

/// The contact of two materials (S16): one friction coefficient, its usual range, and one restitution; either order.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ContactPair {
    pub a: String,
    pub b: String,
    pub mu: f64,
    pub range: [f64; 2],
    pub restitution: f64,
}

/// A model's size, m: x and z in plan, y up.
#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Dimensions {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

/// A model of the asset catalogue, as far as the library and the engines need it: what option it is, how it is
/// called, and its size.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Asset {
    pub id: String,
    pub category: String,
    pub name: Text,
    #[serde(default)]
    pub dimensions: Dimensions,
}

/// The catalogue's categories, in the reference's order.
pub const ASSET_CATEGORIES: [&str; 8] = ["pallets", "boxes", "containers", "drums", "racks", "vehicles", "conveyors", "safety"];

#[derive(Deserialize)]
struct Catalogue {
    assets: Vec<Asset>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Reference {
    pub pallet_types: Vec<PalletType>,
    pub contact_materials: Vec<ContactMaterial>,
    pub contact_pairs: Vec<ContactPair>,
    pub assets: Vec<Asset>,
}

impl Reference {
    /// The built-in reference data, checked: occupancy inside the pallet, pairs of known materials once each, μ in its
    /// range and restitution in [0, 1].
    pub fn built_in() -> Result<Reference, String> {
        let read = |what: &str, text: &str| -> Result<serde_json::Value, String> {
            serde_json::from_str(text).map_err(|e| format!("{what}: {e}"))
        };
        let reference = Reference {
            pallet_types: serde_json::from_value(read("pallet types", PALLET_TYPES)?).map_err(|e| format!("pallet types: {e}"))?,
            contact_materials: serde_json::from_value(read("contact materials", CONTACT_MATERIALS)?)
                .map_err(|e| format!("contact materials: {e}"))?,
            contact_pairs: serde_json::from_value(read("contact pairs", CONTACT_PAIRS)?).map_err(|e| format!("contact pairs: {e}"))?,
            assets: serde_json::from_str::<Catalogue>(CATALOGUE).map_err(|e| format!("asset catalogue: {e}"))?.assets,
        };
        reference.check()?;
        Ok(reference)
    }

    fn check(&self) -> Result<(), String> {
        for pair in &self.contact_pairs {
            if !(pair.mu >= 0.0 && (0.0..=1.0).contains(&pair.restitution)) {
                return Err(format!("contact pair {}–{}: μ or restitution out of range", pair.a, pair.b));
            }
            for m in [&pair.a, &pair.b] {
                if !self.contact_materials.iter().any(|c| &c.id == m) {
                    return Err(format!("contact pair {}–{}: unknown material {m}", pair.a, pair.b));
                }
            }
        }
        Ok(())
    }

    pub fn pallet_type(&self, id: &str) -> Option<&PalletType> {
        self.pallet_types.iter().find(|p| p.id == id)
    }

    /// The pair of materials `a` and `b`, in either order.
    pub fn contact_pair(&self, a: &str, b: &str) -> Option<&ContactPair> {
        self.contact_pairs.iter().find(|p| (p.a == a && p.b == b) || (p.a == b && p.b == a))
    }

    pub fn asset(&self, id: &str) -> Option<&Asset> {
        self.assets.iter().find(|a| a.id == id)
    }

    /// The catalogue's models of a category, in the catalogue's order.
    pub fn assets_of(&self, category: &str) -> impl Iterator<Item = &Asset> {
        self.assets.iter().filter(move |a| a.category == category)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn reference() -> Reference {
        Reference::built_in().unwrap()
    }

    #[test]
    fn includes_eur_1200x1000_and_gma_pallets() {
        let r = reference();
        let ids: Vec<&str> = r.pallet_types.iter().map(|p| p.id.as_str()).collect();
        assert_eq!(ids, ["eur", "iso1200x1000", "gma"]);
    }

    #[test]
    fn occupancy_bands_stack_up_to_the_pallet_height_in_both_orientations() {
        for pallet in reference().pallet_types {
            for (side, bands, extent) in [
                ("alongLength", &pallet.occupancy.along_length, pallet.length),
                ("alongWidth", &pallet.occupancy.along_width, pallet.width),
            ] {
                assert_eq!(bands.first().map(|b| b.from_height), Some(0.0), "{} {side}", pallet.id);
                assert_eq!(bands.last().map(|b| b.to_height), Some(pallet.height), "{} {side}", pallet.id);
                for band in bands {
                    for [from, to] in &band.solid {
                        assert!(*from >= 0.0 && *to <= extent + 1e-9 && to > from, "{} {side}", pallet.id);
                    }
                }
            }
        }
    }

    #[test]
    fn pairs_every_material_with_every_material_once() {
        let r = reference();
        assert_eq!(r.contact_pairs.len(), 55);
        let mut seen = std::collections::BTreeSet::new();
        for pair in &r.contact_pairs {
            let key = if pair.a < pair.b { (pair.a.clone(), pair.b.clone()) } else { (pair.b.clone(), pair.a.clone()) };
            assert!(seen.insert(key), "{}–{} twice", pair.a, pair.b);
            assert!(pair.mu >= pair.range[0] && pair.mu <= pair.range[1], "{}–{}", pair.a, pair.b);
        }
        for a in &r.contact_materials {
            for b in &r.contact_materials {
                assert!(r.contact_pair(&a.id, &b.id).is_some(), "{}–{}", a.id, b.id);
            }
        }
    }

    #[test]
    fn looks_pairs_up_in_either_order() {
        let r = reference();
        assert_eq!(r.contact_pair("wood", "steel-zinc").map(|p| p.mu), Some(0.35));
        assert_eq!(r.contact_pair("steel-zinc", "wood"), r.contact_pair("wood", "steel-zinc"));
        let cardboard = r.contact_pair("cardboard", "steel-zinc").unwrap();
        assert_eq!((cardboard.mu, cardboard.restitution), (0.3, 0.0));
        assert!(r.contact_pair("rubber", "glass").is_none());
    }

    #[test]
    fn grips_better_on_rubber_and_bounces_more_on_rubber_buffers() {
        let r = reference();
        let mu = |a, b| r.contact_pair(a, b).unwrap().mu;
        let e = |a, b| r.contact_pair(a, b).unwrap().restitution;
        assert!(mu("wood", "rubber") > mu("wood", "steel-zinc"));
        assert!(e("wood", "rubber") > e("wood", "steel-painted"));
    }

    #[test]
    fn reads_the_43_catalogue_models() {
        let r = reference();
        assert_eq!(r.assets.len(), 43);
        assert!(r.assets.iter().all(|a| ASSET_CATEGORIES.contains(&a.category.as_str())));
    }
}
