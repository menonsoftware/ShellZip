//! Round-trips for the formats Milestone 1 packs and unpacks.
//!
//! Fixtures are generated here. Nothing under `target/` is an input.

use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

use shellzip_core::{
    CompressOpts, Error, ExtractOpts, NopProgress, Overwrite, add, extract, list, test,
};

fn sample_tree(root: &Path) {
    fs::create_dir_all(root.join("sub")).unwrap();
    fs::create_dir_all(root.join("empty-dir")).unwrap();
    fs::write(root.join("sub/hello.txt"), b"hello").unwrap();
    fs::write(root.join("empty.bin"), b"").unwrap();
    fs::write(root.join("文件.txt"), "你好").unwrap();
    fs::write(root.join("photo.jpg"), vec![0xA5; 64]).unwrap();
}

fn opts_fast() -> CompressOpts {
    CompressOpts {
        level: 1,
        ..CompressOpts::default()
    }
}

fn pack_tree(ext: &str) -> (tempfile::TempDir, PathBuf) {
    let tmp = tempfile::tempdir().unwrap();
    let src = tmp.path().join("src");
    fs::create_dir(&src).unwrap();
    sample_tree(&src);
    let archive = tmp.path().join(format!("bundle.{ext}"));
    add(&archive, &[src], &opts_fast(), &NopProgress).unwrap();
    (tmp, archive)
}

fn extract_all(archive: &Path, output: &Path) {
    fs::create_dir_all(output).unwrap();
    let mut opts = ExtractOpts::new(output);
    opts.overwrite = Overwrite::Always;
    extract(archive, &opts, None, &NopProgress).unwrap();
}

#[test]
fn round_trip_zip_tar_and_compressed_tar() {
    for ext in ["zip", "tar", "tar.gz", "tgz", "tar.zst", "tzst"] {
        let (tmp, archive) = pack_tree(ext);
        let report = test(&archive, &NopProgress).unwrap();
        assert_eq!(report.exit_code(), 0, "{ext}");
        let (_opened, entries) = list(&archive).unwrap();
        assert!(
            entries
                .iter()
                .any(|entry| entry.path.ends_with("hello.txt")),
            "{ext} listing missing hello.txt: {entries:?}"
        );
        assert!(
            entries.iter().any(|entry| entry.path.contains("文件.txt")),
            "{ext}"
        );
        let out = tmp.path().join(format!("out-{ext}").replace('.', "_"));
        extract_all(&archive, &out);
        assert_eq!(fs::read(out.join("src/sub/hello.txt")).unwrap(), b"hello");
        assert_eq!(fs::read(out.join("src/empty.bin")).unwrap(), b"");
        assert_eq!(
            fs::read(out.join("src/文件.txt")).unwrap(),
            "你好".as_bytes()
        );
    }
}

#[test]
fn zip_stores_jpeg_and_ep1_drops_the_top_directory() {
    let tmp = tempfile::tempdir().unwrap();
    let src = tmp.path().join("src");
    fs::create_dir(&src).unwrap();
    sample_tree(&src);
    let archive = tmp.path().join("photos.zip");
    let mut opts = opts_fast();
    opts.ep1 = true;
    add(&archive, &[src], &opts, &NopProgress).unwrap();
    let (_opened, entries) = list(&archive).unwrap();
    let photo = entries
        .iter()
        .find(|entry| entry.path.ends_with("photo.jpg"))
        .expect("photo");
    assert_eq!(photo.method.as_deref(), Some("store"));
    assert!(
        !photo.path.contains("src/"),
        "ep1 kept the top directory: {}",
        photo.path
    );
    let out = tmp.path().join("out");
    extract_all(&archive, &out);
    assert_eq!(fs::read(out.join("sub/hello.txt")).unwrap(), b"hello");
}

#[test]
fn single_file_gzip_and_zstd_round_trip() {
    let tmp = tempfile::tempdir().unwrap();
    let src = tmp.path().join("notes.txt");
    fs::write(&src, b"plain").unwrap();
    for ext in ["gz", "zst"] {
        let archive = tmp.path().join(format!("notes.txt.{ext}"));
        add(
            &archive,
            std::slice::from_ref(&src),
            &opts_fast(),
            &NopProgress,
        )
        .unwrap();
        test(&archive, &NopProgress).unwrap();
        let out = tmp.path().join(format!("out-{ext}"));
        extract_all(&archive, &out);
        assert_eq!(fs::read(out.join("notes.txt")).unwrap(), b"plain");
    }
}

#[test]
fn gzip_rejects_a_second_file() {
    let tmp = tempfile::tempdir().unwrap();
    let a = tmp.path().join("a.txt");
    let b = tmp.path().join("b.txt");
    fs::write(&a, b"a").unwrap();
    fs::write(&b, b"b").unwrap();
    let err = add(
        &tmp.path().join("both.gz"),
        &[a, b],
        &opts_fast(),
        &NopProgress,
    )
    .unwrap_err();
    assert!(matches!(err, Error::Usage(_)));
}

#[test]
fn zip_slip_names_are_rejected() {
    for name in ["../evil.txt", "C:/evil.txt", r"C:\evil.txt"] {
        let tmp = tempfile::tempdir().unwrap();
        let archive = tmp.path().join("slip.zip");
        write_stored_zip(&archive, name, b"pwned");
        let out = tmp.path().join("out");
        fs::create_dir(&out).unwrap();
        let mut opts = ExtractOpts::new(&out);
        opts.overwrite = Overwrite::Always;
        let err = extract(&archive, &opts, None, &NopProgress).unwrap_err();
        assert!(matches!(err, Error::UnsafePath(_)), "{name}: {err}");
        assert!(!tmp.path().join("evil.txt").exists(), "{name}");
        assert!(!out.join("evil.txt").exists(), "{name}");
    }
}

