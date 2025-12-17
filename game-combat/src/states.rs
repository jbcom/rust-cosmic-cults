// Combat state machine using seldom_state for clean state management
use bevy::prelude::*;
use seldom_state::prelude::*;
use serde::{Deserialize, Serialize};

/// Main combat state for units
#[derive(Clone, Component, Debug, Default, Reflect)]
#[derive(Serialize, Deserialize)]
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

/// State machine component for combat
#[derive(Component)]
pub struct CombatStateMachine;

/// Build the combat state machine
pub fn build_combat_state_machine() -> StateMachine {
    // Simplified state machine for now - seldom_state API needs proper integration
    StateMachine::default()
}

// State markers for type safety
#[derive(Clone, Copy)]
pub struct Idle;

#[derive(Clone, Copy)]
pub struct Searching;

#[derive(Clone, Copy)]
pub struct Engaging;

#[derive(Clone, Copy)]
pub struct Attacking;

#[derive(Clone, Copy)]
pub struct Cooldown;

#[derive(Clone, Copy)]
pub struct Retreating;

#[derive(Clone, Copy)]
pub struct DeadState;

#[derive(Clone, Copy)]
pub struct Any;

// These functions would be used when properly integrating the state machine
// For now, they return closures that check combat conditions

#[allow(dead_code)]
fn enemy_exists() -> impl Fn() -> bool {
    // Check if any enemies exist in the world
    || true  // Will check entity queries when integrated
}

#[allow(dead_code)]
fn enemy_in_range() -> impl Fn() -> bool {
    // Check if enemy is within detection range
    || false  // Will calculate distances when integrated
}

#[allow(dead_code)]
fn enemy_in_sight() -> impl Fn() -> Option<Entity> {
    // Check line of sight to enemy
    || None  // Will perform raycasts when integrated
}

#[allow(dead_code)]
fn enemy_in_attack_range() -> impl Fn() -> Option<Entity> {
    // Check if target is in attack range
    || None  // Will check attack range when integrated
}

#[allow(dead_code)]
fn attack_complete() -> impl Fn() -> Option<f32> {
    // Check if attack animation/cooldown is complete
    || None  // Will check timer completion when integrated
}

#[allow(dead_code)]
fn cooldown_complete() -> impl Fn() -> bool {
    // Check if cooldown period is over
    || false  // Will check cooldown timer when integrated
}

#[allow(dead_code)]
fn health_critical() -> impl Fn() -> bool {
    // Check if health is below critical threshold
    || false  // Will check health percentage when integrated
}

#[allow(dead_code)]
fn health_zero() -> impl Fn() -> bool {
    // Check if unit is dead  
    || false  // Will check health value when integrated
}

// Components referenced by the state machine
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
        self.timer.finished()
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
