use crate::{GameState, PlayingState};
use bevy::prelude::{in_state, Condition, IntoSystem};
use paste::paste;

macro_rules! impl_in_state {
    ( $name:ident, $state:path ) => {
        paste! {
            #[inline]
            pub fn [<in_ $name _state>]() -> impl Condition<()> {
                IntoSystem::into_system(in_state($state))
            }
        }
    };
}

impl_in_state!(setup, GameState::Setup);
impl_in_state!(playing, GameState::Playing);
impl_in_state!(exiting, GameState::Exiting);
impl_in_state!(intro, PlayingState::Intro);
impl_in_state!(level, PlayingState::Level);
impl_in_state!(screen_melt, PlayingState::ScreenMelt);
