//! HUD (Heads-Up Display) container component

use bevy::prelude::*;

/// Marker component for the HUD root node
#[derive(Component)]
pub struct HudRoot;

/// Sets up the main HUD container
pub fn setup_hud(mut commands: Commands) {
    // Create HUD root container
    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                position_type: PositionType::Absolute,
                ..default()
            },
            HudRoot,
            Name::new("HUD Root"),
        ));
}
