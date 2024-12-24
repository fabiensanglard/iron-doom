use bevy::input::ButtonState;
use bevy::prelude::App;
use sdl2::event::Event;
use sdl2::EventPump;

use crate::app::EngineExit;

mod converters;

pub struct InputSystem {
    sdl_event_sys: EventPump,
}

impl InputSystem {
    pub fn init() -> Result<Self, String> {
        Ok(Self {
            sdl_event_sys: sdl2::init()?.event_pump()?,
        })
    }

    pub fn handle_event(&mut self, app: &mut App) {
        for event in self.sdl_event_sys.poll_iter() {
            match event {
                Event::Quit { .. } => {
                    app.world_mut().send_event(EngineExit);
                }
                Event::KeyDown {
                    scancode, keycode, ..
                } => {
                    let keyboard_event =
                        converters::convert_keyboard_event(keycode, scancode, ButtonState::Pressed);
                    app.world_mut().send_event(keyboard_event);
                }
                Event::KeyUp {
                    scancode, keycode, ..
                } => {
                    let keyboard_event = converters::convert_keyboard_event(
                        keycode,
                        scancode,
                        ButtonState::Released,
                    );
                    app.world_mut().send_event(keyboard_event);
                }
                _ => (),
            }
        }
    }
}
