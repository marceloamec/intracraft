//! The high-bay warehouse wizard (A22, ADR-0033): a window of the Design module that asks for a warehouse's size, shows
//! what that gives (bays, levels, positions, measures and an estimate of each crane's cycle), and places its storage
//! aisles (S26) with their pick-up and deposit chain conveyors in one step. What to place is worked out by
//! `intracraft_des::flow::warehouse::plan`, the same the CLI writes as a layout file.

use bimcraft_ui_egui::{BimApp, Status};
use egui::{ComboBox, DragValue, Grid, RichText, Ui};
use intracraft_des::flow::warehouse::{AISLE, Station, StationUse, WarehousePlan, WarehouseSpec, aisles_that_fit, plan};
use intracraft_library::Library;
use intracraft_library::definition::Value;
use intracraft_library::layout::Vec3;
use intracraft_library::params::{defaults, num};

/// The aisle values the wizard asks beside the size: the crane's and the operation's, with what they are shown in.
const CRANE: [(&str, &str, &str); 6] = [
    ("travelSpeed", "Travel speed", "m/s"),
    ("travelAccel", "Travel acceleration", "m/s²"),
    ("liftSpeed", "Lift speed", "m/s"),
    ("liftAccel", "Lift acceleration", "m/s²"),
    ("forkTime", "Fork cycle", "s"),
    ("forkTimeDouble", "Fork cycle, rear position", "s"),
];
const RETRIEVAL: [(&str, &str); 3] =
    [("dwell", "After a dwell time"), ("interval", "Orders at an interval"), ("none", "None: pallets stay")];
const USES: [(StationUse, &str); 3] = [(StationUse::InAndOut, "In and out"), (StationUse::In, "In only"), (StationUse::Out, "Out only")];

/// The wizard's window and what is typed in it.
pub struct Wizard {
    pub open: bool,
    spec: WarehouseSpec,
    /// Whether the level pitch is typed, instead of following the load height, and its typed value, m.
    own_pitch: bool,
    pitch: f64,
    /// The width the aisles should fit in, m.
    width: f64,
    /// The crane's values, by key (SI); those left at the library's default are not set on the aisles.
    crane: Vec<f64>,
    initial_fill: f64,
    retrieval: String,
    dwell_time: f64,
    retrieval_interval: f64,
    /// Where the first aisle's front goes on the plan, mm east and north, and how the warehouse is turned, degrees.
    at: [f64; 2],
    angle: f64,
}

impl Wizard {
    pub fn new(library: &Library) -> Wizard {
        let aisle = library.get(AISLE).map(defaults).unwrap_or_default();
        Wizard {
            open: false,
            spec: WarehouseSpec::default(),
            own_pitch: false,
            pitch: num(&aisle, "levelPitch", 1.8),
            width: 20.0,
            crane: CRANE.iter().map(|(key, _, _)| num(&aisle, key, 0.0)).collect(),
            initial_fill: num(&aisle, "initialFill", 0.5),
            retrieval: aisle.get("retrievalMode").and_then(Value::text).unwrap_or("dwell").to_string(),
            dwell_time: num(&aisle, "dwellTime", 3600.0),
            retrieval_interval: num(&aisle, "retrievalInterval", 120.0),
            at: [0.0, 0.0],
            angle: 0.0,
        }
    }

    /// Opens the window, the warehouse to go at `at` (mm, plan).
    pub fn ask(&mut self, at: [f64; 2]) {
        self.open = true;
        self.at = at;
    }

    /// What is typed as a specification: the aisles' own values are those that differ from the library's.
    pub fn spec(&self, library: &Library) -> WarehouseSpec {
        let aisle = library.get(AISLE).map(defaults).unwrap_or_default();
        let mut spec = self.spec.clone();
        spec.level_pitch = self.own_pitch.then_some(self.pitch);
        let mut set = |key: &str, v: Value| {
            if aisle.get(key) != Some(&v) {
                spec.aisle_values.insert(key.to_string(), v);
            }
        };
        for ((key, _, _), v) in CRANE.iter().zip(&self.crane) {
            set(key, Value::Number(*v));
        }
        set("initialFill", Value::Number(self.initial_fill));
        set("retrievalMode", Value::Text(self.retrieval.clone()));
        set("dwellTime", Value::Number(self.dwell_time));
        set("retrievalInterval", Value::Number(self.retrieval_interval));
        // The plan's x east and y north are the simulation's x and −z; a plan angle is the yaw.
        spec.origin = Vec3::new(self.at[0] / 1000.0, 0.0, -self.at[1] / 1000.0);
        spec.yaw = self.angle.to_radians();
        spec
    }

