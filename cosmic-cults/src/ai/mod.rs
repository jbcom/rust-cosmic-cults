pub mod cult_profiles;
pub mod behaviors;
pub mod types;

use bevy::prelude::*;
use bevy_ai_toolkit::prelude::*;

use crate::ai::types::{AICoordination, AIRole};

pub struct CosmicCultsAIPlugin;

impl Plugin for CosmicCultsAIPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(BevyAIToolkitPlugin)
            .add_systems(Update, (
                cult_profiles::update_psychological_state_system,
                cult_profiles::handle_psychological_events,
                ai_coordination_system,
            ));
    }
}

fn ai_coordination_system(
    leaders_query: Query<(Entity, &AICoordination, &Transform)>,
    followers_query: Query<(Entity, &AICoordination, &Transform)>,
    mut commands: Commands,
) {
    for (leader_entity, leader_coord, leader_transform) in leaders_query.iter() {
        if leader_coord.can_give_orders && leader_coord.role == AIRole::Leader {
            for (follower_entity, follower_coord, follower_transform) in followers_query.iter() {
                if leader_entity == follower_entity || !follower_coord.can_receive_orders || follower_coord.team_id != leader_coord.team_id {
                    continue;
                }

                let distance = leader_transform.translation.distance(follower_transform.translation);
                if distance <= leader_coord.coordination_radius {
                    commands.entity(follower_entity).insert(CoordinatedBehavior {
                        leader: leader_entity,
                        role: leader_coord.role.clone(),
                    });
                }
            }
        }
    }
}

#[derive(Component)]
pub struct CoordinatedBehavior {
    pub leader: Entity,
    pub role: AIRole,
}
