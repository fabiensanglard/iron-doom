use bevy::prelude::*;

use engine_core::app::exit_error;
use engine_core::fixed_point::Fixed;
use engine_core::geometry::Sector;
use engine_core::wad_system::{WadSectors, WadSystem};

use crate::level::load_level::{LoadLevelLump, LoadLevelSet};
use crate::level::LevelMap;

const SECTORS: usize = 8;

pub struct LoadSectorsPlugin;

impl Plugin for LoadSectorsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            FixedPreUpdate,
            (clear_sectors, load_sectors.pipe(exit_error))
                .chain()
                .in_set(LoadLevelSet::Component),
        );
    }
}

fn clear_sectors(mut commands: Commands, mut level_map: ResMut<LevelMap>) {
    super::despawn_all(&mut commands, level_map.sectors.iter().cloned());
    level_map.sectors.clear();
}

pub(super) fn load_sectors(
    mut load_event: EventReader<LoadLevelLump>,
    mut wad_sys: NonSendMut<WadSystem>,
    mut commands: Commands,
    mut level_map: ResMut<LevelMap>,
) -> Result<(), String> {
    let Some(ev) = load_event.read().last() else {
        return Ok(());
    };
    let level_lump_idx = ev.lump_idx;
    let sectors_lump_idx = level_lump_idx + SECTORS;

    let lump_data = wad_sys.cache_lump_idx(sectors_lump_idx)?;
    let map_sectors: WadSectors = lump_data.as_slice().try_into()?;

    for map_sec in map_sectors {
        let sector_id = commands
            .spawn(Sector {
                floor_height: Fixed::from(map_sec.floor_height),
                ceiling_height: Fixed::from(map_sec.ceiling_height),
                floor_pic: map_sec.floor_pic,
                ceiling_pic: map_sec.ceiling_pic,
                light_level: map_sec.light_level,
                special: map_sec.special,
                tag: map_sec.tag,
            })
            .id();

        level_map.sectors.push(sector_id);
    }

    Ok(())
}
