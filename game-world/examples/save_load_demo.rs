//! Example demonstrating game state save/load functionality
//!
//! This example shows how to use the save_load module to:
//! - Serialize game state (map, units, resources) to a file
//! - Deserialize and restore game state from a file
//!
//! Run with: cargo run --example save_load_demo

use game_world::{
    GameMap, GameState, PathfindingGrid, SerializableLeader, SerializableUnit, VisibilityMap,
};

fn main() {
    println!("=== Game State Save/Load Demo ===\n");

    // Create a game state with default resources
    let game_map = GameMap::default();
    let pathfinding_grid = PathfindingGrid::default();
    let visibility_map = VisibilityMap::default();

    let mut game_state = GameState::new(game_map, pathfinding_grid, visibility_map);

    println!("1. Creating game state...");
    println!(
        "   - Map size: {}x{}",
        game_state.game_map.width, game_state.game_map.height
    );
    println!("   - Tile count: {}", game_state.game_map.tiles.len());

    // Add some units to the game state
    println!("\n2. Adding units to game state...");

    // Add a player unit
    game_state.units.push(SerializableUnit {
        position: (10.0, 0.0, 10.0),
        rotation: (0.0, 0.0, 0.0, 1.0),
        cult: String::from("Player"),
        unit_type: String::from("Infantry"),
        health: 100.0,
        max_health: 100.0,
        experience: 50,
        veteran_tier: 1,
    });

    // Add another unit
    game_state.units.push(SerializableUnit {
        position: (15.0, 0.0, 15.0),
        rotation: (0.0, 0.707, 0.0, 0.707),
        cult: String::from("Player"),
        unit_type: String::from("Archer"),
        health: 80.0,
        max_health: 100.0,
        experience: 120,
        veteran_tier: 2,
    });

    // Add a leader
    game_state.leaders.push(SerializableLeader {
        position: (0.0, 0.0, 0.0),
        rotation: (0.0, 0.0, 0.0, 1.0),
        name: String::from("Commander Azathoth"),
        cult: String::from("Player"),
        health: 200.0,
        max_health: 200.0,
        shield: 50.0,
        aura_radius: 15.0,
        aura_type: String::from("Leadership"),
        alive: true,
    });

    println!("   - Added {} units", game_state.units.len());
    println!("   - Added {} leaders", game_state.leaders.len());

    // Add some custom data
    game_state
        .custom_data
        .insert(String::from("difficulty"), String::from("Hard"));
    game_state
        .custom_data
        .insert(String::from("mission_id"), String::from("mission_1"));

    println!("   - Added custom data: {:?}", game_state.custom_data);

    // Serialize to bytes
    println!("\n3. Serializing game state to bytes...");
    let bytes = game_state
        .to_bytes()
        .expect("Failed to serialize game state");
    println!("   - Serialized size: {} bytes", bytes.len());

    // Save to file
    let save_path = "/tmp/game_save.bin";
    println!("\n4. Saving game state to file: {}", save_path);
    game_state
        .save_to_file(save_path)
        .expect("Failed to save game state to file");
    println!("   - Save successful!");

    // Load from file
    println!("\n5. Loading game state from file...");
    let loaded_state =
        GameState::load_from_file(save_path).expect("Failed to load game state from file");
    println!("   - Load successful!");

    // Verify loaded state
    println!("\n6. Verifying loaded game state...");
    println!("   - Version: {}", loaded_state.version);
    println!("   - Timestamp: {}", loaded_state.timestamp);
    println!(
        "   - Map size: {}x{}",
        loaded_state.game_map.width, loaded_state.game_map.height
    );
    println!("   - Tile count: {}", loaded_state.game_map.tiles.len());
    println!("   - Units count: {}", loaded_state.units.len());
    println!("   - Leaders count: {}", loaded_state.leaders.len());

    // Print unit details
    println!("\n7. Unit details:");
    for (i, unit) in loaded_state.units.iter().enumerate() {
        println!("   Unit {}: {} ({})", i + 1, unit.unit_type, unit.cult);
        println!(
            "      Position: ({:.1}, {:.1}, {:.1})",
            unit.position.0, unit.position.1, unit.position.2
        );
        println!("      Health: {:.0}/{:.0}", unit.health, unit.max_health);
        println!(
            "      Experience: {}, Veteran Tier: {}",
            unit.experience, unit.veteran_tier
        );
    }

    // Print leader details
    println!("\n8. Leader details:");
    for (i, leader) in loaded_state.leaders.iter().enumerate() {
        println!("   Leader {}: {} ({})", i + 1, leader.name, leader.cult);
        println!(
            "      Position: ({:.1}, {:.1}, {:.1})",
            leader.position.0, leader.position.1, leader.position.2
        );
        println!(
            "      Health: {:.0}/{:.0}, Shield: {:.0}",
            leader.health, leader.max_health, leader.shield
        );
        println!(
            "      Aura: {} (radius: {:.1})",
            leader.aura_type, leader.aura_radius
        );
        println!(
            "      Status: {}",
            if leader.alive { "Alive" } else { "Dead" }
        );
    }

    // Print custom data
    println!("\n9. Custom data:");
    for (key, value) in &loaded_state.custom_data {
        println!("   {}: {}", key, value);
    }

    // Test round-trip integrity
    println!("\n10. Testing round-trip integrity...");
    let bytes1 = game_state.to_bytes().unwrap();
    let bytes2 = loaded_state.to_bytes().unwrap();

    if bytes1 == bytes2 {
        println!("   ✓ Round-trip successful - saved and loaded states are identical!");
    } else {
        println!("   ✗ Warning: Saved and loaded states differ");
        println!("      Original size: {} bytes", bytes1.len());
        println!("      Loaded size: {} bytes", bytes2.len());
    }

    println!("\n=== Demo Complete ===");
    println!("\nSave file located at: {}", save_path);
    println!("You can inspect the binary save file or delete it when done.");
}
