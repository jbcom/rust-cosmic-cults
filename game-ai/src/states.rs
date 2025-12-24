// AI State Machine - Production-ready state management for AI entities
use bevy::prelude::*;
use game_physics::prelude::*;
use game_units::{Leader, Team, Unit};
use std::collections::HashMap;

// Core AI state enum - defines all possible states an AI unit can be in
#[derive(Component, Clone, Debug, PartialEq, Eq, Hash)]
pub enum AIState {
    Idle,
    Patrolling,
    Attacking,
    Fleeing,
    Following,
    Gathering,
    Defending,
    Building,
    Retreating,
    Searching,
}

// State machine component that manages AI state transitions
#[derive(Component, Clone, Debug)]
pub struct AIStateMachine {
    pub current_state: AIState,
    pub previous_state: Option<AIState>,
    pub state_timer: f32,
    pub transition_rules: HashMap<(AIState, StateTransitionTrigger), AIState>,
    pub state_data: StateData,
}

// Data associated with current state
#[derive(Clone, Debug)]
pub struct StateData {
    pub target_entity: Option<Entity>,
    pub target_position: Option<Vec3>,
    pub patrol_points: Vec<Vec3>,
    pub current_patrol_index: usize,
    pub home_position: Vec3,
    pub alert_level: f32,
    pub last_enemy_position: Option<Vec3>,
}

impl Default for StateData {
    fn default() -> Self {
        Self {
            target_entity: None,
            target_position: None,
            patrol_points: Vec::new(),
            current_patrol_index: 0,
            home_position: Vec3::ZERO,
            alert_level: 0.0,
            last_enemy_position: None,
        }
    }
}

// Triggers that cause state transitions
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum StateTransitionTrigger {
    EnemyDetected,
    EnemyLost,
    HealthLow,
    HealthRestored,
    TargetDestroyed,
    ResourceFound,
    ResourceGathered,
    OrderReceived,
    PathBlocked,
    TimerExpired,
    AlertRaised,
    AllClear,
}

impl Default for AIStateMachine {
    fn default() -> Self {
        let mut rules = HashMap::new();

        // Define default state transition rules
        // From Idle
        rules.insert(
            (AIState::Idle, StateTransitionTrigger::EnemyDetected),
            AIState::Attacking,
        );
        rules.insert(
            (AIState::Idle, StateTransitionTrigger::OrderReceived),
            AIState::Following,
        );
        rules.insert(
            (AIState::Idle, StateTransitionTrigger::ResourceFound),
            AIState::Gathering,
        );
        rules.insert(
            (AIState::Idle, StateTransitionTrigger::TimerExpired),
            AIState::Patrolling,
        );

        // From Patrolling
        rules.insert(
            (AIState::Patrolling, StateTransitionTrigger::EnemyDetected),
            AIState::Attacking,
        );
        rules.insert(
            (AIState::Patrolling, StateTransitionTrigger::AlertRaised),
            AIState::Searching,
        );
        rules.insert(
            (AIState::Patrolling, StateTransitionTrigger::OrderReceived),
            AIState::Following,
        );

        // From Attacking
        rules.insert(
            (AIState::Attacking, StateTransitionTrigger::HealthLow),
            AIState::Fleeing,
        );
        rules.insert(
            (AIState::Attacking, StateTransitionTrigger::TargetDestroyed),
            AIState::Searching,
        );
        rules.insert(
            (AIState::Attacking, StateTransitionTrigger::EnemyLost),
            AIState::Searching,
        );

        // From Fleeing
        rules.insert(
            (AIState::Fleeing, StateTransitionTrigger::HealthRestored),
            AIState::Idle,
        );
        rules.insert(
            (AIState::Fleeing, StateTransitionTrigger::EnemyDetected),
            AIState::Retreating,
        );

        // From Following
        rules.insert(
            (AIState::Following, StateTransitionTrigger::EnemyDetected),
            AIState::Attacking,
        );
        rules.insert(
            (AIState::Following, StateTransitionTrigger::OrderReceived),
            AIState::Idle,
        );

        // From Gathering
        rules.insert(
            (AIState::Gathering, StateTransitionTrigger::ResourceGathered),
            AIState::Idle,
        );
        rules.insert(
            (AIState::Gathering, StateTransitionTrigger::EnemyDetected),
            AIState::Defending,
        );

        // From Searching
        rules.insert(
            (AIState::Searching, StateTransitionTrigger::EnemyDetected),
            AIState::Attacking,
        );
        rules.insert(
            (AIState::Searching, StateTransitionTrigger::TimerExpired),
            AIState::Patrolling,
        );
        rules.insert(
            (AIState::Searching, StateTransitionTrigger::AllClear),
            AIState::Idle,
        );

        Self {
            current_state: AIState::Idle,
            previous_state: None,
            state_timer: 0.0,
            transition_rules: rules,
            state_data: StateData::default(),
        }
    }
}

