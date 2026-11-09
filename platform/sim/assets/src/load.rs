//! Loads that stand on a pallet: cartons in layers, the same wrapped in stretch film, four steel drums, and stacked plastic
//! crates. Each fills the pallet's footprint (`length` along x, `width` along z) up to `height`, its base at y = 0.

use crate::{Material, Model, Piece};

/// A carton's usual size, m: as many fit on the pallet as the nearest whole number gives.
const CARTON: [f64; 3] = [0.40, 0.25, 0.30];
/// The gap between neighbouring cartons, m.
const CARTON_GAP: f64 = 0.006;
const TAPE_WIDTH: f64 = 0.05;
const TAPE_THICKNESS: f64 = 0.002;
/// How far the film stands off the cartons, m.
const FILM_OFFSET: f64 = 0.004;
/// A drum's rims: their height and how far they stand out, m.
const RIM_HEIGHT: f64 = 0.02;
const RIM_OUT: f64 = 0.008;
/// A plastic crate's usual size, m, and the height of its darker top band.
const CRATE: [f64; 3] = [0.60, 0.25, 0.40];
const CRATE_BAND: f64 = 0.03;

/// The kinds of load the models cover.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum LoadKind {
    Cartons,
    WrappedCartons,
    Drums,
    Crates,
}

impl LoadKind {
    pub const ALL: [LoadKind; 4] = [LoadKind::Cartons, LoadKind::WrappedCartons, LoadKind::Drums, LoadKind::Crates];

    pub fn name(self) -> &'static str {
        match self {
            LoadKind::Cartons => "cartons",
            LoadKind::WrappedCartons => "wrapped-cartons",
            LoadKind::Drums => "drums",
            LoadKind::Crates => "crates",
        }
    }

    /// The load a source's choice stands for: `block` is a wrapped block of cartons (S09 §1.1); an asset of the catalogue
    /// by its category; `none` is none.
    pub fn of_source(load_kind: &str, load_asset: &str) -> Option<LoadKind> {
        match load_kind {
            "none" => None,
            "asset" => Some(match load_asset.split('.').nth(1) {
                Some("drums") => LoadKind::Drums,
                Some("containers") => LoadKind::Crates,
                Some("boxes") => LoadKind::Cartons,
                _ => LoadKind::WrappedCartons,
            }),
            _ => Some(LoadKind::WrappedCartons),
        }
    }
}

/// The load of `kind` on a pallet `length` × `width`, `height` high.
pub fn load(kind: LoadKind, length: f64, width: f64, height: f64) -> Model {
    let mut pieces = Vec::new();
    match kind {
        LoadKind::Cartons => cartons(&mut pieces, length, width, height),
        LoadKind::WrappedCartons => {
            cartons(&mut pieces, length - 2.0 * FILM_OFFSET, width - 2.0 * FILM_OFFSET, height - FILM_OFFSET);
            pieces.push(Piece::cuboid(Material::Film, [0.0, height / 2.0, 0.0], [length, height, width]));
        }
        LoadKind::Drums => drums(&mut pieces, length, width, height),
        LoadKind::Crates => crates(&mut pieces, length, width, height),
    }
    Model { name: format!("load-{}", kind.name()), pieces }
}

/// How many of `size` fit in `span`, at least one, and how big each then is.
fn fit(span: f64, size: f64) -> (usize, f64) {
    let n = (span / size).round().max(1.0) as usize;
    (n, span / n as f64)
}

/// Cartons in a grid filling the space, each layer turned shade by shade, a strip of tape across each carton on top.
fn cartons(pieces: &mut Vec<Piece>, length: f64, width: f64, height: f64) {
    let ((nx, sx), (ny, sy), (nz, sz)) = (fit(length, CARTON[0]), fit(height, CARTON[1]), fit(width, CARTON[2]));
    for j in 0..ny {
        for i in 0..nx {
            for k in 0..nz {
                // Each carton stands on the layer under it; the gap is above it and beside it.
                let base = j as f64 * sy;
                let centre = [-length / 2.0 + (i as f64 + 0.5) * sx, base + (sy - CARTON_GAP) / 2.0, -width / 2.0 + (k as f64 + 0.5) * sz];
                let shade = if (i + j + k) % 2 == 0 { Material::Carton } else { Material::CartonDark };
                pieces.push(Piece::cuboid(shade, centre, [sx - CARTON_GAP, sy - CARTON_GAP, sz - CARTON_GAP]));
                if j + 1 == ny {
                    let top = base + sy - CARTON_GAP + TAPE_THICKNESS / 2.0;
                    pieces.push(Piece::cuboid(Material::Tape, [centre[0], top, centre[2]], [sx - CARTON_GAP, TAPE_THICKNESS, TAPE_WIDTH]));
                }
            }
        }
    }
}

