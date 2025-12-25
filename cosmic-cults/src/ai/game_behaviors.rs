//! Game-specific behavior components for AI entities
//!
//! These components mark entities as executing specific behaviors
//! and carry relevant data for those behaviors.

use bevy::prelude::*;

/// Gathering behavior marker component
#[derive(Component, Clone, Debug)]
pub struct GatheringBehavior {
    pub target_resource: Option<Entity>,
    pub gathering_rate: f32,
}

/// Building behavior marker component
#[derive(Component, Clone, Debug)]
pub struct BuildingBehavior {
    pub building_type: Option<String>,
    pub progress: f32,
}

/// Attack behavior marker component
#[derive(Component, Clone, Debug)]
pub struct AttackBehavior {
    pub target: Option<Entity>,
    pub aggression_level: f32,
}

/// Defend behavior marker component
#[derive(Component, Clone, Debug)]
pub struct DefendBehavior {
    pub defend_position: Vec3,
    pub patrol_radius: f32,
}

/// Retreat behavior marker component
#[derive(Component, Clone, Debug)]
pub struct RetreatBehavior {
    pub safe_position: Option<Vec3>,
    pub retreat_threshold: f32,
}
