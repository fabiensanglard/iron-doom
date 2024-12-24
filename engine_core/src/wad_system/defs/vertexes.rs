use derive_more::{AsRef, Deref, DerefMut, Index, IndexMut, IntoIterator};

const WAD_VERTEX_BYTE_SIZE: usize = 4;

#[derive(AsRef, Deref, DerefMut, IntoIterator, Index, IndexMut)]
#[into_iterator(owned, ref, ref_mut)]
pub struct WadVertexes(Vec<WadVertex>);

/// Data structure used to represent coordinates on the level map.
pub struct WadVertex {
    /// x position.
    pub x: i16,

    /// y position.
    pub y: i16,
}

impl TryFrom<&[u8]> for WadVertexes {
    type Error = String;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        let len = value.len();
        if len % WAD_VERTEX_BYTE_SIZE != 0 {
            let err_msg = format!(
                "Error while converting WadVertexes: \
                 byte array must be divisible by {WAD_VERTEX_BYTE_SIZE}!"
            );
            return Err(err_msg);
        }

        let vertexes: Result<Vec<WadVertex>, String> = value
            .chunks_exact(WAD_VERTEX_BYTE_SIZE)
            .map(|vertex_bytes| vertex_bytes.try_into())
            .collect();
        let vertexes = vertexes?;

        Ok(Self(vertexes))
    }
}

impl TryFrom<&[u8]> for WadVertex {
    type Error = String;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        let len = value.len();

        if len != WAD_VERTEX_BYTE_SIZE {
            let err_msg = format!(
                "Error while converting WadVertex: \
                 byte array must be {WAD_VERTEX_BYTE_SIZE} bytes wide!"
            );
            return Err(err_msg);
        }

        let x = i16::from_le_bytes([value[0], value[1]]);
        let y = i16::from_le_bytes([value[2], value[3]]);

        Ok(Self { x, y })
    }
}
