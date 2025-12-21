//! Generic behavior tree implementation for AI entities
//!
//! This module provides a composable behavior tree framework with standard
//! composite nodes (Sequence, Selector, Parallel), decorator nodes, and leaf nodes.

use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Behavior Tree component
#[derive(Component, Clone, Debug)]
pub struct BehaviorTree {
    pub root: Box<BehaviorNode>,
    pub blackboard: Blackboard,
    pub tick_rate: f32,
    pub last_tick: f32,
}

/// Blackboard for sharing data between nodes
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Blackboard {
    pub values: HashMap<String, BlackboardValue>,
}

/// Values that can be stored in the blackboard
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum BlackboardValue {
    Bool(bool),
    Float(f32),
    Int(i32),
    String(String),
    Vec3 { x: f32, y: f32, z: f32 },
    Entity(u64), // Store entity as u64 for serialization
}

impl Blackboard {
    pub fn get_bool(&self, key: &str) -> Option<bool> {
        match self.values.get(key) {
            Some(BlackboardValue::Bool(val)) => Some(*val),
            _ => None,
        }
    }

    pub fn get_float(&self, key: &str) -> Option<f32> {
        match self.values.get(key) {
            Some(BlackboardValue::Float(val)) => Some(*val),
            _ => None,
        }
    }

    pub fn get_int(&self, key: &str) -> Option<i32> {
        match self.values.get(key) {
            Some(BlackboardValue::Int(val)) => Some(*val),
            _ => None,
        }
    }

    pub fn get_string(&self, key: &str) -> Option<String> {
        match self.values.get(key) {
            Some(BlackboardValue::String(val)) => Some(val.clone()),
            _ => None,
        }
    }

    pub fn get_vec3(&self, key: &str) -> Option<Vec3> {
        match self.values.get(key) {
            Some(BlackboardValue::Vec3 { x, y, z }) => Some(Vec3::new(*x, *y, *z)),
            _ => None,
        }
    }

    pub fn get_entity(&self, key: &str) -> Option<Entity> {
        match self.values.get(key) {
            Some(BlackboardValue::Entity(bits)) => Some(Entity::from_bits(*bits)),
            _ => None,
        }
    }

    pub fn set_bool(&mut self, key: String, value: bool) {
        self.values.insert(key, BlackboardValue::Bool(value));
    }

    pub fn set_float(&mut self, key: String, value: f32) {
        self.values.insert(key, BlackboardValue::Float(value));
    }

    pub fn set_int(&mut self, key: String, value: i32) {
        self.values.insert(key, BlackboardValue::Int(value));
    }

    pub fn set_string(&mut self, key: String, value: String) {
        self.values.insert(key, BlackboardValue::String(value));
    }

    pub fn set_vec3(&mut self, key: String, value: Vec3) {
        self.values.insert(
            key,
            BlackboardValue::Vec3 {
                x: value.x,
                y: value.y,
                z: value.z,
            },
        );
    }

    pub fn set_entity(&mut self, key: String, value: Entity) {
        self.values
            .insert(key, BlackboardValue::Entity(value.to_bits()));
    }
}

/// Behavior node types
#[derive(Clone, Debug)]
pub enum BehaviorNode {
    // Composite nodes
    Sequence(Vec<Box<BehaviorNode>>),
    Selector(Vec<Box<BehaviorNode>>),
    Parallel(Vec<Box<BehaviorNode>>, usize), // min success count

    // Decorator nodes
    Inverter(Box<BehaviorNode>),
    Repeater(Box<BehaviorNode>, u32),
    Succeeder(Box<BehaviorNode>),
    Failer(Box<BehaviorNode>),

    // Leaf nodes
    Action(ActionNode),
    Condition(ConditionNode),
}

/// Action node for executing behaviors
#[derive(Clone, Debug)]
pub struct ActionNode {
    pub action_type: ActionType,
    pub name: String,
}

/// Generic action types
#[derive(Clone, Debug)]
pub enum ActionType {
    MoveTo,
    Attack,
    Build,
    Gather,
    Patrol,
    Wait,
    Custom(String),
}

/// Condition node for checking state
#[derive(Clone, Debug)]
pub struct ConditionNode {
    pub condition_type: ConditionType,
    pub name: String,
}

/// Generic condition types
#[derive(Clone, Debug)]
pub enum ConditionType {
    HasTarget,
    HasResources,
    IsHealthy,
    IsUnderAttack,
    CanBuild,
    Custom(String),
}

/// Node execution result
#[derive(Clone, Debug, PartialEq)]
pub enum NodeStatus {
    Success,
    Failure,
    Running,
}

