//! What an intralogistics object can do from its right-click menu, in the floor plan and in the 3D view alike (ADR-0028):
//! open its properties; in the Design module also turn it a quarter turn either way about its entry, and copy it to
//! the end of its free exit, joined to it. BIMCraft's own items (Delete, Hide…) follow them in the plan's menu.

use bimcraft_core::model::{Element, Id};
use bimcraft_ui_egui::{BimApp, ExtensionAction, Status};
use intracraft_library::Library;
use intracraft_model::attributes::definition_of;
use serde_json::json;

pub const PROPERTIES: &str = "intracraft.properties";
pub const TURN_LEFT: &str = "intracraft.turn-left";
pub const TURN_RIGHT: &str = "intracraft.turn-right";
pub const DUPLICATE_AFTER: &str = "intracraft.duplicate-after";

/// The items the selection offers now: none unless it is one intralogistics object. Only Properties while a run shows
/// (`design` false): the run's layout is a copy, not to edit.
pub fn offered(app: &BimApp, library: &Library, design: bool) -> Vec<ExtensionAction> {
    let model = &app.session.model;
    let &[id] = app.selection.as_slice() else { return Vec::new() };
    let Some(d) = definition_of(model, library, id) else { return Vec::new() };
    let item = |id: &str, label: &str, enabled: bool| ExtensionAction { id: id.into(), label: label.into(), enabled };
    let mut items = vec![item(PROPERTIES, "Properties…", true)];
    if design {
        let after = intracraft_model::snap::next_after(model, library, id, &d.id).is_some();
        items.extend([
            item(TURN_LEFT, "Turn 90° left", true),
            item(TURN_RIGHT, "Turn 90° right", true),
            item(DUPLICATE_AFTER, "Duplicate after its exit", after),
        ]);
    }
    items
}

/// Does what item `action` says to the selected object; asks for its properties sheet through `open`.
pub fn act(action: &str, app: &mut BimApp, library: &Library, open: &mut dyn FnMut(Id)) {
    let &[id] = app.selection.as_slice() else { return };
    let r = match action {
        PROPERTIES => {
            open(id);
            Ok(())
        }
        TURN_LEFT => turn(app, id, 90.0),
        TURN_RIGHT => turn(app, id, -90.0),
        DUPLICATE_AFTER => intracraft_model::snap::duplicate_after(&mut app.session, library, id).map(|copy| app.selection = vec![copy]),
        _ => Ok(()),
    };
    if let Err(e) = r {
        app.status = Status { text: e, error: true };
    }
}

/// Turns object `id` by `angle` degrees (counter-clockwise) about its entry.
pub fn turn(app: &mut BimApp, id: Id, angle: f64) -> Result<(), String> {
    let Some(Element::Component(c)) = app.session.model.elements.get(id) else { return Err(format!("{id} is not a placed object")) };
    let about = [c.at.0, c.at.1];
    app.session.execute("rotate", &json!({"ids": [id], "about": about, "angle": angle})).map(|_| ())
}

#[cfg(test)]
mod tests {
    use super::*;
    use bimcraft_core::engine::Session;

    fn app_with_roller() -> (BimApp, Library, Id) {
        let lib = Library::built_in().unwrap();
        let mut s = Session::new();
        let id = intracraft_model::place(&mut s, &lib, "conveyor.roller", None, [0.0, 0.0], 0.0, 0.0).unwrap();
        let mut app = BimApp::new(s);
        app.selection = vec![id];
        (app, lib, id)
    }

    fn ids(items: &[ExtensionAction]) -> Vec<&str> {
        items.iter().map(|a| a.id.as_str()).collect()
    }

    #[test]
    fn an_object_offers_its_properties_and_in_design_its_turns_and_a_copy_after_it() {
        let (mut app, lib, _) = app_with_roller();
        assert_eq!(ids(&offered(&app, &lib, true)), [PROPERTIES, TURN_LEFT, TURN_RIGHT, DUPLICATE_AFTER]);
        assert_eq!(ids(&offered(&app, &lib, false)), [PROPERTIES]);
        app.selection.clear();
        assert!(offered(&app, &lib, true).is_empty());
    }

    #[test]
    fn a_turn_keeps_the_entry_and_a_copy_goes_after_the_exit() {
        let (mut app, lib, id) = app_with_roller();
        act(TURN_LEFT, &mut app, &lib, &mut |_| {});
        let Some(Element::Component(c)) = app.session.model.elements.get(id) else { panic!() };
        assert_eq!((c.at.0, c.at.1, c.angle), (0.0, 0.0, 90.0));
        act(DUPLICATE_AFTER, &mut app, &lib, &mut |_| {});
        let copy = app.selection[0];
        assert_ne!(copy, id);
        let Some(Element::Component(c)) = app.session.model.elements.get(copy) else { panic!() };
        assert!((c.at.0).abs() < 1e-6 && (c.at.1 - 3020.0).abs() < 1e-6, "{:?}", c.at);
        let mut opened = None;
        act(PROPERTIES, &mut app, &lib, &mut |o| opened = Some(o));
        assert_eq!(opened, Some(copy));
    }
}
