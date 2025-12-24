//! Generic target selection and prioritization system
//!
//! This module provides smart target selection for AI entities with configurable
//! priority strategies.

use bevy::prelude::*;
use std::cmp::Ordering;

/// Target selector component for AI entities
#[derive(Component, Clone, Debug)]
pub struct TargetSelector {
    pub priority: TargetPriority,
    pub current_target: Option<Entity>,
    pub target_position: Option<Vec3>,
    pub max_range: f32,
    pub reacquisition_time: f32,
    pub last_target_check: f32,
    pub target_history: Vec<Entity>,
}

/// Target priority strategies
#[derive(Clone, Debug, PartialEq)]
pub enum TargetPriority {
    Closest,       // Target nearest enemy
    Weakest,       // Target lowest health enemy
    Strongest,     // Target highest health enemy
    MostDangerous, // Target highest damage enemy
    Leader,        // Prioritize enemy leaders
    Resource,      // Target resources instead of enemies
    Balanced,      // Balanced target selection
}

/// Target evaluation data
#[derive(Clone, Debug)]
pub struct TargetCandidate {
    pub entity: Entity,
    pub position: Vec3,
    pub distance: f32,
    pub health: f32,
    pub max_health: f32,
    pub damage: f32,
    pub is_leader: bool,
    pub threat_level: f32,
    pub priority_score: f32,
}

impl TargetSelector {
    pub fn new(priority: TargetPriority) -> Self {
        Self {
            priority,
            current_target: None,
            target_position: None,
            max_range: 30.0,
            reacquisition_time: 1.0,
            last_target_check: 0.0,
            target_history: Vec::new(),
        }
    }

    pub fn evaluate_targets(
        &mut self,
        candidates: Vec<TargetCandidate>,
        current_time: f32,
    ) -> Option<Entity> {
        if candidates.is_empty() {
            self.current_target = None;
            return None;
        }

        // Sort candidates based on priority
        let mut sorted_candidates = candidates;
        sorted_candidates.sort_by(|a, b| self.compare_targets(a, b));

        // Select best target
        if let Some(best) = sorted_candidates.first() {
            self.current_target = Some(best.entity);
            self.target_position = Some(best.position);
            self.last_target_check = current_time;

            // Add to history
            self.target_history.push(best.entity);
            if self.target_history.len() > 10 {
                self.target_history.remove(0);
            }

            return Some(best.entity);
        }

        None
    }

    fn compare_targets(&self, a: &TargetCandidate, b: &TargetCandidate) -> Ordering {
        match self.priority {
            TargetPriority::Closest => a
                .distance
                .partial_cmp(&b.distance)
                .unwrap_or(Ordering::Equal),

            TargetPriority::Weakest => a.health.partial_cmp(&b.health).unwrap_or(Ordering::Equal),

            TargetPriority::Strongest => b.health.partial_cmp(&a.health).unwrap_or(Ordering::Equal),

            TargetPriority::MostDangerous => b
                .threat_level
                .partial_cmp(&a.threat_level)
                .unwrap_or(Ordering::Equal),

            TargetPriority::Leader => {
                if a.is_leader && !b.is_leader {
                    Ordering::Less
                } else if !a.is_leader && b.is_leader {
                    Ordering::Greater
                } else {
                    a.distance
                        .partial_cmp(&b.distance)
                        .unwrap_or(Ordering::Equal)
                }
            }

            TargetPriority::Resource => {
                // Resources prioritize closest
                a.distance
                    .partial_cmp(&b.distance)
                    .unwrap_or(Ordering::Equal)
            }

            TargetPriority::Balanced => {
                // Balanced scoring
                let a_score = self.calculate_balanced_score(a);
                let b_score = self.calculate_balanced_score(b);
                b_score.partial_cmp(&a_score).unwrap_or(Ordering::Equal)
            }
        }
    }

    fn calculate_balanced_score(&self, target: &TargetCandidate) -> f32 {
        let distance_score = 1.0 / (target.distance + 1.0);
        let health_score = 1.0 - (target.health / target.max_health);
        let threat_score = target.threat_level;
        let leader_bonus = if target.is_leader { 2.0 } else { 1.0 };

        (distance_score + health_score + threat_score) * leader_bonus
    }

    pub fn should_switch_target(&self, new_candidate: &TargetCandidate) -> bool {
        if self.current_target.is_none() {
            return true;
        }

        // Check if new target is significantly better
        match self.priority {
            TargetPriority::Closest => {
                new_candidate.distance < 5.0 // Switch if very close enemy appears
            }
            TargetPriority::Leader => {
                new_candidate.is_leader && new_candidate.distance < self.max_range * 0.5
            }
            _ => false,
        }
    }

    pub fn clear_target(&mut self) {
        self.current_target = None;
        self.target_position = None;
    }

    pub fn has_target(&self) -> bool {
        self.current_target.is_some()
    }
}

/// Target acquisition system
/// Note: This is a basic stub. Games should implement their own version with proper queries.
pub fn target_acquisition_system(
    mut query: Query<(Entity, &mut TargetSelector, &Transform)>,
    time: Res<Time>,
) {
    let current_time = time.elapsed_seconds();

    for (_entity, mut selector, _transform) in query.iter_mut() {
        // Check if it's time to reacquire target
        if current_time - selector.last_target_check < selector.reacquisition_time {
            continue;
        }

        // Games should build their own list of target candidates here
        let candidates = Vec::new();

        // Evaluate and select target
        selector.evaluate_targets(candidates, current_time);
    }
}

/// Target validation system - removes invalid targets
/// Note: Games should implement their own version with proper entity validation.
pub fn target_validation_system(mut query: Query<&mut TargetSelector>) {
    for mut selector in query.iter_mut() {
        // Games should check if target entity still exists
        if selector.current_target.is_some() {
            // Check validity and clear if invalid
            // selector.clear_target();
        }
    }
}

// Helper functions for target selection

/// Get nearest enemy from a position
/// Note: This is a stub - games must implement with their own entity queries
pub fn get_nearest_enemy(_position: Vec3, _team_id: u32) -> Option<Entity> {
    None
}

/// Get weakest enemy in range
/// Note: This is a stub - games must implement with their own entity queries
pub fn get_weakest_enemy(_position: Vec3, _team_id: u32, _max_range: f32) -> Option<Entity> {
    None
}

/// Get all enemies in range
/// Note: This is a stub - games must implement with their own entity queries
pub fn get_enemies_in_range(_position: Vec3, _team_id: u32, _range: f32) -> Vec<Entity> {
    Vec::new()
}

/// Get target priority for a role
pub fn get_target_priority_for_role(role: &str) -> TargetPriority {
    match role {
        "tank" => TargetPriority::Closest,
        "assassin" => TargetPriority::Weakest,
        "support" => TargetPriority::Leader,
        "gatherer" => TargetPriority::Resource,
        _ => TargetPriority::Balanced,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_target_selector_creation() {
        let selector = TargetSelector::new(TargetPriority::Closest);
        assert_eq!(selector.priority, TargetPriority::Closest);
        assert!(!selector.has_target());
    }

    #[test]
    fn test_target_priority_for_role() {
        assert_eq!(
            get_target_priority_for_role("tank"),
            TargetPriority::Closest
        );
        assert_eq!(
            get_target_priority_for_role("assassin"),
            TargetPriority::Weakest
        );
    }

    #[test]
    fn test_clear_target() {
        let mut selector = TargetSelector::new(TargetPriority::Balanced);
        selector.current_target = Some(Entity::from_bits(1));
        selector.clear_target();
        assert!(!selector.has_target());
    }
}
