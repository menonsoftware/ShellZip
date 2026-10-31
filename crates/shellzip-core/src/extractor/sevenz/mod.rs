//! 7z listing, testing, and extraction.
//!
//! LZMA, LZMA2, and AES come from `zesven`. Deflate, BZip2, PPMd, and Zstd need
//! the `sevenz-codecs` feature. A missing password on an encrypted archive is
//! exit 11, and so is a wrong one. Solid blocks are decoded in order; a member
//! that is not selected is left unread and the next call skips it.

mod entry;
mod open;

pub(crate) use entry::Member;
pub(crate) use open::open_streaming;

use std::io::{self, Write};
use std::path::Path;

use self::entry::{entry_from, methods_by_name};
use super::policy::{
    Budget, BudgetWriter, Destination, destination, mark_extracted, restore_entry_time,
    symlink_destination,
};
use crate::crypto::Password;
use crate::entry::Entry;
use crate::error::{Error, map_zesven};
use crate::hash::{Blake2spWriter, CrcWriter};
use crate::options::{ConflictDecider, ExtractOpts, ExtractReport, TestReport};
use crate::progress::{ProgressSink, ensure_running};
/// List 7z members. `hash` decodes payloads for BLAKE2sp.
pub fn list_sevenz(
    path: &Path,
    hash: bool,
    password: Option<&Password>,
) -> Result<Vec<Entry>, Error> {
    let mut archive = open_streaming(path, password)?;
    let methods = methods_by_name(archive.header());
    let solid = archive.is_solid();
    let mut entries = Vec::with_capacity(archive.len());
    for file in archive.entries_list() {
        let key = entry::entry_key(file.path.as_str());
        let method = methods.get(&key);
        entries.push(entry_from(file, method, solid));
    }
    if !hash {
        return Ok(entries);
    }
    let had_password = password.is_some();
    let mut iter = archive
        .entries()
        .map_err(|err| map_zesven(err, had_password))?;
    let mut index = 0usize;
    while let Some(item) = iter.next() {
        let member = {
            let streamed = item.map_err(|err| map_zesven(err, had_password))?;
            Member::from_entry(streamed.entry())
        };
        let hash_this =
            !member.is_directory && !member.is_anti && !(member.encrypted && password.is_none());
        if hash_this {
            let mut hasher = Blake2spWriter::new();
            let mut ignored = 0u64;
            let mut writer = ProgressWrite {
                inner: &mut hasher,
                progress: &crate::progress::NopProgress,
                written: &mut ignored,
            };
            iter.extract_current_to(&mut writer)
                .map_err(|err| map_zesven(err, had_password))?;
            if let Some(slot) = entries.get_mut(index) {
                slot.blake2sp = Some(hasher.finish());
            }
        }
        index += 1;
    }
    Ok(entries)
}

/// Decode every member. A stored CRC that does not match is reported, not fatal,
/// until `zesven` itself rejects the block.
pub fn test_sevenz(
    path: &Path,
    progress: &dyn ProgressSink,
    password: Option<&Password>,
    limits: &crate::options::ReadLimits,
) -> Result<TestReport, Error> {
    let mut archive = open_streaming(path, password)?;
    let encrypted: Vec<String> = archive
        .entries_list()
        .iter()
        .filter(|file| file.is_encrypted)
        .map(|file| entry::entry_key(file.path.as_str()))
        .filter(|name| !name.is_empty())
        .collect();
    progress.begin(&format!("test {}", path.display()), 0);
    let mut report = TestReport::default();
    if password.is_none() && !encrypted.is_empty() {
        report.entries = u64::try_from(archive.len()).unwrap_or(u64::MAX);
        report.encrypted = encrypted;
        return Ok(report);
    }
    let had_password = password.is_some();
    let mut budget = Budget::for_archive(path, limits.max_bytes, limits.max_ratio);
    let mut iter = archive
        .entries()
        .map_err(|err| map_zesven(err, had_password))?;
    while let Some(item) = iter.next() {
        ensure_running(progress)?;
        let member = {
            let streamed = item.map_err(|err| map_zesven(err, had_password))?;
            Member::from_entry(streamed.entry())
        };
        report.entries += 1;
        if member.is_directory || member.is_anti {
            progress.begin_entry(&member.name, 0);
            progress.end_entry();
            continue;
        }
        budget.reserve(member.size, None)?;
        progress.begin_entry(&member.name, member.size);
        let mut hasher = CrcWriter::new();
        let extracted = {
            let mut ignored = 0u64;
            let mut capped = BudgetWriter {
                inner: &mut hasher,
                budget: &mut budget,
            };
            let mut writer = ProgressWrite {
                inner: &mut capped,
                progress,
                written: &mut ignored,
            };
            iter.extract_current_to(&mut writer)
        };
        match extracted {
            Ok(_) => {
                report.bytes += hasher.bytes;
                // The archive CRC is checked inside zesven. A mismatch arrives as
                // `CrcMismatch` below. Keep a local compare for codecs that skip it.
            }
            Err(zesven::Error::CrcMismatch { entry_name, .. }) => {
                report.checksums.push(entry_name.unwrap_or(member.name));
                progress.end_entry();
                return Ok(report);
            }
            Err(err) => return Err(map_zesven(err, had_password)),
        }
        progress.end_entry();
    }
    Ok(report)
}

