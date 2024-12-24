use bevy::prelude::*;

use engine_core::app::exit_error;
use engine_core::geometry::{Angle, Line, Segment, Side};
use engine_core::wad_system::{Direction, WadSegments, WadSystem};

use crate::level::line_defs::load_line_defs;
use crate::level::load_level::{LoadLevelLump, LoadLevelSet};
use crate::level::side_defs::load_side_defs;
use crate::level::vertexes::load_vertexes;
use crate::level::LevelMap;

const SEGS: usize = 5;
const TWO_SIDED: i16 = 4;

pub struct LoadSegmentsPlugin;

impl Plugin for LoadSegmentsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            FixedPreUpdate,
            (clear_segments, load_segments.pipe(exit_error))
                .chain()
                .in_set(LoadLevelSet::Component)
                .after(load_line_defs)
                .after(load_side_defs)
                .after(load_vertexes),
        );
    }
}

fn clear_segments(mut commands: Commands, mut level_map: ResMut<LevelMap>) {
    super::despawn_all(&mut commands, level_map.segments.iter().cloned());
    level_map.segments.clear();
}

pub(super) fn load_segments(
    mut load_event: EventReader<LoadLevelLump>,
    mut wad_sys: NonSendMut<WadSystem>,
    sides_query: Query<&Side>,
    lines_query: Query<&Line>,
    mut commands: Commands,
    mut level_map: ResMut<LevelMap>,
) -> Result<(), String> {
    let Some(ev) = load_event.read().last() else {
        return Ok(());
    };
    let level_lump_idx = ev.lump_idx;
    let segments_lump_idx = level_lump_idx + SEGS;

    let lump_data = wad_sys.cache_lump_idx(segments_lump_idx)?;
    let map_segments: WadSegments = lump_data.as_slice().try_into()?;

    for map_seg in map_segments {
        let angle = Angle::new(map_seg.angle as u32) << 16;
        let line_id = find_line(&level_map, map_seg.line_def)?;
        let line = lines_query.get(line_id).unwrap();

        let side_id = match map_seg.direction {
            Direction::Front => line.front_side_def,
            Direction::Back => line.back_side_def.unwrap(),
        };
        let side = sides_query.get(side_id).unwrap();

        let front_sector = side.sector;
        let back_sector = if line.flags & TWO_SIDED > 0 {
            let side_id = match map_seg.direction {
                Direction::Front => line.back_side_def.unwrap(),
                Direction::Back => line.front_side_def,
            };
            let side = sides_query.get(side_id).unwrap();

            Some(side.sector)
        } else {
            None
        };

        let segment_id = commands
            .spawn(Segment {
                v1: find_vertex(&level_map, map_seg.start_vertex)?,
                v2: find_vertex(&level_map, map_seg.end_vertex)?,
                offset: map_seg.offset.into(),
                angle,
                side: side_id,
                line: line_id,
                front_sector,
                back_sector,
            })
            .id();

        level_map.segments.push(segment_id);
    }

    Ok(())
}

fn find_vertex(level_map: &LevelMap, vertex_idx: u16) -> Result<Entity, String> {
    if let Some(vertex_id) = level_map.vertexes.get(vertex_idx as usize) {
        return Ok(*vertex_id);
    }
    Err(format!(
        "find_vertex: could not find vertex with id {vertex_idx}"
    ))
}

fn find_line(level_map: &LevelMap, line_idx: u16) -> Result<Entity, String> {
    if let Some(line_id) = level_map.lines.get(line_idx as usize) {
        return Ok(*line_id);
    }
    Err(format!("find_line: could not find line with id {line_idx}"))
}
