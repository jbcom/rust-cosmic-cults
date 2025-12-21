//! # Bevy AI Toolkit
//!
//! A collection of generic AI systems for the Bevy game engine.
//!
//! ## Features
//!
//! - **State Machines**: Flexible state machine system with customizable transitions
//! - **Behavior Trees**: Composable behavior tree framework
//! - **Utility AI**: Decision-making based on utility scoring
//! - **Targeting**: Advanced target selection and prioritization

#![allow(unused)]

use bevy::prelude::*;

// Core modules
pub mod behavior_tree;
pub mod state_machine;
pub mod targeting;
pub mod utility_ai;

// Re-exports for convenience
pub use behavior_tree::{
    ActionNode, ActionType, BehaviorNode, BehaviorTree, BehaviorTreeBuilder, Blackboard,
    BlackboardValue, ConditionNode, ConditionType, NodeStatus,
};
pub use state_machine::{AIState, AIStateMachine, AITransition, HierarchicalStateMachine};
pub use targeting::{
    TargetCandidate, TargetPriority, TargetSelector, get_enemies_in_range, get_nearest_enemy,
    get_target_priority_for_role, get_weakest_enemy,
};
pub use utility_ai::{
    Consideration, InputType, ResponseCurve, UtilityAI, UtilityAction, UtilityActionType,
    UtilityContext, UtilityScore, create_aggressive_ai, create_economic_ai,
};

/// Plugin that adds all AI toolkit systems to a Bevy app
pub struct BevyAIToolkitPlugin;

impl Plugin for BevyAIToolkitPlugin {
    fn build(&self, app: &mut App) {
        app
            // Add AI systems
            .add_systems(
                Update,
                (
                    state_machine::state_machine_update_system,
                    behavior_tree::behavior_tree_system,
                    utility_ai::utility_ai_system,
                    targeting::target_acquisition_system,
                    targeting::target_validation_system,
                ),
            );
    }
}

/// Prelude module for convenient imports
pub mod prelude {
    pub use crate::BevyAIToolkitPlugin;
    pub use crate::behavior_tree::*;
    pub use crate::state_machine::*;
    pub use crate::targeting::*;
    pub use crate::utility_ai::*;
}
