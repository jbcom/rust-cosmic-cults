use crate::{Formation, FormationType, Leader, MovementPath, MovementTarget, Selected, Unit};
use bevy::prelude::*;
#[cfg(feature = "web")]
use web_sys::console;

// Formation system for group movement
pub fn formation_system(
    mut unit_query: Query<(&mut MovementTarget, &Transform, &mut Formation), With<Selected>>,
    selection_state: Res<crate::SelectionState>,
) {
    if selection_state.selected_entities.len() > 1 {
        // Apply formation spacing when multiple units are selected
        let formation_spacing = 2.0;
        let units_per_row = (selection_state.selected_entities.len() as f32)
            .sqrt()
            .ceil() as usize;

        for (i, (mut target, transform, mut formation)) in unit_query.iter_mut().enumerate() {
            if !target.reached {
                // Calculate formation offset based on formation type
                let offset = calculate_formation_offset(
                    &formation.formation_type,
                    i,
                    selection_state.selected_entities.len(),
                    formation.spacing,
                );

                // Apply formation offset to movement target
                target.x += offset.x;
                target.y += offset.y;

                // Update formation position
                formation.position_in_formation = offset;
            }
        }
    }
}

// Advanced formation system with leader-based formations
pub fn leader_formation_system(
    mut unit_query: Query<
        (&mut MovementTarget, &Transform, &mut Formation, &Unit),
        Without<Leader>,
    >,
    leader_query: Query<(&Transform, &Leader), With<Leader>>,
    selection_state: Res<crate::SelectionState>,
) {
    // Find selected leader if any
    let selected_leader = leader_query.iter().find(|(_, leader)| {
        selection_state
            .selected_entities
            .iter()
            .any(|entity| leader_query.get(*entity).is_ok())
    });

    if let Some((leader_transform, leader)) = selected_leader {
        // Apply leader-centered formation
        let mut formation_index = 0;

        for (mut target, transform, mut formation, unit) in unit_query.iter_mut() {
            // Only apply to units of the same cult as the leader
            if unit.cult == leader.cult && !target.reached {
                let offset = calculate_leader_formation_offset(
                    &formation.formation_type,
                    formation_index,
                    formation.spacing,
                    leader_transform.translation,
                );

                target.x = leader_transform.translation.x + offset.x;
                target.y = leader_transform.translation.z + offset.y; // Z becomes Y in 2D formation

                formation.leader_entity = Some(Entity::PLACEHOLDER); // Would need actual leader entity
                formation.position_in_formation = offset;

                formation_index += 1;
            }
        }
    }
}

// Formation type switching system
pub fn formation_switching_system(
    input: Res<ButtonInput<KeyCode>>,
    mut formation_query: Query<&mut Formation, With<Selected>>,
) {
    if input.just_pressed(KeyCode::Digit1) {
        set_formation_type(&mut formation_query, FormationType::Line);
    } else if input.just_pressed(KeyCode::Digit2) {
        set_formation_type(&mut formation_query, FormationType::Column);
    } else if input.just_pressed(KeyCode::Digit3) {
        set_formation_type(&mut formation_query, FormationType::Box);
    } else if input.just_pressed(KeyCode::Digit4) {
        set_formation_type(&mut formation_query, FormationType::Wedge);
    } else if input.just_pressed(KeyCode::Digit5) {
        set_formation_type(&mut formation_query, FormationType::Circle);
    }
}

// Helper function to set formation type for selected units
fn set_formation_type(
    formation_query: &mut Query<&mut Formation, With<Selected>>,
    formation_type: FormationType,
) {
    for mut formation in formation_query.iter_mut() {
        formation.formation_type = formation_type.clone();

        #[cfg(feature = "web")]
        console::log_1(&format!("Formation changed to {:?}", formation_type).into());
    }
}

