use bevy::prelude::*;
use engine_core::app::exit_error;
use engine_core::wad_system::WadSystem;

use crate::level::load_level::{LoadLevelLump, LoadLevelSet};
use crate::level::map_object::{Player, SpawnPlayer};
use crate::level::LevelMap;

use engine_core::fixed_point::Fixed;
use engine_core::geometry::Angle;

const THINGS: usize = 1;

pub struct LoadThingsPlugin;

impl Plugin for LoadThingsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            FixedPreUpdate,
            load_things.pipe(exit_error).in_set(LoadLevelSet::Component),
        );
    }
}

pub(super) fn load_things(
    mut wad_sys: NonSendMut<WadSystem>,
    mut load_event: EventReader<LoadLevelLump>,
    mut commands: Commands,
    mut level_map: ResMut<LevelMap>,
    player_query: Query<Entity, With<Player>>,
) -> Result<(), String> {
    let Some(ev) = load_event.read().last() else {
        return Ok(());
    };

    for e in &player_query {
        commands.entity(e).despawn();
    }

    let level_lump_idx = ev.lump_idx;
    let things_lump_idx = level_lump_idx + THINGS;

    let lump_data = wad_sys.cache_lump_idx(things_lump_idx)?;

    level_map.things = lump_data.as_slice().try_into()?;

    for thing in &level_map.things {
        if thing.thing_type == 1 {
            let x = Fixed::from_num(thing.x);
            let y = Fixed::from_num(thing.y);
            let angle = thing.angle / 45;
            let angle = 0x20000000u32.wrapping_mul(angle as u32);
            let angle = Angle::new(angle);
            commands.add(SpawnPlayer { x, y, angle });
            break;
        }
    }

    Ok(())
}
