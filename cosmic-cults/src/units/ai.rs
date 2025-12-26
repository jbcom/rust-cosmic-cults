use bevy::prelude::*;
use big_brain::prelude::*;
use crate::units::components::*;

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
    mut unit_query: Query<(&mut Transform, &Unit, &MovementPath)>,
    time: Res<Time>,
) {
    for (actor, mut state, _move_to) in action_query.iter_mut() {
        if let Ok((mut transform, unit, path)) = unit_query.get_mut(actor.0) {
            match *state {
                ActionState::Requested => {
                    *state = ActionState::Executing;
                }
                ActionState::Executing => {
                    if path.waypoints.is_empty() {
                        *state = ActionState::Success;
                        return;
                    }

                    let target = path.waypoints[path.current_index];
                    let direction = target - transform.translation;
                    let distance = direction.length();

                    if distance < 0.2 {
                        *state = ActionState::Success;
                    } else {
                        let move_dir = direction.normalize();
                        transform.translation += move_dir * unit.movement_speed * time.delta_secs();
                        
                        // Rotate towards movement
                        let target_rotation = Quat::from_rotation_y(move_dir.x.atan2(move_dir.z));
                        transform.rotation = transform.rotation.slerp(target_rotation, 5.0 * time.delta_secs());
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

/// Action for a unit to gather resources
#[derive(ActionBuilder, Component, Debug, Clone, Reflect, Default)]
#[reflect(Component)]
pub struct GatherAction;

pub fn gather_action_system(
    mut action_query: Query<(&Actor, &mut ActionState, &GatherAction)>,
    mut unit_query: Query<(&Transform, &mut Resources, &Team)>,
    mut node_query: Query<(Entity, &Transform, &mut ResourceNode)>,
    time: Res<Time>,
) {
    for (actor, mut state, _gather) in action_query.iter_mut() {
        if let Ok((transform, mut resources, _team)) = unit_query.get_mut(actor.0) {
            match *state {
                ActionState::Requested => {
                    *state = ActionState::Executing;
                }
                ActionState::Executing => {
                    // Find nearest resource node
                    let mut nearest_node = None;
                    let mut min_dist = 2.0; // Gathering range

                    for (node_entity, node_transform, node) in node_query.iter_mut() {
                        let dist = transform.translation.distance(node_transform.translation);
                        if dist < min_dist {
                            min_dist = dist;
                            nearest_node = Some((node_entity, node));
                        }
                    }

                    if let Some((_entity, mut node)) = nearest_node {
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
    node_query: Query<&Transform, With<ResourceNode>>,
    mut scorer_query: Query<(&Actor, &mut Score), With<NearResourceScorer>>,
) {
    for (actor, mut score) in scorer_query.iter_mut() {
        if let Ok(transform) = unit_query.get(actor.0) {
            let mut found = false;
            for node_transform in node_query.iter() {
                if transform.translation.distance(node_transform.translation) < 2.0 {
                    score.set(1.0);
                    found = true;
                    break;
                }
            }
            if !found {
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
            .register_type::<HasPathScorer>()
            .register_type::<NearResourceScorer>()
            .add_systems(Update, (
                move_to_action_system,
                gather_action_system,
                has_path_scorer_system,
                near_resource_scorer_system,
            ));
    }
}
