use bevy::prelude::*;
use serde::{Deserialize, Serialize};

pub mod models;

#[derive(
    Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize, Default, Component, Reflect,
)]
#[reflect(Component)]
pub enum Cult {
    #[default]
    Crimson,
    Deep,
    Void,
}
