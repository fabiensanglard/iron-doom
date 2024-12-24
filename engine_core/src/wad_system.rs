use std::cell::RefCell;
use std::fs::File;
use std::hash::{Hash, Hasher};
use std::io::{Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};
use std::rc::Rc;

use bevy::app::{App, Plugin};
use quick_cache::unsync::Cache;
use quick_cache::Weighter;

pub use defs::*;

use crate::file_system;
use crate::file_system::{GameMission, GameMode};

mod defs;

const WAD_HEADER_SIZE: usize = 12;
const WAD_FILE_LUMP_SIZE: usize = 16;
const MB_SIZE_CACHE: u64 = 6; // MiB

/// Lump names that are unique to particular game types. This lets us check
/// the user is not trying to play with the wrong executable, e.g.
/// iron-doom --iwad hexen.wad.
static UNIQUE_LUMPS: [(GameMission, &str); 4] = [
    (GameMission::Doom, "POSSA1"),
    (GameMission::Heretic, "IMPXA1"),
    (GameMission::Hexen, "ETTNA1"),
    (GameMission::Strife, "AGRDA1"),
];

pub struct WadPlugin;

impl Plugin for WadPlugin {
    fn build(&self, app: &mut App) {
        app.insert_non_send_resource(WadSystem::init());
    }
}

#[derive(Debug)]
pub struct WadSystem {
    lump_cache: Cache<Rc<LumpInfo>, Rc<Vec<u8>>, LumpWeighter>,
    lump_info: Vec<Rc<LumpInfo>>,
    num_lumps: u32,
    reload_lump: u32,
    reload_name: Option<PathBuf>,
}

impl WadSystem {
    pub fn init() -> Self {
        Self {
            lump_cache: Cache::with_weighter(1000, MB_SIZE_CACHE * 1024 * 1024, LumpWeighter),
            lump_info: Vec::new(),
            num_lumps: 0,
            reload_lump: 0,
            reload_name: None,
        }
    }

    pub fn lump_info(&self) -> &[Rc<LumpInfo>] {
        &self.lump_info
    }

    fn check_reload_hack(&mut self, file_path: &Path) -> Result<PathBuf, String> {
        let file_str = file_path.to_str().ok_or(format!(
            "Error while checking reload hack: {file_path:?} is not valid unicode."
        ))?;

        // If the filename begins with a ~, it indicates that we should use the
        // reload hack.
        match file_str.strip_prefix('~') {
            None => Ok(PathBuf::from(file_str)),
            Some(file_str) => {
                if self.reload_name.is_some() {
                    let error_msg = "Prefixing a WAD filename with '~' indicates that the WAD \
                     should be reloaded\n\
                     on each level restart, for use by level authors for rapid \
                     development. You\n\
                     can only reload one WAD file, and it must be the last file \
                     in the --file list.";

                    return Err(error_msg.to_string());
                }
                self.reload_name = Some(file_path.to_path_buf());
                self.reload_lump = self.num_lumps;

                // Remove ~ leading character
                Ok(PathBuf::from(file_str))
            }
        }
    }

    /// All files are optional, but at least one file must be
    ///  found (PWAD, if all required lumps are present).
    /// Files with a .wad extension are wadlink files
    ///  with multiple lumps.
    /// Other files are single lumps with the base filename
    ///  for the lump name.
    pub fn add_file(&mut self, file_path: &Path) -> Result<Rc<WadFile>, String> {
        let file_path = self.check_reload_hack(file_path)?;
        let wad_file = match self.open_file(&file_path) {
            Ok(wad_file) => Rc::new(wad_file),
            Err(error) => {
                println!(" couldn't open {file_path:?}");
                return Err(error);
            }
        };
        let ext = file_path.extension().ok_or(format!(
            "Error while adding WAD file: {file_path:?} has no extension."
        ))?;
        let num_file_lumps;

        let file_info = if !ext.eq_ignore_ascii_case("wad") {
            num_file_lumps = 1;

            vec![FileLump {
                file_pos: 0,
                name: extract_file_base(&file_path)?,
                size: wad_file.length as usize,
            }]
        } else {
            // WAD file
            let header = read_wad_header(&wad_file)?;
            num_file_lumps = header.num_lumps;

            read_file_info(&wad_file, &header)?
        };

        self.num_lumps += num_file_lumps as u32;

        for lump in file_info {
            let lump_info = Rc::new(LumpInfo {
                wad_file: Rc::clone(&wad_file),
                position: lump.file_pos,
                size: lump.size,
                name: lump.name,
            });
            self.lump_info.push(lump_info);
        }

        Ok(wad_file)
    }

