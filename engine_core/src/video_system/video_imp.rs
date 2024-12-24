use std::fmt::{Debug, Formatter};

use sdl2::hint::Hint;
use sdl2::pixels::{Color, Palette, PixelFormatEnum};
use sdl2::rect::Rect;
use sdl2::render::{CanvasBuilder, Texture, TextureAccess, WindowCanvas};
use sdl2::surface::Surface;
use sdl2::video::{Window, WindowBuilder};
use sdl2::VideoSubsystem as SdlVideoSubsystem;

use crate::video_system::{SCREEN_HEIGHT, SCREEN_WIDTH, WINDOW_HEIGHT, WINDOW_WIDTH};

pub struct VideoSystem {
    argb_buffer: Surface<'static>,
    pixel_format: PixelFormatEnum,
    renderer: WindowCanvas,
    screen_buffer: Surface<'static>,
    sdl_video_sys: SdlVideoSubsystem,
    texture: Texture,
    texture_upscaled: Texture,
}

impl VideoSystem {
    pub fn init() -> Result<Self, String> {
        let sdl_video_sys = sdl2::init()?.video()?;

        let screen = create_screen(&sdl_video_sys)?;
        let pixel_format = screen.window_pixel_format();

        let mut renderer = create_renderer(&sdl_video_sys, screen)?;
        renderer.present();

        let mut screen_buffer = create_screen_buffer()?;
        screen_buffer.fill_rect(None, Color::BLACK)?;
        set_palette(&mut screen_buffer)?;

        let mut argb_buffer = create_argb_buffer(pixel_format)?;
        argb_buffer.fill_rect(None, Color::BLACK)?;

        let texture = create_texture(&renderer, pixel_format)?;

        #[cfg(windows)]
        workaround_alt_tab_bug();

        let texture_upscaled = create_upscaled_texture(&renderer, pixel_format)?;

        hide_cursor(&sdl_video_sys);

        Ok(VideoSystem {
            argb_buffer,
            pixel_format,
            renderer,
            screen_buffer,
            sdl_video_sys,
            texture,
            texture_upscaled,
        })
    }

    pub fn finish_update(&mut self, frame_buffer: &[u8]) -> Result<(), String> {
        self.screen_buffer.with_lock_mut(|pixels| {
            pixels.copy_from_slice(frame_buffer);
        });

        let rect = Rect::new(0, 0, SCREEN_WIDTH, SCREEN_HEIGHT);

        self.screen_buffer
            .blit(rect, &mut self.argb_buffer, rect)
            .expect("");

        let pitch = self.argb_buffer.pitch();
        self.argb_buffer
            .with_lock(|pixels| self.texture.update(None, pixels, pitch as usize))
            .expect("");

        self.renderer.clear();

        self.renderer
            .with_texture_canvas(&mut self.texture_upscaled, |renderer| {
                renderer.copy(&self.texture, None, None).expect("")
            })
            .expect("");

        self.renderer
            .copy(&self.texture_upscaled, None, None)
            .expect("");
        self.renderer.present();

        Ok(())
    }
}

impl Debug for VideoSystem {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("").finish()
    }
}

fn create_screen(sdl_video_sys: &SdlVideoSubsystem) -> Result<Window, String> {
    let bounds = sdl_video_sys
        .display_bounds(0)
        .map_err(|_| "CenterWindow: Failed to read display bounds for display #0!".to_owned())?;

    let x = std::cmp::max((bounds.width().wrapping_sub(WINDOW_WIDTH)) / 2, 0);
    let y = std::cmp::max((bounds.height().wrapping_sub(WINDOW_HEIGHT)) / 2, 0);

    let mut builder = WindowBuilder::new(sdl_video_sys, "", WINDOW_WIDTH, WINDOW_HEIGHT);
    builder.fullscreen_desktop().resizable().allow_highdpi();
    builder.position(x as i32, y as i32);

    let mut screen = builder
        .build()
        .map_err(|sdl_err| format!("Error creating window for video startup: {sdl_err}"))?;

    screen
        .set_minimum_size(SCREEN_WIDTH, 240)
        .map_err(|sdl_err| format!("Error setting window minimum size: {sdl_err}"))?;

    Ok(screen)
}

