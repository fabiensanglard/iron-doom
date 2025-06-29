use bevy::prelude::*;
use player::movement::STOP_SPEED;

mod camera;
mod player;

pub mod prelude {
    pub use super::{camera::Camera, player::prelude::*, MapObject};
}

/// Represents an object that can be placed on the map during map loading.
/// Map objects include entities such as players, monsters, items, obstacles,
/// decorations, teleport destinations, boss shooter destinations, and any other
/// entities that should exist on the map.
#[derive(Component, Copy, Clone, Debug)]
pub struct MapObject {
    #[allow(unused)]
    pub pos: Vec2,
    #[allow(unused)]
    pub velocity: Vec2,
    #[allow(unused)]
    pub dir: Dir2,
    #[allow(unused)]
    pub thing_type: i16,
    #[allow(unused)]
    pub options: i16,
}

impl MapObject {
    pub fn is_speed_low(&self) -> bool {
        self.velocity.abs().cmplt(STOP_SPEED).all()
    }
}

impl Default for MapObject {
    fn default() -> Self {
        Self {
            pos: Vec2::default(),
            velocity: Vec2::default(),
            dir: Dir2::X,
            thing_type: i16::default(),
            options: i16::default(),
        }
    }
}
