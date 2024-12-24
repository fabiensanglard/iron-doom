use std::fmt::{Display, Formatter};
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::{env, fs};

use bevy::app::App;
use bevy::prelude::Plugin;
use sdl2::filesystem::PrefPathError::SdlError;

pub use directory::*;
pub use file_format::*;

use crate::command_line::CommandLine;
use crate::sys::platform::file_system::iwad as platform_iwad;

mod directory;
mod file_format;

pub static DOOM_MISSIONS: &[GameMission] = &[
    GameMission::Doom,
    GameMission::Doom2,
    GameMission::PackChex,
    GameMission::PackTnt,
    GameMission::PackPlut,
];
pub static HERETIC_MISSIONS: &[GameMission] = &[GameMission::Heretic];
pub static HEXEN_MISSIONS: &[GameMission] = &[GameMission::Hexen];
pub static STRIFE_MISSIONS: &[GameMission] = &[GameMission::Strife];

pub struct FileSystemPlugin;

impl Plugin for FileSystemPlugin {
    fn build(&self, app: &mut App) {
        let cli = app.world().non_send_resource::<Rc<CommandLine>>();
        let cli = Rc::clone(cli);
        let file_sys = FileSystem::init(cli).expect("Failed to initialize VideoSystem");

        app.insert_non_send_resource(file_sys);
    }
}

#[derive(Debug)]
pub struct FileSystem {
    cli: Rc<CommandLine>,
    current_dir: PathBuf,
    config_dir: PathBuf,
    executable_name: String,
    iwad_dirs: IwadDirs,
}

impl FileSystem {
    pub fn init(cli: Rc<CommandLine>) -> Result<Self, String> {
        let executable_name = executable_name()?;
        let current_dir = current_dir()?;
        let config_dir = if cfg!(windows) {
            // On Windows, behave like Vanilla Doom and use the current directory.
            current_dir.clone()
        } else {
            // An Unix-like OS saves executables in usr/bin, which is generally
            // not writable for everyone. So we ask SDL for a directory to use instead.
            sdl_config_dir()?
        };
        let iwad_dirs = build_iwad_dirs(&current_dir);

        Ok(FileSystem {
            cli,
            current_dir,
            config_dir,
            executable_name,
            iwad_dirs,
        })
    }

    pub fn create_dir<P: AsRef<Path>>(&self, path: P) -> Result<(), String> {
        fs::create_dir_all(path).map_err(|error| error.to_string())
    }

    pub fn current_dir(&self) -> &Path {
        &self.current_dir
    }

    pub fn set_config_dir<T: Into<PathBuf>>(&mut self, dir: T) -> Result<(), String> {
        let dir = dir.into();

        if dir != self.config_dir {
            println!("Using {:?} for configuration and saves\n", self.config_dir);
        }

        self.create_dir(&dir)?;
        self.config_dir = dir;

        Ok(())
    }

    pub fn config_dir(&self) -> &Path {
        &self.config_dir
    }

    pub fn executable_name(&self) -> &str {
        &self.executable_name
    }

    pub fn find_iwad(
        &self,
        missions: &[GameMission],
    ) -> Result<(Option<PathBuf>, GameMission), String> {
        if let Some(iwad_file) = self.cli.iwad() {
            let iwad_path = self
                .find_iwad_by_name(iwad_file)
                .ok_or(format!("IWAD file '{iwad_file:#?}' not found!"))?;
            let mission = identify_iwad_by_name(&iwad_path, missions);

            return Ok((Some(iwad_path), mission));
        } else {
            for iwad in &self.iwad_dirs {
                let (iwad_path, mission) = search_directory_for_iwad(iwad, missions);
                if iwad_path.is_some() {
                    return Ok((iwad_path, mission));
                }
            }
        }

        Ok((None, GameMission::None))
    }

    pub fn find_iwad_by_name(&self, name: &Path) -> Option<PathBuf> {
        if name.exists() {
            return Some(name.to_path_buf());
        }

        for dir in &self.iwad_dirs {
            if let Some(file_name) = dir.file_name() {
                if file_name.eq_ignore_ascii_case(name) {
                    return Some(dir.clone());
                }
            }

            let dir = dir.join(name);
            if dir.exists() {
                return Some(dir);
            }
        }

        None
    }

