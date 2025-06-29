use crate::sdl_window::SdlWindow;
use bevy::prelude::*;
use exit::Exit;
use game_state::conditions::in_exiting_state;
use sdl2::messagebox::{show_simple_message_box, MessageBoxFlag};
use std::io::IsTerminal;

pub struct ExitAppPlugin;

impl Plugin for ExitAppPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            PostUpdate,
            (close_window, handle_exit)
                .chain()
                .run_if(in_exiting_state().and(on_event::<Exit>)),
        );
    }
}

fn close_window(world: &mut World) {
    world.remove_non_send_resource::<SdlWindow>();
}

fn handle_exit(mut exit_events: EventReader<Exit>, mut app_exit: EventWriter<AppExit>) {
    let Some(exit_event) = exit_events.read().next() else {
        return;
    };
    match exit_event {
        Exit::Success => {
            app_exit.send(AppExit::Success);
        }
        Exit::Error(error) => {
            display_error(error);
            app_exit.send(AppExit::error());
        }
    }
}

/// Display the error to the user, either in the terminal or via a dialog box.
fn display_error(error: &anyhow::Error) {
    if std::io::stdout().is_terminal() {
        error!("{error:?}");
        return;
    }

    let flags = MessageBoxFlag::ERROR;
    let title = "Iron Doom";
    let message = error.root_cause().to_string();
    let show_message_result = show_simple_message_box(flags, title, &message, None);
    if let Err(error) = show_message_result {
        error!("{error}");
    }
}
