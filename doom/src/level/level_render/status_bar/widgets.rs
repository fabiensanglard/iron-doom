use bevy::prelude::*;

use engine_core::video_system::{FrameBuffer, Rect, VideoSystem};
use engine_core::wad_system::{WadPatch, WadSystem};

use super::{NUM_PAIN_FACES, NUM_STRAIGHT_FACES, ST_Y};

#[derive(Resource)]
pub struct StatusBarAssets {
    pub cards: Vec<WadPatch>,
    pub faces: Vec<WadPatch>,
    pub negative_symbol: WadPatch,
    pub percent_symbol: WadPatch,
    pub small_gray_nums: Vec<WadPatch>,
    pub small_yellow_nums: Vec<WadPatch>,
    pub status_bar_bg: WadPatch,
    pub tall_nums: Vec<WadPatch>,
    pub weapon_inventory: WadPatch,
}

impl FromWorld for StatusBarAssets {
    fn from_world(world: &mut World) -> Self {
        let mut wad_sys = world.non_send_resource_mut::<WadSystem>();

        Self {
            cards: load_cards(&mut wad_sys),
            faces: load_faces(&mut wad_sys),
            negative_symbol: load_patch(&mut wad_sys, "STTMINUS"),
            percent_symbol: load_patch(&mut wad_sys, "STTPRCNT"),
            small_gray_nums: load_small_gray_nums(&mut wad_sys),
            small_yellow_nums: load_small_yellow_nums(&mut wad_sys),
            status_bar_bg: load_patch(&mut wad_sys, "STBAR"),
            tall_nums: load_tall_nums(&mut wad_sys),
            weapon_inventory: load_patch(&mut wad_sys, "STARMS"),
        }
    }
}

fn load_patch(wad_sys: &mut WadSystem, lump_name: &str) -> WadPatch {
    let err_msg = format!("Not found {lump_name}");
    let patch_data = wad_sys.cache_lump_name(lump_name).expect(&err_msg);

    let err_msg = format!("Failed to convert {lump_name} lump data");
    patch_data.as_slice().try_into().expect(&err_msg)
}

fn load_cards(wad_sys: &mut WadSystem) -> Vec<WadPatch> {
    let mut cards: Vec<WadPatch> = vec![];

    for i in 0..6 {
        let lump_name = format!("STKEYS{i}");
        cards.push(load_patch(wad_sys, &lump_name));
    }

    cards
}

fn load_faces(wad_sys: &mut WadSystem) -> Vec<WadPatch> {
    let mut faces: Vec<WadPatch> = vec![];

    for i in 0..NUM_PAIN_FACES {
        for j in 0..NUM_STRAIGHT_FACES {
            let lump_name = format!("STFST{i}{j}");
            faces.push(load_patch(wad_sys, &lump_name));
        }

        let lump_name = format!("STFTR{i}0");
        faces.push(load_patch(wad_sys, &lump_name));

        let lump_name = format!("STFTL{i}0");
        faces.push(load_patch(wad_sys, &lump_name));

        let lump_name = format!("STFOUCH{i}");
        faces.push(load_patch(wad_sys, &lump_name));

        let lump_name = format!("STFEVL{i}");
        faces.push(load_patch(wad_sys, &lump_name));

        let lump_name = format!("STFKILL{i}");
        faces.push(load_patch(wad_sys, &lump_name));
    }

    faces.push(load_patch(wad_sys, "STFGOD0"));
    faces.push(load_patch(wad_sys, "STFDEAD0"));

    faces
}

fn load_small_gray_nums(wad_sys: &mut WadSystem) -> Vec<WadPatch> {
    let mut tall_nums: Vec<WadPatch> = vec![];

    for i in 0..10 {
        let lump_name = format!("STGNUM{i}");
        tall_nums.push(load_patch(wad_sys, &lump_name));
    }

    tall_nums
}

fn load_small_yellow_nums(wad_sys: &mut WadSystem) -> Vec<WadPatch> {
    let mut tall_nums: Vec<WadPatch> = vec![];

    for i in 0..10 {
        let lump_name = format!("STYSNUM{i}");
        tall_nums.push(load_patch(wad_sys, &lump_name));
    }

    tall_nums
}

fn load_tall_nums(wad_sys: &mut WadSystem) -> Vec<WadPatch> {
    let mut tall_nums: Vec<WadPatch> = vec![];

    for i in 0..10 {
        let lump_name = format!("STTNUM{i}");
        tall_nums.push(load_patch(wad_sys, &lump_name));
    }

    tall_nums
}

