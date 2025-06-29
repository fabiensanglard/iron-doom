use bevy::prelude::Resource;
use derive_more::{Deref, DerefMut};
use game_state::PlayingState;
use window::ScreenBuffer;

#[derive(Resource, Deref, DerefMut)]
pub struct StartScreen(pub ScreenBuffer);

#[derive(Resource, Deref, DerefMut)]
pub struct EndScreen(pub ScreenBuffer);

#[derive(Resource, Deref, DerefMut)]
pub struct NextPlayingState(pub PlayingState);
