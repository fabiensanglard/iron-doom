pub(super) const WAD_TEXTURE_PATCH_BYTE_SIZE: usize = 10;

pub struct WadTexturePatch {
    /// A short int defining the horizontal offset of the patch
    /// relative to the upper-left of the texture.
    pub origin_x: i16,

    /// A short int defining the vertical offset of the patch
    /// relative to the upper-left of the texture.
    pub origin_y: i16,

    /// A short int defining the patch number
    /// (as listed in PNAMES) to draw.
    pub patch: u16,

    /// A short int possibly intended to define if the patch
    /// was to be drawn normally or mirrored.
    ///
    /// Unused.
    pub step_dir: i16,

    /// A short int possibly intended to define a special
    /// colormap to draw the patch with, like a brightmap.
    ///
    /// Unused.
    pub color_map: i16,
}

impl TryFrom<&[u8]> for WadTexturePatch {
    type Error = String;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        let len = value.len();

        if len != WAD_TEXTURE_PATCH_BYTE_SIZE {
            let err_msg = format!(
                "Error while converting WadTexturePatch: \
                 byte array must be {WAD_TEXTURE_PATCH_BYTE_SIZE} bytes wide!"
            );
            return Err(err_msg);
        }

        Ok(Self {
            origin_x: i16::from_le_bytes([value[0], value[1]]),
            origin_y: i16::from_le_bytes([value[2], value[3]]),
            patch: u16::from_le_bytes([value[4], value[5]]),
            step_dir: i16::from_le_bytes([value[6], value[7]]),
            color_map: i16::from_le_bytes([value[8], value[9]]),
        })
    }
}
