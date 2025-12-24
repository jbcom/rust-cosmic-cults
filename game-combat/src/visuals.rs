// Visual effects system for combat
use crate::components::*;
use crate::damage::{DamageEvent, DeathEvent};
use bevy::pbr::StandardMaterial;
use bevy::prelude::*;
use bevy::render::alpha::AlphaMode;

// Visual effect components
#[derive(Component)]
pub struct VisualDamageNumber {
    pub amount: f32,
    pub damage_type: DamageType,
    pub lifetime: f32,
    pub velocity: Vec3,
    pub is_critical: bool,
}

#[derive(Component)]
pub struct VisualDeathEffect {
    pub fade_time: f32,
    pub time_elapsed: f32,
    pub particle_spawned: bool,
}

#[derive(Component)]
pub struct HitFlash {
    pub duration: f32,
    pub elapsed: f32,
    pub original_color: Color,
}

#[derive(Component)]
pub struct VisualCombatParticle {
    pub lifetime: f32,
    pub velocity: Vec3,
    pub particle_type: ParticleType,
}

#[derive(Component)]
pub struct ProjectileTrail {
    pub owner: Entity,
    pub positions: Vec<Vec3>,
    pub max_positions: usize,
}

#[derive(Component)]
pub struct ShieldEffect {
    pub radius: f32,
    pub strength: f32,
    pub hit_time: f32,
}

#[derive(Component)]
pub struct BuffVisualIndicator {
    pub effect_type: StatusEffectType,
    pub base_scale: f32,
}

#[derive(Clone, Debug)]
pub enum ParticleType {
    Blood,     // Crimson Covenant
    Water,     // Deep Ones
    Void,      // Void Seekers
    Fire,      // Burn effect
    Ice,       // Freeze effect
    Poison,    // Poison effect
    Holy,      // Healing effect
    Explosion, // Death/AOE
}

// Plugin for combat visuals
pub struct CombatVisualsPlugin;

impl Plugin for CombatVisualsPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<SpawnVisualDamageNumberEvent>()
            .add_message::<SpawnVisualDeathEffectEvent>()
            .add_systems(
                Update,
                (
                    spawn_damage_numbers,
                    update_damage_numbers,
                    apply_hit_flash,
                    update_hit_flash,
                    handle_death_effects,
                    update_death_effects,
                    update_combat_particles,
                    update_projectile_trails,
                    update_shield_effects,
                    animate_buff_indicators,
                    cleanup_expired_effects,
                ),
            );
    }
}

// Events for spawning visual effects
#[derive(Event)]
pub struct SpawnVisualDamageNumberEvent {
    pub position: Vec3,
    pub damage: f32,
    pub damage_type: DamageType,
    pub is_critical: bool,
}

#[derive(Event)]
pub struct SpawnVisualDeathEffectEvent {
    pub entity: Entity,
    pub position: Vec3,
    pub faction: Faction,
}

/// Spawn floating damage numbers when damage is dealt
pub fn spawn_damage_numbers(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut damage_events: MessageReader<DamageEvent>,
    transform_query: Query<&Transform>,
) {
    for event in damage_events.read() {
        if let Ok(transform) = transform_query.get(event.target) {
            let _color = get_damage_color(&event.damage_type, event.is_critical);
            let size = if event.is_critical { 0.5 } else { 0.3 };

            // Spawn damage number entity (using mesh-based text for 3D)
            commands.spawn((
                Name::new("VisualDamageNumber"),
                VisualDamageNumber {
                    amount: event.amount,
                    damage_type: event.damage_type.clone(),
                    lifetime: 1.5,
                    velocity: Vec3::new(
                        rand::random::<f32>() * 2.0 - 1.0,
                        3.0,
                        rand::random::<f32>() * 2.0 - 1.0,
                    ),
                    is_critical: event.is_critical,
                },
                Transform::from_translation(transform.translation + Vec3::Y * 2.0)
                    .with_scale(Vec3::splat(size)),
            ));

            // Flash the unit when hit
            commands.entity(event.target).insert(HitFlash {
                duration: 0.2,
                elapsed: 0.0,
                original_color: Color::WHITE,
            });

            // Spawn impact particles
            spawn_impact_particles(
                &mut commands,
                &mut meshes,
                &mut materials,
                transform.translation,
                &event.damage_type,
            );
        }
    }
}

