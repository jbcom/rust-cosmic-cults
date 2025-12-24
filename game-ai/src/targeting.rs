// Target Selection and Prioritization System - Smart target selection for AI entities
use bevy::prelude::*;
use game_physics::prelude::*;
use game_units::{Leader, Team, Unit};
use std::cmp::Ordering;

// Target selector component for AI entities
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

// Target priority strategies
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

// Target evaluation data
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
                // Resources would be handled separately
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

// Target acquisition system
pub fn target_acquisition_system(
    mut query: Query<(Entity, &mut TargetSelector, &Transform, &Team)>,
    enemy_query: Query<(Entity, &Transform, &Team, &Unit, Option<&Leader>)>,
    resource_query: Query<(Entity, &Transform), With<ResourceMarker>>,
    time: Res<Time>,
) {
    let current_time = time.elapsed_seconds();

    for (entity, mut selector, transform, team) in query.iter_mut() {
        // Check if it's time to reacquire target
        if current_time - selector.last_target_check < selector.reacquisition_time {
            // Check if current target still exists and is valid
            if let Some(target) = selector.current_target
                && let Ok((_, target_transform, _, target_unit, _)) = enemy_query.get(target)
            {
                let distance = transform.translation.distance(target_transform.translation);
                if distance <= selector.max_range && target_unit.health > 0.0 {
                    continue; // Keep current target
                }
            }
        }

        // Build list of target candidates
        let mut candidates = Vec::new();

        if selector.priority == TargetPriority::Resource {
            // Look for resources
            for (resource_entity, resource_transform) in resource_query.iter() {
                let distance = transform
                    .translation
                    .distance(resource_transform.translation);

                if distance <= selector.max_range {
                    candidates.push(TargetCandidate {
                        entity: resource_entity,
                        position: resource_transform.translation,
                        distance,
                        health: 100.0,
                        max_health: 100.0,
                        damage: 0.0,
                        is_leader: false,
                        threat_level: 0.0,
                        priority_score: 1.0 / distance,
                    });
                }
            }
        } else {
            // Look for enemies
            for (enemy_entity, enemy_transform, enemy_team, enemy_unit, leader) in
                enemy_query.iter()
            {
                // Skip same team
                if enemy_team.id == team.id {
                    continue;
                }

                // Skip dead units
                if enemy_unit.health <= 0.0 {
                    continue;
                }

                let distance = transform.translation.distance(enemy_transform.translation);

                if distance <= selector.max_range {
                    let threat_level = calculate_threat_level(enemy_unit, distance);

                    candidates.push(TargetCandidate {
                        entity: enemy_entity,
                        position: enemy_transform.translation,
                        distance,
                        health: enemy_unit.health,
                        max_health: enemy_unit.max_health,
                        damage: enemy_unit.attack_damage,
                        is_leader: leader.is_some(),
                        threat_level,
                        priority_score: 0.0, // Will be calculated
                    });
                }
            }
        }

        // Evaluate and select target
        selector.evaluate_targets(candidates, current_time);
    }
}

// Calculate threat level of a unit
fn calculate_threat_level(unit: &Unit, distance: f32) -> f32 {
    let damage_threat = unit.attack_damage / 10.0;
    let health_threat = unit.health / unit.max_health;
    let distance_threat = 1.0 / (distance + 1.0);

    (damage_threat + health_threat + distance_threat) / 3.0
}

// Target validation system - removes invalid targets
pub fn target_validation_system(
    mut query: Query<&mut TargetSelector>,
    entity_query: Query<Entity, With<Unit>>,
) {
    let valid_entities: Vec<Entity> = entity_query.iter().collect();

    for mut selector in query.iter_mut() {
        if let Some(target) = selector.current_target
            && !valid_entities.contains(&target)
        {
            selector.clear_target();
        }
    }
}

