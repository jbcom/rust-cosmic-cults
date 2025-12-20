use bevy::prelude::*;
use crate::components::*;
use crate::spatial::GlobalSpatialGrid;
// Use our physics Sphere to avoid name conflict
use crate::components::Sphere as PhysicsSphere;

// ==============================================================================
// COLLISION DETECTION SYSTEMS
// ==============================================================================

/// Broad phase collision detection using spatial partitioning
pub fn broad_phase_collision_system(
    mut spatial_grid: ResMut<GlobalSpatialGrid>,
    query: Query<(Entity, &Transform), With<AABB>>,
    mut collision_pairs: ResMut<BroadPhaseCollisionPairs>,
) {
    collision_pairs.clear();
    
    // Update spatial grid with current positions
    spatial_grid.grid.clear();
    for (entity, transform) in query.iter() {
        spatial_grid.grid.insert(entity, transform.translation);
    }
    
    // Find potential collision pairs using spatial grid
    for (entity, transform) in query.iter() {
        let nearby_entities = spatial_grid.grid.query_range(transform.translation, 5.0); // 5 unit search radius
        
        for nearby_entity in nearby_entities {
            if entity != nearby_entity {
                collision_pairs.add_pair(entity, nearby_entity);
            }
        }
    }
}

/// Narrow phase collision detection using AABB vs AABB
pub fn aabb_collision_system(
    collision_pairs: Res<BroadPhaseCollisionPairs>,
    aabb_query: Query<(&Transform, &AABB)>,
    mut collision_events: MessageWriter<CollisionEvent>,
) {
    for &(entity_a, entity_b) in &collision_pairs.pairs {
        if let (Ok((transform_a, aabb_a)), Ok((transform_b, aabb_b))) = 
            (aabb_query.get(entity_a), aabb_query.get(entity_b)) {
            
            if aabb_a.overlaps(transform_a.translation, aabb_b, transform_b.translation) {
                collision_events.write(CollisionEvent {
                    entity_a,
                    entity_b,
                    collision_type: CollisionType::AABB,
                    contact_point: (transform_a.translation + transform_b.translation) * 0.5,
                    normal: (transform_b.translation - transform_a.translation).normalize_or_zero(),
                });
            }
        }
    }
}

/// Sphere vs Sphere collision detection
pub fn sphere_collision_system(
    collision_pairs: Res<BroadPhaseCollisionPairs>,
    sphere_query: Query<(&Transform, &PhysicsSphere)>,
    mut collision_events: MessageWriter<CollisionEvent>,
) {
    for &(entity_a, entity_b) in &collision_pairs.pairs {
        if let (Ok((transform_a, sphere_a)), Ok((transform_b, sphere_b))) = 
            (sphere_query.get(entity_a), sphere_query.get(entity_b)) {
            
            if sphere_a.overlaps(transform_a.translation, sphere_b, transform_b.translation) {
                let center_a = transform_a.translation + sphere_a.center_offset;
                let center_b = transform_b.translation + sphere_b.center_offset;
                let direction = center_b - center_a;
                
                collision_events.write(CollisionEvent {
                    entity_a,
                    entity_b,
                    collision_type: CollisionType::Sphere,
                    contact_point: center_a + direction.normalize() * sphere_a.radius,
                    normal: direction.normalize_or_zero(),
                });
            }
        }
    }
}

