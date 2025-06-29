use crate::util::{bytes_to_i32, bytes_to_str};
use anyhow::{bail, Result};

#[derive(Copy, Clone, Debug)]
pub struct WadHeader {
    pub dir_offset: usize,
    pub num_lumps: usize,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum WadId {
    Iwad,
    Pwad,
}

pub struct WadHeaderParser;

impl WadHeaderParser {
    pub fn parse(bytes: &[u8]) -> Result<WadHeader> {
        let header_bytes = &bytes[0..12];

        let id_bytes = &header_bytes[0..4];
        let id = WadHeaderParser::parse_id(id_bytes)?;

        let num_lumps_bytes = &header_bytes[4..8];
        let num_lumps = WadHeaderParser::parse_num_lumps(num_lumps_bytes, id)?;

        let dir_offset_bytes = &header_bytes[8..12];
        let dir_offset = WadHeaderParser::parse_dir_offset(dir_offset_bytes)?;

        Ok(WadHeader {
            dir_offset,
            num_lumps,
        })
    }

    fn parse_id(bytes: &[u8]) -> Result<WadId> {
        let id = bytes_to_str(bytes)?;
        if id == "IWAD" {
            return Ok(WadId::Iwad);
        }
        if id == "PWAD" {
            return Ok(WadId::Pwad);
        }
        bail!("WAD has invalid identification {id}");
    }

    fn parse_num_lumps(bytes: &[u8], id: WadId) -> Result<usize> {
        let num_lumps = bytes_to_i32(bytes)?;
        if num_lumps < 0 {
            bail!("WAD has invalid numbers of lumps");
        }
        if id == WadId::Pwad && num_lumps > 4096 {
            // Vanilla Doom doesn't like PWADs with more than 4046 lumps.
            // https://www.doomworld.com/vb/post/1010985
            bail!("PWAD cannot have more than 4046 lumps, but this one has {num_lumps}");
        }
        Ok(num_lumps as usize)
    }

    fn parse_dir_offset(bytes: &[u8]) -> Result<usize> {
        let dir_offset = bytes_to_i32(bytes)?;
        if dir_offset < 0 {
            bail!("WAD has invalid directory offset");
        }
        Ok(dir_offset as usize)
    }
}
