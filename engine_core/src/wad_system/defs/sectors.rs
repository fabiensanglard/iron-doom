use derive_more::{AsRef, Deref, DerefMut, Index, IndexMut, IntoIterator};

use crate::wad_system::defs::convert_c_string;

const WAD_SECTOR_BYTE_SIZE: usize = 26;

#[derive(AsRef, Deref, DerefMut, IntoIterator, Index, IndexMut)]
#[into_iterator(owned, ref, ref_mut)]
pub struct WadSectors(Vec<WadSector>);

/// A sector is an area referenced by sidedefs on the linedefs, with variable
/// height determined by its floor and ceiling values.
pub struct WadSector {
    /// Floor height.
    pub floor_height: i16,

    /// Ceiling height.
    pub ceiling_height: i16,

    /// Name of floor texture.
    pub floor_pic: String,

    /// Name of ceiling texture.
    pub ceiling_pic: String,

    /// Light level.
    pub light_level: i16,

    /// Special Type.
    pub special: i16,

    /// Tag number.
    pub tag: i16,
}

impl TryFrom<&[u8]> for WadSectors {
    type Error = String;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        let len = value.len();

        if len % WAD_SECTOR_BYTE_SIZE != 0 {
            let err_msg = format!(
                "Error while converting MapSegments: \
                 byte array must be divisible by {WAD_SECTOR_BYTE_SIZE}!"
            );
            return Err(err_msg);
        }

        let sectors: Result<Vec<WadSector>, String> = value
            .chunks_exact(WAD_SECTOR_BYTE_SIZE)
            .map(|sector_bytes| sector_bytes.try_into())
            .collect();
        let sectors = sectors?;

        Ok(WadSectors(sectors))
    }
}

impl TryFrom<&[u8]> for WadSector {
    type Error = String;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        let len = value.len();

        if len != WAD_SECTOR_BYTE_SIZE {
            let err_msg = format!(
                "Error while converting WadSector: \
                 byte array must be {WAD_SECTOR_BYTE_SIZE} bytes wide!"
            );
            return Err(err_msg);
        }

        Ok(WadSector {
            floor_height: i16::from_le_bytes([value[0], value[1]]),
            ceiling_height: i16::from_le_bytes([value[2], value[3]]),
            floor_pic: convert_c_string(&value[4..12])?,
            ceiling_pic: convert_c_string(&value[12..20])?,
            light_level: i16::from_le_bytes([value[20], value[21]]),
            special: i16::from_le_bytes([value[22], value[23]]),
            tag: i16::from_le_bytes([value[24], value[25]]),
        })
    }
}
