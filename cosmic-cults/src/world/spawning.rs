//! World entity spawning system for Cosmic Dominion

use crate::world::fog::{Faction, VisionProvider};
use bevy::asset::RenderAssetUsages;
use bevy::mesh::Indices;
use bevy::prelude::*;
use bevy::render::render_resource::PrimitiveTopology;
use crate::world::assets::{Cult, models};
use tracing::info;

/// Marker component for the cult leader
#[derive(Component)]
pub struct CultLeader {
    pub cult: Cult,
    pub level: u32,
}

/// Marker component for the player's starting unit
#[derive(Component)]
pub struct PlayerUnit {
    pub unit_type: UnitType,
}

#[derive(Clone, Copy, Debug)]
pub enum UnitType {
    Acolyte,
    BloodWarrior,
    DeepOne,
    VoidWalker,
}

/// Marker component for the leadership building
#[derive(Component)]
pub struct LeadershipBuilding {
    pub cult: Cult,
}

/// Marker component for totems
#[derive(Component)]
pub struct Totem {
    pub power_level: f32,
}

/// Marker component for initial creature
#[derive(Component)]
pub struct InitialCreature {
    pub creature_type: CreatureType,
}

#[derive(Clone, Copy, Debug)]
pub enum CreatureType {
    CorruptedBeast,
    VoidSpawn,
    BloodFiend,
}

/// Spawn the starting scene with all initial entities
pub fn spawn_starting_scene(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    info!("Spawning starting scene for Cosmic Dominion");

    // Spawn leadership building at center
    spawn_leadership_building(
        &mut commands,
        &asset_server,
        &mut meshes,
        &mut materials,
        Vec3::ZERO,
        Cult::Crimson,
    );

    // Spawn cult leader on platform (slightly elevated and to the side)
    spawn_cult_leader(
        &mut commands,
        &asset_server,
        &mut meshes,
        &mut materials,
        Vec3::new(5.0, 2.0, 0.0),
        Cult::Crimson,
    );

    // Spawn player's starting unit (acolyte)
    spawn_player_unit(
        &mut commands,
        &asset_server,
        Vec3::new(-5.0, 0.0, 0.0),
        UnitType::Acolyte,
    );

    // Spawn initial creature
    spawn_initial_creature(
        &mut commands,
        &asset_server,
        &mut meshes,
        &mut materials,
        Vec3::new(0.0, 0.0, -8.0),
        CreatureType::CorruptedBeast,
    );

    // Spawn ritual totem
    spawn_totem(
        &mut commands,
        &asset_server,
        &mut meshes,
        &mut materials,
        Vec3::new(0.0, 0.0, 5.0),
    );

    // Spawn additional atmospheric elements
    spawn_cult_banners(&mut commands, &mut meshes, &mut materials, Cult::Crimson);
    spawn_ritual_circle(&mut commands, &mut meshes, &mut materials, Vec3::ZERO);
}

/// Spawn the leadership building
fn spawn_leadership_building(
    commands: &mut Commands,
    asset_server: &AssetServer,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    position: Vec3,
    cult: Cult,
) {
    // Try to load the temple GLB model
    let temple_model = asset_server.load(models::buildings::TEMPLE);

    // Spawn the temple
    commands.spawn((
        SceneRoot(temple_model.clone()),
        Transform::from_translation(position).with_scale(Vec3::splat(2.0)),
        LeadershipBuilding { cult },
        VisionProvider {
            sight_range: 50.0,
            faction: Faction::Player,
        },
        Name::new("Leadership Building"),
    ));

    // Add a glowing platform under the building
    let platform_mesh = meshes.add(Cylinder::new(8.0, 0.5));
    let platform_material = materials.add(StandardMaterial {
        base_color: match cult {
            Cult::Crimson => Color::srgb(0.3, 0.0, 0.0),
            Cult::Deep => Color::srgb(0.0, 0.1, 0.2),
            Cult::Void => Color::srgb(0.1, 0.0, 0.2),
        },
        emissive: match cult {
            Cult::Crimson => LinearRgba::from(Color::srgb(0.5, 0.0, 0.0)),
            Cult::Deep => LinearRgba::from(Color::srgb(0.0, 0.2, 0.4)),
            Cult::Void => LinearRgba::from(Color::srgb(0.2, 0.0, 0.4)),
        } * 0.3,
        metallic: 0.3,
        perceptual_roughness: 0.6,
        ..default()
    });

    commands.spawn((
        Mesh3d(platform_mesh),
        MeshMaterial3d(platform_material),
        Transform::from_translation(position + Vec3::Y * -0.25),
        Name::new("Temple Platform"),
    ));
}

