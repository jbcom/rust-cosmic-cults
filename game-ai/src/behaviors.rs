// Behavior Tree Implementation - Production-ready behavior tree for complex AI logic
use bevy::prelude::*;
use game_physics::prelude::*;
use game_units::{Team, Unit};
use std::collections::HashMap;

// Core behavior tree node types
#[derive(Component, Clone, Debug)]
pub enum BehaviorNode {
    // Composite nodes - control flow
    Sequence(Vec<BehaviorNode>),
    Selector(Vec<BehaviorNode>),
    Parallel(Vec<BehaviorNode>, usize), // min success count

    // Decorator nodes - modify child behavior
    Inverter(Box<BehaviorNode>),
    Repeater(Box<BehaviorNode>, u32),
    UntilFail(Box<BehaviorNode>),
    UntilSuccess(Box<BehaviorNode>),

    // Leaf nodes - actual behavior
    Action(AIAction),
    Condition(AICondition),
}

// AI Actions that units can perform
#[derive(Clone, Debug)]
pub enum AIAction {
    MoveToTarget,
    AttackTarget,
    Patrol,
    GatherResource,
    ReturnToBase,
    DefendPosition,
    SearchArea,
    CallForHelp,
    UseAbility(String),
}

// AI Conditions for decision making
#[derive(Clone, Debug)]
pub enum AICondition {
    HasTarget,
    TargetInRange(f32),
    HealthAbove(f32),
    HealthBelow(f32),
    HasResources(u32),
    AlliesNearby(u32),
    EnemiesNearby(u32),
    AtBase,
    PathClear,
}

// Execution status of behavior nodes
#[derive(Clone, Debug, PartialEq)]
pub enum NodeStatus {
    Success,
    Failure,
    Running,
}

// Behavior tree component attached to AI entities
#[derive(Component, Clone, Debug)]
pub struct BehaviorTree {
    pub root: Box<BehaviorNode>,
    pub blackboard: Blackboard,
    pub current_path: Vec<usize>, // Track current execution path
    pub tick_rate: f32,
    pub last_tick: f32,
}

// Blackboard for sharing data between behavior nodes
#[derive(Clone, Debug, Default)]
pub struct Blackboard {
    pub values: HashMap<String, BlackboardValue>,
}

#[derive(Clone, Debug)]
pub enum BlackboardValue {
    Bool(bool),
    Float(f32),
    Int(i32),
    Entity(Entity),
    Vec3(Vec3),
    String(String),
}

impl Blackboard {
    pub fn set(&mut self, key: String, value: BlackboardValue) {
        self.values.insert(key, value);
    }

    pub fn get_bool(&self, key: &str) -> Option<bool> {
        match self.values.get(key) {
            Some(BlackboardValue::Bool(v)) => Some(*v),
            _ => None,
        }
    }

    pub fn get_float(&self, key: &str) -> Option<f32> {
        match self.values.get(key) {
            Some(BlackboardValue::Float(v)) => Some(*v),
            _ => None,
        }
    }

    pub fn get_entity(&self, key: &str) -> Option<Entity> {
        match self.values.get(key) {
            Some(BlackboardValue::Entity(v)) => Some(*v),
            _ => None,
        }
    }

    pub fn get_vec3(&self, key: &str) -> Option<Vec3> {
        match self.values.get(key) {
            Some(BlackboardValue::Vec3(v)) => Some(*v),
            _ => None,
        }
    }
}

