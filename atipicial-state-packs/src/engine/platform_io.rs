//! Cross-platform positioned-I/O and open-flag shims for pack storage.
//!
//! The pack engine was originally Linux-only (`unix::FileExt`,
//! `libc::O_CLOEXEC`, `mmap`). This module gives both platforms the same
//! surface so the crate builds on Windows as well:
//!
//! - [`read_exact_at`] / [`read_at`]: positioned reads. Unix uses
//!   `FileExt`; Windows uses `seek_read` (`ReadFileScatter`-style positioned
//!   reads, same semantics: file cursor is untouched on Unix, repositioned on
//!   Windows — every caller here is single-threaded per handle or uses the
//!   shared-slice mapping, so the cursor difference is not observable).
//! - [`O_CLOEXEC`]: Unix close-on-exec flag; on Windows handles are not
//!   inherited by default for `std::fs` opens, so it is 0.
//!
//! The mmap wrapper remains Unix-only; on Windows the engine falls back to
//! plain positioned reads through these shims (see `mmap_fallback`).

use std::fs::File;
use std::io::Result;

/// Reads exactly `buf.len()` bytes from `file` starting at `offset`.
pub(crate) fn read_exact_at(file: &File, buf: &mut [u8], offset: u64) -> Result<()> {
    let mut filled = 0usize;
    while filled < buf.len() {
        let read = read_at(file, &mut buf[filled..], offset + filled as u64)?;
        if read == 0 {
            return Err(std::io::Error::new(
                std::io::ErrorKind::UnexpectedEof,
                "positioned read hit end of file",
            ));
        }
        filled += read;
    }
    Ok(())
}

/// Reads available bytes (up to `buf.len()`) from `file` at `offset`,
/// returning how many were read (0 at end of file).
#[cfg(unix)]
pub(crate) fn read_at(file: &File, buf: &mut [u8], offset: u64) -> Result<usize> {
    use std::os::unix::fs::FileExt;
    read_at(file, buf, offset)
}

#[cfg(windows)]
pub(crate) fn read_at(file: &File, buf: &mut [u8], offset: u64) -> Result<usize> {
    use std::os::windows::fs::FileExt;
    file.seek_read(buf, offset)
}

/// `O_CLOEXEC` on Unix; a no-op flag value on Windows.
#[cfg(unix)]
pub(crate) const O_CLOEXEC: i32 = libc::O_CLOEXEC;

#[cfg(windows)]
pub(crate) const O_CLOEXEC: i32 = 0;

/// Writes the whole `buf` to `file` starting at `offset` (test-facing helper;
/// mirrors Unix `FileExt::write_all_at` on both platforms).
#[cfg(unix)]
pub(crate) fn write_all_at(file: &File, buf: &[u8], offset: u64) -> Result<()> {
    use std::os::unix::fs::FileExt;
    file.write_all_at(buf, offset)
}

#[cfg(windows)]
pub(crate) fn write_all_at(file: &File, buf: &[u8], offset: u64) -> Result<()> {
    use std::os::windows::fs::FileExt;
    let mut written = 0usize;
    while written < buf.len() {
        written += file.seek_write(&buf[written..], offset + written as u64)?;
    }
    Ok(())
}
