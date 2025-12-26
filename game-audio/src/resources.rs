//! Audio resources for managing loaded sounds and settings

use bevy::prelude::*;

use crate::MusicTrack;

/// Resource containing handles to all audio assets
#[derive(Resource, Default)]
pub struct AudioAssets {
    // Sound effects
    pub unit_select: Option<Handle<AudioSource>>,
    pub unit_deselect: Option<Handle<AudioSource>>,
    pub attack: Option<Handle<AudioSource>>,
    pub damage_received: Option<Handle<AudioSource>>,
    pub unit_death: Option<Handle<AudioSource>>,
    pub resource_gather_start: Option<Handle<AudioSource>>,
    pub resource_gather_complete: Option<Handle<AudioSource>>,
    pub building_constructed: Option<Handle<AudioSource>>,
    
    // Background music
    pub music_menu: Option<Handle<AudioSource>>,
    pub music_exploration: Option<Handle<AudioSource>>,
    pub music_combat: Option<Handle<AudioSource>>,
    pub music_victory: Option<Handle<AudioSource>>,
    pub music_defeat: Option<Handle<AudioSource>>,
}

/// Audio settings resource
#[derive(Resource, Clone)]
pub struct AudioSettings {
    pub master_volume: f32,
    pub sfx_volume: f32,
    pub music_volume: f32,
    pub music_enabled: bool,
    pub sfx_enabled: bool,
    pub current_music_track: Option<MusicTrack>,
}

impl Default for AudioSettings {
    fn default() -> Self {
        Self {
            master_volume: 1.0,
            sfx_volume: 0.7,
            music_volume: 0.5,
            music_enabled: true,
            sfx_enabled: true,
            current_music_track: None,
        }
    }
}
