use bevy::prelude::*;
use std::collections::{HashMap, VecDeque};

// Resource abstraction for AI decision making
#[derive(Clone, Debug)]
pub struct TeamResources {
    pub current: HashMap<String, u32>,
    pub income_rate: HashMap<String, f32>,
    pub capacity: HashMap<String, u32>,
}

impl Default for TeamResources {
    fn default() -> Self {
        let mut current = HashMap::new();
        let mut income_rate = HashMap::new();
        let mut capacity = HashMap::new();

        // Default resources
        current.insert("essence".to_string(), 100);
        current.insert("souls".to_string(), 10);
        current.insert("knowledge".to_string(), 5);

        income_rate.insert("essence".to_string(), 5.0);
        income_rate.insert("souls".to_string(), 1.0);
        income_rate.insert("knowledge".to_string(), 0.5);

        capacity.insert("essence".to_string(), 10000);
        capacity.insert("souls".to_string(), 1000);
        capacity.insert("knowledge".to_string(), 500);

        Self {
            current,
            income_rate,
            capacity,
        }
    }
}

// AI Priorities for decision weighting
#[derive(Clone, Debug)]
pub struct AIPriorities {
    pub economy: f32,
    pub military: f32,
    pub technology: f32,
    pub expansion: f32,
}

impl Default for AIPriorities {
    fn default() -> Self {
        Self {
            economy: 0.4,
            military: 0.3,
            technology: 0.2,
            expansion: 0.1,
        }
    }
}

// Decision types for AI actions
#[derive(Clone, Debug)]
pub enum DecisionType {
    BuildUnit(String),
    BuildStructure(String),
    Research(String),
    Attack(Vec3),
    Defend(Vec3),
    Expand(Vec3),
}

// AI decision maker component
#[derive(Component, Clone, Debug)]
pub struct AIDecisionMaker {
    pub decision_queue: VecDeque<AIDecision>,
    pub evaluation_scores: Vec<(DecisionType, f32)>,
    pub decision_cooldown: f32,
    pub last_decision_time: f32,
}

impl Default for AIDecisionMaker {
    fn default() -> Self {
        Self {
            decision_queue: VecDeque::new(),
            evaluation_scores: Vec::new(),
            decision_cooldown: 1.0,
            last_decision_time: 0.0,
        }
    }
}

#[derive(Clone, Debug)]
pub struct AIDecision {
    pub decision_type: DecisionType,
    pub priority: f32,
    pub timestamp: f32,
    pub expires_at: f32,
}

impl AIDecisionMaker {
    pub fn evaluate_options(
        &mut self,
        priorities: &AIPriorities,
        resources: &TeamResources,
        unit_count: usize,
        building_count: usize,
        current_time: f32,
    ) {
        self.evaluation_scores.clear();

        // Evaluate building units
        let unit_score = evaluate_unit_production(priorities, resources, unit_count);
        self.evaluation_scores
            .push((DecisionType::BuildUnit("warrior".to_string()), unit_score));

        // Evaluate building structures
        let structure_score = evaluate_structure_building(priorities, resources, building_count);
        self.evaluation_scores.push((
            DecisionType::BuildStructure("essence_extractor".to_string()),
            structure_score,
        ));

        // Evaluate research
        let research_score = evaluate_research(priorities, resources);
        self.evaluation_scores.push((
            DecisionType::Research("improved_extraction".to_string()),
            research_score,
        ));

        // Evaluate military actions
        let attack_score = evaluate_attack(priorities, unit_count);
        self.evaluation_scores.push((
            DecisionType::Attack(Vec3::new(100.0, 0.0, 100.0)),
            attack_score,
        ));

        // Evaluate expansion
        let expansion_score = evaluate_expansion(priorities, resources, building_count);
        self.evaluation_scores.push((
            DecisionType::Expand(Vec3::new(50.0, 0.0, 50.0)),
            expansion_score,
        ));

        // Sort by score and add best decisions to queue
        self.evaluation_scores
            .sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Less));

        // Collect decisions to add to avoid borrowing conflicts
        let decisions_to_add: Vec<(DecisionType, f32)> = self
            .evaluation_scores
            .iter()
            .take(3)
            .filter(|(_, score)| *score > 0.5)
            .map(|(decision_type, score)| (decision_type.clone(), *score))
            .collect();

        // Add decisions to queue
        for (decision_type, score) in decisions_to_add {
            self.add_decision(decision_type, score, current_time);
        }
    }

    pub fn add_decision(&mut self, decision_type: DecisionType, priority: f32, current_time: f32) {
        let decision = AIDecision {
            decision_type,
            priority,
            timestamp: current_time,
            expires_at: current_time + 30.0,
        };

        // Insert based on priority
        let position = self
            .decision_queue
            .iter()
            .position(|d| d.priority < priority)
            .unwrap_or(self.decision_queue.len());

        self.decision_queue.insert(position, decision);

        // Limit queue size
        if self.decision_queue.len() > 10 {
            self.decision_queue.truncate(10);
        }
    }

    pub fn get_next_decision(&self) -> Option<&AIDecision> {
        self.decision_queue.front()
    }

    pub fn complete_decision(&mut self) {
        self.decision_queue.pop_front();
    }

    pub fn clear_expired_decisions(&mut self, current_time: f32) {
        self.decision_queue.retain(|d| d.expires_at > current_time);
    }
}

