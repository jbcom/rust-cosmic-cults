// Decision Making System - Strategic decision making for AI entities
use bevy::prelude::*;
use crate::units::{Team, Unit};
use std::collections::VecDeque;

// Decision maker component for strategic AI decisions
#[derive(Component, Clone, Debug)]
pub struct DecisionMaker {
    pub personality: AIPersonality,
    pub current_goal: Option<StrategicGoal>,
    pub goal_queue: VecDeque<StrategicGoal>,
    pub decision_history: Vec<Decision>,
    pub evaluation_interval: f32,
    pub last_evaluation: f32,
}

// AI personality affects decision weights
#[derive(Clone, Debug)]
pub struct AIPersonality {
    pub aggression: f32,  // 0.0 = peaceful, 1.0 = very aggressive
    pub caution: f32,     // 0.0 = reckless, 1.0 = very cautious
    pub greed: f32,       // 0.0 = content, 1.0 = resource hungry
    pub loyalty: f32,     // 0.0 = independent, 1.0 = follows orders
    pub exploration: f32, // 0.0 = stays home, 1.0 = explores map
}

impl Default for AIPersonality {
    fn default() -> Self {
        Self {
            aggression: 0.5,
            caution: 0.5,
            greed: 0.5,
            loyalty: 0.5,
            exploration: 0.5,
        }
    }
}

// Strategic goals the AI can pursue
#[derive(Clone, Debug)]
pub enum StrategicGoal {
    EliminateEnemy(Entity),
    CaptureResource(Vec3),
    DefendPosition(Vec3),
    ExploreTerrain(Vec3),
    EscortUnit(Entity),
    BuildBase(Vec3),
    GatherResources,
    PatrolArea(Vec<Vec3>),
}

// Decision context for evaluation
#[derive(Clone, Debug)]
pub struct DecisionContext {
    pub entity: Entity,
    pub position: Vec3,
    pub team_id: u32,
    pub health_percentage: f32,
    pub nearby_enemies: usize,
    pub nearby_allies: usize,
    pub threat_level: f32,
    pub has_resources: bool,
    pub time_elapsed: f32,
}

// Recorded decision for history
#[derive(Clone, Debug)]
pub struct Decision {
    pub goal: StrategicGoal,
    pub score: f32,
    pub timestamp: f32,
    pub success: bool,
}

impl DecisionMaker {
    pub fn new(personality: AIPersonality) -> Self {
        Self {
            personality,
            current_goal: None,
            goal_queue: VecDeque::new(),
            decision_history: Vec::new(),
            evaluation_interval: 2.0,
            last_evaluation: 0.0,
        }
    }

    pub fn aggressive() -> Self {
        Self::new(AIPersonality {
            aggression: 0.9,
            caution: 0.2,
            greed: 0.4,
            loyalty: 0.6,
            exploration: 0.7,
        })
    }

    pub fn defensive() -> Self {
        Self::new(AIPersonality {
            aggression: 0.3,
            caution: 0.8,
            greed: 0.5,
            loyalty: 0.7,
            exploration: 0.3,
        })
    }

    pub fn economic() -> Self {
        Self::new(AIPersonality {
            aggression: 0.2,
            caution: 0.6,
            greed: 0.9,
            loyalty: 0.5,
            exploration: 0.6,
        })
    }

    pub fn balanced() -> Self {
        Self::new(AIPersonality::default())
    }

