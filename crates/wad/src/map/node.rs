use crate::lump::LumpsDirectory;
use crate::util::bytes_to_i16;
use anyhow::{bail, Result};
use derive_more::{Deref, DerefMut, IntoIterator};

#[derive(Deref, DerefMut, IntoIterator, Debug)]
#[into_iterator(owned, ref, ref_mut)]
pub struct MapNodes(Vec<MapNode>);

pub struct MapNodesParser;

impl MapNodesParser {
    pub fn parse(lumps_dir: &LumpsDirectory, map_lump: usize) -> Result<MapNodes> {
        let nodes_lump = map_lump + 7;
        let Some(lump) = lumps_dir.get_index(nodes_lump) else {
            bail!("Missing Nodes Lump for Map Lump #{map_lump}");
        };

        let mut nodes = Vec::with_capacity(lump.size() / 28);

        for node_data in lump.data().chunks_exact(28) {
            let x = bytes_to_i16(&node_data[0..=1])?;
            let y = bytes_to_i16(&node_data[2..=3])?;
            let dx = bytes_to_i16(&node_data[4..=5])?;
            let dy = bytes_to_i16(&node_data[6..=7])?;
            let right_box_top = bytes_to_i16(&node_data[8..=9])?;
            let right_box_bottom = bytes_to_i16(&node_data[10..=11])?;
            let right_box_left = bytes_to_i16(&node_data[12..=13])?;
            let right_box_right = bytes_to_i16(&node_data[14..=15])?;
            let left_box_top = bytes_to_i16(&node_data[16..=17])?;
            let left_box_bottom = bytes_to_i16(&node_data[18..=19])?;
            let left_box_left = bytes_to_i16(&node_data[20..=21])?;
            let left_box_right = bytes_to_i16(&node_data[22..=23])?;
            let right_child = bytes_to_i16(&node_data[24..=25])?;
            let left_child = bytes_to_i16(&node_data[26..=27])?;
            nodes.push(MapNode {
                x,
                y,
                dx,
                dy,
                right_box_top,
                right_box_bottom,
                right_box_left,
                right_box_right,
                left_box_top,
                left_box_bottom,
                left_box_left,
                left_box_right,
                right_child,
                left_child,
            })
        }

        Ok(MapNodes(nodes))
    }
}

#[derive(Debug)]
pub struct MapNode {
    #[allow(unused)]
    pub x: i16,
    #[allow(unused)]
    pub y: i16,
    #[allow(unused)]
    pub dx: i16,
    #[allow(unused)]
    pub dy: i16,
    #[allow(unused)]
    pub right_box_top: i16,
    #[allow(unused)]
    pub right_box_bottom: i16,
    #[allow(unused)]
    pub right_box_left: i16,
    #[allow(unused)]
    pub right_box_right: i16,
    #[allow(unused)]
    pub left_box_top: i16,
    #[allow(unused)]
    pub left_box_bottom: i16,
    #[allow(unused)]
    pub left_box_left: i16,
    #[allow(unused)]
    pub left_box_right: i16,
    #[allow(unused)]
    pub right_child: i16,
    #[allow(unused)]
    pub left_child: i16,
}
