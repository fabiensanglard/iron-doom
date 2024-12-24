use bevy::prelude::*;

use auto_map::AutoMapPlugin;
use three_d::ThreeDPlugin;

mod auto_map;
mod three_d;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<PlayerViewMode>()
            .add_plugins((AutoMapPlugin, ThreeDPlugin));
    }
}

#[derive(States, Debug, Hash, Eq, PartialEq, Clone, Copy, Default)]
enum PlayerViewMode {
    AutoMap,
    #[default]
    ThreeD,
}
