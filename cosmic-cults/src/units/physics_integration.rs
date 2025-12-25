use crate::units::{Team, Unit};
use bevy::prelude::*;
use game_physics::{
    AABB, CollisionEvent, CollisionType, Mass, MovementCommand, MovementCommandEvent,
    MovementController, RaycastEvent, RaycastHit, RaycastResultEvent, SpatialData, TriggerEvent,
    Velocity,
};

// ==============================================================================
// COLLISION HANDLING SYSTEMS
// ==============================================================================

/// Handle unit-to-unit collisions with proper separation
pub fn unit_collision_handler(
    mut collision_events: MessageReader<CollisionEvent>,
    mut unit_query: Query<(&mut Velocity, &Transform, &Team), With<Unit>>,
) {
    for collision_event in collision_events.read() {
        // Check if both entities are units
        if let Ok([(mut vel_a, trans_a, team_a), (mut vel_b, trans_b, team_b)]) =
            unit_query.get_many_mut([collision_event.entity_a, collision_event.entity_b])
        {
            // Calculate separation force
            let separation_direction = (trans_a.translation - trans_b.translation).normalize();
            let separation_force = separation_direction * 5.0; // Separation strength

            // Apply separation only if units are on the same team (allies shouldn't overlap)
            if team_a.id == team_b.id {
                vel_a.linear += separation_force;
                vel_b.linear -= separation_force;

                // Limit velocity to prevent excessive separation
                vel_a.linear = vel_a.linear.clamp_length_max(10.0);
                vel_b.linear = vel_b.linear.clamp_length_max(10.0);
            }
        }
    }
}

/// Handle units colliding with obstacles
pub fn obstacle_collision_handler(
    mut collision_events: MessageReader<CollisionEvent>,
    mut unit_query: Query<(&mut Velocity, &mut MovementController), With<Unit>>,
    obstacle_query: Query<&Transform, (With<AABB>, Without<Unit>)>,
) {
    for collision_event in collision_events.read() {
        // Check if one entity is a unit and the other is an obstacle
        if let Ok((mut velocity, mut controller)) = unit_query.get_mut(collision_event.entity_a)
            && obstacle_query.get(collision_event.entity_b).is_ok()
        {
            // Stop unit movement and recalculate path
            velocity.linear *= 0.5; // Slow down
            controller.is_moving = false;

            // Mark that unit needs to recalculate path
            if let Some(target) = controller.target_position {
                // Will trigger pathfinding recalculation
                controller.waypoints.clear();
                controller.path_index = 0;
            }
        }

        // Check reverse case
        if let Ok((mut velocity, mut controller)) = unit_query.get_mut(collision_event.entity_b)
            && obstacle_query.get(collision_event.entity_a).is_ok()
        {
            velocity.linear *= 0.5;
            controller.is_moving = false;

            if let Some(target) = controller.target_position {
                controller.waypoints.clear();
                controller.path_index = 0;
            }
        }
    }
}

// ==============================================================================
// SPATIAL QUERY SYSTEMS
// ==============================================================================

/// Find nearby units using spatial indexing
pub fn find_nearby_units(
    position: Vec3,
    radius: f32,
    spatial_grid: &game_physics::GlobalSpatialGrid,
) -> Vec<Entity> {
    spatial_grid.grid.query_range(position, radius)
}

/// System to update unit spatial data
#[allow(clippy::type_complexity)]
pub fn update_unit_spatial_data(
    mut query: Query<(&Transform, &mut SpatialData), (With<Unit>, Changed<Transform>)>,
) {
    for (transform, mut spatial_data) in query.iter_mut() {
        spatial_data.update_position(transform.translation, 10.0); // 10.0 is grid cell size
    }
}

// ==============================================================================
// PHYSICS-BASED MOVEMENT HELPERS
// ==============================================================================

