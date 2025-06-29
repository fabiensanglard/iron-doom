use super::MapObject;
use bevy::prelude::*;

mod controls;
pub mod movement;

pub mod prelude {
    pub use super::{controls::PlayerAction, movement::PlayerMovementPlugin, Player};
}

#[derive(Component, Default)]
#[require(MapObject)]
pub struct Player;
