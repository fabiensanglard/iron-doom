use engine_core::video_system::{SCREEN_HEIGHT, SCREEN_WIDTH};

pub const ST_WIDTH: u32 = SCREEN_WIDTH;
pub const ST_HEIGHT: u32 = 32;
pub const ST_X: u32 = 0;
pub const ST_Y: u32 = SCREEN_HEIGHT - ST_HEIGHT;

pub const NUM_PAIN_FACES: u32 = 5;
pub const NUM_STRAIGHT_FACES: u32 = 3;
pub const NUM_TURN_FACES: u32 = 2;
pub const NUM_SPECIAL_FACES: u32 = 3;
pub const NUM_EXTRA_FACES: u32 = 2;
pub const NUM_FACE_STRIDE: u32 = NUM_STRAIGHT_FACES + NUM_TURN_FACES + NUM_SPECIAL_FACES;
pub const NUM_FACES: u32 = NUM_FACE_STRIDE * NUM_PAIN_FACES + NUM_EXTRA_FACES;
