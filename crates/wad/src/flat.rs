use crate::index_map::IndexMap;
use crate::lump::{Lump, LumpsDirectory};
use anyhow::{bail, Result};

#[derive(Debug)]
pub struct Flat {
    #[allow(unused)]
    data: Vec<u8>,
}

impl Flat {
    const WIDTH: usize = 64;
    const HEIGHT: usize = 64;
    const SIZE: usize = Self::WIDTH * Self::HEIGHT;
}

#[derive(Default, Debug)]
pub struct FlatParser;

impl FlatParser {
    pub fn parse(lump: &Lump) -> Result<Flat> {
        if lump.size() != Flat::SIZE {
            bail!(
                "Flat '{}' has invalid size: expected {} bytes, got {} bytes",
                lump.name(),
                Flat::SIZE,
                lump.size()
            );
        }
        let data = lump.data().to_vec();
        Ok(Flat { data })
    }
}

#[derive(Debug)]
pub struct Flats(IndexMap<String, Flat>);

impl Flats {
    pub fn get_index_of(&self, name: &str) -> Option<usize> {
        self.0.get_index_of(name)
    }

    pub fn get_by_index(&self, tex: usize) -> Option<&Flat> {
        self.0.get_index(tex).map(|(_, texture)| texture)
    }
}

pub struct FlatsParser;

impl FlatsParser {
    pub fn parse(lumps_dir: &LumpsDirectory) -> Result<Flats> {
        let Some(start_marker) = lumps_dir.get_index_of("F_START") else {
            bail!("WAD file is missing flat start marker");
        };
        let Some(end_marker) = lumps_dir.get_index_of("F_END") else {
            bail!("WAD file is missing flat end marker");
        };
        if end_marker < start_marker {
            bail!("WAD file has flat end marker placed before flat start marker")
        }
        if start_marker + 1 >= end_marker {
            bail!("WAD file contains no flats");
        }

        let first_flat = start_marker + 1;
        let last_flat = end_marker - 1;
        // We can safely unwrap here, because markers are guaranteed to exist in lump directory.
        let flat_lumps = lumps_dir.get_index(first_flat..=last_flat).unwrap();

        let mut flats = IndexMap::default();

        for lump in flat_lumps {
            if lump.is_marker() {
                // Skip marker lumps
                continue;
            }
            let flat = FlatParser::parse(lump)?;
            flats.insert(lump.name().to_owned(), flat);
        }

        Ok(Flats(flats))
    }
}
