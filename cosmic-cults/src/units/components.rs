use bevy::prelude::*;
use serde::{Deserialize, Serialize};

// === Core Unit Components ===

#[derive(Component, Clone, Debug, Default, Reflect, Serialize, Deserialize)]
#[reflect(Component)]
pub struct Unit {
    pub cult: String,
    pub unit_type: String,
    pub attack_damage: f32,
    pub attack_speed: f32,
    pub movement_speed: f32,
}

#[derive(Component, Clone, Debug, Default, Reflect, Serialize, Deserialize)]
#[reflect(Component)]
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

// === Leadership & Factions ===

#[derive(Component, Clone, Debug, Default, Reflect, Serialize, Deserialize)]
#[reflect(Component)]
pub struct Team {
    pub id: u32,
    pub cult: String,
    pub color: Color,
}

#[derive(Component, Clone, Debug, Default, Reflect)]
#[reflect(Component)]
pub struct Leader {
    pub name: String,
    pub aura_radius: f32,
    pub aura_type: AuraType,
    pub platform_entity: Option<Entity>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Reflect, Default)]
pub enum AuraType {
    #[default]
    Leadership,
    Crimson,
    Deep,
    Void,
}

// === RTS Systems Data ===

#[derive(Component, Clone, Debug, Default, Reflect)]
#[reflect(Component)]
pub struct SelectionPriority {
    pub value: u32,
}

#[derive(Component, Clone, Debug, Default, Reflect)]
#[reflect(Component)]
pub struct MovementPath {
    pub waypoints: Vec<Vec3>,
    pub current_index: usize,
}

#[derive(Component, Clone, Debug, Default, Reflect)]
#[reflect(Component)]
pub struct Formation {
    pub formation_type: FormationType,
    pub spacing: f32,
}

#[derive(Clone, Debug, PartialEq, Reflect, Default)]
pub enum FormationType {
    #[default]
    Box,
    Line,
    Circle,
    Wedge,
}

// === Progression ===

#[derive(Component, Clone, Debug, Default, Reflect)]
#[reflect(Component)]
pub struct Experience {
    pub level: u32,
    pub total: u32,
}

#[derive(Component, Clone, Debug, Default, Reflect)]
#[reflect(Component)]
pub struct VeteranStatus {
    pub tier: u32,
}
