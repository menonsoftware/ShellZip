//! Filesystem helpers used while packing and extracting.

pub mod metadata;
pub mod motw;
pub mod safe_path;
pub mod walk;

/// Copy `src` to `dst` through `buf`, which is resized to 256 KiB.
pub fn copy_chunked(
    src: &mut dyn std::io::Read,
    dst: &mut dyn std::io::Write,
    buf: &mut Vec<u8>,
) -> std::io::Result<u64> {
    const CHUNK: usize = 256 * 1024;
    if buf.len() != CHUNK {
        buf.resize(CHUNK, 0);
    }
    let mut total = 0u64;
    loop {
        let n = src.read(buf)?;
        if n == 0 {
            break;
        }
        dst.write_all(&buf[..n])?;
        total += n as u64;
    }
    Ok(total)
}
