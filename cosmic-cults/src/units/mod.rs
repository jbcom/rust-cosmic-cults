#![allow(unused)]
use bevy::prelude::*;
use avian3d::prelude::*;

// Module declarations
pub mod components;
pub mod formations;
pub mod leadership;
pub mod spawning;
pub mod visuals;

// Re-exports for easy access
pub use components::*;
pub use formations::*;
pub use leadership::*;
pub use spawning::*;
pub use visuals::*;

// Main plugin for the game-units crate
#[derive(Default)]
pub struct GameUnitsPlugin;

impl Plugin for GameUnitsPlugin {
    fn build(&self, app: &mut App) {
        app
            // Register resources
            .init_resource::<UnitTemplates>()
            // Add startup system for loading assets
            .add_systems(Startup, init_game_assets)
            // Register systems in groups
            .add_systems(
                Update,
                (
                    // Visual systems
                    update_health_bars,
                    animate_aura_visuals,
                    animate_leader_platforms,
                    update_veteran_indicators,
                    handle_death_visuals,
                    update_team_colors,
                    animate_idle_units,
                ),
            )
            .add_systems(
                Update,
                (
                    // Formation systems
                    formation_system,
                    leader_formation_system,
                    formation_switching_system,
                    formation_maintenance_system,
                    formation_spacing_system,
                ),
            )
            .add_systems(
                Update,
                (
                    // Leadership systems
                    defeat_condition_system,
                    leader_abilities_system,
                    buff_application_system,
                    aura_cleanup_system,
                    passive_aura_system,
                    platform_building_system,
                ),
            );
    }
}
