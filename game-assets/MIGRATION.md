# Migration Guide: Using the Centralized Asset Registry

This guide explains how to migrate from scattered asset loading to using the centralized `AssetRegistry`.

## Before: Scattered Asset Loading

Previously, asset loading was done directly in each module:

```rust
// game-world/src/spawning.rs (old pattern)
fn spawn_building(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    let model = asset_server.load("assets/models/buildings/temple.glb#Scene0");
    commands.spawn(SceneRoot(model));
}
```

## After: Centralized Registry

Now, use the `AssetRegistry`:

### Step 1: Add game-assets dependency

```toml
# In your Cargo.toml
[dependencies]
game-assets = { path = "../game-assets" }
```

### Step 2: Access through the registry

```rust
use game_assets::AssetRegistry;

fn spawn_building(
    mut commands: Commands,
    registry: Res<AssetRegistry>,
) {
    // Assets are already loaded and organized
    if let Some(scene) = registry.get_unit_scene("crimson_covenant", "cultist") {
        commands.spawn(SceneRoot(scene));
    }
}
```

## Benefits

1. **Single Source of Truth**: All asset paths defined in one place
2. **Type Safety**: Strong typing prevents typos and invalid paths
3. **Organized Access**: Assets grouped by category and cult
4. **Performance**: Assets loaded once and reused
5. **Testing**: Easier to mock and test asset loading

## For Module Maintainers

### Current State

- ✅ **game-units**: Fully migrated, using wrapper for backward compatibility
- 🚧 **game-world**: Can be migrated to use registry for building/terrain assets
- 🚧 **game-combat**: Can use registry for effect meshes/materials
- 🚧 **game-ai**: No asset loading needed

### Adding New Asset Categories

When you need new assets not in the current registry:

1. **Add to the appropriate structure** or create a new one:

```rust
// In game-assets/src/registry.rs
pub struct BuildingScenes {
    pub temple: Handle<Scene>,
    pub barracks: Handle<Scene>,
    pub tower: Handle<Scene>,
}

impl BuildingScenes {
    fn load(asset_server: &AssetServer) -> Self {
        Self {
            temple: asset_server.load("assets/models/buildings/temple.glb#Scene0"),
            barracks: asset_server.load("assets/models/buildings/barracks.glb#Scene0"),
            tower: asset_server.load("assets/models/buildings/tower.glb#Scene0"),
        }
    }
    
    pub fn get(&self, building_type: &str) -> Option<Handle<Scene>> {
        match building_type {
            "temple" => Some(self.temple.clone()),
            "barracks" => Some(self.barracks.clone()),
            "tower" => Some(self.tower.clone()),
            _ => None,
        }
    }
}
```

2. **Add to AssetRegistry**:

```rust
pub struct AssetRegistry {
    pub unit_scenes: UnitScenes,
    pub leader_scenes: LeaderScenes,
    pub building_scenes: BuildingScenes,  // New!
    pub common_meshes: CommonMeshes,
    pub materials: MaterialRegistry,
}

impl AssetRegistry {
    pub fn load(asset_server: &AssetServer, meshes: &mut Assets<Mesh>) -> Self {
        Self {
            unit_scenes: UnitScenes::load(asset_server),
            leader_scenes: LeaderScenes::load(asset_server),
            building_scenes: BuildingScenes::load(asset_server),  // New!
            common_meshes: CommonMeshes::create(meshes),
            materials: MaterialRegistry::default(),
        }
    }
    
    pub fn get_building_scene(&self, building_type: &str) -> Option<Handle<Scene>> {
        self.building_scenes.get(building_type)
    }
}
```

3. **Write tests**:

```rust
#[test]
fn test_building_scenes_get() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_plugins(AssetPlugin::default())
        .init_asset::<Scene>();
    app.update();

    let asset_server = app.world().resource::<AssetServer>();
    let scenes = BuildingScenes::load(asset_server);

    assert!(scenes.get("temple").is_some());
    assert!(scenes.get("barracks").is_some());
    assert!(scenes.get("invalid").is_none());
}
```

4. **Update documentation** in the README and module docs.

## Common Patterns

### Loading Multiple Asset Types

```rust
fn spawn_complete_unit(
    mut commands: Commands,
    registry: Res<AssetRegistry>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Get the unit model
    let scene = registry.get_unit_scene("void_seekers", "assassin")
        .expect("Unit scene should exist");
    
    // Use common meshes for UI
    let selection = registry.common_meshes.selection_ring.clone();
    let health_bar = registry.common_meshes.health_bar_background.clone();
    
    // Spawn with visual components
    commands.spawn((
        SceneRoot(scene),
        Transform::default(),
    )).with_children(|parent| {
        parent.spawn(Mesh3d(selection));
        parent.spawn(Mesh3d(health_bar));
    });
}
```

### Registering Custom Materials

```rust
fn setup_cult_materials(
    mut registry: ResMut<AssetRegistry>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Create materials for each cult
    let crimson_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.8, 0.2, 0.2),
        emissive: LinearRgba::rgb(0.5, 0.0, 0.0),
        ..default()
    });
    
    registry.materials.register("crimson_team", crimson_mat);
    
    // Reuse materials later
    if let Some(mat) = registry.materials.get("crimson_team") {
        // Apply to entities
    }
}
```

## Best Practices

1. **Access via Resource**: Always access the registry through `Res<AssetRegistry>`
2. **Don't Clone Registry**: Clone individual handles, not the entire registry
3. **Handle Missing Assets**: Use `Option` returns and provide fallbacks
4. **Group Related Assets**: Keep related assets together in sub-structures
5. **Document Asset Paths**: Comment where asset files should be located

## Troubleshooting

### Asset Not Loading

Check:
1. Asset path is correct in the registry
2. Asset file exists in the correct location
3. Asset type is initialized (`init_asset::<T>()`)
4. AssetServer has finished loading (check `AssetServer::load_state()`)

### Type Not Found

Add required Bevy features to `game-assets/Cargo.toml`:
```toml
bevy = { workspace = true, default-features = false, features = [
    "bevy_asset",
    "bevy_scene",
    "bevy_render",
    "bevy_pbr"
]}
```

### Handle Clone Issues

Don't do this:
```rust
let mesh = &registry.common_meshes.selection_ring;  // Borrows
```

Do this instead:
```rust
let mesh = registry.common_meshes.selection_ring.clone();  // Clones handle (cheap)
```
