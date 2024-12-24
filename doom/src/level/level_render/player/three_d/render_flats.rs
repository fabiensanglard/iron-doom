use bevy::prelude::*;
use std::cmp;

use crate::level::level_render::player::three_d::{
    GraphicsAssets, RenderSet, ScreenLUT, WallTexture,
};
use crate::level::map_object::{Player, PlayerAngle, Position};
use engine_core::fixed_point::Fixed;
use engine_core::geometry::Angle;
use engine_core::video_system::{VideoSystem, SCREEN_HEIGHT, SCREEN_WIDTH};
use engine_core::wad_system::WadSystem;

pub struct RenderFlatsPlugin;

impl Plugin for RenderFlatsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<VisPlanes>()
            .add_systems(PostUpdate, clear_planes.in_set(RenderSet::Prepare))
            .add_systems(PostUpdate, render_vis_planes.in_set(RenderSet::Flats));
    }
}

fn clear_planes(mut planes: ResMut<VisPlanes>) {
    planes.floor_plane = 0;
    planes.ceiling_plane = 0;
    planes.planes.clear();
}

fn render_vis_planes(
    mut vis_planes: ResMut<VisPlanes>,
    screen_lut: Res<ScreenLUT>,
    textures: Res<GraphicsAssets>,
    player_query: Query<(&PlayerAngle, &Position), With<Player>>,
    mut wad_sys: NonSendMut<WadSystem>,
    mut video_sys: NonSendMut<VideoSystem>,
) {
    let (player_angle, player_pos) = player_query.single();
    let sky = textures.wall_texture("SKY1").unwrap();
    let height: u32 = 168;
    let center_y: u32 = height / 2;

    while let Some(pl) = vis_planes.planes.pop() {
        if pl.min_x > pl.max_x {
            continue;
        }

        if pl.pic_num == "F_SKY1" {
            let skytexturemid = Fixed::from_num(SCREEN_HEIGHT / 2);

            for x in pl.min_x..=pl.max_x {
                let yl = pl.top[x as usize];
                let yh = pl.bottom[x as usize];
                let yl = yl as i32;
                let yh = yh as i32;

                let angle = (screen_lut.x_to_angle(x) + player_angle.0) >> 22;
                let tex = angle.as_usize() as i32;
                let Some(dc_source) = get_column(sky, tex) else {
                    continue;
                };
                let dc_iscale = Fixed::ONE;
                let mut frac = skytexturemid + (yl - (center_y as i32)) * dc_iscale;

                let yl = yl as u32;
                let yh = yh as u32;

                for j in yl..=yh {
                    let frac_idx = ((frac.to_num::<i32>()) & 127) as usize;
                    let idx = frac_idx % dc_source.len();
                    let color = dc_source[idx];
                    video_sys.draw_pixel(x, j, color);
                    frac += dc_iscale;
                }
            }
            continue;
        }

        let ds_source = wad_sys.cache_lump_name(&pl.pic_num).unwrap();
        let plane_height = (pl.height - player_pos.z).abs();

        let stop = pl.max_x + 1;

        for x in pl.min_x..=stop {
            let (t1, b1) = if x == 0 {
                (255, 0)
            } else {
                (pl.top[(x - 1) as usize], pl.bottom[(x - 1) as usize])
            };
            let (t2, b2) = if x == stop {
                (255, 0)
            } else {
                (pl.top[x as usize], pl.bottom[x as usize])
            };

            make_spans(
                x,
                t1 as u32,
                b1 as u32,
                t2 as u32,
                b2 as u32,
                *player_pos,
                player_angle.0,
                plane_height,
                &ds_source,
                &mut vis_planes,
                &screen_lut,
                &mut video_sys,
            )
        }
    }
}

fn get_column(texture: &WallTexture, col: i32) -> Option<&Vec<u8>> {
    let col_mask = (1 << texture.width.ilog2()) - 1;
    let masked = (col & col_mask) as usize;

    texture.cols.get(&masked)
}

fn make_spans(
    x: u32,
    mut t1: u32,
    mut b1: u32,
    mut t2: u32,
    mut b2: u32,
    player_pos: Position,
    player_ang: Angle,
    plane_height: Fixed,
    ds_source: &[u8],
    vis_planes: &mut VisPlanes,
    screen_lut: &ScreenLUT,
    video_sys: &mut VideoSystem,
) {
    while t1 < t2 && t1 <= b1 {
        map_plane(
            t1,
            vis_planes.span_start[t1 as usize],
            x - 1,
            player_pos,
            player_ang,
            plane_height,
            ds_source,
            screen_lut,
            video_sys,
        );
        t1 += 1;
    }
    while b1 > b2 && b1 >= t1 {
        map_plane(
            b1,
            vis_planes.span_start[b1 as usize],
            x - 1,
            player_pos,
            player_ang,
            plane_height,
            ds_source,
            screen_lut,
            video_sys,
        );
        b1 -= 1;
    }

    while t2 < t1 && t2 <= b2 {
        vis_planes.span_start[t2 as usize] = x;
        t2 += 1;
    }
    while b2 > b1 && b2 >= t2 {
        vis_planes.span_start[b2 as usize] = x;
        b2 -= 1;
    }
}

