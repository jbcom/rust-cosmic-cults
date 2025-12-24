//! Production fog of war system for Cosmic Dominion

use bevy::prelude::*;
use std::collections::HashMap;

/// Component marking an entity as having fog of war applied
#[derive(Component)]
pub struct FogOfWar {
    pub revealed: bool,
    pub visible: bool,
    pub last_seen_time: f32,
}

impl Default for FogOfWar {
    fn default() -> Self {
        Self {
            revealed: false,
            visible: false,
            last_seen_time: 0.0,
        }
    }
}

/// State of visibility for a tile
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum VisibilityState {
    Hidden,   // Never seen
    Revealed, // Seen before but not currently visible
    Visible,  // Currently visible
}

/// Resource storing the visibility map for the entire game world
#[derive(Resource, Default)]
pub struct VisibilityMap {
    pub tiles: HashMap<(i32, i32), VisibilityState>,
    pub sight_blockers: HashMap<(i32, i32), bool>,
}

/// Component marking an entity as a vision provider (units, buildings)
#[derive(Component)]
pub struct VisionProvider {
    pub sight_range: f32,
    pub faction: Faction,
}

/// Faction identifier for vision sharing
#[derive(Component, Clone, Copy, Debug, PartialEq)]
pub enum Faction {
    Player,
    Enemy,
    Neutral,
}

/// Component for fog overlay entities
#[derive(Component)]
pub struct FogOverlay {
    pub tile_x: i32,
    pub tile_z: i32,
}

/// Initialize fog of war system
pub fn initialize_fog_system(
    mut commands: Commands,
    mut visibility_map: ResMut<VisibilityMap>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Initialize visibility map for the game area
    let map_radius = 8;

    for x in -map_radius..=map_radius {
        for z in -map_radius..=map_radius {
            // Start with everything hidden except the immediate starting area
            let distance = ((x * x + z * z) as f32).sqrt();
            let initial_state = if distance < 2.0 {
                VisibilityState::Visible
            } else {
                VisibilityState::Hidden
            };

            visibility_map.tiles.insert((x, z), initial_state);

            // Create fog overlay for this tile if not initially visible
            if initial_state != VisibilityState::Visible {
                spawn_fog_overlay(
                    &mut commands,
                    &mut meshes,
                    &mut materials,
                    x,
                    z,
                    initial_state,
                );
            }
        }
    }
}

/// Spawn a fog overlay for a tile
fn spawn_fog_overlay(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    tile_x: i32,
    tile_z: i32,
    visibility_state: VisibilityState,
) {
    let tile_size = 10.0;
    let fog_height = 5.0; // Height of fog overlay above terrain

    // Create fog mesh (plane above the tile)
    let fog_mesh = meshes.add(Plane3d::new(Vec3::Y, Vec2::splat(tile_size * 0.5)).mesh());

    // Material based on visibility state
    let fog_material = match visibility_state {
        VisibilityState::Hidden => materials.add(StandardMaterial {
            base_color: Color::srgba(0.0, 0.0, 0.0, 0.95), // Nearly opaque black
            alpha_mode: AlphaMode::Blend,
            double_sided: true,
            cull_mode: None,
            ..default()
        }),
        VisibilityState::Revealed => materials.add(StandardMaterial {
            base_color: Color::srgba(0.1, 0.1, 0.15, 0.5), // Semi-transparent dark gray
            alpha_mode: AlphaMode::Blend,
            double_sided: true,
            cull_mode: None,
            ..default()
        }),
        VisibilityState::Visible => materials.add(StandardMaterial {
            base_color: Color::srgba(0.0, 0.0, 0.0, 0.0), // Fully transparent
            alpha_mode: AlphaMode::Blend,
            double_sided: true,
            cull_mode: None,
            ..default()
        }),
    };

    commands.spawn((
        Mesh3d(fog_mesh),
        MeshMaterial3d(fog_material),
        Transform::from_xyz(
            tile_x as f32 * tile_size,
            fog_height,
            tile_z as f32 * tile_size,
        ),
        FogOverlay { tile_x, tile_z },
        FogOfWar {
            revealed: visibility_state != VisibilityState::Hidden,
            visible: visibility_state == VisibilityState::Visible,
            last_seen_time: 0.0,
        },
    ));
}

