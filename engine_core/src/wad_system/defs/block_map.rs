use crate::fixed_point::Fixed;

/// Data structure used for collision detection.
#[derive(Default)]
pub struct WadBlockMap {
    pub header: WadBlockMapHeader,
    pub offsets: Vec<u16>,
    pub block_list: Vec<u16>,
}

#[derive(Default)]
pub struct WadBlockMapHeader {
    /// x coordinate of grid origin.
    pub x: Fixed,

    /// y coordinate of grid origin.
    pub y: Fixed,

    /// Number of columns.
    pub columns: u16,

    /// Number of rows.
    pub rows: u16,
}

impl TryFrom<&[u8]> for WadBlockMap {
    type Error = String;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        let len = value.len();

        let header: WadBlockMapHeader = value.try_into()?;

        let cols = header.columns as usize;
        let rows = header.rows as usize;
        let size_offsets = 8 + (cols * rows * 2);

        if len <= size_offsets {
            return Err("Block offsets too short".to_string());
        }

        let buf = &value[8..size_offsets];
        let offsets: Vec<u16> = buf
            .chunks_exact(2)
            .map(|off_bytes| u16::from_le_bytes([off_bytes[0], off_bytes[1]]))
            .collect();

        let mut block_list = vec![];

        for off in &offsets {
            let off = 2 * (*off as usize);
            if len <= off {
                return Err("Block list too short".to_string());
            }

            let buf = &value[off..];
            for list_bytes in buf.chunks_exact(2) {
                let list = u16::from_le_bytes([list_bytes[0], list_bytes[1]]);
                if list == u16::MAX {
                    break;
                }
                block_list.push(list);
            }
        }

        Ok(Self {
            header,
            offsets,
            block_list,
        })
    }
}

impl TryFrom<&[u8]> for WadBlockMapHeader {
    type Error = String;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        let len = value.len();
        if len < 8 {
            let err_msg = "Error while converting BlockMapHeader: \
                header must be at least 8 bytes wide";
            return Err(err_msg.to_string());
        }

        let x = i16::from_le_bytes([value[0], value[1]]);
        let y = i16::from_le_bytes([value[2], value[3]]);
        let columns = u16::from_le_bytes([value[4], value[5]]);
        let rows = u16::from_le_bytes([value[6], value[7]]);

        Ok(Self {
            x: x.into(),
            y: y.into(),
            columns,
            rows,
        })
    }
}
