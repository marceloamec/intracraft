//! An object's property sheet (ADR-0028): its name, then one section per group of its library definition (Geometry,
//! Rollers, Motion, Accumulation…), the advanced values folded away in each. Every value is edited with the control its
//! type asks for: a number with its unit, kept inside the library's range and step; a list for a choice and for a number
//! with allowed values; a tick box for yes or no; a field for text. A derived value is shown, not edited, and a value
//! that doesn't apply now (`dependsOn`) is greyed out. The square beside each says whether it is inherited (■, from the
//! library or the type) or the object's own (□): a click on □ inherits it again.
//!
//! Each change is applied when the field is let go (Enter, or the end of a drag): a command, undone with Ctrl+Z. A value
//! the library refuses is marked beside its field and changes nothing.
//!
//! A sheet opens in a window of its own (one per object, as many as wanted) or docked in the side panel, where it shows
//! the selected object. In the simulation modules it is read-only, with the object's state and statistics.

use std::collections::BTreeMap;

use bimcraft_core::model::Id;
use bimcraft_ui_egui::{BimApp, Status};
use egui::{Color32, ComboBox, Context, DragValue, Grid, RichText, ScrollArea, Ui};
use intracraft_library::Library;
use intracraft_library::definition::{Definition, ParamType, Parameter, Value};
use intracraft_model::attributes::{Attribute, Source, applies, attributes, definition_of, shown};

const ERROR: Color32 = Color32::from_rgb(200, 60, 60);

/// What a sheet shows of a run besides the values: the object's state and its statistics. None in the Design module.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct RunFacts {
    pub status: Option<&'static str>,
    pub statistics: Vec<(&'static str, f64)>,
}

/// What a sheet asks of the window, beyond editing its object.
#[derive(Clone, Debug, PartialEq)]
pub enum Asked {
    /// Show it docked in the side panel (and select its object).
    Dock(Id),
    /// Show it in a window of its own.
    Float(Id),
    /// Open the code editor on a code block's program.
    Code(Id, String),
}

/// The sheets open in windows, and what is being typed or dragged in any sheet until it is let go.
#[derive(Default)]
pub struct Sheets {
    floating: Vec<Id>,
    /// A number being dragged or typed: its object, key and value as shown (mm, °…).
    editing: Option<(Id, String, f64)>,
    /// Text being typed, by object and key ("" for the name).
    typed: BTreeMap<(Id, String), String>,
    /// Why the last value given to a field was refused.
    errors: BTreeMap<(Id, String), String>,
}

impl Sheets {
    /// Opens object `id`'s sheet in a window of its own (or brings it back to the front of the list).
    pub fn float(&mut self, id: Id) {
        self.floating.retain(|o| *o != id);
        self.floating.push(id);
    }

    pub fn is_floating(&self, id: Id) -> bool {
        self.floating.contains(&id)
    }

    /// The sheets in windows: each closes with its ✕, docks with its button, and closes by itself when its object is gone.
    /// `facts` gives what a run says of an object; none in the Design module, where the sheets edit.
    pub fn windows(&mut self, ctx: &Context, app: &mut BimApp, library: &Library, facts: Option<&dyn Fn(Id) -> RunFacts>) -> Vec<Asked> {
        let mut asked = Vec::new();
        self.floating.retain(|id| definition_of(&app.session.model, library, *id).is_some());
        for id in self.floating.clone() {
            let mut open = true;
            let title = format!("{} · Properties", title_of(app, library, id));
            egui::Window::new(title)
                .id(egui::Id::new(("intracraft-properties", id)))
                .open(&mut open)
                .default_width(400.0)
                .default_height(520.0)
                .resizable(true)
                .collapsible(true)
                .show(ctx, |ui| {
                    if ui.button("⇥ Dock").on_hover_text("Show these properties in the side panel").clicked() {
                        asked.push(Asked::Dock(id));
                    }
                    ScrollArea::vertical().show(ui, |ui| {
                        let f = facts.map(|f| f(id));
                        asked.extend(self.sheet(ui, app, library, id, f.as_ref()));
                    });
                });
            if !open {
                self.floating.retain(|o| *o != id);
            }
        }
        for a in &asked {
            if let Asked::Dock(id) = a {
                self.floating.retain(|o| o != id);
                app.selection = vec![*id];
            }
        }
        asked
    }

