use bevy::prelude::*;

use crate::GameState;
use engine_core::video_system::VideoSystem;
use wipe::WipePlugin;

mod wipe;

pub struct RenderPlugin;

impl Plugin for RenderPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<ChangeRenderState>()
            .init_state::<RenderState>()
            .add_plugins(WipePlugin)
            .add_systems(
                Update,
                change_render_state.run_if(on_event::<ChangeRenderState>()),
            )
            .add_systems(Last, update_screen.run_if(can_update_screen()));
    }
}

#[derive(States, Debug, Hash, Eq, PartialEq, Clone, Copy, Default)]
enum RenderState {
    #[default]
    Default,
    Wipe,
}

#[derive(Event)]
pub enum ChangeRenderState {
    Default,
    Wipe,
}

fn change_render_state(
    mut render_state_events: EventReader<ChangeRenderState>,
    mut render_state: ResMut<NextState<RenderState>>,
) {
    for ev in render_state_events.read() {
        match ev {
            ChangeRenderState::Default => {
                render_state.set(RenderState::Default);
            }
            ChangeRenderState::Wipe => {
                render_state.set(RenderState::Wipe);
            }
        }
    }
}

pub fn update_screen(mut video_sys: NonSendMut<VideoSystem>) {
    video_sys.update();
}

fn can_update_screen() -> impl Condition<()> {
    not(in_state(RenderState::Wipe)).and_then(not(in_state(GameState::Setup)))
}
