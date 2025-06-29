use super::{
    definition::{PatchDescriptor, WallTextureDefinition},
    patch_names::PatchNames,
    WallTexture,
};
use anyhow::Result;
use bevy::utils::HashMap;
use derive_more::{Deref, IntoIterator};

#[derive(Deref, IntoIterator)]
pub struct WallTextureColumns<'a>(Vec<WallTextureColumn<'a>>);

pub struct WallTextureColumn<'a> {
    num: usize,
    descriptors: Vec<&'a PatchDescriptor>,
}

impl WallTextureColumn<'_> {
    fn new(num: usize) -> Self {
        Self {
            num,
            descriptors: vec![],
        }
    }

    fn is_composite(&self) -> bool {
        self.descriptors.len() > 1
    }

    pub fn draw_to_texture(
        &self,
        tex: &mut WallTexture,
        patch_names: &mut PatchNames,
    ) -> Result<()> {
        for patch_descriptor in &self.descriptors {
            // Vanilla Bug: the vertical offset is ignored when the texture
            // column is not composite, i.e. the texture  column is defined
            // with a  single patch. We choose not fix it, as it would break
            // many PWADs that depends on vanilla behaviour.
            let origin_y = if self.is_composite() {
                patch_descriptor.origin_y as isize
            } else {
                0
            };
            let origin_x = patch_descriptor.origin_x as isize;
            let patch_num = patch_descriptor.patch_num;

            // Get corresponding patch column
            let tex_num = self.num;
            let patch_col = tex_num.saturating_add_signed(-origin_x);
            let patch = patch_names.get_patch(patch_num)?;
            let patch_col = patch.column(patch_col);

            patch_col.draw_to_texture(tex, tex_num, origin_y);
        }

        Ok(())
    }
}

pub struct WallTextureColumnsParser;

impl WallTextureColumnsParser {
    pub fn parse<'a>(
        tex_def: &'a WallTextureDefinition,
        patch_names: &mut PatchNames,
    ) -> Result<WallTextureColumns<'a>> {
        // Use a HashMap for optimized insertion
        let mut tex_cols = HashMap::with_capacity(tex_def.width);

        for patch_descriptor in &tex_def.patch_descriptors {
            // We need first to know which texture columns this patch
            // descriptors covers. So first get the patch it refers to.
            let patch_num = patch_descriptor.patch_num;
            let patch = patch_names.get_patch(patch_num)?;
            let width = patch.width;

            // Here, x is the left most texture column and x2 the right
            // most texture column that the descriptor covers.
            let x1 = patch_descriptor.origin_x;
            let mut x2 = width.saturating_add_signed(x1 as isize);
            if x2 > tex_def.width {
                x2 = tex_def.width;
            }
            let x = if x1 < 0 { 0 } else { x1 as usize };

            for i in x..x2 {
                let column = tex_cols.entry(i).or_insert(WallTextureColumn::new(i));
                column.descriptors.push(patch_descriptor);
            }
        }

        // Convert to array
        let tex_cols = tex_cols.into_values().collect();
        Ok(WallTextureColumns(tex_cols))
    }
}