    /// The selected object's sheet, docked in the side panel; a word instead when it is open in its own window.
    pub fn docked(&mut self, ui: &mut Ui, app: &mut BimApp, library: &Library, facts: Option<RunFacts>) -> Vec<Asked> {
        ui.label(RichText::new("Properties").strong());
        let Some(&id) = app.selection.first() else {
            ui.label("Select an object, or right-click it and choose Properties…");
            return Vec::new();
        };
        if definition_of(&app.session.model, library, id).is_none() {
            ui.label(format!("{id} is not an intralogistics object."));
            return Vec::new();
        }
        if self.is_floating(id) {
            ui.label("Open in its own window.");
            return Vec::new();
        }
        let mut asked = Vec::new();
        if ui.button("⇱ Float").on_hover_text("Show these properties in a window of their own").clicked() {
            self.float(id);
            asked.push(Asked::Float(id));
        }
        asked.extend(self.sheet(ui, app, library, id, facts.as_ref()));
        asked
    }

    /// Object `id`'s sheet: editable without `facts`, read-only with them, which it shows first.
    fn sheet(&mut self, ui: &mut Ui, app: &mut BimApp, library: &Library, id: Id, facts: Option<&RunFacts>) -> Vec<Asked> {
        let model = std::sync::Arc::clone(&app.session.model);
        let Some(definition) = definition_of(&model, library, id) else { return Vec::new() };
        let attrs = match attributes(&model, library, id) {
            Ok(a) => a,
            Err(e) => {
                ui.colored_label(ERROR, e);
                return Vec::new();
            }
        };
        let editable = facts.is_none();
        let mut asked = Vec::new();
        let mut change: Option<Change> = None;
        Grid::new(("properties-head", id)).num_columns(2).show(ui, |ui| {
            ui.label("Name");
            let key = (id, String::new());
            let text = self.typed.entry(key.clone()).or_insert_with(|| intracraft_model::name_of(&model, id).unwrap_or_default());
            let r = ui.add_enabled(editable, egui::TextEdit::singleline(text).hint_text("(none)").desired_width(200.0));
            if r.lost_focus() {
                change = Some(Change::Name(text.clone()));
            }
            ui.end_row();
            ui.label("Object");
            ui.label(format!("{} · {id}", definition.name.en));
            ui.end_row();
        });
        if let Some(f) = facts {
            run_sections(ui, id, f);
        }
        for group in groups(&attrs) {
            let (basic, advanced): (Vec<&Attribute>, Vec<&Attribute>) =
                attrs.iter().filter(|a| a.group == group).partition(|a| !a.advanced);
            egui::CollapsingHeader::new(RichText::new(section_title(&group)).strong())
                .id_salt(("properties-section", id, &group))
                .default_open(true)
                .show(ui, |ui| {
                    ui.add_enabled_ui(editable, |ui| {
                        self.rows(ui, id, definition, &attrs, &basic, &mut change, &mut asked);
                        if !advanced.is_empty() {
                            egui::CollapsingHeader::new("Advanced")
                                .id_salt(("properties-advanced", id, &group))
                                .default_open(false)
                                .show(ui, |ui| self.rows(ui, id, definition, &attrs, &advanced, &mut change, &mut asked));
                        }
                    });
                });
        }
        if let Some(c) = change.filter(|_| editable) {
            self.apply(app, library, id, c);
        }
        asked
    }