impl BehaviorNode {
    /// Execute the behavior node
    pub fn tick(
        &mut self,
        blackboard: &mut Blackboard,
        _entity: Entity,
        _world: &World,
    ) -> NodeStatus {
        match self {
            BehaviorNode::Sequence(children) => {
                for child in children.iter_mut() {
                    match child.tick(blackboard, _entity, _world) {
                        NodeStatus::Failure => return NodeStatus::Failure,
                        NodeStatus::Running => return NodeStatus::Running,
                        NodeStatus::Success => continue,
                    }
                }
                NodeStatus::Success
            }

            BehaviorNode::Selector(children) => {
                for child in children.iter_mut() {
                    match child.tick(blackboard, _entity, _world) {
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
                    match child.tick(blackboard, _entity, _world) {
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

            BehaviorNode::Inverter(child) => match child.tick(blackboard, _entity, _world) {
                NodeStatus::Success => NodeStatus::Failure,
                NodeStatus::Failure => NodeStatus::Success,
                NodeStatus::Running => NodeStatus::Running,
            },

            BehaviorNode::Repeater(child, times) => {
                for _ in 0..*times {
                    if child.tick(blackboard, _entity, _world) == NodeStatus::Failure {
                        return NodeStatus::Failure;
                    }
                }
                NodeStatus::Success
            }

            BehaviorNode::Succeeder(child) => {
                child.tick(blackboard, _entity, _world);
                NodeStatus::Success
            }

            BehaviorNode::Failer(child) => {
                child.tick(blackboard, _entity, _world);
                NodeStatus::Failure
            }

            BehaviorNode::Action(action) => execute_action(action, blackboard),

            BehaviorNode::Condition(condition) => {
                if check_condition(condition, blackboard) {
                    NodeStatus::Success
                } else {
                    NodeStatus::Failure
                }
            }
        }
    }
}

/// Execute action nodes (override this in your game for custom behavior)
fn execute_action(action: &ActionNode, blackboard: &mut Blackboard) -> NodeStatus {
    match &action.action_type {
        ActionType::MoveTo => {
            if blackboard.get_vec3("move_target").is_some() {
                NodeStatus::Running
            } else {
                NodeStatus::Failure
            }
        }
        ActionType::Attack => {
            if blackboard.get_entity("attack_target").is_some() {
                NodeStatus::Running
            } else {
                NodeStatus::Failure
            }
        }
        ActionType::Build => NodeStatus::Running,
        ActionType::Gather => NodeStatus::Running,
        ActionType::Patrol => NodeStatus::Running,
        ActionType::Wait => NodeStatus::Success,
        ActionType::Custom(_) => NodeStatus::Success,
    }
}

/// Check condition nodes (override this in your game for custom conditions)
fn check_condition(condition: &ConditionNode, blackboard: &Blackboard) -> bool {
    match &condition.condition_type {
        ConditionType::HasTarget => blackboard.get_entity("attack_target").is_some(),
        ConditionType::HasResources => blackboard.get_float("resources").unwrap_or(0.0) > 100.0,
        ConditionType::IsHealthy => blackboard.get_float("health").unwrap_or(0.0) > 50.0,
        ConditionType::IsUnderAttack => blackboard.get_bool("under_attack").unwrap_or(false),
        ConditionType::CanBuild => blackboard.get_bool("can_build").unwrap_or(false),
        ConditionType::Custom(_) => true,
    }
}

/// Behavior tree builder for convenient tree construction
#[allow(clippy::vec_box)]
pub struct BehaviorTreeBuilder {
    nodes: Vec<Box<BehaviorNode>>,
}

impl Default for BehaviorTreeBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl BehaviorTreeBuilder {
    pub fn new() -> Self {
        Self { nodes: Vec::new() }
    }

    pub fn sequence(mut self) -> Self {
        let sequence = BehaviorNode::Sequence(self.nodes);
        self.nodes = vec![Box::new(sequence)];
        self
    }

    pub fn selector(mut self) -> Self {
        let selector = BehaviorNode::Selector(self.nodes);
        self.nodes = vec![Box::new(selector)];
        self
    }

    pub fn action(mut self, action_type: ActionType, name: &str) -> Self {
        let action = BehaviorNode::Action(ActionNode {
            action_type,
            name: name.to_string(),
        });
        self.nodes.push(Box::new(action));
        self
    }

    pub fn condition(mut self, condition_type: ConditionType, name: &str) -> Self {
        let condition = BehaviorNode::Condition(ConditionNode {
            condition_type,
            name: name.to_string(),
        });
        self.nodes.push(Box::new(condition));
        self
    }

    pub fn build(self) -> BehaviorTree {
        BehaviorTree {
            root: self.nodes.into_iter().next().unwrap_or_else(|| {
                Box::new(BehaviorNode::Succeeder(Box::new(BehaviorNode::Action(
                    ActionNode {
                        action_type: ActionType::Wait,
                        name: "default".to_string(),
                    },
                ))))
            }),
            blackboard: Blackboard::default(),
            tick_rate: 1.0,
            last_tick: 0.0,
        }
    }
}

/// Behavior tree execution system
pub fn behavior_tree_system(
    time: Res<Time>,
    world: &World,
    mut query: Query<(Entity, &mut BehaviorTree)>,
) {
    let current_time = time.elapsed_secs();

    // Collect entities that need ticking
    let mut trees_to_tick = Vec::new();

    for (entity, tree) in query.iter() {
        if current_time - tree.last_tick >= tree.tick_rate {
            trees_to_tick.push(entity);
        }
    }

    // Process each behavior tree
    for entity in trees_to_tick {
        if let Ok((_, mut tree)) = query.get_mut(entity) {
            tree.last_tick = current_time;

            // Create a local copy to avoid borrowing conflicts
            let mut blackboard_copy = tree.blackboard.clone();
            tree.root.tick(&mut blackboard_copy, entity, world);

            // Update the blackboard
            tree.blackboard = blackboard_copy;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_blackboard() {
        let mut blackboard = Blackboard::default();
        blackboard.set_bool("test".to_string(), true);
        assert_eq!(blackboard.get_bool("test"), Some(true));

        blackboard.set_float("value".to_string(), 42.0);
        assert_eq!(blackboard.get_float("value"), Some(42.0));
    }

    #[test]
    fn test_behavior_tree_builder() {
        let tree = BehaviorTreeBuilder::new()
            .condition(ConditionType::HasTarget, "check_target")
            .action(ActionType::Attack, "attack")
            .sequence()
            .build();

        assert_eq!(tree.tick_rate, 1.0);
    }
}
