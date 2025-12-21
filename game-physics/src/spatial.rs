use ahash::HashMap;
use bevy::prelude::*;
use std::collections::HashSet;
use wasm_bindgen::prelude::*;

// ==============================================================================
// SPATIAL INDEXING SYSTEM
// ==============================================================================

/// High-performance spatial grid for entity queries and collision detection
pub struct SpatialGrid {
    pub cell_size: f32,
    pub cells: HashMap<(i32, i32), Vec<Entity>>,
    pub entity_positions: HashMap<Entity, (i32, i32)>,
}

impl SpatialGrid {
    pub fn new(cell_size: f32) -> Self {
        Self {
            cell_size,
            cells: HashMap::default(),
            entity_positions: HashMap::default(),
        }
    }

    /// Clear all entities from the grid
    pub fn clear(&mut self) {
        self.cells.clear();
        self.entity_positions.clear();
    }

    /// Insert an entity at a given position
    pub fn insert(&mut self, entity: Entity, position: Vec3) {
        let cell_x = (position.x / self.cell_size).floor() as i32;
        let cell_z = (position.z / self.cell_size).floor() as i32;
        let cell_coord = (cell_x, cell_z);

        // Remove entity from old cell if it exists
        if let Some(old_coord) = self.entity_positions.get(&entity) {
            if let Some(old_cell) = self.cells.get_mut(old_coord) {
                old_cell.retain(|&e| e != entity);
                if old_cell.is_empty() {
                    self.cells.remove(old_coord);
                }
            }
        }

        // Insert into new cell
        self.cells.entry(cell_coord).or_default().push(entity);

        self.entity_positions.insert(entity, cell_coord);
    }

    /// Remove an entity from the grid
    pub fn remove(&mut self, entity: Entity) {
        if let Some(cell_coord) = self.entity_positions.remove(&entity) {
            if let Some(cell) = self.cells.get_mut(&cell_coord) {
                cell.retain(|&e| e != entity);
                if cell.is_empty() {
                    self.cells.remove(&cell_coord);
                }
            }
        }
    }

    /// Query all entities within a radius of a position
    pub fn query_range(&self, position: Vec3, radius: f32) -> Vec<Entity> {
        let mut results = Vec::new();

        let min_x = ((position.x - radius) / self.cell_size).floor() as i32;
        let max_x = ((position.x + radius) / self.cell_size).ceil() as i32;
        let min_z = ((position.z - radius) / self.cell_size).floor() as i32;
        let max_z = ((position.z + radius) / self.cell_size).ceil() as i32;

        for x in min_x..=max_x {
            for z in min_z..=max_z {
                if let Some(entities) = self.cells.get(&(x, z)) {
                    results.extend(entities.iter().cloned());
                }
            }
        }

        results
    }

    /// Query entities in a specific cell
    pub fn query_cell(&self, cell_x: i32, cell_z: i32) -> Vec<Entity> {
        self.cells
            .get(&(cell_x, cell_z))
            .cloned()
            .unwrap_or_default()
    }

    /// Get all entities in the grid
    pub fn get_all_entities(&self) -> Vec<Entity> {
        self.cells.values().flatten().cloned().collect()
    }

    /// Get the cell coordinate for a world position
    pub fn world_to_cell(&self, position: Vec3) -> (i32, i32) {
        (
            (position.x / self.cell_size).floor() as i32,
            (position.z / self.cell_size).floor() as i32,
        )
    }

    /// Get the world bounds of a cell
    pub fn cell_to_world_bounds(&self, cell_x: i32, cell_z: i32) -> (Vec3, Vec3) {
        let min = Vec3::new(
            cell_x as f32 * self.cell_size,
            0.0,
            cell_z as f32 * self.cell_size,
        );
        let max = Vec3::new(
            (cell_x + 1) as f32 * self.cell_size,
            0.0,
            (cell_z + 1) as f32 * self.cell_size,
        );
        (min, max)
    }
}

/// Resource for managing the global spatial grid
#[derive(Resource)]
pub struct GlobalSpatialGrid {
    pub grid: SpatialGrid,
    pub needs_update: bool,
}

impl GlobalSpatialGrid {
    pub fn new(cell_size: f32) -> Self {
        Self {
            grid: SpatialGrid::new(cell_size),
            needs_update: true,
        }
    }
}

