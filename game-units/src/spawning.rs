use crate::visuals::*;
use crate::{
    AuraType, BaseStats, Experience, Faction, Leader, Selectable, Team, Unit, VeteranBonus,
    VeteranStatus, VeteranTier, VisionProvider,
};
use bevy::pbr::StandardMaterial;
use bevy::prelude::*;
use bevy::render::alpha::AlphaMode;
use game_physics::{
    AABB, CollisionMask, Friction, Mass, MovementController, MovementPath, MovementTarget,
    RigidBodyType, RigidBodyVariant, SpatialData, Velocity,
};
use std::collections::HashMap;
#[cfg(feature = "web")]
use web_sys::console;

/// Resource containing loaded GLB model handles
#[derive(Resource)]
pub struct GameAssets {
    // Unit models by cult
    pub crimson_acolyte: Handle<Scene>,
    pub crimson_warrior: Handle<Scene>,
    pub crimson_berserker: Handle<Scene>,
    pub deep_cultist: Handle<Scene>,
    pub deep_guardian: Handle<Scene>,
    pub deep_horror: Handle<Scene>,
    pub void_initiate: Handle<Scene>,
    pub void_assassin: Handle<Scene>,
    pub void_harbinger: Handle<Scene>,

    // Leader models
    pub blood_lord: Handle<Scene>,
    pub deep_priest: Handle<Scene>,
    pub void_scholar: Handle<Scene>,

    // Common meshes for UI elements
    pub selection_mesh: Handle<Mesh>,
    pub health_bar_mesh: Handle<Mesh>,
    pub health_fill_mesh: Handle<Mesh>,
    pub aura_mesh: Handle<Mesh>,
    pub platform_mesh: Handle<Mesh>,
    pub veteran_star_mesh: Handle<Mesh>,
}

impl GameAssets {
    pub fn load(asset_server: &AssetServer, meshes: &mut Assets<Mesh>) -> Self {
        Self {
            // Load actual GLB models from game-assets folder
            crimson_acolyte: asset_server
                .load("assets/models/units/crimson/blood_acolyte.glb#Scene0"),
            crimson_warrior: asset_server
                .load("assets/models/units/crimson/blood_knight.glb#Scene0"),
            crimson_berserker: asset_server
                .load("assets/models/units/crimson/crimson_berserker.glb#Scene0"),
            deep_cultist: asset_server.load("assets/models/units/deep/coastal_cultist.glb#Scene0"),
            deep_guardian: asset_server.load("assets/models/units/deep/tide_warrior.glb#Scene0"),
            deep_horror: asset_server.load("assets/models/units/deep/abyssal_horror.glb#Scene0"),
            void_initiate: asset_server.load("assets/models/units/void/void_initiate.glb#Scene0"),
            void_assassin: asset_server.load("assets/models/units/void/shadow_blade.glb#Scene0"),
            void_harbinger: asset_server.load("assets/models/units/void/void_harbinger.glb#Scene0"),

            // Leader models
            blood_lord: asset_server.load("assets/models/leaders/crimson/blood_lord.glb#Scene0"),
            deep_priest: asset_server.load("assets/models/leaders/deep/deep_priest.glb#Scene0"),
            void_scholar: asset_server.load("assets/models/leaders/void/void_scholar.glb#Scene0"),

            // Create procedural meshes for UI elements
            selection_mesh: meshes.add(Torus::new(0.15, 1.5)),
            health_bar_mesh: meshes.add(Cuboid::new(HEALTH_BAR_WIDTH, HEALTH_BAR_HEIGHT, 0.05)),
            health_fill_mesh: meshes.add(Cuboid::new(
                HEALTH_BAR_WIDTH * 0.95,
                HEALTH_BAR_HEIGHT * 0.8,
                0.04,
            )),
            aura_mesh: meshes.add(Sphere::new(1.0)),
            platform_mesh: meshes.add(Cylinder::new(2.0, 0.3)),
            veteran_star_mesh: meshes.add(Sphere::new(0.3)),
        }
    }

