//! IntraCraft's window: BIMCraft's own, with IntraCraft's three modules on its ribbon (ADR-0020 §7, A09 §0), through
//! BIMCraft's `Extension` hooks:
//!
//! - Design: BIMCraft whole, and the **Intralogistics** tab with the library's objects to place; beside the views, the
//!   simulation tree and the selected object's attribute table (inherited ✔ / own ✖).
//! - **Discrete simulation**: the run's controls on the ribbon; the pallets moving over BIMCraft's floor plan and 3D view,
//!   each object tinted by its state; the selected object's statistics beside them.
//! - **Physics simulation**: the same over the Rapier run (ADR-0024), its pallets where their bodies are.
//!
//! The window is a thin shell: it reads the model and changes it only through `intracraft_model`'s functions, which run
//! BIMCraft's commands.

use std::collections::BTreeMap;

use bimcraft_core::geom::P2;
use bimcraft_ui_egui::{BimApp, Extension, Sheet, Status, ribbon_button, ribbon_panel};
use egui::{Color32, Grid, Painter, RichText, ScrollArea, Shape, Stroke, Ui};
use intracraft_des::flow::{FlowSimulation, Snapshot, Statistics, simulate};
use intracraft_des::time::{from_seconds, to_seconds};
use intracraft_library::Library;
use intracraft_library::definition::{ParamType, Value};
use intracraft_library::layout::Layout;
use intracraft_model::attributes::{Attribute, Source, attributes, definition_of, shown};
use intracraft_model::tree::simulation_tree;
use intracraft_physics::flow::PhysicsFlow;

use crate::glyphs::Glyph;
use crate::plan::{self, FACES, Point};

/// IntraCraft's ribbon tabs, after BIMCraft's own.
pub const TABS: [&str; 3] = ["Intralogistics", "Discrete simulation", "Physics simulation"];
/// Speeds the discrete run plays at: simulated seconds per real second.
const SPEEDS: [u32; 5] = [1, 10, 100, 1000, 10_000];
/// Speeds the physics run plays at: it is real time at ×1, and a frame never runs more than a second of physics.
const PHYSICS_SPEEDS: [u32; 3] = [1, 2, 5];
const WOOD: Color32 = Color32::from_rgb(181, 136, 84);
const LOAD: Color32 = Color32::from_rgb(196, 170, 120);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Module {
    Design,
    Discrete,
    Physics,
}

impl Module {
    /// The module a ribbon tab of IntraCraft's (or none, BIMCraft's own) is.
    pub fn of(tab: Option<usize>) -> Module {
        match tab {
            Some(1) => Module::Discrete,
            Some(2) => Module::Physics,
            _ => Module::Design,
        }
    }

    /// Its ribbon tab.
    pub fn tab(self) -> Option<usize> {
        match self {
            Module::Design => Some(0),
            Module::Discrete => Some(1),
            Module::Physics => Some(2),
        }
    }
}

/// A discrete run of the layout the model held when it started.
pub struct Run {
    pub sim: FlowSimulation,
    pub layout: Layout,
    pub snapshot: Snapshot,
    pub playing: bool,
    pub speed: f64,
    pub error: Option<String>,
}

impl Run {
    pub fn start(layout: Layout, library: &Library) -> Result<Run, String> {
        let mut sim = simulate(&layout, library, 1)?;
        let snapshot = sim.snapshot();
        Ok(Run { sim, layout, snapshot, playing: false, speed: 10.0, error: None })
    }

    /// Moves the run on by `real` seconds of the window's time at its speed.
    pub fn advance(&mut self, real: f64) {
        if self.error.is_some() {
            return;
        }
        let target = self.sim.now() + from_seconds(real * self.speed).max(1);
        if let Err(e) = self.sim.run_until(target) {
            self.error = Some(e);
            self.playing = false;
        }
        self.snapshot = self.sim.snapshot();
    }

    /// Runs to the next event.
    pub fn step(&mut self) {
        if let Some(t) = self.sim.next_event_time() {
            if let Err(e) = self.sim.run_until(t) {
                self.error = Some(e);
            }
            self.snapshot = self.sim.snapshot();
        }
    }
}

/// A physics run of the layout the model held when it started.
pub struct PhysicsRun {
    pub flow: PhysicsFlow,
    pub layout: Layout,
    pub playing: bool,
    pub speed: f64,
}

