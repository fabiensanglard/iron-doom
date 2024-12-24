use std::cmp;

use bevy::prelude::*;

use engine_core::fixed_point::Fixed;
use engine_core::geometry::{Angle, Line, Sector, Segment, Side, Vertex};
use engine_core::video_system::VideoSystem;

use super::DrawColumn;
use super::*;
use crate::level::level_render::player::three_d::{GraphicsAssets, ScreenLUT};
use crate::level::level_render::player::three_d::render_flats::VisPlanes;
use crate::level::map_object::{Player, PlayerAngle, Position};

pub fn draw(
    screen_lut: &ScreenLUT,
    vertical_clip: &mut VerticalClip,
    draw_column: &DrawColumn,
    sector_query: &Query<&Sector>,
    segment: &Segment,
    vertex_query: &Query<&Vertex>,
    side_query: &Query<&Side>,
    line_query: &Query<&Line>,
    player_query: &Query<(&Position, &PlayerAngle), With<Player>>,
    textures: &GraphicsAssets,
    vis_planes: &mut VisPlanes,
    video_sys: &mut VideoSystem,
) {
    let height: u32 = 168;
    let center_y: u32 = height / 2;
    let fixed_center_y = Fixed::from_num(center_y);

    let (player_pos, player_angle) = player_query.single();
    let player_angle = player_angle.0;

    let v1 = vertex_query.get(segment.v1).unwrap();
    let front_sector = sector_query.get(segment.front_sector).unwrap();
    let side = side_query.get(segment.side).unwrap();
    let line = line_query.get(segment.line).unwrap();
    let start_angle = draw_column.start_angle;

    let normal_angle = segment.angle + Angle::DEG_90;
    let offset_angle = cmp::min(normal_angle - start_angle, start_angle - normal_angle);
    let offset_angle = cmp::min(offset_angle, Angle::DEG_90);

    let dist_angle = Angle::DEG_90 - offset_angle;
    let hyp = player_pos.distance(v1.x, v1.y);
    let sine_val = dist_angle.sin();
    let distance = hyp * sine_val;

    for fragment in &draw_column.fragments {
        let x1 = fragment.start();
        let x2 = fragment.end();

        let ang1 = screen_lut.x_to_angle(x1);
        let mut scale =
            scale_from_global_angle(player_angle, player_angle + ang1, normal_angle, distance);
        let scale_step = if x1 < x2 {
            let ang2 = screen_lut.x_to_angle(x2);
            let scale2 =
                scale_from_global_angle(player_angle, player_angle + ang2, normal_angle, distance);
            (scale2 - scale) / Fixed::from_num(x2 - x1)
        } else {
            Fixed::ZERO
        };

        let mut world_top = front_sector.ceiling_height.wrapping_sub(player_pos.z);
        let mut world_bottom = front_sector.floor_height.wrapping_sub(player_pos.z);

        let mid_texture = textures.wall_texture(&side.middle_texture).unwrap();
        let mut mid_texture_mid = if line.flags & DONT_PEG_DOWN != 0 {
            let vtop = front_sector.floor_height + Fixed::from_num(mid_texture.height);
            vtop - player_pos.z
        } else {
            world_top
        };
        mid_texture_mid += side.y_offset;

        // calculate rw_offset (only needed for textured lines)
        let mut offset_angle = normal_angle - start_angle;

        if offset_angle > Angle::DEG_180 {
            offset_angle = -offset_angle;
        }
        if offset_angle > Angle::DEG_90 {
            offset_angle = Angle::DEG_90;
        }

        let sine_val = offset_angle.sin();
        let mut offset = hyp * sine_val;

        if normal_angle - start_angle < Angle::DEG_180 {
            offset = -offset;
        }

        offset += side.x_offset + segment.offset;
        let center_angle = player_angle - normal_angle;

        // calculate incremental stepping values for texture edges
        world_top >>= 4;
        world_bottom >>= 4;

        let top_step = -(scale_step * world_top);
        let mut top_frac = (fixed_center_y >> 4) - (world_top * scale);

        let bottom_step = -(scale_step * world_bottom);
        let mut bottom_frac = (fixed_center_y >> 4) - (world_bottom * scale);

        let mut mark_floor = true;
        let mut mark_ceiling = true;

        // if a floor / ceiling plane is on the wrong side
        //  of the view plane, it is definitely invisible
        //  and doesn't need to be marked.
        if front_sector.floor_height >= player_pos.z {
            // above view plane
            mark_floor = false;
        }
        if front_sector.ceiling_height <= player_pos.z && front_sector.ceiling_pic != "F_SKY1" {
            // below view plane
            mark_ceiling = false;
        }

        if mark_floor {
            vis_planes.check_floor(x1, x2);
        }
        if mark_ceiling {
            vis_planes.check_ceiling(x1, x2);
        }

        for i in x1..=x2 {
            let height_bits = 12;
            let height_unit = Fixed::from_bits(1 << height_bits);

            let yl: Fixed = (top_frac + height_unit - Fixed::DELTA) >> height_bits;
            let mut yl = yl.to_bits();
            if yl < vertical_clip.ceiling[i as usize] + 1 {
                yl = vertical_clip.ceiling[i as usize] + 1;
            }

            let yh: Fixed = bottom_frac >> height_bits;
            let mut yh = yh.to_bits();
            if yh >= vertical_clip.floor[i as usize] {
                yh = vertical_clip.floor[i as usize] - 1;
            }

            if mark_ceiling {
                let top = vertical_clip.ceiling[i as usize] + 1;
                let mut bottom = yl - 1;

                if bottom >= vertical_clip.floor[i as usize] {
                    bottom = vertical_clip.floor[i as usize] - 1;
                }
                if top <= bottom {
                    vis_planes.ceiling().top[i as usize] = top as u8;
                    vis_planes.ceiling().bottom[i as usize] = bottom as u8;
                }
            }
            if mark_floor {
                let mut top = yh + 1;
                let bottom = vertical_clip.floor[i as usize] - 1;

                if top <= vertical_clip.ceiling[i as usize] {
                    top = vertical_clip.ceiling[i as usize] + 1;
                }
                if top <= bottom {
                    vis_planes.floor().top[i as usize] = top as u8;
                    vis_planes.floor().bottom[i as usize] = bottom as u8;
                }
            }

            let angle = center_angle + screen_lut.x_to_angle(i);
            let texture_column = offset - (angle.tan() * distance);
            let texture_column = texture_column.to_num::<i32>();

            // Determine scaling,
            //  which is the only mapping to be done.
            let Some(dc_source) = get_column(mid_texture, texture_column) else {
                continue;
            };
            let bits = u32::MAX.wrapping_div(scale.to_bits() as u32) as i32;
            let dc_iscale = Fixed::from_bits(bits);
            let frac = mid_texture_mid + (yl - (center_y as i32)) * dc_iscale;

            draw_col(i, yl, yh, frac, dc_iscale, dc_source, video_sys);

            vertical_clip.ceiling[i as usize] = 168;
            vertical_clip.floor[i as usize] = -1;

            scale += scale_step;
            top_frac += top_step;
            bottom_frac += bottom_step;
        }
    }
}
