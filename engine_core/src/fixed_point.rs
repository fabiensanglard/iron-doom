use fixed::types::extra::U16;
use fixed::FixedI32;

pub const FRAC_BITS: u8 = 16;
pub const FRAC_UNIT: Fixed = Fixed::ONE;

pub type Fixed = FixedI32<U16>;
