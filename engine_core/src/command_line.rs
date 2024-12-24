use std::collections::{HashMap, VecDeque};
use std::ffi::CString;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::{env, fs};

use bevy::app::{App, Plugin};
use clap::{Parser, ValueEnum};

use crate::file_system::{FileType, GameMission, GameVersion};
use crate::Skill;

const MAX_ARGS: u8 = 100;

pub struct CommandLinePlugin;

impl Plugin for CommandLinePlugin {
    fn build(&self, app: &mut App) {
        app.insert_non_send_resource(Rc::new(CommandLine::parse()));
    }
}

#[derive(Debug, Parser)]
#[command(version, next_line_help = true)]
pub struct CommandLine {
    /// Equivalent to "--af <files> --as <files>"
    #[arg(long = "aa", num_args = 1.., value_name = "FILES")]
    add_all: Vec<PathBuf>,

    /// Simulates the behavior of NWT's -af option, merging flats
    /// into the main IWAD directory
    #[arg(long = "af", num_args = 1.., value_name = "FILES")]
    add_flats: Vec<PathBuf>,

    /// Simulates the behavior of NWT's -as option, merging sprites
    /// into the main IWAD directory
    #[arg(long = "as", num_args = 1.., value_name = "FILES")]
    add_sprites: Vec<PathBuf>,

    /// Start a deathmatch 2.0 game. Weapons do not stay in place
    /// and all items respawn after 30 seconds
    #[arg(long = "altdeath")]
    alt_death: bool,

    /// Austin Virtual Gaming: end levels after 20 minutes
    #[arg(long)]
    avg: bool,

    /// Save configuration data and savegames in c:\doomdata,
    /// allowing play from CD. (windows only)
    #[cfg(windows)]
    #[arg(long)]
    cdrom: bool,

    /// Load main configuration from the specified file, instead of
    /// the default
    #[arg(long, value_name = "FILE")]
    config: Option<PathBuf>,

    /// Start a deathmatch game
    #[arg(long = "deathmatch")]
    death_match: bool,

    /// Start a dedicated server, routing packets but not
    /// participating in the game itself
    #[arg(long)]
    dedicated: bool,

    /// Load the given dehacked patch(es)
    #[arg(long, num_args = 1.., value_name = "FILES")]
    deh: Vec<PathBuf>,

    /// Developer mode: F1 saves a screenshot in the current
    /// working directory
    #[arg(long = "devparm")]
    dev_mode: bool,

    /// Disable auto-loading of .wad and .deh file_format
    #[arg(long = "noautoload")]
    disable_autoload: bool,

    /// If specified, don't show a GUI window for error messages when
    /// the game exits with an error
    #[arg(long = "nogui")]
    disable_gui: bool,

    /// Disable monsters
    #[arg(long = "nomonsters")]
    disable_monsters: bool,

    /// Reduce the resolution of the game by a factor of N, reducing
    /// the amount of network bandwidth needed
    #[arg(long, value_name = "N")]
    dup: Option<i32>,

    /// Start playing on episode n (1-4)
    #[arg(long)]
    episode: Option<i32>,

    /// Load additional configuration from the specified file, instead of
    /// the default
    #[arg(long = "extraconfig", value_name = "FILE")]
    extra_config: Option<PathBuf>,

    /// Send n extra tics in every packet as insurance against dropped packets
    #[arg(long = "extratics")]
    extra_tics: Option<i32>,

    /// Monsters move faster
    #[arg(long)]
    fast: bool,

    /// Load the specified PWAD files
    #[arg(long, num_args = 1.., value_name = "FILES")]
    file: Vec<PathBuf>,

    /// Specify the game to configure the settings for
    #[arg(long, value_enum)]
    game: Option<GameConfig>,

    /// Emulate a specific version of Doom
    #[arg(long = "gameversion", value_enum)]
    game_version: Option<GameVersionCli>,

    /// Specify an IWAD file to use
    #[arg(long, value_name = "FILE")]
    iwad: Option<PathBuf>,

    /// Load the game in slot SAVEGAME
    #[arg(long = "loadgame", value_name = "SAVEGAME")]
    load_game: Option<i32>,

    /// Search the local LAN for running servers
    #[arg(long = "localsearch")]
    local_search: bool,

    /// Record a high resolution "Doom 1.91" demo
    #[arg(long = "longtics")]
    long_tics: bool,

    /// Simulates the behavior of deutex's -merge option, merging a PWAD
    /// into the main IWAD
    #[arg(long, num_args = 1.., value_name = "FILES")]
    merge: Vec<PathBuf>,

