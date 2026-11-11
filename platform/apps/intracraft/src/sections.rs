//! Sections and load units in IntraCraft's windows (ADR-0029). Two panels, both called from `app.rs`:
//!
//! - [`sections_panel`]: in the Design module's ribbon, one button per section. A click hides or shows the section in the
//!   active view, through a view filter `Section <name>` that picks the components whose `Section` is that name (the
//!   `filter` command, then `graphics`). Also: put the selection in a new or another section, take it out of its section,
//!   and show every section again.
//! - [`pallet_panel`]: in the Discrete simulation's panel, the load units on the plant with their names (`MU3`), their
//!   addresses (`Seccion1.Conveyor1.MU3`), type, size, speed, and the attributes the code gave the selected one.
//!
//! Every change goes through `Session::execute` (undo, checks and the CLI work unchanged). The panels don't keep state of
//! the model; the text box and the selected load unit live in egui's memory.

use bimcraft_core::graphics::{Graphics, graphics_of};
use bimcraft_core::model::Id;
use bimcraft_ui_egui::{BimApp, Sheet, Status, ribbon_panel};
use egui::{Grid, RichText, ScrollArea, Ui};
use intracraft_des::flow::{FlowSimulation, Snapshot};
use intracraft_library::layout::Layout;
use intracraft_model::sections as model_sections;
use serde_json::{Value, json};

/// Load units listed at most, so a big plant doesn't stall the panel.
const MAX_ROWS: usize = 300;
/// The category the section filters pick from (components).
const CATEGORY: &str = "component";

/// The view filter that picks the components of section `section`: `Section Seccion1`.
pub fn filter_name(section: &str) -> String {
    format!("Section {section}")
}

/// The view the active tab is, for the graphics command: none for a sheet or a schedule (as BIMCraft's own).
fn active_view(app: &BimApp) -> Option<Value> {
    match app.sheet {
        Sheet::Plan(level) | Sheet::Ceiling(level) => {
            app.session.model.plan_view(level).is_none_or(|v| v.parent.is_none()).then(|| json!(level))
        }
        Sheet::Section(id) => Some(json!(id)),
        Sheet::View3D => Some(json!("{3D}")),
        _ => None,
    }
}

/// The graphics the active view holds, where it has any.
fn active_graphics(app: &BimApp) -> Option<&Graphics> {
    let m = &app.session.model;
    let id = match app.sheet {
        Sheet::Plan(id) | Sheet::Ceiling(id) | Sheet::Section(id) => Some(id),
        Sheet::View3D => m.default_3d(),
        _ => None,
    };
    id.and_then(|id| graphics_of(m, id))
}

/// Whether the active view hides section `section` now.
fn hidden_here(app: &BimApp, section: &str) -> bool {
    let name = filter_name(section);
    active_graphics(app).is_some_and(|g| g.filters.iter().any(|a| a.filter == name && a.over.hidden))
}

/// Shows or hides section `section` in the active view. Creates its filter the first time.
fn set_hidden_here(app: &mut BimApp, section: &str, hidden: bool) -> Result<(), String> {
    let Some(view) = active_view(app) else {
        return Err("This view has no graphics to override.".to_string());
    };
    let name = filter_name(section);
    if hidden && !app.session.model.view_filters.contains_key(&name) {
        app.session.execute(
            "filter",
            &json!({"name": name, "categories": [CATEGORY], "rules": [{"field": model_sections::SECTION, "op": "equals", "value": section}]}),
        )?;
    }
    app.session.execute("graphics", &json!({"view": view, "filters": [{"filter": name, "hidden": hidden}]})).map(|_| ())
}

