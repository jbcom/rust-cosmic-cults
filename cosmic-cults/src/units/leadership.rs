use bevy::prelude::*;

// Leadership building component for platform mechanics
#[derive(Component, Clone, Debug, Default)]
pub struct LeadershipBuilding {
    pub leader_entity: Option<Entity>,
    pub platform_type: String,
    pub bonuses_active: bool,
    pub destruction_triggers_retreat: bool,
}
