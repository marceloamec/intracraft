//! Reads `result-egui.json` and `result-bevy.json` beside the program (or in the folder given) and writes
//! `resultados.txt` there, the owner's plain-language report.

use std::path::{Path, PathBuf};

use r5b_scene::bench::BenchResult;
use r5b_scene::report;

fn main() {
    let folder = std::env::args().nth(1).map(PathBuf::from).unwrap_or_else(|| {
        std::env::current_exe().ok().and_then(|e| e.parent().map(Path::to_path_buf)).unwrap_or_default()
    });
    let read = |name: &str| -> Option<BenchResult> {
        let text = std::fs::read_to_string(folder.join(name)).ok()?;
        serde_json::from_str(&text).ok()
    };
    let text = report::render(read("result-egui.json").as_ref(), read("result-bevy.json").as_ref());
    let out = folder.join("resultados.txt");
    match std::fs::write(&out, &text) {
        Ok(()) => println!("{text}\n(guardado en {})", out.display()),
        Err(e) => {
            eprintln!("{}: {e}", out.display());
            println!("{text}");
        }
    }
}
