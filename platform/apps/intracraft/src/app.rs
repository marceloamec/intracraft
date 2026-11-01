//! IntraCraft's window: BIMCraft's desktop with a module selector on top (ADR-0020 §7, A09 §0).
//!
//! - Design: BIMCraft whole, and beside it the IntraCraft library, the simulation tree and the selected object's
//!   attribute table (inherited ✔ / overridden ✖).
//! - Discrete simulation: the plan of the layout with its pallets moving, the run controls, and the selected object's
//!   statistics.
//! - Physics simulation: the Rapier world and the Bevy viewer arrive in phase R6; until then the module says so.
//!
//! The window is a thin shell: it reads the model and changes it only through `intracraft_model`'s functions, which run
//! BIMCraft's commands.

use std::collections::BTreeMap;

use bimcraft_core::model::Id;
use bimcraft_ui_egui::BimApp;
use egui::{CentralPanel, Color32, Grid, Panel, RichText, ScrollArea, Sense, Stroke, StrokeKind, Ui, pos2};
use intracraft_des::flow::{FlowSimulation, Snapshot, simulate};
use intracraft_des::time::{from_seconds, to_seconds};
use intracraft_library::Library;
use intracraft_library::definition::{ParamType, Value};
use intracraft_library::layout::Layout;
use intracraft_model::attributes::{Attribute, Source, attributes, definition_of, shown};
use intracraft_model::tree::simulation_tree;

use crate::plan::{self, Point};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Module {
    Design,
    Discrete,
    Physics,
}

impl Module {
    pub const ALL: [Module; 3] = [Module::Design, Module::Discrete, Module::Physics];

    pub fn label(self) -> &'static str {
        match self {
            Module::Design => "Design",
            Module::Discrete => "Discrete simulation",
            Module::Physics => "Physics simulation",
        }
    }
}

/// Speeds the discrete run plays at: simulated seconds per real second.
const SPEEDS: [f64; 5] = [1.0, 10.0, 100.0, 1000.0, 10_000.0];
/// Speeds the physics run plays at: it is real time at ×1, and a frame never runs more than a second of physics.
const PHYSICS_SPEEDS: [f64; 3] = [1.0, 2.0, 5.0];

/// A physics run of the layout the model held when it started.
pub struct PhysicsRun {
    pub world: intracraft_physics::world::World,
    pub layout: Layout,
    pub playing: bool,
    pub speed: f64,
}

impl PhysicsRun {
    pub fn start(layout: Layout, library: &Library) -> Result<PhysicsRun, String> {
        let world = intracraft_physics::world::World::new(&layout, library)?;
        Ok(PhysicsRun { world, layout, playing: false, speed: 1.0 })
    }