    pub fn get_unit_model(&self, unit_type: &str, cult: &str) -> Handle<Scene> {
        match (cult, unit_type) {
            ("crimson_covenant", "cultist") => self.crimson_acolyte.clone(),
            ("crimson_covenant", "warrior") => self.crimson_warrior.clone(),
            ("crimson_covenant", "berserker") => self.crimson_berserker.clone(),
            ("deep_ones", "cultist") => self.deep_cultist.clone(),
            ("deep_ones", "guardian") => self.deep_guardian.clone(),
            ("deep_ones", "horror") => self.deep_horror.clone(),
            ("void_seekers", "scout") => self.void_initiate.clone(),
            ("void_seekers", "assassin") => self.void_assassin.clone(),
            ("void_seekers", "harbinger") => self.void_harbinger.clone(),
            _ => self.crimson_acolyte.clone(), // Default fallback
        }
    }

    pub fn get_leader_model(&self, cult: &str) -> Handle<Scene> {
        match cult {
            "crimson_covenant" => self.blood_lord.clone(),
            "deep_ones" => self.deep_priest.clone(),
            "void_seekers" => self.void_scholar.clone(),
            _ => self.blood_lord.clone(), // Default fallback
        }
    }
}

/// Initialize game assets on startup
pub fn init_game_assets(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    let assets = GameAssets::load(&asset_server, &mut meshes);
    commands.insert_resource(assets);
}

// Helper function to convert color for emissive
fn color_to_emissive(color: Color) -> LinearRgba {
    let srgba = color.to_srgba();
    LinearRgba::rgb(srgba.red, srgba.green, srgba.blue)
}

