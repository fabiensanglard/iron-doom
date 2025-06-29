use bevy::{input::keyboard::KeyboardInput, prelude::*};
use sdl2::{event::Event, EventPump};

mod keyboard;

pub struct InputPlugin;

impl Plugin for InputPlugin {
    fn build(&self, app: &mut App) {
        app.init_non_send_resource::<InputSystem>()
            .add_systems(PreUpdate, dispatch_events);
    }
}

fn dispatch_events(
    mut input_system: NonSendMut<InputSystem>,
    mut exit_events: EventWriter<AppExit>,
    mut keyboard_events: EventWriter<KeyboardInput>,
) {
    for event in input_system.0.poll_iter() {
        match event {
            Event::Quit { .. } => {
                exit_events.send(AppExit::Success);
            }
            Event::KeyDown {
                scancode,
                keycode,
                repeat,
                ..
            } => {
                let bevy_event = keyboard::convert_key_down(keycode, scancode, repeat);
                keyboard_events.send(bevy_event);
            }
            Event::KeyUp {
                scancode,
                keycode,
                repeat,
                ..
            } => {
                let bevy_event = keyboard::convert_key_up(keycode, scancode, repeat);
                keyboard_events.send(bevy_event);
            }
            _ => (),
        }
    }
}

struct InputSystem(EventPump);

impl FromWorld for InputSystem {
    fn from_world(_world: &mut World) -> Self {
        let sdl_ctx = sdl2::init().unwrap();
        let event_pump = sdl_ctx.event_pump().unwrap();

        InputSystem(event_pump)
    }
}
