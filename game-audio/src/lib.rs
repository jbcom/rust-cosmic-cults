//! Game Audio Plugin for Cosmic Cults
//!
//! This crate provides audio management including sound effects and background music.

use bevy::prelude::*;

pub mod events;
pub mod resources;
pub mod systems;

pub use events::*;
pub use resources::*;
pub use systems::*;

/// Main plugin for the game audio systems
pub struct GameAudioPlugin;

impl Plugin for GameAudioPlugin {
    fn build(&self, app: &mut App) {
        // Initialize audio resources
        app.init_resource::<AudioAssets>()
            .init_resource::<AudioSettings>();

        // Register audio events using messages like other game systems
        app.add_message::<AudioEvent>();

        // Add startup systems
        app.add_systems(Startup, load_audio_assets);

        // Add update systems - added separately to avoid tuple length issues
        app.add_systems(Update, handle_audio_events);
        app.add_systems(Update, handle_selection_sounds);
        app.add_systems(Update, handle_combat_sounds);
        app.add_systems(Update, update_background_music);

        println!("Game Audio Plugin loaded successfully");
    }
}