    pub fn open_file(&self, path: &Path) -> Result<WadFile, String> {
        let file = match File::open(path) {
            Ok(file) => file,
            Err(io_error) => {
                return Err(format!("Error while opening file {path:?}: {io_error}"));
            }
        };
        let length = get_file_length(&file)?;

        Ok(WadFile {
            file: RefCell::new(file),
            length,
            path: path.to_path_buf(),
        })
    }

    pub fn check_correct_iwad(&self, mission: GameMission) -> Result<(), String> {
        for (lump_mission, lump_name) in UNIQUE_LUMPS {
            if lump_mission == mission {
                continue;
            }
            if self.lump_exists(lump_name) {
                let prefix = env!("WORKSPACE_PREFIX");
                let suggested_name =
                    file_system::suggest_game_name(lump_mission, GameMode::Indetermined);

                return Err(format!(
                    "\nYou are trying to use a {suggested_name} IWAD file with \
                     the {prefix}{mission} binary.\nThis isn't going to work.\n\
                     You probably want to use the {prefix}{lump_mission} binary."
                ));
            }
        }

        Ok(())
    }

    pub fn get_lump(&self, idx: usize) -> Option<Rc<LumpInfo>> {
        self.lump_info.get(idx).map(Rc::clone)
    }

    pub fn get_lump_or_err(&self, idx: usize) -> Result<Rc<LumpInfo>, String> {
        self.get_lump(idx)
            .ok_or(format!("Could not find lump with index {idx}!"))
    }

    pub fn get_lump_by_name(&self, name: &str) -> Option<Rc<LumpInfo>> {
        // scan backwards so patch lump files take precedence
        for lump in self.lump_info.iter().rev() {
            if lump.name.eq_ignore_ascii_case(name) {
                return Some(Rc::clone(lump));
            }
        }

        None
    }

    pub fn get_lump_by_name_or_err(&self, name: &str) -> Result<Rc<LumpInfo>, String> {
        self.get_lump_by_name(name)
            .ok_or(format!("Could not find lump with name {name}!"))
    }

    pub fn get_lump_idx(&self, name: &str) -> Option<usize> {
        // scan backwards so patch lump files take precedence
        for (i, lump) in self.lump_info.iter().enumerate().rev() {
            if lump.name.eq_ignore_ascii_case(name) {
                return Some(i);
            }
        }

        None
    }

    pub fn get_lump_idx_or_err(&self, name: &str) -> Result<usize, String> {
        self.get_lump_idx(name)
            .ok_or(format!("Could not find lump index for {name}!"))
    }

    pub fn cache_lump_name(&mut self, lump_name: &str) -> Result<Rc<Vec<u8>>, String> {
        let lump = self.get_lump_by_name_or_err(lump_name)?;

        self.get_cache_lump_data(lump)
    }

    pub fn cache_lump_idx(&mut self, lump_idx: usize) -> Result<Rc<Vec<u8>>, String> {
        let lump = self.get_lump_or_err(lump_idx)?;

        self.get_cache_lump_data(lump)
    }

    pub fn lump_exists(&self, name: &str) -> bool {
        self.get_lump_by_name(name).is_some()
    }

