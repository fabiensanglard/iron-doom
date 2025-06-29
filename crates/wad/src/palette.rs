use crate::lump::LumpsDirectory;
use anyhow::{anyhow, bail, Error, Result};
use derive_more::{Deref, Index, IntoIterator};

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum PaletteVariant {
    /// Normal palette, used for most situations.
    Palette0,
    /// Unused.
    ///
    /// 11.1% red tint of RGB(255, 0, 0).
    Palette1,
    /// Used to show pain when the player is hurt, and reddens
    /// the screen when the player picks up a berserk pack.
    ///
    /// 22.2% red tint of RGB(255, 0, 0).
    Palette2,
    /// Used to show pain when the player is hurt, and reddens
    /// the screen when the player picks up a berserk pack.
    ///
    /// 33.3% red tint of RGB(255, 0, 0).
    Palette3,
    /// Used to show pain when the player is hurt, and reddens
    /// the screen when the player picks up a berserk pack.
    ///
    /// 44.4% red tint of RGB(255, 0, 0).
    Palette4,
    /// Used to show pain when the player is hurt, and reddens
    /// the screen when the player picks up a berserk pack.
    ///
    /// 55.5% red tint of RGB(255, 0, 0).
    Palette5,
    /// Used to show pain when the player is hurt, and reddens
    /// the screen when the player picks up a berserk pack.
    ///
    /// 66.6% red tint of RGB(255, 0, 0).
    Palette6,
    /// Used to show pain when the player is hurt, and reddens
    /// the screen when the player picks up a berserk pack.
    ///
    /// 77.7% red tint of RGB(255, 0, 0).
    Palette7,
    /// Used to show pain when the player is hurt, and reddens
    /// the screen when the player picks up a berserk pack.
    ///
    /// 88.8% red tint of RGB(255, 0, 0).
    Palette8,
    /// Unused.
    ///
    /// 12.5% yellow tint of RGB(215, 186, 69)
    Palette9,
    /// Used very briefly as the player picks up items.
    ///
    /// 25% yellow tint of RGB(215, 186, 69)
    Palette10,
    /// Used very briefly as the player picks up items.
    ///
    /// 37.5% yellow tint of RGB(215, 186, 69)
    Palette11,
    /// Used very briefly as the player picks up items.
    ///
    /// 50% yellow tint of RGB(215, 186, 69)
    Palette12,
    /// Green tint, used when the radiation suit is being worn.
    ///
    /// 12.5% of RGB(0, 256, 0).
    Palette13,
}

impl From<PaletteVariant> for usize {
    fn from(value: PaletteVariant) -> Self {
        value as usize
    }
}

impl TryFrom<usize> for PaletteVariant {
    type Error = Error;

    fn try_from(value: usize) -> Result<Self, Self::Error> {
        use PaletteVariant::*;
        match value {
            0 => Ok(Palette0),
            1 => Ok(Palette1),
            2 => Ok(Palette2),
            3 => Ok(Palette3),
            4 => Ok(Palette4),
            5 => Ok(Palette5),
            6 => Ok(Palette6),
            7 => Ok(Palette7),
            8 => Ok(Palette8),
            9 => Ok(Palette9),
            10 => Ok(Palette10),
            11 => Ok(Palette11),
            12 => Ok(Palette12),
            13 => Ok(Palette13),
            _ => Err(anyhow!("Invalid conversion to Palette variant")),
        }
    }
}

#[derive(Deref, Index, Debug)]
pub struct Palettes(Vec<Palette>);

#[derive(Deref, Index, IntoIterator, Debug)]
#[into_iterator(ref)]
pub struct Palette(Vec<Color>);

#[derive(Debug)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

pub struct PalettesParser;

impl PalettesParser {
    pub fn parse(lumps_dir: &LumpsDirectory) -> Result<Palettes> {
        let Some(palettes_lump) = lumps_dir.get("PLAYPAL") else {
            bail!("Missing palette lump");
        };
        let palettes_data = palettes_lump.data();
        let mut palettes = Vec::with_capacity(14);
        for palette_data in palettes_data.chunks_exact(768) {
            let palette = PaletteParser::parse(palette_data)?;
            palettes.push(palette);
        }
        Ok(Palettes(palettes))
    }
}

struct PaletteParser;

impl PaletteParser {
    fn parse(palette_data: &[u8]) -> Result<Palette> {
        let mut colors = Vec::with_capacity(256);
        for color in palette_data.chunks_exact(3) {
            colors.push(Color {
                r: color[0],
                g: color[1],
                b: color[2],
            });
        }
        Ok(Palette(colors))
    }
}
