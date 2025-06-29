use super::utils::{get_back_sector, get_front_sector};
use crate::components::{Line, PortalLine, WallLine};
use crate::resources::LevelMap;
use anyhow::{bail, Result};
use bevy::prelude::*;
use moonshine_kind::{Instance, SpawnInstance};
use wad::prelude::*;

pub trait SpawnLines {
    fn spawn_lines(&mut self, map: &Map, level_map: &LevelMap) -> Result<Vec<Instance<Line>>>;
}

impl SpawnLines for Commands<'_, '_> {
    fn spawn_lines(&mut self, map: &Map, level_map: &LevelMap) -> Result<Vec<Instance<Line>>> {
        let mut lines = Vec::with_capacity(map.lines.len());
        for line in &map.lines {
            let line = create_line(line, map, level_map)?;
            let instance = self.spawn_instance(line).instance();
            lines.push(instance);
        }
        Ok(lines)
    }
}

fn create_line(line: &MapLine, map: &Map, level_map: &LevelMap) -> Result<Line> {
    let v1 = get_start_vertex(line, level_map)?;
    let v2 = get_end_vertex(line, level_map)?;
    let front_sector = get_front_sector(line, map, level_map)?;
    let back_sector = get_back_sector(line, map, level_map)?;

    let line = match back_sector {
        None => Line::Wall(WallLine {
            v1,
            v2,
            flags: line.flags,
            special: line.special,
            tag: line.tag,
            front_sector,
        }),
        Some(back_sector) => Line::Portal(PortalLine {
            v1,
            v2,
            flags: line.flags,
            special: line.special,
            tag: line.tag,
            front_sector,
            back_sector,
        }),
    };

    Ok(line)
}

fn get_start_vertex(line: &MapLine, level_map: &LevelMap) -> Result<Vec2> {
    let v1 = usize::try_from(line.v1)?;
    get_vertex(v1, level_map)
}

fn get_end_vertex(line: &MapLine, level_map: &LevelMap) -> Result<Vec2> {
    let v2 = usize::try_from(line.v2)?;
    get_vertex(v2, level_map)
}

fn get_vertex(vertex: usize, level_map: &LevelMap) -> Result<Vec2> {
    let Some(v1) = level_map.lines_vertexes.get(vertex) else {
        bail!("Line references invalid vertex");
    };
    Ok(*v1)
}
