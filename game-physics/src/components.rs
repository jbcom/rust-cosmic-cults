use bevy::prelude::*;
use serde::{Serialize, Deserialize};

// ==============================================================================
// CORE PHYSICS COMPONENTS
// ==============================================================================

// Position component removed - use Bevy's Transform instead

/// Grid-based position for spatial optimization
#[derive(Component, Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct GridPosition {
    pub x: i32,
    pub y: i32,
}

impl GridPosition {
    pub fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }
    
    pub fn from_world_pos(world_pos: Vec3, grid_size: f32) -> Self {
        Self {
            x: (world_pos.x / grid_size).floor() as i32,
            y: (world_pos.z / grid_size).floor() as i32,
        }
    }
    
    pub fn to_world_pos(&self, grid_size: f32) -> Vec3 {
        Vec3::new(
            self.x as f32 * grid_size + grid_size * 0.5,
            0.0,
            self.y as f32 * grid_size + grid_size * 0.5,
        )
    }
    
    pub fn distance(&self, other: &GridPosition) -> f32 {
        let dx = (self.x - other.x) as f32;
        let dy = (self.y - other.y) as f32;
        (dx * dx + dy * dy).sqrt()
    }
}

/// Movement controller with velocity and acceleration
#[derive(Component, Clone, Debug)]
pub struct MovementController {
    pub target_position: Option<Vec3>,
    pub velocity: Vec3,
    pub max_speed: f32,
    pub acceleration: f32,
    pub rotation_speed: f32,
    pub path_index: usize,
    pub waypoints: Vec<Vec3>,
    pub is_moving: bool,
    pub movement_type: MovementType,
}

