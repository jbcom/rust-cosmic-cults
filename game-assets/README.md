# Game Assets

Centralized asset loading and registry system for Cosmic Cults.

## Overview

This crate provides a unified interface for loading and accessing all game assets including:
- Unit models (GLB scenes)
- Leader models (GLB scenes)
- Common procedural meshes (UI elements, effects)
- Material registry for reusable materials

## Features

- **Centralized Management**: Single source of truth for all game assets
- **Organized Structure**: Assets organized by category and cult affiliation
- **Type-Safe Access**: Strongly-typed methods for retrieving assets
- **Lazy Loading**: Assets loaded asynchronously via Bevy's asset server
- **Extensible**: Easy to add new asset categories

## Usage

### Basic Setup

Add the `init_asset_registry` system to your app's `Startup` schedule:

```rust
use game_assets::init_asset_registry;

app.add_systems(Startup, init_asset_registry);
```

### Accessing Assets

Access the registry through the `AssetRegistry` resource:

```rust
use game_assets::AssetRegistry;

fn spawn_unit_system(
    mut commands: Commands,
    registry: Res<AssetRegistry>,
) {
    // Get a unit scene
    if let Some(scene) = registry.get_unit_scene("crimson_covenant", "cultist") {
        commands.spawn(SceneRoot(scene));
    }
    
    // Get a leader scene
    if let Some(scene) = registry.get_leader_scene("deep_ones") {
        commands.spawn(SceneRoot(scene));
    }
    
    // Access common meshes
    let selection_ring = registry.common_meshes.selection_ring.clone();
    commands.spawn(Mesh3d(selection_ring));
}
```

### Material Registry

Register and reuse materials for consistent visuals:

```rust
use game_assets::AssetRegistry;

fn setup_materials(
    mut registry: ResMut<AssetRegistry>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let red_material = materials.add(StandardMaterial {
        base_color: Color::srgb(1.0, 0.0, 0.0),
        ..default()
    });
    
    registry.materials.register("enemy_red", red_material);
}

fn use_material(
    registry: Res<AssetRegistry>,
) {
    if let Some(material) = registry.materials.get("enemy_red") {
        // Use the material
    }
}
```

## Asset Organization

### Unit Scenes

Unit models are organized by cult:
- **Crimson Covenant**: `crimson_covenant`
  - Cultist/Acolyte
  - Warrior
  - Berserker
- **Deep Ones**: `deep_ones`
  - Cultist
  - Guardian
  - Horror
- **Void Seekers**: `void_seekers`
  - Scout/Initiate
  - Assassin
  - Harbinger

### Leader Scenes

Leader models by cult:
- `crimson_covenant` → Blood Lord
- `deep_ones` → Deep Priest
- `void_seekers` → Void Scholar

### Common Meshes

Procedural meshes for UI and effects:
- `selection_ring` - Torus for selection indicators
- `health_bar_background` - Health bar background
- `health_bar_fill` - Health bar fill
- `aura_sphere` - Sphere for aura effects
- `leader_platform` - Cylinder for leader platforms
- `veteran_star` - Small sphere for veteran indicators

## Architecture

The registry is organized into sub-structures for better organization:

```
AssetRegistry
├── unit_scenes: UnitScenes
│   ├── crimson: CrimsonUnits
│   ├── deep: DeepUnits
│   └── void: VoidUnits
├── leader_scenes: LeaderScenes
├── common_meshes: CommonMeshes
└── materials: MaterialRegistry
```

## Extension

To add new assets:

1. Add new fields to the appropriate structure
2. Update the `load()` method to load the assets
3. Update the `get()` method to return the assets
4. Add tests for the new functionality

Example:

```rust
pub struct CrimsonUnits {
    pub acolyte: Handle<Scene>,
    pub warrior: Handle<Scene>,
    pub berserker: Handle<Scene>,
    pub new_unit: Handle<Scene>,  // New field
}

impl CrimsonUnits {
    fn load(asset_server: &AssetServer) -> Self {
        Self {
            // ... existing loads
            new_unit: asset_server.load("assets/models/units/crimson/new_unit.glb#Scene0"),
        }
    }
}
```

## Testing

Run tests with:

```bash
cargo test -p game-assets
```

Tests verify:
- Unit scene retrieval
- Leader scene retrieval
- Material registry operations
- Fallback behavior for invalid keys
