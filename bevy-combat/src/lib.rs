//! # Bevy Combat Toolkit
//!
//! A collection of generic combat systems for the Bevy game engine.
//!
//! ## Features
//!
//! - **Damage System**: Flexible damage calculation with resistances
//! - **Status Effects**: Buffs and debuffs with duration and stacking
//! - **Combat Stats**: Customizable attributes for combat entities
//! - **Visual Effects**: Damage numbers, health bars, and death effects

pub mod components;
pub mod damage;
pub mod effects;

pub mod prelude {
    pub use crate::components::*;
    pub use crate::damage::{DamageEvent, DamagePlugin, DeathEvent};
    pub use crate::effects::{DeathEffect, DeathEffectType, EffectsPlugin, HealthBar};
    pub use crate::BevyCombatPlugin;
}

use bevy::prelude::*;

/// Plugin that adds all combat toolkit systems to a Bevy app
pub struct BevyCombatPlugin;

impl Plugin for BevyCombatPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            damage::DamagePlugin,
            effects::EffectsPlugin,
        ));
    }
}
