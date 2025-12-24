pub mod ai;
pub mod combat;

use bevy::prelude::*;

pub struct CosmicCultsPlugin;

impl Plugin for CosmicCultsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(ai::CosmicCultsAIPlugin)
            .add_plugins(combat::CosmicCultsCombatPlugin);
    }
}
