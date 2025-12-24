// Damage calculation and application system
use crate::components::*;
use crate::states::Health;
use bevy::prelude::*;

pub struct DamagePlugin;

impl Plugin for DamagePlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<DamageEvent>()
            .add_message::<DeathEvent>()
            .add_systems(
                Update,
                (
                    process_damage_events,
                    apply_damage_modifiers,
                    check_for_deaths,
                )
                    .chain(),
            );
    }
}

#[derive(Event, Clone, Debug)]
pub struct DamageEvent {
    pub attacker: Entity,
    pub target: Entity,
    pub amount: f32,
    pub damage_type: DamageType,
    pub is_critical: bool,
}

#[derive(Event, Clone, Debug)]
pub struct DeathEvent {
    pub entity: Entity,
    pub killer: Option<Entity>,
}

/// Process damage events and apply damage
pub fn process_damage_events(
    mut damage_events: MessageReader<DamageEvent>,
    mut health_query: Query<&mut Health>,
    combat_stats_query: Query<&CombatStats>,
    mut shield_query: Query<&mut Shield>,
    invulnerable_query: Query<&Invulnerable>,
    mut death_events: MessageWriter<DeathEvent>,
) {
    for event in damage_events.read() {
        // Skip if target is invulnerable
        if invulnerable_query.contains(event.target) {
            continue;
        }

        // Calculate final damage after resistances
        let final_damage = calculate_damage(
            event.amount,
            &event.damage_type,
            combat_stats_query.get(event.target).ok(),
        );

        // Apply damage to shield first, then health
        let remaining_damage = if let Ok(mut shield) = shield_query.get_mut(event.target) {
            apply_shield_damage(&mut shield, final_damage)
        } else {
            final_damage
        };

        if let Ok(mut health) = health_query.get_mut(event.target) {
            health.current = (health.current - remaining_damage).max(0.0);

            if health.current <= 0.0 {
                death_events.write(DeathEvent {
                    entity: event.target,
                    killer: Some(event.attacker),
                });
            }
        }
    }
}

fn calculate_damage(
    base_damage: f32,
    damage_type: &DamageType,
    target_stats: Option<&CombatStats>,
) -> f32 {
    if let Some(stats) = target_stats {
        match damage_type {
            DamageType::Physical => {
                // Armor reduces physical damage
                base_damage * (100.0 / (100.0 + stats.armor))
            }
            DamageType::Magic => {
                // Magic resist reduces magic damage
                base_damage * (100.0 / (100.0 + stats.magic_resist))
            }
            DamageType::True => {
                // True damage ignores resistances
                base_damage
            }
            DamageType::Chaos => {
                // Chaos damage is 50% physical, 50% magic
                let physical = base_damage * 0.5 * (100.0 / (100.0 + stats.armor));
                let magic = base_damage * 0.5 * (100.0 / (100.0 + stats.magic_resist));
                physical + magic
            }
        }
    } else {
        base_damage
    }
}

fn apply_shield_damage(shield: &mut Shield, damage: f32) -> f32 {
    if shield.current > 0.0 {
        let absorbed = shield.current.min(damage);
        shield.current -= absorbed;
        shield.time_since_damage = 0.0;
        damage - absorbed
    } else {
        damage
    }
}

/// Apply damage over time effects
pub fn apply_damage_modifiers(mut query: Query<(&mut Health, &StatusEffect)>, time: Res<Time>) {
    for (mut health, status) in query.iter_mut() {
        match &status.effect_type {
            StatusEffectType::Poison(damage_per_second) => {
                health.current -= damage_per_second * time.delta_secs();
            }
            StatusEffectType::Burn(damage_per_second) => {
                health.current -= damage_per_second * time.delta_secs();
            }
            StatusEffectType::Regeneration(heal_per_second) => {
                health.current =
                    (health.current + heal_per_second * time.delta_secs()).min(health.maximum);
            }
            _ => {}
        }
    }
}

/// Check for deaths and mark entities
pub fn check_for_deaths(
    mut commands: Commands,
    query: Query<(Entity, &Health), Without<Dead>>,
    mut death_events: MessageWriter<DeathEvent>,
) {
    for (entity, health) in query.iter() {
        if health.current <= 0.0 {
            commands.entity(entity).insert(Dead {
                killer: None,
                death_time: 0.0,
            });

            death_events.write(DeathEvent {
                entity,
                killer: None,
            });
        }
    }
}
impl bevy::prelude::Message for DamageEvent {}
impl bevy::prelude::Message for DeathEvent {}
