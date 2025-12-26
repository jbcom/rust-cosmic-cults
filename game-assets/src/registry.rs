//! Centralized asset registry for managing all game assets
//!
//! This module provides a unified interface for loading and accessing textures,
//! meshes, scenes, and materials across the game. Assets are organized by
//! category and cult affiliation for easy management.

use bevy::pbr::StandardMaterial;
use bevy::prelude::*;
use std::collections::HashMap;

// UI mesh dimension constants
const HEALTH_BAR_WIDTH: f32 = 2.0;
const HEALTH_BAR_HEIGHT: f32 = 0.2;
const HEALTH_BAR_DEPTH: f32 = 0.05;
const HEALTH_BAR_FILL_SCALE: f32 = 0.95; // 95% of background size
const HEALTH_BAR_FILL_HEIGHT_SCALE: f32 = 0.8; // 80% of background height
const HEALTH_BAR_FILL_DEPTH_SCALE: f32 = 0.8; // 80% of background depth to avoid z-fighting

/// Central registry for all game assets
///
/// This resource provides organized access to all loaded game assets,
/// including unit models, UI meshes, and materials. It replaces scattered
/// asset loading throughout the codebase with a single source of truth.
#[derive(Resource)]
pub struct AssetRegistry {
    /// Scene handles for unit models organized by cult and unit type
    pub unit_scenes: UnitScenes,
    /// Scene handles for leader models organized by cult
    pub leader_scenes: LeaderScenes,
    /// Common procedural meshes used across the game (UI, effects, etc.)
    pub common_meshes: CommonMeshes,
    /// Material handles organized by type
    pub materials: MaterialRegistry,
}

impl AssetRegistry {
    /// Load all game assets and create the registry
    ///
    /// This should be called during startup to initialize all asset handles.
    /// Assets are loaded asynchronously by Bevy's asset server.
    pub fn load(asset_server: &AssetServer, meshes: &mut Assets<Mesh>) -> Self {
        Self {
            unit_scenes: UnitScenes::load(asset_server),
            leader_scenes: LeaderScenes::load(asset_server),
            common_meshes: CommonMeshes::create(meshes),
            materials: MaterialRegistry::default(),
        }
    }

    /// Get a unit model scene handle for a specific cult and unit type
    pub fn get_unit_scene(&self, cult: &str, unit_type: &str) -> Option<Handle<Scene>> {
        self.unit_scenes.get(cult, unit_type)
    }

    /// Get a leader model scene handle for a specific cult
    pub fn get_leader_scene(&self, cult: &str) -> Option<Handle<Scene>> {
        self.leader_scenes.get(cult)
    }
}

/// Scene handles for all unit models organized by cult
pub struct UnitScenes {
    /// Crimson Covenant unit models
    pub crimson: CrimsonUnits,
    /// Deep Ones unit models
    pub deep: DeepUnits,
    /// Void Seekers unit models
    pub void: VoidUnits,
}

impl UnitScenes {
    /// Load all unit scene handles from the asset server
    fn load(asset_server: &AssetServer) -> Self {
        Self {
            crimson: CrimsonUnits::load(asset_server),
            deep: DeepUnits::load(asset_server),
            void: VoidUnits::load(asset_server),
        }
    }

    /// Get a unit scene by cult and unit type
    pub fn get(&self, cult: &str, unit_type: &str) -> Option<Handle<Scene>> {
        match cult {
            "crimson_covenant" => self.crimson.get(unit_type),
            "deep_ones" => self.deep.get(unit_type),
            "void_seekers" => self.void.get(unit_type),
            _ => None,
        }
    }
}

/// Crimson Covenant unit models
pub struct CrimsonUnits {
    pub acolyte: Handle<Scene>,
    pub warrior: Handle<Scene>,
    pub berserker: Handle<Scene>,
}

impl CrimsonUnits {
    fn load(asset_server: &AssetServer) -> Self {
        Self {
            acolyte: asset_server.load("assets/models/units/crimson/blood_acolyte.glb#Scene0"),
            warrior: asset_server.load("assets/models/units/crimson/blood_knight.glb#Scene0"),
            berserker: asset_server
                .load("assets/models/units/crimson/crimson_berserker.glb#Scene0"),
        }
    }