fn create_renderer(
    sdl_video_sys: &SdlVideoSubsystem,
    screen: Window,
) -> Result<WindowCanvas, String> {
    let mode = sdl_video_sys
        .current_display_mode(0)
        .map_err(|sdl_err| format!("Could not get display mode for video display #0: {sdl_err}"))?;

    let renderer_builder = CanvasBuilder::new(screen).target_texture();
    let renderer_builder = if mode.refresh_rate > 0 {
        renderer_builder.present_vsync()
    } else {
        renderer_builder
    };

    let mut renderer = renderer_builder
        .build()
        .map_err(|sdl_err| format!("Error creating renderer for screen window: {sdl_err}"))?;

    renderer
        .set_logical_size(SCREEN_WIDTH, 240)
        .map_err(|sdl_err| {
            format!("Error setting device independent resolution for rendering: {sdl_err}")
        })?;
    renderer.set_integer_scale(false)?;
    renderer.set_draw_color((0, 0, 0, 255));
    renderer.clear();

    Ok(renderer)
}

fn create_screen_buffer() -> Result<Surface<'static>, String> {
    let pixel_masks = PixelFormatEnum::Index8.into_masks()?;
    Surface::from_pixelmasks(SCREEN_WIDTH, SCREEN_HEIGHT, &pixel_masks)
}

fn create_argb_buffer(pixel_format: PixelFormatEnum) -> Result<Surface<'static>, String> {
    let masks = pixel_format.into_masks()?;
    Surface::from_pixelmasks(SCREEN_WIDTH, SCREEN_HEIGHT, &masks)
}

fn create_texture(
    renderer: &WindowCanvas,
    pixel_format: PixelFormatEnum,
) -> Result<Texture, String> {
    // Set the scaling quality for rendering the intermediate texture into
    // the upscaled texture to "nearest", which is gritty and pixelated and
    // resembles software scaling pretty well.
    sdl2::hint::set("SDL_RENDER_SCALE_QUALITY", "nearest");

    let texture_creator = renderer.texture_creator();

    texture_creator
        .create_texture(
            Some(pixel_format),
            TextureAccess::Streaming,
            SCREEN_WIDTH,
            SCREEN_HEIGHT,
        )
        .map_err(|sdl_err| format!("Error creating texture for a rendering context: {sdl_err}"))
}

fn create_upscaled_texture(
    renderer: &WindowCanvas,
    pixel_format: PixelFormatEnum,
) -> Result<Texture, String> {
    let (w, h) = renderer
        .output_size()
        .map_err(|sdl_err| format!("Failed to get renderer output size: {sdl_err}"))?;

    let (w, h) = if w * SCREEN_HEIGHT < SCREEN_WIDTH * h {
        (w, w * SCREEN_HEIGHT / SCREEN_WIDTH)
    } else {
        (h * SCREEN_WIDTH / SCREEN_HEIGHT, h)
    };

    let w_upscale = std::cmp::max((w + SCREEN_WIDTH - 1) / SCREEN_WIDTH, 1);
    let h_upscale = std::cmp::max((h + SCREEN_HEIGHT - 1) / SCREEN_HEIGHT, 1);

    // Set the scaling quality for rendering the upscaled texture to "linear",
    // which looks much softer and smoother than "nearest" but does a better
    // job at downscaling from the upscaled texture to screen.
    sdl2::hint::set("SDL_RENDER_SCALE_QUALITY", "linear");

    let texture_creator = renderer.texture_creator();

    texture_creator
        .create_texture(
            Some(pixel_format),
            TextureAccess::Target,
            w_upscale * SCREEN_WIDTH,
            h_upscale * SCREEN_HEIGHT,
        )
        .map_err(|sdl_err| {
            format!("Error creating upscaled texture for a rendering context: {sdl_err}")
        })
}

#[cfg(windows)]
/// Workaround for SDL 2.0.14+ alt-tab bug (taken from Doom Retro via Prboom-plus and Woof)
fn workaround_alt_tab_bug() {
    let ver = sdl2::version::version();
    if ver.major == 2 && ver.minor == 0 && (ver.patch == 14 || ver.patch == 16) {
        let hint = Hint::Override;
        sdl2::hint::set_with_priority("SDL_VIDEO_MINIMIZE_ON_FOCUS_LOSS", "1", &hint);
    }
}

