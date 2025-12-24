//! Production terrain generation and biome system for Cosmic Dominion

use bevy::asset::RenderAssetUsages;
use bevy::mesh::Indices;
use bevy::prelude::*;
use bevy::render::render_resource::PrimitiveTopology;
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use std::collections::HashMap;

/// Terrain tile component representing a single tile in the game world
#[derive(Component)]
pub struct TerrainTile {
    pub x: i32,
    pub z: i32,
    pub biome: BiomeType,
    pub walkable: bool,
    pub corruption_level: f32,
}

/// Biome types in the game world
#[derive(Component, Clone, Copy, Debug, PartialEq)]
pub enum BiomeType {
    CorruptedForest,
    BloodPlains,
    VoidRift,
    DeepMarsh,
    Wasteland,
    NeutralGround,
}

impl BiomeType {
    /// Get the base color for this biome
    pub fn get_base_color(&self) -> Color {
        match self {
            BiomeType::CorruptedForest => Color::srgb(0.2, 0.1, 0.3), // Dark purple
            BiomeType::BloodPlains => Color::srgb(0.4, 0.1, 0.1),     // Dark red
            BiomeType::VoidRift => Color::srgb(0.05, 0.0, 0.15),      // Very dark blue
            BiomeType::DeepMarsh => Color::srgb(0.1, 0.2, 0.15),      // Dark green
            BiomeType::Wasteland => Color::srgb(0.25, 0.2, 0.15),     // Brown-gray
            BiomeType::NeutralGround => Color::srgb(0.3, 0.35, 0.3),  // Gray-green
        }
    }

    /// Get the emissive color for this biome
    pub fn get_emissive_color(&self) -> Color {
        match self {
            BiomeType::CorruptedForest => Color::srgb(0.3, 0.0, 0.5),
            BiomeType::BloodPlains => Color::srgb(0.5, 0.0, 0.0),
            BiomeType::VoidRift => Color::srgb(0.0, 0.0, 0.3),
            BiomeType::DeepMarsh => Color::srgb(0.0, 0.1, 0.05),
            BiomeType::Wasteland => Color::BLACK,
            BiomeType::NeutralGround => Color::BLACK,
        }
    }

    /// Get the height variation for this biome
    pub fn get_height_variation(&self) -> f32 {
        match self {
            BiomeType::CorruptedForest => 0.5,
            BiomeType::BloodPlains => 0.2,
            BiomeType::VoidRift => 1.0,
            BiomeType::DeepMarsh => 0.3,
            BiomeType::Wasteland => 0.4,
            BiomeType::NeutralGround => 0.1,
        }
    }
}

/// Resource for terrain generation configuration
#[derive(Resource)]
pub struct TerrainConfig {
    pub tile_size: f32,
    pub chunk_size: i32,
    pub seed: u64,
}

impl Default for TerrainConfig {
    fn default() -> Self {
        Self {
            tile_size: 10.0,
            chunk_size: 16,
            seed: 42,
        }
    }
}

/// Generate the game world terrain
pub fn generate_terrain_system(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    terrain_config: Res<TerrainConfig>,
) {
    let mut rng = StdRng::seed_from_u64(terrain_config.seed);

    // Generate a 3x3 starting area with surrounding terrain
    let _start_radius = 5; // 11x11 grid centered at origin
    let fog_radius = 8; // Additional fog tiles beyond visible area

    // Track generated tiles for biome clustering
    let mut tile_biomes: HashMap<(i32, i32), BiomeType> = HashMap::new();

    // First pass: assign biomes with clustering
    for x in -fog_radius..=fog_radius {
        for z in -fog_radius..=fog_radius {
            let distance_from_center = ((x * x + z * z) as f32).sqrt();

            // Center area is always neutral ground for starting
            let biome = if distance_from_center < 2.0 {
                BiomeType::NeutralGround
            } else {
                // Check neighboring tiles for biome clustering
                let mut neighbor_biomes = Vec::new();
                for dx in -1..=1 {
                    for dz in -1..=1 {
                        if dx == 0 && dz == 0 {
                            continue;
                        }
                        if let Some(&neighbor_biome) = tile_biomes.get(&(x + dx, z + dz)) {
                            neighbor_biomes.push(neighbor_biome);
                        }
                    }
                }

                // Weighted biome selection based on neighbors
                if !neighbor_biomes.is_empty() && rng.random::<f32>() < 0.7 {
                    // 70% chance to match a neighbor for clustering
                    neighbor_biomes[rng.random_range(0..neighbor_biomes.len())]
                } else {
                    // Random biome selection weighted by distance
                    select_biome_by_distance(distance_from_center, &mut rng)
                }
            };

            tile_biomes.insert((x, z), biome);
        }
    }

    // Second pass: create tiles with assigned biomes
    for x in -fog_radius..=fog_radius {
        for z in -fog_radius..=fog_radius {
            let biome = *tile_biomes.get(&(x, z)).unwrap();
            let distance_from_center = ((x * x + z * z) as f32).sqrt();

            // Calculate corruption level based on distance and biome
            let corruption_level =
                calculate_corruption_level(distance_from_center, biome, &mut rng);

            // Create tile mesh with height variation
            let height_variation = biome.get_height_variation();
            let tile_height = rng.random_range(-height_variation..height_variation);

            let tile_mesh = meshes.add(create_tile_mesh(
                terrain_config.tile_size,
                tile_height,
                corruption_level,
            ));

            // Create material with biome-specific colors
            let base_color = biome.get_base_color();
            let emissive = biome.get_emissive_color();

            // Add corruption effect to color
            let corrupted_color = Color::srgb(
                base_color.to_srgba().red * (1.0 - corruption_level * 0.3),
                base_color.to_srgba().green * (1.0 - corruption_level * 0.5),
                base_color.to_srgba().blue * (1.0 - corruption_level * 0.2),
            );

            let tile_material = materials.add(StandardMaterial {
                base_color: corrupted_color,
                emissive: LinearRgba::from(emissive) * corruption_level * 0.5,
                metallic: 0.0,
                perceptual_roughness: 0.9,
                ..default()
            });

            // Determine if tile is walkable
            let walkable = biome != BiomeType::VoidRift || corruption_level < 0.8;

            // Spawn tile entity
            commands.spawn((
                Mesh3d(tile_mesh),
                MeshMaterial3d(tile_material),
                Transform::from_xyz(
                    x as f32 * terrain_config.tile_size,
                    tile_height,
                    z as f32 * terrain_config.tile_size,
                ),
                TerrainTile {
                    x,
                    z,
                    biome,
                    walkable,
                    corruption_level,
                },
            ));

            // Add decorative elements based on biome
            spawn_biome_decorations(
                &mut commands,
                &mut meshes,
                &mut materials,
                x,
                z,
                biome,
                corruption_level,
                terrain_config.tile_size,
                &mut rng,
            );
        }
    }
}

