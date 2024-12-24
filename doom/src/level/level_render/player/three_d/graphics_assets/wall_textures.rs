use bevy::prelude::{NonSendMut, ResMut};
use rustc_hash::FxHashMap;

use engine_core::wad_system::{WadPatch, WadSystem, WadTexture, WadTextures, WallPatchesNames};

use super::{GraphicsAssets, WallTexture, WallTexturePatch};

pub fn load(
    mut wad_sys: NonSendMut<WadSystem>,
    mut assets: ResMut<GraphicsAssets>,
) -> Result<(), String> {
    assets.wall_textures = create_texture_map(&mut wad_sys)?;
    Ok(())
}

fn create_texture_map(wad_sys: &mut WadSystem) -> Result<FxHashMap<String, WallTexture>, String> {
    let wad_textures = get_wad_textures(wad_sys)?;
    let patches_lut = create_patch_lut(wad_sys)?;

    let mut texture_map = FxHashMap::default();
    for wad_texture in wad_textures {
        if texture_map.contains_key(&wad_texture.name) {
            continue;
        }
        let patches = create_patches(&wad_texture, &patches_lut)?;
        let cols = create_texture_cols(wad_sys, wad_texture.width, wad_texture.height, &patches);
        let wall_texture = WallTexture {
            width: wad_texture.width,
            height: wad_texture.height,
            name: wad_texture.name.clone(),
            patches,
            cols,
        };
        texture_map.insert(wad_texture.name.clone(), wall_texture);
    }

    Ok(texture_map)
}

fn get_wad_textures(wad_sys: &mut WadSystem) -> Result<WadTextures, String> {
    //  TEXTURE1 for shareware.
    let lump = wad_sys.cache_lump_name("TEXTURE1")?;
    let mut wad_textures: WadTextures = lump.as_slice().try_into()?;

    // TEXTURE2 for commercial.
    if let Ok(lump) = wad_sys.cache_lump_name("TEXTURE2") {
        let mut textures2: WadTextures = lump.as_slice().try_into()?;
        wad_textures.append(&mut textures2);
    }

    Ok(wad_textures)
}

fn create_patch_lut(wad_sys: &mut WadSystem) -> Result<Vec<usize>, String> {
    let lump = wad_sys.cache_lump_name("PNAMES")?;
    let patch_names: WallPatchesNames = lump.as_slice().try_into()?;

    let mut patches_name = vec![];
    for name in patch_names {
        let Some(lump_idx) = wad_sys.get_lump_idx(&name) else {
            continue;
        };
        patches_name.push(lump_idx);
    }

    Ok(patches_name)
}

fn create_patches(
    wad_texture: &WadTexture,
    patch_lut: &[usize],
) -> Result<Vec<WallTexturePatch>, String> {
    let err_msg = format!(
        "create_patches: Missing patch in texture {}",
        wad_texture.name
    );

    let mut wall_patches = vec![];
    for texture_patch in &wad_texture.patches {
        let Some(idx) = patch_lut.get(texture_patch.patch as usize) else {
            return Err(err_msg);
        };
        wall_patches.push(WallTexturePatch {
            origin_x: texture_patch.origin_x,
            origin_y: texture_patch.origin_y,
            lump_idx: *idx,
        })
    }

    Ok(wall_patches)
}

fn create_texture_cols(
    wad_sys: &mut WadSystem,
    tex_width: u16,
    tex_height: u16,
    patches: &[WallTexturePatch],
) -> FxHashMap<usize, Vec<u8>> {
    let mut cols_data: FxHashMap<usize, Vec<u8>> = FxHashMap::default();

    for patch in patches {
        let real_patch = wad_sys.cache_lump_idx(patch.lump_idx).unwrap();
        let real_patch: WadPatch = real_patch.as_slice().try_into().unwrap();
        let x1 = patch.origin_x;
        let x2 = x1.wrapping_add_unsigned(real_patch.header.width);
        let mut x2 = if x2 < 0 {
            continue;
        } else {
            x2 as u16
        };
        let x = if x1 < 0 { 0 } else { x1 as u16 };
        if x2 > tex_width {
            x2 = tex_width;
        }

        for i in x..x2 {
            let col_data = cols_data
                .entry(i as usize)
                .or_insert(vec![0; tex_height as usize]);
            let id = i.wrapping_add_signed(-x1) as usize;
            let col = &real_patch.columns[id];

            for post in col {
                let mut position = patch.origin_y.wrapping_add_unsigned(post.top_delta as u16);
                let mut count = post.data.len();
                if position < 0 {
                    count = count.wrapping_add_signed(position as isize);
                    position = 0;
                }
                let position = position as usize;
                if position >= col_data.len() {
                    break;
                }
                if position + count > col_data.len() {
                    count = col_data.len() - position;
                }
                let col_data = &mut col_data[position..(position + count)];
                let post_data = &post.data[0..count];
                col_data.copy_from_slice(post_data);
            }
        }
    }

    cols_data
}
