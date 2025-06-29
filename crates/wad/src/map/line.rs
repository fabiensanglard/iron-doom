use crate::lump::LumpsDirectory;
use crate::util::bytes_to_i16;
use anyhow::{bail, Result};
use derive_more::{Deref, DerefMut, IntoIterator};

#[derive(Deref, DerefMut, IntoIterator, Debug)]
#[into_iterator(owned, ref, ref_mut)]
pub struct MapLines(Vec<MapLine>);

pub struct MapLinesParser;

impl MapLinesParser {
    pub fn parse(lumps_dir: &LumpsDirectory, map_lump: usize) -> Result<MapLines> {
        let lines_lump = map_lump + 2;
        let Some(lump) = lumps_dir.get_index(lines_lump) else {
            bail!("Missing Lines Lump for Map Lump #{map_lump}");
        };

        let mut lines = Vec::with_capacity(lump.size() / 14);

        for line_data in lump.data().chunks_exact(14) {
            let v1 = bytes_to_i16(&line_data[0..=1])?;
            let v2 = bytes_to_i16(&line_data[2..=3])?;
            let flags = bytes_to_i16(&line_data[4..=5])?;
            let special = bytes_to_i16(&line_data[6..=7])?;
            let tag = bytes_to_i16(&line_data[8..=9])?;
            let front_side = bytes_to_i16(&line_data[10..=11])?;
            let back_side = bytes_to_i16(&line_data[12..=13])?;
            lines.push(MapLine {
                v1,
                v2,
                flags,
                special,
                tag,
                front_side,
                back_side,
            })
        }

        Ok(MapLines(lines))
    }
}

#[derive(Debug)]
pub struct MapLine {
    #[allow(unused)]
    pub v1: i16,
    #[allow(unused)]
    pub v2: i16,
    #[allow(unused)]
    pub flags: i16,
    #[allow(unused)]
    pub special: i16,
    #[allow(unused)]
    pub tag: i16,
    #[allow(unused)]
    pub front_side: i16,
    #[allow(unused)]
    pub back_side: i16,
}