impl PhysicsRun {
    pub fn start(layout: Layout, library: &Library) -> Result<PhysicsRun, String> {
        let flow = PhysicsFlow::new(&layout, library, 1)?;
        Ok(PhysicsRun { flow, layout, playing: false, speed: 1.0 })
    }

    /// Moves the run on by `real` seconds of the window's time at its speed (at most one second of physics a frame).
    pub fn advance(&mut self, real: f64) {
        let target = self.flow.now() + from_seconds((real * self.speed).min(1.0)).max(1);
        self.flow.run_until(target);
    }
}

pub struct IntraApp {
    pub bim: BimApp,
    pub sim: Simulation,
}

/// What IntraCraft adds to BIMCraft's window: its library, its runs and what the attribute table holds while typed.
pub struct Simulation {
    pub library: Library,
    pub run: Option<Run>,
    pub physics_run: Option<PhysicsRun>,
    /// Runs start playing as soon as they are made (`--play`).
    pub autoplay: bool,
    /// What is typed in the attribute table, by attribute key, until it is applied.
    typed: BTreeMap<String, String>,
    /// Where the next object placed from the ribbon goes, mm east.
    next_x: f64,
}

impl IntraApp {
    pub fn new(bim: BimApp, library: Library) -> IntraApp {
        IntraApp { bim, sim: Simulation { library, run: None, physics_run: None, autoplay: false, typed: BTreeMap::new(), next_x: 0.0 } }
    }

    /// Opens a module: its ribbon tab.
    pub fn open(&mut self, module: Module) {
        self.bim.pick_extension_tab(module.tab());
    }

    pub fn module(&self) -> Module {
        Module::of(self.bim.extension_tab())
    }

    pub fn show(&mut self, ui: &mut Ui) {
        let dt = f64::from(ui.input(|i| i.stable_dt)).min(0.1);
        let playing = match self.module() {
            Module::Discrete => self.sim.run.as_mut().filter(|r| r.playing).map(|r| r.advance(dt)).is_some(),
            Module::Physics => self.sim.physics_run.as_mut().filter(|r| r.playing).map(|r| r.advance(dt)).is_some(),
            Module::Design => false,
        };
        if playing {
            ui.ctx().request_repaint();
        }
        self.bim.show_with(ui, &mut self.sim);
    }
}

impl Extension for Simulation {
    fn tabs(&self) -> Vec<String> {
        TABS.iter().map(|t| t.to_string()).collect()
    }

    fn ribbon(&mut self, ui: &mut Ui, k: usize, app: &mut BimApp) {
        match Module::of(Some(k)) {
            Module::Design => self.library_tab(ui, app),
            Module::Discrete => self.discrete_tab(ui, app),
            Module::Physics => self.physics_tab(ui, app),
        }
    }

    fn over_view(&mut self, painter: &Painter, app: &BimApp) {
        let three_d = app.sheet == Sheet::View3D;
        if !three_d && !matches!(app.sheet, Sheet::Plan(_)) {
            return;
        }
        let (layout, status, boxes) = match Module::of(app.extension_tab()) {
            Module::Discrete => {
                let Some(run) = &self.run else { return };
                let boxes = run
                    .snapshot
                    .pallets
                    .iter()
                    .filter_map(|p| {
                        let outline = plan::pallet_outline(&run.layout, p)?;
                        let z = plan::ride_height(&run.layout, p, &run.snapshot.heights);
                        Some((outline, z, p.height, p.load_height))
                    })
                    .collect();
                (&run.layout, run.snapshot.status.clone(), boxes)
            }
            Module::Physics => {
                let Some(run) = &self.physics_run else { return };
                let boxes = run
                    .flow
                    .pallets()
                    .into_iter()
                    .map(|p| (plan::body_outline(p.centre, p.yaw, p.size), p.centre[1], p.size[1], p.load_height))
                    .collect();
                (&run.layout, run.flow.status(), boxes)
            }
            Module::Design => return,
        };
        if three_d {
            pallets_3d(painter, app, boxes);
        } else {
            tint_plan(painter, app, layout, &status);
            for (outline, ..) in boxes {
                let pts: Vec<_> = outline.iter().map(|c| app.screen(mm(*c))).collect();
                painter.add(Shape::convex_polygon(pts, WOOD, Stroke::new(1.0, Color32::from_rgb(90, 60, 30))));
            }
        }
    }

