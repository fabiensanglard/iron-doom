use bevy::ecs::world::Command;
use bevy::prelude::*;

use derive_more::{AsRef, Deref, DerefMut, Into};

use leafwing_input_manager::prelude::*;

use engine_core::fixed_point::Fixed;
use engine_core::geometry::Angle;

use controls::PlayerAction;
use movement::{PlayerMovementPlugin, MAX_MOMENTUM, STOP_SPEED};

mod controls;
mod movement;

pub struct MapObjectPlugin;

impl Plugin for MapObjectPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            InputManagerPlugin::<PlayerAction>::default(),
            PlayerMovementPlugin,
        ));
    }
}

pub struct SpawnPlayer {
    pub x: Fixed,
    pub y: Fixed,
    pub angle: Angle,
}

impl Command for SpawnPlayer {
    fn apply(self, world: &mut World) {
        world
            .commands()
            .spawn(PlayerBundle {
                position: Position {
                    x: self.x,
                    y: self.y,
                    z: Fixed::from_num(41),
                },
                view: PlayerView {
                    height: Fixed::from_num(41),
                    delta: Fixed::ZERO,
                },
                angle: PlayerAngle(self.angle),
                ..default()
            })
            .insert(InputManagerBundle::with_map(PlayerAction::default_map()));
    }
}

#[derive(Bundle, Default)]
struct PlayerBundle {
    player: Player,
    position: Position,
    angle: PlayerAngle,
    momentum: Momentum,
    sub_sector: PlayerSubSector,
    view: PlayerView,
}

#[derive(Component, Default, Copy, Clone)]
pub struct Player;

#[derive(Component, Default, Copy, Clone)]
pub struct PlayerView {
    height: Fixed,
    delta: Fixed,
}

#[derive(Component, Default, Copy, Clone)]
pub struct Position {
    pub x: Fixed,
    pub y: Fixed,
    pub z: Fixed,
}

#[derive(Component, Default, Copy, Clone, Into, AsRef, Deref, DerefMut)]
#[into(owned, ref, ref_mut)]
pub struct PlayerAngle(pub Angle);

#[derive(Component, Default, Copy, Clone)]
pub struct Momentum {
    pub x: Fixed,
    pub y: Fixed,
    pub z: Fixed,
}

#[derive(Component, Copy, Clone)]
pub struct PlayerSubSector(Entity);

impl Default for PlayerSubSector {
    fn default() -> Self {
        Self(Entity::PLACEHOLDER)
    }
}

impl Position {
    pub fn distance(&self, x: Fixed, y: Fixed) -> Fixed {
        let mut dx = x.wrapping_sub(self.x).abs();
        let mut dy = y.wrapping_sub(self.y).abs();

        if dy > dx {
            std::mem::swap(&mut dx, &mut dy);
        }
        let angle = if dx != 0 {
            Angle::atan(dy / dx, Fixed::ONE) + Angle::DEG_90
        } else {
            Angle::atan(Fixed::ZERO, Fixed::ONE) + Angle::DEG_90
        };

        dx / angle.sin()
    }

    pub fn angle(&self, x2: Fixed, y2: Fixed) -> Angle {
        let mut x = x2.wrapping_sub(self.x);
        let mut y = y2.wrapping_sub(self.y);

        if x == 0 && y == 0 {
            return Angle::ZERO;
        }

        if x >= 0 {
            if y >= 0 {
                if x > y {
                    // octant 0
                    return Angle::atan(y, x);
                }
                // octant 1
                return Angle::DEG_90 - Angle::MIN - Angle::atan(x, y);
            }

            y = -y;
            if x > y {
                // octant 8
                return -Angle::atan(y, x);
            }
            // octant 7
            return Angle::DEG_270 + Angle::atan(x, y);
        }

        x = -x;
        if y >= 0 {
            if x > y {
                // octant 3
                return Angle::DEG_180 - Angle::MIN - Angle::atan(y, x);
            }
            // octant 2
            return Angle::DEG_90 + Angle::atan(x, y);
        }

        y = -y;
        if x > y {
            // octant 4
            return Angle::DEG_180 + Angle::atan(y, x);
        }
        // octant 5
        Angle::DEG_270 - Angle::MIN - Angle::atan(x, y)
    }
}

impl From<&PlayerAngle> for Angle {
    fn from(value: &PlayerAngle) -> Self {
        (*value).into()
    }
}

impl From<&mut PlayerAngle> for Angle {
    fn from(value: &mut PlayerAngle) -> Self {
        (*value).into()
    }
}

impl Momentum {
    pub fn thrust(&mut self, ang: Angle, player_move: Fixed) {
        let mom_x = player_move.wrapping_mul(ang.cos());
        let mom_y = player_move.wrapping_mul(ang.sin());

        self.x = self
            .x
            .wrapping_add(mom_x)
            .clamp(-MAX_MOMENTUM, MAX_MOMENTUM);
        self.y = self
            .y
            .wrapping_add(mom_y)
            .clamp(-MAX_MOMENTUM, MAX_MOMENTUM);
    }

    pub fn is_zero(&self) -> bool {
        self.x == 0 && self.y == 0
    }

    pub fn is_stop_range(&self) -> bool {
        let stop_iter = -STOP_SPEED..STOP_SPEED;

        stop_iter.contains(&self.x) && stop_iter.contains(&self.y) && stop_iter.contains(&self.z)
    }
}
