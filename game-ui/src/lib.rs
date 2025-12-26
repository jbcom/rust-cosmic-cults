//! Game UI Plugin for Cosmic Cults
//!
//! This crate provides the HUD, build menu, and minimap UI components.

use bevy::prelude::*;

pub mod hud;
pub mod build_menu;
pub mod minimap;

pub use hud::*;
pub use build_menu::*;
pub use minimap::*;

/// Main plugin for the game UI systems
pub struct GameUIPlugin;

impl Plugin for GameUIPlugin {
    fn build(&self, app: &mut App) {
        app
            // Add UI systems
            .add_systems(Startup, (
                hud::setup_hud,
                build_menu::setup_build_menu,
                minimap::setup_minimap,
            ))
            .add_systems(Update, (
                build_menu::handle_build_menu_input,
                minimap::update_minimap,
                minimap::handle_minimap_click,
            ));
    }
}
