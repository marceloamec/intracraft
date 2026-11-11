//! IntraCraft's window: BIMCraft's own, with IntraCraft's three modules on its ribbon (ADR-0020 §7, A09 §0), through
//! BIMCraft's `Extension` hooks:
//!
//! - Design: BIMCraft whole, and the **Intralogistics** tab with the library's objects to place; beside the views, the
//!   simulation tree and the selected object's attribute table (inherited ✔ / own ✖).
//! - **Discrete simulation**: the run's controls on the ribbon; the pallets moving over BIMCraft's floor plan and 3D view,
//!   each object tinted by its state; the selected object's statistics beside them.
//! - **Physics simulation**: the same over the physics run on Avian (ADR-0024, ADR-0026), its pallets where their bodies
//!   are.
//!
//! Both simulation tabs show the run in 3D beside the floor plan with **2D + 3D** (`view3d`, in this window), or open it
//! in IntraCraft's 3D window (`intracraft-viewer`, Bevy) with **View in 3D**.
//! BIMCraft's MEP tabs (Plumbing, Electrical, Lightning Protection, Data) are hidden: a conveyor plant has no use for them.
//!
//! The window is a thin shell: it reads the model and changes it only through `intracraft_model`'s functions, which run
//! BIMCraft's commands.

use std::collections::BTreeMap;

use bimcraft_core::geom::P2;
use bimcraft_ui_egui::{BimApp, Extension, Sheet, Status, ribbon_button, ribbon_panel};
use egui::{Color32, Grid, Painter, RichText, ScrollArea, Shape, Stroke, Ui};
use intracraft_avian::AvianBench;
use intracraft_des::flow::{FlowSimulation, Snapshot, Statistics, simulate};
use intracraft_des::time::{from_seconds, to_seconds};
use intracraft_library::Library;
use intracraft_library::layout::Layout;
use intracraft_library::layout::Project;
use intracraft_model::attributes::definition_of;
use intracraft_model::tree::simulation_tree;
use intracraft_physics::flow::PhysicsFlow;
use intracraft_scene::pose::{PalletShow, along_width, discrete_pallet, rotation_of, storage_show};

use crate::properties::{Asked, RunFacts};
use crate::view3d::Picture;

use crate::glyphs::Glyph;
use crate::plan::{self, Point};

/// IntraCraft's ribbon tabs, after BIMCraft's own.
pub const TABS: [&str; 3] = ["Intralogistics", "Discrete simulation", "Physics simulation"];
/// BIMCraft's tabs IntraCraft hides: the MEP disciplines, of no use in a conveyor plant.
const HIDDEN_TABS: [&str; 4] = ["Plumbing", "Electrical", "Lightning Protection", "Data"];
/// The seed of the runs started from the design.
const SEED: u64 = 1;
/// Speeds the discrete run plays at: simulated seconds per real second.
const SPEEDS: [u32; 5] = [1, 10, 100, 1000, 10_000];
/// Speeds the physics run plays at: it is real time at ×1, and a frame never runs more than a second of physics.
const PHYSICS_SPEEDS: [u32; 3] = [1, 2, 5];
const WOOD: Color32 = Color32::from_rgb(181, 136, 84);
/// A stacker crane on the plan.
const CRANE: Color32 = Color32::from_rgb(201, 130, 43);

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
        let mut sim = simulate(&layout, library, SEED)?;
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
        let flow = PhysicsFlow::on(Box::new(AvianBench::new()), &layout, library, SEED)?;
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
    /// A picture of the window to save, then close (`--screenshot`).
    pub shot: Option<crate::shot::Shot>,
}