/// Spawn the cult leader
fn spawn_cult_leader(
    commands: &mut Commands,
    _asset_server: &AssetServer,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    position: Vec3,
    cult: Cult,
) {
    // Create a dramatic leader model (using primitive for now, can replace with GLB)
    let leader_mesh = meshes.add(Capsule3d::new(0.5, 2.0));
    let leader_material = materials.add(StandardMaterial {
        base_color: match cult {
            Cult::Crimson => Color::srgb(0.6, 0.1, 0.1),
            Cult::Deep => Color::srgb(0.1, 0.3, 0.4),
            Cult::Void => Color::srgb(0.2, 0.1, 0.3),
        },
        emissive: match cult {
            Cult::Crimson => LinearRgba::from(Color::srgb(0.8, 0.0, 0.0)),
            Cult::Deep => LinearRgba::from(Color::srgb(0.0, 0.4, 0.6)),
            Cult::Void => LinearRgba::from(Color::srgb(0.4, 0.0, 0.6)),
        } * 0.5,
        metallic: 0.6,
        perceptual_roughness: 0.3,
        ..default()
    });

    commands.spawn((
        Mesh3d(leader_mesh),
        MeshMaterial3d(leader_material),
        Transform::from_translation(position),
        CultLeader { cult, level: 1 },
        VisionProvider {
            sight_range: 40.0,
            faction: Faction::Player,
        },
        Name::new("Cult Leader"),
    ));

    // Add leader's aura effect
    let aura_mesh = meshes.add(Sphere::new(2.0));
    let aura_material = materials.add(StandardMaterial {
        base_color: Color::srgba(0.8, 0.2, 0.2, 0.1),
        emissive: LinearRgba::from(Color::srgb(1.0, 0.0, 0.0)) * 0.2,
        alpha_mode: AlphaMode::Blend,
        double_sided: true,
        cull_mode: None,
        ..default()
    });

    commands.spawn((
        Mesh3d(aura_mesh),
        MeshMaterial3d(aura_material),
        Transform::from_translation(position),
        Name::new("Leader Aura"),
    ));
}

/// Spawn the player's starting unit
fn spawn_player_unit(
    commands: &mut Commands,
    asset_server: &AssetServer,
    position: Vec3,
    unit_type: UnitType,
) {
    // Load the appropriate unit model
    let unit_model = match unit_type {
        UnitType::Acolyte => asset_server.load(models::units::ACOLYTE),
        UnitType::BloodWarrior => asset_server.load(models::units::BLOOD_WARRIOR),
        UnitType::DeepOne => asset_server.load(models::units::DEEP_ONE),
        UnitType::VoidWalker => asset_server.load(models::units::VOID_WALKER),
    };

    commands.spawn((
        SceneRoot(unit_model),
        Transform::from_translation(position).with_scale(Vec3::splat(1.5)),
        PlayerUnit { unit_type },
        VisionProvider {
            sight_range: 30.0,
            faction: Faction::Player,
        },
        Name::new("Player Unit"),
    ));
}

/// Spawn the initial creature
fn spawn_initial_creature(
    commands: &mut Commands,
    _asset_server: &AssetServer,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    position: Vec3,
    creature_type: CreatureType,
) {
    // Create a menacing creature model
    let creature_mesh = meshes.add(Cuboid::new(1.5, 1.0, 2.0));
    let creature_material = materials.add(StandardMaterial {
        base_color: match creature_type {
            CreatureType::CorruptedBeast => Color::srgb(0.3, 0.2, 0.3),
            CreatureType::VoidSpawn => Color::srgb(0.1, 0.0, 0.2),
            CreatureType::BloodFiend => Color::srgb(0.4, 0.1, 0.1),
        },
        emissive: LinearRgba::from(Color::srgb(0.5, 0.0, 0.5)) * 0.2,
        metallic: 0.1,
        perceptual_roughness: 0.8,
        ..default()
    });

    commands.spawn((
        Mesh3d(creature_mesh),
        MeshMaterial3d(creature_material),
        Transform::from_translation(position),
        InitialCreature { creature_type },
        Name::new("Corrupted Creature"),
    ));
}

/// Spawn the ritual totem
fn spawn_totem(
    commands: &mut Commands,
    asset_server: &AssetServer,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    position: Vec3,
) {
    // Try to load the obelisk model
    let obelisk_model = asset_server.load(models::terrain::LANDMARK_OBELISK);

    commands.spawn((
        SceneRoot(obelisk_model),
        Transform::from_translation(position).with_scale(Vec3::splat(1.5)),
        Totem { power_level: 1.0 },
        Name::new("Ritual Totem"),
    ));

    // Add glowing runes around the totem
    let rune_mesh = meshes.add(Torus::new(2.0, 0.1));
    let rune_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.6, 0.0, 0.8),
        emissive: LinearRgba::from(Color::srgb(0.8, 0.0, 1.0)) * 0.5,
        ..default()
    });

    commands.spawn((
        Mesh3d(rune_mesh),
        MeshMaterial3d(rune_material),
        Transform::from_translation(position + Vec3::Y * 0.1)
            .with_rotation(Quat::from_rotation_x(std::f32::consts::FRAC_PI_2)),
        Name::new("Totem Runes"),
    ));
}

