//! IntraCraft's 3D view inside the main window (ADR-0026), beside BIMCraft's floor plan: the scene of the run's layout
//! (`intracraft-scene`), turntables turning and decks rising with the run, the pallets with their loads (the own models of
//! `intracraft-assets`) and each object's state as a plate under it. Drawn on the graphics card the BIMCraft way (its own
//! pipeline into its own targets, shown by egui as a texture), with instancing: one mesh drawn many times. No shadows;
//! IntraCraft's 3D window (Bevy) has them.
//!
//! Drag to orbit, drag with the right button to pan, the wheel to come closer, a double click to see the whole plant.
//! Objects are picked as in the floor plan (ADR-0028): a click selects one (the selection is the plan's), a right click
//! opens its menu, a double click its properties; in the Design module a drag from an object moves it over the floor at
//! its height, snapping as in the plan, and lets it go with BIMCraft's `move`.

use std::collections::{BTreeMap, HashMap};
use std::sync::Arc;

use eframe::egui_wgpu::{self, RenderState, wgpu};
use egui::mutex::RwLock;
use egui::{Color32, Rect, Sense, Ui};
use intracraft_assets::{LoadKind, Material};
use intracraft_library::Library;
use intracraft_library::definition::Value;
use intracraft_library::layout::Layout;
use intracraft_library::params::instance_values;
use intracraft_scene::pose::{PalletShow, mul, rotate};
use intracraft_scene::{Mover, Object, Paint, Part, Shape};

const COLOR: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8Unorm;
const DEPTH: wgpu::TextureFormat = wgpu::TextureFormat::Depth32Float;
/// Samples a pixel: the edges are smoothed.
const SAMPLES: u32 = 4;
/// Bytes of a vertex (position, normal, colour) and of an instance (turn and place, scale, tint).
const VERTEX_BYTES: u64 = 36;
const INSTANCE_BYTES: usize = 96;
/// The sun's direction, toward it.
const SUN: [f32; 3] = [0.45, 0.8, 0.4];
const SKY: wgpu::Color = wgpu::Color { r: 0.80, g: 0.84, b: 0.88, a: 1.0 };
/// The states' colours under the objects: as IntraCraft's 3D window's.
const STATES: [(&str, [f32; 3]); 13] = [
    ("working", [0.20, 0.75, 0.30]),
    ("detecting", [0.20, 0.90, 0.45]),
    ("blocked", [0.90, 0.20, 0.15]),
    ("stopped", [0.95, 0.55, 0.10]),
    ("waiting", [0.95, 0.75, 0.10]),
    ("busy", [0.20, 0.45, 0.95]),
    ("processing", [0.20, 0.45, 0.95]),
    ("lifting", [0.30, 0.60, 0.95]),
    ("lowering", [0.30, 0.60, 0.95]),
    ("turning", [0.55, 0.35, 0.90]),
    ("empty", [0.45, 0.47, 0.50]),
    ("idle", [0.45, 0.47, 0.50]),
    ("clear", [0.45, 0.47, 0.50]),
];

/// What a frame of the run shows: the pallets, the turntables' angles, the decks' heights, how far the parts of the
/// stacker cranes have moved, each object's state.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Picture {
    pub pallets: Vec<PalletShow>,
    pub angles: BTreeMap<String, f64>,
    pub heights: BTreeMap<String, f64>,
    pub shifts: BTreeMap<String, [f64; 3]>,
    pub status: BTreeMap<String, &'static str>,
}

/// How the view lets objects be picked and moved, and what it offers on them.
pub struct Interaction<'a> {
    /// The selected object, by its id in the layout.
    pub selected: Option<String>,
    /// Whether objects move by dragging (the Design module).
    pub editable: bool,
    /// How far a drag of object `id` by `(dx, dz)` (m, floor) really moves it: snapped.
    pub snap: &'a dyn Fn(&str, [f64; 2]) -> [f64; 2],
    /// The items of an object's right-click menu, as the plan's.
    pub menu: &'a [bimcraft_ui_egui::ExtensionAction],
}

/// What the hand did on the view this frame.
#[derive(Clone, Debug, PartialEq)]
pub enum Event3D {
    /// An object picked, or none (a click on the floor).
    Select(Option<String>),
    /// Object `id` dragged by `(dx, dz)` (m, floor, snapped) and let go.
    Moved(String, [f64; 2]),
    /// A double click on an object: its properties.
    Open(String),
    /// An item of the right-click menu, by its id.
    Chosen(String),
}

/// An object being dragged: its id, the floor height the hand slides on, where it took hold and how far it is now.
#[derive(Clone, Debug, PartialEq)]
struct Held {
    id: String,
    height: f64,
    from: [f64; 3],
    by: [f64; 2],
}

/// The tint the selected object takes: mixed toward this, by this much.
const PICKED: ([f32; 3], f32) = ([0.15, 0.55, 1.0], 0.45);

/// The 3D view: its scene, camera and what it put on the graphics card.
#[derive(Default)]
pub struct View3D {
    held: Option<Held>,
    gpu: Option<Gpu>,
    /// The layout the scene was made from, to make it again when the layout changes.
    made_from: Option<Layout>,
    objects: Vec<Object>,
    /// The load each source puts on its pallets, and each pallet type's length and width.
    loads: HashMap<String, Option<LoadKind>>,
    footprints: HashMap<String, (f64, f64)>,
    orbit: Option<Orbit>,
    home: Option<Orbit>,
}

