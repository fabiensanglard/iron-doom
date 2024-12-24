use crate::GameState;
use bevy::prelude::{in_state, Condition, IntoSystem};

pub fn in_level() -> impl Condition<()> {
    IntoSystem::into_system(in_state(GameState::Level))
}
