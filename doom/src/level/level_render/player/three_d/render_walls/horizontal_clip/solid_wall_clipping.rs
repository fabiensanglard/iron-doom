use bevy::prelude::Resource;
use derive_more::{AsRef, Deref, DerefMut, Index, IndexMut, IntoIterator};

use engine_core::video_system::SCREEN_WIDTH;

#[derive(Resource, Default, AsRef, Deref, DerefMut, IntoIterator, Index, IndexMut)]
#[into_iterator(owned, ref, ref_mut)]
pub struct HorizontalSpace(Vec<SegmentFragment>);

#[derive(Debug, Clone, Copy)]
pub struct SegmentFragment {
    start: u32,
    end: u32,
}

impl HorizontalSpace {
    fn is_fully_occluded(&self) -> bool {
        if self.len() != 1 {
            return false;
        }
        let solid_segment = &self[0];
        solid_segment.start == 0 && solid_segment.end == SCREEN_WIDTH
    }

    fn find_adjacent_fragment(&self, fragment: SegmentFragment) -> Option<usize> {
        self.iter().position(|frag| frag.end + 1 >= fragment.start)
    }

    pub fn clip_fragment(&mut self, fragment: SegmentFragment) -> Vec<SegmentFragment> {
        // If the screen is already completely filled, then return nothing.
        if self.is_fully_occluded() {
            return vec![];
        }
        // Invalid fragment.
        if fragment.end < fragment.start
            || fragment.start >= SCREEN_WIDTH
            || fragment.end >= SCREEN_WIDTH
        {
            return vec![];
        }
        if self.is_empty() {
            self.push(fragment);
            return vec![fragment];
        }
        let Some(adj_range_idx) = self.find_adjacent_fragment(fragment) else {
            // No adjacent fragment, so fragment must be far right.
            self.push(fragment);
            return vec![fragment];
        };
        let mut clip_right = self.clip_fragment_right(fragment, adj_range_idx);
        let clip_left = self.clip_fragment_left(fragment, adj_range_idx);
        clip_right.extend(clip_left);

        clip_right
    }

    #[inline]
    fn clip_fragment_right(
        &mut self,
        fragment: SegmentFragment,
        adj_range_idx: usize,
    ) -> Vec<SegmentFragment> {
        let adj_range = self[adj_range_idx];

        // Nothing to clip right.
        if fragment.start >= adj_range.start {
            return vec![];
        }
        // Nothing to clip nor merge.
        if fragment.end < adj_range.start - 1 {
            self.insert(adj_range_idx, fragment);
            return vec![fragment];
        }
        let clipped_fragment = SegmentFragment::new(fragment.start, adj_range.start - 1);
        self[adj_range_idx].start = fragment.start;

        vec![clipped_fragment]
    }

    #[inline]
    fn clip_fragment_left(
        &mut self,
        fragment: SegmentFragment,
        adj_range_idx: usize,
    ) -> Vec<SegmentFragment> {
        let mut draw_points = Vec::new();
        let mut adj_range = self[adj_range_idx];

        // Nothing to clip left
        if fragment.end <= adj_range.end {
            return draw_points;
        }

        let len = self.len();
        let mut next_idx = adj_range_idx;

        for (range_idx, next_range_idx) in (adj_range_idx..len).zip(adj_range_idx + 1..len) {
            let seg_range = self[range_idx];
            let next_seg_range = self[next_range_idx];

            if fragment.end < next_seg_range.start - 1 {
                break;
            }

            next_idx = next_range_idx;
            draw_points.push(SegmentFragment::new(
                seg_range.end + 1,
                next_seg_range.start - 1,
            ));

            if fragment.end <= next_seg_range.end {
                adj_range.end = next_seg_range.end;
                self[adj_range_idx] = adj_range;
                if next_range_idx != adj_range_idx {
                    // Delete a range of walls
                    self.drain(adj_range_idx + 1..next_range_idx + 1);
                }
                return draw_points;
            }
        }

        let next_range = self[next_idx];
        draw_points.push(SegmentFragment::new(next_range.end + 1, fragment.end));
        adj_range.end = fragment.end;
        self[adj_range_idx] = adj_range;

        if next_idx != adj_range_idx {
            // Delete a range of walls
            self.drain(adj_range_idx + 1..next_idx + 1);
        }

        draw_points
    }

    pub fn clip_pass_fragment(&mut self, fragment: SegmentFragment) -> Vec<SegmentFragment> {
        // If the screen is already completely filled, then return nothing.
        if self.is_fully_occluded() {
            return vec![];
        }
        // Invalid fragment.
        if fragment.end < fragment.start
            || fragment.start >= SCREEN_WIDTH
            || fragment.end >= SCREEN_WIDTH
        {
            return vec![];
        }
        if self.is_empty() {
            return vec![fragment];
        }
        let Some(adj_range_idx) = self.find_adjacent_fragment(fragment) else {
            // No adjacent fragment, so fragment must be far right.
            return vec![fragment];
        };
        let mut clip_right = self.clip_pass_fragment_right(fragment, adj_range_idx);
        let clip_left = self.clip_pass_fragment_left(fragment, adj_range_idx);
        clip_right.extend(clip_left);

        clip_right
    }

    #[inline]
    fn clip_pass_fragment_right(
        &mut self,
        fragment: SegmentFragment,
        adj_range_idx: usize,
    ) -> Vec<SegmentFragment> {
        let adj_range = self[adj_range_idx];

        // Nothing to clip right.
        if fragment.start >= adj_range.start {
            return vec![];
        }
        // Nothing to clip nor merge.
        if fragment.end < adj_range.start - 1 {
            return vec![fragment];
        }
        let clipped_fragment = SegmentFragment::new(fragment.start, adj_range.start - 1);

        vec![clipped_fragment]
    }

    #[inline]
    fn clip_pass_fragment_left(
        &mut self,
        fragment: SegmentFragment,
        adj_range_idx: usize,
    ) -> Vec<SegmentFragment> {
        let mut draw_points = Vec::new();
        let adj_range = self[adj_range_idx];

        // Nothing to clip left
        if fragment.end <= adj_range.end {
            return draw_points;
        }

        let len = self.len();
        let mut next_idx = adj_range_idx;

        for (range_idx, next_range_idx) in (adj_range_idx..len).zip(adj_range_idx + 1..len) {
            let seg_range = self[range_idx];
            let next_seg_range = self[next_range_idx];

            if fragment.end < next_seg_range.start - 1 {
                break;
            }

            next_idx = next_range_idx;
            draw_points.push(SegmentFragment::new(
                seg_range.end + 1,
                next_seg_range.start - 1,
            ));

            if fragment.end <= next_seg_range.end {
                return draw_points;
            }
        }

        let next_range = self[next_idx];
        draw_points.push(SegmentFragment::new(next_range.end + 1, fragment.end));

        draw_points
    }
}

impl SegmentFragment {
    pub fn new(start: u32, end: u32) -> Self {
        Self { start, end }
    }

    pub fn start(&self) -> u32 {
        self.start
    }

    pub fn end(&self) -> u32 {
        self.end
    }
}
