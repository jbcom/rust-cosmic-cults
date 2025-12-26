# Pathfinding System

## Overview

The pathfinding system in Cosmic Cults uses A* algorithm to find paths for units that respect terrain walkability and obstacles. The implementation is split across two crates:

- **game-world**: Core pathfinding algorithm and grid management
- **game-units**: Integration with unit movement and physics

## Architecture

### Grid-Based Pathfinding

The game world is represented as a grid where each tile has:
- **Walkability**: Whether units can traverse the tile
- **Movement Cost**: Cost multiplier for pathfinding (higher = slower)
- **Tile Type**: Ground, Water, Cliff, Bridge, or Void

### A* Algorithm

The `find_path()` function in `game-world/src/map.rs` implements A* pathfinding:

```rust
pub fn find_path(
    start: (i32, i32),
    goal: (i32, i32),
    pathfinding_grid: &PathfindingGrid,
) -> Option<Vec<(i32, i32)>>
```

Features:
- 8-directional movement (including diagonals)
- Considers movement costs for different terrain types
- Returns `None` if no path exists
- Uses Manhattan distance heuristic

## Path Smoothing

The system includes path smoothing to reduce unnecessary waypoints:

```rust
pub fn smooth_path(
    waypoints: Vec<Vec3>,
    pathfinding_grid: &PathfindingGrid,
    tile_size: f32,
) -> Vec<Vec3>
```

Path smoothing:
- Removes intermediate waypoints when a direct path is possible
- Maintains safety by checking line-of-sight against walkability
- Reduces computation and makes unit movement more natural

## Dynamic Pathfinding

Units automatically recalculate paths when:
- Obstacles block their current path
- They become stuck (velocity drops to near zero)
- A new movement command is issued

The `dynamic_pathfinding_system` monitors unit movement and triggers recalculation as needed.

## Obstacle Handling

The `update_pathfinding_obstacles` system:
- Marks tiles occupied by obstacles as non-walkable
- Adds clearance around obstacles (adjacent tiles)
- Respects terrain-based walkability rules
- Updates dynamically as obstacles move

## Usage

### Basic Movement Command

```rust
movement_events.write(MovementCommandEvent {
    entity: unit_entity,
    command: MovementCommand::MoveTo {
        position: target_position,
        speed: 5.0,
    },
});
```

The pathfinding system will:
1. Convert world positions to grid coordinates
2. Find a path using A*
3. Smooth the path to remove unnecessary waypoints
4. Update the unit's MovementController with waypoints
5. The physics movement system will follow the waypoints

### Manual Path Setting

```rust
movement_events.write(MovementCommandEvent {
    entity: unit_entity,
    command: MovementCommand::SetPath {
        waypoints: vec![pos1, pos2, pos3],
        speed: 5.0,
    },
});
```

## Terrain Types and Walkability

| Tile Type | Default Walkable | Movement Cost | Notes |
|-----------|------------------|---------------|-------|
| Ground    | Yes              | 1.0           | Standard terrain |
| Bridge    | Yes              | 1.2           | Slightly slower |
| Water     | No               | 999.0         | Not traversable |
| Cliff     | No               | 999.0         | Not traversable |
| Void      | Conditional      | 2.0-5.0       | Walkable if corruption < 0.9 |

Corruption increases movement cost: `base_cost * (1.0 + corruption * 0.5)`

## Coordinate Systems

### Grid Coordinates
- Integer-based tile coordinates: `(i32, i32)`
- Center of map is typically `(0, 0)`
- Grid extends equally in all directions

### World Coordinates
- 3D floating-point positions: `Vec3`
- Y-axis represents height (typically 0 for ground units)
- Conversion maintains center alignment

### Conversion Functions

```rust
// World to grid
pub fn world_to_grid(world_pos: Vec3, tile_size: f32) -> (i32, i32)

// Grid to world
pub fn grid_to_world(grid_pos: (i32, i32), tile_size: f32) -> Vec3
```

## Performance Considerations

- A* complexity: O(n log n) where n is grid size
- Path smoothing: O(p²) where p is path length
- Grid updates: O(o) where o is number of obstacles
- Spatial queries use grid-based lookups for O(1) average case

## Testing

The pathfinding system includes comprehensive unit tests:

### game-world tests
- `test_pathfinding_straight_line`: Basic path finding
- `test_pathfinding_with_obstacles`: Obstacle avoidance
- `test_pathfinding_no_path`: Unreachable destinations
- `test_tile_walkability`: Terrain type rules
- `test_movement_cost`: Cost calculations

### game-units tests
- `test_world_grid_conversion`: Coordinate conversions
- `test_is_path_clear`: Line-of-sight checking
- `test_smooth_path_simple`: Path optimization
- `test_smooth_path_with_obstacles`: Safe path smoothing

Run tests with:
```bash
cargo test -p game-world map::tests
cargo test -p game-units pathfinding_integration::tests
```

## Future Enhancements

Possible improvements:
- Hierarchical pathfinding for long distances
- Jump point search for grid optimization
- Flow fields for large groups
- NavMesh for more complex 3D environments
- Cached path segments for common routes
- Formation-aware pathfinding

## Integration Points

The pathfinding system integrates with:

1. **GameMap** resource - Tile data and terrain info
2. **PathfindingGrid** resource - Walkability and costs
3. **MovementController** component - Unit waypoints
4. **Physics system** - Actual unit movement
5. **Obstacle detection** - Dynamic path updates

## Example: Setting Up Pathfinding

```rust
use game_world::GameWorldPlugin;
use game_units::GameUnitsPlugin;

app
    .add_plugins(GameWorldPlugin)      // Adds pathfinding resources
    .add_plugins(GameUnitsPlugin);     // Adds pathfinding integration
```

The plugins automatically set up all necessary systems and resources.