    pub fn update(&mut self, context: &DecisionContext) {
        // Evaluate available goals and their scores
        let mut goal_scores = Vec::new();

        // Score different goals based on context and personality

        // Combat goals
        if context.nearby_enemies > 0 {
            let attack_score = self.score_attack_goal(context);
            goal_scores.push((
                StrategicGoal::EliminateEnemy(Entity::PLACEHOLDER),
                attack_score,
            ));
        }

        // Defense goals
        if context.threat_level > 0.5 {
            let defend_score = self.score_defend_goal(context);
            goal_scores.push((
                StrategicGoal::DefendPosition(context.position),
                defend_score,
            ));
        }

        // Economic goals
        if context.threat_level < 0.3 {
            let gather_score = self.score_gather_goal(context);
            goal_scores.push((StrategicGoal::GatherResources, gather_score));
        }

        // Exploration goals
        if context.nearby_enemies == 0 && context.threat_level < 0.2 {
            let explore_score = self.score_exploration_goal(context);
            goal_scores.push((
                StrategicGoal::ExploreTerrain(Vec3::new(50.0, 0.0, 50.0)),
                explore_score,
            ));
        }

        // Sort goals by score
        goal_scores.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Less));

        // Select best goal
        if let Some((goal, score)) = goal_scores.first()
            && score > &0.5
        {
            self.current_goal = Some(goal.clone());

            // Record decision
            self.decision_history.push(Decision {
                goal: goal.clone(),
                score: *score,
                timestamp: context.time_elapsed,
                success: false, // Will be updated when goal completes
            });

            // Limit history size
            if self.decision_history.len() > 20 {
                self.decision_history.remove(0);
            }
        }
    }

    fn score_attack_goal(&self, context: &DecisionContext) -> f32 {
        let mut score = self.personality.aggression;

        // Modify based on context
        score *= context.health_percentage; // Less likely to attack when hurt
        score *= 1.0 + (context.nearby_allies as f32 * 0.2); // More confident with allies
        score *= 1.0 / (1.0 + context.nearby_enemies as f32 * 0.3); // Less confident when outnumbered

        // Apply caution
        score *= 1.0 - (self.personality.caution * 0.5);

        score.clamp(0.0, 1.0)
    }

    fn score_defend_goal(&self, context: &DecisionContext) -> f32 {
        let mut score = self.personality.caution;

        // Increase score based on threat
        score *= 1.0 + context.threat_level;

        // Consider health
        score *= 2.0 - context.health_percentage; // More defensive when hurt

        // Consider allies
        if context.nearby_allies > 0 {
            score *= 0.8; // Less need to defend with allies around
        }

        score.clamp(0.0, 1.0)
    }

    fn score_gather_goal(&self, context: &DecisionContext) -> f32 {
        let mut score = self.personality.greed;

        // Safety check
        score *= 1.0 - context.threat_level;

        // Resource need (simplified)
        if !context.has_resources {
            score *= 1.5;
        }

        score.clamp(0.0, 1.0)
    }

    fn score_exploration_goal(&self, context: &DecisionContext) -> f32 {
        let mut score = self.personality.exploration;

        // Safety check
        score *= 1.0 - context.threat_level;
        score *= context.health_percentage;

        // Reduce if enemies nearby
        if context.nearby_enemies > 0 {
            score *= 0.3;
        }

        score.clamp(0.0, 1.0)
    }

    pub fn has_goal(&self) -> bool {
        self.current_goal.is_some()
    }

    pub fn complete_goal(&mut self, success: bool) {
        if let Some(ref goal) = self.current_goal {
            // Update last decision with success status
            if let Some(last) = self.decision_history.last_mut() {
                last.success = success;
            }
        }

        self.current_goal = None;

        // Get next goal from queue if available
        self.current_goal = self.goal_queue.pop_front();
    }

    pub fn add_goal(&mut self, goal: StrategicGoal) {
        self.goal_queue.push_back(goal);
    }

    pub fn clear_goals(&mut self) {
        self.current_goal = None;
        self.goal_queue.clear();
    }
}

// Utility scoring for decisions
pub struct UtilityScorer;

impl UtilityScorer {
    pub fn score_action(
        action: &ActionOption,
        context: &DecisionContext,
        personality: &AIPersonality,
    ) -> f32 {
        match action {
            ActionOption::Attack(target) => Self::score_attack(context, personality),
            ActionOption::Defend(position) => Self::score_defend(context, personality),
            ActionOption::Gather => Self::score_gather(context, personality),
            ActionOption::Explore => Self::score_explore(context, personality),
            ActionOption::Retreat => Self::score_retreat(context, personality),
        }
    }

    fn score_attack(context: &DecisionContext, personality: &AIPersonality) -> f32 {
        let health_factor = context.health_percentage;
        let numbers_factor = (context.nearby_allies as f32) / (context.nearby_enemies as f32 + 1.0);
        let aggression_factor = personality.aggression;

        (health_factor * numbers_factor * aggression_factor).clamp(0.0, 1.0)
    }

    fn score_defend(context: &DecisionContext, personality: &AIPersonality) -> f32 {
        let threat_factor = context.threat_level;
        let caution_factor = personality.caution;
        let position_value = 0.5; // Base value of position

        (threat_factor * caution_factor * position_value).clamp(0.0, 1.0)
    }

