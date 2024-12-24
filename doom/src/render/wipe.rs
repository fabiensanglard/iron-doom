use bevy::app::{App, Plugin};
use bevy::prelude::*;

use engine_core::random::Rand;
use engine_core::video_system::{VideoSystem, SCREEN_HEIGHT, SCREEN_WIDTH};

use crate::render::{ChangeRenderState, RenderState};

// Importing as usize to ease array manipulation.
const HEIGHT: usize = SCREEN_HEIGHT as usize;
const WIDTH: usize = SCREEN_WIDTH as usize;
const TOTAL_SCREEN_PIXELS: usize = HEIGHT * WIDTH;

pub struct WipePlugin;

impl Plugin for WipePlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<WipeState>()
            .init_resource::<WipeResource>()
            .add_systems(OnEnter(RenderState::Wipe), setup_wipe)
            .add_systems(
                FixedUpdate,
                wipe_do_melt
                    .run_if(in_state(WipeState::DoMelt))
                    .run_if(in_state(RenderState::Wipe)),
            );
    }
}

#[derive(States, Debug, Hash, Eq, PartialEq, Clone, Copy, Default)]
enum WipeState {
    DoMelt,
    #[default]
    Done,
}

#[derive(Resource)]
struct WipeResource {
    start_screen: [u8; TOTAL_SCREEN_PIXELS],
    end_screen: [u8; TOTAL_SCREEN_PIXELS],
    /// Each pixel column has an offset to separate between start and end screen.
    /// Negative values are used as time delay in order to create the wipe effect.
    column_offsets: [i32; WIDTH],
}

impl Default for WipeResource {
    fn default() -> Self {
        Self {
            start_screen: [0; TOTAL_SCREEN_PIXELS],
            end_screen: [0; TOTAL_SCREEN_PIXELS],
            column_offsets: [0; WIDTH],
        }
    }
}

fn setup_wipe(
    video_sys: NonSend<VideoSystem>,
    mut wipe: ResMut<WipeResource>,
    mut rand: ResMut<Rand>,
    mut wipe_state: ResMut<NextState<WipeState>>,
) {
    // Two consecutive columns have the same offset, i.e. column_offsets[0] == column_offsets[1],
    // column_offsets[2] == column_offsets[3] and so on.
    // That way, consecutive columns end up moving together which enhances the melt screen visual effect.
    // Original DOOM also does this, although the original implementation relies on casting
    // (*uint8_t) to (*uint16_t), which makes the code way less intuitive.
    video_sys.read_screen(&mut wipe.start_screen);
    let len = wipe.column_offsets.len();

    if len > 0 {
        wipe.column_offsets[0] = -(rand.get() % 16);
    }
    if len > 1 {
        wipe.column_offsets[1] = wipe.column_offsets[0];
    }

    for i in (2..len).step_by(2) {
        let rand_num = (rand.get() % 3) - 1;
        let mut offset = wipe.column_offsets[i - 1] + rand_num;

        if offset > 0 {
            offset = 0;
        } else if offset == -16 {
            offset = -15;
        }

        wipe.column_offsets[i] = offset;
        if i != len - 1 {
            wipe.column_offsets[i + 1] = offset;
        }
    }

    wipe_state.set(WipeState::DoMelt);
}

fn wipe_do_melt(
    mut video_sys: NonSendMut<VideoSystem>,
    mut wipe: ResMut<WipeResource>,
    mut wipe_state: ResMut<NextState<WipeState>>,
    mut render_state_event: EventWriter<ChangeRenderState>,
) {
    let mut done = true;

    for col_idx in 0..wipe.column_offsets.len() {
        let col_off = wipe.column_offsets[col_idx];

        if col_off < 0 {
            done = false;
            wipe.column_offsets[col_idx] += 1;
        } else if (col_off as usize) < HEIGHT {
            done = false;
            draw_pixel_column(&mut video_sys, &mut wipe, col_idx, col_off as usize);
        }
    }

    video_sys.update();

    if done {
        wipe_state.set(WipeState::Done);
        render_state_event.send(ChangeRenderState::Default);
    }
}

#[inline(always)]
fn draw_pixel_column(
    video_sys: &mut VideoSystem,
    wipe: &mut WipeResource,
    col_idx: usize,
    col_off: usize,
) {
    let video_buf = video_sys.frame_buf_mut();

    // Calculate how much the pixel column will move
    let mut dy = if col_off < 16 { col_off + 1 } else { 8 };
    if col_off + dy >= HEIGHT {
        dy = HEIGHT - col_off;
    }

    // Draw end screen
    // Note that only the displacement "dy" needs to be drawn
    let idx = col_off * WIDTH + col_idx;
    let video_iter = video_buf.iter_mut().skip(idx).step_by(WIDTH);
    let end_screen_iter = wipe.end_screen.iter().skip(idx).step_by(WIDTH);
    for (dest_pixel, src_pixel) in video_iter.zip(end_screen_iter).take(dy) {
        *dest_pixel = *src_pixel;
    }

    // Update offset
    let col_off = col_off + dy;
    wipe.column_offsets[col_idx] = col_off.try_into().unwrap_or(i32::MAX);

    // Draw start screen
    // Because the start screen pixels are moving down, all of them which are
    // still visible need to be redrawn
    let video_iter = video_buf
        .iter_mut()
        .skip(col_off * WIDTH + col_idx)
        .step_by(WIDTH);
    let start_screen_iter = wipe.start_screen.iter().skip(col_idx).step_by(WIDTH);
    for (dest_pixel, src_pixel) in video_iter.zip(start_screen_iter) {
        *dest_pixel = *src_pixel;
    }
}
