use crate::{Selectable, Selected, Unit};
use bevy::prelude::*;
use game_physics::{
    MovementCommand, MovementCommandEvent, MovementController, MovementPath, MovementTarget,
    Velocity,
};
#[cfg(feature = "web")]
use web_sys::console;

// Selection state resource
#[derive(Resource)]
pub struct SelectionState {
    pub selected_entities: Vec<Entity>,
    pub selection_changed: bool,
    pub last_selection_time: f32,
}

impl Default for SelectionState {
    fn default() -> Self {
        Self {
            selected_entities: Vec::new(),
            selection_changed: false,
            last_selection_time: 0.0,
        }
    }
}

// Input handling resource
#[derive(Resource)]
pub struct InputState {
    pub left_mouse_pressed: bool,
    pub right_mouse_pressed: bool,
    pub shift_held: bool,
    pub ctrl_held: bool,
    pub mouse_world_position: Vec3,
}

impl Default for InputState {
    fn default() -> Self {
        Self {
            left_mouse_pressed: false,
            right_mouse_pressed: false,
            shift_held: false,
            ctrl_held: false,
            mouse_world_position: Vec3::ZERO,
        }
    }
}

// Selection box component for drag selection
#[derive(Component, Clone)]
pub struct SelectionBox {
    pub start_position: Vec2,
    pub end_position: Vec2,
    pub is_active: bool,
}

impl Default for SelectionBox {
    fn default() -> Self {
        Self {
            start_position: Vec2::ZERO,
            end_position: Vec2::ZERO,
            is_active: false,
        }
    }
}

// Game command structure for unit orders
#[derive(Clone, Debug)]
pub struct GameCommand {
    pub command_type: String,
    pub entity_id: Option<u32>,
    pub target_x: Option<f32>,
    pub target_y: Option<f32>,
    pub data: Option<String>,
}

// Command queue resource
#[derive(Resource, Default)]
pub struct CommandQueue {
    pub commands: Vec<GameCommand>,
}

// Main selection system
pub fn selection_system(
    mut commands: Commands,
    mut selection_state: ResMut<SelectionState>,
    input_state: Res<InputState>,
    time: Res<Time>,
    selectable_query: Query<(Entity, &Transform, &Selectable), With<Unit>>,
    mut selected_query: Query<Entity, With<Selected>>,
) {
    let current_time = time.elapsed_seconds();

    if input_state.left_mouse_pressed {
        handle_unit_selection(
            &mut commands,
            &mut selection_state,
            &input_state,
            &selectable_query,
            &mut selected_query,
            current_time,
        );
    }
}

// Movement command system that sends physics-based movement commands
pub fn movement_command_system(
    mut commands: Commands,
    input_state: Res<InputState>,
    selection_state: Res<SelectionState>,
    mut command_queue: ResMut<CommandQueue>,
    mut movement_events: MessageWriter<MovementCommandEvent>,
    mut unit_query: Query<(&mut MovementController, &mut MovementPath), With<Unit>>,
) {
    if input_state.right_mouse_pressed && !selection_state.selected_entities.is_empty() {
        let target_pos = input_state.mouse_world_position;
        let num_units = selection_state.selected_entities.len();

        // Calculate formation positions for multiple units
        let formation_spacing = 2.0;
        let units_per_row = (num_units as f32).sqrt().ceil() as usize;

        for (i, entity) in selection_state.selected_entities.iter().enumerate() {
            // Calculate formation offset
            let row = i / units_per_row;
            let col = i % units_per_row;
            let offset_x = (col as f32 - units_per_row as f32 / 2.0) * formation_spacing;
            let offset_z = (row as f32 - units_per_row as f32 / 2.0) * formation_spacing;

            let unit_target_pos = Vec3::new(
                target_pos.x + offset_x,
                target_pos.y,
                target_pos.z + offset_z,
            );

            // Send physics-based movement command
            movement_events.write(MovementCommandEvent {
                entity: *entity,
                command: MovementCommand::MoveTo {
                    position: unit_target_pos,
                    speed: 5.0,
                },
            });

            // Also update the controller directly for immediate response
            if let Ok((mut controller, mut path)) = unit_query.get_mut(*entity) {
                controller.target_position = Some(unit_target_pos);
                controller.max_speed = 5.0;
                controller.waypoints.clear();
                controller.waypoints.push(unit_target_pos);
                controller.path_index = 0;
                controller.is_moving = true;

                path.waypoints.clear();
                path.waypoints.push(unit_target_pos);
                path.current_waypoint_index = 0;
                path.is_moving = true;
            }

            // Also add to command queue for tracking
            let move_command = GameCommand {
                command_type: "move_unit".to_string(),
                entity_id: Some(entity.index()),
                target_x: Some(unit_target_pos.x),
                target_y: Some(unit_target_pos.z),
                data: None,
            };
            command_queue.commands.push(move_command);
        }

        #[cfg(feature = "web")]
        console::log_1(
            &format!(
                "Move command issued for {} units to ({:.2}, {:.2}) in formation",
                selection_state.selected_entities.len(),
                target_pos.x,
                target_pos.z
            )
            .into(),
        );
    }
}

