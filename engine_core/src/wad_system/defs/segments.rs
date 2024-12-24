use derive_more::{AsRef, Deref, DerefMut, Index, IndexMut, IntoIterator};

const WAD_SEGMENT_BYTE_SIZE: usize = 12;

#[derive(AsRef, Deref, DerefMut, IntoIterator, Index, IndexMut)]
#[into_iterator(owned, ref, ref_mut)]
pub struct WadSegments(Vec<WadSegment>);

/// Segment of linedefs, describing the portion of a linedef that borders the
/// subsector that the segment belongs to.
#[derive(Copy, Clone)]
pub struct WadSegment {
    /// Starting vertex number.
    pub start_vertex: u16,

    /// Ending vertex number.
    pub end_vertex: u16,

    /// Angle, full circle is -32768 to 32767.
    pub angle: i16,

    /// Linedef number.
    pub line_def: u16,

    /// Direction of segment.
    pub direction: Direction,

    /// Distance along linedef to start of seg.
    pub offset: i16,
}

/// Direction of segment.
#[derive(Copy, Clone)]
pub enum Direction {
    /// Same as linedef.
    Front,

    /// Opposite of linedef.
    Back,
}

impl TryFrom<&[u8]> for WadSegments {
    type Error = String;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        let len = value.len();

        if len % WAD_SEGMENT_BYTE_SIZE != 0 {
            let err_msg = format!(
                "Error while converting WadSegments: \
                 byte array must be divisible by {WAD_SEGMENT_BYTE_SIZE}!"
            );
            return Err(err_msg);
        }

        let segments: Result<Vec<WadSegment>, String> = value
            .chunks_exact(WAD_SEGMENT_BYTE_SIZE)
            .map(|segment_bytes| segment_bytes.try_into())
            .collect();
        let segments = segments?;

        Ok(Self(segments))
    }
}

impl TryFrom<&[u8]> for WadSegment {
    type Error = String;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        let len = value.len();

        if len != WAD_SEGMENT_BYTE_SIZE {
            let err_msg = format!(
                "Error while converting WadSegment: \
                 byte array must be {WAD_SEGMENT_BYTE_SIZE} bytes wide!"
            );
            return Err(err_msg);
        }

        let direction = u16::from_le_bytes([value[8], value[9]]);
        let direction = if direction == 0 {
            Direction::Front
        } else {
            Direction::Back
        };

        Ok(Self {
            start_vertex: u16::from_le_bytes([value[0], value[1]]),
            end_vertex: u16::from_le_bytes([value[2], value[3]]),
            angle: i16::from_le_bytes([value[4], value[5]]),
            line_def: u16::from_le_bytes([value[6], value[7]]),
            direction,
            offset: i16::from_le_bytes([value[10], value[11]]),
        })
    }
}
