use derive_more::{AsRef, Deref, DerefMut, Index, IndexMut, IntoIterator};

use crate::BoundingBox;

const WAD_NODE_BYTE_SIZE: usize = 28;

#[derive(AsRef, Deref, DerefMut, IntoIterator, Index, IndexMut)]
#[into_iterator(owned, ref, ref_mut)]
pub struct WadNodes(Vec<WadNode>);

#[derive(Copy, Clone)]
pub struct WadNode {
    /// x coordinate of partition line start.
    pub x: i16,

    /// y coordinate of partition line start.
    pub y: i16,

    /// Change in x from start to end of partition line.
    pub dx: i16,

    /// Change in y from start to end of partition line.
    pub dy: i16,

    /// Right bounding box.
    pub right_bounding_box: BoundingBox,

    /// Left bounding box.
    pub left_bounding_box: BoundingBox,

    /// Right child.
    pub right_child: u16,

    /// Left child.
    pub left_child: u16,
}

impl TryFrom<&[u8]> for WadNodes {
    type Error = String;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        let len = value.len();

        if len % WAD_NODE_BYTE_SIZE != 0 {
            let err_msg = format!(
                "Error while converting WadNodes: \
                 byte array must be divisible by {WAD_NODE_BYTE_SIZE}!"
            );
            return Err(err_msg);
        }

        let nodes: Result<Vec<WadNode>, String> = value
            .chunks_exact(WAD_NODE_BYTE_SIZE)
            .map(|node_bytes| node_bytes.try_into())
            .collect();
        let nodes = nodes?;

        Ok(Self(nodes))
    }
}

impl TryFrom<&[u8]> for WadNode {
    type Error = String;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        let len = value.len();

        if len != WAD_NODE_BYTE_SIZE {
            let err_msg = format!(
                "Error while converting WadNode: \
                 byte array must be {WAD_NODE_BYTE_SIZE} bytes wide!"
            );
            return Err(err_msg);
        }

        Ok(Self {
            x: i16::from_le_bytes([value[0], value[1]]),
            y: i16::from_le_bytes([value[2], value[3]]),
            dx: i16::from_le_bytes([value[4], value[5]]),
            dy: i16::from_le_bytes([value[6], value[7]]),
            right_bounding_box: create_bounding_box(&value[8..16])?,
            left_bounding_box: create_bounding_box(&value[16..24])?,
            right_child: u16::from_le_bytes([value[24], value[25]]),
            left_child: u16::from_le_bytes([value[26], value[27]]),
        })
    }
}

fn create_bounding_box(value: &[u8]) -> Result<BoundingBox, String> {
    let len = value.len();

    if len != 8 {
        return Err("Error while converting BoundingBox: byte array must be 8 wide".to_string());
    }

    let top = i16::from_le_bytes([value[0], value[1]]);
    let bottom = i16::from_le_bytes([value[2], value[3]]);
    let left = i16::from_le_bytes([value[4], value[5]]);
    let right = i16::from_le_bytes([value[6], value[7]]);

    Ok(BoundingBox::new(
        top.into(),
        bottom.into(),
        left.into(),
        right.into(),
    ))
}