// Evaluation functions
pub fn evaluate_decision(
    decision_type: &DecisionType,
    priorities: &AIPriorities,
    resources: &TeamResources,
    context: &EvaluationContext,
) -> f32 {
    match decision_type {
        DecisionType::BuildUnit(_) => {
            evaluate_unit_production(priorities, resources, context.unit_count)
        }
        DecisionType::BuildStructure(_) => {
            evaluate_structure_building(priorities, resources, context.building_count)
        }
        DecisionType::Research(_) => evaluate_research(priorities, resources),
        DecisionType::Attack(_) => evaluate_attack(priorities, context.unit_count),
        DecisionType::Defend(_) => evaluate_defense(priorities, context),
        DecisionType::Expand(_) => {
            evaluate_expansion(priorities, resources, context.building_count)
        }
    }
}

#[derive(Debug)]
pub struct EvaluationContext {
    pub unit_count: usize,
    pub building_count: usize,
    pub enemy_strength: f32,
    pub threat_level: f32,
    pub map_control: f32,
}

fn evaluate_unit_production(
    priorities: &AIPriorities,
    resources: &TeamResources,
    unit_count: usize,
) -> f32 {
    let mut score = priorities.military;

    // Adjust based on unit count
    if unit_count < 5 {
        score *= 2.0; // High priority if few units
    } else if unit_count > 20 {
        score *= 0.5; // Lower priority if many units
    }

    // Check affordability
    let essence = resources.current.get("essence").unwrap_or(&0);
    if essence < &100 {
        score *= 0.1; // Can't afford
    }

    score.min(1.0)
}

fn evaluate_structure_building(
    priorities: &AIPriorities,
    resources: &TeamResources,
    building_count: usize,
) -> f32 {
    let mut score = priorities.economy;

    // Adjust based on building count
    if building_count < 3 {
        score *= 1.5; // Need more infrastructure
    }

    // Check resources
    let essence = resources.current.get("essence").unwrap_or(&0);
    if essence < &200 {
        score *= 0.2;
    }

    // Check income rate
    let income_rate = resources.income_rate.get("essence").unwrap_or(&0.0);
    if income_rate < &10.0 {
        score *= 1.3; // Need more income
    }

    score.min(1.0)
}

fn evaluate_research(priorities: &AIPriorities, resources: &TeamResources) -> f32 {
    let mut score = priorities.technology;

    // Check if we have sufficient resources
    let knowledge = resources.current.get("knowledge").unwrap_or(&0);
    if knowledge < &50 {
        score *= 0.1;
    }

    // Research is more valuable mid-late game
    let essence = resources.current.get("essence").unwrap_or(&0);
    if essence > &2000 {
        score *= 1.5;
    }

    score.min(1.0)
}

fn evaluate_attack(priorities: &AIPriorities, unit_count: usize) -> f32 {
    let mut score = priorities.military * 0.8;

    // Need sufficient units to attack
    if unit_count < 10 {
        score *= 0.2;
    } else if unit_count > 20 {
        score *= 1.5;
    }

    score.min(1.0)
}

