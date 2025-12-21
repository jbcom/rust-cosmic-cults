# Game AI Crate

Sophisticated AI systems for game entities with cult-specific behaviors and physics integration.

## Features

### Core AI Systems
- **State Machine AI**: Hierarchical state machines with customizable transitions
- **Behavior Trees**: Flexible composite/decorator/leaf node system with blackboard
- **Utility AI**: Score-based decision making with response curves and considerations
- **Decision Making**: Goal-oriented planning with resource evaluation

### Cult-Specific Behaviors
- **Order of the Deep**: Knowledge-focused, defensive cult with high research bonus
- **Crimson Covenant**: Blood-focused, aggressive cult with high attack bonus
- **Void Seekers**: Corruption-focused, mystical cult with influence spreading

### Psychological AI Scorers
- **Thirst Drive**: Motivates resource acquisition and ritual behaviors
- **Fear Level**: Affects retreat and defensive behaviors
- **Aggression Level**: Influences attack and expansion behaviors
- **Corruption Influence**: Cult-specific mystical behavior modifier

### Physics Integration
AI behaviors automatically trigger physics movement commands:
- Gathering → Move to resource location
- Attack → Follow and engage target
- Defend → Patrol around position
- Retreat → Move to safety

## Usage

### Basic AI Entity
```rust
use game_ai::presets::*;

// Create Order of the Deep AI
let (profile, utility_ai, coordination, psychological_state) = create_order_of_deep_ai();

let entity = commands.spawn((
    Transform::from_xyz(0.0, 0.0, 0.0),
    profile,
    utility_ai,
    coordination,
    psychological_state,
    MovementController::default(),
    Velocity::default(),
)).id();
```

### Cult Profiles
Each cult has distinct behavioral modifiers:

**Order of the Deep**:
- Research Bonus: 1.6x (highest)
- Attack Bonus: 0.8x (defensive)
- Fear Threshold: 0.3 (more cautious)

**Crimson Covenant**:
- Attack Bonus: 1.5x (aggressive)
- Expansion Bonus: 1.3x
- Aggression Level: 0.8 (high)

**Void Seekers**:
- Research Bonus: 1.4x (mystical)
- Corruption Influence: 0.8 (high)
- Building Bonus: 1.2x (structures)

### AI-Physics Integration Pattern
```rust
// AI behaviors automatically integrate with physics:
commands.entity(entity).insert(AttackBehavior {
    target: Some(enemy_entity),
    aggression_level: 1.0,
});

// This triggers MovementCommandEvent in the next update:
// MovementCommandEvent {
//     entity,
//     command: MovementCommand::Follow { target: enemy_entity, distance: 1.5 }
// }
```

### Psychological Events
```rust
// Trigger psychological state changes
psychological_events.write(PsychologicalEvent {
    entity,
    event_type: PsychologicalEventType::KillEnemy,
    intensity: 0.8,
});
// Increases aggression and thirst levels
```

## Plugin Registration
Add to your Bevy app:
```rust
app.add_plugins(GameAIPlugin);
```

## Testing
Run integration tests:
```bash
cd game-ai
cargo test
```

Integration tests verify:
- AI state machine transitions
- Cult profile creation
- Psychological state systems
- AI-physics movement integration
- Entity spawning helpers
- Full AI system integration

## Architecture

### State Machines
- Default transitions between Idle/Gathering/Building/Attacking/Defending/Retreating
- Timeout-based fallbacks
- Cult-specific state modification

### Behavior Trees
- Composite nodes: Sequence, Selector, Parallel
- Decorator nodes: Inverter, Repeater, Succeeder, Failer
- Leaf nodes: Action, Condition
- Blackboard data sharing

### Utility AI
- Multiple considerations with response curves
- Multiplicative scoring with compensation
- Action selection based on highest score
- Real-time context evaluation

### Coordination
- Team-based coordination with leaders/followers
- Message passing for communication
- Role-based behavior assignment
- Radius-based influence

This crate provides a complete AI framework for complex RTS-style gameplay with deep cult-specific customization and seamless physics integration.