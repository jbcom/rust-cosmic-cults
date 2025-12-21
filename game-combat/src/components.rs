// Core combat components that are shared across systems
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

/// Weapon component
#[derive(Component, Clone, Debug, Serialize, Deserialize)]
pub struct Weapon {
    pub weapon_type: WeaponType,
    pub range: f32,
    pub projectile_speed: Option<f32>,
    pub area_of_effect: Option<f32>,
    pub penetration: f32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum WeaponType {
    Melee,
    Ranged,
    Magic,
    Siege,
}

/// Damage types for resistance calculations
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum DamageType {
    Physical,
    Magic,
    True,  // Ignores armor/resist
    Chaos, // Lovecraftian - mixed damage
}

/// Team component for faction identification
#[derive(Component, Clone, Debug, Serialize, Deserialize)]
pub struct Team {
    pub id: u32,
    pub faction: Faction,
    pub color: Color,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Faction {
    OrderOfTheDeep,
    CrimsonCovenant,
    VoidSeekers,
    Neutral,
}

/// Unit type classification
#[derive(Component, Clone, Debug, Serialize, Deserialize)]
pub struct UnitType {
    pub classification: UnitClassification,
    pub is_flying: bool,
    pub is_mechanical: bool,
    pub is_biological: bool,
    pub is_ethereal: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum UnitClassification {
    Infantry,
    Vehicle,
    Monster,
    Hero,
    Building,
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

    // Lovecraftian effects
    Madness(f32),
    Corruption(f32),
    VoidTouch,
    DeepCurse,
}

/// Combat event tracking
#[derive(Component, Default)]
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

#[derive(Clone, Debug)]
pub struct AreaDamage {
    pub radius: f32,
    pub falloff: f32, // Damage reduction per unit distance
    pub friendly_fire: bool,
}

/// Shield component for extra protection
#[derive(Component, Clone, Debug)]
pub struct Shield {
    pub current: f32,
    pub maximum: f32,
    pub regeneration_rate: f32,
    pub regeneration_delay: f32,
    pub time_since_damage: f32,
}

/// Marker component for invulnerable entities
#[derive(Component)]
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
            time_until_next: 1.0 / base_attack_speed, // Initialize with proper cooldown
            attack_speed_modifier: 1.0,
        }
    }

    pub fn reset(&mut self, base_attack_speed: f32) {
        self.time_until_next = 1.0 / (base_attack_speed * self.attack_speed_modifier);
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

/// Resurrection component
#[derive(Component)]
pub struct Resurrectable {
    pub resurrect_time: f32,
    pub resurrect_health_percent: f32,
}