    /// Moves the run on by `real` seconds of the window's time at its speed (at most one second of physics a frame).
    pub fn advance(&mut self, real: f64) {
        let target = self.world.time() + (real * self.speed).min(1.0);
        self.world.run_until(target);
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

pub struct IntraApp {
    pub bim: BimApp,
    pub library: Library,
    pub module: Module,
    pub run: Option<Run>,
    pub physics_run: Option<PhysicsRun>,
    /// What is typed in the attribute table, by attribute key, until it is applied.
    typed: BTreeMap<String, String>,
    /// Where the next object of the library palette goes, mm east.
    next_x: f64,
    problem: Option<String>,
}

impl IntraApp {
    pub fn new(bim: BimApp, library: Library) -> IntraApp {
        IntraApp { bim, library, module: Module::Design, run: None, physics_run: None, typed: BTreeMap::new(), next_x: 0.0, problem: None }
    }

    pub fn show(&mut self, ui: &mut Ui) {
        Panel::top("intracraft-modules").show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(RichText::new("IntraCraft").strong());
                ui.separator();
                for m in Module::ALL {
                    if ui.selectable_label(self.module == m, m.label()).clicked() {
                        self.module = m;
                    }
                }
                if let Some(p) = &self.problem {
                    ui.separator();
                    ui.colored_label(Color32::from_rgb(200, 60, 60), p);
                }
            });
        });
        match self.module {
            Module::Design => self.design(ui),
            Module::Discrete => self.discrete(ui),
            Module::Physics => self.physics(ui),
        }
    }

    fn design(&mut self, ui: &mut Ui) {
        Panel::right("intracraft-design").default_size(340.0).show(ui, |ui| {
            ScrollArea::vertical().show(ui, |ui| {
                self.library_palette(ui);
                ui.separator();
                self.tree(ui);
                ui.separator();
                self.attribute_table(ui);
            });
        });
        CentralPanel::default().show(ui, |ui| self.bim.show(ui));
    }

    fn library_palette(&mut self, ui: &mut Ui) {
        ui.heading("Library");
        ui.label("Click to place at the origin row; move it with BIMCraft's tools, or place it from the Project Browser's Families.");
        let definitions: Vec<(String, String)> = self.library.list().iter().map(|d| (d.id.clone(), d.name.en.clone())).collect();
        ui.horizontal_wrapped(|ui| {
            for (id, name) in definitions {
                if ui.button(name).clicked() {
                    let at = [self.next_x, 0.0];
                    match intracraft_model::place(&mut self.bim.session, &self.library, &id, None, at, 0.0, 0.0) {
                        Ok(new) => {
                            self.bim.selection = vec![new];
                            self.next_x += 4000.0;
                            self.problem = None;
                        }
                        Err(e) => self.problem = Some(e),
                    }
                }
            }
        });
    }

    fn tree(&mut self, ui: &mut Ui) {
        ui.heading("Simulation tree");
        let tree = simulation_tree(&self.bim.session.model, &self.library);
        if tree.is_empty() {
            ui.label("No IntraCraft object yet.");
        }
        for group in tree {
            egui::CollapsingHeader::new(format!("{} ({})", group.name, group.leaves.len())).id_salt(group.name).default_open(true).show(
                ui,
                |ui| {
                    for leaf in group.leaves {
                        let selected = self.bim.selection.contains(&leaf.id);
                        if ui.selectable_label(selected, leaf.label()).clicked() {
                            self.bim.selection = vec![leaf.id];
                        }
                    }
                },
            );
        }
    }

    fn attribute_table(&mut self, ui: &mut Ui) {
        ui.heading("Attributes");
        let Some(&id) = self.bim.selection.first() else {
            ui.label("Select an object.");
            return;
        };
        let model = std::sync::Arc::clone(&self.bim.session.model);
        let Some(definition) = definition_of(&model, &self.library, id) else {
            ui.label(format!("{id} is not an IntraCraft object."));
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
                    let r = ui.add(egui::TextEdit::singleline(text).desired_width(110.0));
                    if r.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                        match parse(a, text) {
                            Ok(v) => apply = Some((key.clone(), v)),
                            Err(e) => self.problem = Some(format!("{}: {e}", a.label.en)),
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
        if ui.button("Detach from the library").on_hover_text("Every inherited value becomes the object's own").clicked() {
            let r = intracraft_model::detach(&mut self.bim.session, &self.library, id).map(|_| ());
            self.report(r);
        }
        if let Some((key, v)) = apply {
            let r = intracraft_model::set_attribute(&mut self.bim.session, &self.library, id, &key, &v);
            self.report(r);
            self.typed.clear();
        }
        if let Some(key) = reset {
            let r = intracraft_model::reset_attribute(&mut self.bim.session, &self.library, id, &key);
            self.report(r);
            self.typed.clear();
        }
    }

    fn report(&mut self, r: Result<(), String>) {
        self.problem = r.err();
    }

    fn discrete(&mut self, ui: &mut Ui) {
        Panel::left("intracraft-run").default_size(260.0).show(ui, |ui| self.run_controls(ui));
        Panel::right("intracraft-stats").default_size(280.0).show(ui, |ui| self.statistics(ui));
        CentralPanel::default().show(ui, |ui| self.plan(ui));
        if let Some(run) = &mut self.run
            && run.playing
        {
            let dt = f64::from(ui.input(|i| i.stable_dt)).min(0.1);
            run.advance(dt);
            ui.ctx().request_repaint();
        }
    }

    fn run_controls(&mut self, ui: &mut Ui) {
        ui.heading("Run");
        if ui.button("Reset from the design").clicked() || self.run.is_none() {
            let (layout, problems) = intracraft_model::layout::layout(&self.bim.session.model, &self.library);
            match Run::start(layout, &self.library) {
                Ok(run) => {
                    self.run = Some(run);
                    self.problem = problems.first().cloned();
                }
                Err(e) => self.problem = Some(e),
            }
        }
        let Some(run) = &mut self.run else { return };
        ui.horizontal(|ui| {
            if ui.button(if run.playing { "Pause" } else { "Run" }).clicked() {
                run.playing = !run.playing;
            }
            if ui.button("Next event").clicked() {
                run.step();
            }
        });
        ui.horizontal(|ui| {
            ui.label("Speed");
            for s in SPEEDS {
                if ui.selectable_label(run.speed == s, format!("×{s}")).clicked() {
                    run.speed = s;
                }
            }
        });
        ui.label(format!("Time: {:.1} s", to_seconds(run.snapshot.time)));
        ui.label(format!("Pallets: {}", run.snapshot.pallets.len()));
        if let Some(e) = &run.error {
            ui.colored_label(Color32::from_rgb(200, 60, 60), e);
        }
        ui.separator();
        ui.heading("Warnings");
        if run.sim.issues.is_empty() {
            ui.label("None.");
        }
        for i in &run.sim.issues {
            ui.label(&i.message);
        }
    }

    fn statistics(&mut self, ui: &mut Ui) {
        ui.heading("Statistics");
        let Some(run) = &mut self.run else { return };
        let stats = run.sim.statistics();
        let selected = self.bim.selection.first().map(|id| id.to_string());
        let ids: Vec<&String> = match &selected {
            Some(id) if stats.contains_key(id) => vec![id],
            _ => stats.keys().collect(),
        };
        ScrollArea::vertical().show(ui, |ui| {
            for id in ids {
                let name = run.layout.instance(id).map_or(id.as_str(), |i| i.name.as_str());
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
        });
    }

    fn plan(&mut self, ui: &mut Ui) {
        let Some(run) = &self.run else { return };
        let (rect, response) = ui.allocate_exact_size(ui.available_size(), Sense::click());
        let painter = ui.painter_at(rect);
        painter.rect_filled(rect, 0.0, Color32::from_gray(245));
        let footprints = plan::footprints(&run.layout);
        let ((x0, y0), (x1, y1)) = plan::bounds(&footprints);
        let scale = (f64::from(rect.width()) / (x1 - x0)).min(f64::from(rect.height()) / (y1 - y0));
        let to_screen = |p: Point| pos2(rect.left() + ((p.0 - x0) * scale) as f32, rect.bottom() - ((p.1 - y0) * scale) as f32);
        for f in &footprints {
            let colour = match run.snapshot.status.get(&f.id).copied() {
                Some("working") => Color32::from_rgb(120, 190, 120),
                Some("blocked") => Color32::from_rgb(220, 110, 100),
                Some("stopped") => Color32::from_rgb(235, 170, 80),
                Some("detecting") => Color32::from_rgb(90, 140, 230),
                _ => Color32::from_gray(200),
            };
            let selected = self.bim.selection.first().is_some_and(|id| id.to_string() == f.id);
            let stroke = Stroke::new(if selected { 3.0 } else { 1.0 }, Color32::from_gray(if selected { 20 } else { 90 }));
            painter.add(egui::Shape::convex_polygon(f.corners.iter().map(|c| to_screen(*c)).collect(), colour, stroke));
        }
        for p in &run.snapshot.pallets {
            if let Some(o) = plan::pallet_outline(&run.layout, p) {
                let pts: Vec<_> = o.iter().map(|c| to_screen(*c)).collect();
                painter.add(egui::Shape::convex_polygon(
                    pts,
                    Color32::from_rgb(181, 136, 84),
                    Stroke::new(1.0, Color32::from_rgb(90, 60, 30)),
                ));
            }
        }
        painter.rect_stroke(rect, 0.0, Stroke::new(1.0, Color32::from_gray(160)), StrokeKind::Inside);
        if response.clicked()
            && let Some(pos) = response.interact_pointer_pos()
        {
            let p = (x0 + f64::from(pos.x - rect.left()) / scale, y0 + f64::from(rect.bottom() - pos.y) / scale);
            self.bim.selection =
                footprints.iter().find(|f| plan::contains(f, p)).and_then(|f| f.id.parse::<Id>().ok()).into_iter().collect();
        }
    }

    fn physics(&mut self, ui: &mut Ui) {
        Panel::left("intracraft-physics-run").default_size(260.0).show(ui, |ui| self.physics_controls(ui));
        CentralPanel::default().show(ui, |ui| self.physics_plan(ui));
        if let Some(run) = &mut self.physics_run
            && run.playing
        {
            let dt = f64::from(ui.input(|i| i.stable_dt)).min(0.1);
            run.advance(dt);
            ui.ctx().request_repaint();
        }
    }

    fn physics_controls(&mut self, ui: &mut Ui) {
        ui.heading("Physics run");
        ui.label("Rapier: rollers turning at ω = v / r, chains with a moving surface, pallets as rigid bodies. The 3D viewer (Bevy) comes later; this is the plan of the same run.");
        if ui.button("Reset from the design").clicked() || self.physics_run.is_none() {
            let (layout, problems) = intracraft_model::layout::layout(&self.bim.session.model, &self.library);
            match PhysicsRun::start(layout, &self.library) {
                Ok(run) => {
                    self.physics_run = Some(run);
                    self.problem = problems.first().cloned();
                }
                Err(e) => self.problem = Some(e),
            }
        }
        let Some(run) = &mut self.physics_run else { return };
        ui.horizontal(|ui| {
            if ui.button(if run.playing { "Pause" } else { "Run" }).clicked() {
                run.playing = !run.playing;
            }
        });
        ui.horizontal(|ui| {
            ui.label("Speed");
            for s in PHYSICS_SPEEDS {
                if ui.selectable_label(run.speed == s, format!("×{s}")).clicked() {
                    run.speed = s;
                }
            }
        });
        ui.label(format!("Time: {:.1} s", run.world.time()));
        ui.label(format!("Pallets: {}", run.world.pallets.len()));
        for id in run.layout.instances.iter().filter(|i| i.definition_id == "flow.sink").map(|i| i.id.clone()) {
            if let Some((received, per_hour, cycle)) = run.world.sink_statistics(&id) {
                ui.label(format!("Sink {id}: {received} pallets, {per_hour:.0}/h, cycle {cycle:.1} s"));
            }
        }
        ui.separator();
        ui.heading("Warnings");
        if run.world.issues.is_empty() {
            ui.label("None.");
        }
        for i in &run.world.issues {
            ui.label(i);
        }
    }

    fn physics_plan(&mut self, ui: &mut Ui) {
        let Some(run) = &self.physics_run else { return };
        let (rect, _) = ui.allocate_exact_size(ui.available_size(), Sense::hover());
        let painter = ui.painter_at(rect);
        painter.rect_filled(rect, 0.0, Color32::from_gray(245));
        let footprints = plan::footprints(&run.layout);
        let ((x0, y0), (x1, y1)) = plan::bounds(&footprints);
        let scale = (f64::from(rect.width()) / (x1 - x0)).min(f64::from(rect.height()) / (y1 - y0));
        let to_screen = |p: Point| pos2(rect.left() + ((p.0 - x0) * scale) as f32, rect.bottom() - ((p.1 - y0) * scale) as f32);
        for f in &footprints {
            let moving = run.world.conveyor_ids().contains(&f.id.as_str());
            let colour = if moving { Color32::from_rgb(170, 200, 230) } else { Color32::from_gray(205) };
            painter.add(egui::Shape::convex_polygon(
                f.corners.iter().map(|c| to_screen(*c)).collect(),
                colour,
                Stroke::new(1.0, Color32::from_gray(90)),
            ));
        }
        for p in run.world.poses() {
            let pts: Vec<_> = plan::body_outline(p.centre, p.yaw, p.size).iter().map(|c| to_screen(*c)).collect();
            painter.add(egui::Shape::convex_polygon(pts, Color32::from_rgb(181, 136, 84), Stroke::new(1.0, Color32::from_rgb(90, 60, 30))));
        }
        painter.rect_stroke(rect, 0.0, Stroke::new(1.0, Color32::from_gray(160)), StrokeKind::Inside);
    }
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
        if self.module == Module::Design {
            eframe::App::raw_input_hook(&mut self.bim, ctx, raw);
        }
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
            .with_size([1600.0, 900.0])
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
    fn places_a_conveyor_from_the_library_and_lists_it_in_the_tree_with_its_attributes() {
        let mut h = app();
        press(&mut h, "Roller conveyor");
        let id = *h.state().bim.selection.first().unwrap();
        assert!(h.query_by_label_contains(&format!("{id}  Roller conveyor · Default")).is_some());
        assert!(h.query_all_by_label("✔ library").count() > 10, "its attributes inherit from the library");
    }

    #[test]
    fn runs_a_source_conveyor_and_sink_in_the_discrete_module() {
        let mut h = app();
        for name in ["Pallet source", "Roller conveyor", "Pallet sink"] {
            if h.query_by_label(name).is_some() {
                press(&mut h, name);
            }
        }
        press(&mut h, "Discrete simulation");
        assert!(h.state().run.is_some());
        // Run 100 simulated seconds, as Next event and Run would.
        let run = h.state_mut().run.as_mut().unwrap();
        run.advance(10.0);
        assert!(run.snapshot.time > 0);
        press(&mut h, "Physics simulation");
        assert!(h.query_by_label_contains("Rapier").is_some());
        let physics = h.state_mut().physics_run.as_mut().unwrap();
        physics.advance(0.5);
        assert!(physics.world.time() > 0.4);
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
    }
}
