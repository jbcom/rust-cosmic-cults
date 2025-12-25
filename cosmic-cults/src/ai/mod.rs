pub mod cult_profiles;
pub mod behaviors;
pub mod types;

use bevy::prelude::*;
use bevy_ai_toolkit::prelude::*;
use game_physics::{MovementCommand, MovementCommandEvent, MovementController, Velocity};

use crate::ai::types::{AICoordination, AIRole};
use crate::ai::behaviors::{AttackBehavior, DefendBehavior, GatheringBehavior, RetreatBehavior};

pub struct CosmicCultsAIPlugin;

impl Plugin for CosmicCultsAIPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(BevyAIToolkitPlugin)
            .add_systems(Update, (
                cult_profiles::update_psychological_state_system,
                cult_profiles::handle_psychological_events,
                ai_coordination_system,
                ai_action_execution_system,
            ));
    }
}

fn ai_coordination_system(
    leaders_query: Query<(Entity, &AICoordination, &Transform)>,
    followers_query: Query<(Entity, &AICoordination, &Transform)>,
    mut commands: Commands,
) {
    for (leader_entity, leader_coord, leader_transform) in leaders_query.iter() {
        if leader_coord.can_give_orders && leader_coord.role == AIRole::Leader {
            for (follower_entity, follower_coord, follower_transform) in followers_query.iter() {
                if leader_entity == follower_entity || !follower_coord.can_receive_orders || follower_coord.team_id != leader_coord.team_id {
                    continue;
                }

                let distance = leader_transform.translation.distance(follower_transform.translation);
                if distance <= leader_coord.coordination_radius {
                    commands.entity(follower_entity).insert(CoordinatedBehavior {
                        leader: leader_entity,
                        role: leader_coord.role.clone(),
                    });
                }
            }
        }
    }
}

#[derive(Component)]
pub struct CoordinatedBehavior {
    pub leader: Entity,
    pub role: AIRole,
}

fn ai_action_execution_system(
    mut movement_events: EventWriter<MovementCommandEvent>,
    gathering_query: Query<(Entity, &GatheringBehavior, &Transform), Added<GatheringBehavior>>,
    attack_query: Query<(Entity, &AttackBehavior, &Transform), Added<AttackBehavior>>,
    defend_query: Query<(Entity, &DefendBehavior, &Transform), Added<DefendBehavior>>,
    retreat_query: Query<(Entity, &RetreatBehavior, &Transform), Added<RetreatBehavior>>,
    mut commands: Commands,
) {
    for (entity, gathering, _transform) in gathering_query.iter() {
        if let Some(target_resource) = gathering.target_resource {
            movement_events.send(MovementCommandEvent {
                entity,
                command: MovementCommand::Follow {
                    target: target_resource,
                    distance: 2.0,
                },
            });
        }
        commands.entity(entity).insert((MovementController::default(), Velocity::default()));
    }

    for (entity, attack, _transform) in attack_query.iter() {
        if let Some(target) = attack.target {
            movement_events.send(MovementCommandEvent {
                entity,
                command: MovementCommand::Follow {
                    target,
                    distance: 1.5,
                },
            });
        }
        commands.entity(entity).insert((MovementController::default(), Velocity::default()));
    }

    for (entity, defend, _transform) in defend_query.iter() {
        let patrol_points = vec![
            defend.defend_position + Vec3::new(defend.patrol_radius, 0.0, 0.0),
            defend.defend_position + Vec3::new(0.0, 0.0, defend.patrol_radius),
            defend.defend_position + Vec3::new(-defend.patrol_radius, 0.0, 0.0),
            defend.defend_position + Vec3::new(0.0, 0.0, -defend.patrol_radius),
        ];

        movement_events.send(MovementCommandEvent {
            entity,
            command: MovementCommand::SetPath {
                waypoints: patrol_points,
                speed: 3.0,
            },
        });
        commands.entity(entity).insert((MovementController::default(), Velocity::default()));
    }

    for (entity, retreat, transform) in retreat_query.iter() {
        let safe_position = retreat.safe_position.unwrap_or_else(|| {
            transform.translation + Vec3::new(-10.0, 0.0, -10.0)
        });

        movement_events.send(MovementCommandEvent {
            entity,
            command: MovementCommand::MoveTo {
                position: safe_position,
                speed: 5.0,
            },
        });
        commands.entity(entity).insert((MovementController::default(), Velocity::default()));
    }
}
