//! `intracraft`: pallet conveyor design and simulation on BIMCraft, in three modules (ADR-0020).
//!
//! `intracraft [FILE] [--module design|discrete|physics] [--view plan|3d] [--play]`: FILE is a `.bimcraft` project, or a
//! layout of the TypeScript reference (`.intracraft.json`), which is placed in a new project; `--module` opens that
//! module's ribbon tab, `--view 3d` the 3D view, and `--play` starts the module's run at once.

#![cfg_attr(all(windows, not(debug_assertions)), windows_subsystem = "windows")]

mod app;
mod glyphs;
mod plan;

use std::process::ExitCode;

use bimcraft_core::engine::Session;
use bimcraft_ui_egui::BimApp;
use intracraft_library::Library;
use intracraft_library::layout::Project;

fn main() -> ExitCode {
    let library = match Library::built_in() {
        Ok(l) => l,
        Err(e) => {
            eprintln!("intracraft: the library doesn't read: {e}");
            return ExitCode::FAILURE;
        }
    };
    let args: Vec<String> = std::env::args().skip(1).collect();
    let option = |name: &str| args.iter().position(|a| a == name).and_then(|k| args.get(k + 1)).map(String::as_str);
    let module = match option("--module") {
        Some("discrete") => Some(app::Module::Discrete),
        Some("physics") => Some(app::Module::Physics),
        Some("design") => Some(app::Module::Design),
        _ => None,
    };
    let three_d = option("--view") == Some("3d");
    let play = args.iter().any(|a| a == "--play");
    let valued = ["--module", "--view"];
    let file = args.iter().enumerate().find(|(k, a)| !a.starts_with("--") && (*k == 0 || !valued.contains(&args[k - 1].as_str())));
    let file = file.map(|(_, a)| a.clone());
    let mut session = Session::new();
    let mut problem = None;
    if let Some(file) = file {
        problem = open(&mut session, &library, &file).err();
    }
    // The library's objects are families of the project, so the Project Browser places them too.
    if let Err(e) = session.transaction("Load the IntraCraft library", |s| intracraft_model::load_library(s, &library)) {
        problem = Some(e);
    }
    let mut bim = BimApp::new(session);
    if let Some(text) = problem {
        bim.status = bimcraft_ui_egui::Status { text, error: true };
    }
    let viewport = eframe::egui::ViewportBuilder::default().with_title("IntraCraft").with_inner_size([1600.0, 950.0]);
    let options = eframe::NativeOptions { viewport, centered: true, ..Default::default() };
    let start = move |cc: &eframe::CreationContext| -> Result<Box<dyn eframe::App>, _> {
        if let Some(state) = cc.wgpu_render_state.as_ref() {
            bim.use_graphics_card(state);
        }
        let mut app = app::IntraApp::new(bim, library);
        if let Some(m) = module {
            app.open(m);
        }
        if three_d {
            app.bim.open_view(bimcraft_ui_egui::Sheet::View3D);
        }
        app.sim.autoplay = play;
        Ok(Box::new(app))
    };
    match eframe::run_native("IntraCraft", options, Box::new(start)) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("intracraft: {e}");
            ExitCode::FAILURE
        }
    }
}

/// Opens a project, or places a reference layout in the new one.
fn open(session: &mut Session, library: &Library, file: &str) -> Result<(), String> {
    let path = locate(file, &std::env::current_dir().unwrap_or_default())
        .ok_or_else(|| format!("{file}: there is no such file here or in the folders above"))?;
    if file.ends_with(".intracraft.json") {
        let text = std::fs::read_to_string(&path).map_err(|e| format!("{file}: {e}"))?;
        let project = Project::read(&text)?;
        intracraft_model::import_project(session, library, &project).map(|_| ())
    } else {
        session.execute("open", &serde_json::json!({"path": path})).map(|_| ())
    }
}

/// Where `file` is: as given, or, when `folder` has no such file, under the first folder above it that has (`cargo
/// run` runs in `platform/`, while a layout's path is often given from the repository's root).
fn locate(file: &str, folder: &std::path::Path) -> Option<std::path::PathBuf> {
    let given = std::path::Path::new(file);
    if given.is_absolute() {
        return given.exists().then(|| given.to_path_buf());
    }
    folder.ancestors().map(|a| a.join(given)).find(|p| p.exists())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_layout_named_from_the_repository_s_root_is_found_from_the_platform_folder() {
        let platform = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let layout = "packages/scenarios/scenarios/layouts/flow-t1.intracraft.json";
        assert!(locate(layout, &platform).is_some_and(|p| p.ends_with(layout)));
        assert!(locate("no/such/layout.intracraft.json", &platform).is_none());
    }
}
