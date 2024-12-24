use bevy::prelude::*;
use rustc_hash::FxHashMap;

use crate::setup::GameSetupSet;
use engine_core::app::exit_error;

mod flats;
mod wall_textures;

pub struct GraphicsAssetsPlugin;

impl Plugin for GraphicsAssetsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<GraphicsAssets>().add_systems(
            Startup,
            (
                flats::load.pipe(exit_error),
                wall_textures::load.pipe(exit_error),
            )
                .after(GameSetupSet::FileSystem),
        );
    }
}

#[derive(Resource, Default)]
pub struct GraphicsAssets {
    flats: FxHashMap<String, String>,
    wall_textures: FxHashMap<String, WallTexture>,
}

impl GraphicsAssets {
    pub fn wall_texture(&self, name: &str) -> Result<&WallTexture, String> {
        let err_msg = format!("wall_texture: {name} not found!");
        // "NoTexture" marker.
        if name.starts_with('-') {
            return self.wall_textures.values().next().ok_or(err_msg);
        }
        if let Some(texture) = self.wall_textures.get(name) {
            return Ok(texture);
        }
        let name = name.to_uppercase();
        self.wall_textures.get(&name).ok_or(err_msg)
    }
}

pub struct WallTexture {
    pub name: String,
    pub width: u16,
    pub height: u16,
    pub patches: Vec<WallTexturePatch>,
    pub cols: FxHashMap<usize, Vec<u8>>,
}

pub struct WallTexturePatch {
    pub origin_x: i16,
    pub origin_y: i16,
    pub lump_idx: usize,
}
