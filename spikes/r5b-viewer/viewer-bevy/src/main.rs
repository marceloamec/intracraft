//! Spike R5b, the Bevy viewer: the benchmark scene with lights, cascaded shadows and the catalogue pallet loaded by
//! Bevy's own glTF loader. Every frame it takes a snapshot of the scene and moves 2 000 rollers and 200 pallets, as
//! the 3D run's viewer would.
//!
//! `r5b-viewer-bevy` opens the scene; `--bench 30` measures for 30 s and writes `result-bevy.json`.

// Bevy's queries are spelled as types; that is how the engine is used.
#![allow(clippy::type_complexity)]

use std::path::PathBuf;
use std::time::{Duration, Instant};

use bevy::app::AppExit;
use bevy::light::CascadeShadowConfigBuilder;
use bevy::prelude::*;
use bevy::render::renderer::RenderAdapterInfo;
use bevy::window::{PresentMode, WindowResolution};
use bevy_egui::{EguiContexts, EguiPlugin, EguiPrimaryContextPass, egui};

use r5b_scene::bench::{Args, BenchResult, FrameStats, WINDOW};
use r5b_scene::{
    LOAD_SIZE, PALLET_SIZE, PALLETS, ROLLER_LENGTH, ROLLER_RADIUS, ROLLERS_PER_CONVEYOR, SIDES, Scene, Snapshot,
};

/// How long the camera takes to circle the scene when not measuring, s.
const ORBIT: f32 = 60.0;

fn main() -> AppExit {
    let started = Instant::now();
    let args = match Args::parse(std::env::args()) {
        Ok(args) => args,
        Err(e) => {
            eprintln!("{e}\nusage: r5b-viewer-bevy [--bench SECONDS] [--glb FILE] [--out FILE]");
            return AppExit::error();
        }
    };
    let glb = args.glb_or_default();
    // The asset server reads from one folder: the model's.
    let asset_dir = glb
        .as_ref()
        .and_then(|p| p.parent().map(|d| d.to_string_lossy().into_owned()))
        .unwrap_or_else(|| "assets".into());
    let present_mode = if args.bench.is_some() { PresentMode::AutoNoVsync } else { PresentMode::AutoVsync };
    App::new()
        .add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "R5b - Bevy viewer".into(),
                        resolution: WindowResolution::new(WINDOW[0], WINDOW[1]).with_scale_factor_override(1.0),
                        present_mode,
                        ..default()
                    }),
                    ..default()
                })
                .set(AssetPlugin { file_path: asset_dir, ..default() }),
        )
        .add_plugins(EguiPlugin::default())
        .insert_resource(Run {
            scene: Scene::new(),
            snap: Snapshot::default(),
            args,
            glb,
            started,
            glb_requested: started,
            glb_ms: None,
            first_frame_ms: None,
            measuring_since: None,
            last_frame: None,
            frames: Vec::with_capacity(20_000),
            pallet_triangles: 0,
            done: false,
        })
        .insert_resource(GlobalAmbientLight { color: Color::WHITE, brightness: 400.0, ..default() })
        .add_systems(Startup, setup)
        .add_systems(Update, (animate, watch_glb))
        .add_systems(Last, measure)
        .add_systems(EguiPrimaryContextPass, panel)
        .run()
}

/// Everything the viewer keeps between frames.
#[derive(Resource)]
struct Run {
    scene: Scene,
    snap: Snapshot,
    args: Args,
    glb: Option<PathBuf>,
    started: Instant,
    glb_requested: Instant,
    glb_ms: Option<f64>,
    first_frame_ms: Option<f64>,
    measuring_since: Option<Instant>,
    last_frame: Option<Instant>,
    frames: Vec<Duration>,
    pallet_triangles: usize,
    done: bool,
}

#[derive(Component)]
struct Roller(usize);

#[derive(Component)]
struct Pallet(usize);

