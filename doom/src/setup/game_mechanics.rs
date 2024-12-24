use std::rc::Rc;

use bevy::prelude::{NonSend, ResMut};

use engine_core::command_line::CommandLine;
use engine_core::fixed_point::Fixed;

use super::Doom;

pub fn setup(cli: NonSend<Rc<CommandLine>>, mut doom: ResMut<Doom>) {
    set_move(&cli, &mut doom);
}

fn set_move(cli: &CommandLine, doom: &mut Doom) {
    let Some(scale) = cli.turbo() else { return };

    println!("turbo scale: {scale}%");
    let scale = Fixed::from_bits(scale as i32);

    doom.forward_move = doom.forward_move.map(|val| val * scale / 100);
    doom.side_move = doom.side_move.map(|val| val * scale / 100);
}
