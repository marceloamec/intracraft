//! A pallet from its type's occupancy bands. In each band of height, every solid interval along the length (x) meets every
//! solid interval across the width (z) in a board or a block. A band solid all over (a closed deck) is split for the
//! picture into boards that run across the members under or over it, the middle board on the axis.

use intracraft_library::reference::{OccupancyBand, PalletType};

use crate::{Material, Model, Piece};

/// The gap between the boards a closed deck is split into, m.
const BOARD_GAP: f64 = 0.012;
/// The usual pitch of a deck's boards, m: a closed top deck gets as many as fit, an odd number.
const BOARD_PITCH: f64 = 0.17;
/// The width of a closed bottom deck's boards, m.
const BOTTOM_BOARD: f64 = 0.145;

/// The pallet of a type: its boards and blocks, its underside's centre at the origin, its length along x.
pub fn pallet(t: &PalletType) -> Model {
    let mut cuts: Vec<f64> = t
        .occupancy
        .along_length
        .iter()
        .chain(&t.occupancy.along_width)
        .flat_map(|b| [b.from_height, b.to_height])
        .filter(|h| (0.0..=t.height).contains(h))
        .collect();
    cuts.push(0.0);
    cuts.push(t.height);
    cuts.sort_by(f64::total_cmp);
    cuts.dedup_by(|a, b| (*a - *b).abs() < 1e-9);
    let bands: Vec<(f64, f64)> = cuts.windows(2).map(|w| (w[0], w[1])).filter(|(a, b)| b - a > 1e-9).collect();
    let layers: Vec<Layer> = bands
        .iter()
        .filter_map(|&(from, to)| {
            let mid = (from + to) / 2.0;
            let xs = intervals_at(&t.occupancy.along_length, mid, t.length)?;
            let zs = intervals_at(&t.occupancy.along_width, mid, t.width)?;
            Some(Layer { from, to, xs, zs })
        })
        .collect();
    let last = layers.len().saturating_sub(1);
    let mut pieces = Vec::new();
    for (k, layer) in layers.iter().enumerate() {
        let (mut xs, mut zs) = (layer.xs.clone(), layer.zs.clone());
        let closed = xs.len() == 1 && zs.len() == 1 && covers(xs[0], t.length) && covers(zs[0], t.width);
        if closed {
            // The members next to it say which way they run; its boards run across them.
            let near = [k.checked_sub(1).and_then(|i| layers.get(i)), layers.get(k + 1)];
            let members_along_x = near.iter().flatten().any(|n| n.zs.len() > 1 && n.xs.len() == 1);
            let (span, bottom) = if members_along_x { (t.length, k == 0) } else { (t.width, k == 0) };
            let boards = if bottom { spread(span, t.bottom_boards.max(2) as usize, BOTTOM_BOARD) } else { split(span) };
            if members_along_x {
                xs = boards;
            } else {
                zs = boards;
            }
        }
        let deck = k == 0 || k == last;
        let blocks = xs.len() > 1 && zs.len() > 1 && !deck;
        let mut n: usize = 0;
        for &(x0, x1) in &xs {
            for &(z0, z1) in &zs {
                let material = if blocks {
                    Material::WoodDark
                } else if n.is_multiple_of(2) {
                    Material::WoodLight
                } else {
                    Material::WoodMid
                };
                n += 1;
                // Intervals are measured from the front (+x, +z), as the bands give them.
                let centre = [t.length / 2.0 - (x0 + x1) / 2.0, (layer.from + layer.to) / 2.0, t.width / 2.0 - (z0 + z1) / 2.0];
                pieces.push(Piece::cuboid(material, centre, [x1 - x0, layer.to - layer.from, z1 - z0]));
            }
        }
    }
    Model { name: format!("pallet-{}", t.id), pieces }
}

/// A band of height and its solid intervals along x and z, from the front.
struct Layer {
    from: f64,
    to: f64,
    xs: Vec<(f64, f64)>,
    zs: Vec<(f64, f64)>,
}

