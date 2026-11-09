//! IntraCraft's own 3D models of pallets and their loads (ADR-0026), and a glTF 2.0 binary writer that turns them into
//! asset files any 3D program reads. Our own geometry, no third-party models.
//!
//! A pallet is built from its type's occupancy bands (S06, T06): in each band of height, the solid intervals along its
//! length meet the solid intervals across its width in boards and blocks. What a photocell sees of a pallet and what the
//! 3D view draws are therefore the same construction. Where a band is solid all over (a closed deck), it is split into
//! boards for the picture, with a board on each axis so the beam's view is kept.
//!
//! Frames: metres, y up; the pallet's length along x, its width along z; the origin at the centre of its underside, as the
//! physics run places a pallet (`PalletPose::centre`). A load stands on the origin too, its base at y = 0, so it is placed
//! on the pallet's top.

pub mod glb;
pub mod load;
pub mod pallet;

pub use load::{LoadKind, load};
pub use pallet::pallet;

use intracraft_library::reference::Reference;

/// The height the load files are made at, m: the viewer stretches them to a source's load height.
pub const LOAD_FILE_HEIGHT: f64 = 1.0;

/// Where the files live, under the viewer's assets: `pallets/`.
pub const FOLDER: &str = "pallets";

/// The asset files: each pallet type of the library, and each load on each pallet type's footprint, by file name.
pub fn files(reference: &Reference) -> Vec<(String, Vec<u8>)> {
    let mut out = Vec::new();
    for t in &reference.pallet_types {
        out.push((pallet_file(&t.id), glb::write(&pallet(t))));
        for kind in LoadKind::ALL {
            out.push((load_file(&t.id, kind), glb::write(&load(kind, t.length, t.width, LOAD_FILE_HEIGHT))));
        }
    }
    out
}

/// The file of a pallet type's model.
pub fn pallet_file(pallet_type: &str) -> String {
    format!("pallet-{pallet_type}.glb")
}

/// The file of a load on a pallet type's footprint.
pub fn load_file(pallet_type: &str, kind: LoadKind) -> String {
    format!("load-{pallet_type}-{}.glb", kind.name())
}

/// The materials of the models, with their look (physically based).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Material {
    /// Pine boards: two shades, so neighbouring boards read apart.
    WoodLight,
    WoodMid,
    /// The blocks and stringers, darker end grain.
    WoodDark,
    Carton,
    CartonDark,
    Tape,
    /// Stretch film, seen through.
    Film,
    DrumBlue,
    SteelRim,
    CrateGrey,
    CrateBlue,
}

impl Material {
    pub const ALL: [Material; 11] = [
        Material::WoodLight,
        Material::WoodMid,
        Material::WoodDark,
        Material::Carton,
        Material::CartonDark,
        Material::Tape,
        Material::Film,
        Material::DrumBlue,
        Material::SteelRim,
        Material::CrateGrey,
        Material::CrateBlue,
    ];

    pub fn name(self) -> &'static str {
        match self {
            Material::WoodLight => "wood-light",
            Material::WoodMid => "wood-mid",
            Material::WoodDark => "wood-dark",
            Material::Carton => "carton",
            Material::CartonDark => "carton-dark",
            Material::Tape => "tape",
            Material::Film => "stretch-film",
            Material::DrumBlue => "drum-blue",
            Material::SteelRim => "steel-rim",
            Material::CrateGrey => "crate-grey",
            Material::CrateBlue => "crate-blue",
        }
    }

    /// Its colour (linear RGB) and opacity.
    pub fn color(self) -> [f32; 4] {
        match self {
            Material::WoodLight => [0.63, 0.42, 0.20, 1.0],
            Material::WoodMid => [0.55, 0.35, 0.16, 1.0],
            Material::WoodDark => [0.40, 0.25, 0.11, 1.0],
            Material::Carton => [0.50, 0.32, 0.15, 1.0],
            Material::CartonDark => [0.43, 0.27, 0.12, 1.0],
            Material::Tape => [0.62, 0.50, 0.30, 1.0],
            Material::Film => [0.85, 0.88, 0.92, 0.28],
            Material::DrumBlue => [0.03, 0.12, 0.40, 1.0],
            Material::SteelRim => [0.55, 0.57, 0.60, 1.0],
            Material::CrateGrey => [0.18, 0.20, 0.23, 1.0],
            Material::CrateBlue => [0.03, 0.15, 0.55, 1.0],
        }
    }

    pub fn metallic(self) -> f32 {
        match self {
            Material::DrumBlue => 0.5,
            Material::SteelRim => 0.9,
            _ => 0.0,
        }
    }

    pub fn roughness(self) -> f32 {
        match self {
            Material::WoodLight | Material::WoodMid | Material::WoodDark => 0.85,
            Material::Carton | Material::CartonDark => 0.9,
            Material::Tape => 0.5,
            Material::Film => 0.15,
            Material::DrumBlue => 0.45,
            Material::SteelRim => 0.35,
            Material::CrateGrey | Material::CrateBlue => 0.6,
        }
    }

    /// Whether it is seen through.
    pub fn blends(self) -> bool {
        self.color()[3] < 1.0
    }
}

/// A solid of a model: a box by its sizes, or a cylinder standing along y.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Solid {
    Box([f64; 3]),
    Cylinder { radius: f64, height: f64 },
}

/// A solid placed in its model by its centre.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Piece {
    pub solid: Solid,
    pub centre: [f64; 3],
    pub material: Material,
}

impl Piece {
    pub fn cuboid(material: Material, centre: [f64; 3], size: [f64; 3]) -> Piece {
        Piece { solid: Solid::Box(size), centre, material }
    }

    pub fn cylinder(material: Material, centre: [f64; 3], radius: f64, height: f64) -> Piece {
        Piece { solid: Solid::Cylinder { radius, height }, centre, material }
    }

    /// The corners of the box round it.
    pub fn bounds(&self) -> ([f64; 3], [f64; 3]) {
        let half = match self.solid {
            Solid::Box(s) => [s[0] / 2.0, s[1] / 2.0, s[2] / 2.0],
            Solid::Cylinder { radius, height } => [radius, height / 2.0, radius],
        };
        let c = self.centre;
        ([c[0] - half[0], c[1] - half[1], c[2] - half[2]], [c[0] + half[0], c[1] + half[1], c[2] + half[2]])
    }
}

/// A model: its name and its pieces.
#[derive(Clone, Debug, PartialEq)]
pub struct Model {
    pub name: String,
    pub pieces: Vec<Piece>,
}

impl Model {
    /// The corners of the box round all its pieces, or none when it has none.
    pub fn bounds(&self) -> Option<([f64; 3], [f64; 3])> {
        self.pieces
            .iter()
            .map(Piece::bounds)
            .reduce(|(lo, hi), (a, b)| (std::array::from_fn(|i| lo[i].min(a[i])), std::array::from_fn(|i| hi[i].max(b[i]))))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The files kept with the viewer are the ones the models make today: `cargo run -p intracraft-assets` writes them.
    #[test]
    fn the_viewer_s_asset_files_are_up_to_date() {
        let folder = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../apps/intracraft-viewer/assets").join(FOLDER);
        let reference = Reference::built_in().unwrap();
        let files = files(&reference);
        assert_eq!(files.len(), reference.pallet_types.len() * (1 + LoadKind::ALL.len()));
        for (name, bytes) in files {
            let kept = std::fs::read(folder.join(&name)).unwrap_or_default();
            assert!(kept == bytes, "{name} is missing or out of date: run `cargo run -p intracraft-assets`");
            assert!(glb::read_json(&kept).is_ok());
        }
    }
}