impl Default for MovementController {
    fn default() -> Self {
        Self {
            target_position: None,
            velocity: Vec3::ZERO,
            max_speed: 5.0,
            acceleration: 10.0,
            rotation_speed: 5.0,
            path_index: 0,
            waypoints: Vec::new(),
            is_moving: false,
            movement_type: MovementType::Ground,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub enum MovementType {
    Ground,
    Flying,
    Phasing,
    Teleporting,
}

/// Simple movement target for basic movement
#[derive(Component, Clone, Debug)]
pub struct MovementTarget {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub reached: bool,
    pub speed: f32,
}

impl MovementTarget {
    pub fn new(x: f32, y: f32, z: f32, speed: f32) -> Self {
        Self { x, y, z, reached: false, speed }
    }
}

/// Waypoint-based movement path
#[derive(Component, Clone, Debug)]
pub struct MovementPath {
    pub waypoints: Vec<Vec3>,
    pub current_waypoint_index: usize,
    pub movement_speed: f32,
    pub is_moving: bool,
}

impl Default for MovementPath {
    fn default() -> Self {
        Self {
            waypoints: Vec::new(),
            current_waypoint_index: 0,
            movement_speed: 5.0,
            is_moving: false,
        }
    }
}

/// Spatial data for position tracking and grid cell management
#[derive(Component, Clone, Debug)]
pub struct SpatialData {
    pub position: Vec3,
    pub grid_cell: (i32, i32),
    pub last_position: Vec3,
    pub has_moved: bool,
}

impl SpatialData {
    pub fn new(position: Vec3) -> Self {
        let grid_cell = ((position.x / 10.0) as i32, (position.z / 10.0) as i32);
        Self {
            position,
            grid_cell,
            last_position: position,
            has_moved: false,
        }
    }
    
    pub fn update_position(&mut self, new_position: Vec3, grid_size: f32) {
        self.last_position = self.position;
        self.position = new_position;
        
        let new_grid_cell = ((new_position.x / grid_size) as i32, (new_position.z / grid_size) as i32);
        self.has_moved = new_grid_cell != self.grid_cell || self.position != self.last_position;
        self.grid_cell = new_grid_cell;
    }
}

/// Spatial index for grid-based optimization
#[derive(Component, Clone, Debug)]
pub struct SpatialIndex {
    pub grid_x: i32,
    pub grid_z: i32,
    pub last_update: f32,
}

impl SpatialIndex {
    pub fn new(position: Vec3, grid_size: f32) -> Self {
        Self {
            grid_x: (position.x / grid_size) as i32,
            grid_z: (position.z / grid_size) as i32,
            last_update: 0.0,
        }
    }
}

/// Collision mask for layer-based collision filtering
#[derive(Component, Clone, Debug)]
pub struct CollisionMask {
    pub layer: u32,
    pub mask: u32,
}

impl Default for CollisionMask {
    fn default() -> Self {
        Self {
            layer: 1,
            mask: u32::MAX,
        }
    }
}

/// Velocity component for physics-based movement
#[derive(Component, Clone, Debug, Default)]
pub struct Velocity {
    pub linear: Vec3,
    pub angular: Vec3,
}

impl Velocity {
    pub fn new(linear: Vec3) -> Self {
        Self {
            linear,
            angular: Vec3::ZERO,
        }
    }
    
    pub fn with_angular(linear: Vec3, angular: Vec3) -> Self {
        Self { linear, angular }
    }
}

/// Acceleration component for physics forces
#[derive(Component, Clone, Debug, Default)]
pub struct Acceleration {
    pub linear: Vec3,
    pub angular: Vec3,
}

impl Acceleration {
    pub fn new(linear: Vec3) -> Self {
        Self {
            linear,
            angular: Vec3::ZERO,
        }
    }
}

/// Friction component for damping
#[derive(Component, Clone, Debug)]
pub struct Friction {
    pub linear_damping: f32,
    pub angular_damping: f32,
}

impl Default for Friction {
    fn default() -> Self {
        Self {
            linear_damping: 0.1,
            angular_damping: 0.1,
        }
    }
}

/// Mass component for physics calculations
#[derive(Component, Clone, Debug)]
pub struct Mass {
    pub value: f32,
    pub inverse: f32,
}

impl Mass {
    pub fn new(mass: f32) -> Self {
        Self {
            value: mass,
            inverse: if mass > 0.0 { 1.0 / mass } else { 0.0 },
        }
    }
}

impl Default for Mass {
    fn default() -> Self {
        Self::new(1.0)
    }
}

/// AABB collision shape for simple collision detection
#[derive(Component, Clone, Debug)]
pub struct AABB {
    pub half_extents: Vec3,
    pub center_offset: Vec3,
}

impl AABB {
    pub fn new(half_extents: Vec3) -> Self {
        Self {
            half_extents,
            center_offset: Vec3::ZERO,
        }
    }
    
    pub fn from_size(size: Vec3) -> Self {
        Self::new(size * 0.5)
    }
    
    pub fn get_bounds(&self, center: Vec3) -> (Vec3, Vec3) {
        let center = center + self.center_offset;
        (center - self.half_extents, center + self.half_extents)
    }
    
    pub fn overlaps(&self, center_a: Vec3, other: &AABB, center_b: Vec3) -> bool {
        let (min_a, max_a) = self.get_bounds(center_a);
        let (min_b, max_b) = other.get_bounds(center_b);
        
        min_a.x <= max_b.x && max_a.x >= min_b.x &&
        min_a.y <= max_b.y && max_a.y >= min_b.y &&
        min_a.z <= max_b.z && max_a.z >= min_b.z
    }
}

/// Sphere collision shape for simple collision detection
#[derive(Component, Clone, Debug)]
pub struct Sphere {
    pub radius: f32,
    pub center_offset: Vec3,
}

impl Sphere {
    pub fn new(radius: f32) -> Self {
        Self {
            radius,
            center_offset: Vec3::ZERO,
        }
    }
    
    pub fn overlaps(&self, center_a: Vec3, other: &Sphere, center_b: Vec3) -> bool {
        let center_a = center_a + self.center_offset;
        let center_b = center_b + other.center_offset;
        let distance_sq = center_a.distance_squared(center_b);
        let radius_sum = self.radius + other.radius;
        distance_sq <= radius_sum * radius_sum
    }
}

/// Sensor component for trigger-based collision detection
#[derive(Component, Clone, Debug, Default)]
pub struct Sensor {
    pub is_active: bool,
}

/// Rigid body type for physics simulation
#[derive(Component, Clone, Debug)]
pub struct RigidBodyType {
    pub body_type: RigidBodyVariant,
}

#[derive(Clone, Debug, PartialEq)]
pub enum RigidBodyVariant {
    Dynamic,
    Kinematic,
    Static,
}

impl Default for RigidBodyType {
    fn default() -> Self {
        Self {
            body_type: RigidBodyVariant::Dynamic,
        }
    }
}

/// Marker component for obstacles that block movement
#[derive(Component, Clone, Debug, Default)]
pub struct Obstacle;