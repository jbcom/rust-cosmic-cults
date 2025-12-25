// Cult-specific AI profiles and behaviors
use crate::ai::types::{AICoordination, AIRole};
use bevy::prelude::*;
use bevy_ai_toolkit::prelude::*; // Use toolkit types
use std::collections::HashMap;

/// Cult-specific AI behavioral modifiers
#[derive(Component, Clone, Debug)]
pub struct CultProfile {
    pub cult_name: String,
    pub behavioral_modifiers: CultBehaviorModifiers,
    pub preferred_actions: Vec<String>,
    pub fear_threshold: f32,
    pub aggression_level: f32,
    pub thirst_drive: f32,
}

#[derive(Clone, Debug)]
pub struct CultBehaviorModifiers {
    pub attack_bonus: f32,
    pub defense_bonus: f32,
    pub gathering_bonus: f32,
    pub building_bonus: f32,
    pub research_bonus: f32,
    pub expansion_bonus: f32,
}

impl Default for CultBehaviorModifiers {
    fn default() -> Self {
        Self {
            attack_bonus: 1.0,
            defense_bonus: 1.0,
            gathering_bonus: 1.0,
            building_bonus: 1.0,
            research_bonus: 1.0,
            expansion_bonus: 1.0,
        }
    }
}

/// Advanced AI scorers for psychological factors
#[derive(Component, Clone, Debug)]
pub struct PsychologicalState {
    pub thirst_level: f32,         // 0.0 to 1.0 - drives resource acquisition
    pub fear_level: f32,           // 0.0 to 1.0 - affects retreat behavior
    pub aggression_level: f32,     // 0.0 to 1.0 - affects attack behavior
    pub corruption_influence: f32, // 0.0 to 1.0 - cult-specific influence
}

impl Default for PsychologicalState {
    fn default() -> Self {
        Self {
            thirst_level: 0.3,
            fear_level: 0.2,
            aggression_level: 0.5,
            corruption_influence: 0.0,
        }
    }
}

/// Creates cult-specific AI profiles
pub fn create_cult_profile(cult_name: &str) -> CultProfile {
    match cult_name {
        "Order of the Deep" => create_order_of_the_deep_profile(),
        "Crimson Covenant" => create_crimson_covenant_profile(),
        "Void Seekers" => create_void_seekers_profile(),
        _ => create_default_cult_profile(cult_name),
    }
}

/// Order of the Deep - Knowledge-focused, defensive cult
fn create_order_of_the_deep_profile() -> CultProfile {
    CultProfile {
        cult_name: "Order of the Deep".to_string(),
        behavioral_modifiers: CultBehaviorModifiers {
            attack_bonus: 0.8,    // Less aggressive
            defense_bonus: 1.3,   // Strong defense
            gathering_bonus: 1.2, // Good at resource gathering
            building_bonus: 1.4,  // Excellent builders
            research_bonus: 1.6,  // Superior research
            expansion_bonus: 0.9, // Cautious expansion
        },
        preferred_actions: vec![
            "research".to_string(),
            "build_sanctuary".to_string(),
            "gather_knowledge".to_string(),
            "defend_territory".to_string(),
        ],
        fear_threshold: 0.3,   // More cautious
        aggression_level: 0.4, // Low aggression
        thirst_drive: 0.8,     // High knowledge thirst
    }
}

/// Crimson Covenant - Blood-focused, aggressive cult
fn create_crimson_covenant_profile() -> CultProfile {
    CultProfile {
        cult_name: "Crimson Covenant".to_string(),
        behavioral_modifiers: CultBehaviorModifiers {
            attack_bonus: 1.5,    // Highly aggressive
            defense_bonus: 1.1,   // Moderate defense
            gathering_bonus: 1.0, // Normal gathering
            building_bonus: 0.9,  // Less building focus
            research_bonus: 0.8,  // Less research
            expansion_bonus: 1.3, // Aggressive expansion
        },
        preferred_actions: vec![
            "attack_enemies".to_string(),
            "blood_ritual".to_string(),
            "expand_territory".to_string(),
            "hunt_souls".to_string(),
        ],
        fear_threshold: 0.6,   // Less fearful
        aggression_level: 0.8, // High aggression
        thirst_drive: 0.9,     // High blood thirst
    }
}