// Execute behavior tree nodes
impl BehaviorNode {
    pub fn execute(
        &mut self,
        entity: Entity,
        blackboard: &mut Blackboard,
        world: &World,
        commands: &mut Commands,
    ) -> NodeStatus {
        match self {
            BehaviorNode::Sequence(children) => {
                for child in children.iter_mut() {
                    match child.execute(entity, blackboard, world, commands) {
                        NodeStatus::Failure => return NodeStatus::Failure,
                        NodeStatus::Running => return NodeStatus::Running,
                        NodeStatus::Success => continue,
                    }
                }
                NodeStatus::Success
            }

            BehaviorNode::Selector(children) => {
                for child in children.iter_mut() {
                    match child.execute(entity, blackboard, world, commands) {
                        NodeStatus::Success => return NodeStatus::Success,
                        NodeStatus::Running => return NodeStatus::Running,
                        NodeStatus::Failure => continue,
                    }
                }
                NodeStatus::Failure
            }

            BehaviorNode::Parallel(children, min_success) => {
                let mut success_count = 0;
                let mut has_running = false;

                for child in children.iter_mut() {
                    match child.execute(entity, blackboard, world, commands) {
                        NodeStatus::Success => success_count += 1,
                        NodeStatus::Running => has_running = true,
                        NodeStatus::Failure => {}
                    }
                }

                if success_count >= *min_success {
                    NodeStatus::Success
                } else if has_running {
                    NodeStatus::Running
                } else {
                    NodeStatus::Failure
                }
            }

            BehaviorNode::Inverter(child) => {
                match child.execute(entity, blackboard, world, commands) {
                    NodeStatus::Success => NodeStatus::Failure,
                    NodeStatus::Failure => NodeStatus::Success,
                    NodeStatus::Running => NodeStatus::Running,
                }
            }

            BehaviorNode::Repeater(child, times) => {
                for _ in 0..*times {
                    match child.execute(entity, blackboard, world, commands) {
                        NodeStatus::Failure => return NodeStatus::Failure,
                        NodeStatus::Running => return NodeStatus::Running,
                        _ => continue,
                    }
                }
                NodeStatus::Success
            }

            BehaviorNode::UntilFail(child) => {
                // Execute child once per tick to prevent infinite loops
                // Returns Running to allow the game loop to continue
                match child.execute(entity, blackboard, world, commands) {
                    NodeStatus::Failure => NodeStatus::Success,
                    NodeStatus::Running => NodeStatus::Running,
                    NodeStatus::Success => NodeStatus::Running, // Continue next tick
                }
            }

            BehaviorNode::UntilSuccess(child) => {
                // Execute child once per tick to prevent infinite loops
                // Returns Running to allow the game loop to continue
                match child.execute(entity, blackboard, world, commands) {
                    NodeStatus::Success => NodeStatus::Success,
                    NodeStatus::Running => NodeStatus::Running,
                    NodeStatus::Failure => NodeStatus::Running, // Continue next tick
                }
            }

            BehaviorNode::Action(action) => {
                execute_action(action, entity, blackboard, world, commands)
            }

            BehaviorNode::Condition(condition) => {
                if check_condition(condition, entity, blackboard, world) {
                    NodeStatus::Success
                } else {
                    NodeStatus::Failure
                }
            }
        }
    }
}

