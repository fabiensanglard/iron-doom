use std::ffi::OsStr;
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::Path;

use crate::file_system::{GameMission, GameMode, GameVariant};

pub static IWADS: &[Iwad; 15] = &[
    Iwad::new(
        "chex.wad",
        GameMission::PackChex,
        GameMode::Retail,
        "Chex Quest",
    ),
    Iwad::new("doom.wad", GameMission::Doom, GameMode::Retail, "Doom"),
    Iwad::new(
        "doom1.wad",
        GameMission::Doom,
        GameMode::Shareware,
        "Doom Shareware",
    ),
    Iwad::new(
        "doom2.wad",
        GameMission::Doom2,
        GameMode::Commercial,
        "Doom II",
    ),
    Iwad::new(
        "doom2f.wad",
        GameMission::Doom2,
        GameMode::Commercial,
        "Doom II: L'Enfer sur Terre",
    ),
    Iwad::new(
        "freedm.wad",
        GameMission::Doom2,
        GameMode::Commercial,
        "FreeDM",
    ),
    Iwad::new(
        "freedoom1.wad",
        GameMission::Doom,
        GameMode::Retail,
        "Freedoom: Phase 1",
    ),
    Iwad::new(
        "freedoom2.wad",
        GameMission::Doom2,
        GameMode::Commercial,
        "Freedoom: Phase 2",
    ),
    Iwad::new(
        "hacx.wad",
        GameMission::PackHacx,
        GameMode::Commercial,
        "Hacx",
    ),
    Iwad::new(
        "heretic.wad",
        GameMission::Heretic,
        GameMode::Retail,
        "Heretic",
    ),
    Iwad::new(
        "heretic1.wad",
        GameMission::Heretic,
        GameMode::Shareware,
        "Heretic Shareware",
    ),
    Iwad::new(
        "hexen.wad",
        GameMission::Hexen,
        GameMode::Commercial,
        "Hexen",
    ),
    Iwad::new(
        "plutonia.wad",
        GameMission::PackPlut,
        GameMode::Commercial,
        "Final Doom: Plutonia Experiment",
    ),
    Iwad::new(
        "strife1.wad",
        GameMission::Strife,
        GameMode::Commercial,
        "Strife",
    ),
    Iwad::new(
        "tnt.wad",
        GameMission::PackTnt,
        GameMode::Commercial,
        "Final Doom: TNT: Evilution",
    ),
];

pub struct Iwad {
    pub name: &'static str,
    pub mission: GameMission,
    pub mode: GameMode,
    pub description: &'static str,
}

impl Iwad {
    const fn new(
        name: &'static str,
        mission: GameMission,
        mode: GameMode,
        description: &'static str,
    ) -> Self {
        Self {
            name,
            mission,
            mode,
            description,
        }
    }
}

#[derive(Eq, PartialEq, Hash)]
pub enum FileType {
    Unknown,
    Iwad,
    Pwad,
    Deh,
    Demo,
}

impl From<&Path> for FileType {
    fn from(path: &Path) -> Self {
        let stem = path
            .file_stem()
            .unwrap_or(path.as_os_str())
            .to_str()
            .unwrap();

        if is_iwad_name(stem) {
            return FileType::Iwad;
        }

        let extension = path.extension().unwrap_or(OsStr::new("")).to_str().unwrap();

        match extension {
            "deh" | "hhe" | "seh" => FileType::Deh,
            "lmp" => {
                if is_demo_lump(path) {
                    return FileType::Demo;
                }
                FileType::Pwad
            }
            _ => FileType::Unknown,
        }
    }
}

fn is_demo_lump(path: &Path) -> bool {
    // The idea here is the following: every demo lump has a header containing
    // information about the skill level, episode and map, regardless of the
    // version of the game used to record it. The header is located in the first
    // 12 bytes of the demo file (its actual size varies with the game version used
    // to record it). So we read the first 12 bytes of the file into a buffer and then
    // check if the skill level, episode and map information is there. If it is,
    // then the file is considered a demo lump.

    let file = match File::open(path) {
        Err(_) => return false,
        Ok(file) => file,
    };
    let mut reader = BufReader::new(file);
    let mut buf: [u8; 12] = [0; 12];

    if reader.read_exact(&mut buf).is_err() {
        return false;
    }

    // skill level, episode and map are all single bytes inside the demo header.
    let skill: u8;
    let episode: u8;
    let map: u8;

    // Here comes the tricky part: the first byte of the header of a demo
    // recorded in version up to v1.2 represents the skill level, but after v1.2
    // the first byte is used for the game version. Because of this, the location
    // of the skill, episode and map inside the header differs when comparing a
    // pre-v1.2 demo and a post-v1.2 demo. So if the first byte is less than or
    // equal to 4, then it is a valid skill level and we consider it as a pre-v1.2
    // demo lump file.
    if buf[0] <= 4 {
        skill = buf[0];
        episode = buf[1];
        map = buf[2];
    } else {
        // 104 is v1.4, 105 is v1.5 and so on. 111 is v1.91 unofficial patch.
        // If the first byte is not one of these values, then it is not a valid
        // game version and it is also not a valid skill level, so the file is
        // not a demo lump.
        if (buf[0] < 104 || buf[0] > 109) && buf[0] != 111 {
            return false;
        }
        skill = buf[1];
        episode = buf[2];
        map = buf[3];
    }

    // In valid demo lump we have:
    // - skill is a value from 0 ("I'm too young to die") to 4 ("Nightmare!")
    // - episode is at most 6, because Heretic has the most episodes (6)
    // - map is at most 33, because DOOM II from XBOX has the most maps (33)
    skill <= 4 && episode <= 6 && map <= 33
}

pub fn is_iwad_name(file_name: &str) -> bool {
    IWADS.iter().any(|iwad| iwad.name == file_name)
}

pub fn suggest_game_name(mission: GameMission, mode: GameMode) -> String {
    for iwad in IWADS {
        if iwad.mission == mission && (mode == GameMode::Indetermined || iwad.mode == mode) {
            return String::from(iwad.description);
        }
    }

    String::from("Unknown game?")
}

pub fn save_game_iwad_name(mission: GameMission, variant: GameVariant) -> String {
    if variant == GameVariant::FreeDoom {
        if mission == GameMission::Doom {
            return "freedoom1.wad".to_owned();
        }
        if mission == GameMission::Doom2 {
            return "freedoom2.wad".to_owned();
        }
    }
    if variant == GameVariant::FreeDm && mission == GameMission::Doom2 {
        return "freedm.wad".to_owned();
    }

    for iwad in IWADS {
        if mission == iwad.mission {
            return iwad.name.to_owned();
        }
    }

    "unknown.wad".to_owned()
}