    /// The window, while it is open; `Create` places the warehouse in the model, selects it and closes the window.
    pub fn show(&mut self, ctx: &egui::Context, app: &mut BimApp, library: &Library) {
        if !self.open {
            return;
        }
        let mut open = true;
        let mut create = None;
        egui::Window::new("High-bay warehouse")
            .id(egui::Id::new("intracraft-warehouse-wizard"))
            .open(&mut open)
            .default_width(460.0)
            .resizable(true)
            .collapsible(true)
            .show(ctx, |ui| {
                egui::ScrollArea::vertical().max_height(620.0).show(ui, |ui| self.form(ui, library));
                ui.separator();
                let planned = plan(&self.spec(library), library);
                match &planned {
                    Ok(p) => summary(ui, &self.spec, p),
                    Err(e) => {
                        ui.colored_label(ui.visuals().error_fg_color, e);
                    }
                }
                ui.separator();
                ui.horizontal(|ui| {
                    let tip = "Place the aisles and their chain conveyors on the plan; one Undo takes them away";
                    if ui.add_enabled(planned.is_ok(), egui::Button::new("Create")).on_hover_text(tip).clicked() {
                        create = planned.ok();
                    }
                    ui.label("The front zone (loop, entries, exits) is drawn with the library afterwards.");
                });
            });
        self.open = open;
        if let Some(planned) = create {
            match intracraft_model::place_all(&mut app.session, library, &planned.instances, "High-bay warehouse") {
                Ok(ids) => {
                    let n = planned.instances.iter().filter(|i| i.definition_id == AISLE).count();
                    app.status =
                        Status { text: format!("Placed a warehouse of {n} aisles and {} positions", planned.positions), error: false };
                    app.selection = ids;
                    self.open = false;
                }
                Err(e) => app.status = Status { text: e, error: true },
            }
        }
    }

