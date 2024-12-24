use bevy::prelude::*;

use engine_core::geometry::{Angle, Sector, Segment, Side, Vertex};

use super::horizontal_clip::{HorizontalClipPassSegment, HorizontalClipSegment};
use super::{RenderSubSectorSchedule, RenderSubSectorSet};
use crate::level::level_render::player::three_d::ScreenLUT;
use crate::level::map_object::{Player, PlayerAngle, Position};

// FOV of approximately 90.088°.
const CULLING_FOV: Angle = Angle::new(1074790400);
const HALF_CULLING_FOV: Angle = Angle::new(537395200);

pub struct CullPlugin;

impl Plugin for CullPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<CullSegments>().add_systems(
            RenderSubSectorSchedule,
            cull_segments
                .run_if(on_event::<CullSegments>())
                .in_set(RenderSubSectorSet::Cull),
        );
    }
}

fn cull_segments(
    screen_lut: Res<ScreenLUT>,
    mut cull_segments: EventReader<CullSegments>,
    mut clip_segments: EventWriter<HorizontalClipSegment>,
    mut clip_pass_segments: EventWriter<HorizontalClipPassSegment>,
    segment_query: Query<&Segment>,
    vertex_query: Query<&Vertex>,
    sector_query: Query<&Sector>,
    side_query: Query<&Side>,
    player_query: Query<(&PlayerAngle, &Position), With<Player>>,
) {
    let Some(CullSegments { segments }) = cull_segments.read().last() else {
        return;
    };
    let (player_angle, player_pos) = player_query.single();
    for segment_id in segments {
        let segment = segment_query.get(*segment_id).unwrap();
        let start_vertex = vertex_query.get(segment.v1).unwrap();
        let end_vertex = vertex_query.get(segment.v2).unwrap();
        let start_angle = player_pos.angle(start_vertex.x, start_vertex.y);
        let end_angle = player_pos.angle(end_vertex.x, end_vertex.y);

        let span = start_angle - end_angle;

        // Backface culling
        if span >= Angle::DEG_180 {
            continue;
        }

        let mut cull_start_angle = start_angle - player_angle.0;
        let mut cull_end_angle = end_angle - player_angle.0;

        // Left Culling
        let cull_span = cull_start_angle + HALF_CULLING_FOV;
        if cull_span > CULLING_FOV {
            if span <= cull_span - CULLING_FOV {
                continue;
            }
            cull_start_angle = HALF_CULLING_FOV;
        }
        // Right Culling
        let cull_span = HALF_CULLING_FOV - cull_end_angle;
        if cull_span > CULLING_FOV {
            if span <= cull_span - CULLING_FOV {
                continue;
            }
            cull_end_angle = -HALF_CULLING_FOV;
        }

        let start_x = screen_lut.angle_to_x(cull_start_angle);
        let end_x = screen_lut.angle_to_x(cull_end_angle);
        // Does not cross a pixel?
        if start_x == end_x {
            continue;
        }

        match segment.back_sector {
            None => {
                clip_segments.send(HorizontalClipSegment {
                    start_x,
                    end_x: end_x - 1,
                    start_angle,
                    segment: *segment_id,
                });
            }
            Some(back_sector) => {
                let back_sector = sector_query.get(back_sector).unwrap();
                let front_sector = sector_query.get(segment.front_sector).unwrap();
                let side = side_query.get(segment.side).unwrap();

                if is_closed_door(back_sector, front_sector) {
                    clip_segments.send(HorizontalClipSegment {
                        start_x,
                        end_x: end_x - 1,
                        start_angle,
                        segment: *segment_id,
                    });
                    continue;
                }
                if is_window(back_sector, front_sector)
                    || !is_empty_line(back_sector, front_sector, side)
                {
                    clip_pass_segments.send(HorizontalClipPassSegment {
                        start_x,
                        end_x: end_x - 1,
                        start_angle,
                        segment: *segment_id,
                    });
                }
            }
        }
    }
}

fn is_closed_door(back_sector: &Sector, front_sector: &Sector) -> bool {
    // Case 1
    let lower_ceiling = back_sector.ceiling_height <= front_sector.floor_height;
    // Case 2
    let higher_floor = back_sector.floor_height >= front_sector.ceiling_height;

    lower_ceiling || higher_floor
}

fn is_window(back_sector: &Sector, front_sector: &Sector) -> bool {
    back_sector.ceiling_height != front_sector.ceiling_height
        || back_sector.floor_height != front_sector.floor_height
}

fn is_empty_line(back_sector: &Sector, front_sector: &Sector, side: &Side) -> bool {
    back_sector.ceiling_pic == front_sector.ceiling_pic
        && back_sector.floor_pic == front_sector.floor_pic
        && back_sector.light_level == front_sector.light_level
        && (side.middle_texture == "-" || side.middle_texture == "AASTINKY")
}

#[derive(Event)]
pub struct CullSegments {
    pub segments: Vec<Entity>,
}