impl View3D {
    /// Draws the view filling `ui`: the scene of `layout`, as `picture` has the run now.
    pub fn show(
        &mut self,
        ui: &mut Ui,
        graphics: Option<&RenderState>,
        layout: &Layout,
        library: &Library,
        picture: &Picture,
        how: &Interaction,
    ) -> Vec<Event3D> {
        let mut events = Vec::new();
        let rect = ui.available_rect_before_wrap();
        let response = ui.allocate_rect(rect, Sense::click_and_drag());
        let Some(graphics) = graphics else {
            ui.painter().text(
                rect.center(),
                egui::Align2::CENTER_CENTER,
                "The 3D view needs the graphics card",
                egui::FontId::proportional(14.0),
                Color32::GRAY,
            );
            return events;
        };
        if self.made_from.as_ref() != Some(layout) {
            self.make_scene(layout, library);
        }
        let home = self.home.unwrap_or_default();
        let orbit = *self.orbit.get_or_insert(home);
        let aspect = f64::from(rect.width() / rect.height().max(1.0));
        let ray_at = |p: egui::Pos2| {
            let x = f64::from((p.x - rect.min.x) / rect.width().max(1.0)) * 2.0 - 1.0;
            let y = 1.0 - f64::from((p.y - rect.min.y) / rect.height().max(1.0)) * 2.0;
            orbit.ray(aspect, (x, y))
        };
        let objects = &self.objects;
        let under = |p: Option<egui::Pos2>| p.and_then(|p| pick(objects, ray_at(p)));
        // A drag from an object moves it (Design); from the floor it orbits.
        if response.drag_started_by(egui::PointerButton::Primary)
            && how.editable
            && let Some(origin) = ui.input(|i| i.pointer.press_origin())
            && let Some((k, t)) = under(Some(origin))
        {
            let (o, d) = ray_at(origin);
            let from = [o[0] + d[0] * t, o[1] + d[1] * t, o[2] + d[2] * t];
            let id = objects[k].instance().to_string();
            self.held = Some(Held { id: id.clone(), height: from[1], from, by: [0.0, 0.0] });
            events.push(Event3D::Select(Some(id)));
        }
        if let Some(held) = &mut self.held {
            if let Some(now) = response.interact_pointer_pos().and_then(|p| on_floor(ray_at(p), held.height)) {
                held.by = (how.snap)(&held.id, [now[0] - held.from[0], now[2] - held.from[2]]);
            }
            if response.drag_stopped() || !response.dragged() {
                if held.by[0].hypot(held.by[1]) > 1e-6 {
                    events.push(Event3D::Moved(held.id.clone(), held.by));
                }
                self.held = None;
            }
            ui.ctx().request_repaint();
        } else if let Some(orbit) = &mut self.orbit {
            orbit.steer(ui, &response);
        }
        if response.double_clicked() {
            match under(response.interact_pointer_pos()) {
                Some((k, _)) => events.push(Event3D::Open(objects[k].instance().to_string())),
                None => self.orbit = Some(home),
            }
        } else if response.clicked() || response.secondary_clicked() {
            let picked = under(response.interact_pointer_pos()).map(|(k, _)| objects[k].instance().to_string());
            // A right click on the floor keeps the selection: the menu is of it.
            if response.clicked() || picked.is_some() {
                events.push(Event3D::Select(picked));
            }
        }
        response.context_menu(|ui| {
            if how.menu.is_empty() {
                ui.label("Right-click an object for what it can do");
            }
            for item in how.menu {
                if ui.add_enabled(item.enabled, egui::Button::new(&item.label)).clicked() {
                    events.push(Event3D::Chosen(item.id.clone()));
                    ui.close();
                }
            }
        });
        let orbit = self.orbit.unwrap_or(home);
        let gpu = self.gpu.get_or_insert_with(|| Gpu::new(graphics));
        let held = self.held.as_ref().map(|h| (h.id.as_str(), h.by));
        let mark = (how.selected.as_deref(), held);
        let instances = instances(&self.objects, picture, &self.loads, &self.footprints, gpu, library, mark);
        for (batch, list) in instances {
            gpu.set_instances(batch, &list);
        }
        let pixels = ui.ctx().pixels_per_point();
        let size = ((rect.width() * pixels).round() as u32, (rect.height() * pixels).round() as u32);
        let view_proj = orbit.view_proj(rect.width() / rect.height().max(1.0));
        if let Some(texture) = gpu.draw(size, view_proj) {
            let uv = Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0));
            ui.painter().image(texture, rect, uv, Color32::WHITE);
        }
        let hint = if how.editable {
            "3D · drag an object: move it · drag the floor: orbit · right drag: pan · wheel: zoom · right click: menu · double click: properties"
        } else {
            "3D · drag: orbit · right drag: pan · wheel: zoom · right click: menu · double click: properties, or the whole plant"
        };
        ui.painter().text(
            rect.left_top() + egui::vec2(8.0, 6.0),
            egui::Align2::LEFT_TOP,
            hint,
            egui::FontId::proportional(11.0),
            Color32::from_gray(60),
        );
        events
    }

    fn make_scene(&mut self, layout: &Layout, library: &Library) {
        self.objects = intracraft_scene::scene(layout, library);
        let (lo, hi) = intracraft_scene::bounds(&self.objects).unwrap_or(([-5.0, 0.0, -5.0], [5.0, 2.0, 5.0]));
        let home = Orbit::framing(lo, hi);
        // A new layout keeps the camera where the user put it, unless there was none.
        if self.orbit.is_none() || self.made_from.is_none() {
            self.orbit = Some(home);
        }
        self.home = Some(home);
        self.loads = layout
            .instances
            .iter()
            // A storage aisle's initial stock carries a block load, as a source's default.
            .filter(|i| i.definition_id == "flow.source" || i.definition_id == "storage.aisle")
            .filter_map(|i| {
                let v = instance_values(library.get(&i.definition_id)?, &i.params);
                let text = |k: &str| v.get(k).and_then(Value::text).unwrap_or_default().to_string();
                Some((i.id.clone(), LoadKind::of_source(&text("loadKind"), &text("loadAsset"))))
            })
            .collect();
        self.footprints = library.reference.pallet_types.iter().map(|t| (t.id.clone(), (t.length, t.width))).collect();
        self.made_from = Some(layout.clone());
    }
}

