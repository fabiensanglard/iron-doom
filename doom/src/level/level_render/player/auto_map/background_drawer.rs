use bevy::prelude::ResMut;

use super::colors::BLACK;
use super::AutoMap;

const BACKGROUND_COLOR: u8 = BLACK;

pub fn draw(mut auto_map: ResMut<AutoMap>) {
    auto_map.buf.clear(BACKGROUND_COLOR);
}
