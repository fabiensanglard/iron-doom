use bevy::prelude::*;

use player::PlayerPlugin;
use status_bar::StatusBarPlugin;

mod player;
mod status_bar;

pub struct LevelRenderPlugin;

impl Plugin for LevelRenderPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((StatusBarPlugin, PlayerPlugin));
    }
}
