use crate::{Team, Unit};
use bevy::prelude::*;
use game_physics::{
    AABB, MovementCommand, MovementCommandEvent, MovementController, Obstacle, Velocity,
};
use game_world::{GameMap, PathfindingGrid, find_path};

// ==============================================================================
// PATHFINDING INTEGRATION
// ==============================================================================

/// System to handle pathfinding requests for units
pub fn pathfinding_request_system(
    mut movement_events: MessageReader<MovementCommandEvent>,
    mut unit_query: Query<(&Transform, &mut MovementController), With<Unit>>,
    game_map: Res<GameMap>,
    pathfinding_grid: Res<PathfindingGrid>,
) {
    for event in movement_events.read() {
        match &event.command {
            MovementCommand::MoveTo { position, speed } => {
                if let Ok((transform, mut controller)) = unit_query.get_mut(event.entity) {
                    // Convert world position to grid coordinates
                    let start_grid = world_to_grid(transform.translation, game_map.tile_size);
                    let goal_grid = world_to_grid(*position, game_map.tile_size);

                    // Find path using A* from game-world
                    if let Some(grid_path) = find_path(start_grid, goal_grid, &pathfinding_grid) {
                        // Convert grid path to world waypoints
                        let waypoints: Vec<Vec3> = grid_path
                            .iter()
                            .map(|&(x, z)| grid_to_world((x, z), game_map.tile_size))
                            .collect();

                        // Update controller with path
                        controller.waypoints = waypoints;
                        controller.path_index = 0;
                        controller.max_speed = *speed;
                        controller.target_position = controller.waypoints.first().copied();
                        controller.is_moving = !controller.waypoints.is_empty();

                        #[cfg(feature = "web")]
                        web_sys::console::log_1(
                            &format!(
                                "Path found with {} waypoints for unit",
                                controller.waypoints.len()
                            )
                            .into(),
                        );
                    } else {
                        // No path found, try direct movement
                        controller.target_position = Some(*position);
                        controller.waypoints.clear();
                        controller.max_speed = *speed;
                        controller.is_moving = true;

                        #[cfg(feature = "web")]
                        web_sys::console::log_1(
                            &"No path found, attempting direct movement".into(),
                        );
                    }
                }
            }

            MovementCommand::SetPath { waypoints, speed } => {
                if let Ok((_transform, mut controller)) = unit_query.get_mut(event.entity) {
                    controller.waypoints = waypoints.clone();
                    controller.path_index = 0;
                    controller.max_speed = *speed;
                    controller.target_position = waypoints.first().copied();
                    controller.is_moving = !waypoints.is_empty();
                }
            }

            _ => {}
        }
    }
}

/// System to update pathfinding grid based on obstacles
pub fn update_pathfinding_obstacles(
    obstacle_query: Query<&Transform, (With<AABB>, With<Obstacle>, Without<Unit>)>,
    mut pathfinding_grid: ResMut<PathfindingGrid>,
    game_map: Res<GameMap>,
) {
    // First, reset all tiles to their default walkability
    for (&grid_pos, tile_info) in &game_map.tiles {
        let walkable = match tile_info.tile_type {
            game_world::map::TileType::Ground => true,
            game_world::map::TileType::Bridge => true,
            game_world::map::TileType::Water => false,
            game_world::map::TileType::Cliff => false,
            game_world::map::TileType::Void => tile_info.corruption_level < 0.9,
        };
        pathfinding_grid.walkable.insert(grid_pos, walkable);
    }

    // Mark tiles with obstacles as non-walkable
    for obstacle_transform in obstacle_query.iter() {
        let grid_pos = world_to_grid(obstacle_transform.translation, game_map.tile_size);
        pathfinding_grid.walkable.insert(grid_pos, false);

        // Also mark adjacent tiles to give obstacles some clearance
        for dx in -1..=1 {
            for dz in -1..=1 {
                let adjacent = (grid_pos.0 + dx, grid_pos.1 + dz);
                pathfinding_grid.walkable.insert(adjacent, false);
            }
        }
    }
}