impl Default for GlobalSpatialGrid {
    fn default() -> Self {
        Self::new(10.0) // 10 unit cells by default
    }
}

/// WASM-optimized spatial grid for performance-critical operations
#[wasm_bindgen]
pub struct WasmSpatialGrid {
    cell_size: f32,
    cells: HashMap<(i32, i32), Vec<u32>>, // Using u32 IDs instead of entities
}

#[wasm_bindgen]
impl WasmSpatialGrid {
    #[wasm_bindgen(constructor)]
    pub fn new(cell_size: f32) -> Self {
        Self {
            cell_size,
            cells: HashMap::default(),
        }
    }

    /// Clear all entities from the grid
    #[wasm_bindgen]
    pub fn clear(&mut self) {
        self.cells.clear();
    }

    /// Insert an entity by ID at a position
    #[wasm_bindgen]
    pub fn insert(&mut self, entity_id: u32, x: f32, z: f32) {
        let cell_x = (x / self.cell_size).floor() as i32;
        let cell_z = (z / self.cell_size).floor() as i32;

        self.cells
            .entry((cell_x, cell_z))
            .or_default()
            .push(entity_id);
    }

    /// Query entities within radius (returns packed array)
    #[wasm_bindgen]
    pub fn query_range(&self, x: f32, z: f32, radius: f32) -> Vec<u32> {
        let mut results = Vec::new();

        let min_x = ((x - radius) / self.cell_size).floor() as i32;
        let max_x = ((x + radius) / self.cell_size).ceil() as i32;
        let min_z = ((z - radius) / self.cell_size).floor() as i32;
        let max_z = ((z + radius) / self.cell_size).ceil() as i32;

        for cell_x in min_x..=max_x {
            for cell_z in min_z..=max_z {
                if let Some(entities) = self.cells.get(&(cell_x, cell_z)) {
                    results.extend(entities.iter().cloned());
                }
            }
        }

        results
    }
}

/// Proximity detector for efficient neighbor queries
#[derive(Component, Clone, Debug)]
pub struct ProximityDetector {
    pub detection_radius: f32,
    pub detected_entities: HashSet<Entity>,
    pub last_update: f32,
}

impl ProximityDetector {
    pub fn new(radius: f32) -> Self {
        Self {
            detection_radius: radius,
            detected_entities: HashSet::new(),
            last_update: 0.0,
        }
    }
}

/// Spatial hash for very fast lookups
pub struct SpatialHash {
    pub grid_size: f32,
    pub hash_map: HashMap<i64, Vec<Entity>>,
}

impl SpatialHash {
    pub fn new(grid_size: f32) -> Self {
        Self {
            grid_size,
            hash_map: HashMap::default(),
        }
    }

    /// Hash a 2D position to a single integer key
    fn hash_position(&self, x: f32, z: f32) -> i64 {
        let grid_x = (x / self.grid_size).floor() as i32;
        let grid_z = (z / self.grid_size).floor() as i32;

        // Simple spatial hash using bit shifting
        ((grid_x as i64) << 32) | (grid_z as i64 & 0xFFFFFFFF)
    }

    pub fn insert(&mut self, entity: Entity, position: Vec3) {
        let key = self.hash_position(position.x, position.z);
        self.hash_map.entry(key).or_default().push(entity);
    }

    pub fn query_cell(&self, position: Vec3) -> Vec<Entity> {
        let key = self.hash_position(position.x, position.z);
        self.hash_map.get(&key).cloned().unwrap_or_default()
    }

    pub fn clear(&mut self) {
        self.hash_map.clear();
    }
}

/// Broad phase collision detection using spatial partitioning
#[derive(Resource, Default)]
pub struct BroadPhaseCollisionPairs {
    pub pairs: Vec<(Entity, Entity)>,
}

impl BroadPhaseCollisionPairs {
    pub fn clear(&mut self) {
        self.pairs.clear();
    }

    pub fn add_pair(&mut self, entity_a: Entity, entity_b: Entity) {
        // Ensure consistent ordering
        let pair = if entity_a.index() < entity_b.index() {
            (entity_a, entity_b)
        } else {
            (entity_b, entity_a)
        };

        if !self.pairs.contains(&pair) {
            self.pairs.push(pair);
        }
    }
}
