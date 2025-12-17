use bevy::prelude::*;
use crate::components::*;

// ==============================================================================
// MOVEMENT SYSTEMS
// ==============================================================================

/// Physics-based movement system using velocity and acceleration
pub fn physics_movement_system(
    time: Res<Time>,
    mut query: Query<(
        &mut Transform,
        &mut Velocity,
        &Acceleration,
        Option<&Mass>,
        Option<&Friction>,
    )>,
) {
    let dt = time.delta_secs();
    
    for (mut transform, mut velocity, acceleration, mass, friction) in query.iter_mut() {
        let mass_value = mass.map(|m| m.value).unwrap_or(1.0);
        let inv_mass = if mass_value > 0.0 { 1.0 / mass_value } else { 0.0 };
        
        // Apply acceleration (F = ma, so a = F/m)
        velocity.linear += acceleration.linear * inv_mass * dt;
        velocity.angular += acceleration.angular * inv_mass * dt;
        
        // Apply friction/damping
        if let Some(friction) = friction {
            velocity.linear *= 1.0 - (friction.linear_damping * dt).min(1.0);
            velocity.angular *= 1.0 - (friction.angular_damping * dt).min(1.0);
        }
        
        // Update position
        transform.translation += velocity.linear * dt;
        
        // Update rotation from angular velocity
        if velocity.angular.length_squared() > 0.0001 {
            let axis_angle = velocity.angular * dt;
            let angle = axis_angle.length();
            if angle > 0.0001 {
                let axis = axis_angle / angle;
                let rotation_delta = Quat::from_axis_angle(axis, angle);
                transform.rotation = transform.rotation * rotation_delta;
            }
        }
    }
}

/// Simple movement system using MovementTarget
pub fn simple_movement_system(
    time: Res<Time>,
    mut query: Query<(&mut Transform, &mut MovementTarget)>,
) {
    for (mut transform, mut target) in query.iter_mut() {
        if target.reached {
            continue;
        }
        
        let target_position = Vec3::new(target.x, transform.translation.y, target.z);
        let direction = target_position - transform.translation;
        let distance = direction.length();
        
        if distance < 0.1 {
            target.reached = true;
            transform.translation = target_position;
        } else {
            let movement = direction.normalize() * target.speed * time.delta_secs();
            transform.translation += movement;
            
            // Rotate to face movement direction
            if direction.length() > 0.01 {
                let look_direction = direction.normalize();
                let target_rotation = Quat::from_rotation_y(look_direction.x.atan2(look_direction.z));
                transform.rotation = transform.rotation.slerp(target_rotation, 5.0 * time.delta_secs());
            }
        }
    }
}

/// Advanced movement system with pathfinding support
pub fn pathfinding_movement_system(
    time: Res<Time>,
    mut query: Query<(&mut Transform, &mut MovementController)>,
) {
    let dt = time.delta_secs();
    
    for (mut transform, mut controller) in query.iter_mut() {
        // Check if we have a current target
        let current_target = if let Some(target) = controller.target_position {
            target
        } else if !controller.waypoints.is_empty() && controller.path_index < controller.waypoints.len() {
            // Get next waypoint
            let target = controller.waypoints[controller.path_index];
            controller.target_position = Some(target);
            target
        } else {
            controller.is_moving = false;
            continue;
        };
        
        let current_pos = transform.translation;
        let direction = current_target - current_pos;
        let distance = direction.length();
        
        // Check if we reached the current target
        if distance < 0.5 {
            controller.target_position = None;
            controller.path_index += 1;
            
            // Check if we've reached the end of the path
            if controller.path_index >= controller.waypoints.len() {
                controller.is_moving = false;
                controller.waypoints.clear();
                controller.path_index = 0;
                controller.velocity = Vec3::ZERO;
            }
            continue;
        }
        
        // Calculate desired velocity
        let desired_velocity = direction.normalize() * controller.max_speed;
        
        // Apply steering forces (seek behavior)
        let steering_force = (desired_velocity - controller.velocity) * controller.acceleration;
        controller.velocity += steering_force * dt;
        
        // Limit velocity to max speed
        if controller.velocity.length() > controller.max_speed {
            controller.velocity = controller.velocity.normalize() * controller.max_speed;
        }
        
        // Update position
        transform.translation += controller.velocity * dt;
        controller.is_moving = controller.velocity.length() > 0.1;
        
        // Rotate to face movement direction
        if controller.velocity.length() > 0.1 {
            let look_direction = controller.velocity.normalize();
            let target_rotation = Quat::from_rotation_y(look_direction.x.atan2(look_direction.z));
            transform.rotation = transform.rotation.slerp(target_rotation, controller.rotation_speed * dt);
        }
    }
}

