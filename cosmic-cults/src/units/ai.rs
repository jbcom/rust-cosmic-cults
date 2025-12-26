use bevy::prelude::*;
use big_brain::prelude::*;
use crate::units::components::*;
use crate::units::combat::DamageEvent;
use avian3d::prelude::*;

/// Marker component for AI-controlled units
#[derive(Component, Debug, Reflect, Default)]
#[reflect(Component)]
pub struct UnitAI;

// --- Actions ---

/// Action for a unit to move to a target position
#[derive(ActionBuilder, Component, Debug, Clone, Reflect, Default)]
#[reflect(Component)]
pub struct MoveToAction;

pub fn move_to_action_system(
    mut action_query: Query<(&Actor, &mut ActionState, &MoveToAction)>,
    mut unit_query: Query<(&Transform, &mut LinearVelocity, &Unit, &MovementPath)>,
    _time: Res<Time>,
) {
    for (actor, mut state, _move_to) in action_query.iter_mut() {
        if let Ok((transform, mut velocity, unit, path)) = unit_query.get_mut(actor.0) {
            match *state {
                ActionState::Requested => {
                    *state = ActionState::Executing;
                }
                ActionState::Executing => {
                    if path.waypoints.is_empty() {
                        velocity.0 = Vec3::ZERO;
                        *state = ActionState::Success;
                        return;
                    }

                    let target = path.waypoints[path.current_index];
                    let direction = target - transform.translation;
                    let distance = direction.length();

                    if distance < 0.5 {
                        velocity.0 = Vec3::ZERO;
                        *state = ActionState::Success;
                    } else {
                        let move_dir = direction.normalize();
                        velocity.0 = move_dir * unit.movement_speed;
                    }
                }
                ActionState::Cancelled => {
                    velocity.0 = Vec3::ZERO;
                    *state = ActionState::Failure;
                }
                _ => {}
            }
        }
    }
}

/// Action for a unit to gather resources
#[derive(ActionBuilder, Component, Debug, Clone, Reflect, Default)]
#[reflect(Component)]
pub struct GatherAction;

pub fn gather_action_system(
    mut action_query: Query<(&Actor, &mut ActionState, &GatherAction)>,
    mut unit_query: Query<(&Transform, &mut Resources, &Team)>,
    mut node_query: Query<&mut ResourceNode>,
    spatial_query: SpatialQuery,
    time: Res<Time>,
) {
    for (actor, mut state, _gather) in action_query.iter_mut() {
        if let Ok((transform, mut resources, _team)) = unit_query.get_mut(actor.0) {
            match *state {
                ActionState::Requested => {
                    *state = ActionState::Executing;
                }
                ActionState::Executing => {
                    // Find nearest resource node using spatial query
                    let mut nearest_node_entity = None;
                    
                    let intersections = spatial_query.shape_intersections(
                        &Collider::sphere(2.0),
                        transform.translation,
                        Quat::IDENTITY,
                        &SpatialQueryFilter::default(),
                    );

                    for entity in intersections {
                        if node_query.contains(entity) {
                            nearest_node_entity = Some(entity);
                            break;
                        }
                    }

                    if let Some(entity) = nearest_node_entity {
                        if let Ok(mut node) = node_query.get_mut(entity) {
                            let gather_rate = 10.0 * time.delta_secs();
                            let amount = gather_rate.min(node.amount);
                            
                            node.amount -= amount;
                            match node.resource_type {
                                ResourceType::Energy => resources.energy += amount,
                                ResourceType::Materials => resources.materials += amount,
                                ResourceType::Favor => resources.favor += amount,
                            }

                            if node.amount <= 0.0 {
                                *state = ActionState::Success;
                            }
                        }
                    } else {
                        *state = ActionState::Failure;
                    }
                }
                ActionState::Cancelled => {
                    *state = ActionState::Failure;
                }
                _ => {}
            }
        }
    }
}

/// Action for a unit to attack a target
#[derive(ActionBuilder, Component, Debug, Clone, Reflect, Default)]
#[reflect(Component)]
pub struct AttackAction;

