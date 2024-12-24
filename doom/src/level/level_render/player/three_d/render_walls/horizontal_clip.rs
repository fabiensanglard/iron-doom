use bevy::prelude::*;

use super::draw_columns::DrawColumn;
use super::{RenderSubSectorSchedule, RenderSubSectorSet};
use crate::level::level_render::player::three_d::RenderSet;
use engine_core::geometry::Angle;
pub use solid_wall_clipping::*;

mod solid_wall_clipping;

pub struct HorizontalClipPlugin;

impl Plugin for HorizontalClipPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<HorizontalSpace>()
            .add_event::<HorizontalClipSegment>()
            .add_event::<HorizontalClipPassSegment>()
            .add_systems(PostUpdate, clean_up.in_set(RenderSet::Prepare))
            .add_systems(
                RenderSubSectorSchedule,
                clip_segments
                    .run_if(on_event::<HorizontalClipSegment>())
                    .in_set(RenderSubSectorSet::HorizontalClip),
            )
            .add_systems(
                RenderSubSectorSchedule,
                clip_pass_segments
                    .run_if(on_event::<HorizontalClipPassSegment>())
                    .in_set(RenderSubSectorSet::HorizontalClip),
            );
    }
}

fn clean_up(mut horizontal_space: ResMut<HorizontalSpace>) {
    horizontal_space.clear();
}

fn clip_segments(
    mut clip_segments: EventReader<HorizontalClipSegment>,
    mut vertical_clip_segments: EventWriter<DrawColumn>,
    mut horizontal_space: ResMut<HorizontalSpace>,
) {
    for clip_event in clip_segments.read() {
        let initial_segment = SegmentFragment::new(clip_event.start_x, clip_event.end_x);
        let fragments = horizontal_space.clip_fragment(initial_segment);
        if !fragments.is_empty() {
            vertical_clip_segments.send(DrawColumn {
                fragments,
                segment: clip_event.segment,
                start_angle: clip_event.start_angle,
            });
        }
    }
}

fn clip_pass_segments(
    mut clip_segments: EventReader<HorizontalClipPassSegment>,
    mut vertical_clip_segments: EventWriter<DrawColumn>,
    mut horizontal_space: ResMut<HorizontalSpace>,
) {
    for clip_event in clip_segments.read() {
        let initial_segment = SegmentFragment::new(clip_event.start_x, clip_event.end_x);
        let fragments = horizontal_space.clip_pass_fragment(initial_segment);
        if !fragments.is_empty() {
            vertical_clip_segments.send(DrawColumn {
                fragments,
                segment: clip_event.segment,
                start_angle: clip_event.start_angle,
            });
        }
    }
}

#[derive(Event)]
pub struct HorizontalClipSegment {
    pub start_x: u32,
    pub end_x: u32,
    pub start_angle: Angle,
    pub segment: Entity,
}

#[derive(Event)]
pub struct HorizontalClipPassSegment {
    pub start_x: u32,
    pub end_x: u32,
    pub start_angle: Angle,
    pub segment: Entity,
}
