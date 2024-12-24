use derive_more::{AsRef, Deref, DerefMut, Index, IndexMut, IntoIterator};

use crate::wad_system::defs::convert_c_string;

/// WAD lump that includes all the names for wall patches (PNAMES).
#[derive(AsRef, Deref, DerefMut, IntoIterator, Index, IndexMut)]
#[into_iterator(owned, ref, ref_mut)]
pub struct WallPatchesNames(Vec<String>);

impl TryFrom<&[u8]> for WallPatchesNames {
    type Error = String;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        let len = value.len();

        if len < 4 {
            let err_msg = "Error while converting WallPatchesNames: \
                byte array must be at least 4 bytes!";
            return Err(err_msg.to_string());
        }

        let header_num_patches = u32::from_le_bytes([value[0], value[1], value[2], value[3]]);

        let wall_names: Result<Vec<String>, String> = value[4..]
            .chunks_exact(8)
            .map(convert_c_string)
            .collect();
        let wall_names = wall_names?;

        let num_patches = wall_names.len();

        if num_patches != header_num_patches as usize {
            let err_msg = format!(
                "Error while converting WallPatches: number of wall patches \
                 indicated by header ({header_num_patches}) differs from \
                 number of found wall patches ({num_patches})"
            );
            return Err(err_msg);
        }

        Ok(WallPatchesNames(wall_names))
    }
}
