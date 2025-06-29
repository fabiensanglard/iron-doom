use crate::sdl_window::SdlWindow;
use anyhow::Error;
use bevy::prelude::*;
use exit::macros::sys_fail;
use game_state::conditions::{in_playing_state, in_setup_state};
use gamma_table::GAMMA_TABLE;
use sdl2::pixels::{Color as SdlColor, Palette as SdlPalette};
use wad::prelude::*;

mod gamma_table;

pub struct PalettePlugin;

impl Plugin for PalettePlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<SetPalette>()
            .init_resource::<Palette>()
            .add_event::<IncreaseGamma>()
            .init_resource::<GammaLevel>()
            .add_systems(Update, set_window_palette.run_if(in_setup_state()))
            .add_systems(
                PostUpdate,
                update_gamma.run_if(on_event::<IncreaseGamma>.and(in_playing_state())),
            )
            .add_systems(
                PostUpdate,
                (set_palette, set_window_palette)
                    .chain()
                    .after(update_gamma)
                    .run_if(on_event::<SetPalette>.and(in_playing_state())),
            );
    }
}

fn update_gamma(
    mut gamma_level: ResMut<GammaLevel>,
    current_palette: Res<Palette>,
    mut gamma_events: EventReader<IncreaseGamma>,
    mut palette_events: EventWriter<SetPalette>,
) {
    for _ in gamma_events.read() {
        gamma_level.0 = (gamma_level.0 + 1) % GAMMA_TABLE.len();
    }
    // After updating the gamma level, refresh the palette to ensure it
    // stays in sync with the new gamma setting.
    // Note: Vanilla Doom had a bug where changing the gamma level would
    // accidentally reset the palette to the default (palette 0).
    // More info on this bug can be found here:
    // https://doomwiki.org/wiki/Gamma_correction_resets_palette
    palette_events.send(SetPalette(current_palette.0));
}

fn set_palette(mut palette_events: EventReader<SetPalette>, mut current_palette: ResMut<Palette>) {
    let Some(event) = palette_events.read().last() else {
        return;
    };
    current_palette.0 = event.0;
}

#[sys_fail]
fn set_window_palette(
    current_palette: Res<Palette>,
    gamma_level: Res<GammaLevel>,
    mut sdl_window: NonSendMut<SdlWindow>,
    wad_file: Res<WadFile>,
) {
    let palette = wad_file.get_palette(current_palette.0);
    let gamma = &GAMMA_TABLE[gamma_level.0];
    let mut sdl_palette = Vec::with_capacity(256);

    for color in palette {
        let mut r = color.r;
        let mut g = color.g;
        let mut b = color.b;

        // Apply gamma correction.
        r = gamma[r as usize];
        g = gamma[g as usize];
        b = gamma[b as usize];

        // Zero out the bottom two bits of each channel:
        // the PC VGA controller only supports 6 bits of accuracy.
        r &= !3;
        g &= !3;
        b &= !3;

        let sdl_color: SdlColor = (r, g, b).into();
        sdl_palette.push(sdl_color);
    }

    let sdl_palette = SdlPalette::with_colors(&sdl_palette).map_err(Error::msg)?;
    sdl_window.update_palette(&sdl_palette)?;
}

#[derive(Resource, Default, Debug)]
struct GammaLevel(pub usize);

#[derive(Event, Default)]
pub struct IncreaseGamma;

#[derive(Resource, Debug)]
struct Palette(pub PaletteVariant);

impl Default for Palette {
    fn default() -> Self {
        Palette(PaletteVariant::Palette0)
    }
}

#[derive(Event)]
pub struct SetPalette(pub PaletteVariant);
