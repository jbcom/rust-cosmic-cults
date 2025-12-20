use bevy::prelude::*;
use game_physics::prelude::*;

// Behavior Tree component
#[derive(Component, Clone, Debug)]
pub struct BehaviorTree {
    pub root: Box<BehaviorNode>,
    pub blackboard: Blackboard,
    pub tick_rate: f32,
    pub last_tick: f32,
}

// Blackboard for sharing data between nodes
#[derive(Clone, Debug, Default)]
pub struct Blackboard {
    pub values: std::collections::HashMap<String, BlackboardValue>,
}

#[derive(Clone, Debug)]
pub enum BlackboardValue {
    Bool(bool),
    Float(f32),
    Int(i32),
    String(String),
    Entity(Entity),
    Vec3(Vec3),
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

    pub fn get_entity(&self, key: &str) -> Option<Entity> {
        match self.values.get(key) {
            Some(BlackboardValue::Entity(val)) => Some(*val),
            _ => None,
        }
    }

    pub fn get_vec3(&self, key: &str) -> Option<Vec3> {
        match self.values.get(key) {
            Some(BlackboardValue::Vec3(val)) => Some(*val),
            _ => None,
        }
    }

    pub fn set_bool(&mut self, key: String, value: bool) {
        self.values.insert(key, BlackboardValue::Bool(value));
    }

    pub fn set_float(&mut self, key: String, value: f32) {
        self.values.insert(key, BlackboardValue::Float(value));
    }

    pub fn set_entity(&mut self, key: String, value: Entity) {
        self.values.insert(key, BlackboardValue::Entity(value));
    }

    pub fn set_vec3(&mut self, key: String, value: Vec3) {
        self.values.insert(key, BlackboardValue::Vec3(value));
    }
}

// Behavior node types
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

#[derive(Clone, Debug)]
pub struct ActionNode {
    pub action_type: ActionType,
    pub name: String,
}

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

#[derive(Clone, Debug)]
pub struct ConditionNode {
    pub condition_type: ConditionType,
    pub name: String,
}

#[derive(Clone, Debug)]
pub enum ConditionType {
    HasTarget,
    HasResources,
    IsHealthy,
    IsUnderAttack,
    CanBuild,
    Custom(String),
}

// Node execution result
#[derive(Clone, Debug, PartialEq)]
pub enum NodeStatus {
    Success,
    Failure,
    Running,
}

impl BehaviorNode {
    pub fn tick(
        &mut self,
        blackboard: &mut Blackboard,
        entity: Entity,
        world: &World,
    ) -> NodeStatus {
        match self {
            BehaviorNode::Sequence(children) => {
                for child in children.iter_mut() {
                    match child.tick(blackboard, entity, world) {
                        NodeStatus::Failure => return NodeStatus::Failure,
                        NodeStatus::Running => return NodeStatus::Running,
                        NodeStatus::Success => continue,
                    }
                }
                NodeStatus::Success
            }

            BehaviorNode::Selector(children) => {
                for child in children.iter_mut() {
                    match child.tick(blackboard, entity, world) {
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
                    match child.tick(blackboard, entity, world) {
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

            BehaviorNode::Inverter(child) => match child.tick(blackboard, entity, world) {
                NodeStatus::Success => NodeStatus::Failure,
                NodeStatus::Failure => NodeStatus::Success,
                NodeStatus::Running => NodeStatus::Running,
            },

            BehaviorNode::Repeater(child, times) => {
                for _ in 0..*times {
                    if child.tick(blackboard, entity, world) == NodeStatus::Failure {
                        return NodeStatus::Failure;
                    }
                }
                NodeStatus::Success
            }

            BehaviorNode::Succeeder(child) => {
                child.tick(blackboard, entity, world);
                NodeStatus::Success
            }

            BehaviorNode::Failer(child) => {
                child.tick(blackboard, entity, world);
                NodeStatus::Failure
            }

            BehaviorNode::Action(action) => execute_action(action, blackboard, entity, world),

            BehaviorNode::Condition(condition) => {
                if check_condition(condition, blackboard, entity, world) {
                    NodeStatus::Success
                } else {
                    NodeStatus::Failure
                }
            }
        }
    }
}

// Execute action nodes
fn execute_action(
    action: &ActionNode,
    blackboard: &mut Blackboard,
    _entity: Entity,
    _world: &World,
) -> NodeStatus {
    match &action.action_type {
        ActionType::MoveTo => {
            if let Some(_target) = blackboard.get_vec3("move_target") {
                // Move towards target
                NodeStatus::Running
            } else {
                NodeStatus::Failure
            }
        }
        ActionType::Attack => {
            if let Some(_target) = blackboard.get_entity("attack_target") {
                // Attack target
                NodeStatus::Running
            } else {
                NodeStatus::Failure
            }
        }
        ActionType::Build => {
            // Execute build action
            NodeStatus::Running
        }
        ActionType::Gather => {
            // Execute gather action
            NodeStatus::Running
        }
        ActionType::Patrol => {
            // Execute patrol action
            NodeStatus::Running
        }
        ActionType::Wait => {
            // Wait for specified duration
            NodeStatus::Success
        }
        ActionType::Custom(_) => {
            // Custom action implementation
            NodeStatus::Success
        }
    }
}

// Check condition nodes
fn check_condition(
    condition: &ConditionNode,
    blackboard: &Blackboard,
    _entity: Entity,
    _world: &World,
) -> bool {
    match &condition.condition_type {
        ConditionType::HasTarget => blackboard.get_entity("target").is_some(),
        ConditionType::HasResources => blackboard.get_float("resources").unwrap_or(0.0) > 100.0,
        ConditionType::IsHealthy => blackboard.get_float("health").unwrap_or(0.0) > 50.0,
        ConditionType::IsUnderAttack => blackboard.get_bool("under_attack").unwrap_or(false),
        ConditionType::CanBuild => blackboard.get_bool("can_build").unwrap_or(false),
        ConditionType::Custom(_) => {
            // Custom condition implementation
            true
        }
    }
}

// Behavior tree builder
pub struct BehaviorTreeBuilder {
    nodes: Vec<Box<BehaviorNode>>,
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

// Behavior tree execution system
pub fn behavior_tree_system(
    time: Res<Time>,
    world: &World,
    mut query: Query<(Entity, &mut BehaviorTree)>,
) {
    let current_time = time.elapsed_secs();

    // Collect entities and their behavior tree data to avoid borrowing conflicts
    let mut trees_to_tick = Vec::new();

    for (entity, tree) in query.iter() {
        if current_time - tree.last_tick >= tree.tick_rate {
            trees_to_tick.push(entity);
        }
    }

    // Process each behavior tree separately to avoid conflicts
    for entity in trees_to_tick {
        if let Ok((_, mut tree)) = query.get_mut(entity) {
            tree.last_tick = current_time;

            // Create a local copy of blackboard to avoid borrowing conflicts
            let mut blackboard_copy = tree.blackboard.clone();
            let result = tree.root.tick(&mut blackboard_copy, entity, world);

            // Update the blackboard with any changes
            tree.blackboard = blackboard_copy;
        }
    }
}