/// Select a biome based on distance from center
fn select_biome_by_distance(distance: f32, rng: &mut StdRng) -> BiomeType {
    if distance < 3.0 {
        // Inner ring - safer biomes
        match rng.random_range(0..3) {
            0 => BiomeType::NeutralGround,
            1 => BiomeType::BloodPlains,
            _ => BiomeType::DeepMarsh,
        }
    } else if distance < 6.0 {
        // Middle ring - mixed biomes
        match rng.random_range(0..4) {
            0 => BiomeType::BloodPlains,
            1 => BiomeType::CorruptedForest,
            2 => BiomeType::DeepMarsh,
            _ => BiomeType::Wasteland,
        }
    } else {
        // Outer ring - dangerous biomes
        match rng.random_range(0..3) {
            0 => BiomeType::CorruptedForest,
            1 => BiomeType::VoidRift,
            _ => BiomeType::Wasteland,
        }
    }
}

/// Calculate corruption level based on distance and biome
fn calculate_corruption_level(distance: f32, biome: BiomeType, rng: &mut StdRng) -> f32 {
    let base_corruption = match biome {
        BiomeType::VoidRift => 0.8,
        BiomeType::CorruptedForest => 0.6,
        BiomeType::BloodPlains => 0.4,
        BiomeType::DeepMarsh => 0.3,
        BiomeType::Wasteland => 0.5,
        BiomeType::NeutralGround => 0.1,
    };

    // Increase corruption with distance
    let distance_factor = (distance / 10.0).min(1.0);
    let noise = rng.random_range(-0.1..0.1);

    (base_corruption + distance_factor * 0.2 + noise).clamp(0.0, 1.0)
}

/// Create a tile mesh with height variation and corruption effects
fn create_tile_mesh(size: f32, height: f32, corruption_level: f32) -> Mesh {
    let half_size = size / 2.0;

    // Add vertex displacement based on corruption
    let corruption_displacement = corruption_level * 0.5;

    // Create a plane with subdivisions for more detail
    let subdivisions = 4;
    let step = size / subdivisions as f32;

    let mut positions = Vec::new();
    let mut normals = Vec::new();
    let mut uvs = Vec::new();
    let mut indices = Vec::new();

    // Generate vertices
    for i in 0..=subdivisions {
        for j in 0..=subdivisions {
            let x = -half_size + i as f32 * step;
            let z = -half_size + j as f32 * step;

            // Add some noise to height for variation
            let local_height = height
                + (corruption_level * (x * 0.1).sin() * (z * 0.1).cos() * corruption_displacement);

            positions.push([x, local_height, z]);
            normals.push([0.0, 1.0, 0.0]);
            uvs.push([
                i as f32 / subdivisions as f32,
                j as f32 / subdivisions as f32,
            ]);
        }
    }

    // Generate indices
    for i in 0..subdivisions {
        for j in 0..subdivisions {
            let idx = i * (subdivisions + 1) + j;
            let idx_next_row = (i + 1) * (subdivisions + 1) + j;

            // First triangle
            indices.push(idx as u32);
            indices.push(idx_next_row as u32);
            indices.push((idx + 1) as u32);

            // Second triangle
            indices.push((idx + 1) as u32);
            indices.push(idx_next_row as u32);
            indices.push((idx_next_row + 1) as u32);
        }
    }

    Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    )
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
    .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
    .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, uvs)
    .with_inserted_indices(Indices::U32(indices))
}

