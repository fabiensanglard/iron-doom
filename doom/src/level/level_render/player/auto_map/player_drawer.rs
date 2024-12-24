use bevy::prelude::{Query, ResMut, With};

use engine_core::fixed_point::Fixed;
use engine_core::geometry::Angle;

use crate::level::map_object::{Player, PlayerAngle, Position};

use super::colors::WHITE;
use super::map_geometry::{MapLine, MapPoint};
use super::{line_drawer, AutoMap, PLAYER_RADIUS};

pub fn draw(mut auto_map: ResMut<AutoMap>, query: Query<(&PlayerAngle, &Position), With<Player>>) {
    let (angle, pos) = query.single();
    let player_arrow = player_arrow();
    let scale = Fixed::ONE;

    draw_line_character(
        &mut auto_map,
        &player_arrow,
        scale,
        angle.into(),
        WHITE,
        *pos,
    );
}

fn draw_line_character(
    auto_map: &mut AutoMap,
    player_arrow: &[MapLine],
    scale: Fixed,
    angle: Angle,
    color: u8,
    pos: Position,
) {
    let create_point = |mut point: MapPoint| {
        point.scale(scale);
        if angle != Angle::ZERO {
            point.rotate(angle);
        }
        point.add(pos.x, pos.y);

        point
    };

    for line in player_arrow {
        let m_line = MapLine {
            a: create_point(line.a),
            b: create_point(line.b),
        };

        line_drawer::draw_mline(auto_map, m_line, color);
    }
}

fn player_arrow() -> [MapLine; 7] {
    let r: Fixed = 8 * PLAYER_RADIUS / 7;

    [
        MapLine::new(-r + r / 8, Fixed::ZERO, r, Fixed::ZERO), // -----
        MapLine::new(r, Fixed::ZERO, r - r / 2, r / 4),        // ----->
        MapLine::new(r, Fixed::ZERO, r - r / 2, -r / 4),
        MapLine::new(-r + r / 8, Fixed::ZERO, -r - r / 8, r / 4), // >---->
        MapLine::new(-r + r / 8, Fixed::ZERO, -r - r / 8, -r / 4),
        MapLine::new(-r + 3 * r / 8, Fixed::ZERO, -r + r / 8, r / 4), // >>--->
        MapLine::new(-r + 3 * r / 8, Fixed::ZERO, -r + r / 8, -r / 4),
    ]
}
