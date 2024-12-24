use bevy::prelude::ResMut;

use super::colors::GRAY;
use super::{AutoMap, AM_HEIGHT, AM_WIDTH};

const X_HAIR_COLOR: u8 = GRAY;

pub fn draw(mut auto_map: ResMut<AutoMap>) {
    let x = AM_WIDTH / 2;
    let y = AM_HEIGHT / 2;
    auto_map.buf.draw_pixel(x, y, X_HAIR_COLOR);
}