/// Update floating damage numbers (using mesh-based visualization)
pub fn update_damage_numbers(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut Transform, &mut VisualDamageNumber)>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    for (entity, mut transform, mut damage_num) in query.iter_mut() {
        damage_num.lifetime -= time.delta_seconds();

        if damage_num.lifetime <= 0.0 {
            commands.entity(entity).despawn();
        } else {
            // Float upward and fade out
            transform.translation += damage_num.velocity * time.delta_seconds();
            damage_num.velocity.y -= 2.0 * time.delta_seconds(); // Gravity

            // Fade out effect through scale
            let alpha = damage_num.lifetime / 1.5;
            transform.scale = Vec3::splat(alpha * (if damage_num.is_critical { 0.5 } else { 0.3 }));

            // Scale based on critical with pulse effect
            if damage_num.is_critical {
                let pulse = (time.elapsed_seconds() * 10.0).sin() * 0.1 + 1.0;
                transform.scale *= pulse;
            }

            // Add mesh once at the start
            if damage_num.lifetime > 1.4 {
                // Only add mesh once at the start
                let color = get_damage_color(&damage_num.damage_type, damage_num.is_critical);
                commands.entity(entity).insert((
                    Mesh3d(meshes.add(Sphere::new(0.3))),
                    MeshMaterial3d(materials.add(StandardMaterial {
                        base_color: color.with_alpha(alpha),
                        emissive: color_to_emissive(color),
                        alpha_mode: AlphaMode::Blend,
                        ..default()
                    })),
                ));
            }
        }
    }
}

/// Apply red flash effect when unit is hit
pub fn apply_hit_flash(
    mut material_query: Query<(&MeshMaterial3d<StandardMaterial>, &HitFlash), Added<HitFlash>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    for (mat_handle, _flash) in material_query.iter_mut() {
        if let Some(material) = materials.get_mut(&mat_handle.0) {
            // Store original color and apply red tint
            material.base_color = Color::srgb(1.0, 0.2, 0.2);
            material.emissive = LinearRgba::rgb(1.0, 0.0, 0.0);
        }
    }
}

/// Update hit flash effect
pub fn update_hit_flash(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut HitFlash, &MeshMaterial3d<StandardMaterial>)>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    for (entity, mut flash, mat_handle) in query.iter_mut() {
        flash.elapsed += time.delta_seconds();

        if flash.elapsed >= flash.duration {
            // Restore original color
            if let Some(material) = materials.get_mut(&mat_handle.0) {
                material.base_color = flash.original_color;
                material.emissive = LinearRgba::BLACK;
            }
            commands.entity(entity).remove::<HitFlash>();
        } else {
            // Interpolate back to original color
            if let Some(material) = materials.get_mut(&mat_handle.0) {
                let t = flash.elapsed / flash.duration;
                material.base_color = Color::srgb(
                    1.0 - (1.0 - flash.original_color.to_srgba().red) * t,
                    0.2 + (flash.original_color.to_srgba().green - 0.2) * t,
                    0.2 + (flash.original_color.to_srgba().blue - 0.2) * t,
                );
                material.emissive = LinearRgba::rgb(1.0 - t, 0.0, 0.0);
            }
        }
    }
}

/// Handle death effects when units die
pub fn handle_death_effects(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut death_events: MessageReader<DeathEvent>,
    query: Query<(&Transform, Option<&Team>)>,
) {
    for event in death_events.read() {
        if let Ok((transform, team)) = query.get(event.entity) {
            // Add death effect component
            commands.entity(event.entity).insert(VisualDeathEffect {
                fade_time: 1.0,
                time_elapsed: 0.0,
                particle_spawned: false,
            });

            // Determine faction for particle type
            let faction = team.map(|t| t.faction.clone()).unwrap_or(Faction::Neutral);

            // Spawn death particles
            spawn_death_particles(
                &mut commands,
                &mut meshes,
                &mut materials,
                transform.translation,
                &faction,
            );

            // Play death sound (would need audio system)
            // Log death event for debugging
            tracing::info!("Unit died at {:?}", transform.translation);
        }
    }
}

