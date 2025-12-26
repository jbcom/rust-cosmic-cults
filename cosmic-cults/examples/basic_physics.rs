//! Basic Physics Demo
//!
//! Demonstrates the physics system with moving entities and collision detection.
//! Run with: cargo run --example basic_physics

use bevy::prelude::*;
use avian3d::prelude::*;
use rand::random;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(PhysicsPlugins::default())
        .add_systems(Startup, setup)
        .add_systems(Update, keyboard_input)
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
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

    // Ground plane
    commands.spawn((
        Mesh3d(meshes.add(Plane3d::default().mesh().size(50.0, 50.0))),
        MeshMaterial3d(materials.add(Color::srgb(0.3, 0.5, 0.3))),
        RigidBody::Static,
        Collider::cuboid(50.0, 0.1, 50.0),
        Transform::from_xyz(0.0, -0.05, 0.0),
    ));

    // Create some physics entities
    let mesh = meshes.add(Cuboid::from_size(Vec3::ONE));
    let material = materials.add(Color::srgb(0.8, 0.3, 0.3));

    for i in 0..5 {
        let x = (i as f32 - 2.0) * 3.0;
        commands.spawn((
            Mesh3d(mesh.clone()),
            MeshMaterial3d(material.clone()),
            RigidBody::Dynamic,
            Collider::cuboid(1.0, 1.0, 1.0),
            Transform::from_xyz(x, 5.0, 0.0),
        ));
    }

    info!("Physics demo started! Press SPACE to spawn a new entity.");
}

fn keyboard_input(
    mut commands: Commands,
    keys: Res<ButtonInput<KeyCode>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    if keys.just_pressed(KeyCode::Space) {
        let x = (random::<f32>() - 0.5) * 10.0;
        let z = (random::<f32>() - 0.5) * 10.0;
        commands.spawn((
            Mesh3d(meshes.add(Cuboid::from_size(Vec3::ONE))),
            MeshMaterial3d(materials.add(Color::srgb(random(), random(), random()))),
            RigidBody::Dynamic,
            Collider::cuboid(1.0, 1.0, 1.0),
            Transform::from_xyz(x, 10.0, z),
        ));
        info!("Spawned new physics entity!");
    }
}
