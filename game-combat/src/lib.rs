// Game Combat - Clean, modular combat system for Cosmic Dominion
// Uses seldom_state for proper state management and Rapier3D for physics

use bevy::prelude::*;

pub mod states;
pub mod components;
pub mod systems;
pub mod effects;
pub mod xp;
pub mod targeting;
pub mod damage;
pub mod plugin;
pub mod visuals;
pub mod physics_integration;

// Re-export main types
pub use states::*;
pub use components::*;
pub use systems::*;
pub use effects::*;
pub use xp::*;
pub use targeting::*;
pub use damage::*;
pub use plugin::CombatPlugin;
pub use visuals::*;
pub use physics_integration::*;

/// The main combat plugin that integrates all combat systems
pub struct GameCombatPlugin;

impl Plugin for GameCombatPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_plugins(CombatPlugin)
            .add_plugins(TargetingPlugin)
            .add_plugins(DamagePlugin)
            .add_plugins(EffectsPlugin)
            .add_plugins(XPPlugin)
            .add_plugins(CombatVisualsPlugin)
            .add_plugins(CombatPhysicsPlugin);
    }
}
