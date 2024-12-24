use bevy::prelude::*;

use engine_core::app::exit_error;
use engine_core::geometry::Vertex;
use engine_core::wad_system::{WadSystem, WadVertexes};

use crate::level::load_level::{LoadLevelLump, LoadLevelSet};
use crate::level::LevelMap;

const VERTEXES: usize = 4;

pub struct LoadVertexesPlugin;

impl Plugin for LoadVertexesPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            FixedPreUpdate,
            (clear_vertexes, load_vertexes.pipe(exit_error))
                .chain()
                .in_set(LoadLevelSet::Component),
        );
    }
}

fn clear_vertexes(mut commands: Commands, mut level_map: ResMut<LevelMap>) {
    super::despawn_all(&mut commands, level_map.vertexes.iter().cloned());
    level_map.vertexes.clear();
}

pub(super) fn load_vertexes(
    mut load_event: EventReader<LoadLevelLump>,
    mut wad_sys: NonSendMut<WadSystem>,
    mut commands: Commands,
    mut level_map: ResMut<LevelMap>,
) -> Result<(), String> {
    let Some(ev) = load_event.read().last() else {
        return Ok(());
    };
    let level_lump_idx = ev.lump_idx;
    let vertexes_lump_idx = level_lump_idx + VERTEXES;

    let lump_data = wad_sys.cache_lump_idx(vertexes_lump_idx)?;
    let map_vertexes: WadVertexes = lump_data.as_slice().try_into()?;

    for vertex in map_vertexes {
        let vertex_id = commands
            .spawn(Vertex {
                x: vertex.x.into(),
                y: vertex.y.into(),
            })
            .id();

        level_map.vertexes.push(vertex_id);
    }

    Ok(())
}
