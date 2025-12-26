//! Audio events that trigger sound effects

use bevy::prelude::*;

/// Audio events that can be triggered throughout the game
#[derive(Event, Clone, Debug)]
pub enum AudioEvent {
    /// Play sound effect for unit selection
    UnitSelected,
    /// Play sound effect for unit deselection
    UnitDeselected,
    /// Play sound effect for attack
    Attack { attacker: Entity, target: Entity },
    /// Play sound effect for damage received
    DamageReceived { target: Entity, amount: f32 },
    /// Play sound effect for unit death
    UnitDeath { entity: Entity },
    /// Play sound effect for resource gathering start
    ResourceGatherStart { entity: Entity },
    /// Play sound effect for resource gathering complete
    ResourceGatherComplete { entity: Entity },
    /// Play sound effect for building constructed
    BuildingConstructed { entity: Entity },
    /// Change background music to specified track
    ChangeMusic { track: MusicTrack },
    /// Stop all sound effects
    StopAllSfx,
}

/// Available background music tracks
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MusicTrack {
    Menu,
    Exploration,
    Combat,
    Victory,
    Defeat,
}

// Implement Message trait for Bevy's message system
impl bevy::prelude::Message for AudioEvent {}
