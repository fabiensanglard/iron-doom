use derive_more::{AsRef, Deref, DerefMut, Index, IndexMut, IntoIterator};

use crate::wad_system::defs::convert_c_string;

const WAD_SIDE_DEF_BYTE_SIZE: usize = 30;

#[derive(AsRef, Deref, DerefMut, IntoIterator, Index, IndexMut)]
#[into_iterator(owned, ref, ref_mut)]
pub struct WadSideDefs(Vec<WadSideDef>);

pub struct WadSideDef {
    /// x offset.
    pub x_offset: i16,

    /// y offset.
    pub y_offset: i16,

    /// Name of upper texture.
    pub top_texture: String,

    /// Name of lower texture.
    pub bottom_texture: String,

    /// Name of middle texture.
    pub middle_texture: String,

    /// Sector number this sidedef 'faces'.
    pub sector: u16,
}

impl TryFrom<&[u8]> for WadSideDefs {
    type Error = String;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        let len = value.len();

        if len % WAD_SIDE_DEF_BYTE_SIZE != 0 {
            let err_msg = format!(
                "Error while converting WadSideDefs: \
                 byte array must be divisible by {WAD_SIDE_DEF_BYTE_SIZE}!"
            );
            return Err(err_msg);
        }

        let sides: Result<Vec<WadSideDef>, String> = value
            .chunks_exact(WAD_SIDE_DEF_BYTE_SIZE)
            .map(|side_bytes| side_bytes.try_into())
            .collect();
        let sides = sides?;

        Ok(WadSideDefs(sides))
    }
}

impl TryFrom<&[u8]> for WadSideDef {
    type Error = String;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        let len = value.len();

        if len != WAD_SIDE_DEF_BYTE_SIZE {
            let err_msg = format!(
                "Error while converting WadSideDef: \
                 byte array must be {WAD_SIDE_DEF_BYTE_SIZE} bytes wide!"
            );
            return Err(err_msg);
        }

        let x_offset = i16::from_le_bytes([value[0], value[1]]);
        let y_offset = i16::from_le_bytes([value[2], value[3]]);
        let top_texture = convert_c_string(&value[4..12])?;
        let bottom_texture = convert_c_string(&value[12..20])?;
        let middle_texture = convert_c_string(&value[20..28])?;
        let sector = u16::from_le_bytes([value[28], value[29]]);

        Ok(WadSideDef {
            x_offset,
            y_offset,
            top_texture,
            bottom_texture,
            middle_texture,
            sector,
        })
    }
}