    fn get_cache_lump_data(&mut self, lump: Rc<LumpInfo>) -> Result<Rc<Vec<u8>>, String> {
        // Already cached
        if let Some(lump_data) = self.lump_cache.get(&lump) {
            return Ok(Rc::clone(lump_data));
        }

        // Not yet loaded, so load it now
        let mut lump_data = vec![0; lump.len()];
        lump.read(&mut lump_data)?;
        let lump_data = Rc::new(lump_data);
        self.lump_cache.insert(lump, Rc::clone(&lump_data));

        Ok(lump_data)
    }
}

#[derive(Debug)]
pub struct WadFile {
    file: RefCell<File>,
    length: i32,
    path: PathBuf,
}

impl PartialEq for WadFile {
    fn eq(&self, other: &Self) -> bool {
        self.path == other.path
    }
}

impl Eq for WadFile {}

impl Hash for WadFile {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.path.hash(state);
        self.length.hash(state);
    }
}

impl WadFile {
    pub fn read(&self, offset: u32, buffer: &mut [u8]) -> Result<usize, String> {
        let mut handle = self.file.borrow_mut();

        if handle.seek(SeekFrom::Start(offset as u64)).is_err() {
            return Err(format!("Failed to seek offset {offset}"));
        }

        handle
            .read(buffer)
            .map_err(|io_error| format!("Error reading from file: {io_error}"))
    }
}

struct FileLump {
    file_pos: u32,
    name: String,
    size: usize,
}

struct WadInfo {
    identification: String,
    info_table_ofs: i32,
    num_lumps: i32,
}

#[derive(Debug, PartialEq, Eq, Hash)]
pub struct LumpInfo {
    name: String,
    wad_file: Rc<WadFile>,
    position: u32,
    size: usize,
}

impl LumpInfo {
    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn len(&self) -> usize {
        self.size
    }

    pub fn read(&self, buffer: &mut [u8]) -> Result<(), String> {
        let off = self.position;
        let bytes_read = self.wad_file.read(off, buffer)?;
        let size = self.size;

        if bytes_read < size {
            return Err(format!(
                "W_ReadLump: only read {bytes_read} of {size} on lump {}",
                self.name
            ));
        }

        Ok(())
    }
}

#[derive(Debug)]
struct LumpWeighter;

impl Weighter<Rc<LumpInfo>, Rc<Vec<u8>>> for LumpWeighter {
    fn weight(&self, _lump: &Rc<LumpInfo>, lump_data: &Rc<Vec<u8>>) -> u64 {
        lump_data.len().clamp(0, u64::MAX as usize) as u64
    }
}

fn get_file_length(file: &File) -> Result<i32, String> {
    match file.metadata() {
        Ok(meta) => meta
            .len()
            .try_into()
            .map_err(|_| String::from("WAD file is too big!")),
        Err(io_error) => Err(format!(
            "Error while retrieving metadata from file {file:?}: {io_error}"
        )),
    }
}

fn extract_file_base(file_path: &Path) -> Result<String, String> {
    // Copy up to eight characters
    // Note: Vanilla Doom exits with an error if a filename is specified
    // with a base of more than eight characters.  To remove the 8.3
    // filename limit, instead we simply truncate the name.

    let name = file_path
        .file_name()
        .ok_or(format!(
            "Failed to extract file name: no extension for file {file_path:?}"
        ))?
        .to_str()
        .ok_or(format!(
            "Failed to extract file name: {file_path:?} is not valid Unicode"
        ))?;

    let truncated_name = if name.len() >= 8 {
        let truncated = &name[0..8];
        println!("Warning: Truncated '{name}' lump name to '{truncated}'.");

        truncated
    } else {
        name
    };

    Ok(truncated_name.to_uppercase())
}