    /// The rows of `shown`: label, control, inheritance.
    #[allow(clippy::too_many_arguments)]
    fn rows(
        &mut self,
        ui: &mut Ui,
        id: Id,
        definition: &Definition,
        all: &[Attribute],
        rows: &[&Attribute],
        change: &mut Option<Change>,
        asked: &mut Vec<Asked>,
    ) {
        Grid::new(("properties-rows", id, rows.first().map(|a| a.key.as_str()).unwrap_or_default())).num_columns(3).striped(true).show(
            ui,
            |ui| {
                for a in rows {
                    let applies = applies(a, all);
                    let label = ui.add_enabled(applies, egui::Label::new(&a.label.en));
                    let why = match (&a.depends_on, applies) {
                        (Some(rule), false) => format!("{}\n\nApplies only when {rule}.", a.description.en),
                        _ => a.description.en.clone(),
                    };
                    label.on_hover_text(why);
                    ui.add_enabled_ui(applies, |ui| {
                        ui.vertical(|ui| {
                            if let Some(c) = self.control(ui, id, definition, a, asked) {
                                *change = Some(c);
                            }
                            if let Some(e) = self.errors.get(&(id, a.key.clone())) {
                                ui.label(RichText::new(e).small().color(ERROR));
                            }
                        });
                    });
                    if let Some(c) = inheritance(ui, a) {
                        *change = Some(c);
                    }
                    ui.end_row();
                }
            },
        );
    }

    /// The control of attribute `a`, as its type asks; the change let go this frame, if any.
    fn control(&mut self, ui: &mut Ui, id: Id, definition: &Definition, a: &Attribute, asked: &mut Vec<Asked>) -> Option<Change> {
        let set = |v: Value| Some(Change::Set(a.key.clone(), v));
        if a.source == Source::Derived {
            ui.label(RichText::new(shown(a)).weak());
            return None;
        }
        if a.key == "source" && definition.id == intracraft_script::CODE_BLOCK {
            // A program doesn't fit a field: it opens in the code editor.
            if ui.button("Edit code…").on_hover_text("Open the IntraScript editor").clicked() {
                asked.push(Asked::Code(id, a.value.as_ref().and_then(Value::text).unwrap_or_default().to_string()));
            }
            return None;
        }
        let p = definition.parameter(&a.key)?;
        match a.kind {
            ParamType::Boolean => {
                let mut on = matches!(a.value, Some(Value::Bool(true)));
                if ui.checkbox(&mut on, "").changed() {
                    return set(Value::Bool(on));
                }
                None
            }
            ParamType::Enum => {
                let now = a.value.as_ref().and_then(Value::text).unwrap_or_default().to_string();
                let mut picked = None;
                ComboBox::from_id_salt(("properties-choice", id, &a.key)).selected_text(option_label(p, &now)).width(180.0).show_ui(
                    ui,
                    |ui| {
                        for o in &a.options {
                            if ui.selectable_label(*o == now, option_label(p, o)).clicked() && *o != now {
                                picked = Some(o.clone());
                            }
                        }
                    },
                );
                picked.and_then(|o| set(Value::Text(o)))
            }
            ParamType::Text => {
                let key = (id, a.key.clone());
                let text = self.typed.entry(key).or_insert_with(|| a.value.as_ref().and_then(Value::text).unwrap_or_default().to_string());
                let r = ui.add(egui::TextEdit::singleline(text).desired_width(180.0));
                if r.lost_focus() && Some(text.as_str()) != a.value.as_ref().and_then(Value::text) {
                    return set(Value::Text(text.clone()));
                }
                None
            }
            ParamType::Number | ParamType::Integer => {
                let unit = Unit::of(a);
                let now = a.value.as_ref().and_then(Value::number).unwrap_or(0.0);
                if let Some(choices) = &p.choices {
                    let mut picked = None;
                    let label = |x: f64| format!("{} {}", trim(unit.shown(x)), unit.label).trim().to_string();
                    ComboBox::from_id_salt(("properties-choice", id, &a.key)).selected_text(label(now)).width(180.0).show_ui(ui, |ui| {
                        for &c in choices {
                            if ui.selectable_label((c - now).abs() < 1e-9, label(c)).clicked() {
                                picked = Some(c);
                            }
                        }
                    });
                    return picked.filter(|c| (c - now).abs() > 1e-12).and_then(|c| set(Value::Number(c)));
                }
                let mine = self.editing.as_ref().filter(|(o, k, _)| *o == id && *k == a.key).map(|e| e.2);
                let mut shown_value = mine.unwrap_or_else(|| unit.shown(now));
                let (lo, hi) = (p.min.map_or(f64::NEG_INFINITY, |m| unit.shown(m)), p.max.map_or(f64::INFINITY, |m| unit.shown(m)));
                let integer = a.kind == ParamType::Integer;
                let step = p.step.map(|s| unit.shown(s)).filter(|s| *s > 0.0).unwrap_or(if integer { 1.0 } else { drag_step(shown_value) });
                let mut drag = DragValue::new(&mut shown_value).range(lo..=hi).speed(step * 0.25);
                if integer {
                    drag = drag.fixed_decimals(0);
                } else {
                    drag = drag.max_decimals(4);
                }
                if !unit.label.is_empty() {
                    drag = drag.suffix(format!(" {}", unit.label));
                }
                let r = ui.add(drag);
                if r.changed() {
                    self.editing = Some((id, a.key.clone(), shown_value));
                }
                let let_go = r.drag_stopped() || (r.lost_focus() && !r.dragged());
                if let_go && let Some((_, _, x)) = self.editing.take().filter(|(o, k, _)| *o == id && *k == a.key) {
                    let x = if integer { x.round() } else { x };
                    let si = unit.si(x);
                    if (si - now).abs() > 1e-12 {
                        return set(Value::Number(si));
                    }
                }
                None
            }
        }
    }