    /// Sets debug log file path for the network code. If this is absent,
    /// then no log will be created.
    #[arg(long = "netlog")]
    net_log: Option<PathBuf>,

    /// Explicitly specify a Doom II "mission pack" to run as, instead
    /// of detecting it based on the filename
    #[arg(long, value_enum)]
    pack: Option<Doom2MissionPack>,

    /// Use the specified UDP port for communications, instead of
    /// the default (2342)
    #[arg(long)]
    port: Option<u16>,

    /// Play back the demo named demo.lmp
    #[arg(long = "playdemo", value_name = "DEMO")]
    play_demo: Option<PathBuf>,

    /// Monsters respawn after being killed
    #[arg(long)]
    respawn: bool,

    /// Load extra command line arguments from the given response file_format
    #[arg(long, num_args = 1.., value_name = "FILES")]
    response: Vec<PathBuf>,

    /// Specify a path from which to load and save games. If the directory does
    /// not exist then it will automatically be created
    #[arg(long = "savedir", value_name = "DIRECTORY")]
    save_dir: Option<PathBuf>,

    /// Play with low turning resolution to emulate demo recording
    #[arg(long = "shorttics")]
    short_tics: bool,

    /// Set the game skill, 1-5 (1: easiest, 5: hardest). A skill of 0
    /// disables all monsters
    #[arg(long, allow_negative_numbers = true, value_parser = parse_skill_param)]
    skill: Option<Skill>,

    /// For multiplayer games: exit each level after N minutes
    #[arg(long, value_name = "N", default_value_if("avg", "true", Some("20")))]
    timer: Option<i32>,

    /// Turbo mode. The player's speed is multiplied by x%. If unspecified,
    /// x defaults to 200. Values are rounded up to 10 and down to 400.
    #[arg(
    long,
    num_args = 0..=1,
    default_missing_value = "200",
    allow_hyphen_values = true,
    value_parser = parse_turbo_param,
    )]
    turbo: Option<u16>,

    /// Start a game immediately, warping to ExMy (Doom 1) or MAPxy (Doom 2)
    #[arg(long, num_args = 1..=2, value_name = "<X> <Y> | <XY>")]
    warp: Vec<String>,

    /// This is an earlier version of --warp and should NOT be used. If
    /// used, the game will simply ignore it and run as if it was not
    /// present
    #[arg(long, num_args = 1..=2, hide = true)]
    wart: Vec<String>,
}

impl CommandLine {
    pub(super) fn parse() -> Self {
        let args = get_processed_args();
        if args.is_err() {
            panic!("{}", args.unwrap_err());
        }

        CommandLine::parse_from(args.unwrap())
    }

    pub fn add_all(&self) -> &Vec<PathBuf> {
        &self.add_all
    }

    pub fn add_flats(&self) -> &Vec<PathBuf> {
        &self.add_flats
    }

    pub fn add_sprites(&self) -> &Vec<PathBuf> {
        &self.add_sprites
    }

    pub fn alt_death(&self) -> bool {
        self.alt_death
    }

    pub fn avg(&self) -> bool {
        self.avg
    }

    #[cfg(windows)]
    pub fn cdrom(&self) -> bool {
        self.cdrom
    }

    pub fn config(&self) -> &Option<PathBuf> {
        &self.config
    }

    pub fn death_match(&self) -> bool {
        self.death_match
    }

    pub fn dedicated(&self) -> bool {
        self.dedicated
    }

    pub fn deh(&self) -> &Vec<PathBuf> {
        &self.deh
    }

    pub fn dev_mode(&self) -> bool {
        self.dev_mode
    }

    pub fn disable_autoload(&self) -> bool {
        self.disable_autoload
    }

    pub fn disable_gui(&self) -> bool {
        self.disable_gui
    }

    pub fn disable_monsters(&self) -> bool {
        self.disable_monsters
    }

    pub fn dup(&self) -> Option<i32> {
        self.dup
    }

    pub fn episode(&self) -> Option<i32> {
        self.episode
    }

    pub fn extra_config(&self) -> &Option<PathBuf> {
        &self.extra_config
    }

    pub fn extra_tics(&self) -> Option<i32> {
        self.extra_tics
    }

    pub fn fast(&self) -> bool {
        self.fast
    }

    pub fn file(&self) -> &Vec<PathBuf> {
        &self.file
    }

    // ATTENTION! - REMOVE REFERENCE AND IMPLEMENT COPY TRAIT
    pub fn game(&self) -> &Option<GameConfig> {
        &self.game
    }