fn set_palette(surface: &mut Surface) -> Result<(), String> {
    let colors: Vec<Color> = PLAY_PAL
        .chunks(3)
        .map(|idx| {
            let r = PALETTE[idx[0] as usize] & !3;
            let g = PALETTE[idx[1] as usize] & !3;
            let b = PALETTE[idx[2] as usize] & !3;

            (r, g, b).into()
        })
        .collect();
    let pal = Palette::with_colors(&colors)?;

    surface.set_palette(&pal)
}

fn hide_cursor(sdl_video: &SdlVideoSubsystem) {
    let mouse_util = sdl_video.sdl().mouse();
    mouse_util.set_relative_mouse_mode(true);
}

const PALETTE: [u8; 256] = [
    1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26,
    27, 28, 29, 30, 31, 32, 33, 34, 35, 36, 37, 38, 39, 40, 41, 42, 43, 44, 45, 46, 47, 48, 49, 50,
    51, 52, 53, 54, 55, 56, 57, 58, 59, 60, 61, 62, 63, 64, 65, 66, 67, 68, 69, 70, 71, 72, 73, 74,
    75, 76, 77, 78, 79, 80, 81, 82, 83, 84, 85, 86, 87, 88, 89, 90, 91, 92, 93, 94, 95, 96, 97, 98,
    99, 100, 101, 102, 103, 104, 105, 106, 107, 108, 109, 110, 111, 112, 113, 114, 115, 116, 117,
    118, 119, 120, 121, 122, 123, 124, 125, 126, 127, 128, 128, 129, 130, 131, 132, 133, 134, 135,
    136, 137, 138, 139, 140, 141, 142, 143, 144, 145, 146, 147, 148, 149, 150, 151, 152, 153, 154,
    155, 156, 157, 158, 159, 160, 161, 162, 163, 164, 165, 166, 167, 168, 169, 170, 171, 172, 173,
    174, 175, 176, 177, 178, 179, 180, 181, 182, 183, 184, 185, 186, 187, 188, 189, 190, 191, 192,
    193, 194, 195, 196, 197, 198, 199, 200, 201, 202, 203, 204, 205, 206, 207, 208, 209, 210, 211,
    212, 213, 214, 215, 216, 217, 218, 219, 220, 221, 222, 223, 224, 225, 226, 227, 228, 229, 230,
    231, 232, 233, 234, 235, 236, 237, 238, 239, 240, 241, 242, 243, 244, 245, 246, 247, 248, 249,
    250, 251, 252, 253, 254, 255,
];