/// Four drums, two by two out to the pallet's corners, with a rim at the bottom, the middle and the top and a lid.
fn drums(pieces: &mut Vec<Piece>, length: f64, width: f64, height: f64) {
    let radius = (length / 4.0).min(width / 4.0) - RIM_OUT - 0.01;
    for (x, z) in [(-1.0, -1.0), (-1.0, 1.0), (1.0, -1.0), (1.0, 1.0)] {
        // Out to the pallet's edges, as drums are loaded.
        let (cx, cz) = (x * (length / 2.0 - radius - RIM_OUT), z * (width / 2.0 - radius - RIM_OUT));
        pieces.push(Piece::cylinder(Material::DrumBlue, [cx, height / 2.0, cz], radius, height));
        for y in [RIM_HEIGHT / 2.0, height / 2.0, height - RIM_HEIGHT / 2.0] {
            pieces.push(Piece::cylinder(Material::SteelRim, [cx, y, cz], radius + RIM_OUT, RIM_HEIGHT));
        }
        pieces.push(Piece::cylinder(Material::SteelRim, [cx, height + 0.002, cz], radius * 0.9, 0.004));
    }
}

/// Plastic crates in layers, blue and grey by turns, each with a darker band round its top.
fn crates(pieces: &mut Vec<Piece>, length: f64, width: f64, height: f64) {
    let ((nx, sx), (ny, sy), (nz, sz)) = (fit(length, CRATE[0]), fit(height, CRATE[1]), fit(width, CRATE[2]));
    for j in 0..ny {
        let body = if j % 2 == 0 { Material::CrateBlue } else { Material::CrateGrey };
        for i in 0..nx {
            for k in 0..nz {
                let (cx, cz) = (-length / 2.0 + (i as f64 + 0.5) * sx, -width / 2.0 + (k as f64 + 0.5) * sz);
                let base = j as f64 * sy;
                let size = [sx - CARTON_GAP, sy - CRATE_BAND, sz - CARTON_GAP];
                pieces.push(Piece::cuboid(body, [cx, base + size[1] / 2.0, cz], size));
                pieces.push(Piece::cuboid(Material::CrateGrey, [cx, base + sy - CRATE_BAND / 2.0, cz], [sx, CRATE_BAND, sz]));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_load_fills_the_pallet_footprint_to_its_height_and_no_further() {
        for kind in LoadKind::ALL {
            let (lo, hi) = load(kind, 1.2, 0.8, 1.2).bounds().unwrap();
            assert!(lo[1].abs() < 1e-9, "{kind:?} starts at {}", lo[1]);
            assert!(hi[1] <= 1.21 && hi[1] > 1.15, "{kind:?} is {} high", hi[1]);
            assert!(hi[0] - lo[0] <= 1.2 + 1e-9 && hi[2] - lo[2] <= 0.8 + 1e-9, "{kind:?}: {lo:?} {hi:?}");
            assert!(hi[0] - lo[0] > 1.1 && hi[2] - lo[2] > 0.7, "{kind:?} leaves the pallet bare: {lo:?} {hi:?}");
        }
    }

    #[test]
    fn a_eur_pallet_of_cartons_holds_three_by_three_by_five_with_a_tape_on_each_top_carton() {
        let m = load(LoadKind::Cartons, 1.2, 0.8, 1.2);
        let count = |mat: Material| m.pieces.iter().filter(|p| p.material == mat).count();
        assert_eq!(count(Material::Carton) + count(Material::CartonDark), 3 * 5 * 3);
        assert_eq!(count(Material::Tape), 3 * 3);
    }

    #[test]
    fn a_source_s_choice_picks_its_load() {
        assert_eq!(LoadKind::of_source("block", ""), Some(LoadKind::WrappedCartons));
        assert_eq!(LoadKind::of_source("none", ""), None);
        assert_eq!(LoadKind::of_source("asset", "asset.drums.drum_steel_a01"), Some(LoadKind::Drums));
        assert_eq!(LoadKind::of_source("asset", "asset.containers.crate_blue_a01"), Some(LoadKind::Crates));
        assert_eq!(LoadKind::of_source("asset", "asset.boxes.box_cardboard_a01"), Some(LoadKind::Cartons));
    }
}