/// Every instance of the frame, by batch: the parts (fixed, and moving with their platforms), the plates of the states,
/// the floor, the pallets and their loads.
fn instances(
    objects: &[Object],
    picture: &Picture,
    loads: &HashMap<String, Option<LoadKind>>,
    footprints: &HashMap<String, (f64, f64)>,
    gpu: &mut Gpu,
    library: &Library,
    (selected, held): (Option<&str>, Option<(&str, [f64; 2])>),
) -> BTreeMap<usize, Vec<u8>> {
    let mut out: BTreeMap<usize, Vec<u8>> = BTreeMap::new();
    let (cube, roller, post) = (gpu.unit(Unit::Box), gpu.unit(Unit::Roller), gpu.unit(Unit::Post));
    let mut push = |batch: usize, model: Mat, scale: [f64; 3], tint: [f32; 3]| instance(out.entry(batch).or_default(), model, scale, tint);
    let (lo, hi) = intracraft_scene::bounds(objects).unwrap_or(([-5.0, 0.0, -5.0], [5.0, 2.0, 5.0]));
    let size = (hi[0] - lo[0]).max(hi[2] - lo[2]) + 20.0;
    let floor = translation([(lo[0] + hi[0]) / 2.0, -0.011, (lo[2] + hi[2]) / 2.0]);
    push(cube, floor, [size, 0.02, size], [0.62, 0.63, 0.62]);
    for o in objects {
        // The object being dragged is drawn where it would land; the selected one tinted.
        let shift = match held {
            Some((id, by)) if id == o.instance() => translation([by[0], 0.0, by[1]]),
            _ => IDENTITY,
        };
        let picked = selected == Some(o.instance()) || held.is_some_and(|(id, _)| id == o.instance());
        let mover = mat_mul(shift, mover_matrix(o, picture));
        for (part, model) in o.fixed.iter().map(|p| (p, shift)).chain(o.moving.iter().map(|p| (p, mover))) {
            let (batch, scale) = match part.shape {
                Shape::Box(s) => (cube, s),
                Shape::Roller { radius, length } => (roller, [2.0 * radius, 2.0 * radius, length]),
                Shape::Post { radius, height } => (post, [2.0 * radius, height, 2.0 * radius]),
            };
            let colour = paint(part.paint);
            let colour = if picked { std::array::from_fn(|i| colour[i] + (PICKED.0[i] - colour[i]) * PICKED.1) } else { colour };
            push(batch, mat_mul(model, part_matrix(part)), scale, colour);
        }
        if let (Some(state), Some((a, b))) = (picture.status.get(&o.id), o.bounds())
            && let Some((_, color)) = STATES.iter().find(|(s, _)| s == state)
        {
            let at = [(a[0] + b[0]) / 2.0, a[1].max(0.0) + 0.003, (a[2] + b[2]) / 2.0];
            push(cube, translation(at), [b[0] - a[0] + 0.1, 0.004, b[2] - a[2] + 0.1], *color);
        }
    }
    for p in &picture.pallets {
        let model = mat_mul(translation(p.centre), rotation(p.rotation));
        if let Some(batch) = gpu.pallet(&p.type_id, library) {
            push(batch, model, [1.0; 3], [1.0; 3]);
        }
        let kind = loads.get(&p.source).copied().flatten().filter(|_| p.load_height > 0.0);
        let kind = kind.map(|k| if p.wrapped { k.wrapped() } else { k });
        if let (Some(kind), Some(&footprint)) = (kind, footprints.get(&p.type_id)) {
            let batch = gpu.load(kind, &p.type_id, footprint, p.load_height);
            push(batch, mat_mul(model, translation([0.0, p.height, 0.0])), [1.0; 3], [1.0; 3]);
        }
    }
    out
}

/// How an object's moving parts stand now: turned about the pivot by the turntable's angle, or raised by the deck's
/// height above its rest.
fn mover_matrix(o: &Object, picture: &Picture) -> Mat {
    match o.mover {
        Mover::Turn { pivot } => {
            let angle = picture.angles.get(&o.id).copied().unwrap_or(0.0);
            let back = translation([-pivot[0], -pivot[1], -pivot[2]]);
            mat_mul(translation(pivot), mat_mul(rotation(intracraft_scene::pose::about_y(angle)), back))
        }
        Mover::Lift { rest } => translation([0.0, picture.heights.get(&o.id).copied().unwrap_or(rest) - rest, 0.0]),
        Mover::Shift => translation(picture.shifts.get(&o.id).copied().unwrap_or_default()),
        Mover::None => IDENTITY,
    }
}

/// A part's turn and place: its yaw about +y, then its tilt down along its own x. A roller's unit mesh lies along z
/// already, as a roller does.
fn part_matrix(p: &Part) -> Mat {
    let turn = mul(intracraft_scene::pose::about_y(p.yaw), intracraft_scene::pose::about_z(-p.tilt));
    mat_mul(translation(p.centre), rotation(turn))
}