    pub fn game_version(&self) -> Option<GameVersionCli> {
        self.game_version
    }

    pub fn iwad(&self) -> &Option<PathBuf> {
        &self.iwad
    }

    pub fn load_game(&self) -> Option<i32> {
        self.load_game
    }

    pub fn local_search(&self) -> bool {
        self.local_search
    }

    pub fn long_tics(&self) -> bool {
        self.long_tics
    }

    pub fn merge(&self) -> &Vec<PathBuf> {
        &self.merge
    }

    pub fn net_log(&self) -> &Option<PathBuf> {
        &self.net_log
    }

    pub fn pack(&self) -> Option<Doom2MissionPack> {
        self.pack
    }

    pub fn port(&self) -> Option<u16> {
        self.port
    }

    pub fn play_demo(&self) -> &Option<PathBuf> {
        &self.play_demo
    }

    pub fn respawn(&self) -> bool {
        self.respawn
    }

    pub fn response(&self) -> &Vec<PathBuf> {
        &self.response
    }

    pub fn save_dir(&self) -> &Option<PathBuf> {
        &self.save_dir
    }

    pub fn short_tics(&self) -> bool {
        self.short_tics
    }

    pub fn skill(&self) -> Option<Skill> {
        self.skill
    }

    pub fn timer(&self) -> Option<i32> {
        self.timer
    }

    pub fn turbo(&self) -> Option<u16> {
        self.turbo
    }

    pub fn warp(&self) -> &Vec<String> {
        &self.warp
    }

    pub fn wart(&self) -> &Vec<String> {
        &self.wart
    }
}

fn parse_turbo_param(turbo: &str) -> Result<u16, String> {
    if turbo.is_empty() {
        return Ok(200);
    }

    let cstr = CString::new(turbo).map_err(|nul_err| nul_err.to_string())?;
    let scale = unsafe { libc::atoi(cstr.as_ptr()) };
    if scale < 10 {
        return Ok(10);
    }
    if scale > 400 {
        return Ok(400);
    }
    Ok(scale as u16)
}

fn parse_skill_param(skill: &str) -> Result<Skill, String> {
    let first_char = skill.chars().next().ok_or("Empty skill level".to_owned())? as i32;
    let skill_num = first_char - '1' as i32;

    Ok(skill_num.into())
}

#[derive(ValueEnum, Debug, Clone)]
pub enum GameConfig {
    Doom,
    Heretic,
    Hexen,
    Strife,
}

#[derive(ValueEnum, Debug, Clone, Copy)]
pub enum Doom2MissionPack {
    Doom2,
    Tnt,
    Plutonia,
}

impl Doom2MissionPack {
    pub fn to_game_mission(&self) -> GameMission {
        match self {
            Doom2MissionPack::Doom2 => GameMission::Doom2,
            Doom2MissionPack::Tnt => GameMission::PackTnt,
            Doom2MissionPack::Plutonia => GameMission::PackPlut,
        }
    }
}

#[derive(ValueEnum, Debug, Clone, Copy)]
pub enum GameVersionCli {
    /// Chex Quest
    Chex,
    /// Doom 1.2
    #[clap(name = "1.2")]
    Doom12,
    /// Doom 1.666
    #[clap(name = "1.666")]
    Doom1666,
    /// Doom 1.7/1.7a
    #[clap(name = "1.7")]
    Doom17,
    /// Doom 1.8
    #[clap(name = "1.8")]
    Doom18,
    /// Doom 1.9
    #[clap(name = "1.9")]
    Doom19,
    /// Final Doom
    Final,
    /// Final Doom (alt)
    Final2,
    /// Hacx
    Hacx,
    /// Ultimate Doom
    Ultimate,
}

impl GameVersionCli {
    pub fn to_game_version(&self) -> GameVersion {
        match self {
            GameVersionCli::Chex => GameVersion::Chex,
            GameVersionCli::Doom12 => GameVersion::Doom12,
            GameVersionCli::Doom1666 => GameVersion::Doom1666,
            GameVersionCli::Doom17 => GameVersion::Doom17,
            GameVersionCli::Doom18 => GameVersion::Doom18,
            GameVersionCli::Doom19 => GameVersion::Doom19,
            GameVersionCli::Final => GameVersion::Final,
            GameVersionCli::Final2 => GameVersion::Final2,
            GameVersionCli::Hacx => GameVersion::Hacx,
            GameVersionCli::Ultimate => GameVersion::Ultimate,
        }
    }
}

