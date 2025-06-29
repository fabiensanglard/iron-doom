use crate::prelude::Sector;
use anyhow::{bail, Result};
use bevy::prelude::*;
use moonshine_kind::{Instance, SpawnInstance};
use wad::prelude::*;

pub trait SpawnSectors {
    fn spawn_sectors(&mut self, map: &Map, wad: &WadFile) -> Result<Vec<Instance<Sector>>>;
}

impl SpawnSectors for Commands<'_, '_> {
    fn spawn_sectors(&mut self, map: &Map, wad: &WadFile) -> Result<Vec<Instance<Sector>>> {
        let mut sectors = Vec::with_capacity(map.sectors.len());
        for sector in &map.sectors {
            let sector = create_sector(sector, wad)?;
            let instance = self.spawn_instance(sector).instance();
            sectors.push(instance);
        }
        Ok(sectors)
    }
}

fn create_sector(sector: &MapSector, wad: &WadFile) -> Result<Sector> {
    let flats = wad.flats();
    let Some(floor_tex) = flats.get_index_of(&sector.floor_tex) else {
        bail!(
            "Sector references invalid floor texture \"{}\".",
            sector.floor_tex
        );
    };
    let Some(ceiling_tex) = flats.get_index_of(&sector.ceiling_tex) else {
        bail!(
            "Sector references invalid ceiling texture \"{}\".",
            sector.ceiling_tex
        );
    };
    Ok(Sector {
        floor_height: sector.floor_height.into(),
        ceiling_height: sector.ceiling_height.into(),
        floor_tex,
        ceiling_tex,
        light_level: sector.light_level,
        special: sector.special,
        tag: sector.tag,
    })
}