const PLAY_PAL: [u8; 768] = [
    0, 0, 0, 31, 23, 11, 23, 15, 7, 75, 75, 75, 255, 255, 255, 27, 27, 27, 19, 19, 19, 11, 11, 11,
    7, 7, 7, 47, 55, 31, 35, 43, 15, 23, 31, 7, 15, 23, 0, 79, 59, 43, 71, 51, 35, 63, 43, 27, 255,
    183, 183, 247, 171, 171, 243, 163, 163, 235, 151, 151, 231, 143, 143, 223, 135, 135, 219, 123,
    123, 211, 115, 115, 203, 107, 107, 199, 99, 99, 191, 91, 91, 187, 87, 87, 179, 79, 79, 175, 71,
    71, 167, 63, 63, 163, 59, 59, 155, 51, 51, 151, 47, 47, 143, 43, 43, 139, 35, 35, 131, 31, 31,
    127, 27, 27, 119, 23, 23, 115, 19, 19, 107, 15, 15, 103, 11, 11, 95, 7, 7, 91, 7, 7, 83, 7, 7,
    79, 0, 0, 71, 0, 0, 67, 0, 0, 255, 235, 223, 255, 227, 211, 255, 219, 199, 255, 211, 187, 255,
    207, 179, 255, 199, 167, 255, 191, 155, 255, 187, 147, 255, 179, 131, 247, 171, 123, 239, 163,
    115, 231, 155, 107, 223, 147, 99, 215, 139, 91, 207, 131, 83, 203, 127, 79, 191, 123, 75, 179,
    115, 71, 171, 111, 67, 163, 107, 63, 155, 99, 59, 143, 95, 55, 135, 87, 51, 127, 83, 47, 119,
    79, 43, 107, 71, 39, 95, 67, 35, 83, 63, 31, 75, 55, 27, 63, 47, 23, 51, 43, 19, 43, 35, 15,
    239, 239, 239, 231, 231, 231, 223, 223, 223, 219, 219, 219, 211, 211, 211, 203, 203, 203, 199,
    199, 199, 191, 191, 191, 183, 183, 183, 179, 179, 179, 171, 171, 171, 167, 167, 167, 159, 159,
    159, 151, 151, 151, 147, 147, 147, 139, 139, 139, 131, 131, 131, 127, 127, 127, 119, 119, 119,
    111, 111, 111, 107, 107, 107, 99, 99, 99, 91, 91, 91, 87, 87, 87, 79, 79, 79, 71, 71, 71, 67,
    67, 67, 59, 59, 59, 55, 55, 55, 47, 47, 47, 39, 39, 39, 35, 35, 35, 119, 255, 111, 111, 239,
    103, 103, 223, 95, 95, 207, 87, 91, 191, 79, 83, 175, 71, 75, 159, 63, 67, 147, 55, 63, 131,
    47, 55, 115, 43, 47, 99, 35, 39, 83, 27, 31, 67, 23, 23, 51, 15, 19, 35, 11, 11, 23, 7, 191,
    167, 143, 183, 159, 135, 175, 151, 127, 167, 143, 119, 159, 135, 111, 155, 127, 107, 147, 123,
    99, 139, 115, 91, 131, 107, 87, 123, 99, 79, 119, 95, 75, 111, 87, 67, 103, 83, 63, 95, 75, 55,
    87, 67, 51, 83, 63, 47, 159, 131, 99, 143, 119, 83, 131, 107, 75, 119, 95, 63, 103, 83, 51, 91,
    71, 43, 79, 59, 35, 67, 51, 27, 123, 127, 99, 111, 115, 87, 103, 107, 79, 91, 99, 71, 83, 87,
    59, 71, 79, 51, 63, 71, 43, 55, 63, 39, 255, 255, 115, 235, 219, 87, 215, 187, 67, 195, 155,
    47, 175, 123, 31, 155, 91, 19, 135, 67, 7, 115, 43, 0, 255, 255, 255, 255, 219, 219, 255, 187,
    187, 255, 155, 155, 255, 123, 123, 255, 95, 95, 255, 63, 63, 255, 31, 31, 255, 0, 0, 239, 0, 0,
    227, 0, 0, 215, 0, 0, 203, 0, 0, 191, 0, 0, 179, 0, 0, 167, 0, 0, 155, 0, 0, 139, 0, 0, 127, 0,
    0, 115, 0, 0, 103, 0, 0, 91, 0, 0, 79, 0, 0, 67, 0, 0, 231, 231, 255, 199, 199, 255, 171, 171,
    255, 143, 143, 255, 115, 115, 255, 83, 83, 255, 55, 55, 255, 27, 27, 255, 0, 0, 255, 0, 0, 227,
    0, 0, 203, 0, 0, 179, 0, 0, 155, 0, 0, 131, 0, 0, 107, 0, 0, 83, 255, 255, 255, 255, 235, 219,
    255, 215, 187, 255, 199, 155, 255, 179, 123, 255, 163, 91, 255, 143, 59, 255, 127, 27, 243,
    115, 23, 235, 111, 15, 223, 103, 15, 215, 95, 11, 203, 87, 7, 195, 79, 0, 183, 71, 0, 175, 67,
    0, 255, 255, 255, 255, 255, 215, 255, 255, 179, 255, 255, 143, 255, 255, 107, 255, 255, 71,
    255, 255, 35, 255, 255, 0, 167, 63, 0, 159, 55, 0, 147, 47, 0, 135, 35, 0, 79, 59, 39, 67, 47,
    27, 55, 35, 19, 47, 27, 11, 0, 0, 83, 0, 0, 71, 0, 0, 59, 0, 0, 47, 0, 0, 35, 0, 0, 23, 0, 0,
    11, 0, 0, 0, 255, 159, 67, 255, 231, 75, 255, 123, 255, 255, 0, 255, 207, 0, 207, 159, 0, 155,
    111, 0, 107, 167, 107, 107,
];
