//! Map management and grid system for Cosmic Dominion

use bevy::prelude::*;
use std::collections::HashMap;
use tracing::info;

/// Resource representing the game map
#[derive(Resource)]
pub struct GameMap {
    pub width: i32,
    pub height: i32,
    pub starting_position: (i32, i32),
    pub tile_size: f32,
    pub tiles: HashMap<(i32, i32), TileInfo>,
}

impl Default for GameMap {
    fn default() -> Self {
        Self {
            width: 17,  // -8 to 8
            height: 17, // -8 to 8
            starting_position: (0, 0),
            tile_size: 10.0,
            tiles: HashMap::new(),
        }
    }
}

/// Information about a single map tile
#[derive(Clone, Debug)]
pub struct TileInfo {
    pub position: (i32, i32),
    pub tile_type: TileType,
    pub occupied: bool,
    pub corruption_level: f32,
    pub height: f32,
}

/// Types of tiles in the game world
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum TileType {
    Ground,
    Water,
    Cliff,
    Bridge,
    Void,
}

/// Component for tile entities
#[derive(Component)]
pub struct MapTile {
    pub grid_x: i32,
    pub grid_z: i32,
    pub tile_type: TileType,
}

/// Resource for pathfinding and movement
#[derive(Resource, Default)]
pub struct PathfindingGrid {
    pub walkable: HashMap<(i32, i32), bool>,
    pub movement_costs: HashMap<(i32, i32), f32>,
}

/// Initialize the game map
pub fn initialize_map(
    _commands: Commands,
    mut game_map: ResMut<GameMap>,
    mut pathfinding_grid: ResMut<PathfindingGrid>,
) {
    info!(
        "Initializing game map with {}x{} tiles",
        game_map.width, game_map.height
    );

    let half_width = game_map.width / 2;
    let half_height = game_map.height / 2;

    // Initialize all tiles
    for x in -half_width..=half_width {
        for z in -half_height..=half_height {
            let distance_from_center = ((x * x + z * z) as f32).sqrt();

            // Determine tile type based on position
            let tile_type = determine_tile_type(x, z, distance_from_center);

            // Calculate corruption level
            let corruption_level = calculate_tile_corruption(distance_from_center);

            // Determine if walkable
            let walkable = is_tile_walkable(tile_type, corruption_level);

            // Calculate movement cost
            let movement_cost = calculate_movement_cost(tile_type, corruption_level);

            // Store tile info
            let tile_info = TileInfo {
                position: (x, z),
                tile_type,
                occupied: false,
                corruption_level,
                height: 0.0, // Will be set by terrain generation
            };

            game_map.tiles.insert((x, z), tile_info);
            pathfinding_grid.walkable.insert((x, z), walkable);
            pathfinding_grid
                .movement_costs
                .insert((x, z), movement_cost);
        }
    }

    // Mark starting area as safe
    for dx in -1..=1 {
        for dz in -1..=1 {
            if let Some(tile) = game_map.tiles.get_mut(&(dx, dz)) {
                tile.corruption_level = 0.0;
            }
            pathfinding_grid.walkable.insert((dx, dz), true);
            pathfinding_grid.movement_costs.insert((dx, dz), 1.0);
        }
    }
}

/// Determine tile type based on position
fn determine_tile_type(x: i32, z: i32, distance: f32) -> TileType {
    // Create some interesting patterns
    if distance > 7.0 {
        // Outer ring might have void tiles
        if (x + z) % 5 == 0 {
            return TileType::Void;
        }
    }

    // Create water features
    if ((x * 3 + z * 2) % 7 == 0) && distance > 3.0 {
        return TileType::Water;
    }

    // Create cliff areas
    if ((x.abs() + z.abs()) % 8 == 0) && distance > 2.0 {
        return TileType::Cliff;
    }

    // Default to ground
    TileType::Ground
}

/// Calculate corruption level for a tile
fn calculate_tile_corruption(distance: f32) -> f32 {
    // Corruption increases with distance from center
    let base_corruption = (distance / 10.0).min(1.0);

    // Add some variation
    let variation = ((distance * 0.5).sin() * 0.1).abs();

    (base_corruption + variation).min(1.0)
}

/// Check if a tile is walkable
fn is_tile_walkable(tile_type: TileType, corruption_level: f32) -> bool {
    match tile_type {
        TileType::Ground => true,
        TileType::Bridge => true,
        TileType::Water => false,
        TileType::Cliff => false,
        TileType::Void => corruption_level < 0.9, // Can walk on less corrupted void
    }
}

/// Calculate movement cost for pathfinding
fn calculate_movement_cost(tile_type: TileType, corruption_level: f32) -> f32 {
    let base_cost = match tile_type {
        TileType::Ground => 1.0,
        TileType::Bridge => 1.2,
        TileType::Water => 999.0, // Very high cost (not walkable)
        TileType::Cliff => 999.0, // Very high cost (not walkable)
        TileType::Void => 2.0 + corruption_level * 3.0,
    };

    // Corruption increases movement cost
    base_cost * (1.0 + corruption_level * 0.5)
}

