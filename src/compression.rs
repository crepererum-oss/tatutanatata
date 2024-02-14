use anyhow::{Context, Result};

pub fn decompress_value(v: &[u8]) -> Result<Vec<u8>> {
    lz4_flex::block::decompress(v, v.len() * 6).context("decompression")
}