fn read_wad_header(wad_file: &WadFile) -> Result<WadInfo, String> {
    // The first 12 bytes of a WAD file contains the header. It has the
    // following structure (credits goes to Doom Wiki):
    //
    // -------------------------------------------------------------------------------------------------------
    // | Position | Length |      Name      |                         Description                            |
    // | -------- | ------ | -------------- | ---------------------------------------------------------------|
    // | 0x00     | 4      | identification | The ASCII characters "IWAD" or "PWAD".                         |
    // | -------- | ------ | -------------- | ---------------------------------------------------------------|
    // | 0x04     | 4      | numlumps       | An integer specifying the number of lumps in the WAD.          |
    // | -------- | ------ | -------------- | ---------------------------------------------------------------|
    // | 0x08     | 4      | infotableofs   | An integer holding a pointer to the location of the directory. |
    // -------------------------------------------------------------------------------------------------------
    //
    // Also, all WAD files are little endian, so we must have some care
    // when reading the header and consider the use case when the computer
    // is big endian.

    let mut buf = [0u8; WAD_HEADER_SIZE];
    wad_file.read(0, &mut buf)?;

    let identification = std::str::from_utf8(&buf[0..4])
        .map_err(|utf8_error| format!("Failed while retrieving Wad file id: {utf8_error}"))?
        .to_string();
    let num_lumps = i32::from_le_bytes((&buf[4..8]).try_into().unwrap());
    let info_table_ofs = i32::from_le_bytes((&buf[8..]).try_into().unwrap());

    let header = WadInfo {
        info_table_ofs,
        identification,
        num_lumps,
    };

    if header.identification != "IWAD" && header.identification != "PWAD" {
        return Err(format!(
            "Wad file {:?} doesn't have IWAD or PWAD id\n",
            wad_file.path
        ));
    }
    // Vanilla Doom doesn't like WADs with more than 4046 lumps
    // https://doomwiki.org/wiki/PWAD_size_limit
    if header.identification == "PWAD" && header.num_lumps > 4046 {
        return Err(format!(
            "Error: Vanilla limit for lumps in a WAD is 4046, PWAD {:?} has {}",
            wad_file.path, header.num_lumps
        ));
    }

    Ok(header)
}

fn read_file_info(wad_file: &WadFile, header: &WadInfo) -> Result<Vec<FileLump>, String> {
    // The first 16 bytes of a WAD file contains the directory. It has the
    // following structure (credits goes to Doom Wiki):
    //
    // -----------------------------------------------------------------------------------------------------------------
    // | Position | Length |   Name  |                                   Description                                   |
    // | -------- | ------ | ------- | ------------------------------------------------------------------------------- |
    // | 0x00     | 4      | filepos | An integer holding a pointer to the start of the lump's data in the file.       |
    // | -------- | ------ | ------- | ------------------------------------------------------------------------------- |
    // | 0x04     | 4      | size    | An integer representing the size of the lump in bytes.                          |
    // | -------- | ------ | ------- | ------------------------------------------------------------------------------- |
    // | 0x08     | 8      | name    | An ASCII string defining the lump's name, the name has a limit of 8 characters. |
    // -----------------------------------------------------------------------------------------------------------------
    //
    // Also, all WAD files are little endian, so we must have some care
    // when reading the header and consider the use case when the computer
    // is big endian.

    let length = (header.num_lumps as usize * WAD_FILE_LUMP_SIZE) as i32;
    let mut file_info = Vec::with_capacity(length as usize);

    let mut buf = vec![0u8; length as usize];
    wad_file.read(header.info_table_ofs as u32, &mut buf)?;

    for i in (0..buf.len()).step_by(WAD_FILE_LUMP_SIZE) {
        let file_pos = u32::from_le_bytes((&buf[i..i + 4]).try_into().unwrap());
        let size = u32::from_le_bytes((&buf[i + 4..i + 8]).try_into().unwrap());
        let name = std::str::from_utf8(&buf[i + 8..i + 16])
            .map_err(|utf8_error| format!("Failed while retrieving lump name: {utf8_error}"))?
            .trim_matches('\0')
            .to_string();

        file_info.push(FileLump {
            name,
            file_pos,
            size: size as usize,
        });
    }

    Ok(file_info)
}
