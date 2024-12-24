use crate::fixed_point::Fixed;
use crate::geometry::Vertex;

pub mod app;
pub mod command_line;
pub mod file_system;
pub mod fixed_point;
pub mod geometry;
mod input;
pub mod random;
pub mod sys;
pub mod video_system;
pub mod wad_system;

#[derive(Debug, Clone, Copy)]
pub enum Skill {
    NoItem = -1,
    Baby,
    Easy,
    Medium,
    Hard,
    Nightmare,
}

impl From<i32> for Skill {
    fn from(value: i32) -> Self {
        match value {
            0 => Skill::Baby,
            1 => Skill::Easy,
            2 => Skill::Medium,
            3 => Skill::Hard,
            4 => Skill::Nightmare,
            _ => {
                if value < 0 {
                    Skill::NoItem
                } else {
                    Skill::Nightmare
                }
            }
        }
    }
}

#[derive(Default, Copy, Clone)]
pub struct BoundingBox {
    top: Fixed,
    bottom: Fixed,
    left: Fixed,
    right: Fixed,
}

impl BoundingBox {
    pub fn new(top: Fixed, bottom: Fixed, left: Fixed, right: Fixed) -> Self {
        Self {
            top,
            bottom,
            left,
            right,
        }
    }

    pub fn add(&mut self, x: Fixed, y: Fixed) {
        if x < self.left {
            self.left = x;
        } else if x > self.right {
            self.right = x;
        }

        if y < self.bottom {
            self.bottom = y;
        } else if y > self.top {
            self.top = y;
        }
    }

    pub fn clear(&mut self) {
        self.top = Fixed::MIN;
        self.right = Fixed::MIN;
        self.bottom = Fixed::MAX;
        self.bottom = Fixed::MAX;
    }
}

impl From<(Vertex, Vertex)> for BoundingBox {
    fn from((v1, v2): (Vertex, Vertex)) -> Self {
        let pos_x = v1.x < v2.x;
        let pos_y = v1.y < v2.y;

        let left = if pos_x { v1.x } else { v2.x };
        let right = if pos_x { v2.x } else { v1.x };
        let top = if pos_y { v1.y } else { v2.y };
        let bottom = if pos_y { v2.y } else { v1.y };

        BoundingBox::new(top, bottom, left, right)
    }
}
