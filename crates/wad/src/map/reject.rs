use crate::lump::LumpsDirectory;
use anyhow::{bail, Result};

#[derive(Debug)]
#[allow(unused)]
pub struct MapRejectMatrix(Vec<u8>);

pub struct MapRejectMatrixParser;

impl MapRejectMatrixParser {
    pub fn parse(lumps_dir: &LumpsDirectory, map_lump: usize) -> Result<MapRejectMatrix> {
        let vertexes_lump = map_lump + 9;
        let Some(lump) = lumps_dir.get_index(vertexes_lump) else {
            bail!("Missing Reject Matrix Lump for Map Lump #{map_lump}");
        };
        let reject_data = lump.data().to_owned();
        Ok(MapRejectMatrix(reject_data))
    }
}
