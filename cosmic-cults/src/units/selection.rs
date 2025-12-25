use bevy::prelude::*;
use crate::units::components::*;

#[derive(Resource, Default)]
pub struct SelectionState {
    pub selected_entities: Vec<Entity>,
}

pub fn selection_plugin(app: &mut App) {
    app.init_resource::<SelectionState>()
       .add_systems(Update, update_selection_visuals)
       .add_observer(on_unit_click)
       .add_observer(on_ground_click);
}

fn on_unit_click(
    trigger: On<Pointer<Click>>,
    mut selection: ResMut<SelectionState>,
    mut commands: Commands,
    unit_query: Query<Entity, With<Unit>>,
    keyboard: Res<ButtonInput<KeyCode>>,
    selected_query: Query<Entity, With<Selected>>,
) {
    if trigger.button != PointerButton::Primary {
        return;
    }

    let entity = trigger.entity;
    // We check if the clicked entity is a unit
    if unit_query.get(entity).is_err() {
        return;
    }

    let shift = keyboard.pressed(KeyCode::ShiftLeft) || keyboard.pressed(KeyCode::ShiftRight);

    if !shift {
        // Deselect others
        for selected in selected_query.iter() {
            commands.entity(selected).remove::<Selected>();
        }
        selection.selected_entities.clear();
    }

    commands.entity(entity).insert(Selected);
    selection.selected_entities.push(entity);
}

fn on_ground_click(
    trigger: On<Pointer<Click>>,
    selected_query: Query<Entity, With<Selected>>,
    mut unit_query: Query<&mut MovementPath>,
) {
    if trigger.button != PointerButton::Secondary {
        return;
    }

    // Get the hit position
    let Some(hit_pos) = trigger.hit.position else { return };

    for entity in selected_query.iter() {
        if let Ok(mut path) = unit_query.get_mut(entity) {
            path.waypoints = vec![hit_pos];
            path.current_index = 0;
        }
    }
}

pub fn update_selection_visuals(
    mut query: Query<(&mut Visibility, &ChildOf), With<crate::units::visuals::SelectionIndicator>>,
    selected_query: Query<&Selected>,
) {
    for (mut visibility, child_of) in query.iter_mut() {
        if selected_query.get(child_of.parent()).is_ok() {
            *visibility = Visibility::Visible;
        } else {
            *visibility = Visibility::Hidden;
        }
    }
}
