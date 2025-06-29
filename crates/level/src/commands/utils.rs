use crate::components::Sector;
use crate::resources::LevelMap;
use anyhow::{bail, Result};
use moonshine_kind::Instance;
use wad::prelude::{Map, MapLine};

pub fn get_front_sector(
    line: &MapLine,
    map: &Map,
    level_map: &LevelMap,
) -> Result<Instance<Sector>> {
    let front_side = usize::try_from(line.front_side)?;
    let Some(front_side) = map.side_defs.get(front_side) else {
        bail!("");
    };
    let front_sector = usize::try_from(front_side.sector)?;
    get_sector(front_sector, level_map)
}

pub fn get_back_sector(
    line: &MapLine,
    map: &Map,
    level_map: &LevelMap,
) -> Result<Option<Instance<Sector>>> {
    if line.back_side < 0 {
        return Ok(None);
    }
    let back_side = usize::try_from(line.back_side)?;
    let Some(back_side) = map.side_defs.get(back_side) else {
        bail!("");
    };
    let back_sector = usize::try_from(back_side.sector)?;
    let back_sector = get_sector(back_sector, level_map)?;
    Ok(Some(back_sector))
}

fn get_sector(sector: usize, level_map: &LevelMap) -> Result<Instance<Sector>> {
    let Some(sector) = level_map.sectors.get(sector) else {
        bail!("");
    };
    Ok(*sector)
}
