//! Unit Spawning Demo
//!
//! Demonstrates unit spawning and basic unit behavior.
//! Run with: cargo run --example unit_spawning

use bevy::prelude::*;
use cosmic_cults::{GameUnitsPlugin, GameWorldPlugin};
use avian3d::prelude::*;
use big_brain::BigBrainPlugin;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(PhysicsPlugins::default())
        .add_plugins(BigBrainPlugin::new(PreUpdate))
        .add_plugins(MeshPickingPlugin)
        .add_plugins(GameWorldPlugin)
        .add_plugins(GameUnitsPlugin)
        .add_systems(Startup, setup)
        .run();
}

fn setup(mut commands: Commands) {
    // Camera
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 20.0, 40.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));

    // Light
    commands.spawn((
        DirectionalLight {
            illuminance: 10000.0,
            ..default()
        },
        Transform::from_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_4)),
    ));

    info!("Unit spawning demo started!");
    info!("Units should spawn automatically through the GameUnitsPlugin.");
}
