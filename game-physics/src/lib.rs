//! Game Physics Crate
//!
//! Foundational physics layer providing collision detection, spatial indexing,
//! and movement systems for RTS games. This crate forms the base layer that
//! other game crates (combat, units, AI) depend on.

use avian3d::prelude as avian;
use bevy::prelude::*;
// Avoid sphere name conflict with Bevy's math Sphere
use crate::components::Sphere as PhysicsSphere;

pub mod collision;
pub mod components;
pub mod movement;
pub mod spatial;

// Re-export commonly used types
pub use collision::{
    CollisionEvent, CollisionType, RaycastEvent, RaycastHit, RaycastResultEvent, TriggerEvent,
};
pub use components::*;
pub use movement::{FlockingAgent, Formation, FormationMember, FormationType};
pub use spatial::{BroadPhaseCollisionPairs, GlobalSpatialGrid, SpatialGrid};

// ==============================================================================
// PHYSICS PLUGIN
// ==============================================================================

/// Main plugin for the game physics system
pub struct GamePhysicsPlugin {
    /// Cell size for spatial grid optimization
    pub spatial_grid_cell_size: f32,
    /// Enable collision detection systems
    pub enable_collision_detection: bool,
    /// Enable movement systems
    pub enable_movement_systems: bool,
    /// Enable advanced pathfinding
    pub enable_pathfinding: bool,
}

impl Default for GamePhysicsPlugin {
    fn default() -> Self {
        Self {
            spatial_grid_cell_size: 10.0,
            enable_collision_detection: true,
            enable_movement_systems: true,
            enable_pathfinding: false,
        }
    }
}

impl Plugin for GamePhysicsPlugin {
    fn build(&self, app: &mut App) {
        // Add resources
        app.insert_resource(GlobalSpatialGrid::new(self.spatial_grid_cell_size))
            .insert_resource(BroadPhaseCollisionPairs::default());

        // Add collision events
        if self.enable_collision_detection {
            app.add_message::<CollisionEvent>()
                .add_message::<TriggerEvent>()
                .add_message::<RaycastEvent>()
                .add_message::<RaycastResultEvent>();
        }

        // Add movement events
        if self.enable_movement_systems {
            app.add_message::<MovementCommandEvent>();
        }

        // Add core physics systems
        app.add_plugins(avian::PhysicsPlugins::default());

        if self.enable_movement_systems {
            app.add_systems(
                Update,
                (
                    // movement::physics_movement_system, // Replaced by Avian
                    sync_velocity_system,
                    movement::simple_movement_system,
                    movement::pathfinding_movement_system,
                    movement::waypoint_movement_system,
                ),
            );
        }

        if self.enable_collision_detection {
            app.add_systems(
                Update,
                (
                    collision::broad_phase_collision_system,
                    collision::aabb_collision_system,
                    collision::sphere_collision_system,
                    collision::sensor_system,
                    collision::collision_response_system,
                    collision::raycast_system,
                ),
            );
        }

        if self.enable_pathfinding {
            app.add_systems(
                Update,
                (
                    movement::formation_movement_system,
                    movement::flocking_system,
                    movement::obstacle_avoidance_system,
                ),
            );
        }

        // Add spatial indexing update system
        app.add_systems(
            PostUpdate,
            (spatial_indexing_update_system, movement_command_system),
        );
    }
}

// ==============================================================================
// MOVEMENT EVENTS
// ==============================================================================

#[derive(Event)]
pub struct MovementCommandEvent {
    pub entity: Entity,
    pub command: MovementCommand,
}

#[derive(Clone, Debug)]
pub enum MovementCommand {
    MoveTo { position: Vec3, speed: f32 },
    Follow { target: Entity, distance: f32 },
    SetPath { waypoints: Vec<Vec3>, speed: f32 },
    Stop,
}

// ==============================================================================
// AVIAN INTEGRATION
// ==============================================================================

fn sync_velocity_system(
    mut query: Query<
        (
            &Velocity,
            &mut avian::LinearVelocity,
            &mut avian::AngularVelocity,
        ),
        Changed<Velocity>,
    >,
) {
    for (vel, mut lin, mut ang) in query.iter_mut() {
        lin.0 = vel.linear;
        ang.0 = vel.angular;
    }
}

// ==============================================================================
// SPATIAL INDEXING SYSTEMS
// ==============================================================================

/// Update spatial indices for all entities
pub fn spatial_indexing_update_system(
    mut spatial_grid: ResMut<GlobalSpatialGrid>,
    mut query: Query<(Entity, &Transform, &mut SpatialData), Changed<Transform>>,
    time: Res<Time>,
) {
    let _current_time = time.elapsed_secs();

    for (entity, transform, mut spatial_data) in query.iter_mut() {
        // Update spatial data
        spatial_data.update_position(transform.translation, spatial_grid.grid.cell_size);

        // Update spatial grid if position changed significantly
        if spatial_data.has_moved {
            spatial_grid.grid.insert(entity, transform.translation);
            spatial_grid.needs_update = true;
        }
    }
}

