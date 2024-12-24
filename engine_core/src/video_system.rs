use std::rc::Rc;

use bevy::app::{App, Plugin};

use crate::command_line::CommandLine;
use crate::wad_system::WadPatch;

mod video_imp;

pub const SCREEN_HEIGHT: u32 = 200;
pub const SCREEN_WIDTH: u32 = 320;
const WINDOW_HEIGHT: u32 = 600;
const WINDOW_WIDTH: u32 = 800;

pub struct VideoPlugin;

impl Plugin for VideoPlugin {
    fn build(&self, app: &mut App) {
        let cli = app.world().non_send_resource::<Rc<CommandLine>>();
        let cli = Rc::clone(cli);
        let video_sys = VideoSystem::init(cli).expect("Failed to initialize VideoSystem");

        app.insert_non_send_resource(video_sys);
    }
}

#[derive(Debug)]
pub struct VideoSystem {
    #[allow(dead_code)]
    inner: video_imp::VideoSystem,
    cli: Rc<CommandLine>,
    frame_buffer: FrameBuffer,
}

impl VideoSystem {
    pub fn init(cli: Rc<CommandLine>) -> Result<VideoSystem, String> {
        let inner = video_imp::VideoSystem::init()
            .map_err(|err| format!("Failed to initialize video: {err}"))?;

        Ok(VideoSystem {
            inner,
            cli,
            frame_buffer: FrameBuffer::new(SCREEN_HEIGHT, SCREEN_WIDTH),
        })
    }

    pub fn frame_buf_mut(&mut self) -> &mut [u8] {
        &mut self.frame_buffer.buf
    }

    pub fn read_screen(&self, buf: &mut [u8]) {
        buf.copy_from_slice(&self.frame_buffer.buf);
    }

    pub fn draw_from_buf(&mut self, buf: &[u8]) {
        self.frame_buffer.buf.copy_from_slice(buf);
    }

    pub fn draw_patch(&mut self, x: u32, y: u32, patch: &WadPatch) {
        self.frame_buffer.draw_patch(x, y, patch);
    }

    pub fn copy_rect<R: Into<Option<Rect>>>(
        &mut self,
        dest_x: u32,
        dest_y: u32,
        src: &FrameBuffer,
        src_rect: R,
    ) {
        self.frame_buffer.copy_rect(dest_x, dest_y, src, src_rect);
    }

    pub fn clear(&mut self, color: u8) {
        self.frame_buffer.clear(color);
    }

    pub fn update(&mut self) {
        let buf = &mut self.frame_buffer.buf;
        self.inner
            .finish_update(buf)
            .expect("Error while updating video");
        self.frame_buffer.clear(0);
    }

    pub fn draw_column(&mut self, x: u32, y1: u32, y2: u32, color: u8) {
        self.frame_buffer.draw_column(x, y1, y2, color);
    }

    pub fn draw_pixel(&mut self, x: u32, y: u32, color: u8) {
        self.frame_buffer.draw_pixel(x, y, color);
    }
}

#[derive(Debug)]
pub struct FrameBuffer {
    buf: Vec<u8>,
    height: u32,
    width: u32,
}

impl FrameBuffer {
    pub fn new(height: u32, width: u32) -> Self {
        let pixels_count = height * width;

        Self {
            buf: vec![0; pixels_count as usize],
            height,
            width,
        }
    }

    pub fn draw_patch(&mut self, x: u32, y: u32, patch: &WadPatch) {
        let top_off = patch.header.top_off as i32;
        let left_off = patch.header.left_off as i32;

        let y = y.wrapping_add_signed(-top_off);
        let x = x.wrapping_add_signed(-left_off);

        let start_pixel = y.wrapping_mul(SCREEN_WIDTH).wrapping_add(x);
        let column_iterator = (0..SCREEN_WIDTH)
            .zip(&patch.columns)
            .flat_map(|(col_idx, col)| {
                let posts = col.iter();
                posts.map(move |post| (col_idx, post))
            });

        for (col, post) in column_iterator {
            let post_off = SCREEN_WIDTH.wrapping_mul(post.top_delta as u32);
            let skip = start_pixel.wrapping_add(col).wrapping_add(post_off);

            let pixel_iterator = self
                .buf
                .iter_mut()
                .skip(skip as usize)
                .step_by(SCREEN_WIDTH as usize)
                .zip(&post.data);

            for (dest, src) in pixel_iterator {
                *dest = *src;
            }
        }
    }

    pub fn copy_rect<R: Into<Option<Rect>>>(
        &mut self,
        dest_x: u32,
        dest_y: u32,
        src: &FrameBuffer,
        src_rect: R,
    ) {
        // While reading this piece of code, I recommend building the mental
        // image of two overlapping rectangle, where "src_rect.x" and "src_rect.y"
        // specify the second rectangle position relative to the first one. Then,
        // "src_rect.width" and "src_rect.height" tells how much of the second
        // rectangle we want to copy to the first one.

        let src_rect = if let Some(rect) = src_rect.into() {
            rect
        } else {
            // If "None", then copy all the framebuffer.
            Rect::new(0, 0, src.width, src.height)
        };

        // First we divide each framebuffer in pixel rows.
        // Remember "src_rect.y" indicates how many rows we must skip.
        let src_rows = src
            .buf
            .chunks_exact(src.width as usize)
            .skip(src_rect.y as usize);
        let dest_rows = self
            .buf
            .chunks_exact_mut(self.width as usize)
            .skip(dest_y as usize);

        // Now for each pixel row, we must skip some pixels.
        // In the case of the source framebuffer, this value is indicated by "src_rect.x".
        // In the case of the destiny framebuffer, "dest_x" is used.
        let src_rows = src_rows.map(|src_bytes| src_bytes.iter().skip(src_rect.x as usize));
        let dest_rows = dest_rows.map(|dest_bytes| dest_bytes.iter_mut().skip(dest_x as usize));

        // Here we use "src_rect.height" to limit the number of rows...
        for (dest_row, src_row) in dest_rows.zip(src_rows).take(src_rect.height as usize) {
            // And use "src_rect.width" to limit the number of pixels per row.
            for (dest_pixel, src_pixel) in dest_row.zip(src_row).take(src_rect.width as usize) {
                *dest_pixel = *src_pixel;
            }
        }
    }

    pub fn clear(&mut self, color: u8) {
        self.buf.as_mut_slice().fill(color);
    }

    pub fn draw_pixel(&mut self, x: u32, y: u32, color: u8) {
        let idx = (y * self.width + x) as usize;
        if idx < self.buf.len() {
            self.buf[idx] = color;
        }
    }

    pub fn draw_column(&mut self, x: u32, y1: u32, y2: u32, color: u8) {
        let (y1, y2) = if y1 < y2 { (y1, y2) } else { (y2, y1) };

        let buf_iter = self
            .buf
            .iter_mut()
            .skip((y1 * self.width + x) as usize)
            .step_by(self.width as usize)
            .take((y2 - y1) as usize);

        for pixel in buf_iter {
            *pixel = color;
        }
    }
}

pub struct Rect {
    x: u32,
    y: u32,
    width: u32,
    height: u32,
}

impl Rect {
    pub fn new(x: u32, y: u32, width: u32, height: u32) -> Rect {
        Self {
            x,
            y,
            width,
            height,
        }
    }
}
