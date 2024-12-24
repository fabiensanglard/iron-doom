use bevy::ecs::schedule::ScheduleLabel;
use bevy::prelude::*;

use engine_core::geometry::Node;
use engine_core::geometry::*;

use crate::level::map_object::{Player, Position};
use crate::level::LevelMap;

use cull::CullPlugin;
use draw_columns::DrawColumnsPlugin;
use extract::{ExtractPlugin, ExtractSegments};
use horizontal_clip::HorizontalClipPlugin;

use super::RenderSet;

mod cull;
mod draw_columns;
mod extract;
mod horizontal_clip;

pub struct RenderWallsPlugin;

impl Plugin for RenderWallsPlugin {
    fn build(&self, app: &mut App) {
        app.init_schedule(RenderSubSectorSchedule)
            .configure_sets(
                RenderSubSectorSchedule,
                (
                    RenderSubSectorSet::Extract,
                    RenderSubSectorSet::Cull,
                    RenderSubSectorSet::HorizontalClip,
                    RenderSubSectorSet::DrawColumns,
                )
                    .chain()
                    .in_set(RenderSet::Walls),
            )
            .add_plugins((
                ExtractPlugin,
                CullPlugin,
                HorizontalClipPlugin,
                DrawColumnsPlugin,
            ))
            .add_systems(PostUpdate, traverse_bsp_tree.in_set(RenderSet::Walls));
    }
}

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub enum RenderSubSectorSet {
    Extract,
    Cull,
    HorizontalClip,
    DrawColumns,
}

fn traverse_bsp_tree(world: &mut World) {
    // Vanilla DOOM implements BSP traversal using recursion, but this
    // approach is not well-suited for working with Bevy's ECS systems.
    // Instead, we use an iterative implementation with a node stack,
    // similar to what is done in FastDoom.
    let mut player_pos_query = world.query_filtered::<&Position, With<Player>>();
    let player_pos = *player_pos_query.single(world);

    let root = world.resource::<LevelMap>().root_bsp_node();
    let mut node_stack = Vec::new();
    node_stack.push(root);

    while let Some(node) = node_stack.pop() {
        match node {
            Node::Branch(branch_node) => {
                let branch_node = world.get::<BranchNode>(branch_node).unwrap();

                if branch_node.is_on_back(player_pos.x, player_pos.y) {
                    // Traverse left child first
                    node_stack.push(branch_node.right_child);
                    node_stack.push(branch_node.left_child);
                } else {
                    // Traverse right child first
                    node_stack.push(branch_node.left_child);
                    node_stack.push(branch_node.right_child);
                };
            }
            Node::Leaf(sub_sector) => {
                world.send_event(ExtractSegments { sub_sector });
                world.run_schedule(RenderSubSectorSchedule);
            }
        }
    }
}

#[derive(ScheduleLabel, Debug, Hash, PartialEq, Eq, Clone)]
struct RenderSubSectorSchedule;
