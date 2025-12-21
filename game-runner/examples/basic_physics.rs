//! Basic Physics Demo
//!
//! Demonstrates the physics system with moving entities and collision detection.
//! Run with: cargo run --example basic_physics

use bevy::prelude::*;
use game_physics::prelude::*;
use rand::random;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(GamePhysicsPlugin::default())
        .add_systems(Startup, setup)
        .add_systems(Update, keyboard_input)
        .run();
}

fn setup(mut commands: Commands) {
    // Camera
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 15.0, 30.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));

    // Light
    commands.spawn((
        DirectionalLight {
            illuminance: 10000.0,
            ..default()
        },
        Transform::from_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_4)),
    ));

    // Create some physics entities
    for i in 0..5 {
        let x = (i as f32 - 2.0) * 3.0;
        create_physics_entity(&mut commands, Vec3::new(x, 5.0, 0.0), Vec3::ZERO, 1.0);
    }

    // Ground plane
    create_aabb_collider(
        &mut commands,
        Vec3::new(0.0, -1.0, 0.0),
        Vec3::new(50.0, 1.0, 50.0),
        false,
    );

    info!("Physics demo started! Press SPACE to spawn a new entity.");
}

fn keyboard_input(mut commands: Commands, keys: Res<ButtonInput<KeyCode>>) {
    if keys.just_pressed(KeyCode::Space) {
        let x = (random::<f32>() - 0.5) * 10.0;
        let z = (random::<f32>() - 0.5) * 10.0;
        create_physics_entity(&mut commands, Vec3::new(x, 10.0, z), Vec3::ZERO, 1.0);
        info!("Spawned new physics entity!");
    }
}
