// Main combat plugin that integrates all subsystems
use bevy::prelude::*;
use seldom_state::prelude::*;

pub struct CombatPlugin;

impl Plugin for CombatPlugin {
    fn build(&self, app: &mut App) {
        app
            // Add state machine plugin
            .add_plugins(StateMachinePlugin::default())
            
            // Register events
            .add_event::<crate::damage::DamageEvent>()
            .add_event::<crate::damage::DeathEvent>()
            .add_event::<crate::targeting::TargetAcquiredEvent>()
            .add_event::<crate::targeting::TargetLostEvent>()
            .add_event::<crate::xp::XPGainEvent>()
            .add_event::<crate::xp::LevelUpEvent>()
            
            // Add core combat systems
            .add_systems(Update, (
                // Targeting
                crate::targeting::target_acquisition_system,
                crate::targeting::target_validation_system,
                crate::targeting::line_of_sight_system,
                
                // Combat execution
                crate::systems::combat_execution_system,
                crate::systems::update_attack_timers,
                
                // Damage
                crate::damage::process_damage_events,
                crate::damage::apply_damage_modifiers,
                crate::damage::check_for_deaths,
                
                // Effects
                crate::systems::status_effect_system,
                crate::systems::shield_regeneration_system,
                crate::systems::projectile_system,
                
                // XP
                crate::xp::process_xp_events,
                crate::xp::check_level_ups,
                crate::xp::apply_level_bonuses,
                
                // Visual
                crate::effects::damage_number_system,
                crate::effects::health_bar_system,
                crate::effects::death_effect_system,
                crate::effects::combat_particle_system,
                
                // Cleanup
                crate::systems::cleanup_dead_entities,
                crate::systems::combat_log_system,
            ).chain());
    }
}
