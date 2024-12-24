use std::rc::Rc;

use bevy::prelude::{NonSend, ResMut};

use engine_core::command_line::CommandLine;

use super::Doom;
use crate::string_consts::D_DEVSTR;

pub fn setup(cli: NonSend<Rc<CommandLine>>, mut doom: ResMut<Doom>) {
    doom.death_match = death_match(&cli);
    doom.dev_parm = cli.dev_mode();
    doom.fast_parm = cli.fast();
    doom.no_monsters = cli.disable_monsters();
    doom.respawn_parm = cli.respawn();

    if doom.dev_parm {
        print!("{}", D_DEVSTR);
    }
}

fn death_match(cli: &CommandLine) -> i32 {
    if cli.death_match() {
        1
    } else if cli.alt_death() {
        2
    } else {
        0
    }
}