#[test]
fn overwrite_skip_and_rename() {
    let tmp = tempfile::tempdir().unwrap();
    let src = tmp.path().join("src");
    fs::create_dir(&src).unwrap();
    fs::write(src.join("hello.txt"), b"new").unwrap();
    let archive = tmp.path().join("one.zip");
    add(&archive, &[src], &opts_fast(), &NopProgress).unwrap();
    let out = tmp.path().join("out");
    fs::create_dir_all(out.join("src")).unwrap();
    fs::write(out.join("src/hello.txt"), b"old").unwrap();

    let mut skip = ExtractOpts::new(&out);
    skip.overwrite = Overwrite::Never;
    let skipped = extract(&archive, &skip, None, &NopProgress).unwrap();
    assert!(skipped.skipped >= 1);
    assert_eq!(fs::read(out.join("src/hello.txt")).unwrap(), b"old");

    let mut rename = ExtractOpts::new(&out);
    rename.overwrite = Overwrite::Rename;
    extract(&archive, &rename, None, &NopProgress).unwrap();
    assert_eq!(fs::read(out.join("src/hello (1).txt")).unwrap(), b"new");
    assert_eq!(fs::read(out.join("src/hello.txt")).unwrap(), b"old");
}

#[test]
fn byte_cap_stops_extraction() {
    let tmp = tempfile::tempdir().unwrap();
    let src = tmp.path().join("big.txt");
    fs::write(&src, vec![1; 32]).unwrap();
    let archive = tmp.path().join("big.zip");
    add(&archive, &[src], &opts_fast(), &NopProgress).unwrap();
    let mut opts = ExtractOpts::new(tmp.path().join("out"));
    opts.max_bytes = 4;
    opts.overwrite = Overwrite::Always;
    let err = extract(&archive, &opts, None, &NopProgress).unwrap_err();
    assert!(matches!(err, Error::Usage(_)));
}

#[test]
fn adding_to_an_existing_zip_keeps_the_other_files() {
    let tmp = tempfile::tempdir().unwrap();
    let keep = tmp.path().join("keep.txt");
    let replace = tmp.path().join("replace.txt");
    fs::write(&keep, b"keep").unwrap();
    fs::write(&replace, b"old").unwrap();
    let archive = tmp.path().join("bundle.zip");
    add(
        &archive,
        &[keep, replace.clone()],
        &opts_fast(),
        &NopProgress,
    )
    .unwrap();

    fs::write(&replace, b"new").unwrap();
    let extra = tmp.path().join("extra.txt");
    fs::write(&extra, b"extra").unwrap();
    add(&archive, &[replace, extra], &opts_fast(), &NopProgress).unwrap();

    let out = tmp.path().join("out");
    extract_all(&archive, &out);
    assert_eq!(fs::read(out.join("keep.txt")).unwrap(), b"keep");
    assert_eq!(fs::read(out.join("replace.txt")).unwrap(), b"new");
    assert_eq!(fs::read(out.join("extra.txt")).unwrap(), b"extra");

    let only = tmp.path().join("only.txt");
    fs::write(&only, b"only").unwrap();
    let mut opts = opts_fast();
    opts.force = true;
    add(&archive, &[only], &opts, &NopProgress).unwrap();
    let (_opened, entries) = list(&archive).unwrap();
    assert!(entries.iter().any(|entry| entry.path.ends_with("only.txt")));
    assert!(!entries.iter().any(|entry| entry.path.ends_with("keep.txt")));
}

#[test]
fn existing_tar_is_left_in_place_without_force() {
    let (tmp, archive) = pack_tree("tar");
    let src = tmp.path().join("src");
    let err = add(&archive, &[src], &opts_fast(), &NopProgress).unwrap_err();
    assert!(err.to_string().contains("already exists"), "{err}");
}

#[test]
fn rar_signature_is_recognized_and_refused() {
    let tmp = tempfile::tempdir().unwrap();
    let path = tmp.path().join("sample.rar");
    fs::write(&path, b"Rar!\x1a\x07\x01\x00rest").unwrap();
    let err = list(&path).unwrap_err();
    assert!(matches!(err, Error::Unsupported { .. }));
}

#[test]
#[ignore = "writes and packs about 1 GiB"]
fn streams_a_large_file() {
    let tmp = tempfile::tempdir().unwrap();
    let big = tmp.path().join("big.bin");
    let mut file = fs::File::create(&big).unwrap();
    let chunk = vec![0xA5u8; 1024 * 1024];
    for _ in 0..1024 {
        file.write_all(&chunk).unwrap();
    }
    drop(file);
    let archive = tmp.path().join("big.zip");
    let mut opts = opts_fast();
    opts.level = 0;
    add(&archive, &[big], &opts, &NopProgress).unwrap();
    let out = tmp.path().join("out");
    extract_all(&archive, &out);
    assert_eq!(
        fs::metadata(out.join("big.bin")).unwrap().len(),
        1024 * 1024 * 1024
    );
}

/// Stored ZIP with a caller-chosen name. The name is not sanitized on write.
fn write_stored_zip(path: &Path, name: &str, payload: &[u8]) {
    let file = fs::File::create(path).unwrap();
    let mut writer = zip::ZipWriter::new(file);
    writer
        .start_file(name, zip::write::SimpleFileOptions::default())
        .unwrap();
    writer.write_all(payload).unwrap();
    writer.finish().unwrap();
}
