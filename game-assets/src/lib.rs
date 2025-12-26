//! # Game Assets
//!
//! Centralized asset loading and registry system for Cosmic Cults.
//!
//! This crate provides a unified interface for loading and accessing all game assets
//! including unit models, leader models, procedural meshes, and materials. Assets are
//! organized by category and cult affiliation for easy management.
//!
//! ## Quick Start
//!
//! Add the initialization system to your app:
//!
//! ```rust,no_run
//! use game_assets::init_asset_registry;
//! use bevy::prelude::*;
//!
//! App::new()
//!     .add_systems(Startup, init_asset_registry)
//!     .run();
//! ```
//!
//! Access assets through the `AssetRegistry` resource:
//!
//! ```rust,no_run
//! use game_assets::AssetRegistry;
//! use bevy::prelude::*;
//!
//! fn spawn_unit(registry: Res<AssetRegistry>, mut commands: Commands) {
//!     if let Some(scene) = registry.get_unit_scene("crimson_covenant", "cultist") {
//!         commands.spawn(SceneRoot(scene));
//!     }
//! }
//! ```

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

pub mod models;
pub mod registry;

// Re-export commonly used types
pub use registry::{
    AssetRegistry, CommonMeshes, LeaderScenes, MaterialRegistry, UnitScenes, init_asset_registry,
};

/// The three playable cults in Cosmic Cults
///
/// Each cult has unique units, abilities, and visual themes:
/// - **Crimson**: Blood-themed cult focused on melee combat and berserker units
/// - **Deep**: Water-themed cult with defensive units and aquatic creatures
/// - **Void**: Shadow-themed cult specializing in stealth and mobility
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize, Default, Component, Reflect,
)]
#[reflect(Component)]
pub enum Cult {
    /// Crimson Covenant - Blood magic and melee warriors
    #[default]
    Crimson,
    /// Deep Ones - Aquatic horrors and defensive guardians
    Deep,
    /// Void Seekers - Shadow assassins and void magic
    Void,
}
