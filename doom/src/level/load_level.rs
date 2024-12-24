use bevy::prelude::*;

use crate::GameState;
use block_map::LoadBlockMapPlugin;
use engine_core::app::exit_error;
use engine_core::geometry::Node;
use engine_core::wad_system::{WadBlockMap, WadRejectTable, WadSystem, WadThings};
use line_defs::LoadLinesPlugin;
use nodes::LoadNodesPlugin;
use reject_table::LoadRejectTablePlugin;
use sectors::LoadSectorsPlugin;
use segments::LoadSegmentsPlugin;
use side_defs::LoadSidesPlugin;
use sub_sectors::LoadSubSectorsPlugin;
use things::LoadThingsPlugin;
use vertexes::LoadVertexesPlugin;

pub mod block_map;
pub mod line_defs;
pub mod nodes;
pub mod reject_table;
pub mod sectors;
pub mod segments;
pub mod side_defs;
pub mod sub_sectors;
pub mod things;
pub mod vertexes;

pub struct LoadLevelPlugin;

impl Plugin for LoadLevelPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<LoadLevel>()
            .add_event::<LoadLevelLump>()
            .add_event::<LoadLevelDone>()
            .init_resource::<LevelMap>()
            .configure_sets(
                FixedPreUpdate,
                (LoadLevelSet::Component, LoadLevelSet::Done)
                    .chain()
                    .after(load_level)
                    .run_if(on_event::<LoadLevelLump>()),
            )
            .add_plugins((
                LoadBlockMapPlugin,
                LoadLinesPlugin,
                LoadNodesPlugin,
                LoadRejectTablePlugin,
                LoadSectorsPlugin,
                LoadSegmentsPlugin,
                LoadSidesPlugin,
                LoadSubSectorsPlugin,
                LoadThingsPlugin,
                LoadVertexesPlugin,
            ))
            .add_systems(
                FixedPreUpdate,
                load_level.pipe(exit_error).run_if(on_event::<LoadLevel>()),
            )
            .add_systems(
                FixedPreUpdate,
                load_level_done
                    .run_if(on_event::<LoadLevel>())
                    .in_set(LoadLevelSet::Done),
            );
    }
}

fn despawn_all(commands: &mut Commands, ids: impl Iterator<Item = Entity>) {
    ids.for_each(|id| commands.entity(id).despawn());
}

fn load_level(
    wad_sys: NonSend<WadSystem>,
    mut load_level: EventReader<LoadLevel>,
    mut load_lump: EventWriter<LoadLevelLump>,
) -> Result<(), String> {
    let Some(event) = load_level.read().last() else {
        return Ok(());
    };
    let LoadLevel { episode, map } = event;
    let lump_name = format!("E{episode}M{map}");
    let lump_idx = wad_sys.get_lump_idx_or_err(&lump_name)?;
    load_lump.send(LoadLevelLump { lump_idx });

    Ok(())
}

fn load_level_done(
    mut load_done: EventWriter<LoadLevelDone>,
    mut game_state: ResMut<NextState<GameState>>,
) {
    load_done.send_default();
    game_state.set(GameState::Level);
}

#[derive(Event)]
pub struct LoadLevel {
    pub episode: u32,
    pub map: u32,
}

#[derive(Event, Default)]
pub struct LoadLevelDone;

#[derive(Event)]
struct LoadLevelLump {
    pub lump_idx: usize,
}

#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub enum LoadLevelSet {
    Component,
    Done,
}

#[derive(Resource, Default)]
pub struct LevelMap {
    block_map: WadBlockMap,
    bsp_nodes: Vec<Node>,
    lines: Vec<Entity>,
    reject_table: WadRejectTable,
    sectors: Vec<Entity>,
    segments: Vec<Entity>,
    sides: Vec<Entity>,
    sub_sectors: Vec<Entity>,
    things: WadThings,
    vertexes: Vec<Entity>,
}

impl LevelMap {
    pub fn lines(&self) -> &[Entity] {
        &self.lines
    }

    pub fn vertexes(&self) -> &[Entity] {
        &self.vertexes
    }

    pub fn segments(&self) -> &[Entity] {
        &self.segments
    }

    pub fn root_bsp_node(&self) -> Node {
        self.bsp_nodes
            .last()
            .copied()
            .expect("root_bsp_node: BSP tree is empty!")
    }
}