fn paint(p: Paint) -> [f32; 3] {
    p.color().map(|c| f32::from(c) / 255.0)
}

/// Writes an instance: its matrix's columns, its scale, its tint.
fn instance(out: &mut Vec<u8>, model: Mat, scale: [f64; 3], tint: [f32; 3]) {
    for column in model {
        for v in column {
            out.extend_from_slice(&(v as f32).to_le_bytes());
        }
    }
    for v in [scale[0] as f32, scale[1] as f32, scale[2] as f32, 0.0, tint[0], tint[1], tint[2], 1.0] {
        out.extend_from_slice(&v.to_le_bytes());
    }
}

// ── Matrices, column-major ─────────────────────────────────────────────────────────────────────────────────────────────

type Mat = [[f64; 4]; 4];

const IDENTITY: Mat = [[1.0, 0.0, 0.0, 0.0], [0.0, 1.0, 0.0, 0.0], [0.0, 0.0, 1.0, 0.0], [0.0, 0.0, 0.0, 1.0]];

fn translation(t: [f64; 3]) -> Mat {
    let mut m = IDENTITY;
    m[3] = [t[0], t[1], t[2], 1.0];
    m
}

fn rotation(q: [f64; 4]) -> Mat {
    let x = rotate(q, [1.0, 0.0, 0.0]);
    let y = rotate(q, [0.0, 1.0, 0.0]);
    let z = rotate(q, [0.0, 0.0, 1.0]);
    [[x[0], x[1], x[2], 0.0], [y[0], y[1], y[2], 0.0], [z[0], z[1], z[2], 0.0], [0.0, 0.0, 0.0, 1.0]]
}

fn mat_mul(a: Mat, b: Mat) -> Mat {
    std::array::from_fn(|c| std::array::from_fn(|r| (0..4).map(|k| a[k][r] * b[c][k]).sum()))
}

// ── Camera ─────────────────────────────────────────────────────────────────────────────────────────────────────────────

/// Where the camera looks from: the point it circles, its turn and height angles (rad) and its distance (m).
#[derive(Clone, Copy, Debug, PartialEq)]
struct Orbit {
    target: [f64; 3],
    yaw: f64,
    pitch: f64,
    distance: f64,
}

impl Default for Orbit {
    fn default() -> Self {
        Orbit::framing([-5.0, 0.0, -5.0], [5.0, 2.0, 5.0])
    }
}

impl Orbit {
    /// The box `(lo, hi)` seen from the south-east, from above.
    fn framing(lo: [f64; 3], hi: [f64; 3]) -> Orbit {
        let target = [(lo[0] + hi[0]) / 2.0, (lo[1] + hi[1]) / 4.0, (lo[2] + hi[2]) / 2.0];
        let span = (hi[0] - lo[0]).max(hi[2] - lo[2]).max(hi[1] - lo[1]).max(3.0);
        // A tall plant (a high-bay rack) is seen from further away than its length alone asks.
        let tall = (hi[1] - lo[1] - 3.0).max(0.0);
        Orbit { target, yaw: 0.75, pitch: 0.55, distance: span * 1.1 + tall * 1.2 + 2.0 }
    }

    fn eye(&self) -> [f64; 3] {
        let (sy, cy) = self.yaw.sin_cos();
        let (sp, cp) = self.pitch.sin_cos();
        let t = self.target;
        [t[0] + cp * sy * self.distance, t[1] + sp * self.distance, t[2] + cp * cy * self.distance]
    }

    /// Orbits on a drag, pans on a right drag, comes closer by the wheel.
    fn steer(&mut self, ui: &Ui, r: &egui::Response) {
        let d = r.drag_delta();
        if r.dragged_by(egui::PointerButton::Primary) {
            self.yaw -= f64::from(d.x) * 0.008;
            self.pitch = (self.pitch + f64::from(d.y) * 0.008).clamp(0.05, 1.5);
        } else if r.dragged_by(egui::PointerButton::Secondary) || r.dragged_by(egui::PointerButton::Middle) {
            let (sy, cy) = self.yaw.sin_cos();
            let k = self.distance * 0.002;
            let (dx, dy) = (f64::from(d.x) * k, f64::from(d.y) * k);
            // Right is (cos yaw, 0, −sin yaw); away from the camera on the floor is (−sin yaw, 0, −cos yaw).
            self.target[0] += -cy * dx - sy * dy;
            self.target[2] += sy * dx - cy * dy;
        }
        if r.hovered() {
            let scroll = ui.input(|i| i.smooth_scroll_delta.y);
            self.distance = (self.distance * 0.9985f64.powf(f64::from(scroll))).clamp(1.0, 500.0);
        }
        if r.dragged() || r.hovered() {
            ui.ctx().request_repaint();
        }
    }

    /// The ray from the eye through a point of the picture, in normalised coordinates (−1 to 1, y up): its origin and
    /// direction (unit). As `view_proj` sees: a vertical field of view of 45°.
    fn ray(&self, aspect: f64, (x, y): (f64, f64)) -> ([f64; 3], [f64; 3]) {
        let eye = self.eye();
        let f = normalize([self.target[0] - eye[0], self.target[1] - eye[1], self.target[2] - eye[2]]);
        let s = normalize(cross(f, [0.0, 1.0, 0.0]));
        let u = cross(s, f);
        let k = (std::f64::consts::FRAC_PI_4 / 2.0).tan();
        let (a, b) = (x * k * aspect.max(0.1), y * k);
        (eye, normalize(std::array::from_fn(|i| f[i] + s[i] * a + u[i] * b)))
    }