// Execute AI actions
fn execute_action(
    action: &AIAction,
    entity: Entity,
    blackboard: &mut Blackboard,
    world: &World,
    commands: &mut Commands,
) -> NodeStatus {
    match action {
        AIAction::MoveToTarget => {
            if let Some(target_pos) = blackboard.get_vec3("target_position") {
                commands.entity(entity).insert(MovementTarget {
                    x: target_pos.x,
                    y: target_pos.y,
                    z: target_pos.z,
                    reached: false,
                    speed: 5.0,
                });
                NodeStatus::Running
            } else {
                NodeStatus::Failure
            }
        }

        AIAction::AttackTarget => {
            if let Some(target) = blackboard.get_entity("attack_target") {
                commands
                    .entity(entity)
                    .insert(crate::systems::state_machine::AttackBehavior {
                        target: Some(target),
                        aggression_level: 1.0,
                    });
                NodeStatus::Running
            } else {
                NodeStatus::Failure
            }
        }

        AIAction::Patrol => {
            // Get patrol points from blackboard
            if let Some(patrol_index) = blackboard.get_float("patrol_index") {
                let next_index = ((patrol_index as usize + 1) % 4) as f32;
                blackboard.set(
                    "patrol_index".to_string(),
                    BlackboardValue::Float(next_index),
                );

                // Generate patrol point based on index
                let base_pos = blackboard.get_vec3("base_position").unwrap_or(Vec3::ZERO);
                let offset = match next_index as usize {
                    0 => Vec3::new(10.0, 0.0, 0.0),
                    1 => Vec3::new(0.0, 0.0, 10.0),
                    2 => Vec3::new(-10.0, 0.0, 0.0),
                    _ => Vec3::new(0.0, 0.0, -10.0),
                };

                let patrol_point = base_pos + offset;
                blackboard.set(
                    "target_position".to_string(),
                    BlackboardValue::Vec3(patrol_point),
                );

                NodeStatus::Success
            } else {
                blackboard.set("patrol_index".to_string(), BlackboardValue::Float(0.0));
                NodeStatus::Success
            }
        }

        AIAction::GatherResource => {
            if let Some(resource) = blackboard.get_entity("resource_target") {
                commands
                    .entity(entity)
                    .insert(crate::systems::state_machine::GatheringBehavior {
                        target_resource: Some(resource),
                        gathering_rate: 1.0,
                    });
                NodeStatus::Running
            } else {
                NodeStatus::Failure
            }
        }

        AIAction::ReturnToBase => {
            if let Some(base_pos) = blackboard.get_vec3("base_position") {
                commands.entity(entity).insert(MovementTarget {
                    x: base_pos.x,
                    y: base_pos.y,
                    z: base_pos.z,
                    reached: false,
                    speed: 4.0,
                });
                NodeStatus::Running
            } else {
                NodeStatus::Failure
            }
        }

        AIAction::DefendPosition => {
            if let Some(defend_pos) = blackboard.get_vec3("defend_position") {
                commands
                    .entity(entity)
                    .insert(crate::systems::state_machine::DefendBehavior {
                        defend_position: defend_pos,
                        patrol_radius: 10.0,
                    });
                NodeStatus::Running
            } else {
                NodeStatus::Failure
            }
        }

        AIAction::SearchArea => {
            // Implement search pattern
            blackboard.set("searching".to_string(), BlackboardValue::Bool(true));
            NodeStatus::Running
        }

        AIAction::CallForHelp => {
            // Send help request to nearby allies
            blackboard.set("help_requested".to_string(), BlackboardValue::Bool(true));
            NodeStatus::Success
        }

        AIAction::UseAbility(ability_name) => {
            // Trigger ability usage
            blackboard.set(
                "ability_used".to_string(),
                BlackboardValue::String(ability_name.clone()),
            );
            NodeStatus::Success
        }
    }
}

// Check AI conditions
fn check_condition(
    condition: &AICondition,
    entity: Entity,
    blackboard: &Blackboard,
    world: &World,
) -> bool {
    match condition {
        AICondition::HasTarget => {
            blackboard.get_entity("attack_target").is_some()
                || blackboard.get_vec3("target_position").is_some()
        }

        AICondition::TargetInRange(range) => {
            if let Some(target_pos) = blackboard.get_vec3("target_position") {
                if let Ok(entity_ref) = world.get_entity(entity) {
                    if let Some(transform) = entity_ref.get::<Transform>() {
                        return transform.translation.distance(target_pos) <= *range;
                    }
                }
            }
            false
        }

        AICondition::HealthAbove(threshold) => {
            if let Ok(entity_ref) = world.get_entity(entity) {
                if let Some(unit) = entity_ref.get::<Unit>() {
                    return unit.health / unit.max_health > *threshold;
                }
            }
            true
        }

        AICondition::HealthBelow(threshold) => {
            if let Ok(entity_ref) = world.get_entity(entity) {
                if let Some(unit) = entity_ref.get::<Unit>() {
                    return unit.health / unit.max_health < *threshold;
                }
            }
            false
        }

        AICondition::HasResources(amount) => {
            blackboard.get_float("resources").unwrap_or(0.0) >= *amount as f32
        }

        AICondition::AlliesNearby(count) => {
            blackboard.get_float("nearby_allies").unwrap_or(0.0) >= *count as f32
        }

        AICondition::EnemiesNearby(count) => {
            blackboard.get_float("nearby_enemies").unwrap_or(0.0) >= *count as f32
        }

        AICondition::AtBase => blackboard.get_bool("at_base").unwrap_or(false),

        AICondition::PathClear => !blackboard.get_bool("path_blocked").unwrap_or(false),
    }
}