/// Void Seekers - Corruption-focused, mystical cult
fn create_void_seekers_profile() -> CultProfile {
    CultProfile {
        cult_name: "Void Seekers".to_string(),
        behavioral_modifiers: CultBehaviorModifiers {
            attack_bonus: 1.1,    // Moderate attack
            defense_bonus: 0.9,   // Weaker defense
            gathering_bonus: 0.8, // Less conventional gathering
            building_bonus: 1.2,  // Good at mystical structures
            research_bonus: 1.4,  // Strong research focus
            expansion_bonus: 1.1, // Moderate expansion
        },
        preferred_actions: vec![
            "corrupt_land".to_string(),
            "void_ritual".to_string(),
            "summon_entities".to_string(),
            "spread_influence".to_string(),
        ],
        fear_threshold: 0.4,   // Moderate fear
        aggression_level: 0.6, // Moderate aggression
        thirst_drive: 0.7,     // High corruption thirst
    }
}

fn create_default_cult_profile(cult_name: &str) -> CultProfile {
    CultProfile {
        cult_name: cult_name.to_string(),
        behavioral_modifiers: CultBehaviorModifiers::default(),
        preferred_actions: vec![
            "build_units".to_string(),
            "gather_resources".to_string(),
            "expand".to_string(),
        ],
        fear_threshold: 0.5,
        aggression_level: 0.5,
        thirst_drive: 0.5,
    }
}

/// Creates a utility AI configured for specific cult behavior
pub fn create_cult_ai(cult_profile: &CultProfile) -> UtilityAI {
    let mut ai = UtilityAI::new();

    // Add psychological considerations
    let thirst_consideration = ai.add_consideration(Consideration {
        name: "Thirst Drive".to_string(),
        input_type: InputType::Custom("thirst_level".to_string()),
        curve: ResponseCurve::Exponential,
        weight: cult_profile.thirst_drive,
    });

    let fear_consideration = ai.add_consideration(Consideration {
        name: "Fear Level".to_string(),
        input_type: InputType::Custom("fear_level".to_string()),
        curve: ResponseCurve::Sigmoid,
        weight: 1.0 - cult_profile.fear_threshold,
    });

    let aggression_consideration = ai.add_consideration(Consideration {
        name: "Aggression".to_string(),
        input_type: InputType::Custom("aggression_level".to_string()),
        curve: ResponseCurve::Linear,
        weight: cult_profile.aggression_level,
    });

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
        weight: 0.9,
    });

    let resources = ai.add_consideration(Consideration {
        name: "Resources".to_string(),
        input_type: InputType::Resources,
        curve: ResponseCurve::Logarithmic,
        weight: 1.0,
    });

    // Add cult-specific actions based on profile
    add_cult_specific_actions(
        &mut ai,
        cult_profile,
        &[
            thirst_consideration,
            fear_consideration,
            aggression_consideration,
            health_consideration,
            enemy_distance,
            resources,
        ],
    );

    ai
}

