use bevy::prelude::*;
use bevy_egui::EguiPlugin;
use bevy_rts_camera::RtsCameraPlugin;
use cosmic_cults::{GameUnitsPlugin, GameWorldPlugin};
use leafwing_input_manager::prelude::*;
use avian3d::prelude::*;
use big_brain::BigBrainPlugin;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Cosmic Cults".to_string(),
                ..default()
            }),
            ..default()
        }))
        .add_plugins(EguiPlugin::default())
        .add_plugins(RtsCameraPlugin)
        .add_plugins(InputManagerPlugin::<PlayerAction>::default())
        .add_plugins(PhysicsPlugins::default())
        .add_plugins(BigBrainPlugin::new(PreUpdate))
        .add_plugins(MeshPickingPlugin) // For picking 3D meshes
        .add_plugins(GameWorldPlugin)
        .add_plugins(GameUnitsPlugin)
        .run();
}

#[derive(Actionlike, PartialEq, Eq, Clone, Copy, Hash, Debug, Reflect)]
pub enum PlayerAction {
    Select,
    Order,
    PanCamera,
    ZoomCamera,
    RotateCamera,
}
