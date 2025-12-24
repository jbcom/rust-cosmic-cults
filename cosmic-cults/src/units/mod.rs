#![allow(unused)]
use bevy::prelude::*;
use game_physics::{
    AABB, GamePhysicsPlugin, Mass, MovementCommand, MovementCommandEvent, MovementController,
    SpatialData, Velocity,
};

// Module declarations
pub mod components;
pub mod formations;
pub mod leadership;
pub mod pathfinding_integration;
pub mod physics_integration;
pub mod selection;
pub mod spawning;
pub mod visuals;

// Re-exports for easy access
pub use components::*;
pub use formations::*;
pub use leadership::*;
pub use pathfinding_integration::*;
pub use physics_integration::*;
pub use selection::*;
pub use spawning::*;
pub use visuals::*;

// Export additional types that bevy-web might need
pub use leadership::LeadershipBuilding;
pub use spawning::{GameAssets, UnitTemplate, UnitTemplates, init_game_assets};

// Re-export physics types units need
pub use game_physics::{MovementPath, MovementTarget};

// Main plugin for the game-units crate
#[derive(Default)]
pub struct GameUnitsPlugin;

impl Plugin for GameUnitsPlugin {
    fn build(&self, app: &mut App) {
        // Physics plugin is added by app.rs, don't add it here

        // Add physics integration plugin
        app.add_plugins(UnitsPhysicsIntegrationPlugin);

        // Add pathfinding integration plugin
        app.add_plugins(PathfindingIntegrationPlugin);

        app
            // Register resources
            .init_resource::<SelectionState>()
            .init_resource::<InputState>()
            .init_resource::<CommandQueue>()
            .init_resource::<UnitTemplates>()
            // Add startup system for loading assets
            .add_systems(Startup, init_game_assets)
            // Register systems in groups to avoid tuple length limits
            .add_systems(
                Update,
                (
                    // Selection systems
                    selection_system,
                    movement_command_system,
                    enhanced_movement_system,
                    group_selection_system,
                ),
            )
            .add_systems(
                Update,
                (
                    // Visual systems - PRODUCTION visual updates
                    update_health_bars,
                    update_selection_indicators,
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
            )
            .add_systems(
                Update,
                (
                    // Spawning systems (optional debug systems)
                    debug_spawn_system,
                ),
            );
    }
}
