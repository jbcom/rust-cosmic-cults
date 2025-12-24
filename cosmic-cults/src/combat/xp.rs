// XP and progression system for Cosmic Cults
use bevy::prelude::*;
use serde::{Deserialize, Serialize};

pub struct XPPlugin;

impl Plugin for XPPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<XPGainEvent>()
            .add_event::<LevelUpEvent>()
            .add_systems(
                Update,
                (process_xp_events, check_level_ups).chain(),
            );
    }
}

#[derive(Component, Clone, Debug, Serialize, Deserialize)]
pub struct Experience {
    pub current: u32,
    pub total: u32,
    pub level: u32,
    pub next_level_xp: u32,
}

impl Default for Experience {
    fn default() -> Self {
        Self {
            current: 0,
            total: 0,
            level: 1,
            next_level_xp: 100,
        }
    }
}

impl Experience {
    pub fn add_xp(&mut self, amount: u32) {
        self.current += amount;
        self.total += amount;
    }

    pub fn can_level_up(&self) -> bool {
        self.current >= self.next_level_xp
    }

    pub fn level_up(&mut self) {
        self.current -= self.next_level_xp;
        self.level += 1;
        self.next_level_xp = 100 + (self.level * self.level * 50);
    }
}

#[derive(Event, Clone, Debug)]
pub struct XPGainEvent {
    pub entity: Entity,
    pub amount: u32,
    pub source: XPSource,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum XPSource {
    Kill(Entity),
    Assist(Entity),
    Damage(f32),
    Objective,
    Quest,
}

#[derive(Event, Clone, Debug)]
pub struct LevelUpEvent {
    pub entity: Entity,
    pub new_level: u32,
}

pub fn process_xp_events(
    mut xp_events: EventReader<XPGainEvent>,
    mut query: Query<&mut Experience>,
    mut level_up_events: EventWriter<LevelUpEvent>,
) {
    for event in xp_events.read() {
        if let Ok(mut experience) = query.get_mut(event.entity) {
            experience.add_xp(event.amount);

            while experience.can_level_up() {
                experience.level_up();
                level_up_events.write(LevelUpEvent {
                    entity: event.entity,
                    new_level: experience.level,
                });
            }
        }
    }
}

pub fn check_level_ups(mut level_up_events: EventReader<LevelUpEvent>) {
    for event in level_up_events.read() {
        info!("Entity {:?} leveled up to level {}!", event.entity, event.new_level);
    }
}