    pub fn get_save_game_dir(&self, iwad_name: &str) -> Result<PathBuf, String> {
        if let Some(dir) = self.cli.save_dir() {
            if !dir.exists() {
                self.create_dir(dir)?;
            }

            println!("Save directory changed to {:?}.", dir);
            return Ok(dir.clone());
        }

        // In -cdrom mode, we write savegames to a specific directory
        // in addition to configs.
        #[cfg(windows)]
        if self.cli.cdrom() {
            return Ok(self.config_dir.clone());
        }

        // If not "doing" a configuration directory (Windows), don't "do"
        // a savegame directory, either.
        if self.config_dir != self.current_dir {
            return Ok(PathBuf::from(""));
        }

        let top_dir = self.config_dir().join("savegames");
        // ~/.local/share/chocolate-doom/savegames
        self.create_dir(&top_dir)?;

        let save_game_dir = top_dir.join(iwad_name);
        // eg. ~/.local/share/chocolate-doom/savegames/doom2.wad/
        self.create_dir(&save_game_dir)?;

        Ok(save_game_dir)
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum GameMission {
    Doom,
    // Doom 1
    Doom2,
    // Doom 2
    Heretic,
    // Heretic
    Hexen,
    // Hexen
    PackChex,
    // Chex Quest (modded doom)
    PackHacx,
    // Hacx (modded doom2)
    PackPlut,
    // Final Doom: The Plutonia Experiment
    PackTnt,
    // Final Doom: TNT: Evilution
    Strife,
    // Strife
    None,
}

impl GameMission {
    /// 'GameMission' can be equal to PackChex or PackHacx, but these are
    /// just modified versions of Doom and Doom2, and should be interpreted
    /// as the same most of the time.
    pub fn logical_mission(&self) -> GameMission {
        match self {
            GameMission::PackChex => GameMission::Doom,
            GameMission::PackHacx => GameMission::Doom2,
            _ => *self,
        }
    }
}

impl Display for GameMission {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let str = match self {
            GameMission::Doom => String::from("doom"),
            GameMission::Doom2 => String::from("doom2"),
            GameMission::Heretic => String::from("heretic"),
            GameMission::Hexen => String::from("hexen"),
            GameMission::PackChex => String::from("chex"),
            GameMission::PackHacx => String::from("hacx"),
            GameMission::PackPlut => String::from("plutonia"),
            GameMission::PackTnt => String::from("tnt"),
            GameMission::Strife => String::from("strife"),
            GameMission::None => String::from("doom"),
        };
        write!(f, "{}", str)
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum GameMode {
    Shareware,
    // Doom/Heretic shareware
    Registered,
    // Doom/Heretic registered
    Commercial,
    // Doom II/Hexen
    Retail,
    // Ultimate Doom
    Indetermined, // Unknown.
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum GameVersion {
    /// Chex Quest executable (based on Final Doom)
    Chex,
    /// Doom 1.2: shareware and registered
    Doom12,
    /// Doom 1.666: for shareware, registered and commercial
    Doom1666,
    /// Doom 1.7/1.7a
    Doom17,
    /// Doom 1.8
    Doom18,
    /// Doom 1.9
    Doom19,
    /// Final Doom
    Final,
    /// Final Doom (alternate exe)
    Final2,
    /// Hacx
    Hacx,
    /// Heretic 1.3
    Heretic13,
    /// Hexen 1.1
    Hexen11,
    /// Strife v1.2
    Strife12,
    /// Strife v1.31
    Strife131,
    /// Ultimate Doom (retail)
    Ultimate,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum GameVariant {
    /// Vanilla Doom
    Vanilla,
    /// FreeDoom: Phase 1 + 2
    FreeDoom,
    /// FreeDM
    FreeDm,
    /// Doom Classic (Doom 3: BFG Edition)
    BfgEdition,
}

fn current_dir() -> Result<PathBuf, String> {
    env::current_dir().map_err(|error| error.to_string())
}

fn executable_name() -> Result<String, String> {
    let path_exe = env::current_exe().map_err(|error| error.to_string())?;
    let os_exe_name = path_exe.file_name();

    if os_exe_name.is_none() {
        let msg = String::from("No available executable name");
        return Err(msg);
    }

    let exe_name = os_exe_name.unwrap().to_str();

    if exe_name.is_none() {
        let msg = String::from("Executable name is invalid Unicode");
        return Err(msg);
    }

    Ok(exe_name.unwrap().to_string())
}

fn sdl_config_dir() -> Result<PathBuf, String> {
    let sdl_dir = sdl2::filesystem::pref_path("", env!("WORKSPACE_TARNAME"));

    if let Ok(sdl_pref_dir) = sdl_dir {
        return Ok(PathBuf::from(sdl_pref_dir));
    }

    match sdl_dir.unwrap_err() {
        SdlError(error) => Err(error),
        _ => Err("Error while retrieving SDL configuration directory".to_string()),
    }
}

fn build_iwad_dirs(curr_dir: &Path) -> IwadDirs {
    let mut iwad_dirs = IwadDirs::new();

    // Look in the current directory.  Doom always does this.
    iwad_dirs.add_dir(".");

    // Next check the directory where the executable is located. This might
    // be different from the current directory.
    iwad_dirs.add_dir(curr_dir);

    // Add DOOMWADDIR and dirs from DOOMWADPATH if defined in environment
    if let Ok(dir) = env::var("DOOMWADDIR") {
        iwad_dirs.add_dir(&dir);
    }
    if let Ok(path) = env::var("DOOMWADPATH") {
        iwad_dirs.add_path(&path, "");
    }

    platform_iwad::add_iwad_dirs(&mut iwad_dirs);

    iwad_dirs
}

fn identify_iwad_by_name(path: &Path, missions: &[GameMission]) -> GameMission {
    let name = path.file_name();
    if name.is_none() {
        return GameMission::None;
    }
    let name = name.unwrap();

    for iwad in IWADS {
        if missions.contains(&iwad.mission) && name == iwad.name {
            return iwad.mission;
        }
    }

    GameMission::None
}

/// Search a directory to try to find an IWAD
/// Returns the location of the IWAD if found.
fn search_directory_for_iwad(
    dir: &Path,
    missions: &[GameMission],
) -> (Option<PathBuf>, GameMission) {
    for iwad in IWADS {
        if !missions.contains(&iwad.mission) {
            continue;
        }
        if let Some(iwad_path) = check_directory_has_iwad(dir, iwad.name) {
            return (Some(iwad_path), iwad.mission);
        }
    }

    (None, GameMission::None)
}

/// Check if the specified directory contains the specified IWAD
/// file, returning the full path to the IWAD if found.
fn check_directory_has_iwad(dir: &Path, iwad_name: &str) -> Option<PathBuf> {
    if let Some(base) = dir.file_name() {
        if base.eq_ignore_ascii_case(iwad_name) {
            return Some(dir.to_path_buf());
        }
    }

    let file_name = if dir == Path::new(".") {
        PathBuf::from(iwad_name)
    } else {
        dir.join(iwad_name)
    };

    if file_name.exists() {
        Some(file_name)
    } else {
        None
    }
}
