use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use derive_more::{AsRef, Deref, DerefMut, Index, IndexMut, IntoIterator};
use level::prelude::{LineSegment, Sector, SideDef};
use portal::PortalClipStrategy;
use wall::WallClipStrategy;

mod portal;
mod wall;

trait DepthClippingStrategy: Send + Sync + 'static {
    fn clip(
        &self,
        screen_occlusion: &mut ScreenOcclusion,
        fragment: SegmentFragment,
    ) -> Vec<SegmentFragment>;
}

#[derive(SystemParam)]
pub struct DepthClipping<'w, 's> {
    screen_occlusion: ResMut<'w, ScreenOcclusion>,
    sector_query: Query<'w, 's, &'static Sector>,
    side_query: Query<'w, 's, &'static SideDef>,
}

impl DepthClipping<'_, '_> {
    pub fn reset(&mut self) {
        self.screen_occlusion.clear();
    }

    pub fn is_fully_occluded(&self) -> bool {
        self.screen_occlusion.is_fully_occluded()
    }

    pub fn clip(&mut self, segment: &LineSegment, x1: usize, x2: usize) -> Vec<SegmentFragment> {
        let Some(strategy) = self.select_clip_strategy(segment) else {
            return vec![];
        };
        let fragment = SegmentFragment::new(x1, x2);
        strategy.clip(&mut self.screen_occlusion, fragment)
    }

    fn select_clip_strategy(
        &self,
        segment: &LineSegment,
    ) -> Option<Box<dyn DepthClippingStrategy>> {
        match segment {
            LineSegment::Wall(_) => Some(Box::new(WallClipStrategy)),
            LineSegment::Portal(portal) => {
                let front_sec = portal.front_sector;
                let back_sec = portal.back_sector;
                let line_side = portal.side;

                let front_sec = self.sector_query.get(*front_sec).unwrap();
                let back_sec = self.sector_query.get(*back_sec).unwrap();

                if self.is_closed_door(front_sec, back_sec) {
                    return Some(Box::new(WallClipStrategy));
                }
                if self.is_window(front_sec, back_sec) {
                    return Some(Box::new(PortalClipStrategy));
                }
                let line_side = self.side_query.get(*line_side).unwrap();
                if self.is_empty_line(front_sec, back_sec, line_side) {
                    return None;
                }
                Some(Box::new(PortalClipStrategy))
            }
        }
    }

    fn is_closed_door(&self, front_sec: &Sector, back_sec: &Sector) -> bool {
        back_sec.ceiling_height <= front_sec.floor_height
            || back_sec.floor_height >= front_sec.ceiling_height
    }

    fn is_window(&self, front_sec: &Sector, back_sec: &Sector) -> bool {
        back_sec.ceiling_height != front_sec.ceiling_height
            || back_sec.floor_height != front_sec.floor_height
    }

    fn is_empty_line(&self, front_sec: &Sector, back_sec: &Sector, line_side: &SideDef) -> bool {
        back_sec.ceiling_tex == front_sec.ceiling_tex
            && back_sec.floor_tex == front_sec.floor_tex
            && back_sec.light_level == front_sec.light_level
            && line_side.middle_texture == 0
    }
}

#[derive(Resource, Default, AsRef, Deref, DerefMut, IntoIterator, Index, IndexMut)]
pub struct ScreenOcclusion(Vec<SegmentFragment>);

impl ScreenOcclusion {
    fn is_fully_occluded(&self) -> bool {
        if self.len() != 1 {
            return false;
        }
        let solid_segment = &self[0];
        solid_segment.start == 0 && solid_segment.end == 320
    }

    fn find_adjacent_fragment(&self, fragment: SegmentFragment) -> Option<usize> {
        self.iter().position(|frag| frag.end + 1 >= fragment.start)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct SegmentFragment {
    pub start: usize,
    pub end: usize,
}
impl SegmentFragment {
    fn new(start: usize, end: usize) -> Self {
        Self { start, end }
    }
}
