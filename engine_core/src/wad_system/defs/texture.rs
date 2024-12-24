use derive_more::{AsRef, Deref, DerefMut, Index, IndexMut, Into, IntoIterator};

use crate::wad_system::defs::convert_c_string;
use crate::wad_system::WadTexturePatch;

use super::WAD_TEXTURE_PATCH_BYTE_SIZE;

const WAD_TEXTURE_MIN_BYTE_SIZE: usize = 22;

#[derive(AsRef, Deref, DerefMut, Into, IntoIterator, Index, IndexMut)]
#[into_iterator(owned, ref, ref_mut)]
pub struct WadTextures(Vec<WadTexture>);

pub struct WadTexture {
    /// An ASCII string defining the name of the map texture.
    pub name: String,

    /// Obsolete, ignored by all DOOM versions.
    pub masked: bool,

    /// A short integer defining the total width of the map texture.
    pub width: u16,

    /// A short integer defining the total height of the map texture.
    pub height: u16,

    /// Obsolete, ignored by all DOOM versions.
    pub column_directory: i32,

    /// Array with the map patch structures for this texture.
    pub patches: Vec<WadTexturePatch>,
}

impl TryFrom<&[u8]> for WadTextures {
    type Error = String;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        let len = value.len();

        if len < 4 {
            let err_msg = "Error while converting WadTextures: \
                 byte array must have at least 4 elements!";
            return Err(err_msg.to_string());
        }

        let num_textures = u32::from_le_bytes([value[0], value[1], value[2], value[3]]);
        let end_dir = 4 * (1 + num_textures as usize);

        let mut textures = vec![];
        let dir = &value[4..end_dir];

        for offset_bytes in dir.chunks_exact(4) {
            let offset = u32::from_le_bytes([
                offset_bytes[0],
                offset_bytes[1],
                offset_bytes[2],
                offset_bytes[3],
            ]);
            let offset = offset as usize;

            if offset > len {
                let err_msg = format!(
                    "Error while converting WadTextures: texture offset \
                     {offset} points outside byte array!"
                );
                return Err(err_msg);
            }

            let map_texture_bytes = &value[offset..];
            let map_texture: WadTexture = map_texture_bytes.try_into()?;

            textures.push(map_texture);
        }

        Ok(Self(textures))
    }
}

impl TryFrom<&[u8]> for WadTexture {
    type Error = String;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        let len = value.len();

        if len < WAD_TEXTURE_MIN_BYTE_SIZE {
            let err_msg = format!(
                "Error while converting WadTexture: byte array must \
                 be at least {WAD_TEXTURE_MIN_BYTE_SIZE} bytes wide!"
            );
            return Err(err_msg);
        }

        let patch_count = u16::from_le_bytes([value[20], value[21]]);

        let patches: Result<Vec<WadTexturePatch>, String> = value[22..]
            .chunks_exact(WAD_TEXTURE_PATCH_BYTE_SIZE)
            .take(patch_count as usize)
            .map(|patch_bytes| patch_bytes.try_into())
            .collect();
        let patches = patches?;

        let patches_len = patches.len();

        if patches_len != patch_count as usize {
            let err_msg = format!(
                "Error while converting WadTexture: expected {patch_count} \
                 texture patches, but found {patches_len}!"
            );
            return Err(err_msg);
        }

        let masked = u32::from_le_bytes([value[8], value[9], value[10], value[11]]);

        Ok(Self {
            name: convert_c_string(&value[0..8])?,
            masked: masked == 0,
            width: u16::from_le_bytes([value[12], value[13]]),
            height: u16::from_le_bytes([value[14], value[15]]),
            column_directory: i32::from_le_bytes([value[16], value[17], value[18], value[19]]),
            patches,
        })
    }
}
