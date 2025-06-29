use crate::patch::column::Column;
use anyhow::{bail, Result};
use column::ColumnParser;
use common::Buffer;
use header::{PatchHeader, PatchHeaderParser};
use std::cmp;

mod column;
mod header;

#[derive(Debug)]
pub struct Patch {
    pub width: usize,
    #[allow(unused)]
    pub height: usize,
    #[allow(unused)]
    pub left_offset: i16,
    #[allow(unused)]
    pub top_offset: i16,
    columns: Vec<Column>,
}

impl Patch {
    pub fn column(&self, col: usize) -> &Column {
        self.columns.get(col).unwrap()
    }
}

pub struct PatchParser;

impl PatchParser {
    pub fn parse(lump_data: &[u8]) -> Result<Patch> {
        let header = PatchHeaderParser::parse(lump_data)?;
        let columns = Self::parse_columns(lump_data, &header)?;
        Ok(Patch {
            width: header.width,
            height: header.height,
            left_offset: header.left_offset,
            top_offset: header.top_offset,
            columns,
        })
    }

    fn parse_columns(lump_data: &[u8], header: &PatchHeader) -> Result<Vec<Column>> {
        let mut columns = Vec::with_capacity(header.width);

        for offset in &header.column_offsets {
            let column_bytes = &lump_data[*offset..];
            let column = ColumnParser::parse(column_bytes)?;
            columns.push(column);
        }

        Ok(columns)
    }
}

pub trait DrawPath {
    fn draw_patch(&mut self, x: usize, y: usize, patch: &Patch) -> Result<()>;
}

impl DrawPath for Buffer {
    fn draw_patch(&mut self, x: usize, y: usize, patch: &Patch) -> Result<()> {
        let left_offset = patch.left_offset as isize;
        let top_offset = patch.top_offset as isize;

        let Some(x) = x.checked_add_signed(-left_offset) else {
            bail!("Bad V_DrawPatch");
        };
        let Some(y) = y.checked_add_signed(-top_offset) else {
            bail!("Bad V_DrawPatch");
        };
        assert!(
            x < self.width()
                && (x + patch.width) <= self.width()
                && (y + patch.height) <= self.height()
                && y < self.height()
        );

        let width = cmp::min(self.width(), patch.width);
        for col in 0..width {
            let column = patch.column(col);
            column.draw_to_buffer(self, x + col, y);
        }

        Ok(())
    }
}
