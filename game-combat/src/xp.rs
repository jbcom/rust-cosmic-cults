// XP and progression system
use bevy::prelude::*;
use serde::{Deserialize, Serialize};

pub struct XPPlugin;

impl Plugin for XPPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_event::<XPGainEvent>()
            .add_event::<LevelUpEvent>()
            .add_systems(Update, (
                process_xp_events,
                check_level_ups,
                apply_level_bonuses,
            ).chain());
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
        self.next_level_xp = calculate_next_level_xp(self.level);
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

#[derive(Component, Clone, Debug)]
pub struct VeteranStatus {
    pub tier: VeteranTier,
    pub kill_count: u32,
    pub total_damage: f32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum VeteranTier {
    Recruit,
    Regular,
    Veteran,
    Elite,
    Champion,
    Legendary,
}

impl VeteranTier {
    pub fn stat_multiplier(&self) -> f32 {
        match self {
            VeteranTier::Recruit => 1.0,
            VeteranTier::Regular => 1.1,
            VeteranTier::Veteran => 1.25,
            VeteranTier::Elite => 1.4,
            VeteranTier::Champion => 1.6,
            VeteranTier::Legendary => 2.0,
        }
    }
}

fn calculate_next_level_xp(level: u32) -> u32 {
    // Exponential growth formula
    100 + (level * level * 50)
}

pub fn process_xp_events(
    mut xp_events: EventReader<XPGainEvent>,
    mut query: Query<&mut Experience>,
    mut level_up_events: EventWriter<LevelUpEvent>,
) {
    for event in xp_events.read() {
        if let Ok(mut experience) = query.get_mut(event.entity) {
            experience.add_xp(event.amount);
            
            // Check for level up
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

pub fn check_level_ups(
    mut level_up_events: EventReader<LevelUpEvent>,
) {
    for event in level_up_events.read() {
        // Log level ups
        #[cfg(target_arch = "wasm32")]
        web_sys::console::log_1(&format!(
            "Entity {:?} leveled up to level {}!",
            event.entity, event.new_level
        ).into());
        #[cfg(not(target_arch = "wasm32"))]
        println!("Entity {:?} leveled up to level {}!", event.entity, event.new_level);
    }
}

pub fn apply_level_bonuses(
    query: Query<(&Experience, &crate::components::CombatStats)>,
) {
    for (experience, _stats) in query.iter() {
        // Apply level-based stat increases
        let _level_bonus = 1.0 + (experience.level as f32 - 1.0) * 0.05;
        
        // TODO: Apply level bonuses with base stats tracking
        // For now, we'll just recalculate based on level
    }
}