fn add_cult_specific_actions(ai: &mut UtilityAI, profile: &CultProfile, considerations: &[usize]) {
    let modifiers = &profile.behavioral_modifiers;

    // Attack action - modified by cult aggression
    ai.add_action(UtilityAction {
        name: "Attack".to_string(),
        action_type: UtilityActionType::Attack,
        considerations: vec![considerations[2], considerations[3], considerations[4]], // aggression, health, enemy_distance
        base_score: 0.6 * modifiers.attack_bonus,
    });

    // Defend action - modified by cult defense bonus
    ai.add_action(UtilityAction {
        name: "Defend".to_string(),
        action_type: UtilityActionType::Defend,
        considerations: vec![considerations[1], considerations[3]], // fear, health
        base_score: 0.5 * modifiers.defense_bonus,
    });

    // Gather action - modified by thirst and gathering bonus
    ai.add_action(UtilityAction {
        name: "Gather".to_string(),
        action_type: UtilityActionType::Gather,
        considerations: vec![considerations[0], considerations[5]], // thirst, resources
        base_score: 0.7 * modifiers.gathering_bonus,
    });

    // Build action - modified by building bonus
    ai.add_action(UtilityAction {
        name: "Build".to_string(),
        action_type: UtilityActionType::Build,
        considerations: vec![considerations[5]], // resources
        base_score: 0.6 * modifiers.building_bonus,
    });

    // Research action - modified by research bonus
    ai.add_action(UtilityAction {
        name: "Research".to_string(),
        action_type: UtilityActionType::Research,
        considerations: vec![considerations[0], considerations[5]], // thirst, resources
        base_score: 0.4 * modifiers.research_bonus,
    });

    // Retreat action - influenced by fear
    ai.add_action(UtilityAction {
        name: "Retreat".to_string(),
        action_type: UtilityActionType::Retreat,
        considerations: vec![considerations[1], considerations[3]], // fear, health
        base_score: 0.3 * (1.0 + profile.fear_threshold),
    });

    // Cult-specific actions
    for preferred_action in &profile.preferred_actions {
        ai.add_action(UtilityAction {
            name: preferred_action.clone(),
            action_type: UtilityActionType::Custom(preferred_action.clone()),
            considerations: vec![considerations[0], considerations[2]], // thirst, aggression
            base_score: 0.8, // High priority for preferred actions
        });
    }
}

/// System to update psychological states based on game events
pub fn update_psychological_state_system(
    mut query: Query<(&mut PsychologicalState, &CultProfile, &Transform)>,
    time: Res<Time>,
) {
    let delta = time.delta_secs();

    for (mut psychological_state, cult_profile, _transform) in query.iter_mut() {
        // Gradually return to baseline
        psychological_state.thirst_level =
            (psychological_state.thirst_level - delta * 0.1).max(0.0);
        psychological_state.fear_level = (psychological_state.fear_level - delta * 0.2).max(0.0);

        // Aggression increases with thirst for certain cults
        if cult_profile.cult_name == "Crimson Covenant" {
            psychological_state.aggression_level = (psychological_state.aggression_level
                + psychological_state.thirst_level * delta * 0.5)
                .min(1.0);
        }

        // Corruption influence affects void seekers
        if cult_profile.cult_name == "Void Seekers" {
            psychological_state.corruption_influence =
                (psychological_state.corruption_influence + delta * 0.1).min(1.0);
        }
    }
}

/// Helper function to create AI coordination for cult units
pub fn create_cult_coordination(cult_profile: &CultProfile, role: AIRole) -> AICoordination {
    let base_radius = match cult_profile.cult_name.as_str() {
        "Order of the Deep" => 80.0, // Larger coordination for defensive play
        "Crimson Covenant" => 60.0,  // Medium range for aggressive coordination
        "Void Seekers" => 100.0,     // Large range for mystical influence
        _ => 50.0,
    };

    AICoordination {
        team_id: 1,
        role: role.clone(),
        coordination_radius: base_radius,
        can_give_orders: matches!(role, AIRole::Leader),
        can_receive_orders: !matches!(role, AIRole::Leader),
    }
}

/// Event to trigger psychological state changes
#[derive(Event, Clone, Debug)]
pub struct PsychologicalEvent {
    pub entity: Entity,
    pub event_type: PsychologicalEventType,
    pub intensity: f32,
}

