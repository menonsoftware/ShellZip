//! Streaming CRC32 used when testing archive entries.

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
