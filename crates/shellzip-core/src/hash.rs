//! Streaming CRC32 and BLAKE2sp used when testing and listing archive entries.

use std::io::{self, Write};

use crc32fast::Hasher;

/// Hashes bytes as they are written. The buffer is never retained.
#[derive(Debug, Default)]
pub struct CrcWriter {
    hasher: Hasher,
    /// Total bytes hashed.
    pub bytes: u64,
}

impl CrcWriter {
    /// Empty hasher.
    pub fn new() -> Self {
        Self::default()
    }

    /// Current CRC32 of everything written so far.
    pub fn crc(&self) -> u32 {
        self.hasher.clone().finalize()
    }
}

impl Write for CrcWriter {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.hasher.update(buf);
        self.bytes += buf.len() as u64;
        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

/// Incremental BLAKE2sp-256. Bytes are hashed as they are written.
#[derive(Clone)]
pub struct Blake2spWriter {
    state: blake2s_simd::blake2sp::State,
    /// Total bytes hashed.
    pub bytes: u64,
}

impl Blake2spWriter {
    /// Empty BLAKE2sp-256 hasher.
    pub fn new() -> Self {
        Self {
            state: blake2s_simd::blake2sp::State::new(),
            bytes: 0,
        }
    }

    /// Finish and return the 32-byte digest.
    pub fn finish(&self) -> [u8; 32] {
        *self.state.finalize().as_array()
    }
}

impl Default for Blake2spWriter {
    fn default() -> Self {
        Self::new()
    }
}

impl Write for Blake2spWriter {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.state.update(buf);
        self.bytes += buf.len() as u64;
        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn blake2sp_matches_the_reference_vector() {
        let mut hasher = Blake2spWriter::new();
        hasher.write_all(b"foo").unwrap();
        let hex: String = hasher
            .finish()
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect();
        assert_eq!(
            hex,
            "050dc5786037ea72cb9ed9d0324afcab03c97ec02e8c47368fc5dfb4cf49d8c9"
        );
    }
}