    fn panel_title(&self, app: &BimApp) -> Option<String> {
        Some(
            match Module::of(app.extension_tab()) {
                Module::Design => "Simulation objects",
                Module::Discrete => "Discrete run",
                Module::Physics => "Physics run",
            }
            .into(),
        )
    }

    fn panel(&mut self, ui: &mut Ui, app: &mut BimApp) {
        ScrollArea::vertical().show(ui, |ui| match Module::of(app.extension_tab()) {
            Module::Design => {
                self.tree(ui, app);
                ui.separator();
                self.attribute_table(ui, app);
            }
            Module::Discrete => {
                if let Some(run) = &mut self.run {
                    warnings(ui, run.sim.issues.iter().map(|i| i.message.as_str()), run.error.as_deref());
                    statistics_sheet(ui, &run.sim.statistics(), &run.layout, &app.selection);
                }
            }
            Module::Physics => {
                if let Some(run) = &self.physics_run {
                    warnings(ui, run.flow.issues.iter().map(|i| i.message.as_str()), None);
                    statistics_sheet(ui, &run.flow.statistics(), &run.layout, &app.selection);
                }
            }
        });
    }
}

impl Simulation {
    /// The library's objects, by kind, each placed by a click at the next free spot (move it with BIMCraft's tools); the
    /// selected object detached from the library.
    fn library_tab(&mut self, ui: &mut Ui, app: &mut BimApp) {
        let kinds = [
            ("Conveyors", "conveyor.roller conveyor.chain conveyor.gravity"),
            ("Stations", "conveyor.roller_turntable conveyor.chain_turntable conveyor.transfer_rc flow.process_station"),
            ("Lifts", "conveyor.lift_table_manual conveyor.scissor_lift conveyor.elevator"),
            ("Flow", "flow.source flow.sink sensor.photoeye"),
            ("Scenery", "scenery.asset"),
        ];
        for (title, ids) in kinds {
            ribbon_panel(ui, title, |ui| {
                for id in ids.split(' ') {
                    let Some(d) = self.library.get(id) else { continue };
                    let glyph = Glyph::of(id);
                    let tip = format!("Place a {} from the library", d.name.en.to_lowercase());
                    if ribbon_button(ui, &|p, r, c| glyph.paint(p, r, c), &d.name.en, false, &tip).clicked() {
                        match intracraft_model::place(&mut app.session, &self.library, id, None, [self.next_x, 0.0], 0.0, 0.0) {
                            Ok(new) => {
                                app.selection = vec![new];
                                self.next_x += 4000.0;
                            }
                            Err(e) => app.status = Status { text: e, error: true },
                        }
                    }
                }
            });
        }
        ribbon_panel(ui, "Object", |ui| {
            let tip = "Every inherited value of the selected object becomes its own";
            if ribbon_button(ui, &|p, r, c| Glyph::Detach.paint(p, r, c), "Detach", false, tip).clicked()
                && let Some(&id) = app.selection.first()
            {
                let r = intracraft_model::detach(&mut app.session, &self.library, id).map(|_| ());
                report(app, r);
            }
        });
    }

    fn discrete_tab(&mut self, ui: &mut Ui, app: &mut BimApp) {
        let asked = run_panel(ui, self.run.as_ref().map(|r| r.playing));
        if asked.restart || self.run.is_none() {
            let (layout, problems) = intracraft_model::layout::layout(&app.session.model, &self.library);
            match Run::start(layout, &self.library) {
                Ok(mut run) => {
                    run.playing = self.autoplay;
                    self.run = Some(run);
                    if let Some(p) = problems.first() {
                        app.status = Status { text: p.clone(), error: true };
                    }
                }
                Err(e) => app.status = Status { text: e, error: true },
            }
        }
        let Some(run) = &mut self.run else { return };
        if asked.toggle {
            run.playing = !run.playing;
        }
        if asked.step {
            run.step();
        }
        if let Some(s) = speed_panel(ui, &SPEEDS, run.speed) {
            run.speed = s;
        }
        clock_panel(ui, to_seconds(run.snapshot.time), run.snapshot.pallets.len(), None);
    }

