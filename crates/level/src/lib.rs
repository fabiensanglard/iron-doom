use crate::prelude::PlayerMovementPlugin;
use crate::resources::LevelMap;
use anyhow::bail;
use bevy::prelude::*;
use exit::macros::sys_fail;
use game_state::PlayingState;
use wad::WadFile;

mod commands;
mod components;
mod map_object;
mod resources;

pub mod prelude {
    pub use super::{components::*, map_object::prelude::*};
}

#[derive(Default)]
pub struct LevelPlugin;

impl Plugin for LevelPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(PlayerMovementPlugin)
            .add_event::<LoadLevel>()
            .init_non_send_resource::<LevelMap>()
            .add_systems(Update, load_level.run_if(on_event::<LoadLevel>));
    }
}

#[sys_fail]
fn load_level(
    mut load: EventReader<LoadLevel>,
    mut playing_state: ResMut<NextState<PlayingState>>,
    mut level_map: NonSendMut<LevelMap>,
    mut commands: Commands,
    wad: Res<WadFile>,
) {
    let Some(LoadLevel { episode, map }) = load.read().last() else {
        return Ok(());
    };

    debug!("Loading Episode {episode} Map {map}");
    let Some(map) = wad.map(*episode, *map) else {
        bail!("Tried to load invalid map: E{episode}M{map}");
    };

    level_map.load(&mut commands, map, &wad)?;
    playing_state.set(PlayingState::Level);
}

#[derive(Event, Debug)]
pub struct LoadLevel {
    pub episode: usize,
    pub map: usize,
}
