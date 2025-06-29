use bevy::app::{App, MainScheduleOrder, PostUpdate};
use bevy::ecs::schedule::ScheduleLabel;

#[derive(ScheduleLabel, Debug, Hash, PartialEq, Eq, Clone)]
pub struct InitStartScreen;

#[derive(ScheduleLabel, Debug, Hash, PartialEq, Eq, Clone)]
pub struct InitEndScreen;

pub fn setup(app: &mut App) {
    let mut main_schedule_order = app.world_mut().resource_mut::<MainScheduleOrder>();
    main_schedule_order.insert_before(PostUpdate, InitStartScreen);
    main_schedule_order.insert_after(PostUpdate, InitEndScreen);

    app.init_schedule(InitStartScreen);
    app.init_schedule(InitEndScreen);
}