#[derive(Clone, Debug)]
pub enum PsychologicalEventType {
    TakeDamage,
    KillEnemy,
    DiscoverResource,
    LoseAlly,
    CompleteRitual,
    CorruptionExposure,
}

/// System to handle psychological events
pub fn handle_psychological_events(
    mut events: MessageReader<PsychologicalEvent>,
    mut query: Query<&mut PsychologicalState>,
) {
    for event in events.read() {
        if let Ok(mut psychological_state) = query.get_mut(event.entity) {
            match event.event_type {
                PsychologicalEventType::TakeDamage => {
                    psychological_state.fear_level =
                        (psychological_state.fear_level + event.intensity * 0.3).min(1.0);
                }
                PsychologicalEventType::KillEnemy => {
                    psychological_state.aggression_level =
                        (psychological_state.aggression_level + event.intensity * 0.2).min(1.0);
                    psychological_state.thirst_level =
                        (psychological_state.thirst_level + event.intensity * 0.4).min(1.0);
                }
                PsychologicalEventType::DiscoverResource => {
                    psychological_state.thirst_level =
                        (psychological_state.thirst_level + event.intensity * 0.5).min(1.0);
                }
                PsychologicalEventType::LoseAlly => {
                    psychological_state.fear_level =
                        (psychological_state.fear_level + event.intensity * 0.4).min(1.0);
                }
                PsychologicalEventType::CompleteRitual => {
                    psychological_state.thirst_level =
                        (psychological_state.thirst_level - event.intensity * 0.3).max(0.0);
                }
                PsychologicalEventType::CorruptionExposure => {
                    psychological_state.corruption_influence =
                        (psychological_state.corruption_influence + event.intensity * 0.6).min(1.0);
                }
            }
        }
    }
}

/// Preset AI configurations for each cult
pub mod presets {
    use super::*;
    use crate::ai::systems::{AIState, AIStateMachine, AITransition};

    /// Create a complete AI setup for Order of the Deep units
    pub fn create_order_of_deep_ai() -> (CultProfile, UtilityAI, AICoordination, PsychologicalState)
    {
        let profile = create_order_of_the_deep_profile();
        let utility_ai = create_cult_ai(&profile);
        let coordination = create_cult_coordination(&profile, AIRole::Follower);
        let psychological_state = PsychologicalState {
            thirst_level: 0.6,     // High knowledge thirst
            fear_level: 0.4,       // Moderate caution
            aggression_level: 0.3, // Low aggression
            corruption_influence: 0.0,
        };

        (profile, utility_ai, coordination, psychological_state)
    }

    /// Create a complete AI setup for Crimson Covenant units
    pub fn create_crimson_covenant_ai()
    -> (CultProfile, UtilityAI, AICoordination, PsychologicalState) {
        let profile = create_crimson_covenant_profile();
        let utility_ai = create_cult_ai(&profile);
        let coordination = create_cult_coordination(&profile, AIRole::Follower);
        let psychological_state = PsychologicalState {
            thirst_level: 0.8,     // High blood thirst
            fear_level: 0.2,       // Low fear
            aggression_level: 0.7, // High aggression
            corruption_influence: 0.3,
        };

        (profile, utility_ai, coordination, psychological_state)
    }

    /// Create a complete AI setup for Void Seekers units
    pub fn create_void_seekers_ai() -> (CultProfile, UtilityAI, AICoordination, PsychologicalState)
    {
        let profile = create_void_seekers_profile();
        let utility_ai = create_cult_ai(&profile);
        let coordination = create_cult_coordination(&profile, AIRole::Follower);
        let psychological_state = PsychologicalState {
            thirst_level: 0.5,         // Moderate thirst
            fear_level: 0.3,           // Moderate fear
            aggression_level: 0.5,     // Moderate aggression
            corruption_influence: 0.8, // High corruption
        };

        (profile, utility_ai, coordination, psychological_state)
    }
}
impl bevy::prelude::Message for PsychologicalEvent {}
