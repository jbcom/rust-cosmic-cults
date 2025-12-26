//! Unit Collision Avoidance Test
//!
//! Demonstrates unit-to-unit collision avoidance using Avian3D physics.
//! This test spawns multiple units close together to verify they don't overlap.
//! Run with: cargo run --example collision_avoidance_test

use bevy::prelude::*;
use avian3d::prelude as avian;
use game_physics::GamePhysicsPlugin;
use game_units::{spawning::*, GameUnitsPlugin};
use game_world::GameWorldPlugin;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(GamePhysicsPlugin::default())
        .add_plugins(GameWorldPlugin)
        .add_plugins(GameUnitsPlugin)
        .add_systems(Startup, setup)
        .add_systems(Update, (deferred_setup, keyboard_input))
        .run();
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    // Camera
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 30.0, 50.0).looking_at(Vec3::new(0.0, 0.0, 0.0), Vec3::Y),
    ));

    // Light
    commands.spawn((
        DirectionalLight {
            illuminance: 10000.0,
            ..default()
        },
        Transform::from_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_4)),
    ));

    // Load game assets
    commands.insert_resource(GameAssets::load(&asset_server, &mut meshes));

    info!("=== Collision Avoidance Test ===");
    info!("Testing unit-to-unit collision avoidance with Avian3D physics");
    info!("Units will spawn automatically. Watch for proper collision avoidance!");
    info!("");
    info!("Press SPACE to spawn more test units");
    info!("Press 1-3 to run specific tests");
    info!("Watch for:");
    info!("  - Units should NOT overlap/clip through each other");
    info!("  - Units should push each other apart smoothly");
    info!("  - Moving units should bounce off each other realistically");
}

/// Deferred setup that runs after assets are loaded
fn deferred_setup(
    mut commands: Commands,
    assets: Res<GameAssets>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut setup_done: Local<bool>,
) {
    if *setup_done {
        return;
    }
    *setup_done = true;

    info!("Running deferred setup - spawning test units...");

    // Test 1: Spawn a tight cluster of units to test collision
    info!("Test 1: Spawning tight cluster of 9 units...");
    for i in 0..9 {
        let row = i / 3;
        let col = i % 3;
        let x = (col as f32 - 1.0) * 1.5; // Very close spacing (1.5 units apart)
        let z = (row as f32 - 1.0) * 1.5;

        spawn_unit(
            &mut commands,
            "cultist",
            Vec3::new(x, 0.0, z),
            "crimson_covenant",
            1,
            &assets,
            &mut materials,
        );
    }

    // Test 2: Spawn units moving toward each other
    info!("Test 2: Spawning units on collision course...");
    let unit1 = spawn_unit(
        &mut commands,
        "cultist",
        Vec3::new(-10.0, 0.0, 0.0),
        "deep_ones",
        2,
        &assets,
        &mut materials,
    );

    let unit2 = spawn_unit(
        &mut commands,
        "cultist",
        Vec3::new(10.0, 0.0, 0.0),
        "void_seekers",
        3,
        &assets,
        &mut materials,
    );

    // Give them velocities toward each other using Avian's LinearVelocity
    commands
        .entity(unit1)
        .insert(avian3d::prelude::LinearVelocity(Vec3::new(3.0, 0.0, 0.0)));
    commands
        .entity(unit2)
        .insert(avian3d::prelude::LinearVelocity(Vec3::new(-3.0, 0.0, 0.0)));

    // Test 3: Spawn a leader with units around it
    info!("Test 3: Spawning leader surrounded by units...");
    spawn_leader(
        &mut commands,
        "Test Leader",
        Vec3::new(0.0, 0.0, 15.0),
        "crimson_covenant",
        1,
        game_units::AuraType::Crimson,
        &assets,
        &mut materials,
    );

    // Spawn units in a circle around the leader
    for i in 0..8 {
        let angle = (i as f32) * std::f32::consts::PI * 2.0 / 8.0;
        let radius = 3.0;
        let x = angle.cos() * radius;
        let z = 15.0 + angle.sin() * radius;

        spawn_unit(
            &mut commands,
            "cultist",
            Vec3::new(x, 0.0, z),
            "crimson_covenant",
            1,
            &assets,
            &mut materials,
        );
    }

    info!("=== Test Setup Complete ===");
    info!("Press SPACE to spawn more test units");
    info!("Press 1-3 to run specific tests");
    info!("Watch for:");
    info!("  - Units should NOT overlap/clip through each other");
    info!("  - Units should push each other apart smoothly");
    info!("  - Moving units should bounce off each other realistically");
}

fn keyboard_input(
    mut commands: Commands,
    keys: Res<ButtonInput<KeyCode>>,
    assets: Res<GameAssets>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Spawn a single test unit
    if keys.just_pressed(KeyCode::Space) {
        spawn_unit(
            &mut commands,
            "cultist",
            Vec3::new(0.0, 0.0, 0.0),
            "void_seekers",
            3,
            &assets,
            &mut materials,
        );
        info!("Spawned test unit at origin");
    }

    // Test 1: Spawn overlapping units
    if keys.just_pressed(KeyCode::Digit1) {
        info!("Running Test 1: Overlapping units...");
        for i in 0..5 {
            spawn_unit(
                &mut commands,
                "cultist",
                Vec3::new(i as f32 * 0.5, 0.0, -5.0), // Very tight spacing
                "crimson_covenant",
                1,
                &assets,
                &mut materials,
            );
        }
    }

    // Test 2: Spawn units in a line moving forward
    if keys.just_pressed(KeyCode::Digit2) {
        info!("Running Test 2: Moving line of units...");
        for i in 0..5 {
            let entity = spawn_unit(
                &mut commands,
                "cultist",
                Vec3::new((i as f32 - 2.0) * 2.0, 0.0, -10.0),
                "deep_ones",
                2,
                &assets,
                &mut materials,
            );
            commands
                .entity(entity)
                .insert(avian::LinearVelocity(Vec3::new(0.0, 0.0, 5.0)));
        }
    }

    // Test 3: Spawn squad formation
    if keys.just_pressed(KeyCode::Digit3) {
        info!("Running Test 3: Squad formation...");
        spawn_squad(
            &mut commands,
            "cultist",
            Vec3::new(0.0, 0.0, -15.0),
            "void_seekers",
            3,
            12,
            &assets,
            &mut materials,
        );
    }
}