#[derive(Resource)]
struct PalletModel(Handle<WorldAsset>);

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
    mut run: ResMut<Run>,
) {
    for b in run.scene.statics() {
        let material = materials.add(StandardMaterial {
            base_color: Color::srgb(b.color[0], b.color[1], b.color[2]),
            perceptual_roughness: 0.7,
            ..default()
        });
        commands.spawn((
            Mesh3d(meshes.add(Cuboid::new(b.size[0], b.size[1], b.size[2]))),
            MeshMaterial3d(material),
            Transform::from_xyz(b.centre[0], b.centre[1], b.centre[2]).with_rotation(Quat::from_rotation_y(b.yaw)),
        ));
    }

    // One mesh and one material for all rollers, so Bevy batches them into few draws.
    let roller_mesh = meshes.add(Cylinder::new(ROLLER_RADIUS, ROLLER_LENGTH).mesh().resolution(16));
    let roller_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.78, 0.8, 0.82),
        metallic: 0.8,
        perceptual_roughness: 0.35,
        ..default()
    });
    let rollers = ROLLERS_PER_CONVEYOR * SIDES.iter().sum::<usize>();
    for i in 0..rollers {
        commands.spawn((
            Roller(i),
            Mesh3d(roller_mesh.clone()),
            MeshMaterial3d(roller_material.clone()),
            Transform::default(),
        ));
    }

    let load_mesh = meshes.add(Cuboid::new(LOAD_SIZE[0], LOAD_SIZE[1], LOAD_SIZE[2]));
    let load_material = materials.add(StandardMaterial { base_color: Color::srgb(0.72, 0.55, 0.33), ..default() });
    let name = run.glb.as_ref().and_then(|p| p.file_name()).map(|n| n.to_string_lossy().into_owned());
    run.glb_requested = Instant::now();
    let model: Option<Handle<WorldAsset>> =
        name.map(|name| asset_server.load(GltfAssetLabel::Scene(0).from_asset(name)));
    let (box_mesh, box_material) = (
        meshes.add(Cuboid::new(PALLET_SIZE[0], PALLET_SIZE[1], PALLET_SIZE[2])),
        materials.add(StandardMaterial { base_color: Color::srgb(0.6, 0.45, 0.28), ..default() }),
    );
    let pallet_height = if model.is_some() { 0.209 } else { PALLET_SIZE[1] };
    for i in 0..PALLETS {
        let mut pallet = commands.spawn((Pallet(i), Transform::default(), Visibility::default()));
        pallet.with_children(|p| {
            match &model {
                Some(handle) => {
                    p.spawn(WorldAssetRoot(handle.clone()));
                }
                None => {
                    p.spawn((
                        Mesh3d(box_mesh.clone()),
                        MeshMaterial3d(box_material.clone()),
                        Transform::from_xyz(0.0, PALLET_SIZE[1] / 2.0, 0.0),
                    ));
                }
            }
            p.spawn((
                Mesh3d(load_mesh.clone()),
                MeshMaterial3d(load_material.clone()),
                Transform::from_xyz(0.0, pallet_height + LOAD_SIZE[1] / 2.0, 0.0),
            ));
        });
    }
    if model.is_none() {
        run.pallet_triangles = 12;
    }
    if let Some(handle) = model {
        commands.insert_resource(PalletModel(handle));
    }

    let (centre, radius) = run.scene.centre_and_radius();
    commands.spawn((
        DirectionalLight { illuminance: 8_000.0, shadow_maps_enabled: true, ..default() },
        Transform::from_xyz(centre[0] + 30.0, 60.0, centre[1] + 20.0)
            .looking_at(Vec3::new(centre[0], 0.0, centre[1]), Vec3::Y),
        CascadeShadowConfigBuilder { maximum_distance: radius * 3.0, ..default() }.build(),
    ));
    commands.spawn((Camera3d::default(), Transform::default()));
}

/// Moves every roller and pallet to the scene's snapshot, and the camera along its circle.
fn animate(
    time: Res<Time>,
    mut run: ResMut<Run>,
    mut rollers: Query<(&Roller, &mut Transform), (Without<Pallet>, Without<Camera3d>)>,
    mut pallets: Query<(&Pallet, &mut Transform), (Without<Roller>, Without<Camera3d>)>,
    mut camera: Query<&mut Transform, (With<Camera3d>, Without<Roller>, Without<Pallet>)>,
) {
    let t = time.elapsed_secs();
    let run = &mut *run;
    run.scene.snapshot(t, &mut run.snap);
    // The cylinder's axis is +Y; lay it along local +Z, spin it about that axis, then turn it to the conveyor.
    let lay = Quat::from_rotation_x(std::f32::consts::FRAC_PI_2);
    for (Roller(i), mut tf) in &mut rollers {
        if let Some(p) = run.snap.rollers.get(*i) {
            tf.translation = Vec3::from_array(p.pos);
            tf.rotation = Quat::from_rotation_y(p.yaw) * Quat::from_rotation_z(p.spin) * lay;
        }
    }
    for (Pallet(i), mut tf) in &mut pallets {
        if let Some(p) = run.snap.pallets.get(*i) {
            tf.translation = Vec3::from_array(p.pos);
            tf.rotation = Quat::from_rotation_y(p.yaw);
        }
    }
    let period = run.args.bench.unwrap_or(ORBIT);
    let since = run.measuring_since.map(|s| s.elapsed().as_secs_f32()).unwrap_or(0.0);
    let (eye, target) = run.scene.camera(since, period);
    for mut tf in &mut camera {
        *tf = Transform::from_translation(Vec3::from_array(eye)).looking_at(Vec3::from_array(target), Vec3::Y);
    }
}

