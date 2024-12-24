use std::env;

use crate::file_system::IwadDirs;

pub fn add_iwad_dirs(iwad_dirs: &mut IwadDirs) {
    add_xdg_dirs(iwad_dirs);
    #[cfg(target_os = "macos")]
    add_steam_dirs(iwad_dirs);
}

/// Add standard directories where IWADs are located on Unix systems.
/// To respect the freedesktop.org specification we support overriding
/// using standard environment variables. See the XDG Base Directory
/// Specification:
/// <http://standards.freedesktop.org/basedir-spec/basedir-spec-latest.html>
fn add_xdg_dirs(iwad_dirs: &mut IwadDirs) {
    // Quote:
    // > $XDG_DATA_HOME defines the base directory relative to which
    // > user specific data file_format should be stored. If $XDG_DATA_HOME
    // > is either not set or empty, a default equal to
    // > $HOME/.local/share should be used.
    let env = env::var("XDG_DATA_HOME").unwrap_or_else(|_| {
        let home = env::var("HOME").unwrap_or_else(|_| String::from("/"));
        format!("{home}/.local/share")
    });

    // We support $XDG_DATA_HOME/games/doom (which will usually be
    // ~/.local/share/games/doom) as a user-writeable extension to
    // the usual /usr/share/games/doom location.
    iwad_dirs.add_dir(&format!("{env}/games/doom"));

    // Quote:
    // > $XDG_DATA_DIRS defines the preference-ordered set of base
    // > directories to search for data file_format in addition to the
    // > $XDG_DATA_HOME base directory. The directories in $XDG_DATA_DIRS
    // > should be seperated with a colon ':'.
    // >
    // > If $XDG_DATA_DIRS is either not set or empty, a value equal to
    // > /usr/local/share/:/usr/share/ should be used.
    let env =
        env::var("XDG_DATA_DIRS").unwrap_or_else(|_| String::from("/usr/local/share:/usr/share"));

    // The "standard" location for IWADs on Unix that is supported by most
    // source ports is /usr/share/games/doom - we support this through the
    // XDG_DATA_DIRS mechanism, through which it can be overridden.
    iwad_dirs.add_path(&env, "games/doom");
    iwad_dirs.add_path(&env, "doom");

    // The convention set by RBDOOM-3-BFG is to install Doom 3: BFG
    // Edition into this directory, under which includes the Doom
    // Classic WADs.
    iwad_dirs.add_path(&env, "games/doom3bfg/base/wads");
}

/// Steam on Linux allows installing some select Windows games,
/// including the classic Doom series (running DOSBox via Wine).  We
/// could parse *.vdf file_format to more accurately detect installation
/// locations, but the defaults are likely to be good enough for just
/// about everyone.
#[cfg(target_os = "macos")]
fn add_steam_dirs(iwad_dirs: &mut IwadDirs) {
    let home = env::var("HOME").unwrap_or_else(|_| String::from("/"));
    let steam_dir = format!("{home}/.steam/root/steamapps/common");

    iwad_dirs.add_path(&steam_dir, "Doom 2/base");
    iwad_dirs.add_path(&steam_dir, "Doom 2/finaldoombase");
    iwad_dirs.add_path(&steam_dir, "Master Levels of Doom/doom2");
    iwad_dirs.add_path(&steam_dir, "Ultimate Doom/base");
    iwad_dirs.add_path(&steam_dir, "Final Doom/base");
    iwad_dirs.add_path(&steam_dir, "DOOM 3 BFG Edition/base/wads");
    iwad_dirs.add_path(&steam_dir, "Heretic Shadow of the Serpent Riders/base");
    iwad_dirs.add_path(&steam_dir, "Hexen/base");
    iwad_dirs.add_path(&steam_dir, "Hexen Deathkings of the Dark Citadel/base");
    iwad_dirs.add_path(&steam_dir, "Strife");
}
