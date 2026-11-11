//! A picture of IntraCraft's own window, saved as a PNG after so many frames, then the window closes
//! (`--screenshot FILE --frames N`): a check that a view draws, which takes nothing of the screen but the window.

use std::path::PathBuf;
use std::sync::Arc;

use egui::{ColorImage, Context, Event, ViewportCommand};

/// The picture asked for: where it goes and the frames to wait first.
#[derive(Clone, Debug, PartialEq)]
pub struct Shot {
    pub path: PathBuf,
    pub frames: u64,
    asked: bool,
}

impl Shot {
    pub fn new(path: PathBuf, frames: u64) -> Shot {
        Shot { path, frames, asked: false }
    }

    /// One frame on: asks for the window's picture once the frames are counted, saves it when it comes, then closes the
    /// window. Says what went wrong, if anything did.
    pub fn tick(&mut self, ctx: &Context) -> Option<String> {
        ctx.request_repaint();
        if !self.asked {
            self.frames = self.frames.saturating_sub(1);
            if self.frames == 0 {
                self.asked = true;
                ctx.send_viewport_cmd(ViewportCommand::Screenshot(Default::default()));
            }
            return None;
        }
        let image = ctx.input(|i| {
            i.events.iter().find_map(|e| match e {
                Event::Screenshot { image, .. } => Some(Arc::clone(image)),
                _ => None,
            })
        })?;
        let saved = save_png(&image, &self.path);
        ctx.send_viewport_cmd(ViewportCommand::Close);
        saved.err()
    }
}

fn save_png(image: &ColorImage, path: &std::path::Path) -> Result<(), String> {
    let [w, h] = image.size;
    let size = tiny_skia::IntSize::from_wh(w as u32, h as u32).ok_or("the picture is empty")?;
    let data: Vec<u8> = image.pixels.iter().flat_map(|c| c.to_array()).collect();
    let pixmap = tiny_skia::Pixmap::from_vec(data, size).ok_or("the picture doesn't fit its size")?;
    let png = pixmap.encode_png().map_err(|e| e.to_string())?;
    std::fs::write(path, png).map_err(|e| format!("{}: {e}", path.display()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_picture_is_saved_as_a_png_of_its_size() {
        let image = ColorImage::new([3, 2], vec![egui::Color32::from_rgb(10, 20, 30); 6]);
        let path = std::env::temp_dir().join(format!("intracraft-shot-test-{}.png", std::process::id()));
        save_png(&image, &path).unwrap();
        let png = std::fs::read(&path).unwrap();
        let _ = std::fs::remove_file(&path);
        assert_eq!(&png[1..4], b"PNG");
        assert_eq!(u32::from_be_bytes([png[16], png[17], png[18], png[19]]), 3);
    }
}
