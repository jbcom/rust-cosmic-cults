use bevy::prelude::*;
use cosmic_cults::CosmicCultsPlugin;
use game_physics::GamePhysicsPlugin;
use game_units::GameUnitsPlugin;
use game_world::GameWorldPlugin;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(GamePhysicsPlugin::default())
        .add_plugins(GameWorldPlugin)
        .add_plugins(GameUnitsPlugin)
        .add_plugins(CosmicCultsPlugin)
        .run();
}
