use anyhow::{Context, Result};

pub(crate) fn decompress_value(v: &[u8]) -> Result<Vec<u8>> {
    match lz4_flex::block::decompress(v, v.len() * 6) {
        Ok(result) => Ok(result),
        Err(e) => {
            // If it failed and the error suggests buffer too small, retry with 10x
            if e.to_string().contains("too small") {
                lz4_flex::block::decompress(v, v.len() * 10).context("decompression retry")
            } else {
                Err(e).context("decompression")
            }
        }
    }
}
