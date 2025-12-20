use bevy::prelude::*;
use std::collections::HashMap;

// Utility AI component
#[derive(Component, Clone, Debug)]
pub struct UtilityAI {
    pub considerations: Vec<Consideration>,
    pub actions: Vec<UtilityAction>,
    pub current_action: Option<usize>,
    pub update_interval: f32,
    pub last_update: f32,
}

// Consideration for scoring
#[derive(Clone, Debug)]
pub struct Consideration {
    pub name: String,
    pub input_type: InputType,
    pub curve: ResponseCurve,
    pub weight: f32,
}

#[derive(Clone, Debug)]
pub enum InputType {
    Health,
    Resources,
    EnemyDistance,
    AlliedUnits,
    TimeElapsed,
    Custom(String),
}

pub enum ResponseCurve {
    Linear,
    Quadratic,
    Exponential,
    Logarithmic,
    Sigmoid,
    Custom(Box<dyn Fn(f32) -> f32 + Send + Sync>),
}

impl std::fmt::Debug for ResponseCurve {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Linear => write!(f, "Linear"),
            Self::Quadratic => write!(f, "Quadratic"),
            Self::Exponential => write!(f, "Exponential"),
            Self::Logarithmic => write!(f, "Logarithmic"),
            Self::Sigmoid => write!(f, "Sigmoid"),
            Self::Custom(_) => write!(f, "Custom(function)"),
        }
    }
}

impl Clone for ResponseCurve {
    fn clone(&self) -> Self {
        match self {
            Self::Linear => Self::Linear,
            Self::Quadratic => Self::Quadratic,
            Self::Exponential => Self::Exponential,
            Self::Logarithmic => Self::Logarithmic,
            Self::Sigmoid => Self::Sigmoid,
            Self::Custom(_) => Self::Linear, // Fallback to Linear for custom functions
        }
    }
}

impl ResponseCurve {
    pub fn evaluate(&self, input: f32) -> f32 {
        let normalized = input.clamp(0.0, 1.0);

        match self {
            ResponseCurve::Linear => normalized,
            ResponseCurve::Quadratic => normalized * normalized,
            ResponseCurve::Exponential => normalized.exp() / std::f32::consts::E,
            ResponseCurve::Logarithmic => (normalized + 1.0).ln() / 2_f32.ln(),
            ResponseCurve::Sigmoid => {
                let k = 10.0; // Steepness
                1.0 / (1.0 + (-k * (normalized - 0.5)).exp())
            },
            ResponseCurve::Custom(func) => func(normalized),
        }
    }
}

// Utility action
#[derive(Clone, Debug)]
pub struct UtilityAction {
    pub name: String,
    pub action_type: UtilityActionType,
    pub considerations: Vec<usize>, // Indices into consideration list
    pub base_score: f32,
}

#[derive(Clone, Debug)]
pub enum UtilityActionType {
    Attack,
    Defend,
    Gather,
    Build,
    Explore,
    Retreat,
    Trade,
    Research,
    Custom(String),
}

// Utility score
#[derive(Clone, Debug)]
pub struct UtilityScore {
    pub action_index: usize,
    pub score: f32,
    pub consideration_scores: Vec<f32>,
}

impl UtilityAI {
    pub fn new() -> Self {
        Self {
            considerations: Vec::new(),
            actions: Vec::new(),
            current_action: None,
            update_interval: 1.0,
            last_update: 0.0,
        }
    }

    pub fn add_consideration(&mut self, consideration: Consideration) -> usize {
        self.considerations.push(consideration);
        self.considerations.len() - 1
    }

    pub fn add_action(&mut self, action: UtilityAction) {
        self.actions.push(action);
    }

    pub fn evaluate_actions(&mut self, context: &UtilityContext) -> Vec<UtilityScore> {
        let mut scores = Vec::new();

        for (index, action) in self.actions.iter().enumerate() {
            let mut total_score = action.base_score;
            let mut consideration_scores = Vec::new();

            // Evaluate each consideration for this action
            for &consideration_index in &action.considerations {
                if let Some(consideration) = self.considerations.get(consideration_index) {
                    let input = get_input_value(&consideration.input_type, context);
                    let score = consideration.curve.evaluate(input) * consideration.weight;
                    consideration_scores.push(score);
                    total_score *= score; // Multiplicative scoring
                }
            }

            // Apply compensation factor to prevent too many considerations from zeroing the score
            let modification_factor = 1.0 - (1.0 / action.considerations.len() as f32);
            let makeup_value = (1.0 - total_score) * modification_factor;
            total_score = total_score + (makeup_value * total_score);

            scores.push(UtilityScore {
                action_index: index,
                score: total_score,
                consideration_scores,
            });
        }

        // Sort by score (highest first)
        scores.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());
        scores
    }

    pub fn select_action(&mut self, context: &UtilityContext) -> Option<&UtilityAction> {
        let scores = self.evaluate_actions(context);

        if let Some(best) = scores.first() {
            if best.score > 0.0 {
                self.current_action = Some(best.action_index);
                return self.actions.get(best.action_index);
            }
        }

        None
    }
}

// Context for utility evaluation
#[derive(Clone, Debug)]
pub struct UtilityContext {
    pub health_percentage: f32,
    pub resource_amount: f32,
    pub enemy_distance: f32,
    pub allied_unit_count: f32,
    pub time_elapsed: f32,
    pub custom_values: HashMap<String, f32>,
}

