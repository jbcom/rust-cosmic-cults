//! Formation Demo
//!
//! Demonstrates unit formations and group movement.
//! Run with: cargo run --example formations

use bevy::prelude::*;
use game_physics::GamePhysicsPlugin;
use game_units::GameUnitsPlugin;
use game_world::GameWorldPlugin;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(GamePhysicsPlugin::default())
        .add_plugins(GameWorldPlugin)
        .add_plugins(GameUnitsPlugin)
        .add_systems(Startup, setup)
        .add_systems(Update, keyboard_controls)
        .run();
}

fn setup(mut commands: Commands) {
    // Camera
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 30.0, 50.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));

    // Light
    commands.spawn((
        DirectionalLight {
            illuminance: 10000.0,
            ..default()
        },
        Transform::from_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_4)),
    ));

    info!("Formation demo started!");
    info!("Units should spawn through the GameUnitsPlugin");
    info!("Press 1-5 to change formation type:");
    info!("  1: Line  2: Column  3: Box  4: Wedge  5: Circle");
}

fn keyboard_controls(keys: Res<ButtonInput<KeyCode>>) {
    if keys.just_pressed(KeyCode::Digit1) {
        info!("Formation: Line");
    }
    if keys.just_pressed(KeyCode::Digit2) {
        info!("Formation: Column");
    }
    if keys.just_pressed(KeyCode::Digit3) {
        info!("Formation: Box");
    }
    if keys.just_pressed(KeyCode::Digit4) {
        info!("Formation: Wedge");
    }
    if keys.just_pressed(KeyCode::Digit5) {
        info!("Formation: Circle");
    }
}
