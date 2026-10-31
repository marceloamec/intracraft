//! Spike R5b, the BIMCraft-style viewer: eframe and egui-wgpu at BIMCraft's versions, the scene drawn by the viewer's
//! own pipeline into a texture that egui shows (as BIMCraft's 3D view does), plus instancing for the moving bodies.
//! One sun and an ambient term; no shadows, no anti-aliasing, no textures (a material's picture becomes its mean colour).
//!
//! `r5b-viewer-egui` opens the scene; `--bench 30` measures for 30 s and writes `result-egui.json`.

mod gpu;
mod mesh;

use std::time::{Duration, Instant};

use glam::{Mat4, Vec3};

use gpu::{Gpu, Instance};
use r5b_scene::bench::{Args, BenchResult, FrameStats, WINDOW};
use r5b_scene::{LOAD_SIZE, PALLET_SIZE, PALLETS, ROLLER_LENGTH, ROLLER_RADIUS, Scene, Snapshot};

/// How long the camera takes to circle the scene when not measuring, s.
const ORBIT: f32 = 60.0;
const SUN: [f32; 3] = [0.42, 0.82, 0.38];
const ROLLER_TINT: [f32; 3] = [1.0; 3];
const LOAD_TINT: [f32; 3] = [0.72, 0.55, 0.33];
const BOX_PALLET_TINT: [f32; 3] = [0.6, 0.45, 0.28];
/// Height of the catalogue pallet, m (`asset.pallets.pallet_a01`).
const GLB_PALLET_HEIGHT: f32 = 0.209;

fn main() -> eframe::Result {
    let started = Instant::now();
    let args = match Args::parse(std::env::args()) {
        Ok(args) => args,
        Err(e) => {
            eprintln!("{e}\nusage: r5b-viewer-egui [--bench SECONDS] [--glb FILE] [--out FILE]");
            std::process::exit(2);
        }
    };
    let mut options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("R5b - BIMCraft-style viewer (egui + wgpu)")
            .with_inner_size([WINDOW[0] as f32, WINDOW[1] as f32]),
        ..Default::default()
    };
    if args.bench.is_some() {
        options.wgpu_options.surface.present_mode = eframe::egui_wgpu::wgpu::PresentMode::AutoNoVsync;
    }
    eframe::run_native(
        "r5b-viewer-egui",
        options,
        Box::new(move |cc| {
            let state = cc.wgpu_render_state.as_ref().ok_or("this viewer needs wgpu")?;
            Ok(Box::new(Viewer::new(Gpu::new(state), args, started)))
        }),
    )
}

struct Viewer {
    gpu: Gpu,
    scene: Scene,
    snap: Snapshot,
    args: Args,
    started: Instant,
    clock: Instant,
    rollers: usize,
    pallets: usize,
    loads: usize,
    glb: Option<String>,
    glb_ms: f64,
    pallet_triangles: usize,
    notes: Vec<String>,
    first_frame_ms: Option<f64>,
    measuring_since: Option<Instant>,
    last_frame: Option<Instant>,
    frames: Vec<Duration>,
    instances: Vec<Instance>,
    done: bool,
}

impl Viewer {
    fn new(mut gpu: Gpu, args: Args, started: Instant) -> Viewer {
        let scene = Scene::new();
        let unit = mesh::unit_box();
        let statics: Vec<Instance> = scene
            .statics()
            .iter()
            .map(|b| {
                Instance::new(Mat4::from_translation(b.centre.into()) * Mat4::from_rotation_y(b.yaw), b.size, b.color)
            })
            .collect();
        let floor = gpu.add_batch(&unit, statics.len());
        gpu.set_instances(floor, &statics);

        let mut snap = Snapshot::default();
        scene.snapshot(0.0, &mut snap);
        let rollers = gpu.add_batch(&mesh::roller(ROLLER_RADIUS, ROLLER_LENGTH, 16), snap.rollers.len());

        let mut notes = Vec::new();
        let (mut glb, mut glb_ms, mut pallet_mesh) = (None, 0.0, None);
        if let Some(path) = args.glb_or_default() {
            let t = Instant::now();
            match mesh::load_glb(&path) {
                Ok(m) => {
                    glb_ms = t.elapsed().as_secs_f64() * 1000.0;
                    glb = Some(path.display().to_string());
                    pallet_mesh = Some(m);
                }
                Err(e) => notes.push(format!("pallet model not read, boxes instead: {e}")),
            }
        }
        let pallet_triangles = pallet_mesh.as_ref().map_or(12, mesh::MeshData::triangles);
        let pallets = gpu.add_batch(pallet_mesh.as_ref().unwrap_or(&unit), PALLETS);
        let loads = gpu.add_batch(&unit, PALLETS);
        notes.push("eframe/egui-wgpu 0.36.2 (BIMCraft's): one sun, no shadows, no anti-aliasing, no textures".into());
        Viewer {
            gpu,
            scene,
            snap,
            args,
            started,
            clock: Instant::now(),
            rollers,
            pallets,
            loads,
            glb,
            glb_ms,
            pallet_triangles,
            notes,
            first_frame_ms: None,
            measuring_since: None,
            last_frame: None,
            frames: Vec::with_capacity(20_000),
            instances: Vec::with_capacity(2_000),
            done: false,
        }
    }