/// Get world position from grid coordinates
pub fn grid_to_world(x: i32, z: i32, tile_size: f32) -> Vec3 {
    Vec3::new(x as f32 * tile_size, 0.0, z as f32 * tile_size)
}

/// Get grid coordinates from world position
pub fn world_to_grid(position: Vec3, tile_size: f32) -> (i32, i32) {
    let x = (position.x / tile_size).round() as i32;
    let z = (position.z / tile_size).round() as i32;
    (x, z)
}

/// System to update tile occupation based on entities
pub fn update_tile_occupation_system(
    mut game_map: ResMut<GameMap>,
    occupants: Query<&Transform, (With<MapTile>, Changed<Transform>)>,
) {
    // Clear all occupation flags
    for tile in game_map.tiles.values_mut() {
        tile.occupied = false;
    }

    // Mark occupied tiles
    for transform in occupants.iter() {
        let (x, z) = world_to_grid(transform.translation, game_map.tile_size);
        if let Some(tile) = game_map.tiles.get_mut(&(x, z)) {
            tile.occupied = true;
        }
    }
}

/// Find a path between two points using A* pathfinding
pub fn find_path(
    start: (i32, i32),
    goal: (i32, i32),
    pathfinding_grid: &PathfindingGrid,
) -> Option<Vec<(i32, i32)>> {
    use std::cmp::Ordering;
    use std::collections::{BinaryHeap, HashSet};

    #[derive(Clone, Eq, PartialEq)]
    struct Node {
        position: (i32, i32),
        cost: i32,
        heuristic: i32,
    }

    impl Ord for Node {
        fn cmp(&self, other: &Self) -> Ordering {
            (other.cost + other.heuristic).cmp(&(self.cost + self.heuristic))
        }
    }

    impl PartialOrd for Node {
        fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
            Some(self.cmp(other))
        }
    }

    let mut open_set = BinaryHeap::new();
    let mut closed_set = HashSet::new();
    let mut came_from = HashMap::new();
    let mut g_score = HashMap::new();

    // Add starting node
    open_set.push(Node {
        position: start,
        cost: 0,
        heuristic: manhattan_distance(start, goal),
    });
    g_score.insert(start, 0);

    while let Some(current) = open_set.pop() {
        if current.position == goal {
            // Reconstruct path
            let mut path = Vec::new();
            let mut current_pos = goal;

            while current_pos != start {
                path.push(current_pos);
                if let Some(&prev) = came_from.get(&current_pos) {
                    current_pos = prev;
                } else {
                    break;
                }
            }
            path.push(start);
            path.reverse();

            return Some(path);
        }

        if closed_set.contains(&current.position) {
            continue;
        }
        closed_set.insert(current.position);

        // Check neighbors
        for &(dx, dz) in &[
            (0, 1),
            (1, 0),
            (0, -1),
            (-1, 0),
            (1, 1),
            (-1, 1),
            (1, -1),
            (-1, -1),
        ] {
            let neighbor = (current.position.0 + dx, current.position.1 + dz);

            // Check if walkable
            if !pathfinding_grid
                .walkable
                .get(&neighbor)
                .copied()
                .unwrap_or(false)
            {
                continue;
            }

            let tentative_g_score = g_score[&current.position]
                + (pathfinding_grid
                    .movement_costs
                    .get(&neighbor)
                    .copied()
                    .unwrap_or(999.0)
                    * 100.0) as i32;

            if tentative_g_score < *g_score.get(&neighbor).unwrap_or(&i32::MAX) {
                came_from.insert(neighbor, current.position);
                g_score.insert(neighbor, tentative_g_score);

                open_set.push(Node {
                    position: neighbor,
                    cost: tentative_g_score,
                    heuristic: manhattan_distance(neighbor, goal),
                });
            }
        }
    }

    None
}

/// Calculate Manhattan distance for pathfinding heuristic
fn manhattan_distance(a: (i32, i32), b: (i32, i32)) -> i32 {
    ((a.0 - b.0).abs() + (a.1 - b.1).abs()) * 100
}

/// Debug system to visualize the map grid
pub fn debug_draw_map_grid(game_map: Res<GameMap>, mut gizmos: Gizmos) {
    let half_width = game_map.width / 2;
    let half_height = game_map.height / 2;

    // Draw grid lines
    for x in -half_width..=half_width {
        let start = grid_to_world(x, -half_height, game_map.tile_size);
        let end = grid_to_world(x, half_height, game_map.tile_size);
        gizmos.line(start, end, Color::srgba(0.2, 0.2, 0.2, 0.3));
    }

    for z in -half_height..=half_height {
        let start = grid_to_world(-half_width, z, game_map.tile_size);
        let end = grid_to_world(half_width, z, game_map.tile_size);
        gizmos.line(start, end, Color::srgba(0.2, 0.2, 0.2, 0.3));
    }

    // Highlight starting position
    let start_pos = grid_to_world(
        game_map.starting_position.0,
        game_map.starting_position.1,
        game_map.tile_size,
    );
    gizmos.circle(
        start_pos + Vec3::Y * 0.1,
        game_map.tile_size * 0.4,
        Color::srgb(0.0, 1.0, 0.0),
    );
}
