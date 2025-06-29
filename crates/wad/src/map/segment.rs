use crate::lump::LumpsDirectory;
use crate::util::bytes_to_i16;
use anyhow::{bail, Result};
use derive_more::{Deref, DerefMut, IntoIterator};

#[derive(Deref, DerefMut, IntoIterator, Debug)]
#[into_iterator(owned, ref, ref_mut)]
pub struct MapSegments(Vec<MapSegment>);

pub struct MapSegmentsParser;

impl MapSegmentsParser {
    pub fn parse(lumps_dir: &LumpsDirectory, map_lump: usize) -> Result<MapSegments> {
        let segments_lump = map_lump + 5;
        let Some(lump) = lumps_dir.get_index(segments_lump) else {
            bail!("Missing Segments Lump for Map Lump #{map_lump}");
        };

        let mut segments = Vec::with_capacity(lump.size() / 12);

        for segment_data in lump.data().chunks_exact(12) {
            let v1 = bytes_to_i16(&segment_data[0..=1])?;
            let v2 = bytes_to_i16(&segment_data[2..=3])?;
            let angle = bytes_to_i16(&segment_data[4..=5])?;
            let line = bytes_to_i16(&segment_data[6..=7])?;
            let side = bytes_to_i16(&segment_data[8..=9])?;
            let offset = bytes_to_i16(&segment_data[10..=11])?;
            segments.push(MapSegment {
                v1,
                v2,
                angle,
                line,
                side,
                offset,
            })
        }

        Ok(MapSegments(segments))
    }
}

#[derive(Debug)]
pub struct MapSegment {
    #[allow(unused)]
    pub v1: i16,
    #[allow(unused)]
    pub v2: i16,
    #[allow(unused)]
    pub angle: i16,
    #[allow(unused)]
    pub line: i16,
    #[allow(unused)]
    pub side: i16,
    #[allow(unused)]
    pub offset: i16,
}