/// Notes when the catalogue pallet has loaded, and its triangles.
fn watch_glb(
    model: Option<Res<PalletModel>>,
    asset_server: Res<AssetServer>,
    meshes: Res<Assets<Mesh>>,
    mut run: ResMut<Run>,
) {
    let Some(model) = model else { return };
    if run.glb_ms.is_some() || !asset_server.is_loaded_with_dependencies(model.0.id()) {
        return;
    }
    run.glb_ms = Some(run.glb_requested.elapsed().as_secs_f64() * 1000.0);
    // The model's meshes are the only ones the asset server loaded from a file.
    run.pallet_triangles = meshes
        .iter()
        .filter(|(id, _)| asset_server.get_path(*id).is_some())
        .map(|(_, m)| m.indices().map(|i| i.len() / 3).unwrap_or(0))
        .sum();
}

/// Records each frame's time once the scene is ready, and at the end writes the result and quits.
fn measure(mut run: ResMut<Run>, adapter: Option<Res<RenderAdapterInfo>>, mut exit: MessageWriter<AppExit>) {
    let now = Instant::now();
    if run.first_frame_ms.is_none() {
        run.first_frame_ms = Some(run.started.elapsed().as_secs_f64() * 1000.0);
    }
    let ready = run.glb.is_none() || run.glb_ms.is_some();
    if !ready || run.done {
        return;
    }
    if run.measuring_since.is_none() {
        run.measuring_since = Some(now);
    }
    if let Some(last) = run.last_frame {
        run.frames.push(now - last);
    }
    run.last_frame = Some(now);
    let Some(secs) = run.args.bench else { return };
    if run.measuring_since.is_some_and(|s| s.elapsed().as_secs_f32() >= secs) {
        run.done = true;
        let (name, backend) = adapter.map(|a| (a.name.clone(), format!("{:?}", a.backend))).unwrap_or_default();
        let result = BenchResult {
            viewer: "bevy".into(),
            adapter: name,
            backend,
            window: WINDOW,
            rollers: run.snap.rollers.len(),
            pallets: run.snap.pallets.len(),
            pallet_model: run.glb.as_ref().map(|p| p.display().to_string()).unwrap_or_else(|| "boxes".into()),
            pallet_triangles: run.pallet_triangles,
            startup_ms: run.first_frame_ms.unwrap_or(0.0),
            glb_load_ms: run.glb_ms.unwrap_or(0.0),
            frames: FrameStats::from_frames(&run.frames),
            notes: vec![
                "Bevy 0.20.0: PBR, cascaded shadows, tonemapping; rollers batched by Bevy (one mesh, one material)"
                    .into(),
            ],
        };
        match result.write(run.args.out.as_deref()) {
            Ok(path) => println!("result written to {}", path.display()),
            Err(e) => eprintln!("could not write the result: {e}"),
        }
        exit.write(AppExit::Success);
    }
}

/// A small panel: what is drawn and how fast.
fn panel(mut contexts: EguiContexts, run: Res<Run>) -> Result {
    let ctx = contexts.ctx_mut()?;
    let recent = &run.frames[run.frames.len().saturating_sub(60)..];
    let fps = if recent.is_empty() {
        0.0
    } else {
        recent.len() as f64 / recent.iter().map(Duration::as_secs_f64).sum::<f64>()
    };
    egui::Window::new("R5b - Bevy").show(ctx, |ui| {
        ui.label(format!("{fps:.0} frames/s"));
        ui.label(format!("{} rollers, {} pallets", run.snap.rollers.len(), run.snap.pallets.len()));
        ui.label(match &run.glb {
            Some(p) => {
                format!("pallet: {}", p.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default())
            }
            None => "pallet: boxes".into(),
        });
        if let Some(secs) = run.args.bench {
            let since = run.measuring_since.map(|s| s.elapsed().as_secs_f32()).unwrap_or(0.0);
            ui.label(format!("measuring {since:.0} / {secs:.0} s"));
        }
    });
    Ok(())
}
