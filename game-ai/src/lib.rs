// Game AI crate - Production-ready AI systems for Cosmic Dominion
#![allow(unused)]
#![allow(clippy::type_complexity)]
#![allow(clippy::collapsible_if)]
#![allow(clippy::vec_box)]
#![allow(clippy::new_without_default)]

use bevy::prelude::*;
use game_physics::prelude::*;
use game_units::{Leader, Team, Unit};
use std::collections::HashMap;

// Core modules
pub mod behaviors;
pub mod cult_profiles;
pub mod decision;
pub mod states;
pub mod systems;
pub mod targeting;
pub mod types;

#[cfg(test)]
pub mod integration_test;

// Public re-exports for easy access - specific imports to avoid conflicts
pub use behaviors::{BehaviorNode, BehaviorTree, Blackboard, BlackboardValue, NodeStatus};
pub use cult_profiles::{
    CultProfile, PsychologicalEvent, PsychologicalState, create_cult_ai, create_cult_coordination,
    create_cult_profile,
};
pub use decision::*;
pub use states::{AIState, AIStateMachine, StateTransitionTrigger};
pub use systems::ai_execution::{AICommandEvent, AIGlobalState, AIPerceptionEvent};
pub use systems::behavior_tree::BehaviorTree as OldBehaviorTree;
pub use systems::decision_making::AIDecisionMaker;
pub use systems::state_machine::{
    AIStateMachine as OldStateMachine, AttackBehavior, DefendBehavior, GatheringBehavior,
    RetreatBehavior,
};
pub use systems::utility_ai::*;
pub use targeting::*;
pub use types::{AICoordination, AIMessage, AIRole};

// SystemSets for organizing AI system execution order
#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub enum AISystemSet {
    /// Core AI systems that update AI state (state machines, behavior trees, psychology)
    CoreAI,
    /// Decision-making systems that evaluate and queue decisions/actions
    DecisionAI,
    /// Messaging and coordination systems that handle AI communication
    Messaging,
    /// Action execution systems that translate AI decisions into game commands
    Execution,
}

// Main AI plugin that integrates all AI systems
pub struct GameAIPlugin;

impl Plugin for GameAIPlugin {
    fn build(&self, app: &mut App) {
        app
            // Add AI events
            .add_message::<AIMessage>()
            .add_message::<PsychologicalEvent>()
            .add_message::<crate::systems::AICommandEvent>()
            .add_message::<crate::systems::AIPerceptionEvent>()
            // Add resources
            .insert_resource(crate::systems::AIGlobalState::default())
            // Configure SystemSets for AI ordering
            .configure_sets(
                Update,
                (
                    AISystemSet::CoreAI,
                    AISystemSet::DecisionAI,
                    AISystemSet::Messaging,
                    AISystemSet::Execution,
                )
                    .chain()
                    .run_if(any_ai_entities_exist),
            )
            // Core AI systems - update basic AI state
            .add_systems(
                Update,
                (
                    // From systems module
                    crate::systems::state_machine::state_machine_update_system,
                    crate::systems::behavior_tree::behavior_tree_system,
                    crate::systems::utility_ai::utility_ai_system,
                    // From new modules
                    crate::states::state_execution_system,
                    crate::states::state_transition_system,
                    crate::behaviors::behavior_tree_execution_system,
                    crate::cult_profiles::update_psychological_state_system,
                )
                    .chain()
                    .in_set(AISystemSet::CoreAI),
            )
            // Decision-making systems - evaluate and select actions
            .add_systems(
                Update,
                (
                    crate::systems::decision_making::decision_making_system,
                    crate::decision::decision_system,
                    crate::decision::goal_execution_system,
                    crate::targeting::target_acquisition_system,
                    crate::targeting::target_validation_system,
                )
                    .chain()
                    .in_set(AISystemSet::DecisionAI),
            )
            // Communication and coordination systems
            .add_systems(
                Update,
                (
                    ai_coordination_system,
                    crate::cult_profiles::handle_psychological_events,
                    crate::systems::perception_system,
                    crate::systems::squad_coordination_system,
                )
                    .chain()
                    .in_set(AISystemSet::Messaging),
            )
            // Action execution systems - translate decisions to game commands
            .add_systems(
                Update,
                (
                    ai_action_execution_system,
                    crate::systems::ai_movement_system,
                    crate::systems::ai_combat_system,
                    crate::targeting::line_of_sight_system,
                    crate::targeting::target_prediction_system,
                )
                    .chain()
                    .in_set(AISystemSet::Execution),
            );
    }
}

// Condition to check if any AI entities exist
fn any_ai_entities_exist(
    ai_query: Query<
        (),
        Or<(
            With<OldStateMachine>,
            With<OldBehaviorTree>,
            With<UtilityAI>,
            With<AIDecisionMaker>,
            With<AICoordination>,
            With<CultProfile>,
            With<PsychologicalState>,
            With<DecisionMaker>,
            With<TargetSelector>,
        )>,
    >,
) -> bool {
    !ai_query.is_empty()
}

