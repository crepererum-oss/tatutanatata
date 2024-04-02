use anyhow::{Context, Result};

pub(crate) fn decompress_value(v: &[u8]) -> Result<Vec<u8>> {
    if v.is_empty() {
        return Ok(vec![]);
    }

    lz4_flex::block::decompress(v, v.len() * 12).context("decompression")
}
