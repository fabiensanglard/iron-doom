use bevy::prelude::*;

use engine_core::app::exit_error;
use engine_core::wad_system::WadSystem;

use crate::level::load_level::{LoadLevelLump, LoadLevelSet};
use crate::level::LevelMap;

const BLOCK_MAP: usize = 10;

pub struct LoadBlockMapPlugin;

impl Plugin for LoadBlockMapPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            FixedPreUpdate,
            load_block_map
                .pipe(exit_error)
                .in_set(LoadLevelSet::Component),
        );
    }
}

pub(super) fn load_block_map(
    wad_sys: NonSend<WadSystem>,
    mut load_event: EventReader<LoadLevelLump>,
    mut level_map: ResMut<LevelMap>,
) -> Result<(), String> {
    let Some(ev) = load_event.read().last() else {
        return Ok(());
    };
    let level_lump_idx = ev.lump_idx;
    let block_map_lump_idx = level_lump_idx + BLOCK_MAP;
    let lump = wad_sys.get_lump_or_err(block_map_lump_idx)?;

    let len = lump.len();
    let mut buf = vec![0u8; len];
    lump.read(&mut buf)?;

    level_map.block_map = buf.as_slice().try_into()?;

    Ok(())
}
