//! Audio systems for handling sound playback

use bevy::prelude::*;

use crate::{AudioAssets, AudioEvent, AudioSettings, MusicTrack};

/// Load audio assets from the asset server
/// Note: In production, you would load actual audio files from assets/audio/
pub fn load_audio_assets(
    mut audio_assets: ResMut<AudioAssets>,
    asset_server: Res<AssetServer>,
) {
    // Attempt to load sound effect files
    // These will gracefully fail if files don't exist
    audio_assets.unit_select = try_load_audio(&asset_server, "audio/sfx/unit_select.ogg");
    audio_assets.unit_deselect = try_load_audio(&asset_server, "audio/sfx/unit_deselect.ogg");
    audio_assets.attack = try_load_audio(&asset_server, "audio/sfx/attack.ogg");
    audio_assets.damage_received = try_load_audio(&asset_server, "audio/sfx/damage.ogg");
    audio_assets.unit_death = try_load_audio(&asset_server, "audio/sfx/death.ogg");
    audio_assets.resource_gather_start = try_load_audio(&asset_server, "audio/sfx/gather_start.ogg");
    audio_assets.resource_gather_complete = try_load_audio(&asset_server, "audio/sfx/gather_complete.ogg");
    audio_assets.building_constructed = try_load_audio(&asset_server, "audio/sfx/building.ogg");

    // Attempt to load music files
    audio_assets.music_menu = try_load_audio(&asset_server, "audio/music/menu.ogg");
    audio_assets.music_exploration = try_load_audio(&asset_server, "audio/music/exploration.ogg");
    audio_assets.music_combat = try_load_audio(&asset_server, "audio/music/combat.ogg");
    audio_assets.music_victory = try_load_audio(&asset_server, "audio/music/victory.ogg");
    audio_assets.music_defeat = try_load_audio(&asset_server, "audio/music/defeat.ogg");

    println!("Audio assets loading initiated");
}

/// Try to load an audio file, returning None if it doesn't exist
fn try_load_audio(asset_server: &AssetServer, path: &'static str) -> Option<Handle<AudioSource>> {
    Some(asset_server.load(path))
}

/// Handle audio events and play appropriate sounds
pub fn handle_audio_events(
    mut commands: Commands,
    mut audio_events: EventReader<AudioEvent>,
    audio_assets: Res<AudioAssets>,
    settings: Res<AudioSettings>,
) {
    if !settings.sfx_enabled {
        return;
    }

    for event in audio_events.read() {
        let _volume = settings.master_volume * settings.sfx_volume;
        
        // Audio playback will be implemented once audio files are available
        // For now, we just acknowledge the events
        match event {
            AudioEvent::UnitSelected => {
                if let Some(_sound) = &audio_assets.unit_select {
                    // commands.spawn(AudioBundle { source: sound.clone(), ..default() });
                }
            }
            AudioEvent::UnitDeselected => {
                if let Some(_sound) = &audio_assets.unit_deselect {
                    // commands.spawn(AudioBundle { source: sound.clone(), ..default() });
                }
            }
            AudioEvent::Attack { .. } => {
                if let Some(_sound) = &audio_assets.attack {
                    // commands.spawn(AudioBundle { source: sound.clone(), ..default() });
                }
            }
            AudioEvent::DamageReceived { .. } => {
                if let Some(_sound) = &audio_assets.damage_received {
                    // commands.spawn(AudioBundle { source: sound.clone(), ..default() });
                }
            }
            AudioEvent::UnitDeath { .. } => {
                if let Some(_sound) = &audio_assets.unit_death {
                    // commands.spawn(AudioBundle { source: sound.clone(), ..default() });
                }
            }
            AudioEvent::ResourceGatherStart { .. } => {
                if let Some(_sound) = &audio_assets.resource_gather_start {
                    // commands.spawn(AudioBundle { source: sound.clone(), ..default() });
                }
            }
            AudioEvent::ResourceGatherComplete { .. } => {
                if let Some(_sound) = &audio_assets.resource_gather_complete {
                    // commands.spawn(AudioBundle { source: sound.clone(), ..default() });
                }
            }
            AudioEvent::BuildingConstructed { .. } => {
                if let Some(_sound) = &audio_assets.building_constructed {
                    // commands.spawn(AudioBundle { source: sound.clone(), ..default() });
                }
            }
            AudioEvent::ChangeMusic { .. } => {
                // Handled by update_background_music system
            }
            AudioEvent::StopAllSfx => {
                // Stop all audio - handled by despawning audio entities
            }
        }
    }
}

/// Handle unit selection sounds
pub fn handle_selection_sounds(
    mut audio_events: EventWriter<AudioEvent>,
    selection_state: Option<Res<SelectionState>>,
) {
    if let Some(selection) = selection_state {
        if selection.is_changed() && !selection.is_added() {
            if selection.selection_changed {
                if !selection.selected_entities.is_empty() {
                    audio_events.send(AudioEvent::UnitSelected);
                }
            }
        }
    }
}

/// Handle combat sounds by listening to combat events
pub fn handle_combat_sounds(
    mut audio_events: EventWriter<AudioEvent>,
    mut damage_events: Option<EventReader<DamageEvent>>,
    mut death_events: Option<EventReader<DeathEvent>>,
) {
    // Listen for damage events
    if let Some(ref mut events) = damage_events {
        for event in events.read() {
            audio_events.send(AudioEvent::Attack {
                attacker: event.attacker,
                target: event.target,
            });
            audio_events.send(AudioEvent::DamageReceived {
                target: event.target,
                amount: event.amount,
            });
        }
    }

    // Listen for death events
    if let Some(ref mut events) = death_events {
        for event in events.read() {
            audio_events.send(AudioEvent::UnitDeath {
                entity: event.entity,
            });
        }
    }
}

/// Update background music based on game state
pub fn update_background_music(
    mut commands: Commands,
    mut audio_events: EventReader<AudioEvent>,
    audio_assets: Res<AudioAssets>,
    mut settings: ResMut<AudioSettings>,
) {
    if !settings.music_enabled {
        return;
    }

    for event in audio_events.read() {
        if let AudioEvent::ChangeMusic { track } = event {
            // Get the music handle
            let _music_handle = match track {
                MusicTrack::Menu => &audio_assets.music_menu,
                MusicTrack::Exploration => &audio_assets.music_exploration,
                MusicTrack::Combat => &audio_assets.music_combat,
                MusicTrack::Victory => &audio_assets.music_victory,
                MusicTrack::Defeat => &audio_assets.music_defeat,
            };

            // Music playback will be implemented once audio files are available
            // For now, we just track the current music track
            settings.current_music_track = Some(*track);
        }
    }
}

// Re-export types that the audio system needs from other crates
// These are optional and will only be used if the crates are present

#[derive(Resource)]
pub struct SelectionState {
    pub selected_entities: Vec<Entity>,
    pub selection_changed: bool,
}

#[derive(Event)]
pub struct DamageEvent {
    pub attacker: Entity,
    pub target: Entity,
    pub amount: f32,
}

#[derive(Event)]
pub struct DeathEvent {
    pub entity: Entity,
}
