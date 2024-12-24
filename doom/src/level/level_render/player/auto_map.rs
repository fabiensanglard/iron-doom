use std::cmp;

use bevy::prelude::*;
use leafwing_input_manager::prelude::*;

use engine_core::fixed_point::Fixed;
use engine_core::geometry::Vertex;
use engine_core::video_system::{FrameBuffer, VideoSystem, SCREEN_HEIGHT, SCREEN_WIDTH};

use super::PlayerViewMode;
use crate::level::level_render::status_bar::ST_HEIGHT;
use crate::level::map_object::{Player, Position};
use crate::level::{LevelMap, LoadLevelDone};
use crate::utils::in_level;

mod background_drawer;
#[allow(unused)]
mod colors;
mod cross_hair_drawer;
mod line_drawer;
mod map_geometry;
mod player_drawer;
mod walls_drawer;

const AM_WIDTH: u32 = SCREEN_WIDTH;
const AM_HEIGHT: u32 = SCREEN_HEIGHT - ST_HEIGHT;
const PLAYER_RADIUS: Fixed = Fixed::lit("16");

pub struct AutoMapPlugin;

impl Plugin for AutoMapPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(InputManagerPlugin::<AutoMapAction>::default())
            .init_resource::<ActionState<AutoMapAction>>()
            .insert_resource(AutoMapAction::mkb_input_map())
            .init_resource::<AutoMap>()
            .add_systems(FixedPreUpdate, setup.run_if(on_event::<LoadLevelDone>()))
            .add_systems(
                FixedUpdate,
                (toggle_auto_map, update_map_scale).run_if(in_level()),
            )
            .add_systems(
                FixedPostUpdate,
                (follow_player, scale_map).run_if(in_level()),
            )
            .add_systems(
                PostUpdate,
                (
                    background_drawer::draw,
                    walls_drawer::draw,
                    player_drawer::draw,
                    cross_hair_drawer::draw,
                    update_video,
                )
                    .chain()
                    .run_if(in_level())
                    .run_if(in_state(PlayerViewMode::AutoMap)),
            );
    }
}

#[derive(Resource)]
pub struct AutoMap {
    buf: FrameBuffer,
    m_w: Fixed,
    m_h: Fixed,
    m_x: Fixed,
    m_y: Fixed,
    m_x2: Fixed,
    m_y2: Fixed,
    scale_ftom: Fixed,
    scale_mtof: Fixed,
    ftom_zoommul: Fixed,
    mtof_zoommul: Fixed,
    min_scale_mtof: Fixed,
    max_scale_mtof: Fixed,
    min_x: Fixed,
    max_x: Fixed,
    min_y: Fixed,
    max_y: Fixed,
}

impl Default for AutoMap {
    fn default() -> Self {
        Self {
            buf: FrameBuffer::new(AM_HEIGHT, AM_WIDTH),
            m_w: Fixed::from_bits(246129920),
            m_h: Fixed::from_bits(129218208),
            m_x: Fixed::ZERO,
            m_y: Fixed::ZERO,
            m_x2: Fixed::ZERO,
            m_y2: Fixed::ZERO,
            scale_ftom: Fixed::from_bits(769156),
            scale_mtof: Fixed::from_bits(5584),
            ftom_zoommul: Fixed::ONE,
            mtof_zoommul: Fixed::ONE,
            min_scale_mtof: Fixed::ONE,
            max_scale_mtof: Fixed::ONE,
            min_x: Fixed::ONE,
            max_x: Fixed::ONE,
            min_y: Fixed::ONE,
            max_y: Fixed::ONE,
        }
    }
}

impl AutoMap {
    fn ftom(&self, x: i32) -> Fixed {
        Fixed::wrapping_from_num(x) * self.scale_ftom
    }

    fn mtof(&self, x: Fixed) -> i32 {
        (x * self.scale_mtof).to_num()
    }

    fn cxmtof(&self, x: Fixed) -> i32 {
        self.mtof(x - self.m_x)
    }

    fn cymtof(&self, x: Fixed) -> i32 {
        AM_HEIGHT as i32 - self.mtof(x - self.m_y)
    }
}

#[derive(Actionlike, PartialEq, Eq, Clone, Copy, Hash, Debug, Reflect)]
enum AutoMapAction {
    Toggle,
    ZoomIn,
    ZoomOut,
}

impl AutoMapAction {
    fn mkb_input_map() -> InputMap<Self> {
        InputMap::new([
            (Self::Toggle, KeyCode::Tab),
            (Self::ZoomIn, KeyCode::Equal),
            (Self::ZoomOut, KeyCode::Minus),
        ])
    }
}