// AI coordination system - separated queries to avoid borrowing conflicts
fn ai_coordination_system(
    leaders_query: Query<(Entity, &AICoordination, &Transform), With<Leader>>,
    followers_query: Query<
        (Entity, &AICoordination, &Transform),
        (Without<Leader>, With<AICoordination>),
    >,
    mut commands: Commands,
) {
    // Process leaders and their coordination separately
    for (leader_entity, leader_coord, leader_transform) in leaders_query.iter() {
        if leader_coord.can_give_orders && leader_coord.role == AIRole::Leader {
            let leader_position = leader_transform.translation;

            // Check all potential followers
            for (follower_entity, follower_coord, follower_transform) in followers_query.iter() {
                if !follower_coord.can_receive_orders
                    || follower_coord.team_id != leader_coord.team_id
                {
                    continue;
                }

                let distance = leader_position.distance(follower_transform.translation);
                if distance <= leader_coord.coordination_radius {
                    // Add coordination behavior
                    commands
                        .entity(follower_entity)
                        .insert(CoordinatedBehavior {
                            leader: leader_entity,
                            role: leader_coord.role.clone(),
                        });
                }
            }
        }
    }
}

// Coordination behavior component
#[derive(Component)]
pub struct CoordinatedBehavior {
    pub leader: Entity,
    pub role: AIRole,
}

// AI action execution system that translates AI behaviors into physics commands
fn ai_action_execution_system(
    mut movement_events: MessageWriter<MovementCommandEvent>,
    gathering_query: Query<
        (
            Entity,
            &crate::systems::state_machine::GatheringBehavior,
            &Transform,
        ),
        Added<crate::systems::state_machine::GatheringBehavior>,
    >,
    attack_query: Query<
        (
            Entity,
            &crate::systems::state_machine::AttackBehavior,
            &Transform,
        ),
        Added<crate::systems::state_machine::AttackBehavior>,
    >,
    defend_query: Query<
        (
            Entity,
            &crate::systems::state_machine::DefendBehavior,
            &Transform,
        ),
        Added<crate::systems::state_machine::DefendBehavior>,
    >,
    retreat_query: Query<
        (
            Entity,
            &crate::systems::state_machine::RetreatBehavior,
            &Transform,
        ),
        Added<crate::systems::state_machine::RetreatBehavior>,
    >,
    mut commands: Commands,
) {
    // Handle gathering behavior - move to resource location
    for (entity, gathering, transform) in gathering_query.iter() {
        if let Some(target_resource) = gathering.target_resource {
            movement_events.write(MovementCommandEvent {
                entity,
                command: MovementCommand::Follow {
                    target: target_resource,
                    distance: 2.0,
                },
            });
        }

        // Add movement controller if not present
        commands
            .entity(entity)
            .insert((MovementController::default(), Velocity::default()));
    }

    // Handle attack behavior - move to target and engage
    for (entity, attack, _transform) in attack_query.iter() {
        if let Some(target) = attack.target {
            movement_events.write(MovementCommandEvent {
                entity,
                command: MovementCommand::Follow {
                    target,
                    distance: 1.5, // Attack range
                },
            });
        }

        // Add movement controller if not present
        commands
            .entity(entity)
            .insert((MovementController::default(), Velocity::default()));
    }

    // Handle defend behavior - patrol around defense position
    for (entity, defend, _transform) in defend_query.iter() {
        // Create a patrol pattern around the defend position
        let patrol_points = vec![
            defend.defend_position + Vec3::new(defend.patrol_radius, 0.0, 0.0),
            defend.defend_position + Vec3::new(0.0, 0.0, defend.patrol_radius),
            defend.defend_position + Vec3::new(-defend.patrol_radius, 0.0, 0.0),
            defend.defend_position + Vec3::new(0.0, 0.0, -defend.patrol_radius),
        ];

        movement_events.write(MovementCommandEvent {
            entity,
            command: MovementCommand::SetPath {
                waypoints: patrol_points,
                speed: 3.0,
            },
        });

        // Add movement controller if not present
        commands
            .entity(entity)
            .insert((MovementController::default(), Velocity::default()));
    }

    // Handle retreat behavior - move to safety
    for (entity, retreat, transform) in retreat_query.iter() {
        let safe_position = retreat.safe_position.unwrap_or_else(|| {
            // If no specific safe position, move away from current position
            transform.translation + Vec3::new(-10.0, 0.0, -10.0)
        });

        movement_events.write(MovementCommandEvent {
            entity,
            command: MovementCommand::MoveTo {
                position: safe_position,
                speed: 5.0, // Fast retreat
            },
        });

        // Add movement controller if not present
        commands
            .entity(entity)
            .insert((MovementController::default(), Velocity::default()));
    }
}

// Helper functions for AI setup
impl GameAIPlugin {
    /// Create a basic AI entity with state machine
    pub fn spawn_basic_ai(commands: &mut Commands, position: Vec3, ai_type: AIRole) -> Entity {
        commands
            .spawn((
                Transform::from_translation(position),
                GlobalTransform::default(),
                crate::states::AIStateMachine::default(),
                AICoordination {
                    team_id: 1,
                    role: ai_type.clone(),
                    coordination_radius: 50.0,
                    can_give_orders: matches!(ai_type, AIRole::Leader),
                    can_receive_orders: true,
                },
                // Physics components
                MovementController::default(),
                Velocity::default(),
                SpatialData::new(position),
                CollisionMask::default(),
            ))
            .id()
    }