fn map_plane(
    y: u32,
    x1: u32,
    x2: u32,
    player_pos: Position,
    player_ang: Angle,
    plane_height: Fixed,
    ds_source: &[u8],
    screen_lut: &ScreenLUT,
    video_sys: &mut VideoSystem,
) {
    let mut dy = Fixed::from_num((y as i32) - 168 / 2) + Fixed::ONE / 2;
    dy = dy.abs();
    let slope = Fixed::from_num(320 / 2) / dy;

    let angle = player_ang - Angle::DEG_90;
    let basexscale = angle.cos() / Fixed::from_num(160);
    let baseyscale = -(angle.sin() / Fixed::from_num(160));

    let distance = plane_height * slope;
    let xstep = distance * basexscale;
    let ystep = distance * baseyscale;

    let dist_scale = comp_dist_scale(screen_lut, x1);
    let length = distance * dist_scale;
    let angle = player_ang + screen_lut.x_to_angle(x1);

    let ds_xfrac = (player_pos.x + angle.cos() * length).to_bits();
    let ds_xstep = xstep.to_bits();

    let ds_yfrac = (-player_pos.y - (angle.sin() * length)).to_bits();
    let ds_ystep = ystep.to_bits();

    let ds_y = y;
    let ds_x1 = x1;
    let ds_x2 = x2;

    let mut position = (((ds_xfrac << 10) & 0xffff0000u32 as i32) | ((ds_yfrac >> 6) & 0x0000ffff)) as u32;
    let step = (((ds_xstep << 10) & 0xffff0000u32 as i32) | ((ds_ystep >> 6) & 0x0000ffff)) as u32;

    for x in ds_x1..=ds_x2 {
        let ytemp = (position >> 4) & 0x0fc0;
        let xtemp = position >> 26;
        let spot = xtemp | ytemp;
        let color = ds_source[spot as usize];
        video_sys.draw_pixel(x, ds_y, color);
        position = position.wrapping_add(step);
    }
}

fn comp_dist_scale(screen_lut: &ScreenLUT, x: u32) -> Fixed {
    let ang = screen_lut.x_to_angle(x);
    let cosadj = ang.cos().abs();
    Fixed::ONE / cosadj
}

#[derive(Event)]
pub struct MakeSpan {
    pub segments: Vec<Entity>,
}

#[derive(Resource)]
pub struct VisPlanes {
    ceiling_plane: usize,
    floor_plane: usize,
    planes: Vec<VisPlane>,
    span_start: [u32; SCREEN_HEIGHT as usize],
}

impl Default for VisPlanes {
    fn default() -> Self {
        Self {
            ceiling_plane: 0,
            floor_plane: 0,
            planes: vec![],
            span_start: [0; SCREEN_HEIGHT as usize],
        }
    }
}

impl VisPlanes {
    pub fn find_ceiling(&mut self, height: Fixed, pic_num: &str) {
        self.ceiling_plane = self.find_plane(height, pic_num);
    }

    pub fn check_ceiling(&mut self, start: u32, stop: u32) {
        self.ceiling_plane = self.check_plane(self.ceiling_plane, start, stop);
    }

    pub fn ceiling(&mut self) -> &mut VisPlane {
        &mut self.planes[self.ceiling_plane]
    }

    pub fn find_floor(&mut self, height: Fixed, pic_num: &str) {
        self.floor_plane = self.find_plane(height, pic_num);
    }

    pub fn check_floor(&mut self, start: u32, stop: u32) {
        self.floor_plane = self.check_plane(self.floor_plane, start, stop);
    }

    pub fn floor(&mut self) -> &mut VisPlane {
        &mut self.planes[self.floor_plane]
    }

    fn find_plane(&mut self, mut height: Fixed, pic_num: &str) -> usize {
        if pic_num == "F_SKY1" {
            height = Fixed::ZERO;
        }
        for i in 0..self.planes.len() {
            if self.planes[i].height == height && self.planes[i].pic_num == pic_num {
                return i;
            }
        }
        self.planes.push(VisPlane::new(height, pic_num));
        self.planes.len() - 1
    }

    fn check_plane(&mut self, pl_idx: usize, start: u32, stop: u32) -> usize {
        if self.check_overlap(pl_idx, start, stop) {
            // make a new visplane
            let pl = self.planes.get(pl_idx).unwrap();
            let mut new_plane = VisPlane::new(pl.height, &pl.pic_num);
            new_plane.min_x = start;
            new_plane.max_x = stop;

            self.planes.push(new_plane);
            return self.planes.len() - 1;
        }
        // use the same one
        let pl = self.planes.get_mut(pl_idx).unwrap();
        pl.min_x = cmp::min(start, pl.min_x);
        pl.max_x = cmp::max(stop, pl.max_x);

        pl_idx
    }

    fn check_overlap(&self, pl_idx: usize, start: u32, stop: u32) -> bool {
        let pl = self.planes.get(pl_idx).unwrap();
        let intrl = cmp::max(pl.min_x, start);
        let intrh = cmp::min(pl.max_x, stop);

        for x in intrl..=intrh {
            if pl.top[x as usize] != 0xff {
                return true;
            }
        }

        false
    }
}

pub struct VisPlane {
    height: Fixed,
    pic_num: String,
    min_x: u32,
    max_x: u32,
    pub top: [u8; SCREEN_WIDTH as usize],
    pub bottom: [u8; SCREEN_WIDTH as usize],
}

impl VisPlane {
    pub fn new(height: Fixed, pic_num: &str) -> Self {
        Self {
            height,
            pic_num: pic_num.to_string(),
            min_x: 1,
            max_x: 0,
            top: [255; SCREEN_WIDTH as usize],
            bottom: [0; SCREEN_WIDTH as usize],
        }
    }
}
