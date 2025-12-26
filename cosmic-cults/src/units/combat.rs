use bevy::prelude::*;
use crate::units::components::*;

#[derive(Message, Debug, Clone)]
pub struct DamageEvent {
    pub target: Entity,
    pub damage: f32,
    pub attacker: Entity,
}

pub fn combat_plugin(app: &mut App) {
    app.add_message::<DamageEvent>()
       .add_systems(Update, (
           handle_damage_events,
           death_system,
       ));
}

fn handle_damage_events(
    mut events: MessageReader<DamageEvent>,
    mut health_query: Query<&mut Health>,
) {
    for event in events.read() {
        if let Ok(mut health) = health_query.get_mut(event.target) {
            health.current -= event.damage;
        }
    }
}

fn death_system(
    query: Query<(Entity, &Health), Changed<Health>>,
) {
    for (_entity, health) in query.iter() {
        if health.current <= 0.0 {
            // Visual systems handle the rest
        }
    }
}