    /// Create an advanced AI entity with behavior tree
    pub fn spawn_behavior_tree_ai(
        commands: &mut Commands,
        position: Vec3,
        tree: crate::behaviors::BehaviorTree,
    ) -> Entity {
        commands
            .spawn((
                Transform::from_translation(position),
                GlobalTransform::default(),
                tree,
                AICoordination {
                    team_id: 1,
                    role: AIRole::Follower,
                    coordination_radius: 30.0,
                    can_give_orders: false,
                    can_receive_orders: true,
                },
                // Physics components
                MovementController::default(),
                Velocity::default(),
                SpatialData::new(position),
                CollisionMask::default(),
            ))
            .id()
    }

    /// Create a utility-based AI entity
    pub fn spawn_utility_ai(
        commands: &mut Commands,
        position: Vec3,
        utility_ai: UtilityAI,
    ) -> Entity {
        commands
            .spawn((
                Transform::from_translation(position),
                GlobalTransform::default(),
                utility_ai,
                AICoordination {
                    team_id: 1,
                    role: AIRole::Specialist("utility".to_string()),
                    coordination_radius: 40.0,
                    can_give_orders: false,
                    can_receive_orders: true,
                },
                // Physics components
                MovementController::default(),
                Velocity::default(),
                SpatialData::new(position),
                CollisionMask::default(),
            ))
            .id()
    }

    /// Create a cult-specific AI entity
    pub fn spawn_cult_ai(commands: &mut Commands, position: Vec3, cult_name: &str) -> Entity {
        let profile = create_cult_profile(cult_name);
        let ai = create_cult_ai(&profile);
        let coordination = create_cult_coordination(&profile, AIRole::Follower);
        let psychological = PsychologicalState::default();

        commands
            .spawn((
                Transform::from_translation(position),
                GlobalTransform::default(),
                profile,
                ai,
                coordination,
                psychological,
                // Add decision maker
                DecisionMaker::balanced(),
                // Add target selector
                TargetSelector::new(TargetPriority::Balanced),
                // Physics components
                MovementController::default(),
                Velocity::default(),
                SpatialData::new(position),
                CollisionMask::default(),
            ))
            .id()
    }
}

// Preset AI configurations
pub mod presets {
    use super::*;

    pub fn create_aggressive_leader() -> (crate::states::AIStateMachine, AICoordination) {
        let mut state_machine = crate::states::AIStateMachine::default();

        // Add aggressive transitions
        state_machine.transition_rules.insert(
            (
                crate::states::AIState::Idle,
                crate::states::StateTransitionTrigger::EnemyDetected,
            ),
            crate::states::AIState::Attacking,
        );
        state_machine.transition_rules.insert(
            (
                crate::states::AIState::Gathering,
                crate::states::StateTransitionTrigger::EnemyDetected,
            ),
            crate::states::AIState::Attacking,
        );

        let coordination = AICoordination {
            team_id: 1,
            role: AIRole::Leader,
            coordination_radius: 100.0,
            can_give_orders: true,
            can_receive_orders: false,
        };

        (state_machine, coordination)
    }

    pub fn create_defensive_guard() -> (crate::states::AIStateMachine, AICoordination) {
        let mut state_machine = crate::states::AIStateMachine::default();

        // Add defensive transitions
        state_machine.transition_rules.insert(
            (
                crate::states::AIState::Idle,
                crate::states::StateTransitionTrigger::AlertRaised,
            ),
            crate::states::AIState::Defending,
        );
        state_machine.transition_rules.insert(
            (
                crate::states::AIState::Attacking,
                crate::states::StateTransitionTrigger::HealthLow,
            ),
            crate::states::AIState::Defending,
        );

        let coordination = AICoordination {
            team_id: 1,
            role: AIRole::Defender,
            coordination_radius: 50.0,
            can_give_orders: false,
            can_receive_orders: true,
        };

        (state_machine, coordination)
    }

    pub fn create_economic_worker() -> (crate::states::AIStateMachine, AICoordination) {
        let mut state_machine = crate::states::AIStateMachine::default();

        // Add economic-focused transitions
        state_machine.transition_rules.insert(
            (
                crate::states::AIState::Idle,
                crate::states::StateTransitionTrigger::ResourceFound,
            ),
            crate::states::AIState::Gathering,
        );
        state_machine.transition_rules.insert(
            (
                crate::states::AIState::Gathering,
                crate::states::StateTransitionTrigger::ResourceGathered,
            ),
            crate::states::AIState::Building,
        );

        let coordination = AICoordination {
            team_id: 1,
            role: AIRole::Worker,
            coordination_radius: 30.0,
            can_give_orders: false,
            can_receive_orders: true,
        };

        (state_machine, coordination)
    }
}
