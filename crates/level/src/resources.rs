use crate::commands::*;
use crate::components::*;
use crate::map_object::prelude::*;
use anyhow::{bail, Result};
pub use bevy::prelude::*;
use moonshine_kind::{GetInstanceCommands, Instance, Kind};
use std::rc::Rc;
use wad::prelude::*;

pub(crate) struct LevelMap {
    pub lines: Vec<Instance<Line>>,
    pub lines_segments: Vec<Instance<LineSegment>>,
    pub lines_sides: Vec<Instance<SideDef>>,
    pub lines_vertexes: Vec<Vec2>,
    pub sectors: Vec<Instance<Sector>>,
    pub sub_sectors: Vec<Instance<SubSector>>,
    pub map_objects: Vec<Instance<MapObject>>,
    pub root_node: Rc<BspNode>,
    pub num_bsp_nodes: usize,
}

impl Default for LevelMap {
    fn default() -> Self {
        Self {
            lines: vec![],
            lines_segments: vec![],
            lines_sides: vec![],
            lines_vertexes: vec![],
            sectors: vec![],
            sub_sectors: vec![],
            map_objects: vec![],
            root_node: Rc::new(BspNode::Leaf(Instance::PLACEHOLDER)),
            num_bsp_nodes: 0,
        }
    }
}

impl LevelMap {
    pub fn load(&mut self, commands: &mut Commands, map: &Map, wad: &WadFile) -> Result<()> {
        self.load_vertexes(map);
        self.load_sectors(commands, map, wad)?;
        self.load_lines(commands, map)?;
        self.load_lines_sides(commands, map, wad)?;
        self.load_segments(commands, map)?;
        self.load_sub_sectors(commands, map)?;
        self.load_things(commands, map);
        self.load_bsp(map)?;
        Ok(())
    }

    fn load_vertexes(&mut self, map: &Map) {
        self.lines_vertexes.clear();
        for vertex in &map.vertexes {
            self.lines_vertexes.push(Vec2 {
                x: vertex.x.into(),
                y: vertex.y.into(),
            });
        }
    }

    fn load_sectors(&mut self, commands: &mut Commands, map: &Map, wad: &WadFile) -> Result<()> {
        unload_helper(commands, &mut self.sectors);
        self.sectors = commands.spawn_sectors(map, wad)?;
        Ok(())
    }

    fn load_lines(&mut self, commands: &mut Commands, map: &Map) -> Result<()> {
        unload_helper(commands, &mut self.lines);
        self.lines = commands.spawn_lines(map, self)?;
        Ok(())
    }

    fn load_segments(&mut self, commands: &mut Commands, map: &Map) -> Result<()> {
        unload_helper(commands, &mut self.lines_segments);
        self.lines_segments = commands.spawn_segments(map, self)?;
        Ok(())
    }

    fn load_sub_sectors(&mut self, commands: &mut Commands, map: &Map) -> Result<()> {
        unload_helper(commands, &mut self.sub_sectors);
        self.sub_sectors = commands.spawn_sub_sectors(map, self)?;
        Ok(())
    }

    fn load_lines_sides(
        &mut self,
        commands: &mut Commands,
        map: &Map,
        wad: &WadFile,
    ) -> Result<()> {
        unload_helper(commands, &mut self.lines_sides);
        self.lines_sides = commands.spawn_lines_sides(map, self, wad)?;
        Ok(())
    }

    fn load_things(&mut self, commands: &mut Commands, map: &Map) {
        unload_helper(commands, &mut self.map_objects);
        self.map_objects = commands.spawn_map_objects(map);
    }

    fn load_bsp(&mut self, map: &Map) -> Result<()> {
        // The root node is the last node output.
        let root_node_num = map.nodes.len() - 1;
        self.root_node = load_bsp_helper(root_node_num, self, &map.nodes)?;
        self.num_bsp_nodes = map.nodes.len();
        Ok(())
    }
}

fn load_bsp_helper(node: usize, level_map: &LevelMap, map_nodes: &MapNodes) -> Result<Rc<BspNode>> {
    let Some(node) = map_nodes.get(node) else {
        bail!("Tried to access invalid BSP node.");
    };

    // If negative (bit 15 is set), then it is a leaf node.
    let right_child = if node.right_child >= 0 {
        load_bsp_helper(node.right_child as usize, level_map, map_nodes)?
    } else {
        load_bsp_leaf_node(node.right_child, level_map)?
    };
    let left_child = if node.left_child >= 0 {
        load_bsp_helper(node.left_child as usize, level_map, map_nodes)?
    } else {
        load_bsp_leaf_node(node.left_child, level_map)?
    };

    let branch_node = Rc::new(BranchNode {
        origin: Vec2::new(node.x.into(), node.y.into()),
        direction: Vec2::new(node.dx.into(), node.dy.into()),
        right_child,
        left_child,
    });
    let branch_node = Rc::new(BspNode::Branch(branch_node));

    Ok(branch_node)
}

fn load_bsp_leaf_node(leaf: i16, level_map: &LevelMap) -> Result<Rc<BspNode>> {
    // In a leaf node, bits 0-14 determines the sub-sector the leaf points to.
    let sub_sector = (leaf & 0x7FFF) as usize;
    let Some(sub_sector) = level_map.sub_sectors.get(sub_sector) else {
        bail!("BSP leaf node references invalid sub-sector.");
    };
    let leaf_node = Rc::new(BspNode::Leaf(*sub_sector));
    Ok(leaf_node)
}

fn unload_helper<T: Kind>(commands: &mut Commands, components: &mut Vec<Instance<T>>) {
    for entity in components.drain(..) {
        commands.instance(entity).despawn();
    }
}
