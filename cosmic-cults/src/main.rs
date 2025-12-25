use bevy::prelude::*;
use bevy_combat::GameCombatPlugin;
use cosmic_cults::{GameAIPlugin, GameUnitsPlugin, GameWorldPlugin};
use game_physics::GamePhysicsPlugin;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(GamePhysicsPlugin::default())
        .add_plugins(GameWorldPlugin)
        .add_plugins(GameUnitsPlugin)
        .add_plugins(GameCombatPlugin)
        .add_plugins(GameAIPlugin)
        .run();
}
