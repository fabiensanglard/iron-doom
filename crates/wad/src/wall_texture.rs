use crate::index_map::IndexMap;
use crate::lump::{Lump, LumpsDirectory};
use anyhow::{bail, Result};
use column::WallTextureColumnsParser;
use common::Buffer;
use definition::{WallTextureDefinition, WallTextureDefinitionsParser};
use derive_more::{Deref, DerefMut};
use patch_names::{PatchNames, PatchNamesParser};

mod column;
mod definition;
mod patch_names;

#[derive(Debug)]
pub struct WallTextures(IndexMap<String, WallTexture>);

impl WallTextures {
    pub fn get(&self, tex: &str) -> Option<&WallTexture> {
        self.get_index_of(tex)
            .and_then(|tex_num| self.get_by_index(tex_num))
    }

    pub fn get_index_of(&self, tex: &str) -> Option<usize> {
        if self.0.is_empty() {
            return None;
        }
        if tex.starts_with("-") {
            // "NoTexture" marker.
            return Some(0);
        }
        self.0.get_index_of(tex)
    }

    pub fn get_by_index(&self, tex: usize) -> Option<&WallTexture> {
        self.0.get_index(tex).map(|(_, texture)| texture)
    }
}

pub struct WallTexturesParser;

impl WallTexturesParser {
    pub fn parse(lumps_dir: &LumpsDirectory) -> Result<WallTextures> {
        let mut textures = IndexMap::default();
        let mut patch_names = PatchNamesParser::parse(lumps_dir)?;

        Self::add_shareware(&mut textures, lumps_dir, &mut patch_names)?;
        Self::add_commercial(&mut textures, lumps_dir, &mut patch_names)?;

        Ok(WallTextures(textures))
    }

    fn add_shareware(
        textures: &mut IndexMap<String, WallTexture>,
        lumps_dir: &LumpsDirectory,
        patch_names: &mut PatchNames,
    ) -> Result<()> {
        let Some(shareware_lump) = lumps_dir.get("TEXTURE1") else {
            bail!("Missing shareware texture definitions");
        };
        Self::parse_textures(textures, shareware_lump, patch_names)
    }

    fn add_commercial(
        textures: &mut IndexMap<String, WallTexture>,
        lumps_dir: &LumpsDirectory,
        patch_names: &mut PatchNames,
    ) -> Result<()> {
        if let Some(commercial_lump) = lumps_dir.get("TEXTURE2") {
            Self::parse_textures(textures, commercial_lump, patch_names)?;
        }
        Ok(())
    }

    fn parse_textures(
        textures: &mut IndexMap<String, WallTexture>,
        lump_definitions: &Lump,
        patch_names: &mut PatchNames,
    ) -> Result<()> {
        let definitions = WallTextureDefinitionsParser::parse(lump_definitions.data())?;
        for texture_def in definitions.0 {
            let texture = WallTextureParser::parse(&texture_def, patch_names)?;
            textures.insert(texture_def.name.clone(), texture);
        }
        Ok(())
    }
}

#[derive(Deref, DerefMut, Debug)]
pub struct WallTexture(Buffer);

impl WallTexture {
    fn new(width: usize, height: usize) -> Self {
        let mut tex_data = Buffer::new(width, height);
        tex_data.fill(251);

        WallTexture(tex_data)
    }
}

struct WallTextureParser;

impl WallTextureParser {
    fn parse(tex_def: &WallTextureDefinition, patch_names: &mut PatchNames) -> Result<WallTexture> {
        let width = tex_def.width;
        let height = tex_def.height;
        let mut texture = WallTexture::new(width, height);

        let tex_cols = WallTextureColumnsParser::parse(tex_def, patch_names)?;
        for tex_column in tex_cols {
            tex_column.draw_to_texture(&mut texture, patch_names)?;
        }

        Ok(texture)
    }
}
