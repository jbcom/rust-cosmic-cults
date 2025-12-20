// Main combat systems that orchestrate the combat flow
use crate::components::*;
use crate::damage::*;
use crate::states::*;
use crate::targeting::*;
use bevy::prelude::*;

/// Main combat execution system
pub fn combat_execution_system(
    mut query: Query<(
        Entity,
        &CombatState,
        &TargetingSystem,
        &CombatStats,
        &mut AttackCooldown,
        &Transform,
    )>,
    target_query: Query<&Transform>,
    mut damage_events: MessageWriter<DamageEvent>,
    time: Res<Time>,
) {
    for (entity, state, targeting, stats, mut cooldown, transform) in query.iter_mut() {
        // Only attack if we're in the attacking state
        if let CombatState::Attacking(_target) = state {
            if cooldown.tick(time.delta_secs()) {
                // Check if target is still valid and in range
                if let Some(current_target) = targeting.current_target {
                    if let Ok(target_transform) = target_query.get(current_target) {
                        let distance = transform.translation.distance(target_transform.translation);

                        if distance <= targeting.range {
                            // Calculate damage
                            let is_critical = rand::random::<f32>() < stats.critical_chance;
                            let damage = if is_critical {
                                stats.damage * stats.critical_damage
                            } else {
                                stats.damage
                            };

                            // Send damage event
                            damage_events.write(DamageEvent {
                                attacker: entity,
                                target: current_target,
                                amount: damage,
                                damage_type: DamageType::Physical,
                                is_critical,
                            });

                            // Reset cooldown
                            cooldown.reset(stats.attack_speed);
                        }
                    }
                }
            }
        }
    }
}

/// System to update attack timers
pub fn update_attack_timers(mut query: Query<&mut AttackTimer>, time: Res<Time>) {
    for mut timer in query.iter_mut() {
        timer.tick(time.delta());
    }
}

/// System to handle status effect durations
pub fn status_effect_system(
    mut commands: Commands,
    mut query: Query<(Entity, &mut StatusEffect)>,
    time: Res<Time>,
) {
    for (entity, mut status) in query.iter_mut() {
        status.remaining -= time.delta_secs();

        if status.remaining <= 0.0 {
            commands.entity(entity).remove::<StatusEffect>();
        }
    }
}

/// System to handle shield regeneration
pub fn shield_regeneration_system(mut query: Query<&mut Shield>, time: Res<Time>) {
    for mut shield in query.iter_mut() {
        shield.time_since_damage += time.delta_secs();

        if shield.time_since_damage >= shield.regeneration_delay {
            shield.current =
                (shield.current + shield.regeneration_rate * time.delta_secs()).min(shield.maximum);
        }
    }
}

/// System to update combat logs
pub fn combat_log_system(
    mut damage_events: MessageReader<DamageEvent>,
    mut death_events: MessageReader<DeathEvent>,
    mut query: Query<&mut CombatLog>,
    time: Res<Time>,
) {
    // Update damage dealt
    for event in damage_events.read() {
        if let Ok(mut log) = query.get_mut(event.attacker) {
            log.damage_dealt += event.amount;
            log.last_combat_time = time.elapsed_secs();
        }

        if let Ok(mut log) = query.get_mut(event.target) {
            log.damage_taken += event.amount;
            log.last_combat_time = time.elapsed_secs();
        }
    }

    // Update kills
    for event in death_events.read() {
        if let Some(killer) = event.killer {
            if let Ok(mut log) = query.get_mut(killer) {
                log.kills += 1;
            }
        }
    }
}

/// System to handle projectiles
pub fn projectile_system(
    mut commands: Commands,
    mut query: Query<(Entity, &mut Projectile, &Transform)>,
    target_query: Query<(Entity, &Transform)>,
    time: Res<Time>,
    mut damage_events: MessageWriter<DamageEvent>,
) {
    for (entity, mut projectile, transform) in query.iter_mut() {
        projectile.remaining_lifetime -= time.delta_secs();

        if projectile.remaining_lifetime <= 0.0 {
            commands.entity(entity).despawn();
            continue;
        }

        // Check for collisions (simplified - would use Rapier in production)
        for (target_entity, target_transform) in target_query.iter() {
            let distance = transform.translation.distance(target_transform.translation);

            if distance < 1.0 {
                // Hit detection radius
                damage_events.write(DamageEvent {
                    attacker: projectile.owner,
                    target: target_entity,
                    amount: projectile.damage,
                    damage_type: projectile.damage_type.clone(),
                    is_critical: false,
                });

                // Handle area damage
                if let Some(_area) = &projectile.area_damage {
                    // TODO: Implement area damage calculation
                }

                // Destroy projectile unless it pierces
                if projectile.pierce_count == 0 {
                    commands.entity(entity).despawn();
                } else {
                    projectile.pierce_count -= 1;
                }

                break;
            }
        }
    }
}

/// System to clean up dead entities
pub fn cleanup_dead_entities(
    mut commands: Commands,
    query: Query<(Entity, &Dead)>,
    time: Res<Time>,
) {
    for (entity, dead) in query.iter() {
        // Wait a bit before despawning to allow death animations
        if time.elapsed_secs() - dead.death_time > 2.0 {
            commands.entity(entity).despawn();
        }
    }
}