/// Dynamic pathfinding that recalculates when obstacles are detected
pub fn dynamic_pathfinding_system(
    mut unit_query: Query<(Entity, &Transform, &mut MovementController, &Velocity), With<Unit>>,
    obstacle_query: Query<&Transform, (With<AABB>, With<Obstacle>, Without<Unit>)>,
    pathfinding_grid: Res<PathfindingGrid>,
    game_map: Res<GameMap>,
    mut movement_events: MessageWriter<MovementCommandEvent>,
) {
    for (entity, transform, mut controller, velocity) in unit_query.iter_mut() {
        if !controller.is_moving || controller.target_position.is_none() {
            continue;
        }

        // Check if path is blocked by checking next waypoint
        if !controller.waypoints.is_empty() && controller.path_index < controller.waypoints.len() {
            let next_waypoint = controller.waypoints[controller.path_index];

            // Check for obstacles in the path
            let mut path_blocked = false;
            for obstacle_transform in obstacle_query.iter() {
                let distance = obstacle_transform.translation.distance(next_waypoint);
                if distance < 2.0 {
                    // Obstacle too close to waypoint
                    path_blocked = true;
                    break;
                }
            }

            // If path is blocked and velocity is very low (stuck), recalculate
            if path_blocked && velocity.linear.length() < 0.5 {
                if let Some(final_target) = controller.waypoints.last() {
                    // Request new path calculation
                    movement_events.write(MovementCommandEvent {
                        entity,
                        command: MovementCommand::MoveTo {
                            position: *final_target,
                            speed: controller.max_speed,
                        },
                    });

                    #[cfg(feature = "web")]
                    web_sys::console::log_1(&"Path blocked, recalculating route".into());
                }
            }
        }
    }
}

/// Formation pathfinding for group movement
pub fn formation_pathfinding_system(
    selected_units: Query<(Entity, &Transform), (With<Unit>, With<crate::Selected>)>,
    pathfinding_grid: Res<PathfindingGrid>,
    game_map: Res<GameMap>,
    mut movement_events: MessageWriter<MovementCommandEvent>,
) {
    // This system would handle formation-based pathfinding
    // where units maintain formation while navigating
    // Implementation would calculate a single path for the group center
    // then offset individual unit positions to maintain formation
}

// ==============================================================================
// HELPER FUNCTIONS
// ==============================================================================

/// Convert world position to grid coordinates
pub fn world_to_grid(world_pos: Vec3, tile_size: f32) -> (i32, i32) {
    (
        (world_pos.x / tile_size).round() as i32,
        (world_pos.z / tile_size).round() as i32,
    )
}

/// Convert grid coordinates to world position
pub fn grid_to_world(grid_pos: (i32, i32), tile_size: f32) -> Vec3 {
    Vec3::new(
        grid_pos.0 as f32 * tile_size,
        0.0,
        grid_pos.1 as f32 * tile_size,
    )
}

/// Smooth path by removing unnecessary waypoints
pub fn smooth_path(
    waypoints: Vec<Vec3>,
    pathfinding_grid: &PathfindingGrid,
    tile_size: f32,
) -> Vec<Vec3> {
    if waypoints.len() <= 2 {
        return waypoints;
    }

    let mut smoothed = vec![waypoints[0]];
    let mut current_index = 0;

    while current_index < waypoints.len() - 1 {
        let mut farthest_visible = current_index + 1;

        // Find the farthest waypoint we can reach directly
        for i in (current_index + 2)..waypoints.len() {
            if is_path_clear(
                waypoints[current_index],
                waypoints[i],
                pathfinding_grid,
                tile_size,
            ) {
                farthest_visible = i;
            } else {
                break;
            }
        }

        smoothed.push(waypoints[farthest_visible]);
        current_index = farthest_visible;
    }

    smoothed
}

/// Check if a straight path between two points is clear
fn is_path_clear(
    start: Vec3,
    end: Vec3,
    pathfinding_grid: &PathfindingGrid,
    tile_size: f32,
) -> bool {
    let distance = start.distance(end);
    let steps = (distance / tile_size).ceil() as usize;

    for i in 0..=steps {
        let t = i as f32 / steps as f32;
        let pos = start.lerp(end, t);
        let grid_pos = world_to_grid(pos, tile_size);

        if !pathfinding_grid
            .walkable
            .get(&grid_pos)
            .copied()
            .unwrap_or(false)
        {
            return false;
        }
    }

    true
}

// ==============================================================================
// PLUGIN
// ==============================================================================

/// Plugin to integrate pathfinding with physics
pub struct PathfindingIntegrationPlugin;

impl Plugin for PathfindingIntegrationPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                pathfinding_request_system,
                update_pathfinding_obstacles,
                dynamic_pathfinding_system,
                formation_pathfinding_system,
            )
                .chain(),
        );
    }
}
