use std::io;
use std::io::IsTerminal;
use std::rc::Rc;

use bevy::app::{App, Plugin, PluginsState, ScheduleRunnerPlugin};
use bevy::ecs::event::ManualEventReader;
use bevy::input::InputPlugin as BevyInputPlugin;
use bevy::prelude::*;
use bevy::state::app::StatesPlugin as BevyStatesPlugin;
use bevy::tasks::tick_global_task_pools_on_main_thread;
use sdl2::messagebox::MessageBoxFlag;

use crate::command_line::{CommandLine, CommandLinePlugin};
use crate::file_system::FileSystemPlugin;
use crate::input::InputSystem;
use crate::random::RandomNumberPlugin;
use crate::video_system::VideoPlugin;
use crate::wad_system::WadPlugin;

pub const TIC_RATE: f64 = 35.0;

#[derive(Event, Default)]
pub struct EngineExit;

#[derive(Event)]
pub struct EngineExitError {
    message: String,
}

pub struct EnginePlugin;

impl Plugin for EnginePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            MinimalPlugins.build().disable::<ScheduleRunnerPlugin>(),
            BevyInputPlugin,
            BevyStatesPlugin,
            CommandLinePlugin,
            VideoPlugin,
            FileSystemPlugin,
            WadPlugin,
            RandomNumberPlugin,
        ))
        .insert_resource(Time::<Fixed>::from_hz(TIC_RATE))
        .add_event::<EngineExit>()
        .add_event::<EngineExitError>()
        .set_runner(engine_runner);
    }
}

fn engine_runner(mut app: App) -> AppExit {
    let mut input_system = InputSystem::init().expect("Failed to initialize InputSystem");

    'running: loop {
        input_system.handle_events(&mut app);
        update_app(&mut app);

        if should_exit(&app) {
            break 'running;
        }
    }

    AppExit::Success
}

fn update_app(app: &mut App) {
    if app.plugins_state() != PluginsState::Cleaned {
        if app.plugins_state() != PluginsState::Ready {
            #[cfg(not(target_arch = "wasm32"))]
            tick_global_task_pools_on_main_thread();
        } else {
            app.finish();
            app.cleanup();
        }
    }

    app.update();
}

#[inline]
fn should_exit(app: &App) -> bool {
    let mut exit_error_reader = ManualEventReader::<EngineExitError>::default();
    if let Some(exit_error_events) = app.world().get_resource::<Events<EngineExitError>>() {
        if let Some(error_event) = exit_error_reader.read(exit_error_events).next() {
            let cli = app.world().non_send_resource::<Rc<CommandLine>>();
            show_error(cli, &error_event.message);
            return true;
        }
    }

    let mut exit_reader = ManualEventReader::<EngineExit>::default();
    if let Some(exit_events) = app.world().get_resource::<Events<EngineExit>>() {
        return exit_reader.read(exit_events).next().is_some();
    }

    false
}

#[inline]
fn show_error(cli: &CommandLine, message: &str) {
    eprint!("{message}\n\n");

    // Pop up a GUI dialog box to show the error message, if the
    // game was not run from the console (and the user will
    // therefore be unable to otherwise see the message).
    if !cli.disable_gui() && !io::stdout().is_terminal() {
        let name = env!("WORKSPACE_NAME");
        let version = env!("WORKSPACE_VERSION");

        sdl2::messagebox::show_simple_message_box(
            MessageBoxFlag::ERROR,
            &format!("{name} {version}"),
            message,
            None,
        )
        .unwrap();
    }
}

pub fn exit_error<T, R: Into<String>>(
    In(result): In<Result<T, R>>,
    mut exit_engine_error_events: EventWriter<EngineExitError>,
) {
    let Err(message) = result else {
        return;
    };
    let message = message.into();

    exit_engine_error_events.send(EngineExitError { message });
}

#[inline]
pub fn print_startup_banner(game_description: &str) {
    print_divider();
    print_banner(game_description);
    print_divider();

    let project_name = env!("WORKSPACE_NAME");
    let msg = "is free software, covered by the GNU General Public\n \
            License.  There is NO warranty; not even for MERCHANTABILITY or FITNESS\n \
            FOR A PARTICULAR PURPOSE. You are welcome to change and distribute\n \
            copies under certain conditions. See the source for more information.";
    println!(" {project_name} {msg}");

    print_divider();
}

#[inline]
pub fn print_divider() {
    for _ in 0..75 {
        print!("=");
    }

    println!();
}

#[inline]
pub fn print_banner(banner: &str) {
    // Every two chars, we remove one space.
    // This gives a nice layout.
    let len = banner.len() / 2;
    let spaces = if len <= 35 { 35 - len } else { 0 };

    println!("{:spaces$}{}", "", banner);
}