impl AIStateMachine {
    pub fn transition(&mut self, trigger: StateTransitionTrigger) -> bool {
        let key = (self.current_state.clone(), trigger);

        if let Some(new_state) = self.transition_rules.get(&key).cloned() {
            self.previous_state = Some(self.current_state.clone());
            self.current_state = new_state;
            self.state_timer = 0.0;
            return true;
        }

        false
    }

    pub fn force_state(&mut self, new_state: AIState) {
        self.previous_state = Some(self.current_state.clone());
        self.current_state = new_state;
        self.state_timer = 0.0;
    }

    pub fn update(&mut self, delta_time: f32) {
        self.state_timer += delta_time;

        // Auto-transition based on timers
        match self.current_state {
            AIState::Idle => {
                if self.state_timer > 5.0 {
                    self.transition(StateTransitionTrigger::TimerExpired);
                }
            }
            AIState::Searching => {
                if self.state_timer > 10.0 {
                    self.transition(StateTransitionTrigger::TimerExpired);
                }
            }
            _ => {}
        }
    }

    pub fn set_patrol_points(&mut self, points: Vec<Vec3>) {
        self.state_data.patrol_points = points;
        self.state_data.current_patrol_index = 0;
    }

    pub fn get_next_patrol_point(&mut self) -> Option<Vec3> {
        if self.state_data.patrol_points.is_empty() {
            return None;
        }

        let point = self.state_data.patrol_points[self.state_data.current_patrol_index];
        self.state_data.current_patrol_index =
            (self.state_data.current_patrol_index + 1) % self.state_data.patrol_points.len();

        Some(point)
    }
}

// State execution system - handles behavior for each state
#[allow(clippy::type_complexity)]
pub fn state_execution_system(
    mut query: Query<(
        Entity,
        &mut AIStateMachine,
        &Transform,
        Option<&Unit>,
        Option<&Team>,
    )>,
    mut movement_events: MessageWriter<MovementCommandEvent>,
    time: Res<Time>,
    mut commands: Commands,
) {
    let delta = time.delta_seconds();

    for (entity, mut state_machine, transform, unit, team) in query.iter_mut() {
        state_machine.update(delta);

        match state_machine.current_state {
            AIState::Idle => {
                execute_idle_state(entity, &state_machine, transform, &mut commands);
            }
            AIState::Patrolling => {
                execute_patrol_state(entity, &mut state_machine, transform, &mut movement_events);
            }
            AIState::Attacking => {
                execute_attack_state(entity, &state_machine, transform, unit, &mut commands);
            }
            AIState::Fleeing => {
                execute_flee_state(entity, &state_machine, transform, &mut movement_events);
            }
            AIState::Following => {
                execute_follow_state(entity, &state_machine, &mut movement_events);
            }
            AIState::Gathering => {
                execute_gather_state(entity, &state_machine, transform, &mut commands);
            }
            AIState::Searching => {
                execute_search_state(entity, &mut state_machine, transform, &mut movement_events);
            }
            _ => {}
        }
    }
}

// State-specific execution functions
fn execute_idle_state(
    entity: Entity,
    state_machine: &AIStateMachine,
    transform: &Transform,
    commands: &mut Commands,
) {
    // Idle units occasionally look around
    if state_machine.state_timer > 3.0 {
        // Add a small random rotation to simulate looking around
        let rotation = Quat::from_rotation_y(0.1);
        commands.entity(entity).insert(Transform {
            translation: transform.translation,
            rotation: transform.rotation * rotation,
            scale: transform.scale,
        });
    }
}

fn execute_patrol_state(
    entity: Entity,
    state_machine: &mut AIStateMachine,
    transform: &Transform,
    movement_events: &mut MessageWriter<MovementCommandEvent>,
) {
    // Move to next patrol point
    if state_machine.state_data.target_position.is_none() {
        if let Some(next_point) = state_machine.get_next_patrol_point() {
            state_machine.state_data.target_position = Some(next_point);

            movement_events.write(MovementCommandEvent {
                entity,
                command: MovementCommand::MoveTo {
                    position: next_point,
                    speed: 3.0,
                },
            });
        }
    } else if let Some(target) = state_machine.state_data.target_position {
        // Check if we've reached the patrol point
        let distance = transform.translation.distance(target);
        if distance < 2.0 {
            state_machine.state_data.target_position = None;
        }
    }
}

