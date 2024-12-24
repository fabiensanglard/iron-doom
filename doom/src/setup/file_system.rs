use std::path::Path;
use std::rc::Rc;

use bevy::prelude::{NonSend, NonSendMut, ResMut};

use engine_core::command_line::CommandLine;
use engine_core::file_system as engine_file_sys;
use engine_core::file_system::{FileSystem, GameMission, DOOM_MISSIONS};
use engine_core::wad_system::WadSystem;

use super::Doom;

pub fn setup(
    cli: NonSend<Rc<CommandLine>>,
    mut file_sys: NonSendMut<FileSystem>,
    mut wad_sys: NonSendMut<WadSystem>,
    mut doom: ResMut<Doom>,
) -> Result<(), String> {
    println!("V_Init: allocate screens.");
    println!("M_LoadDefaults: Load system defaults.");

    println!("W_Init: Init WADfiles.");

    set_config_dir(&cli, &mut file_sys)?;
    find_iwad(&file_sys, &mut doom)?;
    add_file(&mut wad_sys, doom.iwad_file())?;
    set_save_dir(&file_sys, &mut doom)?;

    Ok(())
}

fn set_config_dir(cli: &CommandLine, file_sys: &mut FileSystem) -> Result<(), String> {
    // Save configuration data and savegames in c:\doomdata,
    // allowing play from CD.
    #[cfg(windows)]
    if cli.cdrom() {
        file_sys.set_config_dir("c:\\doomdata\\")?;
    }

    Ok(())
}

fn find_iwad(file_sys: &FileSystem, doom: &mut Doom) -> Result<(), String> {
    let (iwad_file, game_mission) = file_sys.find_iwad(DOOM_MISSIONS)?;
    let iwad_file = iwad_file.ok_or({
        "Game mode indeterminate.  No IWAD file was found.  Try\n\
                 specifying one with the '--iwad' command line parameter.\n"
    })?;

    doom.iwad_file = iwad_file;
    doom.game_mission = game_mission;

    Ok(())
}

fn add_file(wad_sys: &mut WadSystem, file: &Path) -> Result<(), String> {
    println!(" adding {file:?}");

    wad_sys.add_file(file)?;

    wad_sys.check_correct_iwad(GameMission::Doom)
}

fn set_save_dir(file_sys: &FileSystem, doom: &mut Doom) -> Result<(), String> {
    let iwad_name = engine_file_sys::save_game_iwad_name(doom.game_mission, doom.game_variant);
    doom.save_game_dir = file_sys.get_save_game_dir(&iwad_name)?;
    engine_core::app::print_startup_banner("The Ultimate DOOM");

    Ok(())
}
