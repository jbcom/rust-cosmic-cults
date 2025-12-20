use game_physics::Velocity;
// Targeting system using Rapier3D for physics-based line of sight and range checks
use bevy::prelude::*;
// use bevy_rapier3d::prelude::*;
use serde::{Deserialize, Serialize};

/// Component that marks an entity as targetable
#[derive(Component, Clone, Debug)]
pub struct Targetable {
    pub team_id: u32,
    pub priority: f32, // Higher priority targets are preferred
    pub is_visible: bool,
}

/// Component for entities that can acquire targets
#[derive(Component, Clone, Debug)]
pub struct TargetingSystem {
    pub range: f32,
    pub field_of_view: f32, // In radians
    pub current_target: Option<Entity>,
    pub target_lock_time: f32,
    pub can_target_air: bool,
    pub can_target_ground: bool,
}

impl Default for TargetingSystem {
    fn default() -> Self {
        Self {
            range: 30.0,
            field_of_view: std::f32::consts::PI, // 180 degrees
            current_target: None,
            target_lock_time: 0.0,
            can_target_air: true,
            can_target_ground: true,
        }
    }
}

/// Event fired when a new target is acquired
#[derive(Event, Clone, Debug)]
pub struct TargetAcquiredEvent {
    pub entity: Entity,
    pub target: Entity,
    pub distance: f32,
}

/// Event fired when a target is lost
#[derive(Event, Clone, Debug)]
pub struct TargetLostEvent {
    pub entity: Entity,
    pub previous_target: Entity,
    pub reason: TargetLostReason,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum TargetLostReason {
    OutOfRange,
    LineOfSightBlocked,
    TargetDestroyed,
    ManualDisengage,
}

pub struct TargetingPlugin;

impl Plugin for TargetingPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<TargetAcquiredEvent>()
            .add_message::<TargetLostEvent>()
            .add_systems(
                Update,
                (
                    target_acquisition_system,
                    target_validation_system,
                    line_of_sight_system,
                )
                    .chain(),
            );
    }
}

/// System that handles target acquisition
pub fn target_acquisition_system(
    mut targeting_query: Query<(Entity, &mut TargetingSystem, &Transform, &Targetable)>,
    targetable_query: Query<(Entity, &Transform, &Targetable), Without<TargetingSystem>>,
    // TODO: Add proper Rapier integration when available
    mut target_acquired_events: MessageWriter<TargetAcquiredEvent>,
) {
    for (entity, mut targeting, transform, my_team) in targeting_query.iter_mut() {
        // Skip if we already have a valid target
        if targeting.current_target.is_some() {
            continue;
        }

        let mut best_target: Option<(Entity, f32, f32)> = None; // (entity, distance, score)

        for (target_entity, target_transform, target_team) in targetable_query.iter() {
            // Don't target same team
            if target_team.team_id == my_team.team_id {
                continue;
            }

            // Check if target is visible
            if !target_team.is_visible {
                continue;
            }

            let distance = transform.translation.distance(target_transform.translation);

            // Check range
            if distance > targeting.range {
                continue;
            }

            // Check field of view
            if !is_in_field_of_view(transform, target_transform, targeting.field_of_view) {
                continue;
            }

            // TODO: Add line of sight check with Rapier when available
            // For now, assume line of sight is clear

            // Calculate target score (prefer closer, higher priority targets)
            let distance_score = 1.0 - (distance / targeting.range);
            let priority_score = target_team.priority;
            let total_score = distance_score + priority_score;

            if best_target.is_none() || total_score > best_target.as_ref().unwrap().2 {
                best_target = Some((target_entity, distance, total_score));
            }
        }

        // Acquire the best target
        if let Some((target_entity, distance, _score)) = best_target {
            targeting.current_target = Some(target_entity);
            targeting.target_lock_time = 0.0;

            target_acquired_events.write(TargetAcquiredEvent {
                entity,
                target: target_entity,
                distance,
            });
        }
    }
}

