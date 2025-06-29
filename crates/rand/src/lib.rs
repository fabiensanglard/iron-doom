use bevy::prelude::*;
use table::RAND_TABLE;

// Re-exports from rand
pub use rand::prelude::*;

mod table;

#[derive(Default)]
pub struct RandPlugin;

impl Plugin for RandPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Rand>();
    }
}

#[derive(Resource, Default)]
pub struct Rand(usize);

impl RngCore for Rand {
    #[inline]
    fn next_u32(&mut self) -> u32 {
        self.0 = (self.0 + 1) % RAND_TABLE.len();
        u32::from(RAND_TABLE[self.0])
    }

    #[inline]
    fn next_u64(&mut self) -> u64 {
        u64::from(self.next_u32())
    }

    /// Implement `fill_bytes` via `next_u64` and `next_u32`, little-endian order.
    ///
    /// The fastest way to fill a slice is usually to work as long as possible with
    /// integers. That is why this method mostly uses `next_u64`, and only when
    /// there are 4 or less bytes remaining at the end of the slice it uses
    /// `next_u32` once.
    fn fill_bytes(&mut self, dst: &mut [u8]) {
        let mut left = dst;
        while left.len() >= 8 {
            let (l, r) = { left }.split_at_mut(8);
            left = r;
            let chunk: [u8; 8] = self.next_u64().to_le_bytes();
            l.copy_from_slice(&chunk);
        }
        let n = left.len();
        if n > 4 {
            let chunk: [u8; 8] = self.next_u64().to_le_bytes();
            left.copy_from_slice(&chunk[..n]);
        } else if n > 0 {
            let chunk: [u8; 4] = self.next_u32().to_le_bytes();
            left.copy_from_slice(&chunk[..n]);
        }
    }
}
