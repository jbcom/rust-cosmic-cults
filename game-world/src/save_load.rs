//! Save and load game state serialization system

use crate::{GameMap, PathfindingGrid, VisibilityMap};
use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use tracing::info;

/// Serializable representation of a unit's position and state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerializableUnit {
    pub position: (f32, f32, f32),
    pub rotation: (f32, f32, f32, f32),
    pub cult: String,
    pub unit_type: String,
    pub health: f32,
    pub max_health: f32,
    pub experience: u32,
    pub veteran_tier: u32,
}

/// Serializable representation of a leader unit
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerializableLeader {
    pub position: (f32, f32, f32),
    pub rotation: (f32, f32, f32, f32),
    pub name: String,
    pub cult: String,
    pub health: f32,
    pub max_health: f32,
    pub shield: f32,
    pub aura_radius: f32,
    pub aura_type: String,
    pub alive: bool,
}

/// Complete game state that can be serialized and deserialized
#[derive(Debug, Serialize, Deserialize)]
pub struct GameState {
    /// Version of the save format for compatibility
    pub version: u32,
    /// Timestamp of when the save was created
    pub timestamp: String,
    /// Game map data
    pub game_map: GameMap,
    /// Pathfinding grid data
    pub pathfinding_grid: PathfindingGrid,
    /// Visibility/fog of war data
    pub visibility_map: VisibilityMap,
    /// All units in the game
    pub units: Vec<SerializableUnit>,
    /// All leaders in the game
    pub leaders: Vec<SerializableLeader>,
    /// Custom data for future extensions
    pub custom_data: HashMap<String, String>,
}

impl GameState {
    /// Create a new game state with current timestamp
    pub fn new(
        game_map: GameMap,
        pathfinding_grid: PathfindingGrid,
        visibility_map: VisibilityMap,
    ) -> Self {
        Self {
            version: 1,
            timestamp: chrono::Utc::now().to_rfc3339(),
            game_map,
            pathfinding_grid,
            visibility_map,
            units: Vec::new(),
            leaders: Vec::new(),
            custom_data: HashMap::new(),
        }
    }

    /// Serialize the game state to bytes using bincode
    pub fn to_bytes(&self) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        bincode::serialize(self).map_err(|e| e.into())
    }

    /// Deserialize game state from bytes using bincode
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, Box<dyn std::error::Error>> {
        bincode::deserialize(bytes).map_err(|e| e.into())
    }

    /// Save game state to a file
    pub fn save_to_file<P: AsRef<Path>>(&self, path: P) -> Result<(), Box<dyn std::error::Error>> {
        let bytes = self.to_bytes()?;
        fs::write(path, bytes)?;
        Ok(())
    }

    /// Load game state from a file
    pub fn load_from_file<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn std::error::Error>> {
        let bytes = fs::read(path)?;
        Self::from_bytes(&bytes)
    }
}

/// Save the current game state to a file
pub fn save_game(
    path: &str,
    game_map: Res<GameMap>,
    pathfinding_grid: Res<PathfindingGrid>,
    visibility_map: Res<VisibilityMap>,
    units_query: Query<(&Transform, &crate::PlayerUnit), Without<crate::CultLeader>>,
    leaders_query: Query<(&Transform, &crate::CultLeader)>,
) -> Result<(), Box<dyn std::error::Error>> {
    // Create game state with resource data
    let mut game_state = GameState::new(
        game_map.clone(),
        pathfinding_grid.clone(),
        visibility_map.clone(),
    );

    // Serialize units
    for (transform, _unit) in units_query.iter() {
        let unit = SerializableUnit {
            position: (
                transform.translation.x,
                transform.translation.y,
                transform.translation.z,
            ),
            rotation: (
                transform.rotation.x,
                transform.rotation.y,
                transform.rotation.z,
                transform.rotation.w,
            ),
            cult: String::from("Player"),
            unit_type: String::from("Basic"),
            health: 100.0,
            max_health: 100.0,
            experience: 0,
            veteran_tier: 0,
        };
        game_state.units.push(unit);
    }

    // Serialize leaders
    for (transform, _leader) in leaders_query.iter() {
        let leader = SerializableLeader {
            position: (
                transform.translation.x,
                transform.translation.y,
                transform.translation.z,
            ),
            rotation: (
                transform.rotation.x,
                transform.rotation.y,
                transform.rotation.z,
                transform.rotation.w,
            ),
            name: String::from("Leader"),
            cult: String::from("Player"),
            health: 200.0,
            max_health: 200.0,
            shield: 50.0,
            aura_radius: 15.0,
            aura_type: String::from("Leadership"),
            alive: true,
        };
        game_state.leaders.push(leader);
    }

    // Save to file
    game_state.save_to_file(path)?;
    info!("Game saved successfully to {}", path);
    Ok(())
}

