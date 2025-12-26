use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
#[cfg(feature = "web")]
use web_sys::console;

// Health component
#[derive(Component, Clone, Debug, Serialize, Deserialize)]
pub struct Health {
    pub current: f32,
    pub maximum: f32,
}

impl Health {
    pub fn new(max: f32) -> Self {
        Self {
            current: max,
            maximum: max,
        }
    }
}

// Core unit component - the main entity type for units
#[derive(Component, Clone, Debug, Serialize, Deserialize)]
pub struct Unit {
    pub cult: String,
    pub unit_type: String,
    pub health: f32,
    pub max_health: f32,
    pub experience: u32,
    pub veteran_tier: u32,
    pub attack_damage: f32,
    pub movement_speed: f32,
    pub attack_speed: f32,
}

impl Default for Unit {
    fn default() -> Self {
        Self {
            cult: String::new(),
            unit_type: String::new(),
            health: 100.0,
            max_health: 100.0,
            experience: 0,
            veteran_tier: 0,
            attack_damage: 10.0,
            movement_speed: 5.0,
            attack_speed: 1.0,
        }
    }
}

// Leader component - special units with abilities and auras
#[derive(Component, Clone, Debug)]
pub struct Leader {
    pub name: String,
    pub cult: String,
    pub health: f32,
    pub max_health: f32,
    pub shield: f32,
    pub aura_radius: f32,
    pub aura_type: AuraType,
    pub platform_entity: Option<Entity>,
    pub defeat_on_death: bool,
    pub alive: bool,
    pub last_ability1_use: f32,
    pub last_ability2_use: f32,
}

impl Default for Leader {
    fn default() -> Self {
        Self {
            name: String::new(),
            cult: String::new(),
            health: 200.0,
            max_health: 200.0,
            shield: 50.0,
            aura_radius: 15.0,
            aura_type: AuraType::Leadership,
            platform_entity: None,
            defeat_on_death: true,
            alive: true,
            last_ability1_use: 0.0,
            last_ability2_use: 0.0,
        }
    }
}

// Aura types for different cult leadership styles
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum AuraType {
    Crimson,    // Attack damage boost
    Deep,       // Health and regeneration boost
    Void,       // Speed and XP boost
    Leadership, // Balanced buff to all stats
}

// Team affiliation component
#[derive(Component, Clone, Debug)]
pub struct Team {
    pub id: u32,
    pub cult: String,
    pub color: Color,
}

impl Default for Team {
    fn default() -> Self {
        Self {
            id: 0,
            cult: String::new(),
            color: Color::WHITE,
        }
    }
}

// Selection marker for selected units
#[derive(Component, Clone, Debug)]
pub struct Selected;

// Selectable marker with properties for unit selection
#[derive(Component, Clone, Debug)]
pub struct Selectable {
    pub selection_priority: u32,
    pub selection_radius: f32,
}

impl Default for Selectable {
    fn default() -> Self {
        Self {
            selection_priority: 1,
            selection_radius: 1.5,
        }
    }
}

// Movement target re-exported from game-physics
// See game_physics::components::MovementTarget

// Base stats component for buff calculations
#[derive(Component, Clone, Debug, Serialize, Deserialize)]
pub struct BaseStats {
    pub base_attack_damage: f32,
    pub base_health: f32,
    pub base_speed: f32,
    pub base_attack_speed: f32,
    pub initialized: bool,
}

impl Default for BaseStats {
    fn default() -> Self {
        Self {
            base_attack_damage: 10.0,
            base_health: 100.0,
            base_speed: 5.0,
            base_attack_speed: 1.0,
            initialized: false,
        }
    }
}

// Experience component for unit progression
#[derive(Component, Clone, Debug, Serialize, Deserialize)]
pub struct Experience {
    pub current: u32,
    pub total_earned: u32,
    pub level: u32,
    pub kills: u32,
    pub buildings_destroyed: u32,
}

impl Default for Experience {
    fn default() -> Self {
        Self {
            current: 0,
            total_earned: 0,
            level: 1,
            kills: 0,
            buildings_destroyed: 0,
        }
    }
}

// Veteran status component
#[derive(Component, Clone, Debug, Serialize, Deserialize)]
pub struct VeteranStatus {
    pub tier: VeteranTier,
    pub promotion_ready: bool,
    pub visual_scale: f32,
    pub bonuses: VeteranBonus,
}

impl Default for VeteranStatus {
    fn default() -> Self {
        Self {
            tier: VeteranTier::Recruit,
            promotion_ready: false,
            visual_scale: 1.0,
            bonuses: VeteranBonus::default(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum VeteranTier {
    Recruit,
    Regular,
    Veteran,
    Elite,
    Legendary,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct VeteranBonus {
    pub health_multiplier: f32,
    pub damage_multiplier: f32,
    pub speed_multiplier: f32,
    pub xp_multiplier: f32,
}

impl Default for VeteranBonus {
    fn default() -> Self {
        Self {
            health_multiplier: 1.0,
            damage_multiplier: 1.0,
            speed_multiplier: 1.0,
            xp_multiplier: 1.0,
        }
    }
}

// Aura buff component for temporary stat bonuses
#[derive(Component, Clone, Debug)]
pub struct AuraBuff {
    pub target_unit: Entity,
    pub team: u32,
    pub atk_mul: f32,
    pub hp_mul: f32,
    pub speed_mul: f32,
    pub xp_mul: f32,
    pub expires_at: f32,
    pub strength: f32,
}

impl Default for AuraBuff {
    fn default() -> Self {
        Self {
            target_unit: Entity::PLACEHOLDER,
            team: 0,
            atk_mul: 1.0,
            hp_mul: 1.0,
            speed_mul: 1.0,
            xp_mul: 1.0,
            expires_at: 0.0,
            strength: 1.0,
        }
    }
}

// Movement path re-exported from game-physics
// See game_physics::components::MovementPath

// Unit formation data
#[derive(Component, Clone, Debug)]
pub struct Formation {
    pub formation_type: FormationType,
    pub position_in_formation: Vec2,
    pub leader_entity: Option<Entity>,
    pub spacing: f32,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum FormationType {
    Line,
    Column,
    Box,
    Wedge,
    Circle,
}

impl Default for Formation {
    fn default() -> Self {
        Self {
            formation_type: FormationType::Box,
            position_in_formation: Vec2::ZERO,
            leader_entity: None,
            spacing: 2.0,
        }
    }
}
