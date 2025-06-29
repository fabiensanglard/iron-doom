use bevy::prelude::*;
use derive_more::{Deref, DerefMut};
use exit::macros::sys_fail;
use game_state::{conditions::in_intro_state, PlayingState};
use level::LoadLevel;
use std::time::Duration;
use wad::prelude::*;
use window::ScreenBuffer;

#[derive(Default)]
pub struct TitleScreenPlugin;

impl Plugin for TitleScreenPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            FixedUpdate,
            (draw_title_screen.run_if(run_once), transition)
                .chain()
                .run_if(in_intro_state()),
        );
    }
}

#[sys_fail]
fn draw_title_screen(mut screen: ResMut<ScreenBuffer>, wad: Res<WadFile>, mut commands: Commands) {
    debug!("Entering Title Screen");
    screen.draw_patch(0, 0, wad.title_screen())?;
    commands.spawn((
        StateScoped(PlayingState::Intro),
        ScreenTimer::from_secs_f64(1.7142857142857143),
    ));
}

fn transition(
    time: Res<Time>,
    mut query: Query<&mut ScreenTimer>,
    mut load_level: EventWriter<LoadLevel>,
) {
    let mut timer = query.single_mut();
    if timer.tick(time.delta()).just_finished() {
        debug!("Exiting Title Screen");
        load_level.send(LoadLevel { episode: 1, map: 1 });
    }
}

#[derive(Component, Deref, DerefMut)]
struct ScreenTimer(Timer);

impl ScreenTimer {
    fn from_secs_f64(secs: f64) -> Self {
        let duration = Duration::from_secs_f64(secs);
        Self(Timer::new(duration, TimerMode::Once))
    }
}
