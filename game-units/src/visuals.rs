use crate::{AuraType, Health, Leader, Selected, SelectionState, Team, Unit};
use bevy::pbr::StandardMaterial;
use bevy::prelude::*;
use bevy::render::alpha::AlphaMode;

// Visual component markers
#[derive(Component)]
pub struct SelectionIndicator;

#[derive(Component)]
pub struct HealthBar {
    pub max_width: f32,
}

#[derive(Component)]
pub struct HealthBarFill;

#[derive(Component)]
pub struct AuraVisual {
    pub aura_type: AuraType,
    pub base_radius: f32,
}

#[derive(Component)]
pub struct UnitModel;

#[derive(Component)]
pub struct LeaderPlatform;

#[derive(Component)]
pub struct VeteranIndicator;

// Visual configuration
pub const HEALTH_BAR_HEIGHT: f32 = 0.15;
pub const HEALTH_BAR_WIDTH: f32 = 1.5;
pub const HEALTH_BAR_Y_OFFSET: f32 = 2.5;
pub const SELECTION_INDICATOR_RADIUS: f32 = 1.5;
pub const SELECTION_INDICATOR_Y_OFFSET: f32 = 0.1;

/// Get color for a specific aura type
pub fn get_aura_color(aura_type: &AuraType) -> Color {
    match aura_type {
        AuraType::Crimson => Color::srgba(0.8, 0.1, 0.1, 0.3), // Blood red
        AuraType::Deep => Color::srgba(0.1, 0.3, 0.6, 0.3),    // Deep ocean blue
        AuraType::Void => Color::srgba(0.5, 0.1, 0.8, 0.3),    // Purple void
        AuraType::Leadership => Color::srgba(0.9, 0.7, 0.1, 0.3), // Golden
    }
}

/// Get emissive color for aura glow effect
pub fn get_aura_emissive(aura_type: &AuraType) -> LinearRgba {
    match aura_type {
        AuraType::Crimson => LinearRgba::rgb(2.0, 0.2, 0.2),
        AuraType::Deep => LinearRgba::rgb(0.2, 0.6, 1.2),
        AuraType::Void => LinearRgba::rgb(1.0, 0.2, 1.6),
        AuraType::Leadership => LinearRgba::rgb(1.8, 1.4, 0.2),
    }
}

/// System to update health bars based on combat Health component
pub fn update_health_bars(
    mut health_bar_query: Query<(&ChildOf, &mut Transform, &HealthBar, &HealthBarFill)>,
    health_query: Query<&Health, Changed<Health>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    material_query: Query<&MeshMaterial3d<StandardMaterial>>,
) {
    for (parent, mut transform, health_bar, _fill) in health_bar_query.iter_mut() {
        if let Ok(health) = health_query.get(parent.parent()) {
            let health_percentage = health.current / health.maximum;
            transform.scale.x = health_percentage * health_bar.max_width;

            // Change color based on health level
            // This would need to be done through materials, but for now we just scale
            if health_percentage < 0.3 {
                transform.scale.y = HEALTH_BAR_HEIGHT * 1.2; // Make it slightly bigger when low
            } else {
                transform.scale.y = HEALTH_BAR_HEIGHT;
            }
        }
    }
}

