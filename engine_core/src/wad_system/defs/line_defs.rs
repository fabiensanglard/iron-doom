use derive_more::{AsRef, Deref, DerefMut, Index, IndexMut, IntoIterator};

const WAD_LINE_DEF_BYTE_SIZE: usize = 14;

pub struct WadLineDef {
    pub start_vertex: u16,
    pub end_vertex: u16,
    pub flags: i16,
    pub special: i16,
    pub sector_tag: i16,
    pub front_side_def: u16,
    pub back_side_def: u16,
}

#[derive(AsRef, Deref, DerefMut, IntoIterator, Index, IndexMut)]
#[into_iterator(owned, ref, ref_mut)]
pub struct WadLineDefs(Vec<WadLineDef>);

impl TryFrom<&[u8]> for WadLineDef {
    type Error = String;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        let len = value.len();

        if len != WAD_LINE_DEF_BYTE_SIZE {
            let err_msg = format!(
                "Error while converting WadLineDef: \
                 byte array must be {WAD_LINE_DEF_BYTE_SIZE} bytes wide!"
            );
            return Err(err_msg);
        }

        Ok(Self {
            start_vertex: u16::from_le_bytes([value[0], value[1]]),
            end_vertex: u16::from_le_bytes([value[2], value[3]]),
            flags: i16::from_le_bytes([value[4], value[5]]),
            special: i16::from_le_bytes([value[6], value[7]]),
            sector_tag: i16::from_le_bytes([value[8], value[9]]),
            front_side_def: u16::from_le_bytes([value[10], value[11]]),
            back_side_def: u16::from_le_bytes([value[12], value[13]]),
        })
    }
}

impl TryFrom<&[u8]> for WadLineDefs {
    type Error = String;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        let len = value.len();

        if len % WAD_LINE_DEF_BYTE_SIZE != 0 {
            let err_msg = format!(
                "Error while converting WadLineDefs: \
                 byte array must be divisible by {WAD_LINE_DEF_BYTE_SIZE}!"
            );
            return Err(err_msg);
        }

        let line_defs: Result<Vec<WadLineDef>, String> = value
            .chunks_exact(WAD_LINE_DEF_BYTE_SIZE)
            .map(|line_def_bytes| line_def_bytes.try_into())
            .collect();
        let line_defs = line_defs?;

        Ok(Self(line_defs))
    }
}
