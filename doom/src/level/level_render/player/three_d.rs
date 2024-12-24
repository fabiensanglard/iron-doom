use bevy::prelude::*;
pub use graphics_assets::*;
pub use horizontal_screen::*;

use engine_core::video_system::SCREEN_HEIGHT;

use super::PlayerViewMode;
use crate::level::level_render::status_bar::ST_HEIGHT;
use crate::utils::in_level;
use render_flats::RenderFlatsPlugin;
use render_walls::RenderWallsPlugin;

mod graphics_assets;
mod horizontal_screen;
mod render_flats;
mod render_walls;

const PL_HEIGHT: u32 = SCREEN_HEIGHT - ST_HEIGHT;

pub struct ThreeDPlugin;

impl Plugin for ThreeDPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ScreenLUT>()
            .configure_sets(
                PostUpdate,
                (
                    RenderSet::Prepare,
                    RenderSet::Walls,
                    RenderSet::Flats,
                    RenderSet::Sprites,
                )
                    .run_if(in_level())
                    .run_if(in_state(PlayerViewMode::ThreeD))
                    .chain(),
            )
            .add_plugins((GraphicsAssetsPlugin, RenderWallsPlugin, RenderFlatsPlugin));
    }
}

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub enum RenderSet {
    Prepare,
    Walls,
    Flats,
    Sprites,
}