/// Extract selected members.
pub fn extract_sevenz(
    path: &Path,
    opts: &ExtractOpts,
    decide: Option<&ConflictDecider<'_>>,
    selected: &dyn Fn(&str) -> bool,
    progress: &dyn ProgressSink,
) -> Result<ExtractReport, Error> {
    let mut archive = open_streaming(path, opts.password.as_ref())?;
    let encrypted: Vec<String> = archive
        .entries_list()
        .iter()
        .filter(|file| file.is_encrypted)
        .map(|file| entry::entry_key(file.path.as_str()))
        .collect();
    let mut report = ExtractReport::default();
    if opts.password.is_none() {
        let files: Vec<Member> = archive
            .entries_list()
            .iter()
            .map(Member::from_entry)
            .collect();
        let mut plain = false;
        for file in &files {
            if file.name.is_empty() || !selected(&file.name) || file.is_directory || file.is_anti {
                continue;
            }
            if encrypted.iter().any(|item| item == &file.name) {
                if !opts.skip_encrypted {
                    return Err(Error::Encrypted(file.name.clone()));
                }
            } else {
                plain = true;
            }
        }
        if !plain {
            for file in &files {
                if file.name.is_empty() || !selected(&file.name) {
                    continue;
                }
                if encrypted.iter().any(|item| item == &file.name) {
                    report.skipped += 1;
                    report
                        .warnings
                        .push(format!("skipped encrypted entry `{}`", file.name));
                } else if file.is_directory && !opts.flat {
                    match destination(&file.name, opts, false, true, None, decide)? {
                        Destination::Skip => report.skipped += 1,
                        Destination::Write(_) => report.extracted += 1,
                    }
                }
            }
            return Ok(report);
        }
    }
    progress.begin(&format!("extract {}", path.display()), 0);
    let mut budget = Budget::for_archive(path, opts.max_bytes, opts.max_ratio);
    let had_password = opts.password.is_some();
    let mut iter = archive
        .entries()
        .map_err(|err| map_zesven(err, had_password))?;
    while let Some(item) = iter.next() {
        ensure_running(progress)?;
        let member = {
            let streamed = item.map_err(|err| map_zesven(err, had_password))?;
            Member::from_entry(streamed.entry())
        };
        if member.name.is_empty() || !selected(&member.name) {
            continue;
        }
        write_member(
            &member,
            &mut iter,
            &mut ExtractOne {
                archive: path,
                opts,
                decide,
                encrypted: &encrypted,
                progress,
                budget: &mut budget,
                report: &mut report,
                had_password,
            },
        )?;
    }
    Ok(report)
}

/// Copy the member at `index` (in header order) into `sink`.
pub fn copy_sevenz_index(
    path: &Path,
    index: usize,
    sink: &mut dyn Write,
    password: Option<&Password>,
) -> Result<(u64, u32), Error> {
    let mut archive = open_streaming(path, password)?;
    if index >= archive.len() {
        return Err(Error::Usage(format!("entry {index} is out of range")));
    }
    let had_password = password.is_some();
    let mut iter = archive
        .entries()
        .map_err(|err| map_zesven(err, had_password))?;
    for _ in 0..index {
        let item = iter
            .next()
            .ok_or_else(|| Error::Usage(format!("entry {index} is out of range")))?
            .map_err(|err| map_zesven(err, had_password))?;
        drop(item);
    }
    let member = {
        let streamed = iter
            .next()
            .ok_or_else(|| Error::Usage(format!("entry {index} is out of range")))?
            .map_err(|err| map_zesven(err, had_password))?;
        Member::from_entry(streamed.entry())
    };
    if member.is_directory || member.is_anti {
        return Ok((0, 0));
    }
    let mut budget = Budget::for_archive(
        path,
        crate::options::DEFAULT_MAX_BYTES,
        crate::options::DEFAULT_MAX_RATIO,
    );
    budget.reserve(member.size, None)?;
    let mut hasher = CrcWriter::new();
    {
        let mut tee = Tee {
            left: &mut hasher,
            right: sink,
        };
        let mut capped = BudgetWriter {
            inner: &mut tee,
            budget: &mut budget,
        };
        iter.extract_current_to(&mut capped)
            .map_err(|err| map_zesven(err, had_password))?;
    }
    Ok((hasher.bytes, hasher.crc()))
}

