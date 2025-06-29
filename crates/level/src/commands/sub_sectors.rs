use crate::components::{LineSegment, SubSector};
use crate::resources::LevelMap;
use anyhow::{bail, Result};
use bevy::prelude::*;
use moonshine_kind::{Instance, SpawnInstance};
use wad::prelude::*;

pub trait SpawnSubSectors {
    fn spawn_sub_sectors(
        &mut self,
        map: &Map,
        level_map: &LevelMap,
    ) -> Result<Vec<Instance<SubSector>>>;
}

impl SpawnSubSectors for Commands<'_, '_> {
    fn spawn_sub_sectors(
        &mut self,
        map: &Map,
        level_map: &LevelMap,
    ) -> Result<Vec<Instance<SubSector>>> {
        let mut sub_sectors = Vec::with_capacity(map.sub_sectors.len());
        for sub_sector in &map.sub_sectors {
            let sub_sector = create_sub_sector(sub_sector, level_map)?;
            let instance = self.spawn_instance(sub_sector).instance();
            sub_sectors.push(instance);
        }
        Ok(sub_sectors)
    }
}

fn create_sub_sector(sub_sector: &MapSubSector, level_map: &LevelMap) -> Result<SubSector> {
    let segments = get_segments(sub_sector, level_map)?;
    Ok(SubSector { segments })
}

fn get_segments(
    sub_sector: &MapSubSector,
    level_map: &LevelMap,
) -> Result<Vec<Instance<LineSegment>>> {
    let Ok(num_segs) = usize::try_from(sub_sector.num_segs) else {
        bail!("Subsector contains negative number of segments.");
    };
    let Ok(first_seg) = usize::try_from(sub_sector.first_seg) else {
        bail!("Subsector references invalid segment.");
    };
    let last_seg = first_seg + num_segs;

    let segments = level_map
        .lines_segments
        .get(first_seg..last_seg)
        .map(|segments| segments.to_vec());

    if let Some(segments) = segments {
        return Ok(segments);
    }
    bail!("Subsector references invalid segment.");
}