fn execute_attack_state(
    entity: Entity,
    state_machine: &AIStateMachine,
    transform: &Transform,
    unit: Option<&Unit>,
    commands: &mut Commands,
) {
    if let Some(target) = state_machine.state_data.target_entity {
        // Move towards target and attack
        commands
            .entity(entity)
            .insert(crate::game_behaviors::AttackBehavior {
                target: Some(target),
                aggression_level: unit.map(|u| u.attack_damage / 10.0).unwrap_or(1.0),
            });
    }
}

fn execute_flee_state(
    entity: Entity,
    state_machine: &AIStateMachine,
    transform: &Transform,
    movement_events: &mut MessageWriter<MovementCommandEvent>,
) {
    // Move away from danger towards home position
    let flee_direction = if let Some(enemy_pos) = state_machine.state_data.last_enemy_position {
        (transform.translation - enemy_pos).normalize()
    } else {
        (state_machine.state_data.home_position - transform.translation).normalize()
    };

    let flee_position = transform.translation + flee_direction * 20.0;

    movement_events.write(MovementCommandEvent {
        entity,
        command: MovementCommand::MoveTo {
            position: flee_position,
            speed: 6.0, // Fast retreat
        },
    });
}

fn execute_follow_state(
    entity: Entity,
    state_machine: &AIStateMachine,
    movement_events: &mut MessageWriter<MovementCommandEvent>,
) {
    if let Some(target) = state_machine.state_data.target_entity {
        movement_events.write(MovementCommandEvent {
            entity,
            command: MovementCommand::Follow {
                target,
                distance: 5.0,
            },
        });
    }
}

fn execute_gather_state(
    entity: Entity,
    state_machine: &AIStateMachine,
    transform: &Transform,
    commands: &mut Commands,
) {
    if let Some(resource) = state_machine.state_data.target_entity {
        commands
            .entity(entity)
            .insert(crate::game_behaviors::GatheringBehavior {
                target_resource: Some(resource),
                gathering_rate: 1.0,
            });
    }
}

fn execute_search_state(
    entity: Entity,
    state_machine: &mut AIStateMachine,
    transform: &Transform,
    movement_events: &mut MessageWriter<MovementCommandEvent>,
) {
    // Search in expanding circles from last known enemy position
    let search_center = state_machine
        .state_data
        .last_enemy_position
        .unwrap_or(transform.translation);

    let angle = state_machine.state_timer * 0.5;
    let radius = 5.0 + state_machine.state_timer * 2.0;

    let search_position =
        search_center + Vec3::new(angle.cos() * radius, 0.0, angle.sin() * radius);

    movement_events.write(MovementCommandEvent {
        entity,
        command: MovementCommand::MoveTo {
            position: search_position,
            speed: 4.0,
        },
    });
}

// System to trigger state transitions based on game events
pub fn state_transition_system(
    mut query: Query<(Entity, &mut AIStateMachine, &Transform, Option<&Unit>)>,
    enemy_query: Query<(Entity, &Transform, &Team), Without<AIStateMachine>>,
    time: Res<Time>,
) {
    for (entity, mut state_machine, transform, unit) in query.iter_mut() {
        // Check for enemies in detection range
        let detection_range = 15.0;
        let mut enemy_detected = false;
        let mut closest_enemy: Option<(Entity, f32)> = None;

        for (enemy_entity, enemy_transform, enemy_team) in enemy_query.iter() {
            // Skip if same team (would need proper team checking)
            let distance = transform.translation.distance(enemy_transform.translation);

            if distance < detection_range {
                enemy_detected = true;

                if closest_enemy.is_none() || distance < closest_enemy.unwrap().1 {
                    closest_enemy = Some((enemy_entity, distance));
                }
            }
        }

        // Trigger state transitions based on conditions
        if enemy_detected
            && state_machine.current_state != AIState::Attacking
            && let Some((enemy, _)) = closest_enemy
        {
            state_machine.state_data.target_entity = Some(enemy);
            state_machine.transition(StateTransitionTrigger::EnemyDetected);
        }

        // Check health for flee trigger
        if let Some(unit) = unit {
            if unit.health < unit.max_health * 0.3
                && state_machine.current_state == AIState::Attacking
            {
                state_machine.transition(StateTransitionTrigger::HealthLow);
            } else if unit.health > unit.max_health * 0.5
                && state_machine.current_state == AIState::Fleeing
            {
                state_machine.transition(StateTransitionTrigger::HealthRestored);
            }
        }

        // Check if target was destroyed
        if state_machine.current_state == AIState::Attacking
            && let Some(target) = state_machine.state_data.target_entity
        {
            // Would need to check if target entity still exists
            // For now, assume target lost after some time
            if state_machine.state_timer > 5.0 {
                state_machine.transition(StateTransitionTrigger::EnemyLost);
            }
        }
    }
}