/// What IntraCraft adds to BIMCraft's window: its library, its runs and what the attribute table holds while typed.
pub struct Simulation {
    pub library: Library,
    pub run: Option<Run>,
    pub physics_run: Option<PhysicsRun>,
    /// Runs start playing as soon as they are made (`--play`).
    pub autoplay: bool,
    /// The objects' property sheets: in windows of their own, or the selected one's docked in the panel.
    pub sheets: crate::properties::Sheets,
    /// Where the next object placed from the ribbon goes, mm east.
    next_x: f64,
    /// The editor of a code block's IntraScript code.
    pub code: crate::code::CodeEditor,
    /// The high-bay warehouse wizard (A22).
    pub warehouse: crate::warehouse::Wizard,
    /// The graphics card, when the window has one; the run's 3D view beside the plan, and whether it shows.
    pub graphics: Option<eframe::egui_wgpu::RenderState>,
    pub split: bool,
    view3d: crate::view3d::View3D,
    /// The Design module's layout as the 3D view shows it, and the model it was read from: read again when that changes.
    design: Option<(std::sync::Arc<bimcraft_core::model::Model>, Layout)>,
}

impl IntraApp {
    pub fn new(mut bim: BimApp, library: Library) -> IntraApp {
        // A dragged object snaps onto the free end of another (ADR-0028), in the plan and in the 3D view.
        let shared = std::sync::Arc::new(library.clone());
        bim.move_snap = Some(std::sync::Arc::new(move |model, ids, by| {
            let moved = intracraft_model::snap::snap_move(model, &shared, ids, by);
            let said = moved.said();
            (moved.by, said)
        }));
        let warehouse = crate::warehouse::Wizard::new(&library);
        IntraApp {
            bim,
            shot: None,
            sim: Simulation {
                library,
                warehouse,
                run: None,
                physics_run: None,
                autoplay: false,
                sheets: Default::default(),
                next_x: 0.0,
                code: Default::default(),
                graphics: None,
                split: false,
                view3d: Default::default(),
                design: None,
            },
        }
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
        if let Some(problem) = self.shot.as_mut().and_then(|s| s.tick(ui.ctx())) {
            self.bim.status = Status { text: problem, error: true };
        }
    }
}

impl Extension for Simulation {
    fn tabs(&self) -> Vec<String> {
        TABS.iter().map(|t| t.to_string()).collect()
    }

    fn hides_tab(&self, name: &str) -> bool {
        HIDDEN_TABS.contains(&name)
    }

    fn splits_view(&self, app: &BimApp) -> bool {
        self.split
            && match Module::of(app.extension_tab()) {
                Module::Discrete => self.run.is_some(),
                Module::Physics => self.physics_run.is_some(),
                Module::Design => true,
            }
    }

    /// The open module in 3D, beside the floor plan: the design, where objects are picked, moved and snapped as in the
    /// plan; or the run, where they are picked.
    fn split_view(&mut self, ui: &mut Ui, app: &mut BimApp) {
        let module = Module::of(app.extension_tab());
        if module == Module::Design && !self.design.as_ref().is_some_and(|(m, _)| std::sync::Arc::ptr_eq(m, &app.session.model)) {
            let model = std::sync::Arc::clone(&app.session.model);
            let (layout, _) = intracraft_model::layout::layout(&model, &self.library);
            self.design = Some((model, layout));
        }
        let (layout, picture) = match module {
            Module::Discrete => {
                let Some(run) = &self.run else { return };
                let s = &run.snapshot;
                let mut pallets: Vec<PalletShow> =
                    s.pallets.iter().filter_map(|p| discrete_pallet(&run.layout, &self.library, p, &s.heights)).collect();
                let (stored, shifts) = storage_show(&run.layout, &self.library, &s.storage);
                pallets.extend(stored);
                let (angles, heights, status) = (s.angles.clone(), s.heights.clone(), s.status.clone());
                let picture = Picture { pallets, angles, heights, shifts, status };
                (&run.layout, picture)
            }
            Module::Physics => {
                let Some(run) = &self.physics_run else { return };
                let f = &run.flow;
                let pallets = f
                    .pallets()
                    .into_iter()
                    .map(|p| PalletShow {
                        rotation: rotation_of(p.axes.0, p.axes.1, along_width(&self.library, &p.type_id, p.size[0])),
                        id: p.id,
                        type_id: p.type_id,
                        source: p.source,
                        centre: p.centre,
                        height: p.size[1],
                        load_height: p.load_height,
                        wrapped: p.wrapped,
                    })
                    .collect();
                let mut pallets: Vec<PalletShow> = pallets;
                let (stored, shifts) = storage_show(&run.layout, &self.library, &f.storage());
                pallets.extend(stored);
                (&run.layout, Picture { pallets, angles: f.angles(), heights: f.heights(), shifts, status: f.status() })
            }
            Module::Design => match &self.design {
                Some((_, layout)) => (layout, Picture::default()),
                None => return,
            },
        };
        // A drag in 3D snaps as one in the plan does: the same function, in the model's millimetres.
        let snap = |iid: &str, by: [f64; 2]| snap_3d(app, iid, by);
        let how = crate::view3d::Interaction {
            selected: app.selection.first().map(|id| id.to_string()),
            editable: module == Module::Design,
            snap: &snap,
            menu: &app.extension_actions,
        };
        let events = self.view3d.show(ui, self.graphics.as_ref(), layout, &self.library, &picture, &how);
        for e in events {
            self.view_event(app, e);
        }
    }