/// Update death effects (fade out and remove)
pub fn update_death_effects(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(
        Entity,
        &mut VisualDeathEffect,
        &MeshMaterial3d<StandardMaterial>,
    )>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    for (entity, mut death_effect, mat_handle) in query.iter_mut() {
        death_effect.time_elapsed += time.delta_seconds();

        if death_effect.time_elapsed >= death_effect.fade_time {
            // Remove entity after fade complete
            commands.entity(entity).despawn();
        } else {
            // Fade out
            let alpha = 1.0 - (death_effect.time_elapsed / death_effect.fade_time);
            if let Some(material) = materials.get_mut(&mat_handle.0) {
                material.base_color = material.base_color.with_alpha(alpha);
                material.alpha_mode = AlphaMode::Blend;
            }
        }
    }
}

/// Update combat particles
pub fn update_combat_particles(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut Transform, &mut VisualCombatParticle)>,
) {
    for (entity, mut transform, mut particle) in query.iter_mut() {
        particle.lifetime -= time.delta_seconds();

        if particle.lifetime <= 0.0 {
            commands.entity(entity).despawn();
        } else {
            // Update position
            transform.translation += particle.velocity * time.delta_seconds();

            // Apply gravity to some particle types
            match particle.particle_type {
                ParticleType::Blood | ParticleType::Water => {
                    particle.velocity.y -= 9.8 * time.delta_seconds();
                }
                ParticleType::Fire => {
                    particle.velocity.y += 2.0 * time.delta_seconds(); // Fire rises
                }
                _ => {}
            }

            // Scale down over time
            let scale = particle.lifetime / 1.0; // Assuming 1 second max lifetime
            transform.scale = Vec3::splat(scale * 0.2);
        }
    }
}

/// Update projectile trails
pub fn update_projectile_trails(
    mut query: Query<(&mut ProjectileTrail, &Transform), With<Projectile>>,
) {
    for (mut trail, transform) in query.iter_mut() {
        trail.positions.push(transform.translation);

        // Limit trail length
        if trail.positions.len() > trail.max_positions {
            trail.positions.remove(0);
        }
    }
}

/// Update shield visual effects
pub fn update_shield_effects(
    time: Res<Time>,
    mut query: Query<(
        &mut Transform,
        &mut ShieldEffect,
        &MeshMaterial3d<StandardMaterial>,
    )>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    for (mut transform, mut shield, mat_handle) in query.iter_mut() {
        shield.hit_time -= time.delta_seconds();

        // Pulse effect when hit
        if shield.hit_time > 0.0 {
            let pulse = (shield.hit_time * 20.0).sin() * 0.1 + 1.0;
            transform.scale = Vec3::splat(shield.radius * pulse);

            if let Some(material) = materials.get_mut(&mat_handle.0) {
                let intensity = shield.hit_time / 0.5; // 0.5 second hit effect
                material.emissive =
                    LinearRgba::rgb(0.5 * intensity, 0.8 * intensity, 1.0 * intensity);
            }
        } else {
            // Normal shield animation
            let pulse = (time.elapsed_seconds() * 2.0).sin() * 0.05 + 1.0;
            transform.scale = Vec3::splat(shield.radius * pulse);
        }
    }
}

