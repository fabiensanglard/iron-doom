use bevy::prelude::*;

use engine_core::fixed_point::Fixed;
use engine_core::geometry::{Angle, Line, Sector, Segment, Side, Vertex};
use engine_core::video_system::{VideoSystem, SCREEN_WIDTH};

use super::horizontal_clip::SegmentFragment;
use super::{RenderSubSectorSchedule, RenderSubSectorSet};
use crate::level::level_render::player::three_d::render_flats::VisPlanes;
use crate::level::level_render::player::three_d::{
    GraphicsAssets, RenderSet, ScreenLUT, WallTexture,
};
use crate::level::map_object::{Player, PlayerAngle, Position};

mod portal;
mod solid_wall;

const MAX_SCALE: Fixed = Fixed::lit("64");
const MIN_SCALE: Fixed = Fixed::lit("0.00390625");
const DONT_PEG_DOWN: i16 = 16;
const DONT_PEG_TOP: i16 = 8;

pub struct DrawColumnsPlugin;

impl Plugin for DrawColumnsPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<DrawColumn>()
            .init_resource::<VerticalClip>()
            .add_systems(
                RenderSubSectorSchedule,
                vertical_clip
                    .run_if(on_event::<DrawColumn>())
                    .in_set(RenderSubSectorSet::DrawColumns),
            )
            .add_systems(PostUpdate, clear_clip.in_set(RenderSet::Prepare));
    }
}

fn clear_clip(mut vertical_clip: ResMut<VerticalClip>) {
    vertical_clip.floor = vec![168; SCREEN_WIDTH as usize];
    vertical_clip.ceiling = vec![-1; SCREEN_WIDTH as usize];
}

fn vertical_clip(
    screen_lut: Res<ScreenLUT>,
    mut vertical_clip: ResMut<VerticalClip>,
    mut vertical_clip_segments: EventReader<DrawColumn>,
    sector_query: Query<&Sector>,
    segment_query: Query<&Segment>,
    vertex_query: Query<&Vertex>,
    side_query: Query<&Side>,
    line_query: Query<&Line>,
    player_query: Query<(&Position, &PlayerAngle), With<Player>>,
    textures: Res<GraphicsAssets>,
    mut vis_planes: ResMut<VisPlanes>,
    mut video_sys: NonSendMut<VideoSystem>,
) {
    for draw_event in vertical_clip_segments.read() {
        let segment = segment_query.get(draw_event.segment).unwrap();

        match segment.back_sector {
            None => {
                solid_wall::draw(
                    &screen_lut,
                    &mut vertical_clip,
                    draw_event,
                    &sector_query,
                    segment,
                    &vertex_query,
                    &side_query,
                    &line_query,
                    &player_query,
                    &textures,
                    &mut vis_planes,
                    &mut video_sys,
                );
            }
            Some(_) => {
                portal::draw(
                    &screen_lut,
                    &mut vertical_clip,
                    draw_event,
                    &sector_query,
                    segment,
                    &vertex_query,
                    &side_query,
                    &line_query,
                    &player_query,
                    &textures,
                    &mut vis_planes,
                    &mut video_sys,
                );
            }
        }
    }
}

fn get_column(texture: &WallTexture, col: i32) -> Option<&Vec<u8>> {
    let col_mask = (1 << texture.width.ilog2()) - 1;
    let masked = (col & col_mask) as usize;

    texture.cols.get(&masked)
}

fn scale_from_global_angle(
    player_angle: Angle,
    vis_angle: Angle,
    normal_angle: Angle,
    rw_distance: Fixed,
) -> Fixed {
    let anglea = Angle::DEG_90 + (vis_angle - player_angle);
    let angleb = Angle::DEG_90 + (vis_angle - normal_angle);

    let projection = Fixed::from_num(SCREEN_WIDTH / 2);

    let sina = anglea.sin();
    let sinb = angleb.sin();
    let num = projection * sinb;
    let den = rw_distance * sina;

    if den <= (num >> Fixed::FRAC_NBITS) {
        return MAX_SCALE;
    }
    let scale = num / den;

    scale.clamp(MIN_SCALE, MAX_SCALE)
}

fn draw_col(
    i: u32,
    yl: i32,
    yh: i32,
    mut frac: Fixed,
    dc_iscale: Fixed,
    dc_source: &[u8],
    video_sys: &mut VideoSystem,
) {
    if yh < yl {
        return;
    }
    if yl < 0 || yh < 0 {
        panic!("draw_col: wrong input values for column");
    }

    let yl = yl as u32;
    let yh = yh as u32;

    for j in yl..=yh {
        let frac_idx = ((frac.to_num::<i32>()) & 127) as usize;
        let idx = frac_idx % dc_source.len();
        let color = dc_source[idx];
        video_sys.draw_pixel(i, j, color);
        frac += dc_iscale;
    }
}

#[derive(Event)]
pub struct DrawColumn {
    pub fragments: Vec<SegmentFragment>,
    pub segment: Entity,
    pub start_angle: Angle,
}

#[derive(Resource, Default)]
pub struct VerticalClip {
    floor: Vec<i32>,
    ceiling: Vec<i32>,
}
