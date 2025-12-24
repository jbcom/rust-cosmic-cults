// Visual effects system for Cosmic Cults
use bevy::pbr::StandardMaterial;
use bevy::prelude::*;
use bevy::render::alpha::AlphaMode;
use bevy_combat::prelude::*;

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

pub struct CombatVisualsPlugin;

impl Plugin for CombatVisualsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                spawn_damage_numbers,
                update_damage_numbers,
                apply_hit_flash,
                update_hit_flash,
                handle_death_effects,
                update_death_effects,
                update_combat_particles,
            ),
        );
    }
}

pub fn spawn_damage_numbers(
    mut commands: Commands,
    mut damage_events: EventReader<DamageEvent>,
    transform_query: Query<&Transform>,
) {
    for event in damage_events.read() {
        if let Ok(transform) = transform_query.get(event.target) {
            let size = if event.is_critical { 0.5 } else { 0.3 };

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

            commands.entity(event.target).insert(HitFlash {
                duration: 0.2,
                elapsed: 0.0,
                original_color: Color::WHITE,
            });
        }
    }
}

pub fn update_damage_numbers(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut Transform, &mut VisualDamageNumber)>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    for (entity, mut transform, mut damage_num) in query.iter_mut() {
        damage_num.lifetime -= time.delta_secs();

        if damage_num.lifetime <= 0.0 {
            commands.entity(entity).despawn();
        } else {
            transform.translation += damage_num.velocity * time.delta_secs();
            damage_num.velocity.y -= 2.0 * time.delta_secs();

            let alpha = damage_num.lifetime / 1.5;
            transform.scale = Vec3::splat(alpha * (if damage_num.is_critical { 0.5 } else { 0.3 }));

            if damage_num.lifetime > 1.4 {
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

pub fn apply_hit_flash(
    mut material_query: Query<(&MeshMaterial3d<StandardMaterial>, &HitFlash), Added<HitFlash>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    for (mat_handle, _flash) in material_query.iter_mut() {
        if let Some(material) = materials.get_mut(&mat_handle.0) {
            material.base_color = Color::srgb(1.0, 0.2, 0.2);
            material.emissive = LinearRgba::rgb(1.0, 0.0, 0.0);
        }
    }
}

pub fn update_hit_flash(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut HitFlash, &MeshMaterial3d<StandardMaterial>)>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    for (entity, mut flash, mat_handle) in query.iter_mut() {
        flash.elapsed += time.delta_secs();

        if flash.elapsed >= flash.duration {
            if let Some(material) = materials.get_mut(&mat_handle.0) {
                material.base_color = flash.original_color;
                material.emissive = LinearRgba::BLACK;
            }
            commands.entity(entity).remove::<HitFlash>();
        }
    }
}

pub fn handle_death_effects(
    mut death_events: EventReader<DeathEvent>,
    mut commands: Commands,
) {
    for event in death_events.read() {
        commands.entity(event.entity).insert(VisualDeathEffect {
            fade_time: 1.0,
            time_elapsed: 0.0,
            particle_spawned: false,
        });
    }
}

pub fn update_death_effects(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut VisualDeathEffect, &MeshMaterial3d<StandardMaterial>)>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    for (entity, mut death_effect, mat_handle) in query.iter_mut() {
        death_effect.time_elapsed += time.delta_secs();

        if death_effect.time_elapsed >= death_effect.fade_time {
            commands.entity(entity).despawn();
        } else {
            let alpha = 1.0 - (death_effect.time_elapsed / death_effect.fade_time);
            if let Some(material) = materials.get_mut(&mat_handle.0) {
                material.base_color = material.base_color.with_alpha(alpha);
                material.alpha_mode = AlphaMode::Blend;
            }
        }
    }
}

pub fn update_combat_particles(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut Transform, &mut VisualCombatParticle)>,
) {
    for (entity, mut transform, mut particle) in query.iter_mut() {
        particle.lifetime -= time.delta_secs();

        if particle.lifetime <= 0.0 {
            commands.entity(entity).despawn();
        } else {
            transform.translation += particle.velocity * time.delta_secs();
            let scale = particle.lifetime / 1.0;
            transform.scale = Vec3::splat(scale * 0.2);
        }
    }
}

fn get_damage_color(damage_type: &DamageType, is_critical: bool) -> Color {
    if is_critical {
        Color::srgb(1.0, 0.8, 0.0)
    } else {
        match damage_type {
            DamageType::Physical => Color::srgb(1.0, 1.0, 1.0),
            DamageType::Magic => Color::srgb(0.5, 0.0, 1.0),
            DamageType::True => Color::srgb(1.0, 0.0, 0.0),
            DamageType::Custom(s) if s == "Chaos" => Color::srgb(0.8, 0.0, 0.8),
            _ => Color::WHITE,
        }
    }
}

fn color_to_emissive(color: Color) -> LinearRgba {
    let srgba = color.to_srgba();
    LinearRgba::rgb(srgba.red, srgba.green, srgba.blue)
}

mod rand {
    use std::sync::atomic::{AtomicU32, Ordering};
    static SEED: AtomicU32 = AtomicU32::new(0x12345678);
    pub fn random<T: Random>() -> T { T::random() }
    pub trait Random { fn random() -> Self; }
    impl Random for f32 {
        fn random() -> Self {
            let mut seed = SEED.load(Ordering::Relaxed);
            seed = seed.wrapping_mul(1664525).wrapping_add(1013904223);
            SEED.store(seed, Ordering::Relaxed);
            (seed as f32) / (u32::MAX as f32)
        }
    }
}
