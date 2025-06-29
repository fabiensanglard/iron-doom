use crate::{
    lump::LumpsDirectory,
    patch::{Patch, PatchParser},
    util::{bytes_to_i32, bytes_to_str},
};
use anyhow::{bail, Result};
use bevy::utils::HashMap;

pub struct PatchNames<'a> {
    patch_map: HashMap<usize, String>,
    patch_cache: HashMap<String, Patch>,
    lumps_dir: &'a LumpsDirectory<'a>,
}

impl PatchNames<'_> {
    pub fn get_patch(&mut self, patch_num: usize) -> Result<&Patch> {
        let Some(patch_name) = self.patch_map.get(&patch_num) else {
            bail!("Invalid patch index when retrieving patch");
        };
        if !self.patch_cache.contains_key(patch_name) {
            let Some(patch_lump) = self.lumps_dir.get(patch_name) else {
                bail!("Invalid patch name when retrieving patch");
            };
            let patch = PatchParser::parse(patch_lump.data())?;
            self.patch_cache.insert(patch_name.clone(), patch);
        }
        let patch = self.patch_cache.get(patch_name).unwrap();
        Ok(patch)
    }
}

pub struct PatchNamesParser;

impl PatchNamesParser {
    pub fn parse<'a>(lumps_dir: &'a LumpsDirectory) -> Result<PatchNames<'a>> {
        let Some(patch_names_lump) = lumps_dir.get("PNAMES") else {
            bail!("Missing PNAMES lump");
        };

        let lump_data = patch_names_lump.data();

        let num_patches_bytes = &lump_data[0..4];
        let num_patches = Self::parse_num_patches(num_patches_bytes)?;

        let patch_names_bytes = &lump_data[4..];
        let patch_map = Self::parse_names(patch_names_bytes, num_patches)?;

        let patch_cache = HashMap::with_capacity(num_patches);

        Ok(PatchNames {
            patch_map,
            patch_cache,
            lumps_dir,
        })
    }

    fn parse_num_patches(bytes: &[u8]) -> Result<usize> {
        let num_patches = bytes_to_i32(bytes)?;
        if num_patches < 0 {
            bail!("PNAMES lump has invalid number of patches");
        }
        Ok(num_patches as usize)
    }

    fn parse_names(names_bytes: &[u8], num_patches: usize) -> Result<HashMap<usize, String>> {
        let mut patch_map = HashMap::with_capacity(num_patches);
        for (index, patch_name_bytes) in names_bytes.chunks_exact(8).enumerate() {
            let patch_name = bytes_to_str(patch_name_bytes)?;
            patch_map.insert(index, patch_name.to_owned());
        }
        Ok(patch_map)
    }
}