    fn form(&mut self, ui: &mut Ui, library: &Library) {
        ui.label(RichText::new("Size").strong());
        Grid::new("warehouse-size").num_columns(2).spacing([12.0, 4.0]).show(ui, |ui| {
            ui.label("Rack length");
            ui.add(metres(&mut self.spec.length, 1.0, 240.0));
            ui.end_row();
            ui.label("Rack height");
            ui.add(metres(&mut self.spec.height, 1.0, 46.0));
            ui.end_row();
            ui.label("Aisles");
            ui.horizontal(|ui| {
                ui.add(DragValue::new(&mut self.spec.aisles).range(1..=20));
                ui.label("or as many as fit in");
                ui.add(metres(&mut self.width, 1.0, 400.0));
                let tip = "The most aisles that fit across this width";
                if ui.button("Fit").on_hover_text(tip).clicked() {
                    let own = self.spec(library).aisle_values;
                    self.spec.aisles = aisles_that_fit(self.width, self.spec.deep, &own, library).max(1);
                }
            });
            ui.end_row();
            ui.label("Depth");
            ui.horizontal(|ui| {
                ui.radio_value(&mut self.spec.deep, 1, "Single deep");
                ui.radio_value(&mut self.spec.deep, 2, "Double deep");
            });
            ui.end_row();
            ui.label("Load height on the pallet");
            ui.add(metres(&mut self.spec.load_height, 0.0, 2.5));
            ui.end_row();
            ui.label("Level pitch");
            ui.horizontal(|ui| {
                ui.checkbox(&mut self.own_pitch, "set it myself");
                ui.add_enabled(self.own_pitch, metres(&mut self.pitch, 0.6, 3.0));
            });
            ui.end_row();
        });
        ui.add_space(6.0);
        ui.label(RichText::new("Pick-up and deposit stations (chain conveyors)").strong());
        Grid::new("warehouse-stations").num_columns(3).spacing([12.0, 4.0]).show(ui, |ui| {
            let mut remove = None;
            for (n, station) in self.spec.stations.iter_mut().enumerate() {
                ui.label(format!("Height {}", n + 1));
                ui.add(metres(&mut station.height, 0.2, 40.0));
                ui.horizontal(|ui| {
                    let now = USES.iter().find(|(u, _)| *u == station.usage).map_or("", |(_, label)| label);
                    ComboBox::from_id_salt(("warehouse-station", n)).selected_text(now).show_ui(ui, |ui| {
                        for (usage, label) in USES {
                            ui.selectable_value(&mut station.usage, usage, label);
                        }
                    });
                    if n > 0 && ui.small_button("✖").on_hover_text("Take this height away").clicked() {
                        remove = Some(n);
                    }
                });
                ui.end_row();
            }
            if let Some(n) = remove {
                self.spec.stations.remove(n);
            }
            if self.spec.stations.len() < 3 {
                if ui.button("Add a height").clicked() {
                    let above = self.spec.stations.last().map_or(0.6, |s| s.height + 4.0);
                    self.spec.stations.push(Station { height: above, usage: StationUse::Out });
                }
                ui.end_row();
            }
            ui.label("Conveyor length");
            ui.add(metres(&mut self.spec.conveyor_length, 0.9, 12.0));
            ui.end_row();
        });
        ui.add_space(6.0);
        ui.collapsing("Stacker crane", |ui| {
            Grid::new("warehouse-crane").num_columns(2).spacing([12.0, 4.0]).show(ui, |ui| {
                for ((_, label, unit), v) in CRANE.iter().zip(&mut self.crane) {
                    ui.label(*label);
                    ui.add(DragValue::new(v).range(0.1..=90.0).speed(0.05).max_decimals(2).suffix(format!(" {unit}")));
                    ui.end_row();
                }
            });
        });
        ui.collapsing("Operation", |ui| {
            Grid::new("warehouse-operation").num_columns(2).spacing([12.0, 4.0]).show(ui, |ui| {
                ui.label("Filled at the start");
                let mut percent = self.initial_fill * 100.0;
                ui.add(DragValue::new(&mut percent).range(0.0..=100.0).speed(1.0).fixed_decimals(0).suffix(" %"));
                self.initial_fill = percent / 100.0;
                ui.end_row();
                ui.label("Retrieval");
                let now = RETRIEVAL.iter().find(|(id, _)| *id == self.retrieval).map_or("", |(_, label)| label);
                ComboBox::from_id_salt("warehouse-retrieval").selected_text(now).show_ui(ui, |ui| {
                    for (id, label) in RETRIEVAL {
                        ui.selectable_value(&mut self.retrieval, id.to_string(), label);
                    }
                });
                ui.end_row();
                if self.retrieval == "dwell" {
                    ui.label("Dwell time");
                    ui.add(DragValue::new(&mut self.dwell_time).range(1.0..=2_592_000.0).speed(10.0).fixed_decimals(0).suffix(" s"));
                    ui.end_row();
                }
                if self.retrieval == "interval" {
                    ui.label("An order every");
                    ui.add(DragValue::new(&mut self.retrieval_interval).range(1.0..=86_400.0).speed(1.0).fixed_decimals(0).suffix(" s"));
                    ui.end_row();
                }
            });
        });
        ui.collapsing("Place", |ui| {
            Grid::new("warehouse-place").num_columns(2).spacing([12.0, 4.0]).show(ui, |ui| {
                ui.label("Front of the first aisle");
                ui.horizontal(|ui| {
                    ui.add(DragValue::new(&mut self.at[0]).speed(100.0).fixed_decimals(0).prefix("x ").suffix(" mm"));
                    ui.add(DragValue::new(&mut self.at[1]).speed(100.0).fixed_decimals(0).prefix("y ").suffix(" mm"));
                });
                ui.end_row();
                ui.label("Turned");
                ui.add(DragValue::new(&mut self.angle).range(-360.0..=360.0).speed(5.0).fixed_decimals(0).suffix("°"));
                ui.end_row();
            });
        });
    }
}

/// A length typed in metres between `lo` and `hi`.
fn metres(v: &mut f64, lo: f64, hi: f64) -> DragValue<'_> {
    DragValue::new(v).range(lo..=hi).speed(0.1).max_decimals(2).suffix(" m")
}

/// What the typed values give (A22 §2).
fn summary(ui: &mut Ui, spec: &WarehouseSpec, p: &WarehousePlan) {
    let (single, combined) = p.pallets_per_hour();
    ui.label(
        RichText::new(format!("{} bays × {} levels × {} aisles: {} storage positions", p.bays, p.levels, spec.aisles, p.positions))
            .strong(),
    );
    ui.label(format!(
        "Racks {:.1} m long and {:.1} m high, levels every {:.0} mm; {:.1} m wide in all",
        p.size[0],
        p.size[2],
        p.level_pitch * 1000.0,
        p.size[1]
    ));
    ui.label(format!(
        "Each crane, estimated: single cycle {:.0} s ({single:.0} pallets/h in); combined cycle {:.0} s ({combined:.0} pallets/h in and out)",
        p.single_cycle, p.combined_cycle
    ));
}
