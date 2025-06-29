use bevy::{app::PluginsState, prelude::*};
use common::Buffer;
use derive_more::{Deref, DerefMut};
use exit::ExitAppPlugin;
use input::InputPlugin;
use palette::PalettePlugin;
use sdl_window::SdlWindowPlugin;

mod exit;
mod input;
pub mod palette;
mod sdl_window;

const SCREEN_WIDTH: u32 = 320;
const SCREEN_HEIGHT: u32 = 200;

/// The screen height resolution used to emulate the CRT aspect ratio (4:3) for older video modes,
/// such as Doom's. For more context, refer to the Doom Wiki:
///
/// > Vanilla Doom ran only in a tweaked VGA "Mode Y" 320x200 video mode.
/// > On properly configured CRT monitors, which were the only widely available and inexpensive
/// > consumer display device for computers at the time, this video mode took up the entire screen,
/// > which had a 4:3 physical aspect ratio (the closest 4:3 resolution being 320x240). This meant
/// > that the 320x200 image, with a 16:10 logical ratio, was stretched vertically - each pixel was
/// > 20% taller than it was wide.
const SCREEN_HEIGHT_4_3: u32 = 240;

#[derive(Default)]
pub struct WindowPlugin;

impl Plugin for WindowPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ScreenBuffer>()
            .add_plugins((InputPlugin, PalettePlugin, SdlWindowPlugin, ExitAppPlugin))
            .set_runner(window_runner);
    }
}

fn window_runner(mut app: App) -> AppExit {
    if app.plugins_state() != PluginsState::Cleaned {
        while app.plugins_state() == PluginsState::Adding {
            #[cfg(not(target_arch = "wasm32"))]
            bevy::tasks::tick_global_task_pools_on_main_thread();
        }
        app.finish();
        app.cleanup();
    }

    loop {
        app.update();
        if let Some(exit) = app.should_exit() {
            return exit;
        }
    }
}

#[derive(Resource, Deref, DerefMut, Clone, Debug)]
pub struct ScreenBuffer(Buffer);

impl ScreenBuffer {
    pub fn copy_from(&mut self, src: &ScreenBuffer) {
        self.0.copy_from(&src.0);
    }
}

impl Default for ScreenBuffer {
    fn default() -> Self {
        let width = SCREEN_WIDTH as usize;
        let height = SCREEN_HEIGHT as usize;
        let buffer = Buffer::new(width, height);
        ScreenBuffer(buffer)
    }
}