/// The solid intervals of the band that holds height `y`, kept within `0..extent`.
fn intervals_at(bands: &[OccupancyBand], y: f64, extent: f64) -> Option<Vec<(f64, f64)>> {
    let band = bands.iter().find(|b| b.from_height <= y && y <= b.to_height)?;
    let out: Vec<(f64, f64)> = band.solid.iter().map(|&[a, b]| (a.max(0.0), b.min(extent))).filter(|(a, b)| b > a).collect();
    (!out.is_empty()).then_some(out)
}

fn covers((a, b): (f64, f64), extent: f64) -> bool {
    a <= 1e-6 && b >= extent - 1e-6
}

/// A closed top deck as an odd number of boards across `span`, nearly touching.
fn split(span: f64) -> Vec<(f64, f64)> {
    let mut n: usize = (span / BOARD_PITCH).round().max(1.0) as usize;
    if n.is_multiple_of(2) {
        n += 1;
    }
    let width = (span - (n - 1) as f64 * BOARD_GAP) / n as f64;
    (0..n).map(|i| i as f64 * (width + BOARD_GAP)).map(|a| (a, a + width)).collect()
}

/// `n` boards of `width` spread evenly over `span`, the outer ones at its ends.
fn spread(span: f64, n: usize, width: f64) -> Vec<(f64, f64)> {
    let step = (span - width) / (n - 1) as f64;
    (0..n).map(|i| i as f64 * step).map(|a| (a, a + width)).collect()
}

#[cfg(test)]
mod tests {
    use intracraft_library::reference::Reference;

    use super::*;

    fn types() -> Vec<PalletType> {
        Reference::built_in().unwrap().pallet_types
    }

    #[test]
    fn a_eur_pallet_has_three_bottom_boards_nine_blocks_three_stringers_and_five_top_boards() {
        let eur = types().into_iter().find(|t| t.id == "eur").unwrap();
        let p = pallet(&eur);
        let count = |from: f64, to: f64| p.pieces.iter().filter(|q| q.bounds().0[1] >= from - 1e-9 && q.bounds().1[1] <= to + 1e-9).count();
        assert_eq!(count(0.0, 0.022), 3);
        assert_eq!(count(0.022, 0.1), 9);
        assert_eq!(count(0.1, 0.122), 3);
        assert_eq!(count(0.122, 0.144), 5);
    }

    #[test]
    fn every_pallet_type_fills_its_size_from_the_floor() {
        for t in types() {
            let (lo, hi) = pallet(&t).bounds().unwrap();
            for (axis, size) in [(0, t.length), (1, t.height), (2, t.width)] {
                assert!((hi[axis] - lo[axis] - size).abs() < 1e-6, "{} axis {axis}: {lo:?} {hi:?}", t.id);
            }
            assert!(lo[1].abs() < 1e-9, "{} stands at {}", t.id, lo[1]);
            assert!((lo[0] + hi[0]).abs() < 1e-6 && (lo[2] + hi[2]).abs() < 1e-6, "{} is centred", t.id);
        }
    }

    /// What a photocell sees: at the middle of each solid interval of each band, a beam across the travel meets wood.
    #[test]
    fn a_beam_meets_wood_wherever_the_occupancy_bands_say_solid() {
        for t in types() {
            let p = pallet(&t);
            let hit = |x: f64, y: f64, z_beam: bool| {
                p.pieces.iter().any(|q| {
                    let (lo, hi) = q.bounds();
                    let along = if z_beam { (lo[0], hi[0], x) } else { (lo[2], hi[2], x) };
                    along.0 <= along.2 && along.2 <= along.1 && lo[1] <= y && y <= hi[1]
                })
            };
            for (bands, extent, z_beam) in [(&t.occupancy.along_length, t.length, true), (&t.occupancy.along_width, t.width, false)] {
                for b in bands {
                    for &[a, c] in &b.solid {
                        let s = extent / 2.0 - (a + c) / 2.0;
                        let y = (b.from_height + b.to_height) / 2.0;
                        assert!(hit(s, y, z_beam), "{}: nothing at {s} m, {y} m high", t.id);
                    }
                }
            }
        }
    }
}