/// Spawn cult banners around the starting area
fn spawn_cult_banners(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    cult: Cult,
) {
    let banner_positions = [
        Vec3::new(10.0, 0.0, 10.0),
        Vec3::new(-10.0, 0.0, 10.0),
        Vec3::new(10.0, 0.0, -10.0),
        Vec3::new(-10.0, 0.0, -10.0),
    ];

    let banner_mesh = meshes.add(Cuboid::new(0.2, 4.0, 0.2));
    let flag_mesh = meshes.add(Plane3d::new(Vec3::Z, Vec2::new(1.5, 2.0)));

    let pole_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.2, 0.15, 0.1),
        metallic: 0.3,
        perceptual_roughness: 0.7,
        ..default()
    });

    let flag_material = materials.add(StandardMaterial {
        base_color: match cult {
            Cult::Crimson => Color::srgb(0.6, 0.0, 0.0),
            Cult::Deep => Color::srgb(0.0, 0.2, 0.4),
            Cult::Void => Color::srgb(0.2, 0.0, 0.4),
        },
        double_sided: true,
        cull_mode: None,
        ..default()
    });

    for position in banner_positions.iter() {
        // Banner pole
        commands.spawn((
            Mesh3d(banner_mesh.clone()),
            MeshMaterial3d(pole_material.clone()),
            Transform::from_translation(*position + Vec3::Y * 2.0),
            Name::new("Banner Pole"),
        ));

        // Banner flag
        commands.spawn((
            Mesh3d(flag_mesh.clone()),
            MeshMaterial3d(flag_material.clone()),
            Transform::from_translation(*position + Vec3::Y * 3.5)
                .with_rotation(Quat::from_rotation_y(position.x.atan2(position.z))),
            Name::new("Banner Flag"),
        ));
    }
}

/// Spawn a ritual circle on the ground
fn spawn_ritual_circle(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    center: Vec3,
) {
    // Create concentric circles for the ritual area
    let circle_radii = [3.0, 5.0, 7.0];

    for (i, &radius) in circle_radii.iter().enumerate() {
        let circle_mesh = meshes.add(Torus::new(radius, 0.1));
        let circle_material = materials.add(StandardMaterial {
            base_color: Color::srgb(0.5, 0.0, 0.5),
            emissive: LinearRgba::from(Color::srgb(0.6, 0.0, 0.6)) * (0.3 - i as f32 * 0.1),
            ..default()
        });

        commands.spawn((
            Mesh3d(circle_mesh),
            MeshMaterial3d(circle_material),
            Transform::from_translation(center + Vec3::Y * 0.05)
                .with_rotation(Quat::from_rotation_x(std::f32::consts::FRAC_PI_2)),
            Name::new(format!("Ritual Circle {}", i + 1)),
        ));
    }

    // Add pentagram or cult symbol in the center
    let symbol_mesh = meshes.add(create_cult_symbol_mesh());
    let symbol_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.8, 0.0, 0.0),
        emissive: LinearRgba::from(Color::srgb(1.0, 0.0, 0.0)) * 0.5,
        ..default()
    });

    commands.spawn((
        Mesh3d(symbol_mesh),
        MeshMaterial3d(symbol_material),
        Transform::from_translation(center + Vec3::Y * 0.1)
            .with_rotation(Quat::from_rotation_x(std::f32::consts::FRAC_PI_2)),
        Name::new("Cult Symbol"),
    ));
}

/// Create a mesh for the cult symbol (pentagram)
fn create_cult_symbol_mesh() -> Mesh {
    let mut positions = Vec::new();
    let mut indices = Vec::new();

    // Create a pentagram
    let points = 5;
    let outer_radius = 2.0;
    let inner_radius = 0.8;

    // Center point
    positions.push([0.0, 0.0, 0.0]);

    // Generate star points
    for i in 0..points * 2 {
        let angle =
            (i as f32 / (points * 2) as f32) * std::f32::consts::TAU - std::f32::consts::FRAC_PI_2;
        let radius = if i % 2 == 0 {
            outer_radius
        } else {
            inner_radius
        };

        positions.push([angle.cos() * radius, 0.0, angle.sin() * radius]);
    }

    // Create triangles
    for i in 0..points * 2 {
        indices.push(0);
        indices.push((i + 1) as u32);
        indices.push(((i + 1) % (points * 2) + 1) as u32);
    }

    Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    )
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
    .with_inserted_indices(Indices::U32(indices))
}
