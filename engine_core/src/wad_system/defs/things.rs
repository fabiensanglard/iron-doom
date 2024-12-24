use derive_more::{AsRef, Deref, DerefMut, Index, IndexMut, IntoIterator};

const WAD_THING_BYTE_SIZE: usize = 10;

#[derive(Default, AsRef, Deref, DerefMut, IntoIterator, Index, IndexMut)]
#[into_iterator(owned, ref, ref_mut)]
pub struct WadThings(Vec<WadThing>);

#[derive(Copy, Clone)]
pub struct WadThing {
    /// x position.
    pub x: i16,

    /// x position.
    pub y: i16,

    /// Angle facing.
    pub angle: u16,

    /// DoomEd thing type.
    pub thing_type: u16,

    /// Flags.
    pub flags: u16,
}

impl TryFrom<&[u8]> for WadThings {
    type Error = String;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        let len = value.len();

        if len % WAD_THING_BYTE_SIZE != 0 {
            let err_msg = format!(
                "Error while converting WadThings: \
                 byte array must be divisible by {WAD_THING_BYTE_SIZE}!"
            );
            return Err(err_msg);
        }

        let things: Result<Vec<WadThing>, String> = value
            .chunks_exact(WAD_THING_BYTE_SIZE)
            .map(|thing_bytes| thing_bytes.try_into())
            .collect();
        let things = things?;

        Ok(Self(things))
    }
}

impl TryFrom<&[u8]> for WadThing {
    type Error = String;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        let len = value.len();

        if len != WAD_THING_BYTE_SIZE {
            let err_msg = format!(
                "Error while converting WadThing: \
                 byte array must be {WAD_THING_BYTE_SIZE} bytes wide!"
            );
            return Err(err_msg);
        }

        Ok(Self {
            x: i16::from_le_bytes([value[0], value[1]]),
            y: i16::from_le_bytes([value[2], value[3]]),
            angle: u16::from_le_bytes([value[4], value[5]]),
            thing_type: u16::from_le_bytes([value[6], value[7]]),
            flags: u16::from_le_bytes([value[8], value[9]]),
        })
    }
}
