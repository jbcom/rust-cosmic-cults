use bevy::prelude::*;
use serde::{Deserialize, Serialize};

/// Combat stats for units
#[derive(Component, Clone, Debug, Serialize, Deserialize)]
pub struct CombatStats {
    pub damage: f32,
    pub attack_speed: f32,
    pub armor: f32,
    pub magic_resist: f32,
    pub critical_chance: f32,
    pub critical_damage: f32,
    pub life_steal: f32,
}

impl Default for CombatStats {
    fn default() -> Self {
        Self {
            damage: 10.0,
            attack_speed: 1.0,
            armor: 0.0,
            magic_resist: 0.0,
            critical_chance: 0.1,
            critical_damage: 2.0,
            life_steal: 0.0,
        }
    }
}

/// Health component for entities that can take damage
#[derive(Component, Clone, Debug, Serialize, Deserialize)]
pub struct Health {
    pub current: f32,
    pub maximum: f32,
}

impl Health {
    pub fn new(maximum: f32) -> Self {
        Self {
            current: maximum,
            maximum,
        }
    }

    pub fn percentage(&self) -> f32 {
        if self.maximum <= 0.0 {
            0.0
        } else {
            self.current / self.maximum
        }
    }

    pub fn is_dead(&self) -> bool {
        self.current <= 0.0
    }
}

/// Weapon component
#[derive(Component, Clone, Debug, Serialize, Deserialize)]
pub struct Weapon {
    pub weapon_type: WeaponType,
    pub range: f32,
    pub projectile_speed: Option<f32>,
    pub area_of_effect: Option<f32>,
    pub penetration: f32,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum WeaponType {
    Melee,
    Ranged,
    Magic,
    Siege,
    Custom(String),
}

/// Damage types for resistance calculations
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum DamageType {
    Physical,
    Magic,
    True,
    Custom(String),
}

/// Team component for faction identification
#[derive(Component, Clone, Debug, Serialize, Deserialize)]
pub struct Team {
    pub id: u32,
}

/// Buff/Debuff component
#[derive(Component, Clone, Debug)]
pub struct StatusEffect {
    pub effect_type: StatusEffectType,
    pub duration: f32,
    pub remaining: f32,
    pub stacks: u32,
    pub source: Option<Entity>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum StatusEffectType {
    // Buffs
    AttackSpeed(f32),
    MovementSpeed(f32),
    DamageBoost(f32),
    ArmorBoost(f32),
    Regeneration(f32),

    // Debuffs
    Slow(f32),
    Stun,
    Silence,
    Blind,
    Poison(f32),
    Burn(f32),
    Freeze,
    
    // Custom
    Custom(String),
}

/// Combat event tracking
#[derive(Component, Default, Clone, Debug, Serialize, Deserialize)]
pub struct CombatLog {
    pub damage_dealt: f32,
    pub damage_taken: f32,
    pub healing_done: f32,
    pub kills: u32,
    pub assists: u32,
    pub last_combat_time: f32,
}

/// Projectile component
#[derive(Component)]
pub struct Projectile {
    pub owner: Entity,
    pub damage: f32,
    pub damage_type: DamageType,
    pub speed: f32,
    pub lifetime: f32,
    pub remaining_lifetime: f32,
    pub pierce_count: u32,
    pub area_damage: Option<AreaDamage>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AreaDamage {
    pub radius: f32,
    pub falloff: f32,
    pub friendly_fire: bool,
}

/// Shield component for extra protection
#[derive(Component, Clone, Debug, Serialize, Deserialize)]
pub struct Shield {
    pub current: f32,
    pub maximum: f32,
    pub regeneration_rate: f32,
    pub regeneration_delay: f32,
    pub time_since_damage: f32,
}

/// Marker component for invulnerable entities
#[derive(Component, Clone, Debug, Serialize, Deserialize)]
pub struct Invulnerable {
    pub duration: Option<f32>,
    pub remaining: f32,
}

/// Attack cooldown tracking
#[derive(Component)]
pub struct AttackCooldown {
    pub time_until_next: f32,
    pub attack_speed_modifier: f32,
}

impl AttackCooldown {
    pub fn new(base_attack_speed: f32) -> Self {
        Self {
            time_until_next: if base_attack_speed > 0.0 { 1.0 / base_attack_speed } else { 0.0 },
            attack_speed_modifier: 1.0,
        }
    }

    pub fn tick(&mut self, delta: f32) -> bool {
        self.time_until_next -= delta;
        self.time_until_next <= 0.0
    }
}

/// Death marker component
#[derive(Component)]
pub struct Dead {
    pub killer: Option<Entity>,
    pub death_time: f32,
}