/// Path-based movement system with waypoints
pub fn waypoint_movement_system(
    time: Res<Time>,
    mut query: Query<(&mut Transform, &mut MovementPath)>,
) {
    for (mut transform, mut path) in query.iter_mut() {
        if !path.is_moving || path.waypoints.is_empty() {
            continue;
        }
        
        if path.current_waypoint_index >= path.waypoints.len() {
            path.is_moving = false;
            path.current_waypoint_index = 0;
            continue;
        }
        
        let target = path.waypoints[path.current_waypoint_index];
        let direction = target - transform.translation;
        let distance = direction.length();
        
        if distance < 0.5 {
            // Reached waypoint, move to next
            path.current_waypoint_index += 1;
            if path.current_waypoint_index >= path.waypoints.len() {
                path.is_moving = false;
                path.current_waypoint_index = 0;
            }
        } else {
            // Move toward current waypoint
            let movement = direction.normalize() * path.movement_speed * time.delta_secs();
            transform.translation += movement;
            
            // Rotate to face movement direction
            if direction.length() > 0.01 {
                let look_direction = direction.normalize();
                let target_rotation = Quat::from_rotation_y(look_direction.x.atan2(look_direction.z));
                transform.rotation = transform.rotation.slerp(target_rotation, 5.0 * time.delta_secs());
            }
        }
    }
}

/// Formation movement system for group coordination
pub fn formation_movement_system(
    mut query: Query<(&mut MovementController, &Transform, &FormationMember)>,
    formation_query: Query<(&Transform, &Formation), Without<FormationMember>>,
) {
    for (mut controller, transform, member) in query.iter_mut() {
        if let Ok((formation_transform, formation)) = formation_query.get(member.formation_entity) {
            // Calculate desired position in formation
            let formation_offset = calculate_formation_position(
                member.slot_index,
                formation.formation_type,
                formation.spacing,
            );
            
            // Transform offset by formation rotation
            let rotated_offset = formation_transform.rotation * formation_offset;
            let desired_position = formation_transform.translation + rotated_offset;
            
            // Set target position
            controller.target_position = Some(desired_position);
            
            // Calculate urgency based on distance from formation
            let distance_from_formation = transform.translation.distance(desired_position);
            let urgency = (distance_from_formation / formation.spacing).clamp(0.1, 2.0);
            
            controller.max_speed = controller.max_speed * urgency;
        }
    }
}

/// Flocking/steering behavior system
pub fn flocking_system(
    query: Query<(&mut MovementController, &Transform, &FlockingAgent)>,
    neighbor_query: Query<&Transform, (With<FlockingAgent>, Without<MovementController>)>,
) {
    let mut flocking_forces: Vec<(Entity, Vec3)> = Vec::new();
    
    for (_controller, transform, agent) in query.iter() {
        let mut separation = Vec3::ZERO;
        let mut alignment = Vec3::ZERO;
        let mut cohesion = Vec3::ZERO;
        let mut neighbor_count = 0;
        
        // Calculate flocking forces
        for neighbor_transform in neighbor_query.iter() {
            let distance = transform.translation.distance(neighbor_transform.translation);
            
            if distance > 0.1 && distance < agent.perception_radius {
                neighbor_count += 1;
                
                // Separation: steer away from neighbors
                if distance < agent.separation_radius {
                    let away = (transform.translation - neighbor_transform.translation).normalize();
                    separation += away / distance; // Weight by inverse distance
                }
                
                // Alignment: align with neighbor velocities (would need velocity component)
                // For now, just align with neighbor positions
                alignment += neighbor_transform.translation;
                
                // Cohesion: steer toward average position of neighbors
                cohesion += neighbor_transform.translation;
            }
        }
        
        if neighbor_count > 0 {
            // Average the forces
            alignment = (alignment / neighbor_count as f32 - transform.translation).normalize();
            cohesion = (cohesion / neighbor_count as f32 - transform.translation).normalize();
            
            // Combine forces with weights
            let total_force = separation * agent.separation_weight +
                            alignment * agent.alignment_weight +
                            cohesion * agent.cohesion_weight;
            
            flocking_forces.push((Entity::PLACEHOLDER, total_force)); // Would need actual entity
        }
    }
    
    // Apply flocking forces (would need to match entities)
    // This is simplified - in practice, you'd match entities properly
}

