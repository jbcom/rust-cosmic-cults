// AI Execution Systems - handles AI movement, combat, perception and coordination
use bevy::prelude::*;
use game_physics::prelude::*;
use game_units::{Leader, Team, Unit};

// Events for AI communication
#[derive(Event, Clone, Debug)]
pub struct AICommandEvent {
    pub entity: Entity,
    pub command: AICommand,
}

#[derive(Clone, Debug)]
pub enum AICommand {
    MoveTo(Vec3),
    Attack(Entity),
    Defend(Vec3),
    Follow(Entity),
    Patrol(Vec<Vec3>),
    UseAbility(String),
}

#[derive(Event, Clone, Debug)]
pub struct AIPerceptionEvent {
    pub perceiver: Entity,
    pub perceived: Entity,
    pub perception_type: PerceptionType,
}

#[derive(Clone, Debug)]
pub enum PerceptionType {
    EnemySpotted,
    AllyInDanger,
    ResourceFound,
    ObstacleDetected,
    SoundHeard,
}

// Global AI state resource
#[derive(Resource, Default)]
pub struct AIGlobalState {
    pub active_ai_count: usize,
    pub global_threat_level: f32,
    pub faction_states: std::collections::HashMap<u32, FactionState>,
}

#[derive(Clone, Debug)]
pub struct FactionState {
    pub team_id: u32,
    pub morale: f32,
    pub aggression: f32,
    pub resources: f32,
}

// AI Movement System - handles movement commands for AI-controlled units
pub fn ai_movement_system(
    mut commands: Commands,
    mut ai_command_events: MessageReader<AICommandEvent>,
    mut movement_events: MessageWriter<MovementCommandEvent>,
    mut query: Query<(&mut MovementController, &Transform), With<Unit>>,
) {
    for event in ai_command_events.read() {
        match &event.command {
            AICommand::MoveTo(position) => {
                if let Ok((mut controller, _transform)) = query.get_mut(event.entity) {
                    controller.target_position = Some(*position);
                    controller.is_moving = true;

                    // Send physics movement command
                    movement_events.write(MovementCommandEvent {
                        entity: event.entity,
                        command: MovementCommand::MoveTo {
                            position: *position,
                            speed: 5.0,
                        },
                    });
                }
            }
            AICommand::Follow(target) => {
                // Follow logic would go here
                let target_pos = if let Ok((_, target_transform)) = query.get(*target) {
                    Some(target_transform.translation)
                } else {
                    None
                };

                if let Some(pos) = target_pos {
                    if let Ok((mut controller, _)) = query.get_mut(event.entity) {
                        controller.target_position = Some(pos);
                        controller.is_moving = true;
                    }
                }
            }
            AICommand::Patrol(waypoints) => {
                if let Ok((mut controller, _)) = query.get_mut(event.entity) {
                    controller.waypoints = waypoints.clone();
                    controller.path_index = 0;
                    controller.is_moving = true;
                }
            }
            _ => {}
        }
    }
}

// AI Combat System - handles combat decisions and actions for AI units
pub fn ai_combat_system(
    mut commands: Commands,
    mut ai_command_events: MessageReader<AICommandEvent>,
    mut query: Query<(&Transform, &Team, &Unit)>,
    enemy_query: Query<(Entity, &Transform, &Team), With<Unit>>,
) {
    for event in ai_command_events.read() {
        if let AICommand::Attack(target) = event.command {
            if let Ok((transform, team, _unit)) = query.get(event.entity) {
                // Find and engage target
                if let Ok((_, target_transform, target_team)) = enemy_query.get(target) {
                    if team.id != target_team.id {
                        // Combat logic would go here
                        let distance = transform.translation.distance(target_transform.translation);
                        if distance < 10.0 {
                            // In range, perform attack
                            #[cfg(feature = "web")]
                            web_sys::console::log_1(
                                &format!("AI unit attacking target at distance: {}", distance)
                                    .into(),
                            );
                        }
                    }
                }
            }
        }
    }
}

// Perception System - handles what AI entities can perceive
pub fn perception_system(
    mut perception_events: MessageWriter<AIPerceptionEvent>,
    query: Query<(Entity, &Transform, &Team), With<Unit>>,
) {
    // Simple perception - units detect enemies within range
    for (entity, transform, team) in query.iter() {
        for (other_entity, other_transform, other_team) in query.iter() {
            if entity == other_entity {
                continue;
            }

            let distance = transform.translation.distance(other_transform.translation);

            // Perception range
            if distance < 20.0 {
                if team.id != other_team.id {
                    // Enemy spotted
                    perception_events.write(AIPerceptionEvent {
                        perceiver: entity,
                        perceived: other_entity,
                        perception_type: PerceptionType::EnemySpotted,
                    });
                } else if distance < 5.0 {
                    // Close ally detected
                    perception_events.write(AIPerceptionEvent {
                        perceiver: entity,
                        perceived: other_entity,
                        perception_type: PerceptionType::AllyInDanger,
                    });
                }
            }
        }
    }
}

// Squad Coordination System - coordinates units in squads
pub fn squad_coordination_system(
    mut commands: Commands,
    leader_query: Query<(Entity, &Transform, &Team), With<Leader>>,
    mut follower_query: Query<
        (&mut MovementController, &Transform, &Team),
        (With<Unit>, Without<Leader>),
    >,
    mut movement_events: MessageWriter<MovementCommandEvent>,
) {
    for (leader_entity, leader_transform, leader_team) in leader_query.iter() {
        let leader_pos = leader_transform.translation;

        // Find nearby units of the same team
        let mut squad_members = Vec::new();
        for (controller, transform, team) in follower_query.iter_mut() {
            if team.id == leader_team.id {
                let distance = transform.translation.distance(leader_pos);
                if distance < 15.0 && distance > 3.0 {
                    squad_members.push(transform.translation);
                }
            }
        }

        // Coordinate squad movement
        if !squad_members.is_empty() {
            // Form up around leader
            let formation_radius = 5.0;
            let angle_step = std::f32::consts::TAU / squad_members.len() as f32;

            // Use a separate index for squad members to calculate correct formation angles
            let mut squad_member_index = 0;
            for (mut controller, transform, team) in follower_query.iter_mut() {
                if team.id == leader_team.id {
                    let distance = transform.translation.distance(leader_pos);
                    if distance < 15.0 && distance > 3.0 {
                        let angle = squad_member_index as f32 * angle_step;
                        let offset = Vec3::new(
                            angle.cos() * formation_radius,
                            0.0,
                            angle.sin() * formation_radius,
                        );
                        let target_pos = leader_pos + offset;

                        controller.target_position = Some(target_pos);
                        controller.is_moving = true;
                        squad_member_index += 1;
                    }
                }
            }
        }
    }
}
impl bevy::prelude::Message for AICommandEvent {}
impl bevy::prelude::Message for AIPerceptionEvent {}
