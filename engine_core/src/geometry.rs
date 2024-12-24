use bevy::prelude::*;

pub use angle::*;

use crate::fixed_point::Fixed;
use crate::BoundingBox;

mod angle;

#[derive(Component, Default, Copy, Clone)]
pub struct Vertex {
    pub x: Fixed,
    pub y: Fixed,
}

#[derive(Component, Copy, Clone)]
pub struct Line {
    pub v1: Entity,
    pub v2: Entity,
    pub dx: Fixed,
    pub dy: Fixed,
    pub flags: i16,
    pub special: i16,
    pub sector_tag: i16,
    pub front_side_def: Entity,
    pub back_side_def: Option<Entity>,
    pub bounding_box: BoundingBox,
    pub slope_type: SlopeType,
    pub front_sector: Option<Entity>,
    pub back_sector: Option<Entity>,
    pub valid_count: u16,
}

#[derive(Component, Default, Clone)]
pub struct Sector {
    pub floor_height: Fixed,
    pub ceiling_height: Fixed,
    pub floor_pic: String,
    pub ceiling_pic: String,
    pub light_level: i16,
    pub special: i16,
    pub tag: i16,
}

#[derive(Component, Copy, Clone)]
pub struct SubSector {
    pub sector: Entity,
    pub num_lines: u16,
    pub first_line: u16,
}

#[derive(Component, Clone)]
pub struct Side {
    pub x_offset: Fixed,
    pub y_offset: Fixed,
    pub top_texture: String,
    pub bottom_texture: String,
    pub middle_texture: String,
    pub sector: Entity,
}

#[derive(Component, Copy, Clone)]
pub struct Segment {
    pub v1: Entity,
    pub v2: Entity,
    pub offset: Fixed,
    pub angle: Angle,
    pub side: Entity,
    pub line: Entity,
    pub front_sector: Entity,
    pub back_sector: Option<Entity>,
}

#[derive(Component, Default, Copy, Clone)]
pub enum SlopeType {
    #[default]
    Horizontal,
    Vertical,
    Positive,
    Negative,
}

impl SlopeType {
    pub fn from_fixed(dy: Fixed, dx: Fixed) -> Self {
        if dx == 0 {
            return SlopeType::Vertical;
        }
        if dy == 0 {
            return SlopeType::Horizontal;
        }
        if dy / dx > 0 {
            return SlopeType::Positive;
        }
        SlopeType::Negative
    }
}

#[derive(Component, Copy, Clone)]
pub enum Node {
    Branch(Entity),
    Leaf(Entity),
}

impl Node {
    pub fn is_branch(&self) -> bool {
        match self {
            Node::Branch(_) => true,
            Node::Leaf(_) => false,
        }
    }

    pub fn is_leaf(&self) -> bool {
        match self {
            Node::Branch(_) => false,
            Node::Leaf(_) => true,
        }
    }

    pub fn entity(&self) -> Entity {
        match self {
            Node::Branch(entity) => *entity,
            Node::Leaf(entity) => *entity,
        }
    }
}

#[derive(Component, Copy, Clone)]
pub struct BranchNode {
    pub x: Fixed,
    pub y: Fixed,
    pub dx: Fixed,
    pub dy: Fixed,
    pub right_bounding_box: BoundingBox,
    pub left_bounding_box: BoundingBox,
    pub right_child: Node,
    pub left_child: Node,
}

impl BranchNode {
    pub fn is_on_back(&self, x: Fixed, y: Fixed) -> bool {
        let dx = x - self.x;
        let dy = y - self.y;

        let left = (self.dy >> Fixed::FRAC_NBITS) * dx;
        let right = dy * (self.dx >> Fixed::FRAC_NBITS);

        right >= left
    }
}