/// Apply steering behaviors for smooth unit movement
pub fn apply_steering_behavior(
    current_velocity: Vec3,
    target_position: Vec3,
    current_position: Vec3,
    max_speed: f32,
    max_force: f32,
) -> Vec3 {
    // Calculate desired velocity
    let to_target = target_position - current_position;
    let distance = to_target.length();

    if distance < 0.1 {
        return Vec3::ZERO;
    }

    // Arrival behavior - slow down when approaching target
    let desired_speed = if distance < 5.0 {
        max_speed * (distance / 5.0)
    } else {
        max_speed
    };

    let desired_velocity = to_target.normalize() * desired_speed;

    // Calculate steering force
    let steering = desired_velocity - current_velocity;
    steering.clamp_length_max(max_force)
}

/// System for smooth physics-based unit movement with steering
pub fn physics_steering_movement_system(
    time: Res<Time>,
    mut query: Query<(&mut Velocity, &Transform, &mut MovementController, &Mass), With<Unit>>,
) {
    let dt = time.delta_secs();

    for (mut velocity, transform, mut controller, mass) in query.iter_mut() {
        if !controller.is_moving {
            // Apply friction when not moving
            velocity.linear *= 0.9;
            continue;
        }

        // Get current target (either direct target or current waypoint)
        let target = if !controller.waypoints.is_empty()
            && controller.path_index < controller.waypoints.len()
        {
            controller.waypoints[controller.path_index]
        } else if let Some(target_pos) = controller.target_position {
            target_pos
        } else {
            continue;
        };

        // Apply steering behavior
        let steering_force = apply_steering_behavior(
            velocity.linear,
            target,
            transform.translation,
            controller.max_speed,
            controller.acceleration * mass.value,
        );

        // Apply force to velocity (F = ma, so a = F/m)
        velocity.linear += steering_force * dt / mass.value;

        // Limit velocity to max speed
        velocity.linear = velocity.linear.clamp_length_max(controller.max_speed);

        // Check if reached current waypoint
        let distance_to_target = transform.translation.distance(target);
        if distance_to_target < 1.0 {
            if !controller.waypoints.is_empty() {
                controller.path_index += 1;
                if controller.path_index >= controller.waypoints.len() {
                    // Reached end of path
                    controller.waypoints.clear();
                    controller.path_index = 0;
                    controller.target_position = None;
                    controller.is_moving = false;
                    velocity.linear = Vec3::ZERO;
                }
            } else {
                // Reached direct target
                controller.target_position = None;
                controller.is_moving = false;
                velocity.linear = Vec3::ZERO;
            }
        }
    }
}

// ==============================================================================
// PROJECTILE COLLISION SYSTEM
// ==============================================================================

/// Handle projectile collisions with units
pub fn projectile_collision_system(
    mut collision_events: MessageReader<CollisionEvent>,
    projectile_query: Query<&ProjectileMarker>,
    mut unit_query: Query<&mut Unit>,
    mut commands: Commands,
) {
    for collision_event in collision_events.read() {
        // Check if entity_a is a projectile
        if let Ok(projectile) = projectile_query.get(collision_event.entity_a)
            && let Ok(mut unit) = unit_query.get_mut(collision_event.entity_b)
        {
            // Apply damage
            unit.health -= projectile.damage;

            // Despawn projectile
            commands.entity(collision_event.entity_a).despawn();

            #[cfg(feature = "web")]
            web_sys::console::log_1(
                &format!("Projectile hit unit for {} damage", projectile.damage).into(),
            );
        }

        // Check reverse case
        if let Ok(projectile) = projectile_query.get(collision_event.entity_b)
            && let Ok(mut unit) = unit_query.get_mut(collision_event.entity_a)
        {
            unit.health -= projectile.damage;
            commands.entity(collision_event.entity_b).despawn();

            #[cfg(feature = "web")]
            web_sys::console::log_1(
                &format!("Projectile hit unit for {} damage", projectile.damage).into(),
            );
        }
    }
}

/// Marker component for projectiles
#[derive(Component)]
pub struct ProjectileMarker {
    pub damage: f32,
    pub owner: Entity,
    pub team: u32,
}

// ==============================================================================
// PLUGIN
// ==============================================================================

/// Plugin to integrate physics with units
pub struct UnitsPhysicsIntegrationPlugin;

impl Plugin for UnitsPhysicsIntegrationPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                unit_collision_handler,
                obstacle_collision_handler,
                physics_steering_movement_system,
                update_unit_spatial_data,
                projectile_collision_system,
            ),
        );
    }
}
