//! Minimap for navigation across larger maps

use bevy::prelude::*;
use game_world::{GameMap, VisionProvider};

/// Marker component for the minimap container
#[derive(Component)]
pub struct Minimap;

/// Marker component for minimap camera viewport
#[derive(Component)]
pub struct MinimapCamera;

/// Sets up the minimap UI
pub fn setup_minimap(mut commands: Commands) {
    // Create minimap container at top-right
    commands
        .spawn((
            Node {
                width: Val::Px(200.0),
                height: Val::Px(200.0),
                position_type: PositionType::Absolute,
                right: Val::Px(20.0),
                top: Val::Px(20.0),
                border: UiRect::all(Val::Px(2.0)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.1, 0.1, 0.15, 0.8)),
            BorderColor::all(Color::srgb(0.3, 0.3, 0.4)),
            Minimap,
            Interaction::None,
            Name::new("Minimap"),
        ))
        .with_children(|parent| {
            // Title label
            parent.spawn((
                Node {
                    width: Val::Percent(100.0),
                    height: Val::Px(20.0),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                },
                BackgroundColor(Color::srgba(0.15, 0.15, 0.2, 1.0)),
            ))
            .with_children(|title| {
                title.spawn((
                    Text::new("Minimap"),
                    TextFont {
                        font_size: 12.0,
                        ..default()
                    },
                    TextColor(Color::srgb(0.8, 0.8, 0.8)),
                ));
            });

            // Minimap visualization area
            parent.spawn((
                Node {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    margin: UiRect::top(Val::Px(20.0)),
                    ..default()
                },
                BackgroundColor(Color::srgba(0.05, 0.1, 0.05, 1.0)),
                MinimapCamera,
            ));
        });
}

/// Updates the minimap with current game state
pub fn update_minimap(
    _game_map: Res<GameMap>,
    vision_query: Query<&Transform, With<VisionProvider>>,
    minimap_query: Query<&Node, With<MinimapCamera>>,
    mut gizmos: Gizmos,
) {
    // This is a placeholder for more sophisticated minimap rendering
    // In a full implementation, this would:
    // 1. Render terrain tiles as colored pixels
    // 2. Show unit positions as dots
    // 3. Display fog of war
    // 4. Indicate camera viewport bounds
    
    // For now, we just verify the system is being called
    if let Ok(_node) = minimap_query.single() {
        // Draw a simple representation in 3D space for debugging
        for transform in vision_query.iter().take(10) {
            let pos = transform.translation;
            gizmos.circle(
                Isometry3d::new(Vec3::new(pos.x, 25.0, pos.z), Quat::IDENTITY),
                0.3,
                Color::srgb(0.0, 1.0, 0.0),
            );
        }
    }
}

/// Handles clicking on the minimap to move camera
pub fn handle_minimap_click(
    mut interaction_query: Query<(&Interaction, &Node), (Changed<Interaction>, With<Minimap>)>,
    mut camera_query: Query<&mut Transform, With<Camera3d>>,
    game_map: Res<GameMap>,
    primary_window: Query<&Window, With<bevy::window::PrimaryWindow>>,
) {
    for (interaction, _node) in &mut interaction_query {
        if *interaction == Interaction::Pressed {
            // Get mouse position and convert to map coordinates
            if let Ok(window) = primary_window.single() {
                if let Some(cursor_pos) = window.cursor_position() {
                    // Calculate relative position within minimap (0-1 range)
                    // This is simplified - a full implementation would properly map screen space to minimap space
                    let map_size = game_map.width.max(game_map.height) as f32;
                    
                    // Convert to world coordinates
                    let world_x = (cursor_pos.x / 200.0) * map_size - map_size / 2.0;
                    let world_z = (cursor_pos.y / 200.0) * map_size - map_size / 2.0;
                    
                    // Move camera to clicked location
                    if let Ok(mut camera_transform) = camera_query.single_mut() {
                        camera_transform.translation.x = world_x;
                        camera_transform.translation.z = world_z;
                        info!("Camera moved to minimap position: ({}, {})", world_x, world_z);
                    }
                }
            }
        }
    }
}
