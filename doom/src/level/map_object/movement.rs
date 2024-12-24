use bevy::prelude::*;

use leafwing_input_manager::prelude::*;

use engine_core::fixed_point::Fixed;
use engine_core::geometry::{Angle, BranchNode, Sector, SubSector};

use super::controls::PlayerAction;
use super::{Momentum, Player, PlayerAngle, PlayerSubSector, PlayerView, Position};
use crate::level::LevelMap;
use crate::utils::in_level;
pub use constants::*;

mod constants;

pub struct PlayerMovementPlugin;

impl Plugin for PlayerMovementPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            FixedUpdate,
            (
                update_angle,
                update_forward_momentum,
                update_side_momentum,
                try_move_player,
                apply_friction,
                update_player_sector,
                z_movement,
            )
                .chain()
                .run_if(in_level()),
        );
    }
}

fn update_angle(
    mut turn_held: Local<u32>,
    action_query: Query<&ActionState<PlayerAction>, With<Player>>,
    mut query: Query<&mut PlayerAngle, With<Player>>,
) {
    let action_state = action_query.single();

    update_turn_held(&mut turn_held, action_state);
    let slow_turn = *turn_held < SLOW_TURN_TICS;

    let mut angle_turn: Angle = Angle::ZERO;

    if slow_turn {
        if action_state.pressed(&PlayerAction::TurnLeft)
            || action_state.pressed(&PlayerAction::FastLeftTurn)
        {
            angle_turn = SLOW_TURN;
        }
        if action_state.pressed(&PlayerAction::TurnRight)
            || action_state.pressed(&PlayerAction::FastRightTurn)
        {
            angle_turn = -SLOW_TURN;
        }
    } else {
        if action_state.pressed(&PlayerAction::TurnLeft) {
            angle_turn = TURN;
        }
        if action_state.pressed(&PlayerAction::FastLeftTurn) {
            angle_turn = FAST_TURN;
        }
        if action_state.pressed(&PlayerAction::TurnRight) {
            angle_turn = -TURN;
        }
        if action_state.pressed(&PlayerAction::FastRightTurn) {
            angle_turn = -FAST_TURN;
        }
    }
    angle_turn <<= 16;

    let mut player_angle = query.single_mut();
    player_angle.0 += angle_turn;
}

fn update_turn_held(turn_held: &mut u32, action_state: &ActionState<PlayerAction>) {
    if action_state.pressed(&PlayerAction::TurnLeft)
        || action_state.pressed(&PlayerAction::FastLeftTurn)
        || action_state.pressed(&PlayerAction::TurnRight)
        || action_state.pressed(&PlayerAction::FastRightTurn)
    {
        *turn_held += 1;
    } else {
        *turn_held = 0;
    }
}

fn update_forward_momentum(
    action_query: Query<&ActionState<PlayerAction>, With<Player>>,
    mut query: Query<(&PlayerAngle, &mut Momentum), With<Player>>,
) {
    let (angle, mut momentum) = query.single_mut();
    let action_state = action_query.single();
    let mut forward = 0;

    if action_state.pressed(&PlayerAction::RunForward) {
        forward += RUN_FORWARD;
    } else if action_state.pressed(&PlayerAction::MoveForward) {
        forward += MOVE_FORWARD;
    }
    if action_state.pressed(&PlayerAction::RunBackward) {
        forward -= RUN_FORWARD;
    } else if action_state.pressed(&PlayerAction::MoveBackward) {
        forward -= MOVE_FORWARD;
    }
    forward = forward.clamp(-MAX_MOVE, MAX_MOVE);

    let forward = MOVE_SCALE.wrapping_mul(forward as i32);
    let forward = Fixed::from_bits(forward);

    // Make sure the Bevy change system works properly by
    // avoiding calling thrust if movement is zero
    if !forward.is_zero() {
        momentum.thrust(angle.0, forward);
    }
}

fn update_side_momentum(
    action_query: Query<&ActionState<PlayerAction>, With<Player>>,
    mut query: Query<(&PlayerAngle, &mut Momentum), With<Player>>,
) {
    let (angle, mut momentum) = query.single_mut();
    let action_state = action_query.single();
    let mut side = 0;

    if action_state.pressed(&PlayerAction::FastStrafeLeft) {
        side -= FAST_STRAFE;
    } else if action_state.pressed(&PlayerAction::StrafeLeft) {
        side -= STRAFE;
    }
    if action_state.pressed(&PlayerAction::FastStrafeRight) {
        side += FAST_STRAFE;
    } else if action_state.pressed(&PlayerAction::StrafeRight) {
        side += STRAFE;
    }
    side = side.clamp(-MAX_MOVE, MAX_MOVE);

    let side = MOVE_SCALE.wrapping_mul(side as i32);
    let side = Fixed::from_bits(side);
    // rotate 90º to move side way
    let ang = angle.0 - Angle::DEG_90;

    // Make sure the Bevy change system works properly by
    // avoiding calling thrust if movement is zero
    if !side.is_zero() {
        momentum.thrust(ang, side);
    }
}

