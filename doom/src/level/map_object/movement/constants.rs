use engine_core::fixed_point::Fixed;
use engine_core::geometry::Angle;

pub(super) const MAX_MOVE: i8 = 50;
pub const MAX_MOMENTUM: Fixed = Fixed::lit("30");
pub const MOVE_SCALE: i32 = 2048;
pub(super) const FRICTION: Fixed = Fixed::lit("0.90625");
pub const STOP_SPEED: Fixed = Fixed::lit("0.0625");

pub(super) const MOVE_FORWARD: i8 = 25;
pub(super) const RUN_FORWARD: i8 = 50;

pub(super) const STRAFE: i8 = 24;
pub(super) const FAST_STRAFE: i8 = 40;

pub(super) const SLOW_TURN_TICS: u32 = 6;
pub(super) const SLOW_TURN: Angle = Angle::new(320);
pub(super) const TURN: Angle = Angle::new(640);
pub(super) const FAST_TURN: Angle = Angle::new(1280);
