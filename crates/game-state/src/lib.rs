use bevy::prelude::*;

pub mod conditions;

#[derive(Default)]
pub struct GameStatePlugin;

impl Plugin for GameStatePlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<GameState>()
            .add_sub_state::<PlayingState>()
            .enable_state_scoped_entities::<GameState>()
            .enable_state_scoped_entities::<PlayingState>();
    }
}

#[derive(States, Default, Debug, Hash, PartialEq, Eq, Clone, Copy)]
pub enum GameState {
    /// Setup phase, where systems such as WAD loading, palette initialization,
    /// and other setup procedures occur.
    #[default]
    Setup,
    /// The main gameplay is running, with all game mechanics and interactions active.
    Playing,
    /// The game is in the process of exiting.
    Exiting,
    /// The game is paused, usually during gameplay when the user stops interaction.
    Paused,
}

#[derive(SubStates, Debug, Clone, Copy, Default, Eq, PartialEq, Hash)]
#[source(GameState = GameState::Playing)]
pub enum PlayingState {
    /// The initial state during the game, showing the `TITLEPIC` lump (title screen).
    #[default]
    Intro,
    Level,
    ScreenMelt,
}
