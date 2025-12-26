// Visual effects and feedback systems for combat
use crate::visuals::ParticleType;
use bevy::prelude::*;

pub struct EffectsPlugin;

impl Plugin for EffectsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                damage_number_system,
                health_bar_system,
                death_effect_system,
                combat_particle_system,
            ),
        );
    }
}

/// Component for floating damage numbers
#[derive(Component)]
pub struct DamageNumber {
    pub amount: f32,
    pub color: Color,
    pub lifetime: f32,
    pub velocity: Vec3,
}

/// Component for health bars
#[derive(Component)]
pub struct HealthBar {
    pub offset: Vec3,
    pub width: f32,
    pub height: f32,
}

/// Component for death effects
#[derive(Component)]
pub struct DeathEffect {
    pub effect_type: DeathEffectType,
    pub duration: f32,
    pub remaining: f32,
}

#[derive(Clone)]
pub enum DeathEffectType {
    Explosion,
    Dissolve,
    Fade,
    Shatter,
}

/// System to handle floating damage numbers
pub fn damage_number_system(
    mut commands: Commands,
    mut query: Query<(Entity, &mut Transform, &mut DamageNumber)>,
    time: Res<Time>,
) {
    for (entity, mut transform, mut damage_number) in query.iter_mut() {
        damage_number.lifetime -= time.delta_secs();

        if damage_number.lifetime <= 0.0 {
            commands.entity(entity).despawn();
        } else {
            // Float upward and fade
            transform.translation += damage_number.velocity * time.delta_secs();
            damage_number.velocity.y -= 5.0 * time.delta_secs(); // Gravity
        }
    }
}

/// System to update health bars
pub fn health_bar_system(
    mut gizmos: Gizmos,
    query: Query<(&Transform, &crate::states::Health, &HealthBar)>,
) {
    for (transform, health, health_bar) in query.iter() {
        let position = transform.translation + health_bar.offset;
        let percentage = health.current / health.maximum;

        // Background
        gizmos.rect_2d(
            position.truncate(),
            Vec2::new(health_bar.width, health_bar.height),
            Color::srgb(0.2, 0.2, 0.2),
        );

        // Health fill
        let health_color = if percentage > 0.6 {
            Color::srgb(0.2, 0.8, 0.2)
        } else if percentage > 0.3 {
            Color::srgb(0.8, 0.8, 0.2)
        } else {
            Color::srgb(0.8, 0.2, 0.2)
        };

        let health_position = position - Vec3::X * (health_bar.width * (1.0 - percentage) / 2.0);
        gizmos.rect_2d(
            health_position.truncate(),
            Vec2::new(health_bar.width * percentage, health_bar.height),
            health_color,
        );
    }
}

/// System to handle death effects
pub fn death_effect_system(
    mut commands: Commands,
    mut query: Query<(Entity, &mut DeathEffect)>,
    time: Res<Time>,
) {
    for (entity, mut effect) in query.iter_mut() {
        effect.remaining -= time.delta_secs();

        if effect.remaining <= 0.0 {
            commands.entity(entity).despawn();
        }
    }
}

/// System to handle combat particle effects
pub fn combat_particle_system(mut gizmos: Gizmos, query: Query<(&Transform, &CombatParticle)>) {
    for (transform, particle) in query.iter() {
        // Simple particle visualization using circle instead of sphere
        gizmos.circle_2d(
            transform.translation.truncate(),
            particle.size,
            particle.color,
        );
    }
}

/// Component for combat particles
#[derive(Component)]
pub struct CombatParticle {
    pub particle_type: ParticleType,
    pub size: f32,
    pub color: Color,
    pub lifetime: f32,
}

// Note: ParticleType is defined in visuals.rs to avoid duplication

/// Spawn a damage number at a position
pub fn spawn_damage_number(
    commands: &mut Commands,
    position: Vec3,
    damage: f32,
    is_critical: bool,
) {
    commands.spawn((
        DamageNumber {
            amount: damage,
            color: if is_critical {
                Color::srgb(1.0, 0.8, 0.0)
            } else {
                Color::srgb(1.0, 0.3, 0.3)
            },
            lifetime: 2.0,
            velocity: Vec3::new(
                rand::random::<f32>() * 2.0 - 1.0,
                5.0,
                rand::random::<f32>() * 2.0 - 1.0,
            ),
        },
        Transform::from_translation(position + Vec3::Y * 2.0),
    ));
}

/// Spawn a death effect
pub fn spawn_death_effect(commands: &mut Commands, position: Vec3, effect_type: DeathEffectType) {
    commands.spawn((
        DeathEffect {
            effect_type,
            duration: 1.0,
            remaining: 1.0,
        },
        Transform::from_translation(position),
    ));
}
