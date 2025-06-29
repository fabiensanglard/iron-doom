use super::utils::{get_back_sector, get_front_sector};
use crate::components::{Line, LineSegment, PortalSegment, Sector, SideDef, WallSegment};
use crate::resources::LevelMap;
use anyhow::{bail, Result};
use bevy::prelude::*;
use moonshine_kind::{Instance, SpawnInstance};
use wad::prelude::*;

/// Vanilla Doom angles are stored in a format called BAM: Binary Angle Measure.
/// In this system, increasing values represent angles increasing counterclockwise,
/// using the full range of 32-bit unsigned precision to represent one full circle.
/// Because we want to use Bevy's math lib, we need to convert from BAM angles to radians.
///
/// Equal to 2π/2<sup>32</sup>.
const BAM_TO_RAD: f32 = std::f32::consts::FRAC_PI_4 / ((1u32 << 29) as f32);

pub trait SpawnSegments {
    fn spawn_segments(
        &mut self,
        map: &Map,
        level_map: &LevelMap,
    ) -> Result<Vec<Instance<LineSegment>>>;
}

impl SpawnSegments for Commands<'_, '_> {
    fn spawn_segments(
        &mut self,
        map: &Map,
        level_map: &LevelMap,
    ) -> Result<Vec<Instance<LineSegment>>> {
        let mut segments = Vec::with_capacity(map.segments.len());
        for segment in &map.segments {
            let segment = create_line_segment(segment, map, level_map)?;
            let instance = self.spawn_instance(segment).instance();
            segments.push(instance);
        }
        Ok(segments)
    }
}

fn create_line_segment(seg: &MapSegment, map: &Map, level_map: &LevelMap) -> Result<LineSegment> {
    let v1 = get_start_vertex(seg, level_map)?;
    let v2 = get_end_vertex(seg, level_map)?;
    let normal = get_normal(seg);
    let line = get_line(seg, level_map)?;
    let side = get_side(seg, map, level_map)?;
    let (front_sector, back_sector) = get_sectors(seg, map, level_map)?;
    let offset: f32 = seg.offset.into();

    let segment = match back_sector {
        None => LineSegment::Wall(WallSegment {
            v1,
            v2,
            normal,
            line,
            side,
            offset,
            front_sector,
        }),
        Some(back_sector) => LineSegment::Portal(PortalSegment {
            v1,
            v2,
            normal,
            line,
            side,
            offset,
            front_sector,
            back_sector,
        }),
    };

    Ok(segment)
}

fn get_start_vertex(seg: &MapSegment, level_map: &LevelMap) -> Result<Vec2> {
    let v1 = usize::try_from(seg.v1)?;
    get_vertex(v1, level_map)
}

fn get_end_vertex(seg: &MapSegment, level_map: &LevelMap) -> Result<Vec2> {
    let v2 = usize::try_from(seg.v2)?;
    get_vertex(v2, level_map)
}

fn get_vertex(vertex: usize, level_map: &LevelMap) -> Result<Vec2> {
    let Some(vertex) = level_map.lines_vertexes.get(vertex) else {
        bail!("Line segment references invalid vertex");
    };
    Ok(*vertex)
}

fn get_sectors(
    seg: &MapSegment,
    map: &Map,
    level_map: &LevelMap,
) -> Result<(Instance<Sector>, Option<Instance<Sector>>)> {
    let map_line = get_map_line(seg, map)?;
    let front_sector = get_front_sector(map_line, map, level_map)?;
    let back_sector = get_back_sector(map_line, map, level_map)?;

    if seg.side == 0 {
        return Ok((front_sector, back_sector));
    }
    let back_sector = back_sector.unwrap();
    Ok((back_sector, Some(front_sector)))
}

fn get_map_line<'a>(seg: &MapSegment, map: &'a Map) -> Result<&'a MapLine> {
    let line = usize::try_from(seg.line)?;
    let Some(line) = map.lines.get(line) else {
        bail!("Line segment references invalid line");
    };
    Ok(line)
}

fn get_line(seg: &MapSegment, level_map: &LevelMap) -> Result<Instance<Line>> {
    let line = usize::try_from(seg.line)?;
    let Some(line) = level_map.lines.get(line) else {
        bail!("Line segment references invalid line");
    };
    Ok(*line)
}

fn get_normal(seg: &MapSegment) -> Dir2 {
    let bam_angle = (seg.angle as u32) << 16;
    let angle = (bam_angle as f32) * BAM_TO_RAD;
    // Rotate clockwise 90° to get the normal angle.
    let normal_angle = angle - std::f32::consts::FRAC_PI_2;
    Rot2::radians(normal_angle) * Dir2::X
}

fn get_side(seg: &MapSegment, map: &Map, level_map: &LevelMap) -> Result<Instance<SideDef>> {
    let line = usize::try_from(seg.line)?;
    let Some(line) = map.lines.get(line) else {
        bail!("Line segment references invalid line");
    };
    let side = if seg.side == 0 {
        usize::try_from(line.front_side)?
    } else {
        usize::try_from(line.back_side)?
    };
    let Some(side) = level_map.lines_sides.get(side) else {
        bail!("Line segment references side");
    };
    Ok(*side)
}
