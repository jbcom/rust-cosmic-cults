mod ai;
mod assets;
mod units;
mod world;

use bevy::prelude::*;
use bevy_combat::GameCombatPlugin;
use ai::GameAIPlugin;
use game_physics::GamePhysicsPlugin;
use units::GameUnitsPlugin;
use world::GameWorldPlugin;

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
