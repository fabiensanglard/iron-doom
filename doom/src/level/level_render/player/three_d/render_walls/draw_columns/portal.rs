use std::cmp;

use bevy::prelude::*;

use engine_core::fixed_point::Fixed;
use engine_core::geometry::{Angle, Line, Sector, Segment, Side, Vertex};
use engine_core::video_system::VideoSystem;

use super::DrawColumn;
use super::*;
use crate::level::level_render::player::three_d::{GraphicsAssets, ScreenLUT};
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
    let back_sector = sector_query.get(segment.back_sector.unwrap()).unwrap();
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

        //----------------------------R_CalculatePortalTexture--------------------------------------
        let mut worldhigh = back_sector.ceiling_height.wrapping_sub(player_pos.z);
        let mut worldlow = back_sector.floor_height.wrapping_sub(player_pos.z);

        let mut mark_floor = false;
        let mut mark_ceiling = false;

        // hack to allow height changes in outdoor areas
        if front_sector.ceiling_pic == "F_SKY1" && back_sector.ceiling_pic == "F_SKY1" {
            world_top = worldhigh;
        }

        if worldlow != world_bottom
            || back_sector.floor_pic != front_sector.floor_pic
            || back_sector.light_level != front_sector.light_level
        {
            mark_floor = true;
        } else {
            // same plane on both sides
            mark_floor = false;
        }
        if worldhigh != world_top
            || back_sector.ceiling_pic != front_sector.ceiling_pic
            || back_sector.light_level != front_sector.light_level
        {
            mark_ceiling = true;
        } else {
            // same plane on both sides
            mark_ceiling = false;
        }
        if back_sector.ceiling_height <= front_sector.floor_height
            || back_sector.floor_height >= front_sector.ceiling_height
        {
            // closed door
            mark_ceiling = true;
            mark_floor = true;
        }

        let mut toptexture = None;
        let mut bottomtexture = None;

        let mut rw_toptexturemid = Fixed::ZERO;
        let mut rw_bottomtexturemid = Fixed::ZERO;

        if worldhigh < world_top {
            // top texture
            let texture = textures.wall_texture(&side.top_texture).unwrap();
            if line.flags & DONT_PEG_TOP != 0 {
                // top of texture at top
                rw_toptexturemid = world_top;
            } else {
                // bottom of texture
                let vtop = back_sector.ceiling_height + Fixed::from_num(texture.height);
                rw_toptexturemid = vtop - player_pos.z;
            }
            toptexture = Some(texture);
        }
        if worldlow > world_bottom {
            // bottom texture
            bottomtexture = textures.wall_texture(&side.bottom_texture).ok();
            if line.flags & DONT_PEG_DOWN != 0 {
                // bottom of texture at bottom
                // top of texture at top
                rw_bottomtexturemid = world_top;
            } else {
                // top of texture at top
                rw_bottomtexturemid = worldlow;
            }
        }

        rw_toptexturemid += side.y_offset;
        rw_bottomtexturemid += side.y_offset;
        //----------------------------R_CalculatePortalTexture--------------------------------------

        //if (segtextured) {

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

        //}

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

        // calculate incremental stepping values for texture edges
        world_top >>= 4;
        world_bottom >>= 4;

        let top_step = -(scale_step * world_top);
        let mut top_frac = (fixed_center_y >> 4) - (world_top * scale);

        let bottom_step = -(scale_step * world_bottom);
        let mut bottom_frac = (fixed_center_y >> 4) - (world_bottom * scale);

        worldhigh >>= 4;
        worldlow >>= 4;

        let mut pixhigh = Fixed::ZERO;
        let mut pixhighstep = Fixed::ZERO;
        let mut pixlow = Fixed::ZERO;
        let mut pixlowstep = Fixed::ZERO;

        if worldhigh < world_top {
            pixhigh = (fixed_center_y >> 4) - (worldhigh * scale);
            pixhighstep = -(scale_step * worldhigh);
        }
        if worldlow > world_bottom {
            pixlow = (fixed_center_y >> 4) - (worldlow * scale);
            pixlowstep = -(scale_step * worldlow);
        }

        for i in x1..=x2 {
            let height_bits = 12;
            let height_unit = Fixed::from_bits(1 << height_bits);

            let yl: Fixed = (top_frac + height_unit - Fixed::DELTA) >> height_bits;
            let mut yl = yl.to_bits();
            if yl < vertical_clip.ceiling[i as usize] + 1 {
                let temp = vertical_clip.ceiling[i as usize] + 1;
                yl = temp;
            }

            let yh: Fixed = bottom_frac >> height_bits;
            let mut yh = yh.to_bits();
            if yh >= vertical_clip.floor[i as usize] {
                let temp = vertical_clip.floor[i as usize] - 1;
                yh = temp;
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


            //---------------------------------R_RenderPortal---------------------------------------
            // two sided line
            if let Some(wall_top_tex) = toptexture {
                // top wall
                let mid: Fixed = pixhigh >> 12;
                let mut mid = mid.to_bits();
                pixhigh += pixhighstep;

                if mid >= vertical_clip.floor[i as usize] {
                    mid = vertical_clip.floor[i as usize] - 1;
                }
                if mid >= yl {
                    let Some(dc_source) = get_column(wall_top_tex, texture_column) else {
                        continue;
                    };
                    let bits = u32::MAX.wrapping_div(scale.to_bits() as u32) as i32;
                    let dc_iscale = Fixed::from_bits(bits);
                    let frac = rw_toptexturemid + (yl - (center_y as i32)) * dc_iscale;

                    draw_col(i, yl, mid, frac, dc_iscale, dc_source, video_sys);

                    vertical_clip.ceiling[i as usize] = mid;
                } else {
                    vertical_clip.ceiling[i as usize] = yl - 1;
                }
            } else if mark_ceiling {
                // no top wall
                vertical_clip.ceiling[i as usize] = yl - 1;
            }

            if let Some(wall_bottom_tex) = bottomtexture {
                // bottom wall
                let height_unit = Fixed::from_bits(1 << height_bits);
                let mid: Fixed = (pixlow + height_unit - Fixed::DELTA) >> 12;
                let mut mid = mid.to_bits();
                pixlow += pixlowstep;

                // no space above wall?
                if mid <= vertical_clip.ceiling[i as usize] {
                    mid = vertical_clip.ceiling[i as usize] + 1;
                }
                if mid <= yh {
                    let Some(dc_source) = get_column(wall_bottom_tex, texture_column) else {
                        continue;
                    };
                    let bits = u32::MAX.wrapping_div(scale.to_bits() as u32) as i32;
                    let dc_iscale = Fixed::from_bits(bits);
                    let mut frac = rw_bottomtexturemid + (mid - (center_y as i32)) * dc_iscale;

                    draw_col(i, mid, yh, frac, dc_iscale, dc_source, video_sys);

                    vertical_clip.floor[i as usize] = mid;
                } else {
                    vertical_clip.floor[i as usize] = yh + 1;
                }
            } else if mark_floor {
                // no bottom wall
                vertical_clip.floor[i as usize] = yh + 1;
            }
            //---------------------------------R_RenderPortal---------------------------------------

            scale += scale_step;
            top_frac += top_step;
            bottom_frac += bottom_step;
        }
    }
}
