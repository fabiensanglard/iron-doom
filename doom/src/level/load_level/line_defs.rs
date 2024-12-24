use bevy::prelude::*;

use engine_core::app::exit_error;
use engine_core::geometry::{Line, Side, SlopeType, Vertex};
use engine_core::wad_system::{WadLineDefs, WadSystem};

use crate::level::load_level::{LoadLevelLump, LoadLevelSet};
use crate::level::side_defs::load_side_defs;
use crate::level::vertexes::load_vertexes;
use crate::level::LevelMap;

const LINE_DEFS: usize = 2;

pub struct LoadLinesPlugin;

impl Plugin for LoadLinesPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            FixedPreUpdate,
            (clear_level_lines, load_line_defs.pipe(exit_error))
                .chain()
                .in_set(LoadLevelSet::Component)
                .after(load_side_defs)
                .after(load_vertexes),
        );
    }
}

fn clear_level_lines(mut commands: Commands, mut level_map: ResMut<LevelMap>) {
    super::despawn_all(&mut commands, level_map.lines.iter().cloned());
    level_map.lines.clear();
}

pub(super) fn load_line_defs(
    mut load_event: EventReader<LoadLevelLump>,
    mut wad_sys: NonSendMut<WadSystem>,
    vertex_query: Query<&Vertex>,
    side_query: Query<&Side>,
    mut commands: Commands,
    mut level_map: ResMut<LevelMap>,
) -> Result<(), String> {
    let Some(ev) = load_event.read().last() else {
        return Ok(());
    };
    let level_lump_idx = ev.lump_idx;
    let line_defs_lump_idx = level_lump_idx + LINE_DEFS;
    let lump = wad_sys.cache_lump_idx(line_defs_lump_idx)?;

    let map_line_defs: WadLineDefs = lump.as_slice().try_into()?;

    for map_line in map_line_defs {
        let v1_id = find_vertex(&level_map, map_line.start_vertex)?;
        let v1 = vertex_query.get(v1_id).unwrap();

        let v2_id = find_vertex(&level_map, map_line.end_vertex)?;
        let v2 = vertex_query.get(v2_id).unwrap();

        let dx = v2.x - v1.x;
        let dy = v2.y - v1.y;

        let line_id = commands
            .spawn(Line {
                v1: v1_id,
                v2: v2_id,
                dx,
                dy,
                flags: map_line.flags,
                special: map_line.special,
                sector_tag: map_line.sector_tag,
                front_side_def: find_side(&level_map, map_line.front_side_def)?.unwrap(),
                back_side_def: find_side(&level_map, map_line.back_side_def)?,
                bounding_box: (*v1, *v2).into(),
                slope_type: SlopeType::from_fixed(dy, dx),
                front_sector: find_sector(&level_map, &side_query, map_line.front_side_def)?,
                back_sector: find_sector(&level_map, &side_query, map_line.back_side_def)?,
                valid_count: 0,
            })
            .id();

        level_map.lines.push(line_id);
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

fn find_side(level_map: &LevelMap, side_idx: u16) -> Result<Option<Entity>, String> {
    if side_idx == u16::MAX {
        return Ok(None);
    }
    if let Some(side_id) = level_map.sides.get(side_idx as usize) {
        return Ok(Some(*side_id));
    }

    Err(format!("find_side: could not find side with id {side_idx}"))
}

fn find_sector(
    level_map: &LevelMap,
    side_query: &Query<&Side>,
    side_idx: u16,
) -> Result<Option<Entity>, String> {
    if let Some(side_id) = find_side(level_map, side_idx)? {
        let side = side_query.get(side_id).unwrap();
        return Ok(Some(side.sector));
    }

    Ok(None)
}