// Unit spawning function with ACTUAL VISUAL COMPONENTS
pub fn spawn_unit(
    commands: &mut Commands,
    unit_type: &str,
    position: Vec3,
    cult: &str,
    team_id: u32,
    assets: &GameAssets,
    materials: &mut Assets<StandardMaterial>,
) -> Entity {
    let cult_color = get_cult_color(cult);
    let model_handle = assets.get_unit_model(unit_type, cult);

    // Split spawn into multiple insert calls to avoid tuple size limit
    let entity = commands
        .spawn((
            // === VISUAL COMPONENTS ===
            SceneRoot(model_handle),
            Transform::from_translation(position),
            GlobalTransform::default(),
            Visibility::default(),
            ViewVisibility::default(),
            InheritedVisibility::default(),
            // === CORE GAME COMPONENTS ===
            Unit {
                unit_type: unit_type.to_string(),
                cult: cult.to_string(),
                health: 100.0,
                max_health: 100.0,
                experience: 0,
                veteran_tier: 0,
                attack_damage: 10.0,
                movement_speed: 5.0,
                attack_speed: 1.0,
            },
            Team {
                id: team_id,
                cult: cult.to_string(),
                color: cult_color,
            },
            Selectable {
                selection_priority: 1,
                selection_radius: 1.5,
            },
        ))
        .insert((
            // === PHYSICS COMPONENTS ===
            MovementController {
                target_position: None,
                velocity: Vec3::ZERO,
                max_speed: 5.0,
                acceleration: 10.0,
                rotation_speed: 5.0,
                path_index: 0,
                waypoints: Vec::new(),
                is_moving: false,
                movement_type: game_physics::MovementType::Ground,
            },
            Velocity::default(),
            AABB::from_size(Vec3::new(1.0, 2.0, 1.0)), // Unit collision box
            Mass::new(1.0),                            // Standard unit mass
            Friction::default(),
            RigidBodyType {
                body_type: RigidBodyVariant::Dynamic,
            },
            CollisionMask {
                layer: 1,       // Unit layer
                mask: u32::MAX, // Collide with everything
            },
            SpatialData::new(position), // For spatial indexing
        ))
        .insert((
            // === MOVEMENT & STATS COMPONENTS ===
            MovementTarget::new(position.x, position.z, position.z, 5.0),
            MovementPath {
                waypoints: Vec::new(),
                current_waypoint_index: 0,
                movement_speed: 5.0,
                is_moving: false,
            },
            BaseStats {
                base_attack_damage: 10.0,
                base_health: 100.0,
                base_speed: 5.0,
                base_attack_speed: 1.0,
                initialized: true,
            },
            Experience::default(),
            VeteranStatus {
                tier: VeteranTier::Recruit,
                promotion_ready: false,
                visual_scale: 1.0,
                bonuses: VeteranBonus::default(),
            },
            VisionProvider {
                sight_range: 30.0,
                faction: get_faction(team_id),
            },
        ))
        .with_children(|parent| {
            // === SELECTION INDICATOR (initially hidden) ===
            parent.spawn((
                Name::new("SelectionIndicator"),
                Mesh3d(assets.selection_mesh.clone()),
                MeshMaterial3d(materials.add(StandardMaterial {
                    base_color: cult_color.with_alpha(0.4),
                    alpha_mode: AlphaMode::Blend,
                    emissive: color_to_emissive(cult_color) * 0.5,
                    double_sided: true,
                    cull_mode: None,
                    ..default()
                })),
                Transform::from_translation(Vec3::new(0.0, SELECTION_INDICATOR_Y_OFFSET, 0.0)),
                SelectionIndicator,
                Visibility::Hidden,
            ));

            // === HEALTH BAR ===
            parent
                .spawn((
                    Name::new("HealthBar"),
                    Mesh3d(assets.health_bar_mesh.clone()),
                    MeshMaterial3d(materials.add(StandardMaterial {
                        base_color: Color::srgba(0.1, 0.1, 0.1, 0.8),
                        unlit: true,
                        ..default()
                    })),
                    Transform::from_translation(Vec3::new(0.0, HEALTH_BAR_Y_OFFSET, 0.0)),
                ))
                .with_children(|health_parent| {
                    // Health fill bar
                    health_parent.spawn((
                        Name::new("HealthFill"),
                        Mesh3d(assets.health_fill_mesh.clone()),
                        MeshMaterial3d(materials.add(StandardMaterial {
                            base_color: Color::srgba(0.0, 0.8, 0.0, 0.9),
                            emissive: LinearRgba::rgb(0.0, 0.5, 0.0),
                            unlit: true,
                            ..default()
                        })),
                        Transform::from_translation(Vec3::new(0.0, 0.0, 0.01)),
                        HealthBar {
                            max_width: HEALTH_BAR_WIDTH * 0.95,
                        },
                        HealthBarFill,
                    ));
                });
        })
        .id();

    #[cfg(feature = "web")]
    console::log_1(
        &format!(
            "Spawned {} unit (visual) for {} at ({:.2}, {:.2}, {:.2})",
            unit_type, cult, position.x, position.y, position.z
        )
        .into(),
    );

    entity
}

