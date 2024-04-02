use anyhow::{Context, Result};

pub(crate) fn decompress_value(v: &[u8]) -> Result<Vec<u8>> {
    if v.is_empty() {
        return Ok(vec![]);
    }

    lz4_flex::block::decompress(v, v.len() * 12).context("decompression")
}

#[cfg(test)]
mod tests {
    use super::decompress_value;

    #[test]
    fn test_decompress_value() {
        assert_compress_value_roundtrip(b"test");
        assert_compress_value_roundtrip(b"");
        assert_compress_value_roundtrip(&[0xff; 1024]);

        assert_eq!(decompress_value(b"").unwrap(), b"");
        assert_eq!(
            decompress_value(b"\xff").unwrap_err().to_string(),
            "decompression",
        );
    }

    #[track_caller]
    fn assert_compress_value_roundtrip(v: &[u8]) {
        let compressed = compress_value(v);
        let decompressed = decompress_value(&compressed).unwrap();
        assert_eq!(&decompressed, v);
    }

    fn compress_value(v: &[u8]) -> Vec<u8> {
        lz4_flex::block::compress(v)
    }
}