    fn apply(&mut self, app: &mut BimApp, library: &Library, id: Id, change: Change) {
        let (key, r) = match &change {
            Change::Name(name) => (String::new(), intracraft_model::set_name(&mut app.session, id, name)),
            Change::Set(key, v) => (key.clone(), intracraft_model::set_attribute(&mut app.session, library, id, key, v)),
            Change::Inherit(key) => (key.clone(), intracraft_model::reset_attribute(&mut app.session, library, id, key)),
        };
        self.typed.retain(|(o, _), _| *o != id);
        match r {
            Ok(()) => {
                self.errors.remove(&(id, key));
            }
            Err(e) => {
                app.status = Status { text: e.clone(), error: true };
                self.errors.insert((id, key), e);
            }
        }
    }
}

/// What a sheet changed this frame.
#[derive(Clone, Debug, PartialEq)]
enum Change {
    Name(String),
    Set(String, Value),
    /// The object's own value taken away: inherited again.
    Inherit(String),
}

/// The unit a number is shown in, and how it turns from SI.
struct Unit {
    label: String,
    factor: f64,
}

impl Unit {
    fn of(a: &Attribute) -> Unit {
        let si = a.unit.clone().filter(|u| !u.is_empty() && u != "ratio").unwrap_or_default();
        match a.display_unit.as_deref() {
            Some("mm") => Unit { label: "mm".into(), factor: 1000.0 },
            Some(d @ ("°" | "deg" | "°/s" | "°/s²")) => {
                Unit { label: if d == "deg" { "°".into() } else { d.into() }, factor: 180.0 / std::f64::consts::PI }
            }
            Some(d) if !d.is_empty() => Unit { label: d.into(), factor: 1.0 },
            _ => Unit { label: si, factor: 1.0 },
        }
    }

    fn shown(&self, si: f64) -> f64 {
        si * self.factor
    }

    fn si(&self, shown: f64) -> f64 {
        shown / self.factor
    }
}

/// The square beside a value: ■ inherited (from the library or the type), □ the object's own, which a click inherits again.
fn inheritance(ui: &mut Ui, a: &Attribute) -> Option<Change> {
    match a.source {
        Source::Derived => {
            ui.label(RichText::new("ƒ").weak()).on_hover_text("Worked out from the other values");
            None
        }
        Source::Library | Source::Type => {
            let from = if a.source == Source::Type { "its type" } else { "the library" };
            ui.label("■").on_hover_text(format!("Inherited from {from}: a change there reaches this object"));
            None
        }
        Source::Instance => ui
            .small_button("□")
            .on_hover_text("This object's own value: click to inherit it again")
            .clicked()
            .then(|| Change::Inherit(a.key.clone())),
    }
}

