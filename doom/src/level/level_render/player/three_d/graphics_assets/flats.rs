use bevy::prelude::{NonSend, ResMut};
use rustc_hash::FxHashMap;

use engine_core::wad_system::WadSystem;

use super::GraphicsAssets;

pub fn load(wad_sys: NonSend<WadSystem>, mut assets: ResMut<GraphicsAssets>) -> Result<(), String> {
    let first_flat = wad_sys.get_lump_idx_or_err("F_START")? + 1;
    let last_flat = wad_sys.get_lump_idx_or_err("F_END")? - 1;

    let mut flats = FxHashMap::default();
    for i in first_flat..=last_flat {
        let lump = wad_sys.get_lump(i).unwrap();
        let name = lump.name().to_owned();
        flats.insert(name.clone(), name);
    }
    assets.flats = flats;

    Ok(())
}
