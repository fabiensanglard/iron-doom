use bevy::prelude::*;

use engine_core::app::exit_error;
use engine_core::geometry::{Segment, Side, SubSector};
use engine_core::wad_system::{WadSubSectors, WadSystem};

use crate::level::load_level::{LoadLevelLump, LoadLevelSet};
use crate::level::segments::load_segments;
use crate::level::LevelMap;

const SSECTORS: usize = 6;

pub struct LoadSubSectorsPlugin;

impl Plugin for LoadSubSectorsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            FixedPreUpdate,
            (clear_sub_sectors, load_sub_sectors.pipe(exit_error))
                .chain()
                .in_set(LoadLevelSet::Component)
                .after(load_segments),
        );
    }
}

fn clear_sub_sectors(mut commands: Commands, mut level_map: ResMut<LevelMap>) {
    super::despawn_all(&mut commands, level_map.sub_sectors.iter().cloned());
    level_map.sub_sectors.clear();
}

pub(super) fn load_sub_sectors(
    mut load_event: EventReader<LoadLevelLump>,
    mut wad_sys: NonSendMut<WadSystem>,
    segment_query: Query<&Segment>,
    side_query: Query<&Side>,
    mut commands: Commands,
    mut level_map: ResMut<LevelMap>,
) -> Result<(), String> {
    let Some(ev) = load_event.read().last() else {
        return Ok(());
    };
    let level_lump_idx = ev.lump_idx;
    let sub_sectors_lump_idx = level_lump_idx + SSECTORS;

    let lump_data = wad_sys.cache_lump_idx(sub_sectors_lump_idx)?;
    let map_sub_sectors: WadSubSectors = lump_data.as_slice().try_into()?;

    for map_sub_sec in map_sub_sectors {
        let sector_id = find_sector(
            &level_map,
            &segment_query,
            &side_query,
            map_sub_sec.first_line,
        )?;

        let sub_sector_id = commands
            .spawn(SubSector {
                sector: sector_id,
                num_lines: map_sub_sec.num_segs,
                first_line: map_sub_sec.first_line,
            })
            .id();

        level_map.sub_sectors.push(sub_sector_id);
    }

    Ok(())
}

fn find_sector(
    level_map: &LevelMap,
    segment_query: &Query<&Segment>,
    side_query: &Query<&Side>,
    segment_idx: u16,
) -> Result<Entity, String> {
    if let Some(segment_id) = level_map.segments.get(segment_idx as usize) {
        let segment = segment_query.get(*segment_id).unwrap();
        let side = side_query.get(segment.side).unwrap();
        return Ok(side.sector);
    }
    Err(format!(
        "find_segment: could not find segment with id {segment_idx}"
    ))
}
