pub use block_map::*;
pub use line_defs::*;
pub use node::*;
pub use patch::*;
pub use patches_names::*;
pub use reject_table::*;
pub use sectors::*;
pub use segments::*;
pub use side_defs::*;
pub use sub_sectors::*;
pub use texture::*;
pub use texture_patch::*;
pub use things::*;
pub use vertexes::*;

mod block_map;
mod line_defs;
mod node;
mod patch;
mod patches_names;
mod reject_table;
mod sectors;
mod segments;
mod side_defs;
mod sub_sectors;
mod texture;
mod texture_patch;
mod things;
mod vertexes;

fn convert_c_string(buf: &[u8]) -> Result<String, String> {
    std::str::from_utf8(buf)
        .map(|s| s.trim_matches('\0').to_string())
        .map_err(|err| format!("{err:?}"))
}
