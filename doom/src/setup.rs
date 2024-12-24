use std::path::{Path, PathBuf};

use bevy::prelude::*;

use engine_core::app::exit_error;
use engine_core::file_system::{GameMission, GameMode, GameVariant, GameVersion};
use engine_core::fixed_point::Fixed;

use crate::defs::MAX_PLAYERS;

mod file_system;
mod game_mechanics;
mod game_params;
mod game_version;

pub struct SetupPlugin;

impl Plugin for SetupPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Doom>()
            .configure_sets(
                Startup,
                (
                    GameSetupSet::GameParams,
                    GameSetupSet::GameMechanics,
                    GameSetupSet::FileSystem,
                    GameSetupSet::GameVersion,
                    GameSetupSet::SaveDir,
                )
                    .chain(),
            )
            .add_systems(
                Startup,
                (
                    game_params::setup.in_set(GameSetupSet::GameParams),
                    game_mechanics::setup.in_set(GameSetupSet::GameMechanics),
                    file_system::setup
                        .pipe(exit_error)
                        .in_set(GameSetupSet::FileSystem),
                    game_version::setup
                        .pipe(exit_error)
                        .in_set(GameSetupSet::GameVersion),
                ),
            );
    }
}

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub enum GameSetupSet {
    GameParams,
    GameMechanics,
    FileSystem,
    GameVersion,
    SaveDir,
}

#[derive(Resource)]
pub struct Doom {
    pub console_player: i32,
    pub death_match: i32,
    pub dev_parm: bool,
    pub fast_parm: bool,
    pub forward_move: [Fixed; 2],
    pub game_mission: GameMission,
    pub game_mode: GameMode,
    pub game_variant: GameVariant,
    pub game_version: GameVersion,
    pub iwad_file: PathBuf,
    pub modified_game: bool,
    pub no_monsters: bool,
    pub player_in_game: [bool; MAX_PLAYERS as usize],
    pub respawn_parm: bool,
    pub save_game_dir: PathBuf,
    pub side_move: [Fixed; 2],
}

impl Doom {
    pub fn iwad_file(&self) -> &Path {
        &self.iwad_file
    }
}

impl Default for Doom {
    fn default() -> Self {
        Self {
            console_player: 0,
            death_match: 0,
            dev_parm: false,
            fast_parm: false,
            forward_move: [Fixed::from_bits(0x19), Fixed::from_bits(0x32)],
            game_mission: GameMission::Doom,
            game_mode: GameMode::Indetermined,
            game_variant: GameVariant::Vanilla,
            game_version: GameVersion::Final2,
            iwad_file: PathBuf::new(),
            modified_game: false,
            no_monsters: false,
            player_in_game: [false; MAX_PLAYERS as usize],
            respawn_parm: false,
            save_game_dir: PathBuf::new(),
            side_move: [Fixed::from_bits(0x18), Fixed::from_bits(0x28)],
        }
    }
}
