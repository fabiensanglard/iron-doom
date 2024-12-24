use bevy::prelude::*;

use engine_core::app::exit_error;
use engine_core::wad_system::{WadRejectTable, WadSystem};

use crate::level::load_level::{LoadLevelLump, LoadLevelSet};
use crate::level::sectors::load_sectors;
use crate::level::LevelMap;

const REJECT: usize = 9;

pub struct LoadRejectTablePlugin;

impl Plugin for LoadRejectTablePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            FixedPreUpdate,
            load_reject_table
                .pipe(exit_error)
                .in_set(LoadLevelSet::Component)
                .after(load_sectors),
        );
    }
}

pub(super) fn load_reject_table(
    mut wad_sys: NonSendMut<WadSystem>,
    mut load_event: EventReader<LoadLevelLump>,
    mut level_map: ResMut<LevelMap>,
) -> Result<(), String> {
    let Some(ev) = load_event.read().last() else {
        return Ok(());
    };
    let level_lump_idx = ev.lump_idx;
    let reject_table_lump_idx = level_lump_idx + REJECT;

    let lump_data = wad_sys.cache_lump_idx(reject_table_lump_idx)?;
    let num_sectors = level_map.sectors.len();

    level_map.reject_table = WadRejectTable::new(&lump_data, num_sectors);

    Ok(())
}
