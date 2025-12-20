use bevy::prelude::*;
use serde::{Deserialize, Serialize};
#[derive(Clone, Component, Debug, Default, Reflect, Serialize, Deserialize)]
#[reflect(Component)]
pub enum CombatState {
    #[default]
    Idle,
    Searching,
    Engaging(Entity),
    Attacking(Entity),
    Cooldown(f32),
    Retreating,
    Dead,
}
#[derive(Component)]
pub struct Enemy;
#[derive(Component)]
pub struct AttackTimer {
    timer: Timer,
}
impl AttackTimer {
    pub fn new(duration: f32) -> Self {
        Self {
            timer: Timer::from_seconds(duration, TimerMode::Once),
        }
    }
    pub fn finished(&self) -> bool {
        self.timer.is_finished()
    }
    pub fn duration(&self) -> std::time::Duration {
        self.timer.duration()
    }
    pub fn tick(&mut self, delta: std::time::Duration) {
        self.timer.tick(delta);
    }
}
#[derive(Component)]
pub struct Health {
    pub current: f32,
    pub maximum: f32,
}
impl Health {
    pub fn percentage(&self) -> f32 {
        self.current / self.maximum
    }
}