/// Load game state from a file
pub fn load_game(path: &str) -> Result<GameState, Box<dyn std::error::Error>> {
    let game_state = GameState::load_from_file(path)?;
    info!("Game loaded successfully from {}", path);
    Ok(game_state)
}

/// Apply loaded game state to the world
pub fn apply_game_state(
    mut commands: Commands,
    game_state: GameState,
    mut game_map: ResMut<GameMap>,
    mut pathfinding_grid: ResMut<PathfindingGrid>,
    mut visibility_map: ResMut<VisibilityMap>,
) {
    // Apply resource states
    *game_map = game_state.game_map;
    *pathfinding_grid = game_state.pathfinding_grid;
    *visibility_map = game_state.visibility_map;

    // Spawn units
    for unit in game_state.units {
        commands.spawn((
            Transform::from_translation(Vec3::new(unit.position.0, unit.position.1, unit.position.2))
                .with_rotation(Quat::from_xyzw(
                    unit.rotation.0,
                    unit.rotation.1,
                    unit.rotation.2,
                    unit.rotation.3,
                )),
            crate::PlayerUnit {
                unit_type: crate::spawning::UnitType::Acolyte,
            },
        ));
    }

    // Spawn leaders
    for leader in game_state.leaders {
        commands.spawn((
            Transform::from_translation(Vec3::new(
                leader.position.0,
                leader.position.1,
                leader.position.2,
            ))
            .with_rotation(Quat::from_xyzw(
                leader.rotation.0,
                leader.rotation.1,
                leader.rotation.2,
                leader.rotation.3,
            )),
            crate::CultLeader {
                cult: game_assets::Cult::Crimson,
                level: 1,
            },
        ));
    }

    info!("Game state applied successfully");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_game_state_serialization() {
        let game_map = GameMap::default();
        let pathfinding_grid = PathfindingGrid::default();
        let visibility_map = VisibilityMap::default();

        let game_state = GameState::new(game_map, pathfinding_grid, visibility_map);

        // Test serialization
        let bytes = game_state.to_bytes().expect("Failed to serialize");
        assert!(!bytes.is_empty());

        // Test deserialization
        let loaded_state = GameState::from_bytes(&bytes).expect("Failed to deserialize");
        assert_eq!(loaded_state.version, 1);
    }

    #[test]
    fn test_game_state_with_units() {
        let game_map = GameMap::default();
        let pathfinding_grid = PathfindingGrid::default();
        let visibility_map = VisibilityMap::default();

        let mut game_state = GameState::new(game_map, pathfinding_grid, visibility_map);

        // Add a unit
        game_state.units.push(SerializableUnit {
            position: (10.0, 0.0, 10.0),
            rotation: (0.0, 0.0, 0.0, 1.0),
            cult: String::from("Player"),
            unit_type: String::from("Infantry"),
            health: 100.0,
            max_health: 100.0,
            experience: 50,
            veteran_tier: 1,
        });

        // Add a leader
        game_state.leaders.push(SerializableLeader {
            position: (0.0, 0.0, 0.0),
            rotation: (0.0, 0.0, 0.0, 1.0),
            name: String::from("Commander"),
            cult: String::from("Player"),
            health: 200.0,
            max_health: 200.0,
            shield: 50.0,
            aura_radius: 15.0,
            aura_type: String::from("Leadership"),
            alive: true,
        });

        // Test round-trip serialization
        let bytes = game_state.to_bytes().expect("Failed to serialize");
        let loaded_state = GameState::from_bytes(&bytes).expect("Failed to deserialize");

        assert_eq!(loaded_state.units.len(), 1);
        assert_eq!(loaded_state.leaders.len(), 1);
        assert_eq!(loaded_state.units[0].cult, "Player");
        assert_eq!(loaded_state.leaders[0].name, "Commander");
    }
}