pub struct NumberWidget {
    // upper right-hand corner
    //  of the number (right-justified)
    x: u32,
    y: u32,

    // max # of digits in number
    width: u32,

    // pointer to current value
    num: i32,

    // pointer to boolean stating
    //  whether to update number
    on: bool,
}

impl NumberWidget {
    pub fn new(x: u32, y: u32, width: u32, num: i32, on: bool) -> Self {
        Self {
            x,
            y,
            width,
            num,
            on,
        }
    }

    pub fn draw(&self, video_sys: &mut VideoSystem, patches: &[WadPatch], neg_symbol: &WadPatch) {
        if !self.on {
            return;
        }

        let mut num_digits: u32 = self.width;
        let w = patches[0].header.width as u32;
        let h = patches[0].header.height as u32;
        let mut x = self.x;
        let y = self.y;
        let mut num = self.num;
        let neg = num < 0;

        if neg {
            if num_digits == 2 && num < -9 {
                num = -9;
            } else if num_digits == 3 && num < -99 {
                num = -99;
            }

            num = -num;
        }

        if num == 1994 {
            return;
        }

        if num == 0 {
            video_sys.draw_patch(x - w, y, &patches[0]);
        }

        while num != 0 && num_digits != 0 {
            x -= w;
            video_sys.draw_patch(x, y, &patches[(num % 10) as usize]);
            num /= 10;
            num_digits -= 1;
        }

        if neg {
            video_sys.draw_patch(x - 8, y, neg_symbol);
        }
    }

    pub fn draw_percent(
        &self,
        video_sys: &mut VideoSystem,
        patches: &[WadPatch],
        neg_symbol: &WadPatch,
        percent: &WadPatch,
        refresh: bool,
    ) {
        if refresh && self.on {
            video_sys.draw_patch(self.x, self.y, percent);
        }

        self.draw(video_sys, patches, neg_symbol);
    }
}

pub struct BinIconWidget {
    // center-justified location of icon
    x: u32,
    y: u32,

    // last icon value
    old_val: bool,

    // pointer to current icon status
    val: bool,

    // pointer to boolean
    //  stating whether to update icon
    on: bool,
}

impl BinIconWidget {
    pub fn new(x: u32, y: u32, old_val: bool, val: bool, on: bool) -> Self {
        Self {
            x,
            y,
            old_val,
            val,
            on,
        }
    }

    pub fn draw(
        &mut self,
        video_sys: &mut VideoSystem,
        backing_screen: &FrameBuffer,
        patch: &WadPatch,
        refresh: bool,
    ) {
        if !self.on || (self.old_val == self.val && !refresh) {
            return;
        }

        if self.val {
            video_sys.draw_patch(self.x, self.y, patch);
        } else {
            let left_off = patch.header.left_off as i32;
            let top_off = patch.header.top_off as i32;

            let x = self.x.wrapping_add_signed(-left_off);
            let y = self.y.wrapping_add_signed(-top_off);
            let w = patch.header.width as u32;
            let h = patch.header.height as u32;

            let rect = Rect::new(x, y - ST_Y, w, h);

            video_sys.copy_rect(x, y, backing_screen, Some(rect));
        }

        self.old_val = self.val;
    }
}

pub struct MultiIconWidget {
    // center-justified location of icons
    x: u32,
    y: u32,

    // last icon number
    old_num: u32,

    // pointer to current icon
    num: u32,

    // pointer to boolean stating
    //  whether to update icon
    on: bool,
}

impl MultiIconWidget {
    pub fn new(x: u32, y: u32, old_num: u32, num: u32, on: bool) -> Self {
        Self {
            x,
            y,
            old_num,
            num,
            on,
        }
    }

    pub fn draw(
        &mut self,
        video_sys: &mut VideoSystem,
        backing_screen: &FrameBuffer,
        patches: &[WadPatch],
        refresh: bool,
    ) {
        if !self.on || (self.old_num == self.num && !refresh) {
            return;
        }

        if (self.old_num as usize) < patches.len() {
            let patch = &patches[self.old_num as usize];

            let left_off = patch.header.left_off as i32;
            let top_off = patch.header.top_off as i32;

            let x = self.x.wrapping_add_signed(-left_off);
            let y = self.y.wrapping_add_signed(-top_off);
            let w = patch.header.width as u32;
            let h = patch.header.height as u32;

            let rect = Rect::new(x, y - ST_Y, w, h);

            video_sys.copy_rect(x, y, backing_screen, Some(rect));
        }

        let patch = &patches[self.num as usize];
        video_sys.draw_patch(self.x, self.y, patch);

        self.old_num = self.num;
    }
}
