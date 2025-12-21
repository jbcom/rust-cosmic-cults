use crate::{AuraBuff, AuraType, BaseStats, Leader, Unit};
use bevy::prelude::*;
#[cfg(feature = "web")]
use web_sys::console;

// Game state will be managed by bevy-web, not this crate

// Leadership building component for platform mechanics
#[derive(Component, Clone, Debug, Default)]
pub struct LeadershipBuilding {
    pub leader_entity: Option<Entity>,
    pub platform_type: String,
    pub bonuses_active: bool,
    pub destruction_triggers_retreat: bool,
}

// Defeat condition system - checks if critical leaders have died
pub fn defeat_condition_system(mut commands: Commands, mut leader_query: Query<&mut Leader>) {
    for mut leader in leader_query.iter_mut() {
        if leader.health <= 0.0 && leader.alive && leader.defeat_on_death {
            leader.alive = false;

            #[cfg(feature = "web")]
            console::log_1(
                &format!(
                    "Critical leader {} has fallen! Defeat condition triggered.",
                    leader.name
                )
                .into(),
            );

            // Game over handling will be done by bevy-web orchestration layer
            #[cfg(feature = "web")]
            console::log_1(&"Game Over!".into());
        }
    }
}

// Leader abilities system - handles special leader powers
pub fn leader_abilities_system(
    mut commands: Commands,
    time: Res<Time>,
    mut leader_query: Query<(&mut Leader, &Transform)>,
    unit_query: Query<(Entity, &Transform, &Unit), Without<Leader>>,
) {
    let current_time = time.elapsed_secs();

    for (mut leader, leader_transform) in leader_query.iter_mut() {
        if !leader.alive {
            continue;
        }

        // Ability 1: Combat buff (every 30 seconds)
        if current_time - leader.last_ability1_use >= 30.0 {
            use_ability1(
                &mut commands,
                &mut leader,
                leader_transform,
                &unit_query,
                current_time,
            );
        }

        // Ability 2: Area heal (every 45 seconds)
        if current_time - leader.last_ability2_use >= 45.0 {
            use_ability2(
                &mut commands,
                &mut leader,
                leader_transform,
                &unit_query,
                current_time,
            );
        }
    }
}

fn use_ability1(
    commands: &mut Commands,
    leader: &mut Leader,
    leader_transform: &Transform,
    unit_query: &Query<(Entity, &Transform, &Unit), Without<Leader>>,
    current_time: f32,
) {
    let mut affected_units = 0;

    for (unit_entity, unit_transform, unit) in unit_query.iter() {
        if unit.cult != leader.cult {
            continue;
        }

        let distance = leader_transform
            .translation
            .distance(unit_transform.translation);
        if distance <= 15.0 {
            // Ability radius
            // Apply temporary combat buff
            commands.spawn(AuraBuff {
                target_unit: unit_entity,
                team: 1,        // TODO: Get from unit
                atk_mul: 2.0,   // Double attack for 10 seconds
                hp_mul: 1.5,    // 50% more HP
                speed_mul: 1.3, // 30% speed boost
                xp_mul: 1.0,
                expires_at: current_time + 10.0,
                strength: 1.0,
            });
            affected_units += 1;
        }
    }

    if affected_units > 0 {
        leader.last_ability1_use = current_time;

        #[cfg(feature = "web")]
        console::log_1(
            &format!(
                "Leader {} used combat rally, affecting {} units",
                leader.name, affected_units
            )
            .into(),
        );
    }
}

fn use_ability2(
    commands: &mut Commands,
    leader: &mut Leader,
    leader_transform: &Transform,
    unit_query: &Query<(Entity, &Transform, &Unit), Without<Leader>>,
    current_time: f32,
) {
    let mut healed_units = 0;

    for (unit_entity, unit_transform, unit) in unit_query.iter() {
        if unit.cult != leader.cult {
            continue;
        }

        let distance = leader_transform
            .translation
            .distance(unit_transform.translation);
        if distance <= 25.0 {
            // Heal radius
            // TODO: Apply healing (would need mutable access to Unit health)
            // For now, just count the units that would be healed
            healed_units += 1;
        }
    }

    if healed_units > 0 {
        leader.last_ability2_use = current_time;

        #[cfg(feature = "web")]
        console::log_1(
            &format!(
                "Leader {} used area heal, affecting {} units",
                leader.name, healed_units
            )
            .into(),
        );
    }
}

