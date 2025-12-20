// AI Types - Core AI type definitions and traits
use bevy::prelude::*;

// State machine trait for AI systems
pub trait StateMachine: Component + Send + Sync {
    type State: Clone + std::fmt::Debug + Send + Sync;
    type Trigger: Clone + std::fmt::Debug + Send + Sync;

    fn current_state(&self) -> &Self::State;
    fn transition(&mut self, trigger: Self::Trigger) -> Option<Self::State>;
    fn valid_transitions(&self) -> Vec<(Self::State, Self::Trigger, Self::State)>;
    fn on_enter(&mut self, state: &Self::State);
    fn on_exit(&mut self, state: &Self::State);
}

// Utility AI Intent that feeds into state machines
#[derive(Component, Clone, Debug)]
pub struct AIIntent {
    pub primary_goal: AIGoal,
    pub urgency: f32,
    pub target: Option<Entity>,
    pub position: Option<Vec3>,
}

#[derive(Clone, Debug)]
pub enum AIGoal {
    Idle,
    Patrol,
    Pursue,
    Flee,
    Capture,
    Defend,
    Gather,
    Build,
    Ritual,
}

// Core AI traits for game integration
pub trait AIPerception: Send + Sync {
    fn perceive_enemies(&self, entity: Entity, world: &World) -> Vec<Entity>;
    fn perceive_resources(&self, entity: Entity, world: &World) -> Vec<Entity>;
    fn perceive_allies(&self, entity: Entity, world: &World) -> Vec<Entity>;
    fn get_threat_level(&self, entity: Entity, world: &World) -> f32;
}

pub trait AIExecution: Send + Sync {
    fn execute_move(&self, entity: Entity, target: Vec3, commands: &mut Commands);
    fn execute_attack(&self, entity: Entity, target: Entity, commands: &mut Commands);
    fn execute_build(
        &self,
        entity: Entity,
        building_type: &str,
        position: Vec3,
        commands: &mut Commands,
    );
    fn execute_gather(&self, entity: Entity, resource: Entity, commands: &mut Commands);
}

// AI communication and coordination
#[derive(Component, Clone, Debug)]
pub struct AICoordination {
    pub team_id: u32,
    pub role: AIRole,
    pub coordination_radius: f32,
    pub can_give_orders: bool,
    pub can_receive_orders: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub enum AIRole {
    Leader,
    Follower,
    Scout,
    Worker,
    Defender,
    Specialist(String),
}

// AI communication events
#[derive(Event, Clone, Debug)]
pub struct AIMessage {
    pub sender: Entity,
    pub recipients: Vec<Entity>,
    pub message_type: AIMessageType,
    pub urgency: f32,
    pub position: Option<Vec3>,
}

#[derive(Clone, Debug)]
pub enum AIMessageType {
    EnemySpotted(Entity),
    ResourceFound(Entity),
    HelpRequested,
    OrderGiven(AIOrder),
    StatusUpdate,
    Custom(String),
}

#[derive(Clone, Debug)]
pub enum AIOrder {
    MoveTo(Vec3),
    Attack(Entity),
    Defend(Vec3),
    Gather(Entity),
    Build(String, Vec3),
    Follow(Entity),
    Stop,
}
impl bevy::prelude::Message for AIMessage {}
