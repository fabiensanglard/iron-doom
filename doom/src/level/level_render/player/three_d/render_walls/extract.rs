use bevy::prelude::*;

use crate::level::level_render::player::three_d::render_flats::VisPlanes;
use crate::level::LevelMap;
use engine_core::geometry::{Sector, SubSector};

use super::cull::CullSegments;
use super::{RenderSubSectorSchedule, RenderSubSectorSet};

pub struct ExtractPlugin;

impl Plugin for ExtractPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<ExtractSegments>().add_systems(
            RenderSubSectorSchedule,
            (update_visplanes, extract_segments)
                .in_set(RenderSubSectorSet::Extract)
                .run_if(on_event::<ExtractSegments>()),
        );
    }
}

fn update_visplanes(
    mut vis_planes: ResMut<VisPlanes>,
    sector_query: Query<&Sector>,
    sub_sector_query: Query<&SubSector>,
    mut extract_segments: EventReader<ExtractSegments>,
) {
    let Some(event) = extract_segments.read().last() else {
        return;
    };
    let sub_sector = sub_sector_query.get(event.sub_sector).unwrap();
    let sector = sector_query.get(sub_sector.sector).unwrap();

    vis_planes.find_floor(sector.floor_height, &sector.floor_pic);
    vis_planes.find_ceiling(sector.ceiling_height, &sector.ceiling_pic);
}

fn extract_segments(
    level_map: Res<LevelMap>,
    sub_sector_query: Query<&SubSector>,
    mut extract_segments: EventReader<ExtractSegments>,
    mut cull_segments: EventWriter<CullSegments>,
) {
    let Some(event) = extract_segments.read().last() else {
        return;
    };
    let sub_sector = sub_sector_query.get(event.sub_sector).unwrap();
    let segments: Vec<Entity> = level_map
        .segments()
        .iter()
        .skip(sub_sector.first_line as usize)
        .take(sub_sector.num_lines as usize)
        .cloned()
        .collect();

    cull_segments.send(CullSegments { segments });
}

#[derive(Event)]
pub struct ExtractSegments {
    pub sub_sector: Entity,
}