// Platform building system functionality is implemented below at line 236

// Buff application system - actually applies stat modifications
pub fn buff_application_system(
    mut unit_query: Query<&mut Unit>,
    aura_buff_query: Query<&AuraBuff>,
    base_stats_query: Query<&BaseStats>,
) {
    for aura_buff in aura_buff_query.iter() {
        if let Ok(mut unit) = unit_query.get_mut(aura_buff.target_unit)
            && let Ok(base_stats) = base_stats_query.get(aura_buff.target_unit)
        {
            // Apply multiplicative bonuses using base stats
            // TODO: Add attack damage field to Unit or use separate attack component
            unit.max_health = base_stats.base_health * aura_buff.hp_mul;
            unit.movement_speed = base_stats.base_speed * aura_buff.speed_mul;
            unit.attack_damage = base_stats.base_attack_damage * aura_buff.atk_mul;
            // TODO: Add XP multiplier application
        }
    }
}

// Aura cleanup system - removes expired buffs
pub fn aura_cleanup_system(
    mut commands: Commands,
    time: Res<Time>,
    aura_query: Query<(Entity, &AuraBuff)>,
) {
    let current_time = time.elapsed_secs();

    for (entity, aura_buff) in aura_query.iter() {
        if current_time >= aura_buff.expires_at {
            commands.entity(entity).despawn();
        }
    }
}

// Aura range visualization system - disabled pending gizmos API updates
// TODO: Re-enable when aura visualization is needed

// Passive aura system - applies continuous aura effects
pub fn passive_aura_system(
    mut commands: Commands,
    time: Res<Time>,
    leader_query: Query<(&Transform, &Leader)>,
    unit_query: Query<(Entity, &Transform, &Unit), Without<Leader>>,
) {
    let current_time = time.elapsed_secs();

    for (leader_transform, leader) in leader_query.iter() {
        if !leader.alive {
            continue;
        }

        for (unit_entity, unit_transform, unit) in unit_query.iter() {
            if unit.cult != leader.cult {
                continue;
            }

            let distance = leader_transform
                .translation
                .distance(unit_transform.translation);
            if distance <= leader.aura_radius {
                // Apply aura buff based on leader type
                let (atk_mul, hp_mul, speed_mul, xp_mul) = match leader.aura_type {
                    AuraType::Crimson => (1.2, 1.0, 1.0, 1.0), // Attack bonus
                    AuraType::Deep => (1.0, 1.3, 1.0, 1.0),    // Health bonus
                    AuraType::Void => (1.0, 1.0, 1.2, 1.5),    // Speed and XP bonus
                    AuraType::Leadership => (1.1, 1.1, 1.1, 1.1), // Balanced bonus
                };

                commands.spawn(AuraBuff {
                    target_unit: unit_entity,
                    team: 1, // TODO: Get team from unit
                    atk_mul,
                    hp_mul,
                    speed_mul,
                    xp_mul,
                    expires_at: current_time + 1.0, // Short duration, continuously reapplied
                    strength: 1.0,
                });
            }
        }
    }
}

// Platform building system for leader platforms
pub fn platform_building_system(
    mut commands: Commands,
    leader_query: Query<(Entity, &Leader, &Transform)>,
    building_query: Query<(&LeadershipBuilding, &Transform)>,
) {
    // Simplified platform building logic - can be enhanced later
    for (leader_entity, leader, leader_transform) in leader_query.iter() {
        if leader.platform_entity.is_none() && leader.alive {
            // Basic platform creation logic could go here
            #[cfg(feature = "web")]
            console::log_1(&format!("Creating platform for leader {}", leader.name).into());
        }
    }
}
