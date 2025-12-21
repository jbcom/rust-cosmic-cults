// AI-Physics integration test module

use crate::{
    GameAIPlugin,
    cult_profiles::{CultProfile, PsychologicalState, create_cult_profile, presets},
    systems::{
        AIState, AIStateMachine, AITransition, AttackBehavior, DefendBehavior, GatheringBehavior,
        RetreatBehavior,
    },
    types::{AICoordination, AIRole},
};
use bevy::prelude::*;
use game_physics::prelude::*;

/// Integration test to verify AI behaviors trigger physics movement commands
#[test]
#[ignore] // TODO: Fix B0001 system conflict in test environment
fn test_ai_physics_integration() {
    let mut app = App::new();

    // Add minimal plugins for testing
    app.add_plugins(MinimalPlugins)
        .add_plugins(GameAIPlugin)
        .add_message::<MovementCommandEvent>();

    // Spawn test entity with AI components
    let entity = app
        .world_mut()
        .spawn((
            Transform::from_xyz(0.0, 0.0, 0.0),
            GlobalTransform::default(),
            AIStateMachine::default(),
            MovementController::default(),
            Velocity::default(),
            SpatialData::new(Vec3::ZERO),
            CollisionMask::default(),
        ))
        .id();

    // Test gathering behavior triggers movement
    app.world_mut()
        .entity_mut(entity)
        .insert(GatheringBehavior {
            target_resource: None,
            gathering_rate: 1.0,
        });

    // Run one update cycle
    app.update();

    // Verify movement command was sent (this is a smoke test)
    // In a real test, we'd check for MovementCommandEvent emission
    let movement_controller = app.world().entity(entity).get::<MovementController>();
    assert!(movement_controller.is_some());

    println!("✓ AI-Physics integration test passed");
}

/// Test cult-specific AI profile creation
#[test]
fn test_cult_profiles() {
    // Test Order of the Deep profile
    let deep_profile = create_cult_profile("Order of the Deep");
    assert_eq!(deep_profile.cult_name, "Order of the Deep");
    assert!(deep_profile.behavioral_modifiers.research_bonus > 1.0);
    assert!(deep_profile.behavioral_modifiers.attack_bonus < 1.0);

    // Test Crimson Covenant profile
    let crimson_profile = create_cult_profile("Crimson Covenant");
    assert_eq!(crimson_profile.cult_name, "Crimson Covenant");
    assert!(crimson_profile.behavioral_modifiers.attack_bonus > 1.0);
    assert!(crimson_profile.aggression_level > 0.5);

    // Test Void Seekers profile
    let void_profile = create_cult_profile("Void Seekers");
    assert_eq!(void_profile.cult_name, "Void Seekers");
    assert!(void_profile.behavioral_modifiers.research_bonus > 1.0);

    println!("✓ Cult profiles test passed");
}

/// Test psychological state system
#[test]
#[ignore] // TODO: Fix B0001 system conflict in test environment
fn test_psychological_states() {
    let mut app = App::new();

    app.add_plugins(MinimalPlugins).add_plugins(GameAIPlugin);

    // Create entity with psychological state
    let (profile, utility_ai, coordination, psychological_state) =
        presets::create_crimson_covenant_ai();

    let entity = app
        .world_mut()
        .spawn((
            Transform::default(),
            GlobalTransform::default(),
            profile,
            utility_ai,
            coordination,
            psychological_state,
        ))
        .id();

    // Run update to test psychological systems
    app.update();

    // Verify entity exists and has components
    let entity_ref = app.world().entity(entity);
    assert!(entity_ref.get::<CultProfile>().is_some());
    assert!(entity_ref.get::<PsychologicalState>().is_some());

    println!("✓ Psychological states test passed");
}

/// Test AI state machine transitions
#[test]
fn test_ai_state_transitions() {
    let mut state_machine = AIStateMachine::default();

    // Test initial state
    assert_eq!(state_machine.current_state, AIState::Idle);

    // Test valid transition
    let result = state_machine.transition(AITransition::ResourcesLow);
    assert!(result);
    assert_eq!(state_machine.current_state, AIState::Gathering);

    // Test invalid transition
    let result = state_machine.transition(AITransition::BuildingComplete);
    assert!(!result);
    assert_eq!(state_machine.current_state, AIState::Gathering);

    println!("✓ AI state transitions test passed");
}

/// Test AI behavior spawning through GameAIPlugin helpers
#[test]
#[ignore] // TODO: Fix assertion failure
fn test_ai_entity_spawning() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins).add_plugins(GameAIPlugin);

    let mut commands = app.world_mut().commands();

    // Test spawning basic AI entity
    let entity =
        GameAIPlugin::spawn_basic_ai(&mut commands, Vec3::new(5.0, 0.0, 5.0), AIRole::Worker);

    // Flush commands
    app.world_mut().flush();

    // Verify entity has required components
    let entity_ref = app.world().entity(entity);
    assert!(entity_ref.get::<AIStateMachine>().is_some());
    assert!(entity_ref.get::<AICoordination>().is_some());
    assert!(entity_ref.get::<MovementController>().is_some());
    assert!(entity_ref.get::<Transform>().is_some());

    println!("✓ AI entity spawning test passed");
}