/// Spawn decorative elements for biomes
#[allow(clippy::too_many_arguments)]
fn spawn_biome_decorations(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    tile_x: i32,
    tile_z: i32,
    biome: BiomeType,
    corruption_level: f32,
    tile_size: f32,
    rng: &mut StdRng,
) {
    // Chance to spawn decoration based on biome
    let spawn_chance = match biome {
        BiomeType::CorruptedForest => 0.3,
        BiomeType::BloodPlains => 0.1,
        BiomeType::VoidRift => 0.2,
        BiomeType::DeepMarsh => 0.2,
        BiomeType::Wasteland => 0.05,
        BiomeType::NeutralGround => 0.02,
    };

    if rng.random::<f32>() > spawn_chance {
        return;
    }

    let world_x = tile_x as f32 * tile_size;
    let world_z = tile_z as f32 * tile_size;

    match biome {
        BiomeType::CorruptedForest => {
            // Spawn twisted trees or corrupted crystals
            let crystal_mesh = meshes.add(create_crystal_mesh(rng));
            let crystal_material = materials.add(StandardMaterial {
                base_color: Color::srgb(0.5, 0.0, 0.8),
                emissive: LinearRgba::from(Color::srgb(0.3, 0.0, 0.5)) * corruption_level,
                metallic: 0.3,
                perceptual_roughness: 0.4,
                ..default()
            });

            commands.spawn((
                Mesh3d(crystal_mesh),
                MeshMaterial3d(crystal_material),
                Transform::from_xyz(
                    world_x + rng.random_range(-tile_size * 0.3..tile_size * 0.3),
                    rng.random_range(0.5..2.0),
                    world_z + rng.random_range(-tile_size * 0.3..tile_size * 0.3),
                )
                .with_scale(Vec3::splat(rng.random_range(0.5..1.5))),
            ));
        }
        BiomeType::VoidRift => {
            // Spawn void rifts or dark pillars
            let pillar_mesh = meshes.add(create_void_pillar_mesh());
            let pillar_material = materials.add(StandardMaterial {
                base_color: Color::srgb(0.0, 0.0, 0.1),
                emissive: LinearRgba::from(Color::srgb(0.0, 0.0, 0.3)) * corruption_level,
                metallic: 0.8,
                perceptual_roughness: 0.2,
                ..default()
            });

            commands.spawn((
                Mesh3d(pillar_mesh),
                MeshMaterial3d(pillar_material),
                Transform::from_xyz(
                    world_x + rng.random_range(-tile_size * 0.2..tile_size * 0.2),
                    0.0,
                    world_z + rng.random_range(-tile_size * 0.2..tile_size * 0.2),
                ),
            ));
        }
        BiomeType::DeepMarsh => {
            // Spawn marsh pools or toxic bubbles
            let pool_mesh = meshes.add(Sphere::new(rng.random_range(1.0..2.0)));
            let pool_material = materials.add(StandardMaterial {
                base_color: Color::srgba(0.1, 0.3, 0.2, 0.7),
                emissive: LinearRgba::from(Color::srgb(0.0, 0.1, 0.0)) * 0.2,
                metallic: 0.0,
                perceptual_roughness: 0.1,
                alpha_mode: AlphaMode::Blend,
                ..default()
            });

            commands.spawn((
                Mesh3d(pool_mesh),
                MeshMaterial3d(pool_material),
                Transform::from_xyz(world_x, -0.3, world_z).with_scale(Vec3::new(1.0, 0.2, 1.0)),
            ));
        }
        _ => {}
    }
}

/// Create a procedural crystal mesh
fn create_crystal_mesh(rng: &mut StdRng) -> Mesh {
    let height = rng.random_range(2.0..4.0);
    let radius = rng.random_range(0.3..0.6);
    let sides = 6;

    let mut positions = Vec::new();
    let mut normals = Vec::new();
    let mut indices = Vec::new();

    // Bottom center vertex
    positions.push([0.0, 0.0, 0.0]);
    normals.push([0.0, -1.0, 0.0]);

    // Top vertex
    positions.push([0.0, height, 0.0]);
    normals.push([0.0, 1.0, 0.0]);

    // Side vertices
    for i in 0..sides {
        let angle = (i as f32 / sides as f32) * std::f32::consts::TAU;
        let x = angle.cos() * radius;
        let z = angle.sin() * radius;

        positions.push([x, height * 0.3, z]);
        normals.push([x / radius, 0.0, z / radius]);
    }

    // Create faces
    for i in 0..sides {
        let next = (i + 1) % sides;

        // Bottom face
        indices.push(0);
        indices.push(2 + i as u32);
        indices.push(2 + next as u32);

        // Top face
        indices.push(1);
        indices.push(2 + next as u32);
        indices.push(2 + i as u32);
    }

    Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    )
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
    .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
    .with_inserted_indices(Indices::U32(indices))
}

/// Create a void pillar mesh
fn create_void_pillar_mesh() -> Mesh {
    Cuboid::new(0.5, 3.0, 0.5).into()
}
