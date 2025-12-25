// Main combat plugin
use bevy::prelude::*;
pub struct CombatPlugin;
impl Plugin for CombatPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                crate::systems::combat_execution_system,
                crate::systems::update_attack_timers,
                crate::systems::status_effect_system,
                crate::systems::shield_regeneration_system,
                crate::systems::projectile_system,
                crate::systems::cleanup_dead_entities,
                crate::systems::combat_log_system,
            )
                .chain(),
        );
    }
}
