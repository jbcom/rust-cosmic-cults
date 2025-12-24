use crate::components::Health;
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

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DeathEffectType {
    Explosion,
    Dissolve,
    Fade,
    Shatter,
    Custom(String),
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
            transform.translation += damage_number.velocity * time.delta_secs();
            damage_number.velocity.y -= 5.0 * time.delta_secs();
        }
    }
}

/// System to update health bars
pub fn health_bar_system(
    mut gizmos: Gizmos,
    query: Query<(&GlobalTransform, &Health, &HealthBar)>,
) {
    for (transform, health, health_bar) in query.iter() {
        let position = transform.translation() + health_bar.offset;
        let percentage = health.percentage();

        gizmos.rect(
            position,
            Quat::IDENTITY,
            Vec2::new(health_bar.width, health_bar.height),
            Color::srgb(0.2, 0.2, 0.2),
        );

        let health_color = if percentage > 0.6 {
            Color::srgb(0.2, 0.8, 0.2)
        } else if percentage > 0.3 {
            Color::srgb(0.8, 0.8, 0.2)
        } else {
            Color::srgb(0.8, 0.2, 0.2)
        };

        let health_width = health_bar.width * percentage;
        let health_position = position - Vec3::X * (health_bar.width - health_width) / 2.0;
        
        gizmos.rect(
            health_position,
            Quat::IDENTITY,
            Vec2::new(health_width, health_bar.height),
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
