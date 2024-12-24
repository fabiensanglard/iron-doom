use std::cmp;
use std::fmt::{Display, Formatter};

use paste::paste;

use tables::*;

use crate::fixed_point::Fixed;

mod tables;

macro_rules! impl_shift_ops {
    ($typ:ty, $trait:ident, $op_name:ident) => {
        impl<Rhs: TryInto<u32>> std::ops::$trait<Rhs> for $typ {
            type Output = Self;

            fn $op_name(self, rhs: Rhs) -> Self {
                if let Ok(val) = rhs.try_into() {
                    paste! {
                        Self(self.0.[<wrapping_ $op_name>](val))
                    }
                } else {
                    self
                }
            }
        }

        paste! {
            impl<Rhs: TryInto<u32>> std::ops::[<$trait Assign>]<Rhs> for $typ {
                fn [<$op_name _assign>](&mut self, rhs: Rhs) {
                    *self = std::ops::$trait::$op_name(*self, rhs);
                }
            }
        }
    };
}

macro_rules! impl_num_ops {
    ($typ:ty, $trait:ident, $op_name:ident) => {
        impl std::ops::$trait for $typ {
            type Output = Self;

            fn $op_name(self, other: Self) -> Self {
                paste! {
                    let val = self.0.[<wrapping_ $op_name>](other.0);
                }
                Self(val)
            }
        }

        paste! {
            impl std::ops::[<$trait Assign>] for $typ {
                fn [<$op_name _assign>](&mut self, rhs: Self) {
                    *self = std::ops::$trait::$op_name(*self, rhs);
                }
            }
        }
    };
}

#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub struct Angle(u32);

impl Angle {
    pub const ZERO: Self = Self::new(0);
    pub const DEG_1: Self = Self::new(0xb60b60);
    pub const DEG_45: Self = Self::new(0x20000000);
    pub const DEG_60: Self = Self::new(0x2aaaaaaa);
    pub const DEG_90: Self = Self::new(0x40000000);
    pub const DEG_180: Self = Self::new(0x80000000);
    pub const DEG_270: Self = Self::new(0xc0000000);
    pub const MIN: Self = Self::new(1);
    pub const MAX: Self = Self::new(u32::MAX);
    pub const TAN_STEP: Self = Self::new(1 << TAN_SHIFT);

    pub const fn new(val: u32) -> Self {
        Self(val)
    }

    pub const fn as_usize(self) -> usize {
        self.0 as usize
    }

    pub fn sin(self) -> Fixed {
        let x = self >> ANGLE_TO_FINE_SHIFT;

        Fixed::from_bits(FINE_SINE[x.as_usize()])
    }

    pub fn cos(self) -> Fixed {
        // cos(x) == sin(x + PI/2)
        let x = self >> ANGLE_TO_FINE_SHIFT;
        let shift = Angle::DEG_90 >> ANGLE_TO_FINE_SHIFT;

        Fixed::from_bits(FINE_SINE[(x + shift).as_usize()])
    }

    pub fn tan(mut self) -> Fixed {
        if self > Angle::DEG_90 && self < Angle::DEG_270 {
            self += Angle::DEG_180
        }
        self += Angle::DEG_90;
        self >>= TAN_SHIFT;
        // We can actually replace cmp::min with an if statement, as
        // the only time x >= FINE_TAN.len() is when calling this function
        // with ANG_90. However, I find using cmp::min to be more clear
        // and explicit.
        let idx = cmp::min(self.as_usize(), FINE_TAN.len() - 1);

        Fixed::from_bits(FINE_TAN[idx])
    }

    pub fn atan(num: Fixed, den: Fixed) -> Self {
        let tan = slope_div(num, den);
        Angle::new(TAN_TO_ANGLE[tan])
    }

    pub fn to_screen_space(&self, screen_width: u32, half_fov: Self) -> u32 {
        let tan = self.tan();

        if tan > 2 * Fixed::ONE {
            return 0;
        }
        if tan < -2 * Fixed::ONE {
            return screen_width;
        }

        let width_frac = Fixed::from_num(screen_width);
        let center_x = width_frac / 2;
        let focal_length = center_x / half_fov.tan();

        let t = focal_length * tan;
        let t = (center_x - t + Fixed::ONE - Fixed::DELTA).int();
        let t = t.to_num::<i32>();

        if t < 0 {
            return 0;
        }
        let t = t as u32;
        if t > screen_width {
            return screen_width;
        }
        
        t
    }

    pub fn from_screen_space(x: u32, screen_width: u32, half_fov: Self) -> Self {
        let fixed_width = Fixed::from_num(screen_width);
        let center_x = fixed_width / 2;

        let fixed_x = Fixed::from_num(x);
        let co = if fixed_x <= center_x {
            center_x - fixed_x
        } else {
            fixed_x - center_x
        };
        let focal_length = center_x / half_fov.tan();
        let mut ang = Angle::atan(co, focal_length);
        if fixed_x > center_x {
            ang = -ang;
        }

        let tan_step = Self::new(1 << TAN_SHIFT);
        while ang.to_screen_space(screen_width, half_fov) > x {
            ang += tan_step;
        }
        let mut before_angle = ang - tan_step;
        while before_angle.to_screen_space(screen_width, half_fov) <= x {
            ang = before_angle;
            before_angle -= tan_step;
        }

        ang >>= TAN_SHIFT;
        ang <<= TAN_SHIFT;
        
        ang
    }
}

fn slope_div(num: Fixed, den: Fixed) -> usize {
    let num = num.to_bits() as u32;
    let den = den.to_bits() as u32;

    if den < 512 {
        return SLOPE_RANGE;
    }
    let ans = (num << 3) / (den >> 8);

    cmp::min(ans as usize, SLOPE_RANGE)
}

impl Default for Angle {
    fn default() -> Self {
        Self::ZERO
    }
}

impl Display for Angle {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let scale = 360.0 / ((1u64 << u32::BITS) as f64);
        let deg = self.0 as f64 * scale;
        write!(f, "{deg}")
    }
}

impl_num_ops!(Angle, Add, add);
impl_num_ops!(Angle, Sub, sub);
impl_num_ops!(Angle, Mul, mul);
impl_num_ops!(Angle, Div, div);
impl_shift_ops!(Angle, Shr, shr);
impl_shift_ops!(Angle, Shl, shl);

impl std::ops::Neg for Angle {
    type Output = Self;

    fn neg(self) -> Self::Output {
        Angle::ZERO - self
    }
}