    fn score_gather(context: &DecisionContext, personality: &AIPersonality) -> f32 {
        let safety_factor = 1.0 - context.threat_level;
        let greed_factor = personality.greed;
        let need_factor = if context.has_resources { 0.3 } else { 1.0 };

        (safety_factor * greed_factor * need_factor).clamp(0.0, 1.0)
    }

    fn score_explore(context: &DecisionContext, personality: &AIPersonality) -> f32 {
        let safety_factor = 1.0 - context.threat_level;
        let exploration_factor = personality.exploration;
        let health_factor = context.health_percentage;

        (safety_factor * exploration_factor * health_factor).clamp(0.0, 1.0)
    }

    fn score_retreat(context: &DecisionContext, personality: &AIPersonality) -> f32 {
        let danger_factor = context.threat_level;
        let health_factor = 1.0 - context.health_percentage;
        let caution_factor = personality.caution;

        (danger_factor * health_factor * caution_factor).clamp(0.0, 1.0)
    }
}

// Available actions for utility scoring
#[derive(Clone, Debug)]
pub enum ActionOption {
    Attack(Entity),
    Defend(Vec3),
    Gather,
    Explore,
    Retreat,
}

// Decision making system
#[allow(clippy::type_complexity)]
pub fn decision_system(
    mut query: Query<(
        Entity,
        &mut DecisionMaker,
        &Transform,
        Option<&Unit>,
        Option<&Team>,
    )>,
    enemy_query: Query<(Entity, &Transform, &Team)>,
    time: Res<Time>,
) {
    let current_time = time.elapsed_secs();

    for (entity, mut decision_maker, transform, unit, team) in query.iter_mut() {
        // Check if it's time to re-evaluate
        if current_time - decision_maker.last_evaluation < decision_maker.evaluation_interval {
            continue;
        }

        decision_maker.last_evaluation = current_time;

        // Build context
        let mut context = DecisionContext {
            entity,
            position: transform.translation,
            team_id: team.map(|t| t.id).unwrap_or(0),
            health_percentage: unit.map(|u| u.health / u.max_health).unwrap_or(1.0),
            nearby_enemies: 0,
            nearby_allies: 0,
            threat_level: 0.0,
            has_resources: false,
            time_elapsed: current_time,
        };

        // Count nearby units
        let detection_range = 30.0;
        for (enemy_entity, enemy_transform, enemy_team) in enemy_query.iter() {
            let distance = transform.translation.distance(enemy_transform.translation);
            if distance <= detection_range {
                if enemy_team.id != context.team_id {
                    context.nearby_enemies += 1;
                    context.threat_level += 1.0 / distance.max(1.0);
                } else {
                    context.nearby_allies += 1;
                }
            }
        }

        // Update decision
        decision_maker.update(&context);
    }
}

// Goal execution system
pub fn goal_execution_system(
    mut query: Query<(Entity, &mut DecisionMaker, &Transform)>,
    mut commands: Commands,
) {
    for (entity, mut decision_maker, transform) in query.iter_mut() {
        if let Some(ref goal) = decision_maker.current_goal {
            match goal {
                StrategicGoal::EliminateEnemy(target) => {
                    commands
                        .entity(entity)
                        .insert(crate::ai::game_behaviors::AttackBehavior {
                            target: Some(*target),
                            aggression_level: decision_maker.personality.aggression,
                        });
                }

                StrategicGoal::DefendPosition(position) => {
                    commands
                        .entity(entity)
                        .insert(crate::ai::game_behaviors::DefendBehavior {
                            defend_position: *position,
                            patrol_radius: 10.0,
                        });
                }

                StrategicGoal::GatherResources => {
                    commands
                        .entity(entity)
                        .insert(crate::ai::game_behaviors::GatheringBehavior {
                            target_resource: None,
                            gathering_rate: 1.0,
                        });
                }

                StrategicGoal::ExploreTerrain(target) => {
                    commands.entity(entity).insert(crate::units::MovementTarget {
                        x: target.x,
                        y: target.y,
                        z: target.z,
                        reached: false,
                        speed: 4.0,
                    });
                }

                _ => {}
            }
        }
    }
}
