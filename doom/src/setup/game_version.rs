use std::rc::Rc;

use bevy::prelude::{NonSend, NonSendMut, ResMut};

use engine_core::command_line::CommandLine;
use engine_core::file_system::{GameMission, GameMode, GameVariant, GameVersion};
use engine_core::wad_system::WadSystem;

use super::Doom;

pub fn setup(
    cli: NonSend<Rc<CommandLine>>,
    mut wad_sys: NonSendMut<WadSystem>,
    mut doom: ResMut<Doom>,
) -> Result<(), String> {
    identify_game_version(&cli, &wad_sys, &mut doom)?;
    determine_game_version(&cli, &mut wad_sys, &mut doom)?;
    validate_game_version(&mut doom);
    determine_game_variant(&wad_sys, &mut doom);

    Ok(())
}

fn identify_game_version(
    cli: &CommandLine,
    wad_sys: &WadSystem,
    doom: &mut Doom,
) -> Result<(), String> {
    let game_mission = try_identify_game_mission(doom.game_mission, wad_sys)?;

    let game_mode = identify_game_mode(game_mission, wad_sys);
    let game_mission = override_game_mission(game_mission, game_mode, cli);

    doom.game_mission = game_mission;
    doom.game_mode = game_mode;

    Ok(())
}

/// gamemission is set up by the find_iwad function. But if
/// we specify '--iwad', we have to identify using
/// IdentifyIWADByName.  However, if the iwad does not match
/// any known IWAD name, we may have a dilemma.  Try to
/// identify by its contents.
fn try_identify_game_mission(
    game_mission: GameMission,
    wad_sys: &WadSystem,
) -> Result<GameMission, String> {
    if game_mission != GameMission::None {
        return Ok(game_mission);
    }

    for lump in wad_sys.lump_info() {
        if lump.name().eq_ignore_ascii_case("MAP01") {
            return Ok(GameMission::Doom2);
        }
        if lump.name().eq_ignore_ascii_case("E1M1") {
            return Ok(GameMission::Doom);
        }
    }

    Err("Unknown or invalid IWAD file.".to_owned())
}

fn identify_game_mode(game_mission: GameMission, wad_sys: &WadSystem) -> GameMode {
    match game_mission.logical_mission() {
        GameMission::Doom => {
            if wad_sys.get_lump_by_name("E4M1").is_some() {
                return GameMode::Retail;
            }
            if wad_sys.get_lump_by_name("E3M1").is_some() {
                return GameMode::Registered;
            }

            GameMode::Shareware
        }
        _ => GameMode::Commercial,
    }
}

/// We can manually override the game mission that we got from the
/// IWAD detection code. This allows us to e.g. play Plutonia 2
/// with Freedoom and get the right level names.
fn override_game_mission(
    game_mission: GameMission,
    game_mode: GameMode,
    cli: &CommandLine,
) -> GameMission {
    match game_mode {
        // Doom 2 of some kind.
        GameMode::Commercial => {
            if let Some(pack) = cli.pack() {
                return pack.to_game_mission();
            }
            game_mission
        }
        _ => game_mission,
    }
}

fn determine_game_version(
    cli: &CommandLine,
    wad_sys: &mut WadSystem,
    doom: &mut Doom,
) -> Result<(), String> {
    if let Some(cli_version) = cli.game_version() {
        doom.game_version = cli_version.to_game_version();
        return Ok(());
    }

    match doom.game_mission {
        GameMission::Doom2 if doom.game_mode == GameMode::Commercial => {
            let game_version = check_demo_version(wad_sys)?.unwrap_or(GameVersion::Doom19);
            doom.game_version = game_version;
            return Ok(());
        }
        GameMission::PackChex => {
            doom.game_version = GameVersion::Chex;
            return Ok(());
        }
        GameMission::PackHacx => {
            doom.game_version = GameVersion::Hacx;
            return Ok(());
        }
        _ => {}
    }

    match doom.game_mode {
        GameMode::Shareware | GameMode::Registered => {
            let game_version = check_demo_version(wad_sys)?.unwrap_or(GameVersion::Doom19);
            doom.game_version = game_version;
            return Ok(());
        }
        GameMode::Retail => {
            doom.game_version = GameVersion::Ultimate;
            return Ok(());
        }
        GameMode::Commercial => {
            // Final Doom: tnt or plutonia
            // Defaults to emulating the first Final Doom executable,
            // which has the crash in the demo loop; however, having
            // this as the default should mean that it plays back
            // most demos correctly.
            doom.game_version = GameVersion::Final;
            return Ok(());
        }
        _ => {}
    }

    Ok(())
}

fn check_demo_version(wad_sys: &mut WadSystem) -> Result<Option<GameVersion>, String> {
    for i in 1..=3 {
        let demo_lump = format!("demo{i}");
        let demo_lump = wad_sys.cache_lump_name(&demo_lump)?;
        let demo_version = demo_lump[0];

        match demo_version {
            0..=4 => return Ok(Some(GameVersion::Doom12)),
            106 => return Ok(Some(GameVersion::Doom1666)),
            107 => return Ok(Some(GameVersion::Doom17)),
            108 => return Ok(Some(GameVersion::Doom18)),
            109 => return Ok(Some(GameVersion::Doom19)),
            _ => {}
        };
    }

    Ok(None)
}

fn validate_game_version(doom: &mut Doom) {
    // Deathmatch 2.0 did not exist until Doom v1.4
    let death_match = match doom.game_version {
        GameVersion::Doom12 if doom.death_match == 2 => 1,
        _ => 0,
    };

    // The original exe does not support retail - 4th episode not supported
    let game_mode = match doom.game_version {
        GameVersion::Doom12
        | GameVersion::Doom17
        | GameVersion::Doom18
        | GameVersion::Doom19
        | GameVersion::Hacx => GameMode::Registered,
        _ => doom.game_mode,
    };

    // EXEs prior to the Final Doom exes do not support Final Doom.
    let game_mission = match doom.game_version {
        GameVersion::Doom12
        | GameVersion::Doom17
        | GameVersion::Doom18
        | GameVersion::Doom19
        | GameVersion::Ultimate
        | GameVersion::Hacx => {
            if game_mode == GameMode::Commercial
                && (doom.game_mission == GameMission::PackTnt
                    || doom.game_mission == GameMission::PackPlut)
            {
                GameMission::Doom2
            } else {
                doom.game_mission
            }
        }
        _ => doom.game_mission,
    };

    doom.game_mode = game_mode;
    doom.game_mission = game_mission;
    doom.death_match = death_match;
}

fn determine_game_variant(wad_sys: &WadSystem, doom: &mut Doom) {
    let game_variant = if wad_sys.lump_exists("FREEDOOM") {
        if wad_sys.lump_exists("FREEDM") {
            GameVariant::FreeDm
        } else {
            GameVariant::FreeDoom
        }
    } else if wad_sys.lump_exists("FREEDM") {
        GameVariant::BfgEdition
    } else {
        doom.game_variant
    };

    doom.game_variant = game_variant;
}
