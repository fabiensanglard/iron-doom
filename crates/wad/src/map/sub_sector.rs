use crate::lump::LumpsDirectory;
use crate::util::bytes_to_i16;
use anyhow::{bail, Result};
use derive_more::{Deref, DerefMut, IntoIterator};

#[derive(Deref, DerefMut, IntoIterator, Debug)]
#[into_iterator(owned, ref, ref_mut)]
pub struct MapSubSectors(Vec<MapSubSector>);

pub struct MapSubSectorsParser;

impl MapSubSectorsParser {
    pub fn parse(lumps_dir: &LumpsDirectory, map_lump: usize) -> Result<MapSubSectors> {
        let sub_sectors_lump = map_lump + 6;
        let Some(lump) = lumps_dir.get_index(sub_sectors_lump) else {
            bail!("Missing SubSectors Lump for Map Lump #{map_lump}");
        };

        let mut sub_sectors = Vec::with_capacity(lump.size() / 4);

        for sub_sector_data in lump.data().chunks_exact(4) {
            let num_segs = bytes_to_i16(&sub_sector_data[0..=1])?;
            let first_seg = bytes_to_i16(&sub_sector_data[2..=3])?;
            sub_sectors.push(MapSubSector {
                num_segs,
                first_seg,
            })
        }

        Ok(MapSubSectors(sub_sectors))
    }
}

#[derive(Debug)]
pub struct MapSubSector {
    #[allow(unused)]
    pub num_segs: i16,
    #[allow(unused)]
    pub first_seg: i16,
}
