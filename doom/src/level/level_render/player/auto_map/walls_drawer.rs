use bevy::prelude::{Query, Res, ResMut};

use engine_core::geometry::{Line, Sector, Vertex};

use crate::level::LevelMap;

use super::colors::{BROWN, RED, RED_RANGE, YELLOW};
use super::map_geometry::MapLine;
use super::{line_drawer, AutoMap};

const NEVER_SEE: i16 = 128;
const SECRET: i16 = 32;
const WALLS_COLOR: u8 = RED;
const WALLS_RANGE: u8 = RED_RANGE;
const FD_WALLS_COLOR: u8 = BROWN;
const CD_WALLS_COLOR: u8 = YELLOW;

pub fn draw(
    mut auto_map: ResMut<AutoMap>,
    level_map: Res<LevelMap>,
    line_query: Query<&Line>,
    sector_query: Query<&Sector>,
    vertex_query: Query<&Vertex>,
) {
    for l in level_map.lines() {
        let l = line_query.get(*l).unwrap();
        if l.flags & NEVER_SEE != 0 {
            continue;
        }

        let v1 = vertex_query.get(l.v1).unwrap();
        let v2 = vertex_query.get(l.v2).unwrap();
        let ml: MapLine = (*v1, *v2).into();

        if let Some(back) = l.back_sector {
            if l.special == 39 {
                line_drawer::draw_mline(&mut auto_map, ml, WALLS_COLOR + WALLS_RANGE / 2);
                continue;
            }
            if l.flags & SECRET != 0 {
                line_drawer::draw_mline(&mut auto_map, ml, WALLS_COLOR);
                continue;
            }
            if let Some(front) = l.front_sector {
                let back = sector_query.get(back).unwrap();
                let front = sector_query.get(front).unwrap();

                if back.floor_height != front.floor_height {
                    line_drawer::draw_mline(&mut auto_map, ml, FD_WALLS_COLOR);
                } else if back.ceiling_height != front.ceiling_height {
                    line_drawer::draw_mline(&mut auto_map, ml, CD_WALLS_COLOR);
                }
            }
        } else {
            line_drawer::draw_mline(&mut auto_map, ml, WALLS_COLOR);
        }
    }
}
