# Game State Save/Load System

This document describes the game state serialization system implemented in the `game-world` crate.

## Overview

The save/load system allows you to serialize and deserialize the current game state, including:
- Map data (tiles, corruption levels, terrain)
- Pathfinding grid
- Fog of war/visibility state
- Unit positions and attributes
- Leader positions and attributes
- Custom game data

## Usage

### Basic Save/Load

```rust
use game_world::{GameState, GameMap, PathfindingGrid, VisibilityMap};

// Create or get your game resources
let game_map = GameMap::default();
let pathfinding_grid = PathfindingGrid::default();
let visibility_map = VisibilityMap::default();

// Create a game state
let mut game_state = GameState::new(game_map, pathfinding_grid, visibility_map);

// Save to file
game_state.save_to_file("saves/game1.sav")
    .expect("Failed to save game");

// Load from file
let loaded_state = GameState::load_from_file("saves/game1.sav")
    .expect("Failed to load game");
```

### In a Bevy System

```rust
use bevy::prelude::*;
use game_world::{save_game, GameMap, PathfindingGrid, VisibilityMap};

fn save_game_system(
    game_map: Res<GameMap>,
    pathfinding_grid: Res<PathfindingGrid>,
    visibility_map: Res<VisibilityMap>,
    units_query: Query<(&Transform, &PlayerUnit), Without<CultLeader>>,
    leaders_query: Query<(&Transform, &CultLeader)>,
) {
    if let Err(e) = save_game(
        "saves/autosave.sav",
        game_map,
        pathfinding_grid,
        visibility_map,
        units_query,
        leaders_query,
    ) {
        error!("Failed to save game: {}", e);
    }
}
```

### Adding Custom Data

You can store custom metadata with your save:

```rust
game_state.custom_data.insert(
    String::from("difficulty"),
    String::from("Hard"),
);
game_state.custom_data.insert(
    String::from("mission_id"),
    String::from("mission_1"),
);
game_state.custom_data.insert(
    String::from("playtime_seconds"),
    String::from("3600"),
);
```

### Serialization Format

The system uses [bincode](https://github.com/bincode-org/bincode) for efficient binary serialization. The save files are compact and fast to read/write.

## Architecture

### GameState Structure

```rust
pub struct GameState {
    pub version: u32,           // Save format version for compatibility
    pub timestamp: String,       // When the save was created
    pub game_map: GameMap,      // Complete map state
    pub pathfinding_grid: PathfindingGrid,
    pub visibility_map: VisibilityMap,
    pub units: Vec<SerializableUnit>,
    pub leaders: Vec<SerializableLeader>,
    pub custom_data: HashMap<String, String>,
}
```

### Serializable Components

Units and leaders are stored in a simplified serializable format:

```rust
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
```

## Save File Format

- **Format**: Binary (bincode)
- **Version**: 1
- **Extension**: `.sav` or `.bin` (your choice)
- **Typical Size**: ~400-1000 bytes for small maps, scales with map size and unit count

## Example

See `game-world/examples/save_load_demo.rs` for a complete working example:

```bash
cargo run --example save_load_demo
```

This example demonstrates:
- Creating a game state
- Adding units and leaders
- Saving to file
- Loading from file
- Verifying data integrity

## Version Compatibility

The `GameState` structure includes a `version` field for save format versioning. Future versions can implement migration logic:

```rust
match loaded_state.version {
    1 => {
        // Current format
        Ok(loaded_state)
    }
    v => {
        Err(format!("Unsupported save version: {}", v))
    }
}
```

## Performance Considerations

- **Serialization**: Fast, typically <1ms for small-medium games
- **File I/O**: Depends on disk speed, typically <10ms
- **Memory**: Entire game state is loaded into memory
- **Compression**: Not currently implemented, could be added if save file sizes become an issue

## Future Enhancements

Potential improvements for the save/load system:

1. **Compression**: Add optional gzip/zstd compression for save files
2. **Incremental Saves**: Save only changed data
3. **Cloud Storage**: Integration with cloud save services
4. **Multiple Save Slots**: UI for managing multiple saves
5. **Autosave**: Automatic periodic saves
6. **Screenshots**: Store thumbnail images with saves
7. **Metadata**: Track playtime, achievements, etc.
8. **Encryption**: Optional encryption for save files

## Testing

Run the tests:

```bash
cargo test -p game-world
```

The test suite includes:
- Basic serialization/deserialization
- Round-trip integrity checks
- Save file I/O
- Unit and leader serialization

## API Reference

For detailed API documentation, run:

```bash
cargo doc --no-deps --open -p game-world
```