    fn get(&self, unit_type: &str) -> Option<Handle<Scene>> {
        match unit_type {
            "cultist" | "acolyte" => Some(self.acolyte.clone()),
            "warrior" => Some(self.warrior.clone()),
            "berserker" => Some(self.berserker.clone()),
            _ => None,
        }
    }
}

/// Deep Ones unit models
pub struct DeepUnits {
    pub cultist: Handle<Scene>,
    pub guardian: Handle<Scene>,
    pub horror: Handle<Scene>,
}

impl DeepUnits {
    fn load(asset_server: &AssetServer) -> Self {
        Self {
            cultist: asset_server.load("assets/models/units/deep/coastal_cultist.glb#Scene0"),
            guardian: asset_server.load("assets/models/units/deep/tide_warrior.glb#Scene0"),
            horror: asset_server.load("assets/models/units/deep/abyssal_horror.glb#Scene0"),
        }
    }

    fn get(&self, unit_type: &str) -> Option<Handle<Scene>> {
        match unit_type {
            "cultist" => Some(self.cultist.clone()),
            "guardian" => Some(self.guardian.clone()),
            "horror" => Some(self.horror.clone()),
            _ => None,
        }
    }
}

/// Void Seekers unit models
pub struct VoidUnits {
    pub initiate: Handle<Scene>,
    pub assassin: Handle<Scene>,
    pub harbinger: Handle<Scene>,
}

impl VoidUnits {
    fn load(asset_server: &AssetServer) -> Self {
        Self {
            initiate: asset_server.load("assets/models/units/void/void_initiate.glb#Scene0"),
            assassin: asset_server.load("assets/models/units/void/shadow_blade.glb#Scene0"),
            harbinger: asset_server.load("assets/models/units/void/void_harbinger.glb#Scene0"),
        }
    }

    fn get(&self, unit_type: &str) -> Option<Handle<Scene>> {
        match unit_type {
            "scout" | "initiate" => Some(self.initiate.clone()),
            "assassin" => Some(self.assassin.clone()),
            "harbinger" => Some(self.harbinger.clone()),
            _ => None,
        }
    }
}

/// Scene handles for all leader models organized by cult
pub struct LeaderScenes {
    pub blood_lord: Handle<Scene>,
    pub deep_priest: Handle<Scene>,
    pub void_scholar: Handle<Scene>,
}

impl LeaderScenes {
    /// Load all leader scene handles from the asset server
    fn load(asset_server: &AssetServer) -> Self {
        Self {
            blood_lord: asset_server.load("assets/models/leaders/crimson/blood_lord.glb#Scene0"),
            deep_priest: asset_server.load("assets/models/leaders/deep/deep_priest.glb#Scene0"),
            void_scholar: asset_server.load("assets/models/leaders/void/void_scholar.glb#Scene0"),
        }
    }

    /// Get a leader scene by cult
    pub fn get(&self, cult: &str) -> Option<Handle<Scene>> {
        match cult {
            "crimson_covenant" => Some(self.blood_lord.clone()),
            "deep_ones" => Some(self.deep_priest.clone()),
            "void_seekers" => Some(self.void_scholar.clone()),
            _ => None,
        }
    }
}

/// Common procedural meshes used across the game
///
/// These meshes are created programmatically and used for UI elements,
/// selection indicators, health bars, and other visual effects.
pub struct CommonMeshes {
    /// Torus mesh for selection indicators
    pub selection_ring: Handle<Mesh>,
    /// Cuboid mesh for health bar background
    pub health_bar_background: Handle<Mesh>,
    /// Cuboid mesh for health bar fill
    pub health_bar_fill: Handle<Mesh>,
    /// Sphere mesh for aura effects
    pub aura_sphere: Handle<Mesh>,
    /// Cylinder mesh for leader platforms
    pub leader_platform: Handle<Mesh>,
    /// Small sphere mesh for veteran indicators
    pub veteran_star: Handle<Mesh>,
}