fn setup(mut auto_map: ResMut<AutoMap>, level_map: Res<LevelMap>, vertex_query: Query<&Vertex>) {
    let mut min_x = Fixed::MAX;
    let mut min_y = Fixed::MAX;
    let mut max_x = Fixed::MIN;
    let mut max_y = Fixed::MIN;

    for vertex_id in level_map.vertexes() {
        let v = vertex_query.get(*vertex_id).unwrap();
        min_x = cmp::min(min_x, v.x);
        max_x = cmp::max(max_x, v.x);
        min_y = cmp::min(min_y, v.y);
        max_y = cmp::max(max_y, v.y);
    }

    let max_w = max_x - min_x;
    let max_h = max_y - min_y;

    let a = Fixed::from_num(AM_WIDTH) / max_w;
    let b = Fixed::from_num(AM_HEIGHT) / max_h;

    auto_map.min_scale_mtof = cmp::min(a, b);
    auto_map.max_scale_mtof = Fixed::from_num(AM_HEIGHT) / (2 * PLAYER_RADIUS);
    auto_map.min_x = min_x;
    auto_map.max_x = max_x;
    auto_map.min_y = min_y;
    auto_map.max_y = max_y;

    auto_map.scale_mtof = auto_map.min_scale_mtof / Fixed::from_bits(45875);
    if auto_map.scale_mtof > auto_map.max_scale_mtof {
        auto_map.scale_mtof = auto_map.min_scale_mtof;
    }
    auto_map.scale_ftom = Fixed::ONE / auto_map.scale_mtof;
}

fn follow_player(mut auto_map: ResMut<AutoMap>, query: Query<&Position, With<Player>>) {
    let player_pos = query.single();
    let pos_x = player_pos.x;
    let pos_y = player_pos.y;

    auto_map.m_x = auto_map.ftom(auto_map.mtof(pos_x)) - auto_map.m_w / 2;
    auto_map.m_x2 = auto_map.m_x + auto_map.m_w;

    auto_map.m_y = auto_map.ftom(auto_map.mtof(pos_y)) - auto_map.m_h / 2;
    auto_map.m_y2 = auto_map.m_y + auto_map.m_h;
}

const ZOOM_IN: Fixed = Fixed::from_bits(0x1051e);
const ZOOM_OUT: Fixed = Fixed::from_bits(0xfafa);

fn update_map_scale(mut auto_map: ResMut<AutoMap>, action_state: Res<ActionState<AutoMapAction>>) {
    if action_state.pressed(&AutoMapAction::ZoomIn) {
        auto_map.mtof_zoommul = ZOOM_IN;
        auto_map.ftom_zoommul = ZOOM_OUT;
        return;
    }
    if action_state.pressed(&AutoMapAction::ZoomOut) {
        auto_map.mtof_zoommul = ZOOM_OUT;
        auto_map.ftom_zoommul = ZOOM_IN;
        return;
    }
    if action_state.released(&AutoMapAction::ZoomIn)
        || action_state.released(&AutoMapAction::ZoomOut)
    {
        auto_map.mtof_zoommul = Fixed::ONE;
        auto_map.ftom_zoommul = Fixed::ONE;
    }
}

fn scale_map(mut auto_map: ResMut<AutoMap>) {
    auto_map.scale_mtof = auto_map.scale_mtof.wrapping_mul(auto_map.mtof_zoommul);
    auto_map.scale_ftom = Fixed::ONE.wrapping_div(auto_map.scale_mtof);

    if auto_map.scale_mtof < auto_map.min_scale_mtof {
        min_out_window_scale(&mut auto_map);
    } else if auto_map.scale_mtof > auto_map.max_scale_mtof {
        max_out_window_scale(&mut auto_map);
    } else {
        activate_new_scale(&mut auto_map);
    }
}

fn min_out_window_scale(auto_map: &mut AutoMap) {
    auto_map.scale_mtof = auto_map.min_scale_mtof;
    auto_map.scale_ftom = Fixed::ONE / auto_map.scale_mtof;
    activate_new_scale(auto_map);
}

fn max_out_window_scale(auto_map: &mut AutoMap) {
    auto_map.scale_mtof = auto_map.max_scale_mtof;
    auto_map.scale_ftom = Fixed::ONE / auto_map.scale_mtof;
    activate_new_scale(auto_map);
}

fn activate_new_scale(auto_map: &mut AutoMap) {
    auto_map.m_x += auto_map.m_w / 2;
    auto_map.m_y += auto_map.m_h / 2;
    auto_map.m_w = auto_map.ftom(AM_WIDTH as i32);
    auto_map.m_h = auto_map.ftom(AM_HEIGHT as i32);
    auto_map.m_x -= auto_map.m_w / 2;
    auto_map.m_y -= auto_map.m_h / 2;
    auto_map.m_x2 = auto_map.m_x + auto_map.m_w;
    auto_map.m_y2 = auto_map.m_y + auto_map.m_h;
}

fn update_video(mut video_sys: NonSendMut<VideoSystem>, auto_map: Res<AutoMap>) {
    video_sys.copy_rect(0, 0, &auto_map.buf, None);
}

fn toggle_auto_map(
    action_state: Res<ActionState<AutoMapAction>>,
    view_state: Res<State<PlayerViewMode>>,
    mut next_view_state: ResMut<NextState<PlayerViewMode>>,
) {
    if action_state.just_pressed(&AutoMapAction::Toggle) {
        match view_state.get() {
            PlayerViewMode::AutoMap => {
                next_view_state.set(PlayerViewMode::ThreeD);
            }
            PlayerViewMode::ThreeD => {
                next_view_state.set(PlayerViewMode::AutoMap);
            }
        }
    }
}
