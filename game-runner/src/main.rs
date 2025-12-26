use bevy::prelude::*;
use game_ai::GameAIPlugin;
use game_audio::GameAudioPlugin;
use game_combat::GameCombatPlugin;
use game_physics::GamePhysicsPlugin;
use game_units::GameUnitsPlugin;
use game_world::GameWorldPlugin;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(GamePhysicsPlugin::default())
        .add_plugins(GameWorldPlugin)
        .add_plugins(GameUnitsPlugin)
        .add_plugins(GameCombatPlugin)
        .add_plugins(GameAIPlugin)
        .add_plugins(GameAudioPlugin)
        .run();
}
