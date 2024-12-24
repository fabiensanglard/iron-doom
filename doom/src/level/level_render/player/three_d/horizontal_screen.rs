use bevy::prelude::Resource;
use rustc_hash::FxHashMap;

use engine_core::fixed_point::Fixed;
use engine_core::geometry::Angle;
use engine_core::video_system::SCREEN_WIDTH;

#[allow(unused)]
pub const FOV: Angle = Angle::DEG_90;
pub const HALF_FOV: Angle = Angle::DEG_45;

#[derive(Resource)]
pub struct ScreenLUT {
    angle_to_x: FxHashMap<Fixed, u32>,
    x_to_angle: FxHashMap<u32, Angle>,
}

impl ScreenLUT {
    pub fn angle_to_x(&self, angle: Angle) -> u32 {
        let tan = angle.tan();
        *self.angle_to_x
            .get(&tan)
            .expect("angle_to_x: LUT should map all possible tangent values!")
    }

    pub fn x_to_angle(&self, x: u32) -> Angle {
        *self.x_to_angle
            .get(&x)
            .expect("x_to_angle: trying to project outside screen!")
    }
}

impl Default for ScreenLUT {
    fn default() -> Self {
        let width = SCREEN_WIDTH;
        let angle_to_x = create_x_lut(width);
        let x_to_angle = create_angle_lut(width);

        Self {
            angle_to_x,
            x_to_angle,
        }
    }
}

fn create_x_lut(screen_width: u32) -> FxHashMap<Fixed, u32> {
    let mut lut = FxHashMap::default();
    let mut ang = Angle::DEG_270;
    let mut tan = ang.tan();
    let tan_max = Angle::DEG_90.tan();

    while tan <= tan_max {
        let x = ang.to_screen_space(screen_width, HALF_FOV);
        lut.insert(tan, x);
        if tan == tan_max {
            break;
        }
        ang += Angle::TAN_STEP;
        tan = ang.tan();
    }

    lut
}

fn create_angle_lut(screen_width: u32) -> FxHashMap<u32, Angle> {
    let mut lut = FxHashMap::default();
    for x in 0..screen_width {
        let angle = Angle::from_screen_space(x, screen_width, HALF_FOV);
        lut.insert(x, angle);
    }

    lut
}
