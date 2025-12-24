use bevy::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum AIRole {
    Leader,
    Follower,
    Worker,
    Defender,
    Specialist(String),
}

#[derive(Component, Clone, Debug, Serialize, Deserialize)]
pub struct AICoordination {
    pub team_id: u32,
    pub role: AIRole,
    pub coordination_radius: f32,
    pub can_give_orders: bool,
    pub can_receive_orders: bool,
}

#[derive(Event, Clone, Debug)]
pub struct AIMessage {
    pub sender: Entity,
    pub receiver: Option<Entity>,
    pub content: String,
}