/// The sections, one panel one button wide: a menu lists every section with a box to show or hide it in the active view
/// (the box is off while the section is hidden), and "Show all". The selection's actions are a second panel, one row:
/// a name box, "Put in", and a "More" menu with "Take out". The panels keep their width whatever the number of sections,
/// so the ribbon's other tabs still fit a narrow window.
pub fn sections_panel(ui: &mut Ui, app: &mut BimApp) {
    let names = model_sections::sections(&app.session.model);
    ribbon_panel(ui, "Sections", |ui| {
        if names.is_empty() {
            ui.label("None yet");
            return;
        }
        ui.menu_button("Sections", |ui| {
            for section in &names {
                let mut shown = !hidden_here(app, section);
                if ui.checkbox(&mut shown, section).changed()
                    && let Err(text) = set_hidden_here(app, section, !shown)
                {
                    report(app, Err(text));
                }
            }
            ui.separator();
            if ui.button("Show all").clicked() {
                let hidden: Vec<String> = names.iter().filter(|s| hidden_here(app, s)).cloned().collect();
                let result = hidden.iter().try_for_each(|s| set_hidden_here(app, s, false));
                report(app, result);
            }
        });
    });
    ribbon_panel(ui, "Section", |ui| {
        let key = egui::Id::new("section-name-to-put");
        let mut name: String = ui.data_mut(|d| d.get_temp(key).unwrap_or_default());
        ui.add(egui::TextEdit::singleline(&mut name).desired_width(90.0).hint_text("Name"));
        ui.data_mut(|d| d.insert_temp(key, name.clone()));
        let picked = app.selection.clone();
        if ui.button("Put in").clicked() {
            let result = put_in_section(app, &picked, name.trim());
            report(app, result);
        }
        ui.menu_button("More", |ui| {
            if ui.button("Take out").clicked() {
                let result = put_in_section(app, &picked, "");
                report(app, result);
            }
        });
    });
}

/// Puts each object of `ids` in `section` (empty: in none), as one step of the undo each.
fn put_in_section(app: &mut BimApp, ids: &[Id], section: &str) -> Result<(), String> {
    if ids.is_empty() {
        return Err("Select the objects first.".to_string());
    }
    for &id in ids {
        model_sections::set_section(&mut app.session, id, section)?;
    }
    Ok(())
}

fn report(app: &mut BimApp, result: Result<(), String>) {
    app.status = match result {
        Ok(()) => Status::default(),
        Err(text) => Status { text, error: true },
    };
}

/// The address of the object `carrier` in the layout: its section and its name, `Seccion1.Conveyor1`; empty when it isn't
/// in the layout.
fn carrier_address(layout: &Layout, carrier: &str) -> String {
    let Some(instance) = layout.instance(carrier) else { return String::new() };
    instance.attributes.get("address").and_then(|v| v.text()).map(str::to_string).unwrap_or_else(|| instance.name.clone())
}