/// System to show/hide selection indicators based on selection state
pub fn update_selection_indicators(
    selection_state: Res<SelectionState>,
    mut indicator_query: Query<(&ChildOf, &mut Visibility), With<SelectionIndicator>>,
) {
    for (parent, mut visibility) in indicator_query.iter_mut() {
        let is_selected = selection_state.selected_entities.contains(&parent.parent());
        *visibility = if is_selected {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }
}

/// System to animate aura visuals
pub fn animate_aura_visuals(time: Res<Time>, mut aura_query: Query<(&mut Transform, &AuraVisual)>) {
    for (mut transform, aura) in aura_query.iter_mut() {
        // Pulsing animation
        let pulse = (time.elapsed_secs() * 2.0).sin() * 0.1 + 1.0;
        let scale = aura.base_radius * pulse;
        transform.scale = Vec3::splat(scale);

        // Slow rotation for mystical effect
        transform.rotate_y(0.5 * time.delta_secs());
    }
}

/// System to update leader platform rotation
pub fn animate_leader_platforms(
    time: Res<Time>,
    mut platform_query: Query<&mut Transform, With<LeaderPlatform>>,
) {
    for mut transform in platform_query.iter_mut() {
        transform.rotate_y(0.3 * time.delta_secs());

        // Gentle floating motion
        let float_offset = (time.elapsed_secs() * 1.5).sin() * 0.1;
        transform.translation.y = float_offset;
    }
}

/// System to update veteran indicator visuals
pub fn update_veteran_indicators(
    mut indicator_query: Query<(&mut Transform, &ChildOf), With<VeteranIndicator>>,
    unit_query: Query<&Unit>,
) {
    for (mut transform, parent) in indicator_query.iter_mut() {
        if let Ok(unit) = unit_query.get(parent.parent()) {
            // Scale based on veteran tier
            let scale = 0.5 + (unit.veteran_tier as f32 * 0.2);
            transform.scale = Vec3::splat(scale);
        }
    }
}

/// Create health bar entity
pub fn create_health_bar(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
) -> Entity {
    commands
        .spawn((
            Name::new("HealthBar"),
            Mesh3d(meshes.add(Cuboid::new(HEALTH_BAR_WIDTH, HEALTH_BAR_HEIGHT, 0.05))),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: Color::srgba(0.1, 0.1, 0.1, 0.8),
                unlit: true,
                ..default()
            })),
            Transform::from_translation(Vec3::new(0.0, HEALTH_BAR_Y_OFFSET, 0.0)),
        ))
        .with_children(|parent| {
            // Health fill
            parent.spawn((
                Name::new("HealthFill"),
                Mesh3d(meshes.add(Cuboid::new(
                    HEALTH_BAR_WIDTH * 0.95,
                    HEALTH_BAR_HEIGHT * 0.8,
                    0.04,
                ))),
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
        })
        .id()
}

/// Create selection indicator entity
pub fn create_selection_indicator(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    team_color: Color,
) -> Entity {
    commands
        .spawn((
            Name::new("SelectionIndicator"),
            Mesh3d(meshes.add(Torus::new(
                SELECTION_INDICATOR_RADIUS * 0.1,
                SELECTION_INDICATOR_RADIUS,
            ))),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: team_color.with_alpha(0.5),
                alpha_mode: AlphaMode::Blend,
                emissive: color_to_emissive(team_color),
                ..default()
            })),
            Transform::from_translation(Vec3::new(0.0, SELECTION_INDICATOR_Y_OFFSET, 0.0)),
            SelectionIndicator,
            Visibility::Hidden,
        ))
        .id()
}

/// Create aura visual entity
pub fn create_aura_visual(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    aura_type: AuraType,
    radius: f32,
) -> Entity {
    commands
        .spawn((
            Name::new("AuraVisual"),
            Mesh3d(meshes.add(Sphere::new(radius))),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: get_aura_color(&aura_type),
                alpha_mode: AlphaMode::Blend,
                emissive: get_aura_emissive(&aura_type),
                double_sided: true,
                cull_mode: None,
                ..default()
            })),
            Transform::default(),
            AuraVisual {
                aura_type,
                base_radius: radius,
            },
        ))
        .id()
}

/// Create leader platform entity
pub fn create_leader_platform(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    cult_color: Color,
) -> Entity {
    commands
        .spawn((
            Name::new("LeaderPlatform"),
            Mesh3d(meshes.add(Cylinder::new(2.0, 0.3))),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: cult_color,
                metallic: 0.8,
                perceptual_roughness: 0.3,
                emissive: color_to_emissive(cult_color) * 0.3,
                ..default()
            })),
            Transform::from_translation(Vec3::new(0.0, -0.5, 0.0)),
            LeaderPlatform,
        ))
        .id()
}

/// Create veteran star indicator
pub fn create_veteran_indicator(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    tier: u32,
) -> Entity {
    let color = match tier {
        1 => Color::srgb(0.7, 0.7, 0.7),  // Silver
        2 => Color::srgb(1.0, 0.85, 0.0), // Gold
        3 => Color::srgb(0.0, 0.8, 1.0),  // Diamond
        _ => Color::srgb(0.8, 0.0, 0.8),  // Legendary purple
    };

    commands
        .spawn((
            Name::new("VeteranIndicator"),
            Mesh3d(meshes.add(Sphere::new(0.3))),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: color,
                emissive: color_to_emissive(color) * 2.0,
                metallic: 0.9,
                perceptual_roughness: 0.1,
                ..default()
            })),
            Transform::from_translation(Vec3::new(0.0, 3.0, 0.0)),
            VeteranIndicator,
        ))
        .id()
}