    /// The camera's view and perspective (right-handed, depth 0 to 1), as columns of f32.
    fn view_proj(&self, aspect: f32) -> [[f32; 4]; 4] {
        let (eye, target) = (self.eye(), self.target);
        let f = normalize([target[0] - eye[0], target[1] - eye[1], target[2] - eye[2]]);
        let s = normalize(cross(f, [0.0, 1.0, 0.0]));
        let u = cross(s, f);
        let dot = |a: [f64; 3], b: [f64; 3]| a[0] * b[0] + a[1] * b[1] + a[2] * b[2];
        let view: Mat =
            [[s[0], u[0], -f[0], 0.0], [s[1], u[1], -f[1], 0.0], [s[2], u[2], -f[2], 0.0], [-dot(s, eye), -dot(u, eye), dot(f, eye), 1.0]];
        let (near, far) = ((self.distance * 0.01).max(0.05), self.distance * 20.0 + 100.0);
        let g = 1.0 / (std::f64::consts::FRAC_PI_4 / 2.0).tan();
        let proj: Mat = [
            [g / f64::from(aspect.max(0.1)), 0.0, 0.0, 0.0],
            [0.0, g, 0.0, 0.0],
            [0.0, 0.0, far / (near - far), -1.0],
            [0.0, 0.0, near * far / (near - far), 0.0],
        ];
        mat_mul(proj, view).map(|c| c.map(|v| v as f32))
    }
}

/// The object a ray meets first (by the box round its parts) and how far along the ray.
fn pick(objects: &[Object], (origin, dir): ([f64; 3], [f64; 3])) -> Option<(usize, f64)> {
    objects
        .iter()
        .enumerate()
        .filter_map(|(k, o)| o.bounds().and_then(|(lo, hi)| meets_box(origin, dir, lo, hi)).map(|t| (k, t)))
        .min_by(|a, b| a.1.total_cmp(&b.1))
}

/// How far along a ray it enters box `(lo, hi)`, if it does (slabs).
fn meets_box(origin: [f64; 3], dir: [f64; 3], lo: [f64; 3], hi: [f64; 3]) -> Option<f64> {
    let (mut near, mut far) = (0.0f64, f64::INFINITY);
    for i in 0..3 {
        if dir[i].abs() < 1e-12 {
            if origin[i] < lo[i] || origin[i] > hi[i] {
                return None;
            }
            continue;
        }
        let (a, b) = ((lo[i] - origin[i]) / dir[i], (hi[i] - origin[i]) / dir[i]);
        near = near.max(a.min(b));
        far = far.min(a.max(b));
    }
    (near <= far).then_some(near)
}

/// Where a ray meets the level floor at `height`, ahead of it.
fn on_floor((origin, dir): ([f64; 3], [f64; 3]), height: f64) -> Option<[f64; 3]> {
    if dir[1].abs() < 1e-9 {
        return None;
    }
    let t = (height - origin[1]) / dir[1];
    (t > 0.0).then(|| std::array::from_fn(|i| origin[i] + dir[i] * t))
}

fn cross(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [a[1] * b[2] - a[2] * b[1], a[2] * b[0] - a[0] * b[2], a[0] * b[1] - a[1] * b[0]]
}

fn normalize(v: [f64; 3]) -> [f64; 3] {
    let n = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt().max(1e-12);
    v.map(|c| c / n)
}

// ── Meshes ─────────────────────────────────────────────────────────────────────────────────────────────────────────────

/// The unit meshes the parts are scaled from.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
enum Unit {
    /// A box from −0.5 to 0.5.
    Box,
    /// A cylinder of radius 0.5 along z, from −0.5 to 0.5.
    Roller,
    /// The same along y.
    Post,
}

/// Triangles with their normals and colours (sRGB), as the card takes them.
#[derive(Default)]
struct MeshData {
    vertices: Vec<u8>,
    indices: Vec<u32>,
    count: u32,
}

impl MeshData {
    fn vertex(&mut self, p: [f32; 3], n: [f32; 3], c: [f32; 3]) {
        for v in p.into_iter().chain(n).chain(c) {
            self.vertices.extend_from_slice(&v.to_le_bytes());
        }
        self.count += 1;
    }

    fn unit(u: Unit) -> MeshData {
        let mut m = MeshData::default();
        let white = [1.0; 3];
        match u {
            Unit::Box => {
                let model = intracraft_assets::Model {
                    name: String::new(),
                    pieces: vec![intracraft_assets::Piece::cuboid(Material::Carton, [0.0; 3], [1.0; 3])],
                };
                m.add_model(&model, Some(white));
            }
            Unit::Post => {
                let model = intracraft_assets::Model {
                    name: String::new(),
                    pieces: vec![intracraft_assets::Piece::cylinder(Material::Carton, [0.0; 3], 0.5, 1.0)],
                };
                m.add_model(&model, Some(white));
            }
            Unit::Roller => {
                // A post laid along z: (x, y, z) → (x, −z, y) keeps it wound outward.
                let mut post = MeshData::default();
                let model = intracraft_assets::Model {
                    name: String::new(),
                    pieces: vec![intracraft_assets::Piece::cylinder(Material::Carton, [0.0; 3], 0.5, 1.0)],
                };
                post.add_model(&model, Some(white));
                for k in 0..post.count as usize {
                    let at = |i: usize| f32::from_le_bytes(post.vertices[k * 36 + i * 4..k * 36 + i * 4 + 4].try_into().unwrap_or([0; 4]));
                    m.vertex([at(0), -at(2), at(1)], [at(3), -at(5), at(4)], white);
                }
                m.indices = post.indices;
            }
        }
        m
    }

