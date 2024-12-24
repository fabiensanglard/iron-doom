use bevy::prelude::*;

use crate::utils::in_level;
pub use draw_consts::ST_HEIGHT;
pub use draw_consts::ST_WIDTH;
use draw_consts::*;
use engine_core::random::Rand;
use engine_core::video_system::{FrameBuffer, VideoSystem};
use widgets::*;

#[allow(unused)]
mod draw_consts;
mod widgets;

pub struct StatusBarPlugin;

impl Plugin for StatusBarPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<StatusBar>()
            .add_systems(PostStartup, setup_status_bar)
            .add_systems(
                PostUpdate,
                (
                    background_drawer,
                    ready_drawer,
                    ammo_drawer,
                    health_drawer,
                    armor_drawer,
                    frags_drawer,
                    arms_drawer,
                    face_drawer,
                    cards_drawer,
                )
                    .chain()
                    .run_if(in_level()),
            );
    }
}

#[derive(Resource)]
struct StatusBar {
    backing_screen: FrameBuffer,
}

impl Default for StatusBar {
    fn default() -> Self {
        Self {
            backing_screen: FrameBuffer::new(ST_HEIGHT, ST_WIDTH),
        }
    }
}

fn setup_status_bar(mut commands: Commands) {
    commands.init_resource::<StatusBarAssets>();
}

fn background_drawer(
    st_assets: Res<StatusBarAssets>,
    mut st_bar: ResMut<StatusBar>,
    mut video_sys: NonSendMut<VideoSystem>,
) {
    st_bar
        .backing_screen
        .draw_patch(0, 0, &st_assets.status_bar_bg);

    video_sys.copy_rect(ST_X, ST_Y, &st_bar.backing_screen, None);
}

fn ready_drawer(st_assets: Res<StatusBarAssets>, mut video_sys: NonSendMut<VideoSystem>) {
    let n = NumberWidget::new(44, 171, 3, 50, true);
    n.draw(
        &mut video_sys,
        &st_assets.tall_nums,
        &st_assets.negative_symbol,
    );
}

fn ammo_drawer(st_assets: Res<StatusBarAssets>, mut video_sys: NonSendMut<VideoSystem>) {
    let short_nums = &st_assets.small_yellow_nums;

    for i in 0..4 {
        let y = 173 + i * 6;

        let ammo = NumberWidget::new(288, y, 3, 50, true);
        let max_ammo = NumberWidget::new(314, y, 3, 200, true);

        ammo.draw(&mut video_sys, short_nums, &st_assets.negative_symbol);
        max_ammo.draw(&mut video_sys, short_nums, &st_assets.negative_symbol);
    }
}

fn health_drawer(st_assets: Res<StatusBarAssets>, mut video_sys: NonSendMut<VideoSystem>) {
    let tall_nums = &st_assets.tall_nums;
    let neg_symbol = &st_assets.negative_symbol;
    let percent = &st_assets.percent_symbol;

    let n = NumberWidget::new(90, 171, 3, 100, true);
    n.draw_percent(&mut video_sys, tall_nums, neg_symbol, percent, true);
}

fn armor_drawer(st_assets: Res<StatusBarAssets>, mut video_sys: NonSendMut<VideoSystem>) {
    let tall_nums = &st_assets.tall_nums;
    let neg_symbol = &st_assets.negative_symbol;
    let percent = &st_assets.percent_symbol;
    let n = NumberWidget::new(221, 171, 3, 0, true);

    n.draw_percent(&mut video_sys, tall_nums, neg_symbol, percent, true);
}

fn frags_drawer(
    st_assets: Res<StatusBarAssets>,
    st_bar: Res<StatusBar>,
    mut video_sys: NonSendMut<VideoSystem>,
) {
    let arms_patch = &st_assets.weapon_inventory;
    let backing_screen = &st_bar.backing_screen;
    let mut bin = BinIconWidget::new(104, 168, true, true, true);

    bin.draw(&mut video_sys, backing_screen, arms_patch, true);
}

fn arms_drawer(
    st_assets: Res<StatusBarAssets>,
    st_bar: Res<StatusBar>,
    mut video_sys: NonSendMut<VideoSystem>,
) {
    let arms = &st_assets.small_gray_nums;
    let backing_screen = &st_bar.backing_screen;

    for i in 0..6 {
        let x = 111 + (i % 3) * 12;
        let y = 172 + (i / 3) * 10;
        let mut mi = MultiIconWidget::new(x, y, 0, i + 2, true);

        mi.draw(&mut video_sys, backing_screen, arms, true);
    }
}

fn face_drawer(
    st_assets: Res<StatusBarAssets>,
    st_bar: Res<StatusBar>,
    mut video_sys: NonSendMut<VideoSystem>,
    mut count: Local<usize>,
    mut index: Local<u32>,
    mut rand: ResMut<Rand>,
) {
    if *count == 0 {
        *index = (rand.get() % 3) as u32;
        *count = 17 * 4;
    }
    *count -= 1;
    let faces = &st_assets.faces;
    let backing_screen = &st_bar.backing_screen;
    let mut mi = MultiIconWidget::new(143, 168, 0, *index, true);

    mi.draw(&mut video_sys, backing_screen, faces, true);
}

fn cards_drawer(
    st_assets: Res<StatusBarAssets>,
    st_bar: Res<StatusBar>,
    mut video_sys: NonSendMut<VideoSystem>,
) {
    let cards = &st_assets.cards;
    let backing_screen = &st_bar.backing_screen;

    for i in 0..3 {
        let mut mi = MultiIconWidget::new(239, 171 + 10 * i, 0, i, true);
        mi.draw(&mut video_sys, backing_screen, cards, true);
    }
}
