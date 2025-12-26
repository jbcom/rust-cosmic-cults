//! Build menu for spawning units and buildings

use bevy::prelude::*;
use game_units::{spawn_unit, GameAssets};

/// Marker component for the build menu container
#[derive(Component)]
pub struct BuildMenu;

/// Marker component for build menu buttons
#[derive(Component)]
pub struct BuildButton {
    pub unit_type: UnitType,
}

/// Types of units/buildings that can be spawned
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnitType {
    Worker,
    Warrior,
    Leader,
    Totem,
}

impl UnitType {
    fn label(&self) -> &str {
        match self {
            UnitType::Worker => "Worker",
            UnitType::Warrior => "Warrior",
            UnitType::Leader => "Leader",
            UnitType::Totem => "Totem",
        }
    }

    fn hotkey(&self) -> &str {
        match self {
            UnitType::Worker => "W",
            UnitType::Warrior => "A",
            UnitType::Leader => "L",
            UnitType::Totem => "T",
        }
    }

    fn to_unit_type_str(&self) -> &str {
        match self {
            UnitType::Worker => "cultist",
            UnitType::Warrior => "warrior",
            UnitType::Leader => "leader",
            UnitType::Totem => "totem",
        }
    }
}

/// Sets up the build menu UI
pub fn setup_build_menu(mut commands: Commands) {
    let unit_types = vec![
        UnitType::Worker,
        UnitType::Warrior,
        UnitType::Leader,
        UnitType::Totem,
    ];

    // Create build menu container at bottom center
    commands
        .spawn((
            Node {
                width: Val::Px(400.0),
                height: Val::Px(80.0),
                position_type: PositionType::Absolute,
                left: Val::Percent(50.0),
                bottom: Val::Px(20.0),
                margin: UiRect {
                    left: Val::Px(-200.0), // Center by offsetting half width
                    ..default()
                },
                padding: UiRect::all(Val::Px(10.0)),
                flex_direction: FlexDirection::Row,
                justify_content: JustifyContent::SpaceEvenly,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(Color::srgba(0.15, 0.15, 0.2, 0.9)),
            BuildMenu,
            Name::new("Build Menu"),
        ))
        .with_children(|parent| {
            // Create buttons for each unit type
            for unit_type in unit_types {
                parent
                    .spawn((
                        Button,
                        Node {
                            width: Val::Px(80.0),
                            height: Val::Px(60.0),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            flex_direction: FlexDirection::Column,
                            ..default()
                        },
                        BackgroundColor(Color::srgba(0.2, 0.2, 0.3, 1.0)),
                        BuildButton { unit_type },
                        Name::new(format!("{} Button", unit_type.label())),
                    ))
                    .with_children(|button| {
                        // Unit type label
                        button.spawn((
                            Text::new(unit_type.label()),
                            TextFont {
                                font_size: 14.0,
                                ..default()
                            },
                            TextColor(Color::srgb(1.0, 1.0, 1.0)),
                        ));
                        
                        // Hotkey hint
                        button.spawn((
                            Text::new(format!("({})", unit_type.hotkey())),
                            TextFont {
                                font_size: 10.0,
                                ..default()
                            },
                            TextColor(Color::srgb(0.7, 0.7, 0.7)),
                        ));
                    });
            }
        });
}

/// Handles build menu button clicks and keyboard shortcuts
pub fn handle_build_menu_input(
    mut commands: Commands,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut interaction_query: Query<
        (&Interaction, &BuildButton, &mut BackgroundColor),
        Changed<Interaction>,
    >,
    camera_query: Query<&Transform, With<Camera3d>>,
    game_assets: Option<Res<GameAssets>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Return early if assets aren't loaded yet
    let Some(assets) = game_assets else {
        return;
    };

    // Handle keyboard shortcuts
    let spawn_type = if keyboard.just_pressed(KeyCode::KeyW) {
        Some(UnitType::Worker)
    } else if keyboard.just_pressed(KeyCode::KeyA) {
        Some(UnitType::Warrior)
    } else if keyboard.just_pressed(KeyCode::KeyL) {
        Some(UnitType::Leader)
    } else if keyboard.just_pressed(KeyCode::KeyT) {
        Some(UnitType::Totem)
    } else {
        None
    };

    if let Some(unit_type) = spawn_type {
        spawn_unit_from_build_menu(&mut commands, unit_type, &camera_query, &assets, &mut materials);
    }

    // Handle button clicks
    for (interaction, button, mut color) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                *color = BackgroundColor(Color::srgba(0.3, 0.5, 0.3, 1.0));
                spawn_unit_from_build_menu(&mut commands, button.unit_type, &camera_query, &assets, &mut materials);
            }
            Interaction::Hovered => {
                *color = BackgroundColor(Color::srgba(0.3, 0.3, 0.4, 1.0));
            }
            Interaction::None => {
                *color = BackgroundColor(Color::srgba(0.2, 0.2, 0.3, 1.0));
            }
        }
    }
}

/// Spawns a unit based on the build menu selection
fn spawn_unit_from_build_menu(
    commands: &mut Commands,
    unit_type: UnitType,
    camera_query: &Query<&Transform, With<Camera3d>>,
    assets: &GameAssets,
    materials: &mut Assets<StandardMaterial>,
) {
    // Get camera position to spawn units in front of camera
    let spawn_position = if let Ok(camera_transform) = camera_query.single() {
        // Spawn unit in front of camera at ground level
        let forward = camera_transform.forward().as_vec3();
        camera_transform.translation + forward.normalize() * Vec3::new(10.0, 0.0, 10.0) - Vec3::new(0.0, camera_transform.translation.y - 0.5, 0.0)
    } else {
        Vec3::new(0.0, 0.5, 5.0)
    };

    // Spawn the unit using the existing spawning system
    let unit_type_str = unit_type.to_unit_type_str();
    let cult = "crimson_covenant"; // Default cult for player
    let team_id = 0; // Player team
    
    spawn_unit(
        commands,
        unit_type_str,
        spawn_position,
        cult,
        team_id,
        assets,
        materials,
    );
    
    info!("Spawned {} at {:?}", unit_type.label(), spawn_position);
}