fn evaluate_defense(priorities: &AIPriorities, context: &EvaluationContext) -> f32 {
    let mut score = priorities.military * 0.6;

    // Increase if under threat
    score *= 1.0 + context.threat_level;

    // Decrease if we have strong defense
    if context.unit_count > 15 {
        score *= 0.7;
    }

    score.min(1.0)
}

fn evaluate_expansion(
    priorities: &AIPriorities,
    resources: &TeamResources,
    building_count: usize,
) -> f32 {
    let mut score = priorities.expansion;

    // Need resources to expand
    let essence = resources.current.get("essence").unwrap_or(&0);
    if essence < &500 {
        score *= 0.1;
    } else if essence > &2000 {
        score *= 1.5;
    }

    // Don't over-expand
    if building_count > 10 {
        score *= 0.5;
    }

    score.min(1.0)
}

// Goal-oriented planning
#[derive(Component, Clone, Debug)]
pub struct AIGoal {
    pub goal_type: GoalType,
    pub target_value: f32,
    pub current_value: f32,
    pub priority: f32,
    pub sub_goals: Vec<AIGoal>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum GoalType {
    AchieveUnitCount(usize),
    AchieveResourceAmount(String, u32),
    DestroyEnemy(Entity),
    CaptureLocation(Vec3),
    DefendLocation(Vec3),
    ResearchTechnology(String),
}

impl AIGoal {
    pub fn evaluate_progress(&self) -> f32 {
        if self.target_value > 0.0 {
            (self.current_value / self.target_value).min(1.0)
        } else {
            1.0
        }
    }

    pub fn is_complete(&self) -> bool {
        self.current_value >= self.target_value
    }

    pub fn update_progress(&mut self, new_value: f32) {
        self.current_value = new_value;
    }
}

// Decision-making system that processes AIDecisionMaker components
pub fn decision_making_system(
    time: Res<Time>,
    mut query: Query<(Entity, &mut AIDecisionMaker, &Transform), With<AIDecisionMaker>>,
    mut commands: Commands,
) {
    let current_time = time.elapsed_seconds();

    for (entity, mut decision_maker, transform) in query.iter_mut() {
        // Check cooldown
        if current_time - decision_maker.last_decision_time < decision_maker.decision_cooldown {
            continue;
        }

        // Clear expired decisions
        decision_maker.clear_expired_decisions(current_time);

        // Re-evaluate decisions periodically
        if decision_maker.decision_queue.is_empty()
            || current_time - decision_maker.last_decision_time > 5.0
        {
            // Create evaluation context (in real implementation, this would come from game state)
            let priorities = AIPriorities::default();
            let resources = TeamResources::default();
            let unit_count = 1; // Would count actual units
            let building_count = 1; // Would count actual buildings

            // Evaluate new decisions
            decision_maker.evaluate_options(
                &priorities,
                &resources,
                unit_count,
                building_count,
                current_time,
            );
            decision_maker.last_decision_time = current_time;
        }

        // Execute next decision
        if let Some(decision) = decision_maker.get_next_decision() {
            match &decision.decision_type {
                DecisionType::BuildUnit(unit_type) => {
                    // Would add unit construction behavior
                    println!("AI Decision: Build unit {}", unit_type);
                }
                DecisionType::BuildStructure(structure_type) => {
                    // Would add structure construction behavior
                    println!("AI Decision: Build structure {}", structure_type);
                }
                DecisionType::Research(tech) => {
                    // Would add research behavior
                    println!("AI Decision: Research {}", tech);
                }
                DecisionType::Attack(target_pos) => {
                    commands.entity(entity).insert(crate::AttackBehavior {
                        target: None, // Would get target entity from position
                        aggression_level: 0.8,
                    });
                    println!("AI Decision: Attack at {:?}", target_pos);
                }
                DecisionType::Defend(defend_pos) => {
                    commands.entity(entity).insert(crate::DefendBehavior {
                        defend_position: *defend_pos,
                        patrol_radius: 15.0,
                    });
                    println!("AI Decision: Defend at {:?}", defend_pos);
                }
                DecisionType::Expand(expansion_pos) => {
                    // Would add expansion behavior
                    println!("AI Decision: Expand to {:?}", expansion_pos);
                }
            }

            // Mark decision as completed
            let mut decision_maker_mut = decision_maker.into_inner();
            decision_maker_mut.complete_decision();
        }
    }
}
