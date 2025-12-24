// Cult-specific AI profiles and behaviors for Cosmic Cults
use bevy::prelude::*;
use bevy_ai_toolkit::prelude::*;
use serde::{Deserialize, Serialize};

/// Cult-specific AI behavioral modifiers
#[derive(Component, Clone, Debug, Serialize, Deserialize)]
pub struct CultProfile {
    pub cult_name: String,
    pub behavioral_modifiers: CultBehaviorModifiers,
    pub preferred_actions: Vec<String>,
    pub fear_threshold: f32,
    pub aggression_level: f32,
    pub thirst_drive: f32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
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
#[derive(Component, Clone, Debug, Serialize, Deserialize)]
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

pub fn create_cult_profile(cult_name: &str) -> CultProfile {
    match cult_name {
        "Order of the Deep" => create_order_of_the_deep_profile(),
        "Crimson Covenant" => create_crimson_covenant_profile(),
        "Void Seekers" => create_void_seekers_profile(),
        _ => create_default_cult_profile(cult_name),
    }
}

fn create_order_of_the_deep_profile() -> CultProfile {
    CultProfile {
        cult_name: "Order of the Deep".to_string(),
        behavioral_modifiers: CultBehaviorModifiers {
            attack_bonus: 0.8,
            defense_bonus: 1.3,
            gathering_bonus: 1.2,
            building_bonus: 1.4,
            research_bonus: 1.6,
            expansion_bonus: 0.9,
        },
        preferred_actions: vec![
            "research".to_string(),
            "build_sanctuary".to_string(),
            "gather_knowledge".to_string(),
            "defend_territory".to_string(),
        ],
        fear_threshold: 0.3,
        aggression_level: 0.4,
        thirst_drive: 0.8,
    }
}

fn create_crimson_covenant_profile() -> CultProfile {
    CultProfile {
        cult_name: "Crimson Covenant".to_string(),
        behavioral_modifiers: CultBehaviorModifiers {
            attack_bonus: 1.5,
            defense_bonus: 1.1,
            gathering_bonus: 1.0,
            building_bonus: 0.9,
            research_bonus: 0.8,
            expansion_bonus: 1.3,
        },
        preferred_actions: vec![
            "attack_enemies".to_string(),
            "blood_ritual".to_string(),
            "expand_territory".to_string(),
            "hunt_souls".to_string(),
        ],
        fear_threshold: 0.6,
        aggression_level: 0.8,
        thirst_drive: 0.9,
    }
}

fn create_void_seekers_profile() -> CultProfile {
    CultProfile {
        cult_name: "Void Seekers".to_string(),
        behavioral_modifiers: CultBehaviorModifiers {
            attack_bonus: 1.1,
            defense_bonus: 0.9,
            gathering_bonus: 0.8,
            building_bonus: 1.2,
            research_bonus: 1.4,
            expansion_bonus: 1.1,
        },
        preferred_actions: vec![
            "corrupt_land".to_string(),
            "void_ritual".to_string(),
            "summon_entities".to_string(),
            "spread_influence".to_string(),
        ],
        fear_threshold: 0.4,
        aggression_level: 0.6,
        thirst_drive: 0.7,
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

pub fn create_cult_ai(cult_profile: &CultProfile) -> UtilityAI {
    let mut ai = UtilityAI::new();

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

    ai.add_action(UtilityAction {
        name: "Attack".to_string(),
        action_type: UtilityActionType::Attack,
        considerations: vec![considerations[2], considerations[3], considerations[4]],
        base_score: 0.6 * modifiers.attack_bonus,
    });

    ai.add_action(UtilityAction {
        name: "Defend".to_string(),
        action_type: UtilityActionType::Defend,
        considerations: vec![considerations[1], considerations[3]],
        base_score: 0.5 * modifiers.defense_bonus,
    });

    ai.add_action(UtilityAction {
        name: "Gather".to_string(),
        action_type: UtilityActionType::Gather,
        considerations: vec![considerations[0], considerations[5]],
        base_score: 0.7 * modifiers.gathering_bonus,
    });

    ai.add_action(UtilityAction {
        name: "Build".to_string(),
        action_type: UtilityActionType::Build,
        considerations: vec![considerations[5]],
        base_score: 0.6 * modifiers.building_bonus,
    });

    ai.add_action(UtilityAction {
        name: "Research".to_string(),
        action_type: UtilityActionType::Research,
        considerations: vec![considerations[0], considerations[5]],
        base_score: 0.4 * modifiers.research_bonus,
    });

    ai.add_action(UtilityAction {
        name: "Retreat".to_string(),
        action_type: UtilityActionType::Retreat,
        considerations: vec![considerations[1], considerations[3]],
        base_score: 0.3 * (1.0 + profile.fear_threshold),
    });

    for preferred_action in &profile.preferred_actions {
        ai.add_action(UtilityAction {
            name: preferred_action.clone(),
            action_type: UtilityActionType::Custom(preferred_action.clone()),
            considerations: vec![considerations[0], considerations[2]],
            base_score: 0.8,
        });
    }
}

pub fn update_psychological_state_system(
    mut query: Query<(&mut PsychologicalState, &CultProfile)>,
    time: Res<Time>,
) {
    let delta = time.delta_seconds();

    for (mut psychological_state, cult_profile) in query.iter_mut() {
        psychological_state.thirst_level = (psychological_state.thirst_level - delta * 0.1).max(0.0);
        psychological_state.fear_level = (psychological_state.fear_level - delta * 0.2).max(0.0);

        if cult_profile.cult_name == "Crimson Covenant" {
            psychological_state.aggression_level = (psychological_state.aggression_level
                + psychological_state.thirst_level * delta * 0.5)
                .min(1.0);
        }

        if cult_profile.cult_name == "Void Seekers" {
            psychological_state.corruption_influence =
                (psychological_state.corruption_influence + delta * 0.1).min(1.0);
        }
    }
}

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

pub fn handle_psychological_events(
    mut events: EventReader<PsychologicalEvent>,
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