// Behavior tree factory functions for common AI patterns
pub fn create_aggressive_behavior() -> BehaviorTree {
    let root = Box::new(BehaviorNode::Selector(vec![
        // Priority 1: Attack if enemy in range
        BehaviorNode::Sequence(vec![
            BehaviorNode::Condition(AICondition::HasTarget),
            BehaviorNode::Condition(AICondition::TargetInRange(15.0)),
            BehaviorNode::Action(AIAction::AttackTarget),
        ]),
        // Priority 2: Search for enemies
        BehaviorNode::Sequence(vec![
            BehaviorNode::Action(AIAction::SearchArea),
            BehaviorNode::Action(AIAction::MoveToTarget),
        ]),
        // Priority 3: Patrol
        BehaviorNode::Sequence(vec![
            BehaviorNode::Action(AIAction::Patrol),
            BehaviorNode::Action(AIAction::MoveToTarget),
        ]),
    ]));

    BehaviorTree {
        root,
        blackboard: Blackboard::default(),
        current_path: Vec::new(),
        tick_rate: 0.5,
        last_tick: 0.0,
    }
}

pub fn create_defensive_behavior() -> BehaviorTree {
    let root = Box::new(BehaviorNode::Selector(vec![
        // Priority 1: Flee if health low
        BehaviorNode::Sequence(vec![
            BehaviorNode::Condition(AICondition::HealthBelow(0.3)),
            BehaviorNode::Action(AIAction::ReturnToBase),
        ]),
        // Priority 2: Defend position if enemies nearby
        BehaviorNode::Sequence(vec![
            BehaviorNode::Condition(AICondition::EnemiesNearby(1)),
            BehaviorNode::Action(AIAction::DefendPosition),
        ]),
        // Priority 3: Patrol around base
        BehaviorNode::Sequence(vec![
            BehaviorNode::Action(AIAction::Patrol),
            BehaviorNode::Action(AIAction::MoveToTarget),
        ]),
    ]));

    BehaviorTree {
        root,
        blackboard: Blackboard::default(),
        current_path: Vec::new(),
        tick_rate: 0.5,
        last_tick: 0.0,
    }
}

pub fn create_gatherer_behavior() -> BehaviorTree {
    let root = Box::new(BehaviorNode::Selector(vec![
        // Priority 1: Flee if under attack
        BehaviorNode::Sequence(vec![
            BehaviorNode::Condition(AICondition::HealthBelow(0.8)),
            BehaviorNode::Action(AIAction::CallForHelp),
            BehaviorNode::Action(AIAction::ReturnToBase),
        ]),
        // Priority 2: Gather resources
        BehaviorNode::Sequence(vec![
            BehaviorNode::Condition(AICondition::HasTarget),
            BehaviorNode::Action(AIAction::GatherResource),
        ]),
        // Priority 3: Return resources to base
        BehaviorNode::Sequence(vec![
            BehaviorNode::Condition(AICondition::HasResources(10)),
            BehaviorNode::Action(AIAction::ReturnToBase),
        ]),
        // Priority 4: Search for resources
        BehaviorNode::Action(AIAction::SearchArea),
    ]));

    BehaviorTree {
        root,
        blackboard: Blackboard::default(),
        current_path: Vec::new(),
        tick_rate: 1.0,
        last_tick: 0.0,
    }
}

// Behavior tree execution system
pub fn behavior_tree_execution_system(
    mut query: Query<(Entity, &mut BehaviorTree, &Transform)>,
    time: Res<Time>,
    world: &World,
    mut commands: Commands,
) {
    let current_time = time.elapsed_secs();

    for (entity, mut tree, transform) in query.iter_mut() {
        // Check if it's time to tick this behavior tree
        if current_time - tree.last_tick < tree.tick_rate {
            continue;
        }

        tree.last_tick = current_time;

        // Update blackboard with current entity state
        tree.blackboard.set(
            "current_position".to_string(),
            BlackboardValue::Vec3(transform.translation),
        );

        // Execute the behavior tree
        let mut root = tree.root.clone();
        let status = root.execute(entity, &mut tree.blackboard, world, &mut commands);
        tree.root = root;

        // Handle completion status
        match status {
            NodeStatus::Success | NodeStatus::Failure => {
                // Reset for next tick
                tree.current_path.clear();
            }
            NodeStatus::Running => {
                // Continue execution next tick
            }
        }
    }
}