/// Animate buff/debuff indicators
pub fn animate_buff_indicators(
    time: Res<Time>,
    mut query: Query<(&mut Transform, &BuffVisualIndicator)>,
) {
    for (mut transform, indicator) in query.iter_mut() {
        // Rotate and pulse based on effect type
        match indicator.effect_type {
            StatusEffectType::AttackSpeed(_) => {
                transform.rotate_y(3.0 * time.delta_seconds());
                let pulse = (time.elapsed_seconds() * 5.0).sin() * 0.1 + 1.0;
                transform.scale = Vec3::splat(indicator.base_scale * pulse);
            }
            StatusEffectType::Poison(_) | StatusEffectType::Burn(_) => {
                transform.rotate_x(1.0 * time.delta_seconds());
                transform.rotate_z(1.0 * time.delta_seconds());
            }
            StatusEffectType::Freeze => {
                // No rotation for freeze, just subtle pulse
                let pulse = (time.elapsed_seconds() * 1.0).sin() * 0.02 + 1.0;
                transform.scale = Vec3::splat(indicator.base_scale * pulse);
            }
            _ => {
                // Default rotation
                transform.rotate_y(1.0 * time.delta_seconds());
            }
        }
    }
}

/// Clean up expired visual effects
pub fn cleanup_expired_effects(
    _commands: Commands,
    _particles: Query<Entity, With<VisualCombatParticle>>,
    _damage_nums: Query<Entity, With<VisualDamageNumber>>,
) {
    // This is handled in individual update systems, but we could add additional cleanup here
}

// Helper functions for spawning specific effects

fn spawn_impact_particles(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    position: Vec3,
    damage_type: &DamageType,
) {
    let particle_type = match damage_type {
        DamageType::Physical => ParticleType::Blood,
        DamageType::Magic => ParticleType::Void,
        DamageType::Chaos => ParticleType::Explosion,
        _ => ParticleType::Blood,
    };

    let color = get_particle_color(&particle_type);

    // Spawn multiple particles
    for _ in 0..10 {
        commands.spawn((
            Name::new("ImpactParticle"),
            VisualCombatParticle {
                lifetime: 1.0,
                velocity: Vec3::new(
                    rand::random::<f32>() * 4.0 - 2.0,
                    rand::random::<f32>() * 5.0 + 2.0,
                    rand::random::<f32>() * 4.0 - 2.0,
                ),
                particle_type: particle_type.clone(),
            },
            Mesh3d(meshes.add(Sphere::new(0.1))),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: color,
                emissive: color_to_emissive(color),
                alpha_mode: AlphaMode::Blend,
                ..default()
            })),
            Transform::from_translation(position),
        ));
    }
}

fn spawn_death_particles(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    position: Vec3,
    faction: &Faction,
) {
    let particle_type = match faction {
        Faction::CrimsonCovenant => ParticleType::Blood,
        Faction::OrderOfTheDeep => ParticleType::Water,
        Faction::VoidSeekers => ParticleType::Void,
        Faction::Neutral => ParticleType::Explosion,
    };

    let color = get_particle_color(&particle_type);

    // Spawn death explosion particles
    for _ in 0..20 {
        commands.spawn((
            Name::new("DeathParticle"),
            VisualCombatParticle {
                lifetime: 1.5,
                velocity: Vec3::new(
                    rand::random::<f32>() * 8.0 - 4.0,
                    rand::random::<f32>() * 10.0,
                    rand::random::<f32>() * 8.0 - 4.0,
                ),
                particle_type: particle_type.clone(),
            },
            Mesh3d(meshes.add(Sphere::new(0.15))),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: color.with_alpha(0.8),
                emissive: color_to_emissive(color) * 2.0,
                alpha_mode: AlphaMode::Blend,
                ..default()
            })),
            Transform::from_translation(position),
        ));
    }
}

/// Create a projectile trail effect
pub fn create_projectile_trail(
    commands: &mut Commands,
    projectile_entity: Entity,
    _damage_type: &DamageType,
) -> Entity {
    commands
        .spawn((
            Name::new("ProjectileTrail"),
            ProjectileTrail {
                owner: projectile_entity,
                positions: Vec::new(),
                max_positions: 10,
            },
        ))
        .id()
}

/// Create a shield bubble effect
pub fn create_shield_effect(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    radius: f32,
) -> Entity {
    commands
        .spawn((
            Name::new("ShieldEffect"),
            ShieldEffect {
                radius,
                strength: 1.0,
                hit_time: 0.0,
            },
            Mesh3d(meshes.add(Sphere::new(radius))),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: Color::srgba(0.3, 0.6, 1.0, 0.2),
                alpha_mode: AlphaMode::Blend,
                emissive: LinearRgba::rgb(0.1, 0.3, 0.5),
                double_sided: true,
                cull_mode: None,
                ..default()
            })),
            Transform::default(),
        ))
        .id()
}

