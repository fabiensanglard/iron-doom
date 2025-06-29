use crate::components::Column;
use crate::events::InitMeltScreen;
use crate::resources::{EndScreen, NextPlayingState, StartScreen};
use bevy::{log::debug, prelude::*};
use game_state::PlayingState;
use rand::{Rand, Rng};
use window::ScreenBuffer;

pub fn check_state_transition(
    mut transition: EventReader<StateTransitionEvent<PlayingState>>,
    mut init_melt_screen: EventWriter<InitMeltScreen>,
) {
    use PlayingState::*;

    let Some(event) = transition.read().next() else {
        return;
    };
    let StateTransitionEvent {
        exited: Some(exited),
        entered: Some(entered),
    } = *event
    else {
        return;
    };

    // Skip screen melt transitions to avoid re-triggering it.
    if exited == ScreenMelt || entered == ScreenMelt {
        return;
    }
    // Trigger screen melt effect if the state is changing. Also trigger it
    // when loading a level from a level (identity transition (Level -> Level)).
    if exited != entered || exited == Level {
        init_melt_screen.send(InitMeltScreen);
    }
}

pub fn init_start_screen(
    mut commands: Commands,
    screen: Res<ScreenBuffer>,
    state: Res<State<PlayingState>>,
) {
    // Make sure we save the current state, so we can go
    // back to it after finishing the screen melt.
    commands.insert_resource(NextPlayingState(*state.get()));
    commands.insert_resource(StartScreen(screen.clone()));
}

pub fn init_end_screen(
    mut commands: Commands,
    mut screen: ResMut<ScreenBuffer>,
    start_screen: Res<StartScreen>,
) {
    commands.insert_resource(EndScreen(screen.clone()));
    commands.set_state(PlayingState::ScreenMelt);
    // Make sure the start screen is the one currently
    // displayed when starting the screen melt.
    screen.copy_from(&start_screen);
}

pub fn init_columns(mut commands: Commands, mut rng: ResMut<Rand>) {
    debug!("Starting Screen Melt");

    let _ = rng.random::<u8>();

    // The screen is divided into groups of two columns, where
    // each pair of columns moves together at the same speed.
    let mut wait = rng.random::<u8>() % 16;
    commands.spawn(Column::new(0, wait));
    commands.spawn(Column::new(1, wait));
    for i in (2..320).step_by(2) {
        // Generate a random value of -1, 0, or 1.
        let r = 1 - (rng.random::<i32>() % 3);
        wait = wait.saturating_add_signed(r as i8);
        if wait > 15 {
            // Limit max difference between column groups when scrolling.
            wait = 15;
        }
        commands.spawn(Column::new(i, wait));
        commands.spawn(Column::new(i + 1, wait));
    }
}

pub fn move_columns(
    mut columns: Query<(Entity, &mut Column)>,
    mut screen: ResMut<ScreenBuffer>,
    start_screen: Res<StartScreen>,
    end_screen: Res<EndScreen>,
    next_state: Res<NextPlayingState>,
    mut commands: Commands,
) {
    let mut done = true;

    for (entity, mut column) in &mut columns {
        let pos = column.position();
        let ready_move = column.update_position();
        if !ready_move {
            continue;
        }
        let new_pos = column.position();
        let col = column.num();
        done = false;

        // Move column.
        for y in pos..200 {
            let color = if y < new_pos {
                end_screen[(col, y)]
            } else {
                start_screen[(col, y - new_pos)]
            };
            screen[(col, y)] = color;
        }

        if new_pos == 200 {
            // Column reached end of screen, so remove it.
            commands.entity(entity).despawn();
        }
    }

    if done {
        debug!("Finished Screen Melt");
        commands.remove_resource::<StartScreen>();
        commands.remove_resource::<EndScreen>();
        commands.remove_resource::<NextPlayingState>();
        commands.set_state(next_state.0);
    }
}