// Physics-based movement system using velocity for smooth unit movement
pub fn enhanced_movement_system(
    time: Res<Time>,
    mut unit_query: Query<
        (
            &mut Velocity,
            &Transform,
            &mut MovementController,
            &mut MovementPath,
        ),
        With<Unit>,
    >,
) {
    for (mut velocity, transform, mut controller, mut movement_path) in unit_query.iter_mut() {
        // Check if we have waypoints to follow
        if !movement_path.waypoints.is_empty() {
            // Get current waypoint
            let current_waypoint_index = movement_path.current_waypoint_index;
            if current_waypoint_index < movement_path.waypoints.len() {
                let waypoint = movement_path.waypoints[current_waypoint_index];
                let distance = transform.translation.distance(waypoint);

                if distance < 0.5 {
                    // Reached waypoint, move to next
                    movement_path.current_waypoint_index += 1;
                    if movement_path.current_waypoint_index >= movement_path.waypoints.len() {
                        // Reached final destination
                        movement_path.waypoints.clear();
                        movement_path.current_waypoint_index = 0;
                        movement_path.is_moving = false;

                        #[cfg(feature = "web")]
                        console::log_1(&"Unit reached final destination".into());
                    }
                } else {
                    // Use physics velocity to move towards waypoint
                    let direction = (waypoint - transform.translation).normalize();

                    // Apply steering forces for smooth movement
                    let desired_velocity = direction * controller.max_speed;
                    let steering = (desired_velocity - velocity.linear) * controller.acceleration;

                    // Update velocity with steering force
                    velocity.linear += steering * time.delta_seconds();

                    // Limit velocity to max speed
                    if velocity.linear.length() > controller.max_speed {
                        velocity.linear = velocity.linear.normalize() * controller.max_speed;
                    }
                }
            }
        } else if let Some(target_pos) = controller.target_position {
            // Direct movement to target using physics
            let direction = target_pos - transform.translation;
            let distance = direction.length();

            if distance < 0.2 {
                controller.target_position = None;
                controller.is_moving = false;
                velocity.linear = Vec3::ZERO;
                #[cfg(feature = "web")]
                console::log_1(&"Unit reached destination".into());
            } else {
                // Apply physics-based movement
                let desired_velocity = direction.normalize() * controller.max_speed;
                let steering = (desired_velocity - velocity.linear) * controller.acceleration;

                velocity.linear += steering * time.delta_seconds();

                // Limit velocity
                if velocity.linear.length() > controller.max_speed {
                    velocity.linear = velocity.linear.normalize() * controller.max_speed;
                }

                controller.is_moving = true;
            }
        } else {
            // No target, gradually stop
            velocity.linear *= 0.9; // Apply friction
            if velocity.linear.length() < 0.1 {
                velocity.linear = Vec3::ZERO;
                controller.is_moving = false;
            }
        }
    }
}

/* Selection visualization system - temporarily disabled due to gizmos API changes
pub fn selection_visualization_system(
    mut gizmos: Gizmos,
    selected_query: Query<&Transform, With<Selected>>,
    selection_state: Res<SelectionState>,
) {
    // Implementation would go here when gizmos API is fixed
}
*/

// Movement visualization system - disabled pending API updates
// TODO: Re-enable when movement visualization is needed

// Group selection system (marquee/box selection)
pub fn group_selection_system(
    mut commands: Commands,
    mut selection_state: ResMut<SelectionState>,
    input_state: Res<InputState>,
    selectable_query: Query<(Entity, &Transform, &Selectable), With<Unit>>,
    mut selected_query: Query<Entity, With<Selected>>,
) {
    // TODO: Implement drag selection logic
    // This would handle creating a selection box and selecting multiple units within it
}

// Helper function for unit selection logic
fn handle_unit_selection(
    commands: &mut Commands,
    selection_state: &mut SelectionState,
    input_state: &InputState,
    selectable_query: &Query<(Entity, &Transform, &Selectable), With<Unit>>,
    selected_query: &mut Query<Entity, With<Selected>>,
    current_time: f32,
) {
    // Find closest selectable unit to mouse position
    let mut closest_unit: Option<Entity> = None;
    let mut closest_distance = f32::MAX;

    for (entity, transform, selectable) in selectable_query.iter() {
        let distance = transform
            .translation
            .distance(input_state.mouse_world_position);
        if distance <= selectable.selection_radius && distance < closest_distance {
            closest_distance = distance;
            closest_unit = Some(entity);
        }
    }

    // Handle selection logic
    if let Some(unit_entity) = closest_unit {
        if !input_state.shift_held {
            // Clear previous selection if not holding shift
            for selected_entity in selected_query.iter_mut() {
                commands.entity(selected_entity).remove::<Selected>();
            }
            selection_state.selected_entities.clear();
        }

        // Add to selection
        commands.entity(unit_entity).insert(Selected);
        if !selection_state.selected_entities.contains(&unit_entity) {
            selection_state.selected_entities.push(unit_entity);
        }

        selection_state.selection_changed = true;
        selection_state.last_selection_time = current_time;

        #[cfg(feature = "web")]
        console::log_1(&format!("Selected unit: {:?}", unit_entity).into());
    } else if !input_state.shift_held {
        // Clear selection if clicking empty space
        for selected_entity in selected_query.iter_mut() {
            commands.entity(selected_entity).remove::<Selected>();
        }
        selection_state.selected_entities.clear();
        selection_state.selection_changed = true;
    }
}
