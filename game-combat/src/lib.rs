// Game Combat - Clean, modular combat system for Cosmic Dominion
// Uses seldom_state for proper state management and Rapier3D for physics

#![allow(clippy::collapsible_if)]
#![allow(clippy::let_and_return)]

use bevy::prelude::*;

pub mod components;
pub mod damage;
pub mod effects;
pub mod physics_integration;
pub mod plugin;
pub mod states;
pub mod systems;
pub mod targeting;
pub mod visuals;
pub mod xp;

// Re-export main types
pub use components::*;
pub use damage::*;
pub use effects::{
    CombatParticle, DamageNumber, DeathEffect, DeathEffectType, EffectsPlugin, HealthBar,
    ParticleType as EffectParticleType, combat_particle_system, damage_number_system,
    death_effect_system, health_bar_system, spawn_damage_number, spawn_death_effect,
};
// physics_integration is empty, no need to re-export
pub use plugin::CombatPlugin;
pub use states::*;
pub use systems::*;
pub use targeting::*;
pub use visuals::{
    BuffVisualIndicator, CombatVisualsPlugin, HitFlash, ParticleType, ProjectileTrail,
    ShieldEffect, SpawnVisualDamageNumberEvent, SpawnVisualDeathEffectEvent, VisualCombatParticle,
    VisualDamageNumber, VisualDeathEffect, animate_buff_indicators, apply_hit_flash,
    cleanup_expired_effects, create_buff_indicator, create_projectile_trail, create_shield_effect,
    handle_death_effects, spawn_damage_numbers, update_combat_particles, update_damage_numbers,
    update_death_effects, update_hit_flash, update_projectile_trails, update_shield_effects,
};
pub use xp::*;

/// The main combat plugin that integrates all combat systems
pub struct GameCombatPlugin;

impl Plugin for GameCombatPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(CombatPlugin)
            .add_plugins(TargetingPlugin)
            .add_plugins(DamagePlugin)
            .add_plugins(EffectsPlugin)
            .add_plugins(XPPlugin)
            .add_plugins(CombatVisualsPlugin);
    }
}