// Calculate formation offset based on formation type
fn calculate_formation_offset(
    formation_type: &FormationType,
    index: usize,
    total_units: usize,
    spacing: f32,
) -> Vec2 {
    match formation_type {
        FormationType::Line => {
            // Horizontal line formation
            let center_offset = (total_units - 1) as f32 * spacing / 2.0;
            Vec2::new(index as f32 * spacing - center_offset, 0.0)
        }
        FormationType::Column => {
            // Vertical column formation
            Vec2::new(0.0, index as f32 * spacing)
        }
        FormationType::Box => {
            // Box/grid formation
            let units_per_row = (total_units as f32).sqrt().ceil() as usize;
            let row = index / units_per_row;
            let col = index % units_per_row;

            let center_offset_x = (units_per_row - 1) as f32 * spacing / 2.0;
            let x = col as f32 * spacing - center_offset_x;
            let y = row as f32 * spacing;

            Vec2::new(x, y)
        }
        FormationType::Wedge => {
            // V-shaped wedge formation
            let row = ((index as f32 * 2.0).sqrt()).floor() as usize;
            let pos_in_row = index - (row * (row + 1)) / 2;

            let x = if row == 0 {
                0.0
            } else {
                (pos_in_row as f32 - row as f32 / 2.0) * spacing
            };
            let y = row as f32 * spacing;

            Vec2::new(x, y)
        }
        FormationType::Circle => {
            // Circular formation
            if total_units == 1 {
                Vec2::ZERO
            } else {
                let angle = (index as f32 / total_units as f32) * 2.0 * std::f32::consts::PI;
                let radius = spacing * 2.0; // Adjust radius based on spacing
                Vec2::new(angle.cos() * radius, angle.sin() * radius)
            }
        }
    }
}

// Calculate formation offset relative to leader position
fn calculate_leader_formation_offset(
    formation_type: &FormationType,
    index: usize,
    spacing: f32,
    _leader_position: Vec3,
) -> Vec2 {
    match formation_type {
        FormationType::Line => {
            // Line behind the leader
            Vec2::new((index as f32 - 2.0) * spacing, -spacing * 2.0)
        }
        FormationType::Circle => {
            // Circle around the leader
            let angle = (index as f32 / 8.0) * 2.0 * std::f32::consts::PI; // Assume max 8 units in circle
            let radius = spacing * 3.0;
            Vec2::new(angle.cos() * radius, angle.sin() * radius)
        }
        _ => {
            // Default to box formation around leader
            let units_per_side = 3; // 3x3 grid around leader
            let row = index / units_per_side;
            let col = index % units_per_side;

            let x = (col as f32 - 1.0) * spacing;
            let y = (row as f32 - 1.0) * spacing;

            Vec2::new(x, y)
        }
    }
}

// Formation maintenance system - keeps units in formation during movement
pub fn formation_maintenance_system(
    time: Res<Time>,
    mut unit_query: Query<(&mut Transform, &mut MovementTarget, &Formation), With<Unit>>,
) {
    let dt = time.delta_seconds();

    for (mut transform, mut target, formation) in unit_query.iter_mut() {
        if !target.reached {
            // Calculate desired position based on formation
            let desired_position = Vec3::new(
                target.x + formation.position_in_formation.x,
                transform.translation.y,
                target.y + formation.position_in_formation.y,
            );

            let direction = desired_position - transform.translation;
            let distance = direction.length();

            if distance > 0.1 {
                // Move towards formation position
                let movement_speed = 5.0; // Base movement speed
                let movement = direction.normalize() * movement_speed * dt;
                transform.translation += movement;

                // Smooth rotation towards movement direction
                if direction.length() > 0.01 {
                    let look_direction = direction.normalize();
                    let target_rotation =
                        Quat::from_rotation_y(look_direction.x.atan2(look_direction.z));
                    transform.rotation = transform.rotation.slerp(target_rotation, 3.0 * dt);
                }
            } else {
                target.reached = true;
            }
        }
    }
}

// Formation visualization system - disabled pending gizmos API updates
// TODO: Re-enable when formation visualization is needed

// Formation spacing adjustment system
pub fn formation_spacing_system(
    input: Res<ButtonInput<KeyCode>>,
    mut formation_query: Query<&mut Formation, With<Selected>>,
) {
    let mut spacing_change = 0.0;

    if input.pressed(KeyCode::Equal) || input.pressed(KeyCode::NumpadAdd) {
        spacing_change = 0.5; // Increase spacing
    } else if input.pressed(KeyCode::Minus) || input.pressed(KeyCode::NumpadSubtract) {
        spacing_change = -0.5; // Decrease spacing
    }

    if spacing_change != 0.0 {
        for mut formation in formation_query.iter_mut() {
            formation.spacing = (formation.spacing + spacing_change).max(1.0); // Minimum spacing of 1.0

            #[cfg(feature = "web")]
            console::log_1(
                &format!("Formation spacing changed to {:.1}", formation.spacing).into(),
            );
        }
    }
}