    fn physics_tab(&mut self, ui: &mut Ui, app: &mut BimApp) {
        let asked = run_panel(ui, self.physics_run.as_ref().map(|r| r.playing));
        if asked.restart || self.physics_run.is_none() {
            let (layout, problems) = intracraft_model::layout::layout(&app.session.model, &self.library);
            match PhysicsRun::start(layout, &self.library) {
                Ok(mut run) => {
                    run.playing = self.autoplay;
                    self.physics_run = Some(run);
                    if let Some(p) = problems.first() {
                        app.status = Status { text: p.clone(), error: true };
                    }
                }
                Err(e) => app.status = Status { text: e, error: true },
            }
        }
        let Some(run) = &mut self.physics_run else { return };
        if asked.toggle {
            run.playing = !run.playing;
        }
        if asked.step {
            run.flow.step();
        }
        if let Some(s) = speed_panel(ui, &PHYSICS_SPEEDS, run.speed) {
            run.speed = s;
        }
        clock_panel(ui, to_seconds(run.flow.now()), run.flow.pallets().len(), Some(run.flow.fallen_pallets()));
    }

    fn tree(&mut self, ui: &mut Ui, app: &mut BimApp) {
        ui.label(RichText::new("Simulation tree").strong());
        let tree = simulation_tree(&app.session.model, &self.library);
        if tree.is_empty() {
            ui.label("No intralogistics object yet: place one from the Intralogistics tab.");
        }
        for group in tree {
            egui::CollapsingHeader::new(format!("{} ({})", group.name, group.leaves.len())).id_salt(group.name).default_open(true).show(
                ui,
                |ui| {
                    for leaf in group.leaves {
                        let selected = app.selection.contains(&leaf.id);
                        if ui.selectable_label(selected, leaf.label()).clicked() {
                            app.selection = vec![leaf.id];
                        }
                    }
                },
            );
        }
    }

    fn attribute_table(&mut self, ui: &mut Ui, app: &mut BimApp) {
        ui.label(RichText::new("Attributes").strong());
        let Some(&id) = app.selection.first() else {
            ui.label("Select an object.");
            return;
        };
        let model = std::sync::Arc::clone(&app.session.model);
        let Some(definition) = definition_of(&model, &self.library, id) else {
            ui.label(format!("{id} is not an intralogistics object."));
            return;
        };
        ui.label(format!("{id} · {}", definition.name.en));
        let attrs = match attributes(&model, &self.library, id) {
            Ok(a) => a,
            Err(e) => {
                ui.colored_label(Color32::from_rgb(200, 60, 60), e);
                return;
            }
        };
        let mut apply: Option<(String, Value)> = None;
        let mut reset: Option<String> = None;
        Grid::new(("attributes", id)).num_columns(4).striped(true).show(ui, |ui| {
            ui.label(RichText::new("Attribute").strong());
            ui.label(RichText::new("Value").strong());
            ui.label(RichText::new("✔/✖").strong());
            ui.label("");
            ui.end_row();
            for a in &attrs {
                ui.label(&a.label.en).on_hover_text(&a.description.en);
                if a.source == Source::Derived {
                    ui.label(RichText::new(shown(a)).weak());
                } else {
                    let key = a.key.clone();
                    let text = self.typed.entry(key.clone()).or_insert_with(|| edit_text(a));
                    let r = ui.add(egui::TextEdit::singleline(text).desired_width(90.0));
                    if r.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                        match parse(a, text) {
                            Ok(v) => apply = Some((key.clone(), v)),
                            Err(e) => app.status = Status { text: format!("{}: {e}", a.label.en), error: true },
                        }
                    }
                }
                let flag = match a.source {
                    Source::Instance => "✖ own",
                    Source::Type => "✔ type",
                    Source::Library => "✔ library",
                    Source::Derived => "derived",
                };
                ui.label(flag);
                if a.source == Source::Instance && ui.small_button("↺").on_hover_text("Inherit again").clicked() {
                    reset = Some(a.key.clone());
                }
                ui.end_row();
            }
        });
        if let Some((key, v)) = apply {
            let r = intracraft_model::set_attribute(&mut app.session, &self.library, id, &key, &v);
            report(app, r);
            self.typed.clear();
        }
        if let Some(key) = reset {
            let r = intracraft_model::reset_attribute(&mut app.session, &self.library, id, &key);
            report(app, r);
            self.typed.clear();
        }
    }
}