    fn ribbon(&mut self, ui: &mut Ui, k: usize, app: &mut BimApp) {
        match Module::of(Some(k)) {
            Module::Design => self.library_tab(ui, app),
            Module::Discrete => self.discrete_tab(ui, app),
            Module::Physics => self.physics_tab(ui, app),
        }
    }

    /// Over the floor plan, each object tinted by its state and the pallets where they are. The 3D run is shown in
    /// IntraCraft's 3D window (View in 3D).
    fn over_view(&mut self, painter: &Painter, app: &BimApp) {
        if !matches!(app.sheet, Sheet::Plan(_)) {
            return;
        }
        let (layout, status, boxes): (&Layout, BTreeMap<String, &str>, Vec<[Point; 4]>) = match Module::of(app.extension_tab()) {
            Module::Discrete => {
                let Some(run) = &self.run else { return };
                let boxes = run.snapshot.pallets.iter().filter_map(|p| plan::pallet_outline(&run.layout, p)).collect();
                (&run.layout, run.snapshot.status.clone(), boxes)
            }
            Module::Physics => {
                let Some(run) = &self.physics_run else { return };
                let boxes = run.flow.pallets().into_iter().map(|p| plan::body_outline(p.centre, p.yaw, p.size)).collect();
                (&run.layout, run.flow.status(), boxes)
            }
            Module::Design => return,
        };
        tint_plan(painter, app, layout, &status);
        // The stacker cranes, where they are in their aisles.
        if let (Module::Discrete, Some(run)) = (Module::of(app.extension_tab()), &self.run) {
            for outline in plan::crane_outlines(&run.layout, &run.snapshot.storage) {
                let pts: Vec<_> = outline.iter().map(|c| app.screen(mm(*c))).collect();
                painter.add(Shape::convex_polygon(pts, CRANE, Stroke::new(1.0, Color32::from_rgb(110, 60, 10))));
            }
        }
        for outline in boxes {
            let pts: Vec<_> = outline.iter().map(|c| app.screen(mm(*c))).collect();
            painter.add(Shape::convex_polygon(pts, WOOD, Stroke::new(1.0, Color32::from_rgb(90, 60, 30))));
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
        let module = Module::of(app.extension_tab());
        self.object_actions(ui.ctx(), app, module);
        ScrollArea::vertical().show(ui, |ui| match module {
            Module::Design => {
                self.code.show(ui.ctx(), app, &self.library);
                self.warehouse.show(ui.ctx(), app, &self.library);
                self.tree(ui, app);
                ui.separator();
                let asked = self.sheets.docked(ui, app, &self.library, None);
                self.asked(asked);
            }
            Module::Discrete => {
                if let Some(run) = &mut self.run {
                    warnings(ui, run.sim.issues.iter().map(|i| i.message.as_str()), run.error.as_deref());
                    crate::code::run_panel(ui, &run.sim, &run.layout);
                    crate::sections::pallet_panel(ui, &run.snapshot, &run.sim, &run.layout);
                    let stats = run.sim.statistics();
                    let asked = sheets_or_statistics(&mut self.sheets, &self.library, ui, app, (&stats, &run.snapshot.status), &run.layout);
                    self.asked(asked);
                }
            }
            Module::Physics => {
                if let Some(run) = &self.physics_run {
                    warnings(ui, run.flow.issues.iter().map(|i| i.message.as_str()), None);
                    let (stats, status, layout) = (run.flow.statistics(), run.flow.status(), run.layout.clone());
                    let asked = sheets_or_statistics(&mut self.sheets, &self.library, ui, app, (&stats, &status), &layout);
                    self.asked(asked);
                }
            }
        });
    }
}

impl Simulation {
    /// The library's objects, by kind, each placed by a click after the selected object (joined to its free exit), else at
    /// the next free spot (move it with BIMCraft's tools, or drag it: it snaps); the selected object detached from the
    /// library.
    fn library_tab(&mut self, ui: &mut Ui, app: &mut BimApp) {
        let kinds = [
            ("Conveyors", "conveyor.roller conveyor.chain conveyor.gravity"),
            (
                "Stations",
                "conveyor.roller_turntable conveyor.chain_turntable conveyor.transfer_rc flow.process_station flow.stretch_wrapper",
            ),
            ("Lifts", "conveyor.lift_table_manual conveyor.scissor_lift conveyor.elevator"),
            ("Storage", "storage.aisle"),
            ("Flow", "flow.source flow.sink sensor.photoeye"),
            ("Scenery", "scenery.asset"),
            ("Logic", "info.code"),
        ];
        for (title, ids) in kinds {
            ribbon_panel(ui, title, |ui| {
                for id in ids.split(' ') {
                    let Some(d) = self.library.get(id) else { continue };
                    let glyph = Glyph::of(id);
                    let tip = format!("Place a {} from the library", d.name.en.to_lowercase());
                    if ribbon_button(ui, &|p, r, c| glyph.paint(p, r, c), &d.name.en, false, &tip).clicked() {
                        // After the selected object, joined to its free exit; else at the next free spot.
                        let after = match app.selection.as_slice() {
                            &[sel] if intracraft_model::snap::next_after(&app.session.model, &self.library, sel, id).is_some() => Some(sel),
                            _ => None,
                        };
                        let placed = match after {
                            Some(sel) => intracraft_model::snap::place_after(&mut app.session, &self.library, sel, id),
                            None => intracraft_model::place(&mut app.session, &self.library, id, None, [self.next_x, 0.0], 0.0, 0.0),
                        };
                        match placed {
                            Ok(new) => {
                                app.selection = vec![new];
                                if after.is_none() {
                                    self.next_x += 4000.0;
                                }
                            }
                            Err(e) => app.status = Status { text: e, error: true },
                        }
                    }
                }
                if title == "Storage" {
                    let tip = "Lay out a high-bay warehouse from its size: aisles, stacker cranes and their chain conveyors";
                    if ribbon_button(ui, &|p, r, c| Glyph::Warehouse.paint(p, r, c), "Warehouse", self.warehouse.open, tip).clicked() {
                        self.warehouse.ask([self.next_x, 0.0]);
                    }
                }
            });
        }
        crate::sections::sections_panel(ui, app);
        ribbon_panel(ui, "3D", |ui| {
            let tip = "Show the design in 3D beside the floor plan: pick, move and turn objects there too";
            if ribbon_button(ui, &|p, r, c| Glyph::Split.paint(p, r, c), "2D + 3D", self.split, tip).clicked() {
                self.split = !self.split;
            }
        });
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
        let layout = run.layout.clone();
        view_3d_panel(ui, app, &layout, "discrete", &mut self.split);
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
        let layout = run.layout.clone();
        view_3d_panel(ui, app, &layout, "physics", &mut self.split);
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

    /// What the right-click menus and double clicks asked of an object, and the property sheets in their windows (with
    /// what the open run says of their objects).
    fn object_actions(&mut self, ctx: &egui::Context, app: &mut BimApp, module: Module) {
        app.extension_actions = crate::actions::offered(app, &self.library, module == Module::Design);
        if let Some(action) = app.extension_chosen.take() {
            let sheets = &mut self.sheets;
            crate::actions::act(&action, app, &self.library, &mut |id| sheets.float(id));
        }
        if let Some(id) = app.double_clicked.take().filter(|id| definition_of(&app.session.model, &self.library, *id).is_some()) {
            self.sheets.float(id);
        }
        let (stats, status) = match module {
            Module::Design => (None, BTreeMap::new()),
            Module::Discrete => match &mut self.run {
                Some(run) => (Some(run.sim.statistics()), run.snapshot.status.clone()),
                None => (None, BTreeMap::new()),
            },
            Module::Physics => match &self.physics_run {
                Some(run) => (Some(run.flow.statistics()), run.flow.status()),
                None => (None, BTreeMap::new()),
            },
        };
        let asked = match &stats {
            Some(stats) => {
                let facts = |id: bimcraft_core::model::Id| run_facts(stats, &status, id);
                self.sheets.windows(ctx, app, &self.library, Some(&facts))
            }
            None => self.sheets.windows(ctx, app, &self.library, None),
        };
        self.asked(asked);
    }

    /// What the hand did in the 3D view, done to the model as in the plan: picks, moves (BIMCraft's `move`), properties
    /// and the menu's items (taken next frame, as the plan's).
    fn view_event(&mut self, app: &mut BimApp, event: crate::view3d::Event3D) {
        use crate::view3d::Event3D;
        let id_of = |iid: &str| iid.parse::<bimcraft_core::model::Id>().ok();
        match event {
            Event3D::Select(picked) => app.selection = picked.as_deref().and_then(id_of).into_iter().collect(),
            Event3D::Moved(iid, [dx, dz]) => {
                let Some(id) = id_of(&iid) else { return };
                let by = P2(dx * 1000.0, -dz * 1000.0);
                let said = app.move_snap.as_ref().and_then(|f| f(&app.session.model.clone(), &[id], by).1);
                let r = app.session.execute("move", &serde_json::json!({"ids": [id], "by": [by.0, by.1]})).map(|_| ());
                report(app, r);
                if let Some(text) = said.filter(|_| !app.status.error) {
                    app.status = Status { text, error: false };
                }
            }
            Event3D::Open(iid) => {
                if let Some(id) = id_of(&iid) {
                    app.selection = vec![id];
                    self.sheets.float(id);
                }
            }
            Event3D::Chosen(action) => app.extension_chosen = Some(action),
        }
    }

    fn asked(&mut self, asked: Vec<Asked>) {
        for a in asked {
            if let Asked::Code(id, code) = a {
                self.code.open(id, &code);
            }
        }
    }
}

/// In a run's panel: the selected object's sheet, read-only with its state and statistics; else every object's
/// statistics.
fn sheets_or_statistics(
    sheets: &mut crate::properties::Sheets,
    library: &Library,
    ui: &mut Ui,
    app: &mut BimApp,
    (stats, status): (&Statistics, &BTreeMap<String, &'static str>),
    layout: &Layout,
) -> Vec<Asked> {
    match app.selection.first().copied().filter(|id| definition_of(&app.session.model, library, *id).is_some()) {
        Some(id) => sheets.docked(ui, app, library, Some(run_facts(stats, status, id))),
        None => {
            statistics_sheet(ui, stats, layout, &app.selection);
            Vec::new()
        }
    }
}

/// How far a drag in the 3D view moves object `iid` (m over the floor, x east and z south): snapped as one in the plan,
/// by the same function in the model's millimetres (y north).
fn snap_3d(app: &BimApp, iid: &str, [dx, dz]: [f64; 2]) -> [f64; 2] {
    let (Ok(id), Some(f)) = (iid.parse::<bimcraft_core::model::Id>(), app.move_snap.as_ref()) else { return [dx, dz] };
    let by = f(&app.session.model, &[id], P2(dx * 1000.0, -dz * 1000.0)).0;
    [by.0 / 1000.0, -by.1 / 1000.0]
}

/// What a run says of object `id`: its state and its statistics.
fn run_facts(stats: &Statistics, status: &BTreeMap<String, &'static str>, id: bimcraft_core::model::Id) -> RunFacts {
    let key = id.to_string();
    RunFacts {
        status: status.get(&key).copied(),
        statistics: stats.get(&key).map(|m| m.iter().map(|(k, v)| (*k, *v)).collect()).unwrap_or_default(),
    }
}

/// The 3D panel: 2D + 3D shows the run in 3D beside the floor plan, in this window; View in 3D opens it in IntraCraft's
/// 3D window, playing.
fn view_3d_panel(ui: &mut Ui, app: &mut BimApp, layout: &Layout, module: &str, split: &mut bool) {
    ribbon_panel(ui, "3D", |ui| {
        let tip = "Show the run in 3D beside the floor plan, in this window";
        if ribbon_button(ui, &|p, r, c| Glyph::Split.paint(p, r, c), "2D + 3D", *split, tip).clicked() {
            *split = !*split;
            if *split && !matches!(app.sheet, Sheet::Plan(_)) {
                app.status = Status { text: "Open a floor plan to see the run in 2D beside the 3D view".into(), error: false };
            }
        }
        let tip = "Open this run in IntraCraft's 3D window: rollers, chains and pallets with their loads, in motion";
        if ribbon_button(ui, &|p, r, c| Glyph::View3D.paint(p, r, c), "View in 3D", false, tip).clicked() {
            report(app, open_viewer(layout, module));
        }
    });
}

/// Writes the layout with the runs' seed to a project file and starts the 3D window on it, the program beside this one.
fn open_viewer(layout: &Layout, module: &str) -> Result<(), String> {
    let name = if cfg!(windows) { "intracraft-viewer.exe" } else { "intracraft-viewer" };
    let viewer = std::env::current_exe()
        .ok()
        .and_then(|e| e.parent().map(|d| d.join(name)))
        .filter(|p| p.exists())
        .ok_or("the 3D window isn't built beside IntraCraft: run `cargo build --release -p intracraft-viewer` in platform/")?;
    let project = Project { schema_version: 1, name: "IntraCraft run".into(), seed: SEED, layout: layout.clone() };
    let text = serde_json::to_string_pretty(&project).map_err(|e| e.to_string())?;
    let path = std::env::temp_dir().join(format!("intracraft-view-{}.intracraft.json", std::process::id()));
    std::fs::write(&path, text).map_err(|e| format!("{}: {e}", path.display()))?;
    let started = std::process::Command::new(viewer).arg(&path).args(["--module", module, "--play"]).spawn();
    started.map(|_| ()).map_err(|e| e.to_string())
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
    use intracraft_library::definition::Value;
    use intracraft_model::attributes::attributes;

    fn app() -> Harness<'static, IntraApp> {
        let library = Library::built_in().unwrap();
        let mut session = Session::new();
        let _ = intracraft_model::load_library(&mut session, &library);
        let bim = BimApp::new(session);
        let mut h = Harness::builder()
            .with_size([1920.0, 1000.0])
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
        assert!(h.query_all_by_label("■").count() > 10, "its attributes inherit from the library");
    }

    #[test]
    fn bimcraft_s_mep_tabs_are_left_out_of_the_ribbon_and_both_runs_offer_the_3d_window() {
        let mut h = app();
        for tab in HIDDEN_TABS {
            assert!(h.query_by_label(tab).is_none(), "{tab} is on the ribbon");
        }
        assert!(h.query_by_label("Architecture").is_some());
        for module in ["Discrete simulation", "Physics simulation"] {
            press(&mut h, module);
            assert!(h.query_by_label("View in 3D").is_some(), "{module}");
        }
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
    fn a_code_block_is_placed_edited_checked_saved_and_run() {
        let mut h = app();
        press(&mut h, "Intralogistics");
        for name in ["Pallet source", "Roller conveyor", "Pallet sink", "Code block"] {
            press(&mut h, name);
        }
        let block = *h.state().bim.selection.first().unwrap();
        press(&mut h, "Edit code…");
        assert_eq!(h.state().sim.code.element, Some(block));
        // A typo is found against the layout before any run.
        h.state_mut().sim.code.text = "@on_start
def go():
    Roler.run = False
"
        .into();
        h.run();
        assert_eq!(h.state().sim.code.diagnostics()[0].code, "unknownName");
        let roller = intracraft_script::code_name(&format!("Roller conveyor {}", block - 2));
        let code = format!(
            "@on_start
def go():
    print('{roller} runs:', {roller}.run, {roller}.length)

@on_enter({roller})
def seen(pallet):
    print(pallet.id)
"
        );
        h.state_mut().sim.code.text = code.clone();
        h.run();
        assert!(h.state().sim.code.diagnostics().is_empty(), "{:?}", h.state().sim.code.diagnostics());
        press(&mut h, "Save (Ctrl+S) •");
        let saved = attributes(&h.state().bim.session.model, &h.state().sim.library, block).unwrap();
        assert_eq!(saved.iter().find(|a| a.key == "source").and_then(|a| a.value.clone()), Some(Value::Text(code)));
        press(&mut h, "Discrete simulation");
        let run = h.state_mut().sim.run.as_mut().unwrap();
        run.advance(5.0);
        let expected = format!("{roller} runs: True 3000 mm");
        assert_eq!(run.sim.script_console().first().map(|l| l.text.as_str()), Some(expected.as_str()));
    }

    #[test]
    fn objects_placed_one_after_another_join_and_a_right_click_opens_a_sheet_that_edits_a_choice() {
        let mut h = app();
        press(&mut h, "Intralogistics");
        for name in ["Pallet source", "Roller conveyor", "Pallet sink"] {
            press(&mut h, name);
        }
        let (layout, _) = intracraft_model::layout::layout(&h.state().bim.session.model, &h.state().sim.library);
        assert_eq!(layout.connections.len(), 2, "source → roller → sink, each joined to the one before");
        let roller = layout.instances.iter().find(|i| i.definition_id == "conveyor.roller").unwrap();
        let id: bimcraft_core::model::Id = roller.id.parse().unwrap();
        // A right click halfway along the roller picks it and offers its properties.
        let Some(bimcraft_core::model::Element::Component(c)) = h.state().bim.session.model.elements.get(id) else { panic!() };
        let middle = h.state().bim.screen(P2(c.at.0 + 1500.0, c.at.1));
        h.hover_at(middle);
        h.run();
        for pressed in [true, false] {
            h.event(egui::Event::PointerButton {
                pos: middle,
                button: egui::PointerButton::Secondary,
                pressed,
                modifiers: Default::default(),
            });
        }
        h.run();
        assert_eq!(h.state().bim.selection, [id]);
        h.get_all_by_label("Properties…").last().unwrap().click();
        h.run();
        assert!(h.state().sim.sheets.is_floating(id));
        assert!(h.query_by_label_contains("· Properties").is_some(), "its window is open");
        for section in ["Geometry", "Motion", "Rollers"] {
            assert!(h.query_all_by_label(section).count() > 0, "{section}");
        }
        // Its direction is a choice: picked from the list, it becomes the object's own; undone, it is inherited again.
        let direction = |h: &Harness<IntraApp>| {
            let a = attributes(&h.state().bim.session.model, &h.state().sim.library, id).unwrap();
            let d = a.into_iter().find(|a| a.key == "direction").unwrap();
            (d.value.and_then(|v| v.text().map(str::to_string)), d.source)
        };
        assert_eq!(direction(&h).0.as_deref(), Some("forward"));
        h.get_all_by(|n| n.value().as_deref() == Some("forward") || n.label().as_deref() == Some("forward")).last().unwrap().click();
        h.run();
        h.get_all_by_label("reverse").last().unwrap().click();
        h.run();
        assert_eq!(direction(&h), (Some("reverse".into()), intracraft_model::attributes::Source::Instance));
        h.state_mut().bim.session.execute("undo", &serde_json::json!({})).unwrap();
        h.run();
        assert_eq!(direction(&h).0.as_deref(), Some("forward"));
        // Docked, the sheet shows in the side panel and its window closes.
        h.get_all_by_label("⇥ Dock").last().unwrap().click();
        h.run();
        assert!(!h.state().sim.sheets.is_floating(id));
        assert!(h.query_by_label("⇱ Float").is_some());
    }

    #[test]
    fn a_drag_in_3d_snaps_as_in_the_plan_and_moves_the_object_in_the_model() {
        let mut h = app();
        let st = h.state_mut();
        let lib = st.sim.library.clone();
        let first = intracraft_model::place(&mut st.bim.session, &lib, "conveyor.roller", None, [0.0, 0.0], 0.0, 0.0).unwrap();
        let second = intracraft_model::place(&mut st.bim.session, &lib, "conveyor.roller", None, [10_000.0, 0.0], 0.0, 0.0).unwrap();
        // Dragged 6.8 m west and 0.15 m north (−z) on the floor: it lines up 20 mm after the first one's exit.
        let by = snap_3d(&st.bim, &second.to_string(), [-6.8, -0.15]);
        assert!((by[0] + 6.98).abs() < 1e-9 && by[1].abs() < 1e-9, "{by:?}");
        st.sim.view_event(&mut st.bim, crate::view3d::Event3D::Moved(second.to_string(), by));
        let (layout, _) = intracraft_model::layout::layout(&st.bim.session.model, &lib);
        let joined: Vec<String> = layout.connections.iter().map(|c| format!("{}→{}", c.from.instance_id, c.to.instance_id)).collect();
        assert_eq!(joined, [format!("{first}→{second}")]);
        assert!(st.bim.status.text.starts_with("Connected to"), "{}", st.bim.status.text);
        // A double click opens its properties; a click on the floor lets the selection go.
        st.sim.view_event(&mut st.bim, crate::view3d::Event3D::Open(first.to_string()));
        assert!(st.sim.sheets.is_floating(first));
        st.sim.view_event(&mut st.bim, crate::view3d::Event3D::Select(None));
        assert!(st.bim.selection.is_empty());
    }

    #[test]
    fn the_warehouse_wizard_places_aisles_with_their_chain_conveyors_and_one_undo_takes_them_away() {
        let mut h = app();
        press(&mut h, "Intralogistics");
        press(&mut h, "Warehouse");
        assert!(h.state().sim.warehouse.open);
        // What the defaults give is shown before anything is placed: 2 aisles of 20 bays and 9 levels.
        assert!(h.query_by_label_contains("20 bays × 9 levels × 2 aisles: 720 storage positions").is_some());
        press(&mut h, "Create");
        assert!(!h.state().sim.warehouse.open);
        let (layout, problems) = intracraft_model::layout::layout(&h.state().bim.session.model, &h.state().sim.library);
        assert!(problems.is_empty(), "{problems:?}");
        let count = |definition: &str| layout.instances.iter().filter(|i| i.definition_id == definition).count();
        assert_eq!((count("storage.aisle"), count("conveyor.chain")), (2, 4));
        // Every conveyor is joined to its aisle: two in and two out.
        assert_eq!(layout.connections.len(), 4, "{:?}", layout.connections);
        assert_eq!(h.state().bim.selection.len(), 6);
        assert!(h.query_by_label_contains("Storage (2)").is_some());
        h.state_mut().bim.session.execute("undo", &serde_json::json!({})).unwrap();
        assert!(intracraft_model::layout::layout(&h.state().bim.session.model, &h.state().sim.library).0.instances.is_empty());
    }

    #[test]
    fn the_clock_reads_hours_minutes_and_seconds() {
        assert_eq!(clock(3725.5), "1:02:05.5");
    }
}
