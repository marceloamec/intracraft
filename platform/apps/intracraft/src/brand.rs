//! IntraCraft's brand on BIMCraft's window: the name, the line under it and the rail logo on the start page, the window's
//! icon, and `.intracraft` as the file a project is saved to (`.bimcraft` files open as well). The logo is variant A of
//! `brand/logo/variants/a-rail.svg`: a navy tile with two white rails and an amber pallet between them. It is drawn here,
//! and the pictures in `brand/export` (the program's icon among them) are made from this drawing:
//! `cargo run -p intracraft -- --write-brand` writes them again, and a test says when they are out of date.

use bimcraft_ui_egui::Brand;
use egui::{Color32, CornerRadius, Rect};
use tiny_skia::{FillRule, Paint, PathBuilder, Pixmap, Transform};

pub const BRAND: Brand = Brand {
    name: "IntraCraft",
    tagline: "Pallet conveyors, designed and simulated: simulate before you build.",
    project: "IntraCraft project",
    extension: "intracraft",
    source: "https://github.com/marceloamec/intracraft",
    logo,
};

/// The brand's colours (`brand/tokens/colors.json`).
const NAVY: [u8; 3] = [0x0B, 0x25, 0x45];
const AMBER: [u8; 3] = [0xF2, 0xA9, 0x00];
const WHITE: [u8; 3] = [0xFF, 0xFF, 0xFF];

/// The symbol on its 64-unit grid: the tile's corner radius, then each bar as x, y, width, height and colour.
const RADIUS: f32 = 12.0;
const BARS: [([f32; 4], [u8; 3]); 3] =
    [([16.0, 14.0, 32.0, 8.0], WHITE), ([16.0, 42.0, 32.0, 8.0], WHITE), ([28.0, 22.0, 8.0, 20.0], AMBER)];

/// The sizes of the pictures written: the window's icon is the 256 one; the program's `.ico` holds 16 to 256.
const PNG_SIZES: [u32; 4] = [64, 256, 512, 1024];
const ICO_SIZES: [u32; 5] = [16, 32, 48, 64, 256];
/// Where the pictures are kept, from this crate.
pub const EXPORT: &str = "../../../brand/export";

/// The rail logo, drawn into the square `r` of the start page.
pub fn logo(p: &egui::Painter, r: Rect) {
    let k = r.width() / 64.0;
    let rgb = |c: [u8; 3]| Color32::from_rgb(c[0], c[1], c[2]);
    p.rect_filled(r, CornerRadius::same((RADIUS * k).round() as u8), rgb(NAVY));
    for ([x, y, w, h], c) in BARS {
        let at = Rect::from_min_size(r.min + egui::vec2(x, y) * k, egui::vec2(w, h) * k);
        p.rect_filled(at, 0.0, rgb(c));
    }
}

/// The logo as a square PNG `size` pixels wide, its corners clear.
pub fn png(size: u32) -> Result<Vec<u8>, String> {
    let mut pixmap = Pixmap::new(size, size).ok_or("a picture of no size")?;
    let k = size as f32 / 64.0;
    let fill = |pixmap: &mut Pixmap, path: &tiny_skia::Path, c: [u8; 3]| {
        let mut paint = Paint::default();
        paint.set_color_rgba8(c[0], c[1], c[2], 255);
        paint.anti_alias = true;
        pixmap.fill_path(path, &paint, FillRule::Winding, Transform::from_scale(k, k), None);
    };
    let tile = rounded(64.0, RADIUS).ok_or("the tile has no outline")?;
    fill(&mut pixmap, &tile, NAVY);
    for ([x, y, w, h], c) in BARS {
        let bar = tiny_skia::Rect::from_xywh(x, y, w, h).map(PathBuilder::from_rect).ok_or("a bar has no outline")?;
        fill(&mut pixmap, &bar, c);
    }
    pixmap.encode_png().map_err(|e| e.to_string())
}

/// The pictures kept in `brand/export`, by name: the PNGs, and `intracraft.ico`, the program's icon on Windows.
pub fn files() -> Result<Vec<(String, Vec<u8>)>, String> {
    let mut files = Vec::new();
    for size in PNG_SIZES {
        files.push((format!("intracraft-{size}.png"), png(size)?));
    }
    let images = ICO_SIZES.iter().map(|&s| png(s).map(|p| (s, p))).collect::<Result<Vec<_>, _>>()?;
    files.push(("intracraft.ico".into(), ico(&images)));
    Ok(files)
}