// Leader spawning function with VISUAL COMPONENTS AND AURA
#[allow(clippy::too_many_arguments)]
pub fn spawn_leader(
    commands: &mut Commands,
    name: &str,
    position: Vec3,
    cult: &str,
    team_id: u32,
    aura_type: AuraType,
    assets: &GameAssets,
    materials: &mut Assets<StandardMaterial>,
) -> Entity {
    let cult_color = get_cult_color(cult);
    let leader_model = assets.get_leader_model(cult);
    let aura_color = get_aura_color(&aura_type);
    let aura_emissive = get_aura_emissive(&aura_type);

    let entity = commands
        .spawn((
            // === VISUAL COMPONENTS ===
            SceneRoot(leader_model),
            Transform::from_translation(position).with_scale(Vec3::splat(1.2)), // Leaders are bigger
            GlobalTransform::default(),
            Visibility::default(),
            ViewVisibility::default(),
            InheritedVisibility::default(),
            // === GAME COMPONENTS ===
            Leader {
                name: name.to_string(),
                cult: cult.to_string(),
                health: 200.0,
                max_health: 200.0,
                shield: 50.0,
                aura_radius: 15.0,
                aura_type: aura_type.clone(),
                platform_entity: None,
                defeat_on_death: true,
                alive: true,
                last_ability1_use: 0.0,
                last_ability2_use: 0.0,
            },
            Unit {
                unit_type: "leader".to_string(),
                cult: cult.to_string(),
                health: 200.0,
                max_health: 200.0,
                experience: 0,
                veteran_tier: 3,
                attack_damage: 25.0,
                movement_speed: 6.0,
                attack_speed: 1.5,
            },
            Team {
                id: team_id,
                cult: cult.to_string(),
                color: cult_color,
            },
            Selectable {
                selection_priority: 10,
                selection_radius: 2.0,
            },
        ))
        .insert((
            MovementTarget::new(position.x, position.z, position.z, 6.0),
            MovementPath {
                waypoints: Vec::new(),
                current_waypoint_index: 0,
                movement_speed: 6.0,
                is_moving: false,
            },
            BaseStats {
                base_attack_damage: 25.0,
                base_health: 200.0,
                base_speed: 6.0,
                base_attack_speed: 1.5,
                initialized: true,
            },
            Experience {
                current: 0,
                total_earned: 1000,
                level: 5,
                kills: 0,
                buildings_destroyed: 0,
            },
            VeteranStatus {
                tier: VeteranTier::Veteran,
                promotion_ready: false,
                visual_scale: 1.2,
                bonuses: VeteranBonus {
                    health_multiplier: 1.5,
                    damage_multiplier: 1.3,
                    speed_multiplier: 1.2,
                    xp_multiplier: 1.0,
                },
            },
            VisionProvider {
                sight_range: 40.0,
                faction: get_faction(team_id),
            },
        ))
        .with_children(|parent| {
            // === AURA VISUAL EFFECT ===
            parent.spawn((
                Name::new("AuraVisual"),
                Mesh3d(assets.aura_mesh.clone()),
                MeshMaterial3d(materials.add(StandardMaterial {
                    base_color: aura_color,
                    alpha_mode: AlphaMode::Blend,
                    emissive: aura_emissive,
                    double_sided: true,
                    cull_mode: None,
                    ..default()
                })),
                Transform::from_scale(Vec3::splat(15.0)), // Aura radius
                AuraVisual {
                    aura_type,
                    base_radius: 15.0,
                },
            ));

            // === LEADER PLATFORM ===
            parent.spawn((
                Name::new("LeaderPlatform"),
                Mesh3d(assets.platform_mesh.clone()),
                MeshMaterial3d(materials.add(StandardMaterial {
                    base_color: cult_color,
                    metallic: 0.8,
                    perceptual_roughness: 0.3,
                    emissive: color_to_emissive(cult_color) * 0.3,
                    ..default()
                })),
                Transform::from_translation(Vec3::new(0.0, -0.5, 0.0)),
                LeaderPlatform,
            ));

            // === SELECTION INDICATOR ===
            parent.spawn((
                Name::new("SelectionIndicator"),
                Mesh3d(assets.selection_mesh.clone()),
                MeshMaterial3d(materials.add(StandardMaterial {
                    base_color: Color::srgba(1.0, 0.85, 0.0, 0.6), // Gold for leaders
                    alpha_mode: AlphaMode::Blend,
                    emissive: LinearRgba::rgb(1.0, 0.85, 0.0),
                    double_sided: true,
                    cull_mode: None,
                    ..default()
                })),
                Transform::from_translation(Vec3::new(0.0, SELECTION_INDICATOR_Y_OFFSET, 0.0))
                    .with_scale(Vec3::splat(1.3)),
                SelectionIndicator,
                Visibility::Hidden,
            ));

            // === HEALTH BAR (bigger for leaders) ===
            parent
                .spawn((
                    Name::new("HealthBar"),
                    Mesh3d(assets.health_bar_mesh.clone()),
                    MeshMaterial3d(materials.add(StandardMaterial {
                        base_color: Color::srgba(0.1, 0.1, 0.1, 0.8),
                        unlit: true,
                        ..default()
                    })),
                    Transform::from_translation(Vec3::new(0.0, HEALTH_BAR_Y_OFFSET + 1.0, 0.0))
                        .with_scale(Vec3::new(1.5, 1.5, 1.0)),
                ))
                .with_children(|health_parent| {
                    health_parent.spawn((
                        Name::new("HealthFill"),
                        Mesh3d(assets.health_fill_mesh.clone()),
                        MeshMaterial3d(materials.add(StandardMaterial {
                            base_color: Color::srgba(0.8, 0.6, 0.0, 0.9), // Gold health for leaders
                            emissive: LinearRgba::rgb(0.5, 0.4, 0.0),
                            unlit: true,
                            ..default()
                        })),
                        Transform::from_translation(Vec3::new(0.0, 0.0, 0.01)),
                        HealthBar {
                            max_width: HEALTH_BAR_WIDTH * 0.95 * 1.5,
                        },
                        HealthBarFill,
                    ));
                });

            // === VETERAN STAR (leaders always have one) ===
            parent.spawn((
                Name::new("VeteranIndicator"),
                Mesh3d(assets.veteran_star_mesh.clone()),
                MeshMaterial3d(materials.add(StandardMaterial {
                    base_color: Color::srgb(1.0, 0.85, 0.0),
                    emissive: LinearRgba::rgb(2.0, 1.7, 0.0),
                    metallic: 0.9,
                    perceptual_roughness: 0.1,
                    ..default()
                })),
                Transform::from_translation(Vec3::new(0.0, 3.5, 0.0)),
                VeteranIndicator,
            ));
        })
        .id();

    #[cfg(feature = "web")]
    console::log_1(
        &format!(
            "Spawned leader {} (visual) for {} at ({:.2}, {:.2}, {:.2})",
            name, cult, position.x, position.y, position.z
        )
        .into(),
    );

    entity
}

