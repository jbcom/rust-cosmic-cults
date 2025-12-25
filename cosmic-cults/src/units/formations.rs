use crate::units::{FormationType};
use bevy::prelude::*;

// Calculate formation offset based on formation type
pub fn calculate_formation_offset(
    formation_type: &FormationType,
    index: usize,
    total_units: usize,
    spacing: f32,
) -> Vec2 {
    match formation_type {
        FormationType::Line => {
            // Horizontal line formation
            let center_offset = (total_units - 1) as f32 * spacing / 2.0;
            Vec2::new(index as f32 * spacing - center_offset, 0.0)
        }
        FormationType::Box => {
            // Box/grid formation
            let units_per_row = (total_units as f32).sqrt().ceil() as usize;
            let row = index / units_per_row;
            let col = index % units_per_row;

            let center_offset_x = (units_per_row - 1) as f32 * spacing / 2.0;
            let x = col as f32 * spacing - center_offset_x;
            let y = row as f32 * spacing;

            Vec2::new(x, y)
        }
        FormationType::Wedge => {
            // V-shaped wedge formation
            let row = ((index as f32 * 2.0).sqrt()).floor() as usize;
            let pos_in_row = index - (row * (row + 1)) / 2;

            let x = if row == 0 {
                0.0
            } else {
                (pos_in_row as f32 - row as f32 / 2.0) * spacing
            };
            let y = row as f32 * spacing;

            Vec2::new(x, y)
        }
        FormationType::Circle => {
            // Circular formation
            if total_units == 1 {
                Vec2::ZERO
            } else {
                let angle = (index as f32 / total_units as f32) * 2.0 * std::f32::consts::PI;
                let radius = spacing * 2.0; // Adjust radius based on spacing
                Vec2::new(angle.cos() * radius, angle.sin() * radius)
            }
        }
    }
}