/// Writes [`files`] into `folder`, and what it wrote.
pub fn write(folder: &std::path::Path) -> Result<Vec<std::path::PathBuf>, String> {
    std::fs::create_dir_all(folder).map_err(|e| format!("{}: {e}", folder.display()))?;
    let mut written = Vec::new();
    for (name, bytes) in files()? {
        let path = folder.join(name);
        std::fs::write(&path, bytes).map_err(|e| format!("{}: {e}", path.display()))?;
        written.push(path);
    }
    Ok(written)
}

/// A Windows icon file holding PNG images, each with its size (256 is written as 0, as the format says).
fn ico(images: &[(u32, Vec<u8>)]) -> Vec<u8> {
    let mut out = Vec::new();
    out.extend(0u16.to_le_bytes());
    out.extend(1u16.to_le_bytes());
    out.extend((images.len() as u16).to_le_bytes());
    let mut offset = 6 + 16 * images.len();
    for (size, png) in images {
        let side = if *size >= 256 { 0 } else { *size as u8 };
        out.extend([side, side, 0, 0]);
        out.extend(1u16.to_le_bytes()); // colour planes
        out.extend(32u16.to_le_bytes()); // bits per pixel
        out.extend((png.len() as u32).to_le_bytes());
        out.extend((offset as u32).to_le_bytes());
        offset += png.len();
    }
    for (_, png) in images {
        out.extend(png);
    }
    out
}

/// A square `side` wide with corners rounded to `r`.
fn rounded(side: f32, r: f32) -> Option<tiny_skia::Path> {
    // How far a cubic's handles reach along a quarter circle.
    const K: f32 = 0.552_284_8;
    let (s, h) = (side, r * (1.0 - K));
    let mut p = PathBuilder::new();
    p.move_to(r, 0.0);
    p.line_to(s - r, 0.0);
    p.cubic_to(s - h, 0.0, s, h, s, r);
    p.line_to(s, s - r);
    p.cubic_to(s, s - h, s - h, s, s - r, s);
    p.line_to(r, s);
    p.cubic_to(h, s, 0.0, s - h, 0.0, s - r);
    p.line_to(0.0, r);
    p.cubic_to(0.0, h, h, 0.0, r, 0.0);
    p.close();
    p.finish()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The pictures kept in `brand/export` (and built into the program as its icon) are the ones the logo draws today.
    #[test]
    fn the_brand_s_pictures_are_up_to_date() {
        let folder = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(EXPORT);
        for (name, bytes) in files().unwrap() {
            let kept = std::fs::read(folder.join(&name)).unwrap_or_default();
            assert!(kept == bytes, "brand/export/{name} is missing or out of date: run `cargo run -p intracraft -- --write-brand`");
        }
    }

    #[test]
    fn the_icon_is_the_rail_logo_navy_with_an_amber_pallet_and_clear_corners() {
        let png = png(64).unwrap();
        let pixmap = Pixmap::decode_png(&png).unwrap();
        let at = |x: u32, y: u32| {
            let c = pixmap.pixel(x, y).unwrap().demultiply();
            [c.red(), c.green(), c.blue(), c.alpha()]
        };
        assert_eq!(at(0, 0)[3], 0, "a rounded corner is clear");
        assert_eq!(at(6, 32), [NAVY[0], NAVY[1], NAVY[2], 255]);
        assert_eq!(at(32, 18), [255, 255, 255, 255], "the upper rail");
        assert_eq!(at(32, 32), [AMBER[0], AMBER[1], AMBER[2], 255], "the pallet");
    }

    #[test]
    fn the_icon_file_holds_each_size_with_its_image() {
        let images: Vec<(u32, Vec<u8>)> = ICO_SIZES.iter().map(|&s| (s, png(s).unwrap())).collect();
        let ico = ico(&images);
        assert_eq!(&ico[..6], &[0, 0, 1, 0, ICO_SIZES.len() as u8, 0]);
        let entry = |k: usize| &ico[6 + 16 * k..6 + 16 * (k + 1)];
        assert_eq!(entry(0)[0], 16);
        assert_eq!(entry(ICO_SIZES.len() - 1)[0], 0, "256 is written as 0");
        let (size, offset) = (entry(1)[8..12].to_vec(), entry(1)[12..16].to_vec());
        let (size, offset) =
            (u32::from_le_bytes(size.try_into().unwrap()) as usize, u32::from_le_bytes(offset.try_into().unwrap()) as usize);
        assert_eq!(&ico[offset..offset + size], images[1].1.as_slice());
    }
}