/// System that validates current targets are still valid
pub fn target_validation_system(
    mut targeting_query: Query<(Entity, &mut TargetingSystem, &Transform, &Targetable)>,
    targetable_query: Query<(&Transform, &Targetable)>,
    mut target_lost_events: MessageWriter<TargetLostEvent>,
    time: Res<Time>,
) {
    for (entity, mut targeting, transform, my_team) in targeting_query.iter_mut() {
        if let Some(target_entity) = targeting.current_target {
            let mut lose_target = false;
            let mut reason = TargetLostReason::TargetDestroyed;

            // Check if target still exists and is valid
            if let Ok((target_transform, target_team)) = targetable_query.get(target_entity) {
                let distance = transform.translation.distance(target_transform.translation);

                // Check range
                if distance > targeting.range * 1.2 {
                    // Add some buffer to prevent flickering
                    lose_target = true;
                    reason = TargetLostReason::OutOfRange;
                }

                // Check if still enemy
                if target_team.team_id == my_team.team_id {
                    lose_target = true;
                    reason = TargetLostReason::ManualDisengage;
                }

                // Check if still visible
                if !target_team.is_visible {
                    lose_target = true;
                    reason = TargetLostReason::LineOfSightBlocked;
                }

                // Update lock time
                if !lose_target {
                    targeting.target_lock_time += time.delta_secs();
                }
            } else {
                // Target no longer exists
                lose_target = true;
                reason = TargetLostReason::TargetDestroyed;
            }

            if lose_target {
                target_lost_events.write(TargetLostEvent {
                    entity,
                    previous_target: target_entity,
                    reason,
                });

                targeting.current_target = None;
                targeting.target_lock_time = 0.0;
            }
        }
    }
}

/// System that performs detailed line of sight checks
pub fn line_of_sight_system(
    targeting_query: Query<(Entity, &TargetingSystem, &Transform)>,
    transform_query: Query<&Transform>,
) {
    for (_entity, targeting, _transform) in targeting_query.iter() {
        if let Some(target_entity) = targeting.current_target {
            if let Ok(_target_transform) = transform_query.get(target_entity) {
                // TODO: Add detailed raycast with Rapier when available
                // For now, assume line of sight is always clear
            }
        }
    }
}

/// Helper function to check if target is within field of view
fn is_in_field_of_view(observer: &Transform, target: &Transform, field_of_view: f32) -> bool {
    let to_target = (target.translation - observer.translation).normalize();
    let forward = observer.rotation * Vec3::Z;
    let angle = forward.angle_between(to_target);
    angle <= field_of_view / 2.0
}

/// Component for projectiles that tracks their target
#[derive(Component)]
pub struct HomingProjectile {
    pub target: Entity,
    pub turn_speed: f32,
    pub acceleration: f32,
}

/// System for homing projectiles
pub fn homing_projectile_system(
    mut projectile_query: Query<(&mut Transform, &mut Velocity, &HomingProjectile)>,
    target_query: Query<&Transform, Without<HomingProjectile>>,
    time: Res<Time>,
) {
    for (mut transform, mut velocity, homing) in projectile_query.iter_mut() {
        if let Ok(target_transform) = target_query.get(homing.target) {
            // Calculate direction to target
            let to_target = (target_transform.translation - transform.translation).normalize();

            // Smoothly rotate towards target
            let current_dir = transform.rotation * Vec3::Z;
            let new_dir = current_dir.lerp(to_target, homing.turn_speed * time.delta_secs());
            transform.look_to(new_dir, Vec3::Y);

            // Accelerate towards target
            velocity.linear += new_dir * homing.acceleration * time.delta_secs();

            // Cap max speed
            let max_speed = 50.0;
            if velocity.linear.length() > max_speed {
                velocity.linear = velocity.linear.normalize() * max_speed;
            }
        }
    }
}
impl bevy::prelude::Message for TargetAcquiredEvent {}
impl bevy::prelude::Message for TargetLostEvent {}