    /// A model's opaque triangles in its materials' colours (or all in `color`); film is left out.
    fn add_model(&mut self, model: &intracraft_assets::Model, color: Option<[f32; 3]>) {
        for (material, p) in intracraft_assets::glb::primitives(model) {
            if material.blends() {
                continue;
            }
            let [r, g, b, _] = material.color();
            // The materials are linear (physically based); the picture is shown as is, so they are put in sRGB.
            let c = color.unwrap_or([r, g, b].map(|v| v.max(0.0).powf(1.0 / 2.2)));
            let base = self.count;
            for (pos, n) in p.positions.iter().zip(&p.normals) {
                self.vertex(*pos, *n, c);
            }
            self.indices.extend(p.indices.iter().map(|i| i + base));
        }
    }
}

// ── The card ───────────────────────────────────────────────────────────────────────────────────────────────────────────

/// A mesh on the card and room for its instances.
struct Batch {
    vertices: wgpu::Buffer,
    indices: wgpu::Buffer,
    index_count: u32,
    instances: wgpu::Buffer,
    capacity: usize,
    count: u32,
}

struct Targets {
    size: (u32, u32),
    msaa: wgpu::TextureView,
    color: wgpu::TextureView,
    depth: wgpu::TextureView,
}

struct Gpu {
    device: wgpu::Device,
    queue: wgpu::Queue,
    renderer: Arc<RwLock<egui_wgpu::Renderer>>,
    pipeline: wgpu::RenderPipeline,
    globals: wgpu::Buffer,
    bind: wgpu::BindGroup,
    batches: Vec<Batch>,
    units: HashMap<Unit, usize>,
    pallets: HashMap<String, usize>,
    loads: HashMap<String, usize>,
    targets: Option<Targets>,
    texture: Option<egui::TextureId>,
}