fn get_processed_args() -> Result<VecDeque<String>, String> {
    let mut args: VecDeque<String> = env::args().collect();

    handle_drag_n_drop_files(&mut args);
    handle_legacy_response_files(&mut args)?;
    load_response_files(&mut args)?;
    remove_extra_response_files(&mut args);

    if args.len() > MAX_ARGS as usize {
        let message = format!("Too many arguments! Maximum arguments accepted is {MAX_ARGS}\n");
        return Err(message);
    }

    Ok(args)
}

fn handle_drag_n_drop_files(args: &mut VecDeque<String>) {
    if args.len() < 2 {
        return;
    }

    let mut files_map: HashMap<FileType, Vec<String>> = HashMap::from([
        (FileType::Unknown, vec![]),
        (FileType::Iwad, vec![String::from("--iwad")]),
        (FileType::Pwad, vec![String::from("--merge")]),
        (FileType::Deh, vec![String::from("--deh")]),
        (FileType::Demo, vec![String::from("--playdemo")]),
    ]);

    // We iterate over the CMD arguments, skipping the first element,
    // because it points to the program executable location. We then check
    // if there is some argument which is not a LFS or UNC path. If it does
    // happen, then we simply exit the function.
    for arg in args.iter() {
        let path = Path::new(arg);

        if !path.has_root() {
            return;
        }

        let file_type = FileType::from(path);
        let files = files_map.get_mut(&file_type).unwrap();
        files.push(arg.clone());
    }

    args.clear();

    for files in files_map.into_values() {
        let mut files_deq: VecDeque<String> = VecDeque::from(files);
        args.append(&mut files_deq);
    }
}

fn handle_legacy_response_files(args: &mut VecDeque<String>) -> Result<(), String> {
    let mut new_args = VecDeque::with_capacity(args.len());

    while let Some(arg) = args.pop_front() {
        let mut chars = arg.chars();

        if arg.is_empty() || chars.next().unwrap() != '@' || arg.contains(char::is_whitespace) {
            new_args.push_back(arg);
            continue;
        }
        if arg.len() == 1 {
            return Err("'@' must be followed by a response file\n".to_owned());
        }
        if !args.is_empty() && !is_param(&args[0]) {
            return Err("'@' accepts only one argument\n".to_owned());
        }

        new_args.push_back(String::from("--response"));
        // The next function consumes the first char, so collect
        // here returns arg without '@'.
        new_args.push_back(chars.collect());
    }

    args.append(&mut new_args);

    Ok(())
}

fn load_response_files(args: &mut VecDeque<String>) -> Result<(), String> {
    let mut new_args = VecDeque::with_capacity(args.len());

    while let Some(arg) = args.pop_front() {
        if arg != "--response" {
            new_args.push_back(arg);
            continue;
        }

        if args.is_empty() {
            return Err("'--response' must be followed by a response file\n".to_owned());
        }

        while let Some(next_arg) = args.pop_front() {
            if is_param(&next_arg) {
                args.push_front(next_arg);
                break;
            }

            let file = next_arg;
            let mut file_args = parse_file(&file)?;

            new_args.append(&mut file_args);
        }
    }

    args.append(&mut new_args);

    Ok(())
}

fn parse_file(file: &str) -> Result<VecDeque<String>, String> {
    let file_content = fs::read(file).map_err(|error| error.to_string())?;
    let file_size = file_content.len();
    let mut file_args = VecDeque::new();
    let mut i: usize = 0;

    while i < file_size {
        while i < file_size && (file_content[i] as char).is_whitespace() {
            i += 1;
        }

        if i == file_size {
            break;
        }

        let mut j = i + 1;

        // Treat
        if (file_content[i] as char) == '"' {
            while j < file_size
                && (file_content[j] as char) != '"'
                && (file_content[j] as char) != '\n'
            {
                j += 1;
            }

            if j == file_size || (file_content[j] as char) == '\n' {
                let message = format!("Quotes unclosed in response file '{file}'\n");
                return Err(message);
            }

            i += 1;
        } else {
            while j < file_size && !(file_content[j] as char).is_whitespace() {
                j += 1;
            }
        }

        let param = std::str::from_utf8(&file_content[i..j]).unwrap().to_owned();
        file_args.push_back(param);
        i = j + 1;
    }

    Ok(file_args)
}

fn remove_extra_response_files(args: &mut VecDeque<String>) {
    let mut keep = true;

    args.retain(|arg| {
        if arg == "--response" {
            keep = false;
        } else if is_param(arg) {
            keep = true;
        }
        keep
    });
}

fn is_param(str: &str) -> bool {
    str.starts_with('-') && !str.contains(char::is_whitespace)
}
