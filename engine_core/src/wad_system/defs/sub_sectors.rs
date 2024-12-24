use derive_more::{AsRef, Deref, DerefMut, Index, IndexMut, IntoIterator};

const WAD_SUB_SECTOR_BYTE_SIZE: usize = 4;

#[derive(AsRef, Deref, DerefMut, IntoIterator, Index, IndexMut)]
#[into_iterator(owned, ref, ref_mut)]
pub struct WadSubSectors(Vec<WadSubSector>);

pub struct WadSubSector {
    pub num_segs: u16,
    pub first_line: u16,
}

impl TryFrom<&[u8]> for WadSubSectors {
    type Error = String;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        let len = value.len();

        if len % WAD_SUB_SECTOR_BYTE_SIZE != 0 {
            let err_msg = format!(
                "Error while converting WadSubSectors: \
                 byte array must be divisible by {WAD_SUB_SECTOR_BYTE_SIZE}!"
            );
            return Err(err_msg);
        }

        let sub_sectors: Result<Vec<WadSubSector>, String> = value
            .chunks_exact(WAD_SUB_SECTOR_BYTE_SIZE)
            .map(|sub_sector_bytes| sub_sector_bytes.try_into())
            .collect();
        let sub_sectors = sub_sectors?;

        Ok(Self(sub_sectors))
    }
}

impl TryFrom<&[u8]> for WadSubSector {
    type Error = String;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        let len = value.len();

        if len != WAD_SUB_SECTOR_BYTE_SIZE {
            let err_msg = format!(
                "Error while converting WadSubSectors: \
                 byte array must be {WAD_SUB_SECTOR_BYTE_SIZE} bytes wide!"
            );
            return Err(err_msg);
        }

        Ok(Self {
            num_segs: u16::from_le_bytes([value[0], value[1]]),
            first_line: u16::from_le_bytes([value[2], value[3]]),
        })
    }
}