/// Create buff/debuff visual indicator
pub fn create_buff_indicator(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    effect_type: StatusEffectType,
) -> Entity {
    let (color, mesh) = match effect_type {
        StatusEffectType::AttackSpeed(_) => {
            (Color::srgb(1.0, 0.5, 0.0), meshes.add(Torus::new(0.1, 0.3)))
        }
        StatusEffectType::MovementSpeed(_) => (
            Color::srgb(0.0, 1.0, 0.5),
            meshes.add(Cylinder::new(0.2, 0.5)),
        ),
        StatusEffectType::Poison(_) => (Color::srgb(0.0, 0.8, 0.0), meshes.add(Sphere::new(0.2))),
        StatusEffectType::Burn(_) => (Color::srgb(1.0, 0.3, 0.0), meshes.add(Sphere::new(0.25))),
        StatusEffectType::Freeze => (
            Color::srgb(0.5, 0.8, 1.0),
            meshes.add(Cuboid::new(0.3, 0.3, 0.3)),
        ),
        _ => (Color::WHITE, meshes.add(Sphere::new(0.2))),
    };

    commands
        .spawn((
            Name::new("BuffIndicator"),
            BuffVisualIndicator {
                effect_type,
                base_scale: 1.0,
            },
            Mesh3d(mesh),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: color.with_alpha(0.7),
                emissive: color_to_emissive(color),
                alpha_mode: AlphaMode::Blend,
                ..default()
            })),
            Transform::from_translation(Vec3::Y * 3.0),
        ))
        .id()
}

// Helper functions

fn get_damage_color(damage_type: &DamageType, is_critical: bool) -> Color {
    if is_critical {
        Color::srgb(1.0, 0.8, 0.0) // Gold for critical
    } else {
        match damage_type {
            DamageType::Physical => Color::srgb(1.0, 1.0, 1.0),
            DamageType::Magic => Color::srgb(0.5, 0.0, 1.0),
            DamageType::True => Color::srgb(1.0, 0.0, 0.0),
            DamageType::Chaos => Color::srgb(0.8, 0.0, 0.8),
        }
    }
}

fn get_particle_color(particle_type: &ParticleType) -> Color {
    match particle_type {
        ParticleType::Blood => Color::srgb(0.8, 0.1, 0.1),
        ParticleType::Water => Color::srgb(0.1, 0.4, 0.8),
        ParticleType::Void => Color::srgb(0.5, 0.0, 0.8),
        ParticleType::Fire => Color::srgb(1.0, 0.5, 0.0),
        ParticleType::Ice => Color::srgb(0.6, 0.9, 1.0),
        ParticleType::Poison => Color::srgb(0.2, 0.8, 0.2),
        ParticleType::Holy => Color::srgb(1.0, 1.0, 0.5),
        ParticleType::Explosion => Color::srgb(1.0, 0.7, 0.0),
    }
}

fn color_to_emissive(color: Color) -> LinearRgba {
    let srgba = color.to_srgba();
    LinearRgba::rgb(srgba.red, srgba.green, srgba.blue)
}

// Add rand module for simple random numbers
mod rand {
    use std::sync::atomic::{AtomicU32, Ordering};

    static SEED: AtomicU32 = AtomicU32::new(0x12345678);

    pub fn random<T>() -> T
    where
        T: Random,
    {
        T::random()
    }

    pub trait Random {
        fn random() -> Self;
    }

    impl Random for f32 {
        fn random() -> Self {
            let mut seed = SEED.load(Ordering::Relaxed);
            seed = seed.wrapping_mul(1664525).wrapping_add(1013904223);
            SEED.store(seed, Ordering::Relaxed);
            (seed as f32) / (u32::MAX as f32)
        }
    }
}
impl bevy::prelude::Message for SpawnVisualDamageNumberEvent {}
impl bevy::prelude::Message for SpawnVisualDeathEffectEvent {}
