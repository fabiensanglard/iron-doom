use bevy::prelude::*;

use engine_core::app::exit_error;
use engine_core::geometry::Side;
use engine_core::wad_system::{WadSideDefs, WadSystem};

use crate::level::load_level::{LoadLevelLump, LoadLevelSet};
use crate::level::sectors::load_sectors;
use crate::level::LevelMap;

const SIDE_DEFS: usize = 3;

pub struct LoadSidesPlugin;

impl Plugin for LoadSidesPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            FixedPreUpdate,
            (clear_sides, load_side_defs.pipe(exit_error))
                .chain()
                .in_set(LoadLevelSet::Component)
                .after(load_sectors),
        );
    }
}

fn clear_sides(mut commands: Commands, mut level_map: ResMut<LevelMap>) {
    super::despawn_all(&mut commands, level_map.sides.iter().cloned());
    level_map.sides.clear();
}

pub(super) fn load_side_defs(
    mut load_event: EventReader<LoadLevelLump>,
    mut wad_sys: NonSendMut<WadSystem>,
    mut commands: Commands,
    mut level_map: ResMut<LevelMap>,
) -> Result<(), String> {
    let Some(ev) = load_event.read().last() else {
        return Ok(());
    };
    let level_lump_idx = ev.lump_idx;
    let sectors_lump_idx = level_lump_idx + SIDE_DEFS;

    let lump_data = wad_sys.cache_lump_idx(sectors_lump_idx)?;
    let map_side_defs: WadSideDefs = lump_data.as_slice().try_into()?;

    for map_side_def in map_side_defs {
        let side_id = commands
            .spawn(Side {
                x_offset: map_side_def.x_offset.into(),
                y_offset: map_side_def.y_offset.into(),
                top_texture: map_side_def.top_texture,
                bottom_texture: map_side_def.bottom_texture,
                middle_texture: map_side_def.middle_texture,
                sector: level_map.sectors[map_side_def.sector as usize],
            })
            .id();

        level_map.sides.push(side_id);
    }

    Ok(())
}