/// Sensor trigger detection system
pub fn sensor_system(
    collision_pairs: Res<BroadPhaseCollisionPairs>,
    sensor_query: Query<(&Transform, &PhysicsSphere, &Sensor), With<Sensor>>,
    entity_query: Query<&Transform, Without<Sensor>>,
    mut trigger_events: MessageWriter<TriggerEvent>,
) {
    for &(sensor_entity, other_entity) in &collision_pairs.pairs {
        // Check if one entity is a sensor
        if let (Ok((sensor_transform, sensor_sphere, sensor)), Ok(other_transform)) = 
            (sensor_query.get(sensor_entity), entity_query.get(other_entity)) {
            
            if sensor.is_active && 
               sensor_sphere.overlaps(sensor_transform.translation, &PhysicsSphere::new(0.1), other_transform.translation) {
                
                trigger_events.write(TriggerEvent {
                    sensor_entity,
                    triggered_by: other_entity,
                    entered: true,
                });
            }
        }
        // Check reverse (other entity might be the sensor)
        else if let (Ok((sensor_transform, sensor_sphere, sensor)), Ok(other_transform)) = 
            (sensor_query.get(other_entity), entity_query.get(sensor_entity)) {
            
            if sensor.is_active && 
               sensor_sphere.overlaps(sensor_transform.translation, &PhysicsSphere::new(0.1), other_transform.translation) {
                
                trigger_events.write(TriggerEvent {
                    sensor_entity: other_entity,
                    triggered_by: sensor_entity,
                    entered: true,
                });
            }
        }
    }
}

/// Collision response system for physics-based collision resolution
pub fn collision_response_system(
    mut collision_events: MessageReader<CollisionEvent>,
    mut velocity_query: Query<&mut Velocity>,
    mass_query: Query<&Mass>,
    rigid_body_query: Query<&RigidBodyType>,
) {
    for collision_event in collision_events.read() {
        let entity_a = collision_event.entity_a;
        let entity_b = collision_event.entity_b;
        
        // Get rigid body types
        let body_type_a = rigid_body_query.get(entity_a).map(|rb| &rb.body_type).ok();
        let body_type_b = rigid_body_query.get(entity_b).map(|rb| &rb.body_type).ok();
        
        // Skip collision response for static bodies
        if matches!(body_type_a, Some(RigidBodyVariant::Static)) && 
           matches!(body_type_b, Some(RigidBodyVariant::Static)) {
            continue;
        }
        
        // Skip if both entities are static
        if matches!(body_type_a, Some(RigidBodyVariant::Static)) && 
           matches!(body_type_b, Some(RigidBodyVariant::Static)) {
            continue;
        }
        
        // Handle collision between two dynamic entities
        if entity_a != entity_b {
            if let Ok(velocities) = velocity_query.get_many_mut([entity_a, entity_b]) {
                let [mut vel_a, mut vel_b] = velocities;
            
            let mass_a = mass_query.get(entity_a).map(|m| m.value).unwrap_or(1.0);
            let mass_b = mass_query.get(entity_b).map(|m| m.value).unwrap_or(1.0);
            
            // Simple elastic collision response
            let restitution = 0.8; // Bounciness factor
            let relative_velocity = vel_a.linear - vel_b.linear;
            let velocity_along_normal = relative_velocity.dot(collision_event.normal);
            
            // Don't resolve if objects are separating
            if velocity_along_normal > 0.0 {
                continue;
            }
            
            // Calculate collision impulse
            let impulse_magnitude = -(1.0 + restitution) * velocity_along_normal / (mass_a + mass_b);
            let impulse = collision_event.normal * impulse_magnitude;
            
            // Apply impulse based on rigid body types
            if !matches!(body_type_a, Some(RigidBodyVariant::Static)) {
                vel_a.linear += impulse * mass_a;
            }
            if !matches!(body_type_b, Some(RigidBodyVariant::Static)) {
                vel_b.linear -= impulse * mass_b;
            }
            }
        }
    }
}

/// Raycast system for line-of-sight and projectile collision
pub fn raycast_system(
    mut raycast_events: MessageReader<RaycastEvent>,
    obstacle_query: Query<(&Transform, &AABB), With<Obstacle>>,
    mut raycast_results: MessageWriter<RaycastResultEvent>,
) {
    for raycast_event in raycast_events.read() {
        let mut hit_found = false;
        let mut closest_hit = RaycastHit {
            entity: Entity::PLACEHOLDER,
            distance: f32::INFINITY,
            point: Vec3::ZERO,
            normal: Vec3::ZERO,
        };
        
        // Check intersection with all obstacles
        for (transform, aabb) in obstacle_query.iter() {
            if let Some(hit) = ray_aabb_intersection(
                raycast_event.origin,
                raycast_event.direction,
                transform.translation,
                aabb,
            ) {
                if hit.distance < closest_hit.distance {
                    closest_hit = hit;
                    hit_found = true;
                }
            }
        }
        
        raycast_results.write(RaycastResultEvent {
            ray_id: raycast_event.ray_id,
            hit: if hit_found { Some(closest_hit) } else { None },
        });
    }
}

