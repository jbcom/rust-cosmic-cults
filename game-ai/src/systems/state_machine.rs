use bevy::prelude::*;
use std::collections::HashMap;

// AI State Machine component
#[derive(Component, Clone, Debug)]
pub struct AIStateMachine {
    pub current_state: AIState,
    pub previous_state: Option<AIState>,
    pub state_transitions: HashMap<(AIState, AITransition), AIState>,
    pub state_timers: HashMap<AIState, f32>,
    pub global_timer: f32,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum AIState {
    Idle,
    Gathering,
    Building,
    Attacking,
    Defending,
    Retreating,
    Expanding,
    Scouting,
    Researching,
    Trading,
    Custom(String),
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum AITransition {
    ResourcesLow,
    ResourcesHigh,
    UnderAttack,
    EnemySpotted,
    EnemyDefeated,
    BuildingComplete,
    ResearchComplete,
    HealthLow,
    HealthHigh,
    GoalAchieved,
    Timeout,
    Custom(String),
}

impl Default for AIStateMachine {
    fn default() -> Self {
        let mut transitions = HashMap::new();

        // Define default state transitions
        transitions.insert(
            (AIState::Idle, AITransition::ResourcesLow),
            AIState::Gathering,
        );
        transitions.insert(
            (AIState::Idle, AITransition::UnderAttack),
            AIState::Defending,
        );
        transitions.insert(
            (AIState::Idle, AITransition::EnemySpotted),
            AIState::Attacking,
        );

        transitions.insert(
            (AIState::Gathering, AITransition::ResourcesHigh),
            AIState::Building,
        );
        transitions.insert(
            (AIState::Gathering, AITransition::UnderAttack),
            AIState::Defending,
        );

        transitions.insert(
            (AIState::Building, AITransition::BuildingComplete),
            AIState::Idle,
        );
        transitions.insert(
            (AIState::Building, AITransition::UnderAttack),
            AIState::Defending,
        );

        transitions.insert(
            (AIState::Attacking, AITransition::HealthLow),
            AIState::Retreating,
        );
        transitions.insert(
            (AIState::Attacking, AITransition::EnemyDefeated),
            AIState::Idle,
        );

        transitions.insert(
            (AIState::Defending, AITransition::EnemyDefeated),
            AIState::Idle,
        );
        transitions.insert(
            (AIState::Defending, AITransition::HealthLow),
            AIState::Retreating,
        );

        transitions.insert(
            (AIState::Retreating, AITransition::HealthHigh),
            AIState::Idle,
        );

        Self {
            current_state: AIState::Idle,
            previous_state: None,
            state_transitions: transitions,
            state_timers: HashMap::new(),
            global_timer: 0.0,
        }
    }
}

impl AIStateMachine {
    pub fn transition(&mut self, trigger: AITransition) -> bool {
        let key = (self.current_state.clone(), trigger);

        if let Some(new_state) = self.state_transitions.get(&key).cloned() {
            self.previous_state = Some(self.current_state.clone());
            self.current_state = new_state.clone();

            // Reset state timer
            self.state_timers.insert(new_state, 0.0);

            return true;
        }

        false
    }

    pub fn update(&mut self, delta_time: f32) {
        self.global_timer += delta_time;

        // Update current state timer
        if let Some(timer) = self.state_timers.get_mut(&self.current_state) {
            *timer += delta_time;
        } else {
            self.state_timers
                .insert(self.current_state.clone(), delta_time);
        }
    }

    pub fn get_state_duration(&self) -> f32 {
        self.state_timers
            .get(&self.current_state)
            .copied()
            .unwrap_or(0.0)
    }

    pub fn force_state(&mut self, new_state: AIState) {
        self.previous_state = Some(self.current_state.clone());
        self.current_state = new_state.clone();
        self.state_timers.insert(new_state, 0.0);
    }

    pub fn add_transition(&mut self, from: AIState, trigger: AITransition, to: AIState) {
        self.state_transitions.insert((from, trigger), to);
    }

    pub fn can_transition(&self, trigger: &AITransition) -> bool {
        self.state_transitions
            .contains_key(&(self.current_state.clone(), trigger.clone()))
    }
}

// Hierarchical State Machine
#[derive(Component, Clone, Debug)]
pub struct HierarchicalStateMachine {
    pub root_state: AIState,
    pub sub_states: HashMap<AIState, AIStateMachine>,
    pub active_sub_state: Option<AIState>,
}

impl Default for HierarchicalStateMachine {
    fn default() -> Self {
        Self::new()
    }
}

impl HierarchicalStateMachine {
    pub fn new() -> Self {
        Self {
            root_state: AIState::Idle,
            sub_states: HashMap::new(),
            active_sub_state: None,
        }
    }

    pub fn add_sub_state_machine(&mut self, parent: AIState, sub_machine: AIStateMachine) {
        self.sub_states.insert(parent, sub_machine);
    }

    pub fn transition_root(&mut self, new_state: AIState) {
        self.root_state = new_state.clone();

        // Activate sub-state if available
        if self.sub_states.contains_key(&new_state) {
            self.active_sub_state = Some(new_state);
        } else {
            self.active_sub_state = None;
        }
    }

    pub fn transition_sub(&mut self, trigger: AITransition) -> bool {
        if let Some(active) = &self.active_sub_state
            && let Some(sub_machine) = self.sub_states.get_mut(active)
        {
            return sub_machine.transition(trigger);
        }
        false
    }

    pub fn update(&mut self, delta_time: f32) {
        if let Some(active) = &self.active_sub_state
            && let Some(sub_machine) = self.sub_states.get_mut(active)
        {
            sub_machine.update(delta_time);
        }
    }
}

// State machine system
pub fn state_machine_update_system(time: Res<Time>, mut query: Query<&mut AIStateMachine>) {
    let delta = time.delta_secs();

    for mut state_machine in query.iter_mut() {
        state_machine.update(delta);

        // Check for timeout transitions
        if state_machine.get_state_duration() > 30.0 {
            state_machine.transition(AITransition::Timeout);
        }
    }
}

// State-specific behavior execution
pub fn execute_state_behavior(state: &AIState, entity: Entity, commands: &mut Commands) {
    match state {
        AIState::Idle => {
            // Do nothing special
        }
        AIState::Gathering => {
            // Move to resource and collect
            commands.entity(entity).insert(GatheringBehavior {
                target_resource: None,
                gathering_rate: 1.0,
            });
        }
        AIState::Building => {
            // Construct buildings
            commands.entity(entity).insert(BuildingBehavior {
                building_type: None,
                progress: 0.0,
            });
        }
        AIState::Attacking => {
            // Engage enemies
            commands.entity(entity).insert(AttackBehavior {
                target: None,
                aggression_level: 1.0,
            });
        }
        AIState::Defending => {
            // Protect area
            commands.entity(entity).insert(DefendBehavior {
                defend_position: Vec3::ZERO,
                patrol_radius: 10.0,
            });
        }
        AIState::Retreating => {
            // Move to safety
            commands.entity(entity).insert(RetreatBehavior {
                safe_position: None,
                retreat_threshold: 0.3,
            });
        }
        _ => {}
    }
}

// Behavior marker components
#[derive(Component)]
pub struct GatheringBehavior {
    pub target_resource: Option<Entity>,
    pub gathering_rate: f32,
}

#[derive(Component)]
pub struct BuildingBehavior {
    pub building_type: Option<String>,
    pub progress: f32,
}

#[derive(Component)]
pub struct AttackBehavior {
    pub target: Option<Entity>,
    pub aggression_level: f32,
}

#[derive(Component)]
pub struct DefendBehavior {
    pub defend_position: Vec3,
    pub patrol_radius: f32,
}

#[derive(Component)]
pub struct RetreatBehavior {
    pub safe_position: Option<Vec3>,
    pub retreat_threshold: f32,
}