/// Obstacle avoidance system
pub fn obstacle_avoidance_system(
    mut query: Query<(&mut MovementController, &Transform)>,
    obstacle_query: Query<&Transform, (With<Obstacle>, Without<MovementController>)>,
) {
    for (mut controller, transform) in query.iter_mut() {
        if !controller.is_moving {
            continue;
        }
        
        let look_ahead_distance = controller.velocity.length() * 2.0; // 2 second look ahead
        let look_ahead_pos = transform.translation + controller.velocity.normalize() * look_ahead_distance;
        
        let mut avoidance_force = Vec3::ZERO;
        let mut closest_obstacle_distance = f32::INFINITY;
        
        // Check for obstacles in the path
        for obstacle_transform in obstacle_query.iter() {
            let obstacle_pos = obstacle_transform.translation;
            let distance_to_obstacle = look_ahead_pos.distance(obstacle_pos);
            
            if distance_to_obstacle < 3.0 && distance_to_obstacle < closest_obstacle_distance {
                closest_obstacle_distance = distance_to_obstacle;
                
                // Calculate avoidance direction (perpendicular to movement)
                let to_obstacle = obstacle_pos - transform.translation;
                let movement_dir = controller.velocity.normalize();
                
                // Use cross product to get perpendicular direction
                let avoid_dir = movement_dir.cross(Vec3::Y).normalize();
                
                // Determine which side to avoid to
                if to_obstacle.dot(avoid_dir) < 0.0 {
                    let _ = avoid_dir * -1.0;
                }
                
                // Stronger avoidance for closer obstacles
                let avoidance_strength = (3.0 - distance_to_obstacle) / 3.0;
                avoidance_force += avoid_dir * avoidance_strength * controller.max_speed;
            }
        }
        
        // Apply avoidance force
        if avoidance_force.length() > 0.1 {
            controller.velocity += avoidance_force * 0.5; // Moderate influence
            
            // Ensure we don't exceed max speed
            if controller.velocity.length() > controller.max_speed {
                controller.velocity = controller.velocity.normalize() * controller.max_speed;
            }
        }
    }
}

// ==============================================================================
// MOVEMENT HELPER COMPONENTS AND FUNCTIONS
// ==============================================================================

#[derive(Component, Clone, Debug)]
pub struct FormationMember {
    pub formation_entity: Entity,
    pub slot_index: usize,
}

#[derive(Component, Clone, Debug)]
pub struct Formation {
    pub formation_type: FormationType,
    pub spacing: f32,
    pub members: Vec<Entity>,
}

#[derive(Clone, Copy, Debug)]
pub enum FormationType {
    Line,
    Column,
    Wedge,
    Circle,
}

#[derive(Component, Clone, Debug)]
pub struct FlockingAgent {
    pub perception_radius: f32,
    pub separation_radius: f32,
    pub separation_weight: f32,
    pub alignment_weight: f32,
    pub cohesion_weight: f32,
}

impl Default for FlockingAgent {
    fn default() -> Self {
        Self {
            perception_radius: 10.0,
            separation_radius: 3.0,
            separation_weight: 1.5,
            alignment_weight: 1.0,
            cohesion_weight: 1.0,
        }
    }
}


/// Calculate position offset for a unit in formation
fn calculate_formation_position(
    slot_index: usize,
    formation_type: FormationType,
    spacing: f32,
) -> Vec3 {
    match formation_type {
        FormationType::Line => {
            let offset_x = (slot_index as f32 - 2.0) * spacing;
            Vec3::new(offset_x, 0.0, 0.0)
        }
        FormationType::Column => {
            let offset_z = -(slot_index as f32) * spacing;
            Vec3::new(0.0, 0.0, offset_z)
        }
        FormationType::Wedge => {
            let row = (slot_index as f32 / 2.0).floor();
            let side = if slot_index % 2 == 0 { -1.0 } else { 1.0 };
            let offset_x = side * (row + 1.0) * spacing * 0.5;
            let offset_z = -row * spacing;
            Vec3::new(offset_x, 0.0, offset_z)
        }
        FormationType::Circle => {
            let angle = (slot_index as f32) * (2.0 * std::f32::consts::PI / 8.0); // 8 positions
            let radius = spacing * 2.0;
            Vec3::new(radius * angle.cos(), 0.0, radius * angle.sin())
        }
    }
}