impl CommonMeshes {
    /// Create all common procedural meshes
    fn create(meshes: &mut Assets<Mesh>) -> Self {
        Self {
            selection_ring: meshes.add(Torus::new(0.15, 1.5)),
            health_bar_background: meshes.add(Cuboid::new(
                HEALTH_BAR_WIDTH,
                HEALTH_BAR_HEIGHT,
                HEALTH_BAR_DEPTH,
            )),
            health_bar_fill: meshes.add(Cuboid::new(
                HEALTH_BAR_WIDTH * HEALTH_BAR_FILL_SCALE,
                HEALTH_BAR_HEIGHT * HEALTH_BAR_FILL_HEIGHT_SCALE,
                HEALTH_BAR_DEPTH * HEALTH_BAR_FILL_DEPTH_SCALE,
            )),
            aura_sphere: meshes.add(Sphere::new(1.0)),
            leader_platform: meshes.add(Cylinder::new(2.0, 0.3)),
            veteran_star: meshes.add(Sphere::new(0.3)),
        }
    }
}

/// Registry for material handles
///
/// Materials can be registered and retrieved by name for reuse across
/// different entities, improving performance and consistency.
#[derive(Default)]
pub struct MaterialRegistry {
    materials: HashMap<String, Handle<StandardMaterial>>,
}

impl MaterialRegistry {
    /// Register a material with a given name
    pub fn register(&mut self, name: impl Into<String>, handle: Handle<StandardMaterial>) {
        self.materials.insert(name.into(), handle);
    }

    /// Get a material by name
    pub fn get(&self, name: &str) -> Option<Handle<StandardMaterial>> {
        self.materials.get(name).cloned()
    }

    /// Check if a material is registered
    pub fn contains(&self, name: &str) -> bool {
        self.materials.contains_key(name)
    }
}

/// System to initialize the asset registry on startup
pub fn init_asset_registry(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    let registry = AssetRegistry::load(&asset_server, &mut meshes);
    commands.insert_resource(registry);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_crimson_units_get() {
        // Create a test app to have access to AssetServer
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_plugins(AssetPlugin::default())
            .init_asset::<Scene>();
        app.update();

        let asset_server = app.world().resource::<AssetServer>();
        let units = CrimsonUnits::load(asset_server);

        // Test that we can retrieve unit models
        assert!(units.get("cultist").is_some());
        assert!(units.get("warrior").is_some());
        assert!(units.get("berserker").is_some());
        assert!(units.get("invalid").is_none());
    }

    #[test]
    fn test_material_registry() {
        let mut registry = MaterialRegistry::default();

        // Create a test app to get materials
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_plugins(AssetPlugin::default())
            .init_asset::<StandardMaterial>();
        app.update();

        let mut materials = app.world_mut().resource_mut::<Assets<StandardMaterial>>();
        let handle = materials.add(StandardMaterial::default());

        // Test registration and retrieval
        registry.register("test_material", handle.clone());
        assert!(registry.contains("test_material"));
        assert_eq!(registry.get("test_material").unwrap().id(), handle.id());
        assert!(!registry.contains("nonexistent"));
    }

    #[test]
    fn test_unit_scenes_get() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_plugins(AssetPlugin::default())
            .init_asset::<Scene>();
        app.update();

        let asset_server = app.world().resource::<AssetServer>();
        let scenes = UnitScenes::load(asset_server);

        // Test cult-based retrieval
        assert!(scenes.get("crimson_covenant", "cultist").is_some());
        assert!(scenes.get("deep_ones", "guardian").is_some());
        assert!(scenes.get("void_seekers", "assassin").is_some());
        assert!(scenes.get("invalid_cult", "cultist").is_none());
    }

    #[test]
    fn test_leader_scenes_get() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_plugins(AssetPlugin::default())
            .init_asset::<Scene>();
        app.update();

        let asset_server = app.world().resource::<AssetServer>();
        let scenes = LeaderScenes::load(asset_server);

        // Test cult-based retrieval
        assert!(scenes.get("crimson_covenant").is_some());
        assert!(scenes.get("deep_ones").is_some());
        assert!(scenes.get("void_seekers").is_some());
        assert!(scenes.get("invalid_cult").is_none());
    }
}
