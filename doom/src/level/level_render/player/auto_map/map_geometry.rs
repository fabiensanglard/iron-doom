use engine_core::fixed_point::Fixed;
use engine_core::geometry::{Angle, Vertex};

#[derive(Default, Copy, Clone)]
pub struct MapPoint {
    pub x: Fixed,
    pub y: Fixed,
}

#[derive(Default, Copy, Clone)]
pub struct FramePoint {
    pub x: i32,
    pub y: i32,
}

#[derive(Default, Copy, Clone)]
pub struct MapLine {
    pub a: MapPoint,
    pub b: MapPoint,
}

#[derive(Default, Copy, Clone)]
pub struct FrameLine {
    pub a: FramePoint,
    pub b: FramePoint,
}

impl From<Vertex> for MapPoint {
    fn from(vertex: Vertex) -> Self {
        Self {
            x: vertex.x,
            y: vertex.y,
        }
    }
}

impl MapPoint {
    pub fn new(x: Fixed, y: Fixed) -> Self {
        Self { x, y }
    }

    pub fn add(&mut self, x: Fixed, y: Fixed) {
        self.x = self.x.wrapping_add(x);
        self.y = self.y.wrapping_add(y);
    }

    pub fn scale(&mut self, scale: Fixed) {
        self.x = self.x.wrapping_mul(scale);
        self.y = self.y.wrapping_mul(scale);
    }

    pub fn rotate(&mut self, angle: Angle) {
        // If you are confused how this works, check out this link:
        // https://en.wikipedia.org/wiki/Rotation_(mathematics)#Two_dimensions

        let old_x = self.x;
        let old_y = self.y;

        self.x = (old_x * angle.cos()) - (old_y * angle.sin());
        self.y = (old_x * angle.sin()) + (old_y * angle.cos());
    }
}

impl MapLine {
    pub fn new(x1: Fixed, y1: Fixed, x2: Fixed, y2: Fixed) -> Self {
        Self {
            a: MapPoint::new(x1, y1),
            b: MapPoint::new(x2, y2),
        }
    }
}

impl From<(Vertex, Vertex)> for MapLine {
    fn from((v1, v2): (Vertex, Vertex)) -> Self {
        Self {
            a: v1.into(),
            b: v2.into(),
        }
    }
}
