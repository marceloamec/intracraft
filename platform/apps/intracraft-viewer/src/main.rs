//! `intracraft-viewer`: IntraCraft's 3D window (ADR-0026). A layout's discrete run or physics run (the plant on Avian),
//! drawn with Bevy: the objects with their rollers, chains, frames and legs, turntables turning and decks rising, the
//! pallets with their loads (the own glTF models), each object's state as a colour under it.
//!
//! `intracraft-viewer FILE.intracraft.json [--module discrete|physics] [--play] [--frames N [--screenshot FILE.png]]`:
//! `--play` starts the run at once, `--speed N` at one of its speeds (×1, ×2, ×5 physics; ×1 to ×10 000 discrete); `--frames N` closes the window after N frames and prints what was on show (a check
//! that it runs), with `--screenshot` saving a picture of the window first.

// Bevy's systems are spelled as functions of many queries; that is how the engine is used (ADR-0026).
#![allow(clippy::type_complexity, clippy::too_many_arguments)]
#![cfg_attr(all(windows, not(debug_assertions)), windows_subsystem = "windows")]

mod camera;
mod panel;
mod run;
mod view;

use std::path::{Path, PathBuf};

use bevy::prelude::*;
use bevy::render::view::screenshot::{Screenshot, save_to_disk};
use bevy::window::WindowResolution;
use bevy_egui::{EguiPlugin, EguiPrimaryContextPass};
use intracraft_library::Library;
use intracraft_library::layout::Project;

use camera::{Drag, Home, Orbit, Picked};
use panel::FileName;
use run::{Module, Run};
use view::{Latest, LayoutRes, Objects, OnShow, ShowStatus};

fn main() -> AppExit {
    match start() {
        Ok(app) => app,
        Err(e) => {
            eprintln!("intracraft-viewer: {e}");
            AppExit::error()
        }
    }
}

fn start() -> Result<AppExit, String> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let option = |name: &str| args.iter().position(|a| a == name).and_then(|k| args.get(k + 1)).cloned();
    let module = match option("--module").as_deref() {
        Some("discrete") => Module::Discrete,
        _ => Module::Physics,
    };
    let frames = option("--frames").and_then(|n| n.parse::<u64>().ok());
    let screenshot = option("--screenshot").map(PathBuf::from);
    let valued = ["--module", "--frames", "--screenshot", "--speed"];
    let file = args
        .iter()
        .enumerate()
        .find(|(k, a)| !a.starts_with("--") && (*k == 0 || !valued.contains(&args[k - 1].as_str())))
        .map(|(_, a)| a.clone())
        .ok_or("usage: intracraft-viewer FILE.intracraft.json [--module discrete|physics] [--play] [--frames N [--screenshot FILE]]")?;
    let path = locate(&file, &std::env::current_dir().unwrap_or_default())
        .ok_or_else(|| format!("{file}: no such file here or in the folders above"))?;
    let text = std::fs::read_to_string(&path).map_err(|e| format!("{}: {e}", path.display()))?;
    let project = Project::read(&text)?;
    let library = Library::built_in()?;
    let objects = intracraft_scene::scene(&project.layout, &library);
    let (lo, hi) = intracraft_scene::bounds(&objects).unwrap_or(([-5.0, 0.0, -5.0], [5.0, 2.0, 5.0]));
    let mut run = Run::start(module, project.layout.clone(), project.seed, library.clone())?;
    run.playing = args.iter().any(|a| a == "--play");
    if let Some(speed) = option("--speed").and_then(|s| s.parse::<f64>().ok()).filter(|s| run.speeds().contains(s)) {
        run.speed = speed;
    }
    let name = path.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or(file);

    let mut app = App::new();
    app.add_plugins(
        DefaultPlugins
            .set(WindowPlugin {
                primary_window: Some(Window {
                    title: format!("IntraCraft 3D – {name}"),
                    resolution: WindowResolution::new(1600, 900),
                    ..default()
                }),
                ..default()
            })
            .set(AssetPlugin { file_path: assets_folder().to_string_lossy().into_owned(), ..default() }),
    )
    .add_plugins(EguiPlugin::default())
    .insert_resource(ClearColor(Color::srgb(0.80, 0.84, 0.88)))
    .insert_resource(GlobalAmbientLight { color: Color::WHITE, brightness: 500.0, ..default() })
    .insert_non_send(run)
    .insert_resource(Objects(objects))
    .insert_resource(LayoutRes { layout: project.layout, library })
    .insert_resource(Home(Orbit::framing(lo, hi)))
    .insert_resource(FileName(name))
    .insert_resource(ShowStatus(true))
    .init_resource::<Latest>()
    .init_resource::<OnShow>()
    .init_resource::<Picked>()
    .init_resource::<Drag>()
    .add_systems(Startup, (view::spawn_scene, view::load_looks, camera::spawn_camera))
    .add_systems(Update, (step_run, view::move_platforms, view::paint_states, view::place_pallets).chain())
    .add_systems(Update, (camera::steer, camera::pick).chain())
    .add_systems(EguiPrimaryContextPass, panel::panel);
    if let Some(n) = frames {
        app.insert_resource(FrameLimit { left: n, screenshot }).add_systems(Last, close_after);
    }
    Ok(app.run())
}

/// Moves the run on by the frame's time and takes its picture.
fn step_run(time: Res<Time>, mut run: NonSendMut<Run>, mut latest: ResMut<Latest>) {
    run.advance(time.delta_secs_f64().min(0.1));
    latest.0 = run.picture();
}

/// The frames left before the window closes, and the picture to take before.
#[derive(Resource)]
struct FrameLimit {
    left: u64,
    screenshot: Option<PathBuf>,
}

/// Frames the picture is taken before the window closes: saving it takes a few.
const SCREENSHOT_LEAD: u64 = 30;

/// Closes the window after the frames asked for, saying what it showed; takes the picture asked for just before.
fn close_after(
    mut commands: Commands,
    mut limit: ResMut<FrameLimit>,
    run: NonSend<Run>,
    latest: Res<Latest>,
    objects: Res<Objects>,
    mut exit: MessageWriter<AppExit>,
) {
    limit.left = limit.left.saturating_sub(1);
    if limit.left <= SCREENSHOT_LEAD
        && let Some(path) = limit.screenshot.take()
    {
        commands.spawn(Screenshot::primary_window()).observe(save_to_disk(path));
    }
    if limit.left == 0 {
        println!("intracraft-viewer: {} objects, {} pallets on show at {}", objects.0.len(), latest.0.pallets.len(), run.clock());
        exit.write(AppExit::Success);
    }
}

/// The viewer's assets: next to its sources when built here, else next to the program.
fn assets_folder() -> PathBuf {
    let built = Path::new(env!("CARGO_MANIFEST_DIR")).join("assets");
    if built.is_dir() {
        return built;
    }
    std::env::current_exe().ok().and_then(|e| e.parent().map(|d| d.join("assets"))).unwrap_or(built)
}

/// Where `file` is: as given, or under the first folder above `folder` that has it.
fn locate(file: &str, folder: &Path) -> Option<PathBuf> {
    let given = Path::new(file);
    if given.is_absolute() {
        return given.exists().then(|| given.to_path_buf());
    }
    folder.ancestors().map(|a| a.join(given)).find(|p| p.exists())
}