pub fn attack_action_system(
    mut action_query: Query<(&Actor, &mut ActionState, &AttackAction)>,
    mut attacker_query: Query<(&Transform, &mut CombatStats, &AttackTarget, &Team)>,
    target_query: Query<(&Transform, &Team, &Health)>,
    time: Res<Time>,
    mut damage_events: MessageWriter<DamageEvent>,
) {
    for (actor, mut state, _attack) in action_query.iter_mut() {
        if let Ok((attacker_transform, mut stats, target_comp, attacker_team)) = attacker_query.get_mut(actor.0) {
            let Some(target_entity) = target_comp.entity else {
                *state = ActionState::Failure;
                continue;
            };

            if let Ok((target_transform, target_team, target_health)) = target_query.get(target_entity) {
                if target_health.current <= 0.0 || target_team.id == attacker_team.id {
                    *state = ActionState::Success; // Target gone or ally
                    continue;
                }

                match *state {
                    ActionState::Requested => {
                        *state = ActionState::Executing;
                    }
                    ActionState::Executing => {
                        let dist = attacker_transform.translation.distance(target_transform.translation);
                        if dist <= stats.attack_range {
                            // Can attack
                            if time.elapsed_secs() - stats.last_attack_time >= 1.0 / stats.attack_speed {
                                damage_events.write(DamageEvent {
                                    target: target_entity,
                                    damage: stats.attack_damage,
                                    attacker: actor.0,
                                });
                                stats.last_attack_time = time.elapsed_secs();
                            }
                        } else {
                            // Too far - check if we can find any other target in range?
                            // For now, failure if current target is out of range
                            *state = ActionState::Failure;
                        }
                    }
                    ActionState::Cancelled => {
                        *state = ActionState::Failure;
                    }
                    _ => {}
                }
            } else {
                // Target might have been despawned
                *state = ActionState::Failure;
            }
        }
    }
}

// --- Scorers ---

/// Scorer that returns high value if the unit has a movement path
#[derive(ScorerBuilder, Component, Debug, Clone, Reflect, Default)]
#[reflect(Component)]
pub struct HasPathScorer;

pub fn has_path_scorer_system(
    unit_query: Query<&MovementPath>,
    mut scorer_query: Query<(&Actor, &mut Score), With<HasPathScorer>>,
) {
    for (actor, mut score) in scorer_query.iter_mut() {
        if let Ok(path) = unit_query.get(actor.0) {
            if !path.waypoints.is_empty() {
                score.set(1.0);
            } else {
                score.set(0.0);
            }
        }
    }
}

/// Scorer that returns high value if the unit is near a resource node
#[derive(ScorerBuilder, Component, Debug, Clone, Reflect, Default)]
#[reflect(Component)]
pub struct NearResourceScorer;

pub fn near_resource_scorer_system(
    unit_query: Query<&Transform, With<Unit>>,
    node_query: Query<&ResourceNode>,
    spatial_query: SpatialQuery,
    mut scorer_query: Query<(&Actor, &mut Score), With<NearResourceScorer>>,
) {
    for (actor, mut score) in scorer_query.iter_mut() {
        if let Ok(transform) = unit_query.get(actor.0) {
            let intersections = spatial_query.shape_intersections(
                &Collider::sphere(2.0),
                transform.translation,
                Quat::IDENTITY,
                &SpatialQueryFilter::default(),
            );
            
            let found = intersections.iter().any(|&entity| node_query.contains(entity));
            
            if found {
                score.set(1.0);
            } else {
                score.set(0.0);
            }
        }
    }
}

/// Scorer that returns high value if an enemy is in range
#[derive(ScorerBuilder, Component, Debug, Clone, Reflect, Default)]
#[reflect(Component)]
pub struct EnemyInRangeScorer;

pub fn enemy_in_range_scorer_system(
    attacker_query: Query<(&Transform, &Team, &CombatStats), With<Unit>>,
    target_query: Query<(&Team, &Health), With<Unit>>,
    spatial_query: SpatialQuery,
    mut scorer_query: Query<(&Actor, &mut Score), With<EnemyInRangeScorer>>,
) {
    for (actor, mut score) in scorer_query.iter_mut() {
        if let Ok((transform, team, stats)) = attacker_query.get(actor.0) {
            let intersections = spatial_query.shape_intersections(
                &Collider::sphere(stats.attack_range),
                transform.translation,
                Quat::IDENTITY,
                &SpatialQueryFilter::default(),
            );

            let mut found = false;
            for entity in intersections {
                if let Ok((t_team, t_health)) = target_query.get(entity) {
                    if t_team.id != team.id && t_health.current > 0.0 {
                        found = true;
                        break;
                    }
                }
            }

            if found {
                score.set(1.0);
            } else {
                score.set(0.0);
            }
        }
    }
}

/// Plugin to register AI systems
pub struct UnitAIPlugin;

impl Plugin for UnitAIPlugin {
    fn build(&self, app: &mut App) {
        app
            .register_type::<UnitAI>()
            .register_type::<MoveToAction>()
            .register_type::<GatherAction>()
            .register_type::<AttackAction>()
            .register_type::<HasPathScorer>()
            .register_type::<NearResourceScorer>()
            .register_type::<EnemyInRangeScorer>()
            .add_systems(Update, (
                move_to_action_system,
                gather_action_system,
                attack_action_system,
                has_path_scorer_system,
                near_resource_scorer_system,
                enemy_in_range_scorer_system,
            ));
    }
}