/// What the Control panel's buttons asked for.
#[derive(Default)]
struct RunAsked {
    restart: bool,
    toggle: bool,
    step: bool,
}

/// The Control panel: start again from the design, run or pause, one step.
fn run_panel(ui: &mut Ui, playing: Option<bool>) -> RunAsked {
    let mut asked = RunAsked::default();
    let on = playing == Some(true);
    ribbon_panel(ui, "Control", |ui| {
        let reset = "Start the run again from the design as it is now";
        asked.restart |= ribbon_button(ui, &|p, r, c| Glyph::Reset.paint(p, r, c), "Reset", false, reset).clicked();
        let (glyph, label) = if on { (Glyph::Pause, "Pause") } else { (Glyph::Run, "Run") };
        asked.toggle |= ribbon_button(ui, &|p, r, c| glyph.paint(p, r, c), label, on, "Run or pause the simulation").clicked();
        let step = "Run to the next event (discrete) or one physics step";
        asked.step |= ribbon_button(ui, &|p, r, c| Glyph::Step.paint(p, r, c), "Step", false, step).clicked();
    });
    asked
}

/// The Speed panel: the speed picked, if one was.
fn speed_panel(ui: &mut Ui, speeds: &[u32], now: f64) -> Option<f64> {
    let mut picked = None;
    ribbon_panel(ui, "Speed", |ui| {
        for &s in speeds {
            let tip = format!("{s} simulated seconds per second");
            if ribbon_button(ui, &|p, r, c| Glyph::Speed(s).paint(p, r, c), &format!("×{s}"), f64::from(s) == now, &tip).clicked() {
                picked = Some(f64::from(s));
            }
        }
    });
    picked
}

/// The Clock panel: the run's time, its pallets, and those that fell (physics).
fn clock_panel(ui: &mut Ui, seconds: f64, pallets: usize, fallen: Option<u64>) {
    ribbon_panel(ui, "Clock", |ui| {
        ui.vertical(|ui| {
            ui.label(RichText::new(clock(seconds)).monospace().size(18.0));
            ui.label(format!("{pallets} pallets in the plant"));
            if let Some(f) = fallen {
                ui.label(format!("{f} fallen"));
            }
        });
    });
}

/// A time as h:mm:ss.s.
fn clock(seconds: f64) -> String {
    let s = seconds.max(0.0);
    format!("{}:{:02}:{:04.1}", (s / 3600.0).floor(), ((s % 3600.0) / 60.0).floor(), s % 60.0)
}

fn report(app: &mut BimApp, r: Result<(), String>) {
    app.status = match r {
        Ok(()) => Status::default(),
        Err(text) => Status { text, error: true },
    };
}

fn warnings<'a>(ui: &mut Ui, issues: impl Iterator<Item = &'a str>, error: Option<&str>) {
    if let Some(e) = error {
        ui.colored_label(Color32::from_rgb(200, 60, 60), e);
    }
    let issues: Vec<&str> = issues.collect();
    if !issues.is_empty() {
        egui::CollapsingHeader::new(format!("Warnings ({})", issues.len())).show(ui, |ui| {
            for i in issues {
                ui.label(i);
            }
        });
    }
}

/// The statistics of a run: the selected object's, or every object's.
fn statistics_sheet(ui: &mut Ui, stats: &Statistics, layout: &Layout, selection: &[bimcraft_core::model::Id]) {
    ui.label(RichText::new("Statistics").strong());
    let selected = selection.first().map(|id| id.to_string());
    let ids: Vec<&String> = match &selected {
        Some(id) if stats.contains_key(id) => vec![id],
        _ => stats.keys().collect(),
    };
    for id in ids {
        let name = layout.instance(id).map_or(id.as_str(), |i| i.name.as_str());
        ui.label(RichText::new(name).strong());
        Grid::new(("stats", id)).num_columns(2).show(ui, |ui| {
            for (k, v) in &stats[id] {
                ui.label(*k);
                ui.label(format!("{v:.3}"));
                ui.end_row();
            }
        });
        ui.separator();
    }
}

/// A plan point (m) as a model point (mm).
fn mm(p: Point) -> P2 {
    P2(p.0 * 1000.0, p.1 * 1000.0)
}

