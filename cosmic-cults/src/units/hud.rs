use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};
use crate::units::components::*;
use crate::units::selection::SelectionState;

pub fn hud_plugin(app: &mut App) {
    app.add_systems(Update, (
        draw_resource_hud,
        draw_selection_hud,
    ));
}

fn draw_resource_hud(
    mut contexts: EguiContexts,
    player_resources: Query<&Resources, (With<Leader>, With<Unit>)>, // Assume player is leader for now
) {
    let Ok(ctx) = contexts.ctx_mut() else { return };

    egui::TopBottomPanel::top("resource_hud").show(ctx, |ui| {
        ui.horizontal(|ui| {
            if let Some(resources) = player_resources.iter().next() {
                ui.label(format!("Energy: {:.0}", resources.energy));
                ui.separator();
                ui.label(format!("Materials: {:.0}", resources.materials));
                ui.separator();
                ui.label(format!("Favor: {:.0}", resources.favor));
            } else {
                ui.label("Waiting for Leader...");
            }
        });
    });
}

fn draw_selection_hud(
    mut contexts: EguiContexts,
    selection: Res<SelectionState>,
    unit_query: Query<(&Unit, &Health, Option<&CombatStats>)>,
) {
    if selection.selected_entities.is_empty() {
        return;
    }

    let Ok(ctx) = contexts.ctx_mut() else { return };

    egui::SidePanel::right("selection_hud").show(ctx, |ui| {
        ui.heading("Selection");
        ui.separator();

        for &entity in &selection.selected_entities {
            if let Ok((unit, health, combat)) = unit_query.get(entity) {
                ui.group(|ui| {
                    ui.label(format!("Type: {}", unit.unit_type));
                    ui.label(format!("Health: {:.0}/{:.0}", health.current, health.maximum));
                    if let Some(stats) = combat {
                        ui.label(format!("Damage: {:.0}", stats.attack_damage));
                    }
                });
            }
        }
    });
}