fn try_move_player(mut query: Query<(&Momentum, &mut Position), With<Player>>) {
    let (mo, mut pos) = query.single_mut();

    if mo.is_zero() {
        return;
    }
    let mut xmove = mo.x;
    let mut ymove = mo.y;

    loop {
        let ptryx: Fixed;
        let ptryy: Fixed;

        if xmove > MAX_MOMENTUM / 2 || ymove > MAX_MOMENTUM / 2 {
            ptryx = pos.x + xmove / 2;
            ptryy = pos.y + ymove / 2;
            xmove >>= 1;
            ymove >>= 1;
        } else {
            ptryx = pos.x + xmove;
            ptryy = pos.y + ymove;
            xmove = Fixed::ZERO;
            ymove = Fixed::ZERO;
        }

        pos.x = ptryx;
        pos.y = ptryy;

        if xmove == 0 || ymove == 0 {
            break;
        }
    }
}

fn apply_friction(mut query: Query<&mut Momentum, With<Player>>) {
    let mut mo = query.single_mut();

    // No need to apply friction if not moving
    if mo.is_zero() {
        return;
    }
    // If momentum is within stop range and there is no
    // user input changing momentum, then zero it out.
    if mo.is_stop_range() && !mo.is_changed() {
        mo.x = Fixed::ZERO;
        mo.y = Fixed::ZERO;
        return;
    }
    mo.x = mo.x.wrapping_mul(FRICTION);
    mo.y = mo.y.wrapping_mul(FRICTION);
}

fn update_player_sector(
    level_map: Res<LevelMap>,
    angle_query: Query<&PlayerAngle, With<Player>>,
    position_query: Query<&Position, (With<Player>, Changed<Position>)>,
    mut sector_query: Query<&mut PlayerSubSector, With<Player>>,
    branch_node_query: Query<&BranchNode>,
) {
    let Ok(player_pos) = position_query.get_single() else {
        return;
    };
    let mut player_sector = sector_query.single_mut();
    let player_angle = angle_query.single();
    let player_angle = player_angle.0;

    let delta = Fixed::ONE * 13;
    let x = player_pos.x + delta * player_angle.cos();
    let y = player_pos.y + delta * player_angle.sin();

    let root = level_map.root_bsp_node();
    let mut node_stack = Vec::new();
    node_stack.push(root);

    while let Some(node) = node_stack.pop() {
        match node {
            engine_core::geometry::Node::Branch(branch_node) => {
                let branch_node = branch_node_query.get(branch_node).unwrap();

                if branch_node.is_on_back(x, y) {
                    // Traverse left child first
                    node_stack.push(branch_node.right_child);
                    node_stack.push(branch_node.left_child);
                } else {
                    // Traverse right child first
                    node_stack.push(branch_node.left_child);
                    node_stack.push(branch_node.right_child);
                };
            }
            engine_core::geometry::Node::Leaf(sub_sector) => {
                player_sector.0 = sub_sector;
                break;
            }
        }
    }
}

fn z_movement(
    mut position_query: Query<&mut Position, With<Player>>,
    mut momentum_query: Query<&mut Momentum, With<Player>>,
    mut view_query: Query<&mut PlayerView, With<Player>>,
    player_sector_query: Query<&PlayerSubSector, With<Player>>,
    sub_sector_query: Query<&SubSector>,
    sector_query: Query<&Sector>,
) {
    let mut position = position_query.single_mut();
    let mut momentum = momentum_query.single_mut();
    let mut view = view_query.single_mut();

    let sub_sector = player_sector_query.single();
    let sub_sector = sub_sector_query.get(sub_sector.0).unwrap();
    let sector = sector_query.get(sub_sector.sector).unwrap();

    let mut z = position.z - view.height;

    let max = Fixed::from_num(41);
    view.height = view.height + view.delta;
    if view.height > max {
        view.height = max;
        view.delta = Fixed::ZERO;
    }
    if view.height < max / 2 {
        view.height = max / 2;
        if view.delta <= 0 {
            view.delta = Fixed::DELTA;
        }
    }
    if view.delta != 0 {
        view.delta += Fixed::ONE / 4;
        if view.delta == 0 {
            view.delta = Fixed::DELTA;
        }
    }

    position.z = z + view.height;

    if z < sector.floor_height {
        view.height -= sector.floor_height - z;
        view.delta = (max - view.height) >> 3;
    }

    z = position.z - view.height;

    if z == sector.floor_height && momentum.z == 0 {
        return;
    }

    z += momentum.z;

    if z <= sector.floor_height {
        if momentum.z < 0 {
            momentum.z = Fixed::ZERO;
        }
        z = sector.floor_height;
    } else if momentum.z == 0 {
        momentum.z = -(2 * Fixed::ONE);
    } else {
        momentum.z -= Fixed::ONE;
    }

    position.z = z + view.height;
}