impl Default for UtilityContext {
    fn default() -> Self {
        Self {
            health_percentage: 1.0,
            resource_amount: 0.0,
            enemy_distance: 100.0,
            allied_unit_count: 0.0,
            time_elapsed: 0.0,
            custom_values: HashMap::new(),
        }
    }
}

fn get_input_value(input_type: &InputType, context: &UtilityContext) -> f32 {
    match input_type {
        InputType::Health => context.health_percentage,
        InputType::Resources => (context.resource_amount / 1000.0).min(1.0),
        InputType::EnemyDistance => (1.0 - (context.enemy_distance / 100.0)).max(0.0),
        InputType::AlliedUnits => (context.allied_unit_count / 10.0).min(1.0),
        InputType::TimeElapsed => (context.time_elapsed / 300.0).min(1.0), // 5 minutes
        InputType::Custom(key) => *context.custom_values.get(key).unwrap_or(&0.0),
    }
}

// Preset AI personalities using utility AI
pub fn create_aggressive_ai() -> UtilityAI {
    let mut ai = UtilityAI::new();

    // Considerations
    let health_consideration = ai.add_consideration(Consideration {
        name: "Health".to_string(),
        input_type: InputType::Health,
        curve: ResponseCurve::Linear,
        weight: 0.8,
    });

    let enemy_distance = ai.add_consideration(Consideration {
        name: "Enemy Distance".to_string(),
        input_type: InputType::EnemyDistance,
        curve: ResponseCurve::Exponential,
        weight: 1.2,
    });

    // Actions
    ai.add_action(UtilityAction {
        name: "Attack".to_string(),
        action_type: UtilityActionType::Attack,
        considerations: vec![health_consideration, enemy_distance],
        base_score: 0.8,
    });

    ai.add_action(UtilityAction {
        name: "Retreat".to_string(),
        action_type: UtilityActionType::Retreat,
        considerations: vec![health_consideration],
        base_score: 0.3,
    });

    ai
}

pub fn create_economic_ai() -> UtilityAI {
    let mut ai = UtilityAI::new();

    // Considerations
    let resources = ai.add_consideration(Consideration {
        name: "Resources".to_string(),
        input_type: InputType::Resources,
        curve: ResponseCurve::Logarithmic,
        weight: 1.5,
    });

    let time = ai.add_consideration(Consideration {
        name: "Time".to_string(),
        input_type: InputType::TimeElapsed,
        curve: ResponseCurve::Linear,
        weight: 0.5,
    });

    // Actions
    ai.add_action(UtilityAction {
        name: "Gather".to_string(),
        action_type: UtilityActionType::Gather,
        considerations: vec![resources, time],
        base_score: 0.9,
    });

    ai.add_action(UtilityAction {
        name: "Build".to_string(),
        action_type: UtilityActionType::Build,
        considerations: vec![resources],
        base_score: 0.6,
    });

    ai.add_action(UtilityAction {
        name: "Trade".to_string(),
        action_type: UtilityActionType::Trade,
        considerations: vec![resources],
        base_score: 0.4,
    });

    ai
}

// Utility AI system that processes UtilityAI components
pub fn utility_ai_system(
    time: Res<Time>,
    mut query: Query<(Entity, &mut UtilityAI, &Transform), With<UtilityAI>>,
    mut commands: Commands,
) {
    let current_time = time.elapsed_secs();

    for (entity, mut utility_ai, transform) in query.iter_mut() {
        // Check if it's time to update this AI
        if current_time - utility_ai.last_update < utility_ai.update_interval {
            continue;
        }

        utility_ai.last_update = current_time;

        // Create utility context from current state
        let context = UtilityContext {
            health_percentage: 1.0, // Would get from health component
            resource_amount: 100.0, // Would get from resource component
            enemy_distance: 50.0, // Would get from nearby enemy detection
            allied_unit_count: 1.0, // Would count nearby allied units
            time_elapsed: current_time,
            custom_values: std::collections::HashMap::new(),
        };

        // Select best action based on utility scoring
        if let Some(selected_action) = utility_ai.select_action(&context) {
            // Convert utility action to behavior component
            match &selected_action.action_type {
                UtilityActionType::Attack => {
                    commands.entity(entity).insert(crate::AttackBehavior {
                        target: None, // Would need target detection
                        aggression_level: 0.8,
                    });
                },
                UtilityActionType::Defend => {
                    commands.entity(entity).insert(crate::DefendBehavior {
                        defend_position: transform.translation,
                        patrol_radius: 10.0,
                    });
                },
                UtilityActionType::Gather => {
                    commands.entity(entity).insert(crate::GatheringBehavior {
                        target_resource: None, // Would need resource detection
                        gathering_rate: 1.0,
                    });
                },
                UtilityActionType::Build => {
                    // Add building behavior when we have building system
                },
                UtilityActionType::Explore => {
                    // Add exploration behavior
                },
                UtilityActionType::Retreat => {
                    commands.entity(entity).insert(crate::RetreatBehavior {
                        safe_position: Some(transform.translation + Vec3::new(-20.0, 0.0, -20.0)),
                        retreat_threshold: 0.3,
                    });
                },
                UtilityActionType::Trade => {
                    // Add trade behavior when we have trade system
                },
                UtilityActionType::Research => {
                    // Add research behavior when we have research system
                },
                UtilityActionType::Custom(_) => {
                    // Handle custom action types
                },
            }
        }
    }
}