/// Update fog of war based on vision providers
pub fn update_fog_system(
    mut visibility_map: ResMut<VisibilityMap>,
    vision_providers: Query<(&Transform, &VisionProvider)>,
    mut fog_overlays: Query<(
        &FogOverlay,
        &mut FogOfWar,
        &MeshMaterial3d<StandardMaterial>,
    )>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    time: Res<Time>,
) {
    let tile_size = 10.0;

    // Reset all tiles to not visible (but keep revealed state)
    for (_, state) in visibility_map.tiles.iter_mut() {
        if *state == VisibilityState::Visible {
            *state = VisibilityState::Revealed;
        }
    }

    // Update visibility based on vision providers
    for (transform, vision_provider) in vision_providers.iter() {
        // Only process player faction vision
        if vision_provider.faction != Faction::Player {
            continue;
        }

        let provider_tile_x = (transform.translation.x / tile_size).round() as i32;
        let provider_tile_z = (transform.translation.z / tile_size).round() as i32;

        // Calculate visible tiles using line of sight
        let sight_range_tiles = (vision_provider.sight_range / tile_size).ceil() as i32;

        for dx in -sight_range_tiles..=sight_range_tiles {
            for dz in -sight_range_tiles..=sight_range_tiles {
                let tile_x = provider_tile_x + dx;
                let tile_z = provider_tile_z + dz;

                // Check if within sight range
                let distance = ((dx * dx + dz * dz) as f32).sqrt() * tile_size;
                if distance > vision_provider.sight_range {
                    continue;
                }

                // Check line of sight
                if has_line_of_sight(
                    provider_tile_x,
                    provider_tile_z,
                    tile_x,
                    tile_z,
                    &visibility_map.sight_blockers,
                ) {
                    visibility_map
                        .tiles
                        .insert((tile_x, tile_z), VisibilityState::Visible);
                }
            }
        }
    }

    // Update fog overlay visuals
    for (fog_overlay, mut fog, material_handle) in fog_overlays.iter_mut() {
        if let Some(&visibility_state) = visibility_map
            .tiles
            .get(&(fog_overlay.tile_x, fog_overlay.tile_z))
        {
            // Update fog component
            let was_visible = fog.visible;
            fog.revealed = visibility_state != VisibilityState::Hidden;
            fog.visible = visibility_state == VisibilityState::Visible;

            if was_visible && !fog.visible {
                fog.last_seen_time = time.elapsed_seconds();
            }

            // Update material based on new visibility state
            if let Some(material) = materials.get_mut(material_handle) {
                material.base_color = match visibility_state {
                    VisibilityState::Hidden => Color::srgba(0.0, 0.0, 0.0, 0.95),
                    VisibilityState::Revealed => Color::srgba(0.1, 0.1, 0.15, 0.5),
                    VisibilityState::Visible => Color::srgba(0.0, 0.0, 0.0, 0.0),
                };
            }
        }
    }
}

/// Check if there's line of sight between two tiles
fn has_line_of_sight(
    x1: i32,
    z1: i32,
    x2: i32,
    z2: i32,
    sight_blockers: &HashMap<(i32, i32), bool>,
) -> bool {
    // Bresenham's line algorithm for line of sight
    let dx = (x2 - x1).abs();
    let dz = (z2 - z1).abs();
    let sx = if x1 < x2 { 1 } else { -1 };
    let sz = if z1 < z2 { 1 } else { -1 };

    let mut x = x1;
    let mut z = z1;
    let mut err = dx - dz;

    loop {
        // Check if current tile blocks sight (skip the starting tile)
        if (x != x1 || z != z1) && sight_blockers.get(&(x, z)).copied().unwrap_or(false) {
            return false;
        }

        if x == x2 && z == z2 {
            break;
        }

        let e2 = 2 * err;

        if e2 > -dz {
            err -= dz;
            x += sx;
        }

        if e2 < dx {
            err += dx;
            z += sz;
        }
    }

    true
}

/// System to reveal tiles around newly spawned units
pub fn reveal_around_spawn_system(
    mut visibility_map: ResMut<VisibilityMap>,
    new_vision_providers: Query<(&Transform, &VisionProvider), Added<VisionProvider>>,
) {
    let tile_size = 10.0;

    for (transform, vision_provider) in new_vision_providers.iter() {
        if vision_provider.faction != Faction::Player {
            continue;
        }

        let provider_tile_x = (transform.translation.x / tile_size).round() as i32;
        let provider_tile_z = (transform.translation.z / tile_size).round() as i32;

        // Immediately reveal tiles in range
        let sight_range_tiles = (vision_provider.sight_range / tile_size).ceil() as i32;

        for dx in -sight_range_tiles..=sight_range_tiles {
            for dz in -sight_range_tiles..=sight_range_tiles {
                let tile_x = provider_tile_x + dx;
                let tile_z = provider_tile_z + dz;

                let distance = ((dx * dx + dz * dz) as f32).sqrt() * tile_size;
                if distance <= vision_provider.sight_range {
                    visibility_map
                        .tiles
                        .insert((tile_x, tile_z), VisibilityState::Visible);
                }
            }
        }
    }
}

/// System to handle entities entering/exiting fog
pub fn fog_entity_visibility_system(
    visibility_map: Res<VisibilityMap>,
    mut entities_with_fog: Query<(&Transform, &mut Visibility), With<FogOfWar>>,
) {
    let tile_size = 10.0;

    for (transform, mut visibility) in entities_with_fog.iter_mut() {
        let tile_x = (transform.translation.x / tile_size).round() as i32;
        let tile_z = (transform.translation.z / tile_size).round() as i32;

        // Check visibility state of the tile this entity is on
        if let Some(&visibility_state) = visibility_map.tiles.get(&(tile_x, tile_z)) {
            // Hide entities in fog
            *visibility = match visibility_state {
                VisibilityState::Visible => Visibility::Visible,
                VisibilityState::Revealed => Visibility::Hidden, // Could show ghosts here
                VisibilityState::Hidden => Visibility::Hidden,
            };
        }
    }
}
