// Game Combat - Clean, modular combat system for Cosmic Dominion
// Uses seldom_state for proper state management and Rapier3D for physics

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
pub use effects::*;
pub use plugin::CombatPlugin;
pub use states::*;
pub use systems::*;
pub use targeting::*;
pub use visuals::*;
pub use xp::*;
// physics_integration module is currently disabled

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