/// Each object of the plan tinted by its state in the run.
fn tint_plan(painter: &Painter, app: &BimApp, layout: &Layout, status: &BTreeMap<String, &'static str>) {
    for f in plan::footprints(layout) {
        let tint = match status.get(&f.id).copied() {
            Some("working" | "lifting" | "lowering" | "processing") => Color32::from_rgba_unmultiplied(80, 190, 90, 70),
            Some("blocked") => Color32::from_rgba_unmultiplied(220, 70, 60, 80),
            Some("stopped") => Color32::from_rgba_unmultiplied(235, 160, 50, 80),
            Some("detecting") => Color32::from_rgba_unmultiplied(70, 120, 230, 90),
            _ => continue,
        };
        painter.add(Shape::convex_polygon(f.corners.iter().map(|c| app.screen(mm(*c))).collect(), tint, Stroke::NONE));
    }
}

/// Pallets and their loads as shaded boxes in the 3D view: the faces turned to the camera, the farthest pallet first.
fn pallets_3d(painter: &Painter, app: &BimApp, boxes: Vec<([Point; 4], f64, f64, f64)>) {
    let mut solids = Vec::new();
    for (outline, z, pallet, load) in boxes {
        solids.push((plan::solid(outline, z, z + pallet), WOOD));
        if load > 0.0 {
            solids.push((plan::solid(outline, z + pallet, z + pallet + load), LOAD));
        }
    }
    let projected: Vec<(Vec<egui::Pos2>, Color32)> =
        solids.into_iter().map(|(corners, colour)| (corners.iter().map(|c| app.screen3(*c)).collect(), colour)).collect();
    let mut order: Vec<usize> = (0..projected.len()).collect();
    // Seen from above, what lies higher on the screen lies farther.
    order.sort_by(|&a, &b| centre_y(&projected[a].0).total_cmp(&centre_y(&projected[b].0)));
    for k in order {
        let (pts, colour) = &projected[k];
        for (f, face) in FACES.iter().enumerate() {
            let quad: Vec<egui::Pos2> = face.iter().map(|&i| pts[i]).collect();
            // Counter-clockwise from outside is clockwise on the screen, whose y points down.
            let area: f32 = (0..4).map(|i| quad[i].x * quad[(i + 1) % 4].y - quad[(i + 1) % 4].x * quad[i].y).sum();
            if area >= 0.0 {
                continue;
            }
            let shade = [1.0, 0.5, 0.8, 0.65, 0.75, 0.7][f];
            let fill = Color32::from_rgb(
                (f32::from(colour.r()) * shade) as u8,
                (f32::from(colour.g()) * shade) as u8,
                (f32::from(colour.b()) * shade) as u8,
            );
            painter.add(Shape::convex_polygon(quad, fill, Stroke::new(0.6, Color32::from_rgb(70, 50, 30))));
        }
    }
}

fn centre_y(pts: &[egui::Pos2]) -> f32 {
    pts.iter().map(|p| p.y).sum::<f32>() / pts.len().max(1) as f32
}

/// The text a value is edited as: mm and degrees where the library shows them so.
fn edit_text(a: &Attribute) -> String {
    match (&a.value, a.display_unit.as_deref()) {
        (Some(Value::Number(x)), Some("mm")) => trim(x * 1000.0),
        (Some(Value::Number(x)), Some("°" | "deg" | "°/s" | "°/s²")) => trim(x.to_degrees()),
        (Some(Value::Number(x)), _) => trim(*x),
        (Some(Value::Bool(b)), _) => if *b { "yes" } else { "no" }.into(),
        (Some(Value::Text(t)), _) => t.clone(),
        (None, _) => String::new(),
    }
}

/// What was typed, as the value it stands for (SI).
fn parse(a: &Attribute, text: &str) -> Result<Value, String> {
    let t = text.trim();
    match a.kind {
        ParamType::Boolean => match t {
            "yes" | "true" | "1" | "sí" | "si" => Ok(Value::Bool(true)),
            "no" | "false" | "0" => Ok(Value::Bool(false)),
            _ => Err(format!("{t:?} is not yes or no")),
        },
        ParamType::Enum | ParamType::Text => Ok(Value::Text(t.to_string())),
        ParamType::Number | ParamType::Integer => {
            let x: f64 = t.replace(',', ".").parse().map_err(|_| format!("{t:?} is not a number"))?;
            Ok(Value::Number(match a.display_unit.as_deref() {
                Some("mm") => x / 1000.0,
                Some("°" | "deg" | "°/s" | "°/s²") => x.to_radians(),
                _ => x,
            }))
        }
    }
}

