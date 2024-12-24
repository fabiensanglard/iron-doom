use derive_more::{AsRef, Deref, DerefMut, Index, IndexMut, IntoIterator};

const END_COLUMN_BYTE: u8 = 255;
const POST_HEADER_SIZE: usize = 4;

/// Special picture format used by DOOM to store graphic assets in WAD files.
pub struct WadPatch {
    pub header: WadPatchHeader,
    pub columns: Vec<Column>,
}

pub struct WadPatchHeader {
    /// Width of graphic.
    pub width: u16,

    /// Height of graphic.
    pub height: u16,

    /// Offset in pixels to the left of the origin.
    pub left_off: i16,

    /// Offset in pixels below the origin.
    pub top_off: i16,

    /// Array of column offsets relative to the beginning of the patch header.
    pub columns_off: Vec<u32>,
}

#[derive(Default, AsRef, Deref, DerefMut, IntoIterator, Index, IndexMut)]
#[into_iterator(owned, ref, ref_mut)]
pub struct Column(Vec<Post>);

pub struct Post {
    /// The y offset of this post in this patch. If 255, then end-of-column
    /// (not a valid post).
    pub top_delta: u8,

    /// Array of pixels in this post. Each data pixel is an index into the
    /// Doom palette.
    pub data: Vec<u8>,
}

impl TryFrom<&[u8]> for WadPatch {
    type Error = String;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        let header: WadPatchHeader = value.try_into()?;
        let mut columns: Vec<Column> = vec![];

        for off in &header.columns_off {
            let off = *off as usize;
            let col: Column = value[off..].try_into()?;

            columns.push(col);
        }

        Ok(Self { header, columns })
    }
}

impl TryFrom<&[u8]> for WadPatchHeader {
    type Error = String;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        let len = value.len();

        if len < 8 {
            return Err("A patch header must be at least 8 bytes long".to_owned());
        }

        let width = u16::from_le_bytes([value[0], value[1]]);
        let height = u16::from_le_bytes([value[2], value[3]]);
        let left_off = i16::from_le_bytes([value[4], value[5]]);
        let top_off = i16::from_le_bytes([value[6], value[7]]);

        let size_header = 8 + 4 * width as usize;
        if len < size_header {
            return Err(format!(
                "Error while converting PatchHeader: actual size is \
                 {len} bytes while it should be at least {size_header} bytes!"
            ));
        }

        let columns_off: Vec<u32> = value[8..size_header]
            .chunks(4)
            .map(|columns_off_bytes| {
                u32::from_le_bytes([
                    columns_off_bytes[0],
                    columns_off_bytes[1],
                    columns_off_bytes[2],
                    columns_off_bytes[3],
                ])
            })
            .collect();

        Ok(WadPatchHeader {
            width,
            height,
            left_off,
            top_off,
            columns_off,
        })
    }
}

impl TryFrom<&[u8]> for Column {
    type Error = String;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        let len = value.len();

        if len < 4 {
            let err_msg = "Error while converting Column: \
                A patch header must be at least 4 bytes long!";
            return Err(err_msg.to_string());
        }

        let mut col: Vec<Post> = vec![];
        let mut off = 0usize;
        let len = value.len();

        while off < len {
            let post_bytes = &value[off..];
            let post: Post = post_bytes.try_into()?;

            if post.top_delta == END_COLUMN_BYTE {
                break;
            }

            off += POST_HEADER_SIZE + post.data.len();
            col.push(post);
        }

        Ok(Column(col))
    }
}

impl TryFrom<&[u8]> for Post {
    type Error = String;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        let len = value.len();
        if len == 0 || value[0] == END_COLUMN_BYTE {
            return Ok(Post {
                top_delta: END_COLUMN_BYTE,
                data: vec![],
            });
        }
        if len < POST_HEADER_SIZE {
            return Err(format!(
                "Error while converting Post: A post must be \
                 at least {POST_HEADER_SIZE} bytes long!"
            ));
        }

        let top_delta = value[0];
        let data_length = value[1];

        let size_post = POST_HEADER_SIZE + data_length as usize;
        if size_post > len {
            return Err(format!(
                "Error while converting Post: indicated size is \
                 {size_post} bytes while actual size is {len} bytes!"
            ));
        }

        let data = value[3..(size_post - 1)].to_vec();

        Ok(Post { top_delta, data })
    }
}