impl Gpu {
    fn new(state: &RenderState) -> Gpu {
        let device = state.device.clone();
        let module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("intracraft-3d"),
            source: wgpu::ShaderSource::Wgsl(include_str!("view3d.wgsl").into()),
        });
        let group = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("intracraft-3d-globals"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer { ty: wgpu::BufferBindingType::Uniform, has_dynamic_offset: false, min_binding_size: None },
                count: None,
            }],
        });
        let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("intracraft-3d"),
            bind_group_layouts: &[Some(&group)],
            immediate_size: 0,
        });
        let vertex_attributes = wgpu::vertex_attr_array![0 => Float32x3, 1 => Float32x3, 2 => Float32x3];
        let instance_attributes =
            wgpu::vertex_attr_array![3 => Float32x4, 4 => Float32x4, 5 => Float32x4, 6 => Float32x4, 7 => Float32x4, 8 => Float32x4];
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("intracraft-3d"),
            layout: Some(&layout),
            vertex: wgpu::VertexState {
                module: &module,
                entry_point: Some("vs_main"),
                compilation_options: Default::default(),
                buffers: &[
                    Some(wgpu::VertexBufferLayout {
                        array_stride: VERTEX_BYTES,
                        step_mode: wgpu::VertexStepMode::Vertex,
                        attributes: &vertex_attributes,
                    }),
                    Some(wgpu::VertexBufferLayout {
                        array_stride: INSTANCE_BYTES as u64,
                        step_mode: wgpu::VertexStepMode::Instance,
                        attributes: &instance_attributes,
                    }),
                ],
            },
            primitive: wgpu::PrimitiveState { cull_mode: Some(wgpu::Face::Back), ..Default::default() },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: DEPTH,
                depth_write_enabled: Some(true),
                depth_compare: Some(wgpu::CompareFunction::Less),
                stencil: Default::default(),
                bias: Default::default(),
            }),
            multisample: wgpu::MultisampleState { count: SAMPLES, ..Default::default() },
            fragment: Some(wgpu::FragmentState {
                module: &module,
                entry_point: Some("fs_main"),
                compilation_options: Default::default(),
                targets: &[Some(wgpu::ColorTargetState { format: COLOR, blend: None, write_mask: wgpu::ColorWrites::ALL })],
            }),
            multiview_mask: None,
            cache: None,
        });
        let globals = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("intracraft-3d-globals"),
            size: 80,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let bind = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("intracraft-3d-globals"),
            layout: &group,
            entries: &[wgpu::BindGroupEntry { binding: 0, resource: globals.as_entire_binding() }],
        });
        Gpu {
            device,
            queue: state.queue.clone(),
            renderer: state.renderer.clone(),
            pipeline,
            globals,
            bind,
            batches: Vec::new(),
            units: HashMap::new(),
            pallets: HashMap::new(),
            loads: HashMap::new(),
            targets: None,
            texture: None,
        }
    }

    fn unit(&mut self, u: Unit) -> usize {
        if let Some(&b) = self.units.get(&u) {
            return b;
        }
        let b = self.add_batch(&MeshData::unit(u));
        self.units.insert(u, b);
        b
    }

    /// The batch of a pallet type's model, made the first time.
    fn pallet(&mut self, type_id: &str, library: &Library) -> Option<usize> {
        if let Some(&b) = self.pallets.get(type_id) {
            return Some(b);
        }
        let t = library.reference.pallet_type(type_id)?;
        let mut mesh = MeshData::default();
        mesh.add_model(&intracraft_assets::pallet(t), None);
        let b = self.add_batch(&mesh);
        self.pallets.insert(type_id.to_string(), b);
        Some(b)
    }

    /// The batch of a load on a pallet type's footprint at its height, made the first time.
    fn load(&mut self, kind: LoadKind, type_id: &str, (length, width): (f64, f64), height: f64) -> usize {
        let key = format!("{}-{type_id}-{height:.3}", kind.name());
        if let Some(&b) = self.loads.get(&key) {
            return b;
        }
        let mut mesh = MeshData::default();
        mesh.add_model(&intracraft_assets::load(kind, length, width, height), None);
        let b = self.add_batch(&mesh);
        self.loads.insert(key, b);
        b
    }

    fn add_batch(&mut self, mesh: &MeshData) -> usize {
        let indices: Vec<u8> = mesh.indices.iter().flat_map(|i| i.to_le_bytes()).collect();
        let buffer = |label, bytes: &[u8], usage| {
            let b = self.device.create_buffer(&wgpu::BufferDescriptor {
                label: Some(label),
                size: (bytes.len() as u64).max(16).next_multiple_of(4),
                usage: usage | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });
            self.queue.write_buffer(&b, 0, bytes);
            b
        };
        let vertices = buffer("intracraft-3d-vertices", &mesh.vertices, wgpu::BufferUsages::VERTEX);
        let index_buffer = buffer("intracraft-3d-indices", &indices, wgpu::BufferUsages::INDEX);
        let instances = self.instance_buffer(64);
        self.batches.push(Batch {
            vertices,
            indices: index_buffer,
            index_count: mesh.indices.len() as u32,
            instances,
            capacity: 64,
            count: 0,
        });
        self.batches.len() - 1
    }

    fn instance_buffer(&self, capacity: usize) -> wgpu::Buffer {
        self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("intracraft-3d-instances"),
            size: (capacity.max(1) * INSTANCE_BYTES) as u64,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        })
    }

    /// A batch's instances this frame, the buffer grown when they don't fit; the batches given none are not drawn.
    fn set_instances(&mut self, batch: usize, bytes: &[u8]) {
        let n = bytes.len() / INSTANCE_BYTES;
        if self.batches.get(batch).is_some_and(|b| n > b.capacity) {
            let capacity = n.next_power_of_two();
            let buffer = self.instance_buffer(capacity);
            if let Some(b) = self.batches.get_mut(batch) {
                (b.instances, b.capacity) = (buffer, capacity);
            }
        }
        if let Some(b) = self.batches.get_mut(batch) {
            self.queue.write_buffer(&b.instances, 0, bytes);
            b.count = n as u32;
        }
    }

    /// Draws every batch at `size` pixels and returns the picture as an egui texture; then forgets the instances.
    fn draw(&mut self, size: (u32, u32), view_proj: [[f32; 4]; 4]) -> Option<egui::TextureId> {
        let size = (size.0.max(1), size.1.max(1));
        if self.targets.as_ref().is_none_or(|t| t.size != size) {
            self.targets = Some(self.make_targets(size));
            if let (Some(id), Some(t)) = (self.texture, &self.targets) {
                self.renderer.write().update_egui_texture_from_wgpu_texture(&self.device, &t.color, wgpu::FilterMode::Linear, id);
            }
        }
        let targets = self.targets.as_ref()?;
        let mut globals: Vec<u8> = view_proj.iter().flatten().flat_map(|v| v.to_le_bytes()).collect();
        let sun = normalize(SUN.map(f64::from)).map(|v| v as f32);
        globals.extend([sun[0], sun[1], sun[2], 0.0].iter().flat_map(|v| v.to_le_bytes()));
        self.queue.write_buffer(&self.globals, 0, &globals);
        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("intracraft-3d") });
        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("intracraft-3d"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &targets.msaa,
                    depth_slice: None,
                    resolve_target: Some(&targets.color),
                    ops: wgpu::Operations { load: wgpu::LoadOp::Clear(SKY), store: wgpu::StoreOp::Discard },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &targets.depth,
                    depth_ops: Some(wgpu::Operations { load: wgpu::LoadOp::Clear(1.0), store: wgpu::StoreOp::Discard }),
                    stencil_ops: None,
                }),
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });
            pass.set_pipeline(&self.pipeline);
            pass.set_bind_group(0, &self.bind, &[]);
            for b in self.batches.iter().filter(|b| b.count > 0) {
                pass.set_vertex_buffer(0, b.vertices.slice(..));
                pass.set_vertex_buffer(1, b.instances.slice(..));
                pass.set_index_buffer(b.indices.slice(..), wgpu::IndexFormat::Uint32);
                pass.draw_indexed(0..b.index_count, 0, 0..b.count);
            }
        }
        self.queue.submit([encoder.finish()]);
        for b in &mut self.batches {
            b.count = 0;
        }
        if self.texture.is_none() {
            self.texture = Some(self.renderer.write().register_native_texture(&self.device, &targets.color, wgpu::FilterMode::Linear));
        }
        self.texture
    }

    fn make_targets(&self, size: (u32, u32)) -> Targets {
        let texture = |label, format, usage, samples| {
            self.device
                .create_texture(&wgpu::TextureDescriptor {
                    label: Some(label),
                    size: wgpu::Extent3d { width: size.0, height: size.1, depth_or_array_layers: 1 },
                    mip_level_count: 1,
                    sample_count: samples,
                    dimension: wgpu::TextureDimension::D2,
                    format,
                    usage,
                    view_formats: &[],
                })
                .create_view(&wgpu::TextureViewDescriptor::default())
        };
        Targets {
            size,
            msaa: texture("intracraft-3d-msaa", COLOR, wgpu::TextureUsages::RENDER_ATTACHMENT, SAMPLES),
            color: texture("intracraft-3d-color", COLOR, wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING, 1),
            depth: texture("intracraft-3d-depth", DEPTH, wgpu::TextureUsages::RENDER_ATTACHMENT, SAMPLES),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn apply(m: Mat, p: [f64; 3]) -> [f64; 3] {
        std::array::from_fn(|r| m[0][r] * p[0] + m[1][r] * p[1] + m[2][r] * p[2] + m[3][r])
    }

    #[test]
    fn a_turntable_s_platform_turns_about_its_pivot_and_a_lift_deck_rises_by_its_height() {
        let turn = Object {
            id: "t".into(),
            name: String::new(),
            definition: String::new(),
            fixed: Vec::new(),
            moving: Vec::new(),
            mover: Mover::Turn { pivot: [2.0, 0.0, 0.0] },
        };
        let picture = Picture { angles: BTreeMap::from([("t".to_string(), std::f64::consts::FRAC_PI_2)]), ..Picture::default() };
        let p = apply(mover_matrix(&turn, &picture), [3.0, 0.6, 0.0]);
        assert!((p[0] - 2.0).abs() < 1e-9 && (p[2] + 1.0).abs() < 1e-9, "{p:?}");
        let lift = Object { id: "l".into(), mover: Mover::Lift { rest: 0.6 }, ..turn };
        let picture = Picture { heights: BTreeMap::from([("l".to_string(), 4.6)]), ..Picture::default() };
        assert!((apply(mover_matrix(&lift, &picture), [0.0, 0.6, 0.0])[1] - 4.6).abs() < 1e-9);
    }

    #[test]
    fn a_part_of_a_crane_moves_by_its_shift_and_belongs_to_its_aisle() {
        let forks = Object {
            id: "a1#forks".into(),
            name: String::new(),
            definition: "storage.aisle".into(),
            fixed: Vec::new(),
            moving: Vec::new(),
            mover: Mover::Shift,
        };
        assert_eq!(forks.instance(), "a1");
        let picture = Picture { shifts: BTreeMap::from([("a1#forks".to_string(), [12.0, 5.4, -1.4])]), ..Picture::default() };
        let p = apply(mover_matrix(&forks, &picture), [-0.4, 0.6, 0.0]);
        assert!((p[0] - 11.6).abs() < 1e-9 && (p[1] - 6.0).abs() < 1e-9 && (p[2] + 1.4).abs() < 1e-9, "{p:?}");
        assert_eq!(apply(mover_matrix(&forks, &Picture::default()), [1.0, 2.0, 3.0]), [1.0, 2.0, 3.0]);
    }

    #[test]
    fn the_camera_sees_its_target_in_the_middle_of_the_picture() {
        let o = Orbit::framing([0.0, 0.0, 0.0], [10.0, 2.0, 4.0]);
        let m = o.view_proj(1.6);
        let t = o.target.map(|v| v as f32);
        let clip: [f32; 4] = std::array::from_fn(|r| m[0][r] * t[0] + m[1][r] * t[1] + m[2][r] * t[2] + m[3][r]);
        assert!((clip[0] / clip[3]).abs() < 1e-4 && (clip[1] / clip[3]).abs() < 1e-4, "{clip:?}");
        assert!(clip[2] / clip[3] > 0.0 && clip[2] / clip[3] < 1.0);
    }

    #[test]
    fn the_ray_through_the_middle_of_the_picture_picks_what_the_camera_looks_at_and_finds_the_floor() {
        let o = Orbit::framing([0.0, 0.0, 0.0], [10.0, 2.0, 4.0]);
        let part = Part { shape: Shape::Box([1.0, 1.0, 1.0]), centre: o.target, yaw: 0.0, tilt: 0.0, paint: Paint::Frame };
        let near = Object {
            id: "near".into(),
            name: String::new(),
            definition: String::new(),
            fixed: vec![part],
            moving: Vec::new(),
            mover: Mover::None,
        };
        let aside =
            Object { id: "aside".into(), fixed: vec![Part { centre: [o.target[0] + 30.0, 0.5, o.target[2]], ..part }], ..near.clone() };
        let ray = o.ray(1.6, (0.0, 0.0));
        assert_eq!(pick(&[aside, near.clone()], ray).map(|(k, _)| k), Some(1));
        // Off to the side of the picture, the ray misses it.
        assert_eq!(pick(&[near], o.ray(1.6, (0.9, 0.0))), None);
        // The middle of the picture lies on the floor at the target's height, where the camera looks.
        let floor = on_floor(ray, o.target[1]).unwrap();
        assert!((0..3).all(|i| (floor[i] - o.target[i]).abs() < 1e-9), "{floor:?}");
        // A ray parallel to the floor meets none.
        assert_eq!(on_floor(([0.0, 1.0, 0.0], [1.0, 0.0, 0.0]), 0.0), None);
    }

    #[test]
    fn the_unit_meshes_are_closed_and_their_instances_are_96_bytes() {
        for u in [Unit::Box, Unit::Roller, Unit::Post] {
            let m = MeshData::unit(u);
            assert_eq!(m.vertices.len() as u64, u64::from(m.count) * VERTEX_BYTES);
            assert!(m.indices.iter().all(|&i| i < m.count) && !m.indices.is_empty());
        }
        let mut out = Vec::new();
        instance(&mut out, IDENTITY, [1.0; 3], [1.0; 3]);
        assert_eq!(out.len(), INSTANCE_BYTES);
    }
}