// Squad spawning function - spawns multiple units in formation
#[allow(clippy::too_many_arguments)]
pub fn spawn_squad(
    commands: &mut Commands,
    unit_type: &str,
    center_position: Vec3,
    cult: &str,
    team_id: u32,
    count: usize,
    assets: &GameAssets,
    materials: &mut Assets<StandardMaterial>,
) -> Vec<Entity> {
    let mut entities = Vec::new();
    let spacing = 2.0;
    let units_per_row = (count as f32).sqrt().ceil() as usize;

    for i in 0..count {
        let row = i / units_per_row;
        let col = i % units_per_row;

        let offset_x = (col as f32 - units_per_row as f32 / 2.0) * spacing;
        let offset_z = row as f32 * spacing;

        let spawn_position = center_position + Vec3::new(offset_x, 0.0, offset_z);

        let entity = spawn_unit(
            commands,
            unit_type,
            spawn_position,
            cult,
            team_id,
            assets,
            materials,
        );
        entities.push(entity);
    }

    #[cfg(feature = "web")]
    console::log_1(
        &format!(
            "Spawned squad of {} {} units (visual) for {}",
            count, unit_type, cult
        )
        .into(),
    );

    entities
}

// Get cult color for team identification
fn get_cult_color(cult: &str) -> Color {
    match cult {
        "crimson_covenant" => Color::srgba(0.8, 0.2, 0.2, 1.0),
        "deep_ones" => Color::srgba(0.2, 0.2, 0.8, 1.0),
        "void_seekers" => Color::srgba(0.6, 0.2, 0.8, 1.0),
        _ => Color::srgba(0.5, 0.5, 0.5, 1.0),
    }
}

// Get faction from team_id
fn get_faction(team_id: u32) -> Faction {
    match team_id {
        1 => Faction::Player,
        _ => Faction::Enemy,
    }
}