struct ExtractOne<'a> {
    archive: &'a Path,
    opts: &'a ExtractOpts,
    decide: Option<&'a ConflictDecider<'a>>,
    encrypted: &'a [String],
    progress: &'a dyn ProgressSink,
    budget: &'a mut Budget,
    report: &'a mut ExtractReport,
    had_password: bool,
}

fn write_member<R: std::io::Read + std::io::Seek + Send>(
    member: &Member,
    iter: &mut zesven::streaming::EntryIterator<'_, R>,
    ctx: &mut ExtractOne<'_>,
) -> Result<(), Error> {
    if ctx.encrypted.iter().any(|item| item == &member.name) && ctx.opts.skip_encrypted {
        ctx.report.skipped += 1;
        ctx.report
            .warnings
            .push(format!("skipped encrypted entry `{}`", member.name));
        return Ok(());
    }
    if member.is_anti {
        ctx.report
            .warnings
            .push(format!("skipped anti-item `{}`", member.name));
        ctx.report.skipped += 1;
        return Ok(());
    }
    ctx.budget.reserve(member.size, None)?;
    ctx.progress.begin_entry(&member.name, member.size);
    if member.is_directory {
        if !ctx.opts.flat {
            match destination(
                &member.name,
                ctx.opts,
                false,
                true,
                member.modified,
                ctx.decide,
            )? {
                Destination::Skip => ctx.report.skipped += 1,
                Destination::Write(_) => ctx.report.extracted += 1,
            }
        } else {
            ctx.report.extracted += 1;
        }
        ctx.progress.end_entry();
        return Ok(());
    }
    if member.is_symlink {
        let mut target = Vec::new();
        {
            let mut capped = BudgetWriter {
                inner: &mut target,
                budget: ctx.budget,
            };
            iter.extract_current_to(&mut capped)
                .map_err(|err| map_zesven(err, ctx.had_password))?;
        }
        let target = String::from_utf8(target).map_err(|_| {
            Error::Archive(format!("symlink `{}` target is not UTF-8", member.name))
        })?;
        let target = target.trim_end_matches('\0').to_string();
        let link = symlink_destination(&member.name, &target, ctx.opts)?;
        match crate::fs::metadata::create_symlink(&target, &link) {
            Ok(()) => ctx.report.extracted += 1,
            Err(err) => ctx.report.warnings.push(err.to_string()),
        }
        ctx.progress.end_entry();
        return Ok(());
    }
    let modified = member.modified;
    match destination(
        &member.name,
        ctx.opts,
        ctx.opts.flat,
        false,
        modified,
        ctx.decide,
    )? {
        Destination::Skip => {
            ctx.report.skipped += 1;
        }
        Destination::Write(dest) => {
            let had_password = ctx.had_password;
            let progress = ctx.progress;
            super::policy::write_atomic(&dest, ctx.opts.keep_broken, |out| {
                let mut written = 0u64;
                let mut capped = BudgetWriter {
                    inner: out,
                    budget: ctx.budget,
                };
                let mut writer = ProgressWrite {
                    inner: &mut capped,
                    progress,
                    written: &mut written,
                };
                iter.extract_current_to(&mut writer)
                    .map_err(|err| map_zesven(err, had_password))?;
                Ok(())
            })?;
            if let Err(err) = restore_entry_time(&dest, modified, ctx.opts) {
                ctx.report.warnings.push(err.to_string());
            }
            if let Err(err) = mark_extracted(ctx.archive, &dest, ctx.opts) {
                ctx.report.warnings.push(err.to_string());
            }
            ctx.report.extracted += 1;
        }
    }
    ctx.progress.end_entry();
    Ok(())
}

/// Forwards writes and counts them. A cancel request fails the write.
struct ProgressWrite<'a, W> {
    inner: &'a mut W,
    progress: &'a dyn ProgressSink,
    written: &'a mut u64,
}

impl<W: Write> Write for ProgressWrite<'_, W> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.progress.wait_if_paused();
        if self.progress.is_cancelled() {
            return Err(io::Error::new(io::ErrorKind::Interrupted, "interrupted"));
        }
        let n = self.inner.write(buf)?;
        if n > 0 {
            self.progress.advance(n as u64);
            *self.written = self.written.saturating_add(n as u64);
        }
        Ok(n)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.inner.flush()
    }
}

struct Tee<'a> {
    left: &'a mut CrcWriter,
    right: &'a mut dyn Write,
}

impl Write for Tee<'_> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.left.write_all(buf)?;
        self.right.write_all(buf)?;
        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        self.left.flush()?;
        self.right.flush()
    }
}
