//! Game World Plugin for Cosmic Dominion
//!
//! This crate provides the complete world generation, terrain, fog of war,
//! and entity spawning systems for the game.

use bevy::prelude::*;
use tracing::info;

pub mod fog;
pub mod map;
pub mod save_load;
pub mod spawning;
pub mod terrain;

pub use fog::{Faction, FogOfWar, VisibilityMap, VisionProvider};
pub use map::{GameMap, MapTile, PathfindingGrid, find_path};
pub use save_load::{
    GameState, SerializableLeader, SerializableUnit, apply_game_state, load_game, save_game,
};
pub use spawning::{CultLeader, InitialCreature, LeadershipBuilding, PlayerUnit, Totem};
pub use terrain::{BiomeType, TerrainConfig, TerrainTile};

/// Main plugin for the game world systems
pub struct GameWorldPlugin;

impl Plugin for GameWorldPlugin {
    fn build(&self, app: &mut App) {
        // Initialize resources
        app.init_resource::<GameMap>()
            .init_resource::<PathfindingGrid>()
            .init_resource::<VisibilityMap>()
            .init_resource::<TerrainConfig>();

        // Add startup systems in the correct order
        app.add_systems(
            Startup,
            (
                map::initialize_map,
                terrain::generate_terrain_system,
                fog::initialize_fog_system,
                spawning::spawn_starting_scene,
            )
                .chain(),
        );

        // Add update systems
        app.add_systems(
            Update,
            (
                fog::update_fog_system,
                fog::reveal_around_spawn_system,
                fog::fog_entity_visibility_system,
                map::update_tile_occupation_system,
            ),
        );

        // Add debug visualization (can be disabled in production)
        #[cfg(debug_assertions)]
        app.add_systems(Update, map::debug_draw_map_grid);

        info!("Game World Plugin loaded successfully");
    }
}

/// Helper function to set up a basic game world for testing
pub fn setup_test_world(mut commands: Commands) {
    // Add ambient light
    commands.spawn(DirectionalLight {
        color: Color::srgb(0.8, 0.8, 0.9),
        illuminance: 5000.0,
        shadows_enabled: true,
        ..default()
    });

    // Add camera
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 30.0, 30.0).looking_at(Vec3::ZERO, Vec3::Y),
        Name::new("Main Camera"),
    ));

    // Add fog effect
    commands.spawn((
        EnvironmentMapLight {
            intensity: 500.0,
            diffuse_map: Handle::default(),
            specular_map: Handle::default(),
            affects_lightmapped_mesh_diffuse: true,
            rotation: Quat::IDENTITY,
        },
        Name::new("Environment Light"),
    ));
}

/// Configuration for different game modes
#[derive(Debug, Clone, Resource)]
pub struct WorldConfig {
    pub map_size: i32,
    pub starting_units: u32,
    pub fog_enabled: bool,
    pub corruption_rate: f32,
}

impl Default for WorldConfig {
    fn default() -> Self {
        Self {
            map_size: 17,
            starting_units: 1,
            fog_enabled: true,
            corruption_rate: 1.0,
        }
    }
}

/// Create a custom world configuration
pub fn create_world_config(mode: GameMode) -> WorldConfig {
    match mode {
        GameMode::Tutorial => WorldConfig {
            map_size: 11,
            starting_units: 2,
            fog_enabled: false,
            corruption_rate: 0.5,
        },
        GameMode::Standard => WorldConfig::default(),
        GameMode::Hardcore => WorldConfig {
            map_size: 21,
            starting_units: 1,
            fog_enabled: true,
            corruption_rate: 2.0,
        },
    }
}

#[derive(Debug, Clone, Copy)]
pub enum GameMode {
    Tutorial,
    Standard,
    Hardcore,
}