// Line of sight system - checks if target is visible
pub fn line_of_sight_system(
    mut query: Query<(&mut TargetSelector, &Transform)>,
    target_query: Query<&Transform, Without<TargetSelector>>,
    obstacle_query: Query<(&Transform, &CollisionMask), Without<Unit>>,
) {
    for (mut selector, transform) in query.iter_mut() {
        if let Some(target) = selector.current_target
            && let Ok(target_transform) = target_query.get(target)
        {
            // Check if line of sight is blocked
            if is_line_of_sight_blocked(
                transform.translation,
                target_transform.translation,
                &obstacle_query,
            ) {
                // Can't see target, clear it
                selector.clear_target();
            }
        }
    }
}

fn is_line_of_sight_blocked(
    from: Vec3,
    to: Vec3,
    obstacles: &Query<(&Transform, &CollisionMask), Without<Unit>>,
) -> bool {
    let direction = (to - from).normalize();
    let distance = from.distance(to);

    // Simple raycast check
    for (obstacle_transform, _) in obstacles.iter() {
        let obstacle_pos = obstacle_transform.translation;

        // Check if obstacle is near the line
        let to_obstacle = obstacle_pos - from;
        let projection = to_obstacle.dot(direction);

        if projection > 0.0 && projection < distance {
            let closest_point = from + direction * projection;
            let distance_to_line = closest_point.distance(obstacle_pos);

            if distance_to_line < 2.0 {
                return true; // Line of sight blocked
            }
        }
    }

    false
}

// Target prediction system - predicts where moving targets will be
pub fn target_prediction_system(
    mut query: Query<(&mut TargetSelector, &Transform)>,
    target_query: Query<(&Transform, &Velocity), Without<TargetSelector>>,
) {
    for (mut selector, transform) in query.iter_mut() {
        if let Some(target) = selector.current_target
            && let Ok((target_transform, target_velocity)) = target_query.get(target)
        {
            // Predict where target will be
            let time_to_intercept =
                transform.translation.distance(target_transform.translation) / 10.0;
            let predicted_position =
                target_transform.translation + target_velocity.linear * time_to_intercept;

            selector.target_position = Some(predicted_position);
        }
    }
}

// Resource marker component
#[derive(Component)]
pub struct ResourceMarker;

// Helper functions for target selection
pub fn get_nearest_enemy(
    position: Vec3,
    team_id: u32,
    enemies: &Query<(Entity, &Transform, &Team), With<Unit>>,
) -> Option<Entity> {
    let mut nearest = None;
    let mut min_distance = f32::MAX;

    for (entity, transform, team) in enemies.iter() {
        if team.id == team_id {
            continue;
        }

        let distance = position.distance(transform.translation);
        if distance < min_distance {
            min_distance = distance;
            nearest = Some(entity);
        }
    }

    nearest
}

pub fn get_weakest_enemy(
    position: Vec3,
    team_id: u32,
    max_range: f32,
    enemies: &Query<(Entity, &Transform, &Team, &Unit)>,
) -> Option<Entity> {
    let mut weakest = None;
    let mut min_health = f32::MAX;

    for (entity, transform, team, unit) in enemies.iter() {
        if team.id == team_id {
            continue;
        }

        let distance = position.distance(transform.translation);
        if distance <= max_range && unit.health < min_health {
            min_health = unit.health;
            weakest = Some(entity);
        }
    }

    weakest
}

pub fn get_enemies_in_range(
    position: Vec3,
    team_id: u32,
    range: f32,
    enemies: &Query<(Entity, &Transform, &Team), With<Unit>>,
) -> Vec<Entity> {
    let mut in_range = Vec::new();

    for (entity, transform, team) in enemies.iter() {
        if team.id == team_id {
            continue;
        }

        let distance = position.distance(transform.translation);
        if distance <= range {
            in_range.push(entity);
        }
    }

    in_range
}

// Target priority presets for different unit types
pub fn get_target_priority_for_role(role: &str) -> TargetPriority {
    match role {
        "tank" => TargetPriority::Closest,
        "assassin" => TargetPriority::Weakest,
        "support" => TargetPriority::Leader,
        "gatherer" => TargetPriority::Resource,
        _ => TargetPriority::Balanced,
    }
}
