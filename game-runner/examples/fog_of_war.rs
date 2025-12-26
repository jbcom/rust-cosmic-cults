//! Fog of War Demo
//!
//! Demonstrates the fog of war system with units providing vision.
//! Run with: cargo run --example fog_of_war
//!
//! Controls:
//! - WASD: Move camera
//! - Mouse: Look around
//! - U: Spawn test unit at origin
//! - L: Spawn test leader

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
        .add_systems(Update, camera_controls)
        .run();
}

fn setup(mut commands: Commands) {
    // Camera positioned high to see the fog of war
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 50.0, 50.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));

    // Bright directional light to see the fog overlays clearly
    commands.spawn((
        DirectionalLight {
            illuminance: 15000.0,
            shadows_enabled: true,
            ..default()
        },
        Transform::from_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_4)),
    ));

    info!("=== Fog of War Demo Started ===");
    info!("The fog of war system is active:");
    info!("- Hidden areas are shown as dark overlays");
    info!("- Revealed areas (seen before) are semi-transparent");
    info!("- Visible areas (currently in sight) are clear");
    info!("");
    info!("Units and buildings provide vision in a radius around them:");
    info!("- Regular units: 30 unit sight range");
    info!("- Leaders: 40 unit sight range");
    info!("- Buildings: 50 unit sight range");
    info!("");
    info!("Controls:");
    info!("- Press U to spawn a test unit at origin");
    info!("- Press L to spawn a test leader");
    info!("- Use WASD to move the camera around");
}

fn camera_controls(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut camera_query: Query<&mut Transform, With<Camera3d>>,
    time: Res<Time>,
) {
    if let Ok(mut transform) = camera_query.single_mut() {
        let speed = 20.0;
        let dt = time.delta_secs();

        if keyboard.pressed(KeyCode::KeyW) {
            transform.translation.z -= speed * dt;
        }
        if keyboard.pressed(KeyCode::KeyS) {
            transform.translation.z += speed * dt;
        }
        if keyboard.pressed(KeyCode::KeyA) {
            transform.translation.x -= speed * dt;
        }
        if keyboard.pressed(KeyCode::KeyD) {
            transform.translation.x += speed * dt;
        }
    }
}
