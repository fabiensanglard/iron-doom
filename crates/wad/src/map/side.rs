use crate::lump::LumpsDirectory;
use crate::util::{bytes_to_i16, bytes_to_str};
use anyhow::{bail, Result};
use derive_more::{Deref, DerefMut, IntoIterator};

#[derive(Deref, DerefMut, IntoIterator, Debug)]
#[into_iterator(owned, ref, ref_mut)]
pub struct MapSideDefs(Vec<MapSideDef>);

pub struct MapSideDefsParser;

impl MapSideDefsParser {
    pub fn parse(lumps_dir: &LumpsDirectory, map_lump: usize) -> Result<MapSideDefs> {
        let side_defs_lump = map_lump + 3;
        let Some(lump) = lumps_dir.get_index(side_defs_lump) else {
            bail!("Missing SideDefs Lump for Map Lump #{map_lump}");
        };

        let mut side_defs = Vec::with_capacity(lump.size() / 30);

        for side_def_data in lump.data().chunks_exact(30) {
            let x = bytes_to_i16(&side_def_data[0..=1])?;
            let y = bytes_to_i16(&side_def_data[2..=3])?;
            let top_texture = bytes_to_str(&side_def_data[4..=11])?;
            let lower_texture = bytes_to_str(&side_def_data[12..=19])?;
            let middle_texture = bytes_to_str(&side_def_data[20..=27])?;
            let sector = bytes_to_i16(&side_def_data[28..=29])?;
            side_defs.push(MapSideDef {
                x_offset: x,
                y_offset: y,
                top_texture: top_texture.to_owned(),
                lower_texture: lower_texture.to_owned(),
                middle_texture: middle_texture.to_owned(),
                sector,
            });
        }

        Ok(MapSideDefs(side_defs))
    }
}

#[derive(Debug)]
pub struct MapSideDef {
    #[allow(unused)]
    pub x_offset: i16,
    #[allow(unused)]
    pub y_offset: i16,
    #[allow(unused)]
    pub top_texture: String,
    #[allow(unused)]
    pub lower_texture: String,
    #[allow(unused)]
    pub middle_texture: String,
    #[allow(unused)]
    pub sector: i16,
}