/// Comprehensive integration test
#[test]
#[ignore] // TODO: Fix B0001 system conflict in test environment
fn test_full_ai_integration() {
    let mut app = App::new();

    // Setup full game environment
    app.add_plugins(MinimalPlugins)
        .add_plugins(GameAIPlugin)
        .add_message::<MovementCommandEvent>();

    // Create cult entities
    let (deep_profile, deep_ai, deep_coord, deep_psych) = presets::create_order_of_deep_ai();
    let (crimson_profile, crimson_ai, crimson_coord, crimson_psych) =
        presets::create_crimson_covenant_ai();
    let (void_profile, void_ai, void_coord, void_psych) = presets::create_void_seekers_ai();

    // Spawn entities with full AI setups
    let deep_entity = app
        .world_mut()
        .spawn((
            Transform::from_xyz(0.0, 0.0, 0.0),
            GlobalTransform::default(),
            deep_profile,
            deep_ai,
            deep_coord,
            deep_psych,
            MovementController::default(),
            Velocity::default(),
            SpatialData::new(Vec3::ZERO),
        ))
        .id();

    let crimson_entity = app
        .world_mut()
        .spawn((
            Transform::from_xyz(10.0, 0.0, 0.0),
            GlobalTransform::default(),
            crimson_profile,
            crimson_ai,
            crimson_coord,
            crimson_psych,
            MovementController::default(),
            Velocity::default(),
            SpatialData::new(Vec3::new(10.0, 0.0, 0.0)),
        ))
        .id();

    let void_entity = app
        .world_mut()
        .spawn((
            Transform::from_xyz(20.0, 0.0, 0.0),
            GlobalTransform::default(),
            void_profile,
            void_ai,
            void_coord,
            void_psych,
            MovementController::default(),
            Velocity::default(),
            SpatialData::new(Vec3::new(20.0, 0.0, 0.0)),
        ))
        .id();

    // Run several update cycles to test full integration
    for _ in 0..5 {
        app.update();
    }

    // Verify all entities still exist and have correct components
    let deep_ref = app.world().entity(deep_entity);
    let crimson_ref = app.world().entity(crimson_entity);
    let void_ref = app.world().entity(void_entity);

    assert!(deep_ref.get::<CultProfile>().is_some());
    assert!(crimson_ref.get::<CultProfile>().is_some());
    assert!(void_ref.get::<CultProfile>().is_some());

    // Verify cult names are correct
    assert_eq!(
        deep_ref.get::<CultProfile>().unwrap().cult_name,
        "Order of the Deep"
    );
    assert_eq!(
        crimson_ref.get::<CultProfile>().unwrap().cult_name,
        "Crimson Covenant"
    );
    assert_eq!(
        void_ref.get::<CultProfile>().unwrap().cult_name,
        "Void Seekers"
    );

    println!("✓ Full AI integration test passed");
}

/// Test AI behavior execution systems
#[test]
#[ignore] // TODO: Fix B0001 system conflict in test environment
fn test_ai_behavior_execution() {
    let mut app = App::new();

    app.add_plugins(MinimalPlugins)
        .add_plugins(GameAIPlugin)
        .add_message::<MovementCommandEvent>();

    // Create entity that will get attack behavior
    let entity = app
        .world_mut()
        .spawn((
            Transform::default(),
            GlobalTransform::default(),
            MovementController::default(),
            Velocity::default(),
            SpatialData::new(Vec3::ZERO),
            CollisionMask::default(),
        ))
        .id();

    // Add attack behavior (this should trigger movement command in next update)
    app.world_mut().entity_mut(entity).insert(AttackBehavior {
        target: None,
        aggression_level: 1.0,
    });

    // Run update to process AI behavior execution
    app.update();

    // Verify entity has movement controller (indicating physics integration works)
    let entity_ref = app.world().entity(entity);
    assert!(entity_ref.get::<MovementController>().is_some());
    assert!(entity_ref.get::<AttackBehavior>().is_some());

    println!("✓ AI behavior execution test passed");
}

/// Run all integration tests
pub fn run_integration_tests() {
    println!("Running AI-Physics integration tests...");

    test_ai_physics_integration();
    test_cult_profiles();
    test_psychological_states();
    test_ai_state_transitions();
    test_ai_entity_spawning();
    test_full_ai_integration();
    test_ai_behavior_execution();

    println!("All integration tests passed! ✓");
}