/// The object's state and statistics in a run, first in a read-only sheet.
fn run_sections(ui: &mut Ui, id: Id, f: &RunFacts) {
    egui::CollapsingHeader::new(RichText::new("State").strong()).id_salt(("properties-state", id)).default_open(true).show(ui, |ui| {
        ui.label(f.status.unwrap_or("—"));
    });
    egui::CollapsingHeader::new(RichText::new("Statistics").strong()).id_salt(("properties-stats", id)).default_open(true).show(ui, |ui| {
        if f.statistics.is_empty() {
            ui.label("No statistics yet.");
        }
        Grid::new(("properties-stats-grid", id)).num_columns(2).striped(true).show(ui, |ui| {
            for (k, v) in &f.statistics {
                ui.label(*k);
                ui.label(RichText::new(trim(*v)).monospace());
                ui.end_row();
            }
        });
    });
}

/// The groups of a definition's attributes, in the order they first come.
fn groups(attrs: &[Attribute]) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    for a in attrs {
        if !out.contains(&a.group) {
            out.push(a.group.clone());
        }
    }
    out
}

/// A group's key as a section's title: "rollers" → "Rollers".
fn section_title(group: &str) -> String {
    let mut c = group.chars();
    c.next().map_or_else(String::new, |f| f.to_uppercase().chain(c).collect())
}

fn option_label(p: &Parameter, option: &str) -> String {
    p.option_labels.as_ref().and_then(|l| l.get(option)).map_or_else(|| option.to_string(), |t| t.en.clone())
}

/// A drag's step for a value about this big: a hundredth of its size, in a round number.
fn drag_step(x: f64) -> f64 {
    let size = x.abs().max(1e-3);
    10f64.powf((size / 100.0).log10().floor())
}

fn title_of(app: &BimApp, library: &Library, id: Id) -> String {
    let model = &app.session.model;
    intracraft_model::name_of(model, id)
        .or_else(|| definition_of(model, library, id).map(|d| format!("{} {id}", d.name.en)))
        .unwrap_or_else(|| id.to_string())
}

fn trim(x: f64) -> String {
    let s = format!("{x:.4}");
    let s = s.trim_end_matches('0').trim_end_matches('.');
    if s == "-0" { "0".into() } else { s.into() }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn attr(display: Option<&str>, unit: Option<&str>) -> Attribute {
        Attribute {
            key: "k".into(),
            label: intracraft_library::definition::Text::en("k"),
            kind: ParamType::Number,
            unit: unit.map(Into::into),
            display_unit: display.map(Into::into),
            value: Some(Value::Number(0.2063)),
            description: intracraft_library::definition::Text::en("d"),
            source: Source::Library,
            group: "rollers".into(),
            advanced: false,
            options: vec![],
            depends_on: None,
        }
    }

    #[test]
    fn numbers_are_shown_in_their_display_unit_and_given_back_in_si() {
        let mm = Unit::of(&attr(Some("mm"), Some("m")));
        assert_eq!((mm.label.as_str(), trim(mm.shown(0.2063))), ("mm", "206.3".into()));
        assert!((mm.si(3000.0) - 3.0).abs() < 1e-12);
        let deg = Unit::of(&attr(Some("°/s"), Some("rad/s")));
        assert_eq!(trim(deg.shown(std::f64::consts::FRAC_PI_2)), "90");
        let speed = Unit::of(&attr(None, Some("m/s")));
        assert_eq!((speed.label.as_str(), speed.shown(0.25)), ("m/s", 0.25));
        assert_eq!(Unit::of(&attr(None, Some("ratio"))).label, "");
    }

    #[test]
    fn sections_follow_the_definitions_groups_in_order() {
        let lib = Library::built_in().unwrap();
        let mut s = bimcraft_core::engine::Session::new();
        let id = intracraft_model::place(&mut s, &lib, "conveyor.roller", None, [0.0, 0.0], 0.0, 0.0).unwrap();
        let g = groups(&attributes(&s.model, &lib, id).unwrap());
        assert!(g.len() > 2, "{g:?}");
        assert_eq!(g.iter().filter(|x| *x == "geometry").count(), 1);
        assert_eq!(section_title("rollers"), "Rollers");
        assert_eq!(drag_step(3000.0), 10.0);
        assert_eq!(drag_step(0.25), 0.001);
    }
}
