use bevy::prelude::*;

use engine_core::app::exit_error;
use engine_core::geometry::{BranchNode, Node};
use engine_core::wad_system::{WadNodes, WadSystem};

use crate::level::load_level::{LoadLevelLump, LoadLevelSet};
use crate::level::sub_sectors::load_sub_sectors;
use crate::level::LevelMap;

const NODES: usize = 7;
const SUB_SECTOR_FLAG: u16 = 0x8000;

pub struct LoadNodesPlugin;

impl Plugin for LoadNodesPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            FixedPreUpdate,
            (clear_nodes, load_nodes.pipe(exit_error))
                .chain()
                .in_set(LoadLevelSet::Component)
                .after(load_sub_sectors),
        );
    }
}

fn clear_nodes(mut commands: Commands, mut level_map: ResMut<LevelMap>) {
    let level_node_ids = level_map
        .bsp_nodes
        .iter()
        .filter(|node| node.is_branch())
        .map(|node| node.entity());
    super::despawn_all(&mut commands, level_node_ids);
    level_map.bsp_nodes.clear();
}

pub(super) fn load_nodes(
    mut load_event: EventReader<LoadLevelLump>,
    mut wad_sys: NonSendMut<WadSystem>,
    mut commands: Commands,
    mut level_map: ResMut<LevelMap>,
) -> Result<(), String> {
    let Some(ev) = load_event.read().last() else {
        return Ok(());
    };
    let level_lump_idx = ev.lump_idx;
    let nodes_lump_idx = level_lump_idx + NODES;

    let lump_data = wad_sys.cache_lump_idx(nodes_lump_idx)?;
    let map_nodes: WadNodes = lump_data.as_slice().try_into()?;

    let node_num = map_nodes.len();
    if node_num > u16::MAX as usize || node_num == 0 {
        return Err("load_nodes: incorrect node_num!".to_string());
    }

    traverse_bsp(
        &mut commands,
        &mut level_map,
        &map_nodes,
        (node_num - 1) as u16,
    );

    Ok(())
}

fn traverse_bsp(
    commands: &mut Commands,
    level_map: &mut LevelMap,
    map_nodes: &WadNodes,
    node_num: u16,
) -> Node {
    if node_num & SUB_SECTOR_FLAG != 0 {
        let sub_sector_num = (node_num & !SUB_SECTOR_FLAG) as usize;
        let sub_sector_id = level_map.sub_sectors[sub_sector_num];
        return Node::Leaf(sub_sector_id);
    }

    let node = map_nodes[node_num as usize];
    let right_child = traverse_bsp(commands, level_map, map_nodes, node.right_child);
    let left_child = traverse_bsp(commands, level_map, map_nodes, node.left_child);
    let node_id = commands
        .spawn(BranchNode {
            x: node.x.into(),
            y: node.y.into(),
            dx: node.dx.into(),
            dy: node.dy.into(),
            right_bounding_box: node.right_bounding_box,
            left_bounding_box: node.left_bounding_box,
            right_child,
            left_child,
        })
        .id();

    let node = Node::Branch(node_id);
    level_map.bsp_nodes.push(node);

    node
}