// Debug spawning system for testing (updated with visual assets)
pub fn debug_spawn_system(
    input: Res<ButtonInput<KeyCode>>,
    mut commands: Commands,
    assets: Res<GameAssets>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    if input.just_pressed(KeyCode::KeyU) {
        // Spawn test unit with visuals
        spawn_unit(
            &mut commands,
            "cultist",
            Vec3::new(0.0, 0.0, 0.0),
            "crimson_covenant",
            1,
            &assets,
            &mut materials,
        );
    }

    if input.just_pressed(KeyCode::KeyL) {
        // Spawn test leader with visuals
        spawn_leader(
            &mut commands,
            "Test Leader",
            Vec3::new(5.0, 0.0, 5.0),
            "crimson_covenant",
            1,
            AuraType::Crimson,
            &assets,
            &mut materials,
        );
    }

    if input.just_pressed(KeyCode::KeyS) {
        // Spawn test squad with visuals
        spawn_squad(
            &mut commands,
            "cultist",
            Vec3::new(-10.0, 0.0, -10.0),
            "deep_ones",
            2,
            6,
            &assets,
            &mut materials,
        );
    }
}

// Unit type definitions for different cults (with model references)
#[derive(Resource)]
pub struct UnitTemplates {
    pub templates: HashMap<String, UnitTemplate>,
}

#[derive(Clone, Debug)]
pub struct UnitTemplate {
    pub unit_type: String,
    pub model_name: String, // Which GLB model to use
    pub base_health: f32,
    pub base_attack: f32,
    pub base_speed: f32,
    pub attack_speed: f32,
    pub cost: HashMap<String, u32>,
    pub build_time: f32,
}

impl Default for UnitTemplates {
    fn default() -> Self {
        let mut templates = HashMap::new();

        // Crimson Covenant units
        templates.insert(
            "crimson_cultist".to_string(),
            UnitTemplate {
                unit_type: "crimson_cultist".to_string(),
                model_name: "blood_acolyte".to_string(),
                base_health: 80.0,
                base_attack: 12.0,
                base_speed: 5.5,
                attack_speed: 1.2,
                cost: HashMap::from([("energy".to_string(), 50)]),
                build_time: 30.0,
            },
        );

        templates.insert(
            "crimson_warrior".to_string(),
            UnitTemplate {
                unit_type: "crimson_warrior".to_string(),
                model_name: "blood_knight".to_string(),
                base_health: 150.0,
                base_attack: 20.0,
                base_speed: 4.0,
                attack_speed: 0.8,
                cost: HashMap::from([("energy".to_string(), 100), ("materials".to_string(), 25)]),
                build_time: 60.0,
            },
        );

        // Deep Ones units
        templates.insert(
            "deep_acolyte".to_string(),
            UnitTemplate {
                unit_type: "deep_acolyte".to_string(),
                model_name: "coastal_cultist".to_string(),
                base_health: 120.0,
                base_attack: 8.0,
                base_speed: 4.5,
                attack_speed: 1.0,
                cost: HashMap::from([("energy".to_string(), 60)]),
                build_time: 35.0,
            },
        );

        templates.insert(
            "deep_guardian".to_string(),
            UnitTemplate {
                unit_type: "deep_guardian".to_string(),
                model_name: "tide_warrior".to_string(),
                base_health: 200.0,
                base_attack: 15.0,
                base_speed: 3.5,
                attack_speed: 0.6,
                cost: HashMap::from([("energy".to_string(), 120), ("materials".to_string(), 30)]),
                build_time: 75.0,
            },
        );

        // Void Seekers units
        templates.insert(
            "void_scout".to_string(),
            UnitTemplate {
                unit_type: "void_scout".to_string(),
                model_name: "void_initiate".to_string(),
                base_health: 60.0,
                base_attack: 10.0,
                base_speed: 7.0,
                attack_speed: 1.5,
                cost: HashMap::from([("energy".to_string(), 40)]),
                build_time: 25.0,
            },
        );

        templates.insert(
            "void_assassin".to_string(),
            UnitTemplate {
                unit_type: "void_assassin".to_string(),
                model_name: "shadow_blade".to_string(),
                base_health: 90.0,
                base_attack: 25.0,
                base_speed: 6.5,
                attack_speed: 2.0,
                cost: HashMap::from([("energy".to_string(), 80), ("materials".to_string(), 20)]),
                build_time: 45.0,
            },
        );

        Self { templates }
    }
}

