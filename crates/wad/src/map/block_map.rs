use crate::lump::LumpsDirectory;
use crate::util::bytes_to_i16;
use anyhow::{bail, Result};

#[derive(Debug)]
pub struct MapBlockMap {
    #[allow(unused)]
    pub origin_x: i16,
    #[allow(unused)]
    pub origin_y: i16,
    #[allow(unused)]
    pub width: i16,
    #[allow(unused)]
    pub height: i16,
    #[allow(unused)]
    pub offsets: Vec<i16>,
    #[allow(unused)]
    pub block_lists: Vec<i16>,
}

pub struct MapBlockMapParser;

impl MapBlockMapParser {
    pub fn parse(lumps_dir: &LumpsDirectory, map_lump: usize) -> Result<MapBlockMap> {
        let block_map_lump = map_lump + 10;
        let Some(lump) = lumps_dir.get_index(block_map_lump) else {
            bail!("Missing Block Map Lump for Map Lump #{map_lump}");
        };

        let block_map_data = lump.data();

        let origin_x = bytes_to_i16(&block_map_data[0..=1])?;
        let origin_y = bytes_to_i16(&block_map_data[2..=3])?;
        let width = bytes_to_i16(&block_map_data[4..=5])?;
        let height = bytes_to_i16(&block_map_data[6..=7])?;

        let size = (width * height) as usize;

        let mut offsets = Vec::with_capacity(size);
        let offsets_data = &block_map_data[8..(8 + 2 * size)];
        for offset_data in offsets_data.chunks_exact(2) {
            let offset = bytes_to_i16(offset_data)?;
            offsets.push(offset);
        }

        let mut block_lists = Vec::new();
        let blocks_data = &block_map_data[(8 + 2 * size)..];
        for block_data in blocks_data.chunks_exact(2) {
            let line_index = bytes_to_i16(block_data)?;
            block_lists.push(line_index);
        }

        Ok(MapBlockMap {
            origin_x,
            origin_y,
            width,
            height,
            offsets,
            block_lists,
        })
    }
}