// ==============================================================================
// COLLISION EVENTS AND TYPES
// ==============================================================================

#[derive(Event)]
pub struct CollisionEvent {
    pub entity_a: Entity,
    pub entity_b: Entity,
    pub collision_type: CollisionType,
    pub contact_point: Vec3,
    pub normal: Vec3,
}

#[derive(Event)]
pub struct TriggerEvent {
    pub sensor_entity: Entity,
    pub triggered_by: Entity,
    pub entered: bool, // true for enter, false for exit
}

#[derive(Event)]
pub struct RaycastEvent {
    pub ray_id: u32,
    pub origin: Vec3,
    pub direction: Vec3,
    pub max_distance: f32,
}

#[derive(Event)]
pub struct RaycastResultEvent {
    pub ray_id: u32,
    pub hit: Option<RaycastHit>,
}

#[derive(Clone, Debug)]
pub enum CollisionType {
    AABB,
    Sphere,
    Ray,
}

#[derive(Clone, Debug)]
pub struct RaycastHit {
    pub entity: Entity,
    pub distance: f32,
    pub point: Vec3,
    pub normal: Vec3,
}

use crate::spatial::BroadPhaseCollisionPairs;

// ==============================================================================
// COLLISION UTILITY FUNCTIONS
// ==============================================================================

/// Ray-AABB intersection test
fn ray_aabb_intersection(
    ray_origin: Vec3,
    ray_direction: Vec3,
    aabb_center: Vec3,
    aabb: &AABB,
) -> Option<RaycastHit> {
    let (aabb_min, aabb_max) = aabb.get_bounds(aabb_center);
    
    let mut t_min: f32 = 0.0;
    let mut t_max: f32 = f32::INFINITY;
    
    for i in 0..3 {
        let origin_component = ray_origin[i];
        let direction_component = ray_direction[i];
        let min_component = aabb_min[i];
        let max_component = aabb_max[i];
        
        if direction_component.abs() < 0.0001 {
            // Ray is parallel to this axis
            if origin_component < min_component || origin_component > max_component {
                return None; // Ray misses the AABB
            }
        } else {
            let t1 = (min_component - origin_component) / direction_component;
            let t2 = (max_component - origin_component) / direction_component;
            
            let (t_near, t_far) = if t1 < t2 { (t1, t2) } else { (t2, t1) };
            
            t_min = t_min.max(t_near);
            t_max = t_max.min(t_far);
            
            if t_min > t_max {
                return None; // No intersection
            }
        }
    }
    
    if t_min >= 0.0 {
        let hit_point = ray_origin + ray_direction * t_min;
        let normal = calculate_aabb_normal(hit_point, aabb_center, aabb);
        
        Some(RaycastHit {
            entity: Entity::PLACEHOLDER, // Would be filled in by the caller
            distance: t_min,
            point: hit_point,
            normal,
        })
    } else {
        None
    }
}

/// Calculate the surface normal of an AABB at a hit point
fn calculate_aabb_normal(hit_point: Vec3, aabb_center: Vec3, _aabb: &AABB) -> Vec3 {
    let local_point = hit_point - aabb_center;
    let abs_local = local_point.abs();
    
    // Find the axis with the largest component (closest to face)
    if abs_local.x >= abs_local.y && abs_local.x >= abs_local.z {
        Vec3::new(local_point.x.signum(), 0.0, 0.0)
    } else if abs_local.y >= abs_local.z {
        Vec3::new(0.0, local_point.y.signum(), 0.0)
    } else {
        Vec3::new(0.0, 0.0, local_point.z.signum())
    }
}
impl bevy::prelude::Message for CollisionEvent {}
impl bevy::prelude::Message for TriggerEvent {}
impl bevy::prelude::Message for RaycastEvent {}
impl bevy::prelude::Message for RaycastResultEvent {}