// Spawn unit from template with visuals
pub fn spawn_unit_from_template(
    commands: &mut Commands,
    template: &UnitTemplate,
    position: Vec3,
    cult: &str,
    team_id: u32,
    assets: &GameAssets,
    materials: &mut Assets<StandardMaterial>,
) -> Entity {
    let cult_color = get_cult_color(cult);
    let model_handle = assets.get_unit_model(&template.model_name, cult);

    let entity = commands
        .spawn((
            // === VISUAL COMPONENTS ===
            SceneRoot(model_handle),
            Transform::from_translation(position),
            GlobalTransform::default(),
            Visibility::default(),
            ViewVisibility::default(),
            InheritedVisibility::default(),
            // === GAME COMPONENTS ===
            Unit {
                unit_type: template.unit_type.clone(),
                cult: cult.to_string(),
                health: template.base_health,
                max_health: template.base_health,
                experience: 0,
                veteran_tier: 0,
                attack_damage: template.base_attack,
                movement_speed: template.base_speed,
                attack_speed: template.attack_speed,
            },
            Team {
                id: team_id,
                cult: cult.to_string(),
                color: cult_color,
            },
            Selectable {
                selection_priority: 1,
                selection_radius: 1.5,
            },
            MovementTarget::new(position.x, position.z, position.z, template.base_speed),
            MovementPath {
                waypoints: Vec::new(),
                current_waypoint_index: 0,
                movement_speed: template.base_speed,
                is_moving: false,
            },
            BaseStats {
                base_attack_damage: template.base_attack,
                base_health: template.base_health,
                base_speed: template.base_speed,
                base_attack_speed: template.attack_speed,
                initialized: true,
            },
            Experience::default(),
            VeteranStatus {
                tier: VeteranTier::Recruit,
                promotion_ready: false,
                visual_scale: 1.0,
                bonuses: VeteranBonus::default(),
            },
            VisionProvider {
                sight_range: 30.0,
                faction: get_faction(team_id),
            },
        ))
        .with_children(|parent| {
            // Add visual children (health bar, selection indicator, etc.)
            // === SELECTION INDICATOR ===
            parent.spawn((
                Name::new("SelectionIndicator"),
                Mesh3d(assets.selection_mesh.clone()),
                MeshMaterial3d(materials.add(StandardMaterial {
                    base_color: cult_color.with_alpha(0.4),
                    alpha_mode: AlphaMode::Blend,
                    emissive: color_to_emissive(cult_color) * 0.5,
                    double_sided: true,
                    cull_mode: None,
                    ..default()
                })),
                Transform::from_translation(Vec3::new(0.0, SELECTION_INDICATOR_Y_OFFSET, 0.0)),
                SelectionIndicator,
                Visibility::Hidden,
            ));

            // === HEALTH BAR ===
            parent
                .spawn((
                    Name::new("HealthBar"),
                    Mesh3d(assets.health_bar_mesh.clone()),
                    MeshMaterial3d(materials.add(StandardMaterial {
                        base_color: Color::srgba(0.1, 0.1, 0.1, 0.8),
                        unlit: true,
                        ..default()
                    })),
                    Transform::from_translation(Vec3::new(0.0, HEALTH_BAR_Y_OFFSET, 0.0)),
                ))
                .with_children(|health_parent| {
                    health_parent.spawn((
                        Name::new("HealthFill"),
                        Mesh3d(assets.health_fill_mesh.clone()),
                        MeshMaterial3d(materials.add(StandardMaterial {
                            base_color: Color::srgba(0.0, 0.8, 0.0, 0.9),
                            emissive: LinearRgba::rgb(0.0, 0.5, 0.0),
                            unlit: true,
                            ..default()
                        })),
                        Transform::from_translation(Vec3::new(0.0, 0.0, 0.01)),
                        HealthBar {
                            max_width: HEALTH_BAR_WIDTH * 0.95,
                        },
                        HealthBarFill,
                    ));
                });
        })
        .id();

    #[cfg(feature = "web")]
    console::log_1(
        &format!(
            "Spawned {} (visual) for {} at ({:.2}, {:.2}, {:.2})",
            template.unit_type, cult, position.x, position.y, position.z
        )
        .into(),
    );

    entity
}