/// System to handle unit death visual effects from combat Health
pub fn handle_death_visuals(
    mut commands: Commands,
    query: Query<(Entity, &Health, &Transform, Option<&Team>), Changed<Health>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    for (entity, health, transform, team) in query.iter() {
        if health.current <= 0.0 {
            // Add fade-out component instead of immediate despawn
            commands.entity(entity).insert(DeathFadeOut {
                timer: Timer::from_seconds(1.0, TimerMode::Once),
            });

            // Spawn death particles based on team
            if let Some(team) = team {
                spawn_death_particles(
                    &mut commands,
                    &mut meshes,
                    &mut materials,
                    transform.translation,
                    team.color,
                );
            }

            #[cfg(feature = "web")]
            web_sys::console::log_1(
                &format!(
                    "Unit died with health: {}/{}",
                    health.current, health.maximum
                )
                .into(),
            );
        }
    }
}

/// System to update unit colors based on team
pub fn update_team_colors(
    mut material_query: Query<
        (&ChildOf, &MeshMaterial3d<StandardMaterial>),
        With<SelectionIndicator>,
    >,
    team_query: Query<&Team, Changed<Team>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    for (parent, mat_handle) in material_query.iter_mut() {
        if let Ok(team) = team_query.get(parent.parent())
            && let Some(material) = materials.get_mut(&mat_handle.0)
        {
            material.base_color = team.color.with_alpha(0.5);
            material.emissive = color_to_emissive(team.color);
        }
    }
}

/// Idle animation system for units
pub fn animate_idle_units(
    time: Res<Time>,
    mut query: Query<(&mut Transform, &Unit), Without<LeaderPlatform>>,
) {
    for (mut transform, _unit) in query.iter_mut() {
        // Subtle breathing/idle animation
        let idle_scale = 1.0 + (time.elapsed_secs() * 2.0).sin() * 0.02;
        transform.scale = Vec3::splat(idle_scale);
    }
}

#[derive(Component)]
pub struct DeathFadeOut {
    pub timer: Timer,
}

#[derive(Component)]
pub struct DeathParticle {
    pub velocity: Vec3,
    pub lifetime: Timer,
}

/// System to update death fade-out effect
pub fn update_death_fade_out(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut DeathFadeOut, &MeshMaterial3d<StandardMaterial>)>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    for (entity, mut fade_out, mat_handle) in query.iter_mut() {
        fade_out.timer.tick(time.delta());

        let progress = fade_out.timer.fraction();
        let alpha = 1.0 - progress;

        if let Some(material) = materials.get_mut(&mat_handle.0) {
            material.base_color = material.base_color.with_alpha(alpha);
            material.alpha_mode = AlphaMode::Blend;
        }

        if fade_out.timer.is_finished() {
            commands.entity(entity).despawn();
        }
    }
}

/// Spawn death particles for visual effect
fn spawn_death_particles(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    position: Vec3,
    team_color: Color,
) {
    for i in 0..10 {
        let angle = (i as f32 / 10.0) * std::f32::consts::TAU;
        let velocity = Vec3::new(angle.cos() * 3.0, 5.0, angle.sin() * 3.0);

        commands.spawn((
            Name::new("DeathParticle"),
            Mesh3d(meshes.add(Sphere::new(0.2))),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: team_color.with_alpha(0.8),
                emissive: color_to_emissive(team_color) * 2.0,
                alpha_mode: AlphaMode::Blend,
                ..default()
            })),
            Transform::from_translation(position),
            DeathParticle {
                velocity,
                lifetime: Timer::from_seconds(1.5, TimerMode::Once),
            },
        ));
    }
}

/// Update death particle movement and lifetime
pub fn update_death_particles(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut Transform, &mut DeathParticle)>,
) {
    for (entity, mut transform, mut particle) in query.iter_mut() {
        particle.lifetime.tick(time.delta());

        // Update position with gravity
        transform.translation += particle.velocity * time.delta_secs();
        particle.velocity.y -= 9.8 * time.delta_secs();

        // Scale down over time
        let scale = 1.0 - particle.lifetime.fraction();
        transform.scale = Vec3::splat(scale * 0.2);

        if particle.lifetime.is_finished() {
            commands.entity(entity).despawn();
        }
    }
}

// Helper function to convert color for emissive
fn color_to_emissive(color: Color) -> LinearRgba {
    let srgba = color.to_srgba();
    LinearRgba::rgb(srgba.red, srgba.green, srgba.blue)
}