    /// Writes this frame's snapshot into the instance buffers.
    fn update_instances(&mut self) {
        self.scene.snapshot(self.clock.elapsed().as_secs_f32(), &mut self.snap);
        self.instances.clear();
        self.instances.extend(self.snap.rollers.iter().map(|p| {
            let m = Mat4::from_translation(p.pos.into()) * Mat4::from_rotation_y(p.yaw) * Mat4::from_rotation_z(p.spin);
            Instance::new(m, [1.0; 3], ROLLER_TINT)
        }));
        self.gpu.set_instances(self.rollers, &self.instances);

        let glb = self.glb.is_some();
        let pallet_height = if glb { GLB_PALLET_HEIGHT } else { PALLET_SIZE[1] };
        self.instances.clear();
        self.instances.extend(self.snap.pallets.iter().map(|p| {
            let at = Mat4::from_translation(p.pos.into()) * Mat4::from_rotation_y(p.yaw);
            if glb {
                Instance::new(at, [1.0; 3], [1.0; 3])
            } else {
                Instance::new(at * Mat4::from_translation(Vec3::Y * PALLET_SIZE[1] / 2.0), PALLET_SIZE, BOX_PALLET_TINT)
            }
        }));
        self.gpu.set_instances(self.pallets, &self.instances);

        self.instances.clear();
        self.instances.extend(self.snap.pallets.iter().map(|p| {
            let at = Mat4::from_translation(p.pos.into())
                * Mat4::from_rotation_y(p.yaw)
                * Mat4::from_translation(Vec3::Y * (pallet_height + LOAD_SIZE[1] / 2.0));
            Instance::new(at, LOAD_SIZE, LOAD_TINT)
        }));
        self.gpu.set_instances(self.loads, &self.instances);
    }

    fn record_frame(&mut self) {
        let now = Instant::now();
        if self.first_frame_ms.is_none() {
            self.first_frame_ms = Some(self.started.elapsed().as_secs_f64() * 1000.0);
            self.measuring_since = Some(now);
        }
        if let Some(last) = self.last_frame {
            self.frames.push(now - last);
        }
        self.last_frame = Some(now);
    }

    fn finish(&mut self) {
        self.done = true;
        let result = BenchResult {
            viewer: "egui".into(),
            adapter: self.gpu.adapter.clone(),
            backend: self.gpu.backend.clone(),
            window: WINDOW,
            rollers: self.snap.rollers.len(),
            pallets: self.snap.pallets.len(),
            pallet_model: self.glb.clone().unwrap_or_else(|| "boxes".into()),
            pallet_triangles: self.pallet_triangles,
            startup_ms: self.first_frame_ms.unwrap_or(0.0),
            glb_load_ms: self.glb_ms,
            frames: FrameStats::from_frames(&self.frames),
            notes: self.notes.clone(),
        };
        match result.write(self.args.out.as_deref()) {
            Ok(path) => println!("result written to {}", path.display()),
            Err(e) => eprintln!("could not write the result: {e}"),
        }
    }
}

impl eframe::App for Viewer {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        self.record_frame();
        self.update_instances();
        let since = self.measuring_since.map(|s| s.elapsed().as_secs_f32()).unwrap_or(0.0);
        let (eye, target) = self.scene.camera(since, self.args.bench.unwrap_or(ORBIT));
        // The 3D picture is always WINDOW pixels, whatever the screen's scale, so both viewers draw the same amount.
        let aspect = WINDOW[0] as f32 / WINDOW[1] as f32;
        let view_proj = Mat4::perspective_rh(45_f32.to_radians(), aspect, 0.1, 1000.0)
            * Mat4::look_at_rh(eye.into(), target.into(), Vec3::Y);
        let rect = ui.max_rect();
        if let Some(id) = self.gpu.draw((WINDOW[0], WINDOW[1]), view_proj, SUN) {
            let uv = egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0));
            ui.painter().image(id, rect, uv, egui::Color32::WHITE);
        }

        let recent = &self.frames[self.frames.len().saturating_sub(60)..];
        let fps = if recent.is_empty() {
            0.0
        } else {
            recent.len() as f64 / recent.iter().map(Duration::as_secs_f64).sum::<f64>()
        };
        egui::Window::new("R5b - egui").show(ui.ctx(), |ui| {
            ui.label(format!("{fps:.0} frames/s"));
            ui.label(format!("{} rollers, {} pallets", self.snap.rollers.len(), self.snap.pallets.len()));
            ui.label(format!("pallet: {}", if self.glb.is_some() { "catalogue GLB" } else { "boxes" }));
            ui.label(format!("card: {}", self.gpu.adapter));
            if let Some(secs) = self.args.bench {
                ui.label(format!("measuring {since:.0} / {secs:.0} s"));
            }
        });

        if let Some(secs) = self.args.bench
            && !self.done
            && since >= secs
        {
            self.finish();
            ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
        }
        ui.ctx().request_repaint();
    }
}
