// On Windows, detach console when compiling in release mode.
// This attribute is ignored on non-Windows targets.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::env;
use std::rc::Rc;

use bevy::prelude::*;

use engine_core::app::{exit_error, EnginePlugin};
use engine_core::command_line::CommandLine;
use engine_core::video_system::VideoSystem;
use engine_core::wad_system::WadSystem;
use engine_core::Skill;

use crate::defs::MAX_PLAYERS;
use crate::level::{LevelPlugin, LoadLevel};
use crate::render::{ChangeRenderState, RenderPlugin};
use crate::setup::{Doom, SetupPlugin};

mod defs;
mod level;
mod render;
mod setup;
#[allow(unused)]
mod string_consts;
mod utils;

#[derive(States, Debug, Hash, Eq, PartialEq, Clone, Copy, Default)]
enum GameState {
    DemoScreen,
    Finale,
    Intermission,
    Level,
    #[default]
    Setup,
}

fn main() {
    App::new()
        .add_plugins((EnginePlugin, SetupPlugin, RenderPlugin, LevelPlugin))
        .init_state::<GameState>()
        .add_systems(PostStartup, doom_main.pipe(exit_error))
        .add_systems(Update, change_level)
        .run();
}

fn doom_main(
    cli: NonSend<Rc<CommandLine>>,
    mut load_level: EventWriter<LoadLevel>,
) -> Result<(), String> {
    print_banner();

    // Printing for historical reasons only. Iron Doom does not actually
    // use any custom allocator.
    println!("Z_Init: Init zone memory allocation daemon. ");

    if cli.dedicated() {
        println!("Dedicated server mode.");
    }

    if cli.disable_autoload() {
        println!("Auto loading disabled");
    };

    load_level.send(LoadLevel { episode: 1, map: 1 });

    Ok(())
}

fn print_banner() {
    let banner = format!("{} {}", env!("WORKSPACE_NAME"), env!("WORKSPACE_VERSION"));
    engine_core::app::print_banner(&banner);
}

fn change_level(keyboard_input: Res<ButtonInput<KeyCode>>, mut load_level: EventWriter<LoadLevel>) {
    if keyboard_input.pressed(KeyCode::Numpad1) {
        load_level.send(LoadLevel { episode: 1, map: 1 });
    } else if keyboard_input.pressed(KeyCode::Numpad2) {
        load_level.send(LoadLevel { episode: 1, map: 2 });
    } else if keyboard_input.pressed(KeyCode::Numpad3) {
        load_level.send(LoadLevel { episode: 1, map: 3 });
    } else if keyboard_input.pressed(KeyCode::Numpad4) {
        load_level.send(LoadLevel { episode: 1, map: 4 });
    } else if keyboard_input.pressed(KeyCode::Numpad5) {
        load_level.send(LoadLevel { episode: 2, map: 1 });
    } else if keyboard_input.pressed(KeyCode::Numpad6) {
        load_level.send(LoadLevel { episode: 3, map: 1 });
    } else if keyboard_input.pressed(KeyCode::Numpad7) {
        load_level.send(LoadLevel { episode: 4, map: 1 });
    }
}

#[allow(unused)]
fn load_demo(mut wad_sys: NonSendMut<WadSystem>, mut doom: ResMut<Doom>) {
    let demo_name = "demo1";
    let demo_lump = wad_sys
        .cache_lump_name(demo_name)
        .expect("Demo could not be loaded");

    let demo_version = demo_lump[0];
    let skill = demo_lump[1];
    let episode = demo_lump[2];
    let map = demo_lump[3];
    let deathmatch = demo_lump[4];
    let respawnparm = demo_lump[5];
    let fastparm = demo_lump[6];
    let nomonsters = demo_lump[7];
    let consoleplayer = demo_lump[8];

    for i in 0..MAX_PLAYERS as usize {
        doom.player_in_game[i] = demo_lump[i + 9] == 1;
    }

    g_init_new((skill as i32).into(), episode as u32, map as u32);
}

#[allow(unused)]
fn g_init_new(skill: Skill, episode: u32, map: u32) {}

#[allow(unused)]
fn wipe(
    mut video_sys: NonSendMut<VideoSystem>,
    mut wad_sys: NonSendMut<WadSystem>,
    mut render_state_event: EventWriter<ChangeRenderState>,
) {
    let buf = wad_sys
        .cache_lump_name("TITLEPIC")
        .expect("Not found TITLEPIC");
    let patch = buf
        .as_slice()
        .try_into()
        .expect("Failed to convert TITLEPIC lump data");

    video_sys.draw_patch(0, 0, &patch);

    render_state_event.send(ChangeRenderState::Wipe);
}
