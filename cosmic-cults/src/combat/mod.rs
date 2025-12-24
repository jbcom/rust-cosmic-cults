pub mod xp;
pub mod visuals;

use bevy::prelude::*;
pub use bevy_combat::prelude::*;

pub struct CosmicCultsCombatPlugin;

impl Plugin for CosmicCultsCombatPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(BevyCombatPlugin)
            .add_plugins(xp::XPPlugin)
            .add_plugins(visuals::CombatVisualsPlugin);
    }
}