fn trim(x: f64) -> String {
    let s = format!("{x:.4}");
    s.trim_end_matches('0').trim_end_matches('.').to_string()
}

impl eframe::App for IntraApp {
    fn ui(&mut self, ui: &mut Ui, _frame: &mut eframe::Frame) {
        self.show(ui);
    }

    fn raw_input_hook(&mut self, ctx: &egui::Context, raw: &mut egui::RawInput) {
        eframe::App::raw_input_hook(&mut self.bim, ctx, raw);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bimcraft_core::engine::Session;
    use egui_kittest::Harness;
    use egui_kittest::kittest::Queryable;

    fn app() -> Harness<'static, IntraApp> {
        let library = Library::built_in().unwrap();
        let mut session = Session::new();
        let _ = intracraft_model::load_library(&mut session, &library);
        let bim = BimApp::new(session);
        let mut h = Harness::builder()
            .with_size([1800.0, 1000.0])
            .with_max_steps(16)
            .build_ui_state(|ui, app: &mut IntraApp| app.show(ui), IntraApp::new(bim, library));
        h.run();
        h
    }

    fn press(h: &mut Harness<IntraApp>, label: &str) {
        h.get_by_label(label).click();
        h.run();
    }

    #[test]
    fn the_modules_are_tabs_of_bimcrafts_ribbon_and_place_a_conveyor_with_its_inherited_attributes() {
        let mut h = app();
        press(&mut h, "Intralogistics");
        assert_eq!(h.state().module(), Module::Design);
        press(&mut h, "Roller conveyor");
        let id = *h.state().bim.selection.first().unwrap();
        assert!(h.query_by_label_contains(&format!("{id}  Roller conveyor · Default")).is_some());
        assert!(h.query_all_by_label("✔ library").count() > 10, "its attributes inherit from the library");
    }

    #[test]
    fn runs_a_source_conveyor_and_sink_in_both_simulation_modules() {
        let mut h = app();
        press(&mut h, "Intralogistics");
        for name in ["Pallet source", "Roller conveyor", "Pallet sink"] {
            if h.query_by_label(name).is_some() {
                press(&mut h, name);
            }
        }
        press(&mut h, "Discrete simulation");
        assert_eq!(h.state().module(), Module::Discrete);
        let run = h.state_mut().sim.run.as_mut().unwrap();
        run.advance(10.0);
        assert!(run.snapshot.time > 0);
        press(&mut h, "Physics simulation");
        let physics = h.state_mut().sim.physics_run.as_mut().unwrap();
        physics.advance(0.5);
        assert!(physics.flow.now() > from_seconds(0.4));
        // Playing, the run draws again every frame: one step, not until the window rests.
        h.get_by_label("Run").click();
        h.step();
        h.step();
        assert!(h.state().sim.physics_run.as_ref().unwrap().playing);
    }

    #[test]
    fn parses_typed_values_in_the_libraries_units() {
        let a = |kind, display: Option<&str>| Attribute {
            key: "k".into(),
            label: intracraft_library::definition::Text::en("k"),
            kind,
            unit: None,
            display_unit: display.map(Into::into),
            value: Some(Value::Number(0.2063)),
            description: intracraft_library::definition::Text::en("d"),
            source: Source::Library,
            group: "g".into(),
            advanced: false,
            options: vec![],
        };
        assert_eq!(edit_text(&a(ParamType::Number, Some("mm"))), "206.3");
        assert_eq!(parse(&a(ParamType::Number, Some("mm")), "3000"), Ok(Value::Number(3.0)));
        assert_eq!(
            parse(&a(ParamType::Number, Some("°")), "90").map(|v| v.number().map(|x| (x - std::f64::consts::FRAC_PI_2).abs() < 1e-12)),
            Ok(Some(true))
        );
        assert_eq!(parse(&a(ParamType::Boolean, None), "yes"), Ok(Value::Bool(true)));
        assert!(parse(&a(ParamType::Number, None), "fast").is_err());
        assert_eq!(parse(&a(ParamType::Number, None), "0,25"), Ok(Value::Number(0.25)));
        assert_eq!(clock(3725.5), "1:02:05.5");
    }
}
