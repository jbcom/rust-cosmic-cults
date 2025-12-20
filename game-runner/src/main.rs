use bevy::prelude::*;
use game_physics::GamePhysicsPlugin;
use game_world::GameWorldPlugin;
use game_units::GameUnitsPlugin;
use game_combat::GameCombatPlugin;
use game_ai::GameAIPlugin;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(GamePhysicsPlugin::default())
        .add_plugins(GameWorldPlugin)
        .add_plugins(GameUnitsPlugin::default())
        .add_plugins(GameCombatPlugin)
        .add_plugins(GameAIPlugin)
        .run();
}
