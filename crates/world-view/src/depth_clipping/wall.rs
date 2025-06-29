use crate::depth_clipping::{DepthClippingStrategy, ScreenOcclusion, SegmentFragment};

#[derive(Copy, Clone)]
pub struct WallClipStrategy;

impl WallClipStrategy {
    #[inline]
    fn clip_fragment_right(
        &self,
        screen_occlusion: &mut ScreenOcclusion,
        fragment: SegmentFragment,
        adj_range_idx: usize,
    ) -> Vec<SegmentFragment> {
        let adj_range = screen_occlusion[adj_range_idx];

        // Nothing to clip right.
        if fragment.start >= adj_range.start {
            return vec![];
        }
        // Nothing to clip nor merge.
        if fragment.end < adj_range.start - 1 {
            screen_occlusion.insert(adj_range_idx, fragment);
            return vec![fragment];
        }
        let clipped_fragment = SegmentFragment::new(fragment.start, adj_range.start - 1);
        screen_occlusion[adj_range_idx].start = fragment.start;

        vec![clipped_fragment]
    }

    #[inline]
    fn clip_fragment_left(
        &self,
        screen_occlusion: &mut ScreenOcclusion,
        fragment: SegmentFragment,
        adj_range_idx: usize,
    ) -> Vec<SegmentFragment> {
        let mut draw_points = Vec::new();
        let mut adj_range = screen_occlusion[adj_range_idx];

        // Nothing to clip left
        if fragment.end <= adj_range.end {
            return draw_points;
        }

        let len = screen_occlusion.len();
        let mut next_idx = adj_range_idx;

        for (range_idx, next_range_idx) in (adj_range_idx..len).zip(adj_range_idx + 1..len) {
            let seg_range = screen_occlusion[range_idx];
            let next_seg_range = screen_occlusion[next_range_idx];

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
                screen_occlusion[adj_range_idx] = adj_range;
                if next_range_idx != adj_range_idx {
                    // Delete a range of walls
                    screen_occlusion.drain(adj_range_idx + 1..next_range_idx + 1);
                }
                return draw_points;
            }
        }

        let next_range = screen_occlusion[next_idx];
        draw_points.push(SegmentFragment::new(next_range.end + 1, fragment.end));
        adj_range.end = fragment.end;
        screen_occlusion[adj_range_idx] = adj_range;

        if next_idx != adj_range_idx {
            // Delete a range of walls
            screen_occlusion.drain(adj_range_idx + 1..next_idx + 1);
        }

        draw_points
    }
}

impl DepthClippingStrategy for WallClipStrategy {
    fn clip(
        &self,
        screen_occlusion: &mut ScreenOcclusion,
        fragment: SegmentFragment,
    ) -> Vec<SegmentFragment> {
        // If the screen is already completely filled, then return nothing.
        if screen_occlusion.is_fully_occluded() {
            return vec![];
        }
        // Invalid fragment.
        if fragment.end < fragment.start || fragment.start >= 320 || fragment.end >= 320 {
            return vec![];
        }
        if screen_occlusion.is_empty() {
            screen_occlusion.push(fragment);
            return vec![fragment];
        }
        let Some(adj_range_idx) = screen_occlusion.find_adjacent_fragment(fragment) else {
            // No adjacent fragment, so fragment must be far right.
            screen_occlusion.push(fragment);
            return vec![fragment];
        };
        let mut clip_right = self.clip_fragment_right(screen_occlusion, fragment, adj_range_idx);
        let clip_left = self.clip_fragment_left(screen_occlusion, fragment, adj_range_idx);
        clip_right.extend(clip_left);

        clip_right
    }
}
