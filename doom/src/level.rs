use bevy::prelude::*;

pub use load_level::*;

use map_object::MapObjectPlugin;
use level_render::LevelRenderPlugin;

mod level_render;
mod load_level;
mod map_object;

pub struct LevelPlugin;

impl Plugin for LevelPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((MapObjectPlugin, LoadLevelPlugin, LevelRenderPlugin));
    }
}
