//! Parallel ZIP, update/freshen/delete, stdin streaming, and technical hashes.

use std::fs;
use std::io::Cursor;
use std::path::Path;
use std::time::{Duration, UNIX_EPOCH};

use shellzip_core::{
    CompressOpts, ExtractOpts, ListOpts, NopProgress, Overwrite, UpdateMode, add, add_stdin,
    delete, extract, list, list_with, test, write_member,
};

fn opts(threads: u16) -> CompressOpts {
    CompressOpts {
        level: 1,
        threads,
        ..CompressOpts::default()
    }
}

fn touch(path: &Path, secs: u64) {
    let file = fs::OpenOptions::new().write(true).open(path).unwrap();
    file.set_modified(UNIX_EPOCH + Duration::from_secs(secs))
        .unwrap();
}

#[test]
fn parallel_zip_matches_single_thread_bytes() {
    let tmp = tempfile::tempdir().unwrap();
    let src = tmp.path().join("src");
    fs::create_dir(&src).unwrap();
    fs::write(src.join("a.txt"), b"alpha-alpha-alpha").unwrap();
    fs::write(src.join("b.txt"), vec![0x11; 64 * 1024]).unwrap();
    fs::write(src.join("empty.bin"), b"").unwrap();

    let single = tmp.path().join("one.zip");
    let multi = tmp.path().join("many.zip");
    add(&single, std::slice::from_ref(&src), &opts(1), &NopProgress).unwrap();
    add(&multi, std::slice::from_ref(&src), &opts(2), &NopProgress).unwrap();

    let (_o, left) = list(&single).unwrap();
    let (_o, right) = list(&multi).unwrap();
    let mut left_files: Vec<_> = left
        .iter()
        .filter(|entry| entry.kind == shellzip_core::EntryKind::File)
        .map(|entry| {
            (
                entry.path.clone(),
                entry.crc32,
                entry.packed_size,
                entry.size,
            )
        })
        .collect();
    let mut right_files: Vec<_> = right
        .iter()
        .filter(|entry| entry.kind == shellzip_core::EntryKind::File)
        .map(|entry| {
            (
                entry.path.clone(),
                entry.crc32,
                entry.packed_size,
                entry.size,
            )
        })
        .collect();
    left_files.sort();
    right_files.sort();
    assert_eq!(left_files, right_files, "packed bytes diverged");

    for (archive, label) in [(&single, "1"), (&multi, "2")] {
        let out = tmp.path().join(label);
        fs::create_dir(&out).unwrap();
        let mut extract_opts = ExtractOpts::new(&out);
        extract_opts.overwrite = Overwrite::Always;
        extract(archive, &extract_opts, None, &NopProgress).unwrap();
        assert_eq!(
            fs::read(out.join("src/a.txt")).unwrap(),
            b"alpha-alpha-alpha"
        );
        assert_eq!(fs::read(out.join("src/empty.bin")).unwrap(), b"");
    }
    test(&multi, &NopProgress).unwrap();
}

#[test]
fn update_freshen_and_delete_rewrite_zip() {
    let tmp = tempfile::tempdir().unwrap();
    let old = tmp.path().join("a.txt");
    let extra = tmp.path().join("b.txt");
    fs::write(&old, b"old").unwrap();
    touch(&old, 1_700_000_000);
    let archive = tmp.path().join("notes.zip");
    add(&archive, std::slice::from_ref(&old), &opts(1), &NopProgress).unwrap();

    fs::write(&old, b"stale").unwrap();
    touch(&old, 1_600_000_000);
    let mut update = opts(1);
    update.update = UpdateMode::Update;
    add(&archive, std::slice::from_ref(&old), &update, &NopProgress).unwrap();
    let mut sink = Vec::new();
    write_member(&archive, Some("a.txt"), &mut sink).unwrap();
    assert_eq!(sink, b"old", "older source must not replace the member");

    fs::write(&old, b"fresh").unwrap();
    touch(&old, 1_800_000_000);
    fs::write(&extra, b"new").unwrap();
    touch(&extra, 1_800_000_000);
    let mut freshen = opts(1);
    freshen.update = UpdateMode::Freshen;
    add(
        &archive,
        &[old.clone(), extra.clone()],
        &freshen,
        &NopProgress,
    )
    .unwrap();
    let (_o, entries) = list(&archive).unwrap();
    assert!(entries.iter().all(|entry| entry.path != "b.txt"));
    sink.clear();
    write_member(&archive, Some("a.txt"), &mut sink).unwrap();
    assert_eq!(sink, b"fresh");

    update.update = UpdateMode::Update;
    add(
        &archive,
        std::slice::from_ref(&extra),
        &update,
        &NopProgress,
    )
    .unwrap();
    let (_o, entries) = list(&archive).unwrap();
    assert!(entries.iter().any(|entry| entry.path == "b.txt"));

    delete(&archive, &["b.txt".to_string()], &NopProgress).unwrap();
    let (_o, entries) = list(&archive).unwrap();
    assert!(entries.iter().all(|entry| entry.path != "b.txt"));
    assert!(entries.iter().any(|entry| entry.path == "a.txt"));
}

#[test]
fn stdin_streams_zst_and_spools_tar() {
    let tmp = tempfile::tempdir().unwrap();
    let zst = tmp.path().join("in.zst");
    let mut input = Cursor::new(b"from-stdin".to_vec());
    add_stdin(&zst, "payload.bin", &mut input, &opts(1), &NopProgress).unwrap();
    let mut out = Vec::new();
    write_member(&zst, None, &mut out).unwrap();
    assert_eq!(out, b"from-stdin");

    let tar = tmp.path().join("in.tar.zst");
    let mut input = Cursor::new(b"tar-body".to_vec());
    add_stdin(&tar, "note.txt", &mut input, &opts(1), &NopProgress).unwrap();
    out.clear();
    write_member(&tar, Some("note.txt"), &mut out).unwrap();
    assert_eq!(out, b"tar-body");
}

#[test]
fn technical_list_fills_blake2sp() {
    let tmp = tempfile::tempdir().unwrap();
    let file = tmp.path().join("a.txt");
    fs::write(&file, b"foo").unwrap();
    let archive = tmp.path().join("a.zip");
    add(&archive, &[file], &opts(1), &NopProgress).unwrap();
    let (_o, entries) = list_with(&archive, &ListOpts { blake2sp: true }).unwrap();
    let file = entries.iter().find(|entry| entry.path == "a.txt").unwrap();
    let hex: String = file
        .blake2sp
        .unwrap()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect();
    assert_eq!(
        hex,
        "050dc5786037ea72cb9ed9d0324afcab03c97ec02e8c47368fc5dfb4cf49d8c9"
    );
    assert!(file.crc32.is_some());
    assert_eq!(file.host_os.as_deref(), Some("Unix"));
}

#[test]
fn freshen_missing_archive_is_not_found() {
    let tmp = tempfile::tempdir().unwrap();
    let file = tmp.path().join("a.txt");
    fs::write(&file, b"a").unwrap();
    let archive = tmp.path().join("missing.zip");
    let mut freshen = opts(1);
    freshen.update = UpdateMode::Freshen;
    let err = add(&archive, &[file], &freshen, &NopProgress).unwrap_err();
    assert_eq!(err.exit_code(), 6, "{err}");
}
