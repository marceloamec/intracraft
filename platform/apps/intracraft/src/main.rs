//! `intracraft`: pallet conveyor design and simulation on BIMCraft, in three modules (ADR-0020).
//!
//! `intracraft [FILE]`: FILE is a `.bimcraft` project, or a layout of the TypeScript reference (`.intracraft.json`),
//! which is placed in a new project.

#![cfg_attr(all(windows, not(debug_assertions)), windows_subsystem = "windows")]

mod app;
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
    let mut session = Session::new();
    let mut problem = None;
    if let Some(file) = std::env::args().nth(1) {
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
        Ok(Box::new(app::IntraApp::new(bim, library)))
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
    if file.ends_with(".intracraft.json") {
        let text = std::fs::read_to_string(file).map_err(|e| format!("{file}: {e}"))?;
        let project = Project::read(&text)?;
        intracraft_model::import_project(session, library, &project).map(|_| ())
    } else {
        session.execute("open", &serde_json::json!({"path": file})).map(|_| ())
    }
}