/// The load units on the plant, and the facts of the selected one. Facts the snapshot doesn't carry (creation time, age)
/// are not shown.
pub fn pallet_panel(ui: &mut Ui, snapshot: &Snapshot, sim: &FlowSimulation, layout: &Layout) {
    ui.label(RichText::new("Load units").strong());
    if snapshot.pallets.is_empty() {
        ui.label("No load unit on the plant now.");
        return;
    }
    ui.label(format!("{} on the plant", snapshot.pallets.len()));
    let key = egui::Id::new("load-unit-selected");
    let mut selected: u64 = ui.data_mut(|d| d.get_temp(key).unwrap_or(0));
    ScrollArea::vertical().id_salt("load-units").max_height(260.0).show(ui, |ui| {
        Grid::new("load-units-grid").num_columns(5).striped(true).show(ui, |ui| {
            for heading in ["MU", "Address", "Type", "Size", "Speed"] {
                ui.label(RichText::new(heading).strong());
            }
            ui.end_row();
            for p in snapshot.pallets.iter().take(MAX_ROWS) {
                let name = model_sections::load_unit_name(p.id);
                let address = model_sections::load_unit_address(Some(&carrier_address(layout, &p.carrier)), &name);
                if ui.selectable_label(selected == p.id, &name).clicked() {
                    selected = p.id;
                }
                ui.label(address);
                ui.label(&p.type_id);
                ui.label(format!("{:.0} × {:.0} × {:.0} mm", p.length * 1000.0, p.width * 1000.0, p.height * 1000.0));
                ui.label(format!("{:.2} m/s", p.speed));
                ui.end_row();
            }
        });
    });
    ui.data_mut(|d| d.insert_temp(key, selected));
    if let Some(p) = snapshot.pallets.iter().find(|p| p.id == selected) {
        ui.separator();
        ui.label(RichText::new(model_sections::load_unit_name(p.id)).strong());
        let attributes = sim.pallet_attributes(p.id);
        Grid::new("load-unit-attributes").num_columns(2).striped(true).show(ui, |ui| {
            ui.label("Address");
            ui.label(model_sections::load_unit_address(Some(&carrier_address(layout, &p.carrier)), &model_sections::load_unit_name(p.id)));
            ui.end_row();
            ui.label("Type");
            ui.label(&p.type_id);
            ui.end_row();
            for (k, v) in attributes {
                ui.label(k);
                ui.label(v);
                ui.end_row();
            }
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use intracraft_library::definition::Value as LibValue;
    use intracraft_library::layout::{Instance, Vec3};

    #[test]
    fn a_section_filter_is_named_for_its_section() {
        assert_eq!(filter_name("Seccion1"), "Section Seccion1");
    }

    #[test]
    fn a_carrier_is_known_by_its_address_or_its_name() {
        let instance = |id: &str, name: &str, address: Option<&str>| Instance {
            id: id.into(),
            definition_id: "conveyor.roller".into(),
            definition_version: "0.1.0".into(),
            name: name.into(),
            position: Vec3::new(0.0, 0.0, 0.0),
            yaw: 0.0,
            params: Default::default(),
            attributes: address.map(|a| ("address".to_string(), LibValue::Text(a.into()))).into_iter().collect(),
        };
        let layout = Layout {
            instances: vec![instance("roller1", "Conveyor 1", Some("Seccion1.Conveyor 1")), instance("roller2", "Line B", None)],
            ..Layout::default()
        };
        assert_eq!(carrier_address(&layout, "roller1"), "Seccion1.Conveyor 1");
        assert_eq!(carrier_address(&layout, "roller2"), "Line B");
        assert_eq!(carrier_address(&layout, "missing"), "");
        assert_eq!(model_sections::load_unit_address(Some(&carrier_address(&layout, "roller1")), "MU3"), "Seccion1.Conveyor 1.MU3");
    }

    #[test]
    fn a_section_button_hides_its_section_in_the_active_view() {
        use egui_kittest::Harness;
        use egui_kittest::kittest::Queryable;
        let library = intracraft_library::Library::built_in().unwrap();
        let mut session = bimcraft_core::engine::Session::new();
        intracraft_model::load_library(&mut session, &library).unwrap();
        let id = intracraft_model::place(&mut session, &library, "conveyor.roller", None, [0.0, 0.0], 0.0, 0.0).unwrap();
        model_sections::set_section(&mut session, id, "Seccion1").unwrap();
        let level = session.model.levels().first().map(|(id, _)| *id).expect("a level");
        let mut app = BimApp::new(session);
        app.sheet = Sheet::Plan(level);
        let mut h = Harness::builder()
            .with_size([1800.0, 1000.0])
            .with_max_steps(16)
            .build_ui_state(|ui, app: &mut BimApp| sections_panel(ui, app), app);
        h.run();
        assert!(!hidden_here(h.state(), "Seccion1"));
        h.get_by_label("Sections").click();
        h.run();
        h.get_by_label("Seccion1").click();
        h.run();
        let app = h.state();
        assert!(app.session.model.view_filters.contains_key("Section Seccion1"), "{:?}", app.status.text);
        assert!(hidden_here(app, "Seccion1"));
        // The box again shows it (the menu closes after each change, so it is opened again).
        h.get_by_label("Sections").click();
        h.run();
        h.get_by_label("Seccion1").click();
        h.run();
        assert!(!hidden_here(h.state(), "Seccion1"));
    }

    #[test]
    fn the_panels_stay_narrow_in_a_1800_pixel_window() {
        use egui_kittest::Harness;
        use egui_kittest::kittest::Queryable;
        let library = intracraft_library::Library::built_in().unwrap();
        let mut session = bimcraft_core::engine::Session::new();
        intracraft_model::load_library(&mut session, &library).unwrap();
        let id = intracraft_model::place(&mut session, &library, "conveyor.roller", None, [0.0, 0.0], 0.0, 0.0).unwrap();
        model_sections::set_section(&mut session, id, "Seccion1").unwrap();
        let mut h = Harness::builder()
            .with_size([1800.0, 1000.0])
            .with_max_steps(16)
            .build_ui_state(|ui, app: &mut BimApp| sections_panel(ui, app), BimApp::new(session));
        h.run();
        let left = h.get_by_label("Sections").rect().min.x;
        let right = h.get_by_label("More").rect().max.x;
        let width = right - left;
        println!("sections panels: {width:.0} px wide, from x {left:.0}");
        assert!(width < 330.0, "the two panels take {width} px");
    }

    #[test]
    fn selecting_nothing_to_put_in_a_section_says_so() {
        let mut app = BimApp::new(bimcraft_core::engine::Session::new());
        let e = put_in_section(&mut app, &[], "Seccion1").unwrap_err();
        assert!(e.contains("Select"), "{e}");
    }
}
