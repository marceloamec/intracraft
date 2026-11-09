//! The window's panel: the run's controls and clock, what is in the plant, the warnings, and the picked object's state and
//! statistics.

use bevy::prelude::*;
use bevy_egui::{EguiContexts, egui};

use crate::camera::Picked;
use crate::run::{Module, Run};
use crate::view::{Latest, Objects, ShowStatus};

/// The file the window shows, for its title.
#[derive(Resource)]
pub struct FileName(pub String);

pub fn panel(
    mut contexts: EguiContexts,
    mut run: NonSendMut<Run>,
    objects: Res<Objects>,
    picked: Res<Picked>,
    latest: Res<Latest>,
    file: Res<FileName>,
    mut show: ResMut<ShowStatus>,
) -> Result {
    let ctx = contexts.ctx_mut()?;
    egui::Window::new("IntraCraft 3D").default_pos([12.0, 12.0]).default_width(300.0).resizable(false).show(ctx, |ui| {
        let module = match run.module {
            Module::Discrete => "Discrete simulation",
            Module::Physics => "Physics simulation (Avian)",
        };
        ui.label(egui::RichText::new(module).strong());
        ui.label(egui::RichText::new(&file.0).small());
        ui.separator();
        ui.horizontal(|ui| {
            let label = if run.playing { "⏸ Pause" } else { "▶ Run" };
            if ui.button(label).clicked() {
                run.playing = !run.playing;
            }
            if ui.button("⟲ Restart").clicked() {
                run.restart();
            }
            let current = run.speed;
            egui::ComboBox::from_id_salt("speed").selected_text(format!("×{current}")).show_ui(ui, |ui| {
                for &s in run.speeds() {
                    if ui.selectable_label(s == current, format!("×{s}")).clicked() {
                        run.speed = s;
                    }
                }
            });
        });
        ui.label(egui::RichText::new(run.clock()).monospace().size(20.0));
        let stats = run.statistics();
        let delivered: f64 = objects
            .0
            .iter()
            .filter(|o| o.definition == "flow.sink")
            .filter_map(|o| stats.get(&o.id).and_then(|s| s.get("palletsReceived")))
            .sum();
        egui::Grid::new("counts").num_columns(2).show(ui, |ui| {
            ui.label("Pallets in the plant");
            ui.label(latest.0.pallets.len().to_string());
            ui.end_row();
            ui.label("Pallets delivered");
            ui.label(format!("{delivered:.0}"));
            ui.end_row();
            if let Some(fallen) = run.fallen() {
                ui.label("Pallets fallen");
                ui.label(fallen.to_string());
                ui.end_row();
            }
        });
        ui.checkbox(&mut show.0, "State colours under the objects");
        if let Some(e) = &run.error {
            ui.colored_label(egui::Color32::from_rgb(220, 60, 50), e);
        }
        let issues = run.issues();
        if !issues.is_empty() {
            egui::CollapsingHeader::new(format!("Warnings ({})", issues.len())).show(ui, |ui| {
                for i in &issues {
                    ui.label(egui::RichText::new(i).small());
                }
            });
        }
        ui.separator();
        match picked.0.and_then(|k| objects.0.get(k)) {
            Some(o) => {
                let name = if o.name.is_empty() { o.id.clone() } else { format!("{} ({})", o.name, o.id) };
                ui.label(egui::RichText::new(name).strong());
                ui.label(egui::RichText::new(&o.definition).small());
                if let Some(state) = latest.0.status.get(&o.id) {
                    ui.label(format!("State: {state}"));
                }
                if let Some(s) = stats.get(&o.id) {
                    egui::Grid::new("statistics").num_columns(2).striped(true).show(ui, |ui| {
                        for (key, value) in s {
                            ui.label(*key);
                            ui.label(format!("{value:.2}"));
                            ui.end_row();
                        }
                    });
                }
            }
            None => {
                ui.label(egui::RichText::new("Click an object to see its state and statistics.").small());
            }
        }
        ui.separator();
        ui.label(egui::RichText::new("Left drag: orbit · Right drag: pan · Wheel: zoom · F: whole plant").small());
    });
    Ok(())
}