/// System to handle movement commands
pub fn movement_command_system(
    mut movement_events: MessageReader<MovementCommandEvent>,
    mut movement_query: Query<&mut MovementController>,
    mut target_query: Query<&mut MovementTarget>,
    mut path_query: Query<&mut MovementPath>,
) {
    for event in movement_events.read() {
        match &event.command {
            MovementCommand::MoveTo { position, speed } => {
                // Try MovementController first
                if let Ok(mut controller) = movement_query.get_mut(event.entity) {
                    controller.target_position = Some(*position);
                    controller.max_speed = *speed;
                    controller.waypoints.clear();
                    controller.path_index = 0;
                    controller.is_moving = true;
                }
                // Fallback to MovementTarget
                else if let Ok(mut target) = target_query.get_mut(event.entity) {
                    target.x = position.x;
                    target.y = position.y;
                    target.z = position.z;
                    target.speed = *speed;
                    target.reached = false;
                }
            }

            MovementCommand::SetPath { waypoints, speed } => {
                // Try MovementController first
                if let Ok(mut controller) = movement_query.get_mut(event.entity) {
                    controller.waypoints = waypoints.clone();
                    controller.path_index = 0;
                    controller.max_speed = *speed;
                    controller.target_position = waypoints.first().copied();
                    controller.is_moving = !waypoints.is_empty();
                }
                // Fallback to MovementPath
                else if let Ok(mut path) = path_query.get_mut(event.entity) {
                    path.waypoints = waypoints.clone();
                    path.current_waypoint_index = 0;
                    path.movement_speed = *speed;
                    path.is_moving = !waypoints.is_empty();
                }
            }

            MovementCommand::Stop => {
                // Stop all movement components
                if let Ok(mut controller) = movement_query.get_mut(event.entity) {
                    controller.target_position = None;
                    controller.waypoints.clear();
                    controller.velocity = Vec3::ZERO;
                    controller.is_moving = false;
                }
                if let Ok(mut target) = target_query.get_mut(event.entity) {
                    target.reached = true;
                }
                if let Ok(mut path) = path_query.get_mut(event.entity) {
                    path.waypoints.clear();
                    path.is_moving = false;
                }
            }

            MovementCommand::Follow {
                target: _,
                distance: _,
            } => {
                // Implementation would require target tracking system
                // For now, just clear current movement
                if let Ok(mut controller) = movement_query.get_mut(event.entity) {
                    controller.target_position = None;
                    controller.waypoints.clear();
                    // Would set follow target here
                }
            }
        }
    }
}

// ==============================================================================
// UTILITY FUNCTIONS
// ==============================================================================

/// Create a physics entity bundle with common components
pub fn create_physics_entity(
    commands: &mut Commands,
    position: Vec3,
    velocity: Vec3,
    mass: f32,
) -> Entity {
    commands
        .spawn((
            Transform::from_translation(position),
            GlobalTransform::default(),
            SpatialData::new(position),
            SpatialIndex::new(position, 10.0),
            Velocity::new(velocity),
            Acceleration::default(),
            Mass::new(mass),
            Friction::default(),
            // Avian components
            avian::RigidBody::Dynamic,
            avian::LinearVelocity(velocity),
            avian::AngularVelocity::ZERO,
        ))
        .id()
}

/// Create a collider entity with AABB collision
pub fn create_aabb_collider(
    commands: &mut Commands,
    position: Vec3,
    size: Vec3,
    is_sensor: bool,
) -> Entity {
    let mut entity_commands = commands.spawn((
        Transform::from_translation(position),
        GlobalTransform::default(),
        SpatialData::new(position),
        AABB::from_size(size),
        CollisionMask::default(),
        RigidBodyType::default(),
        // Avian
        avian::Collider::cuboid(size.x, size.y, size.z),
    ));

    if is_sensor {
        entity_commands.insert((Sensor { is_active: true }, avian::Sensor));
    }

    entity_commands.id()
}

/// Create a sphere collider entity
pub fn create_sphere_collider(
    commands: &mut Commands,
    position: Vec3,
    radius: f32,
    is_sensor: bool,
) -> Entity {
    let mut entity_commands = commands.spawn((
        Transform::from_translation(position),
        GlobalTransform::default(),
        SpatialData::new(position),
        PhysicsSphere::new(radius),
        CollisionMask::default(),
        RigidBodyType::default(),
        // Avian
        avian::Collider::sphere(radius),
    ));

    if is_sensor {
        entity_commands.insert((Sensor { is_active: true }, avian::Sensor));
    }

    entity_commands.id()
}

// ==============================================================================
// PRELUDE MODULE
// ==============================================================================

/// Commonly used imports
pub mod prelude {
    pub use crate::{
        AABB,
        Acceleration,
        BroadPhaseCollisionPairs,
        CollisionEvent,
        CollisionMask,
        Friction,
        GamePhysicsPlugin,
        // Resources
        GlobalSpatialGrid,
        // Components
        GridPosition,
        Mass,
        MovementCommand,
        // Events
        MovementCommandEvent,
        MovementController,
        MovementPath,
        MovementTarget,
        MovementType,
        RaycastEvent,
        RaycastResultEvent,
        RigidBodyType,
        RigidBodyVariant,
        Sensor,
        SpatialData,
        SpatialIndex,
        TriggerEvent,
        Velocity,
        create_aabb_collider,
        // Utilities
        create_physics_entity,
        create_sphere_collider,
        movement_command_system,
        // Systems
        spatial_indexing_update_system,
    };
}
impl bevy::prelude::Message for MovementCommandEvent {}
