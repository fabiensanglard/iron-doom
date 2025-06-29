use crate::wall_texture::WallTexture;
use anyhow::Result;
use common::Buffer;

#[derive(Debug)]
pub struct Column {
    pub posts: Vec<Post>,
    length: usize,
}

impl Column {
    pub fn length(&self) -> usize {
        self.length
    }

    pub fn draw_to_texture(&self, tex: &mut WallTexture, tex_col: usize, origin_y: isize) {
        for post in &self.posts {
            // The number of pixels to copy from this post to buffer.
            let mut count = post.length;
            // The position to start copying the pixels.
            let position;

            if let Some(new_pos) = post.top_delta.checked_add_signed(origin_y) {
                position = new_pos;
            } else {
                // Vanilla Bug: the position is incorrectly set to 0, it should
                // be negated. We can't fix it without breaking many PWADs.
                let removed = origin_y.saturating_add_unsigned(post.top_delta);
                count = count.saturating_add_signed(removed);
                position = 0;
            }
            if position >= tex.height() {
                // Patch column is bigger than texture column, so can not draw to it.
                break;
            }
            if position + count > tex.height() {
                // Clip count to avoid going beyond buffer area.
                count = tex.height() - position;
            }

            for i in 0..count {
                let y = position + i;
                tex[(tex_col, y)] = post.data[i];
            }
        }
    }

    pub fn draw_to_buffer(&self, buffer: &mut Buffer, x: usize, y: usize) {
        for post in &self.posts {
            for row in 0..post.length {
                let buffer_y = y + row + post.top_delta;
                buffer[(x, buffer_y)] = post.data[row];
            }
        }
    }
}

#[derive(Debug)]
pub struct Post {
    pub top_delta: usize,
    #[allow(unused)]
    pub length: usize,
    pub data: Vec<u8>,
}

pub struct ColumnParser;

impl ColumnParser {
    pub fn parse(bytes: &[u8]) -> Result<Column> {
        let mut posts = Vec::new();

        let mut col_length = 0;
        let mut post_bytes = bytes;
        loop {
            let top_delta = post_bytes[0];
            if top_delta == 255 {
                break;
            }
            let length = post_bytes[1] as usize;
            let post_data = &post_bytes[3..(3 + length)];

            let post = Post {
                top_delta: top_delta as usize,
                length,
                data: post_data.to_vec(),
            };
            col_length += post.data.len();
            posts.push(post);

            post_bytes = &post_bytes[(length + 4)..];
        }

        Ok(Column {
            posts,
            length: col_length,
        })
    }
}
