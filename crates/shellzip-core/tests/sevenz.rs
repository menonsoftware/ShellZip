//! 7z reading through `zesven`.
//!
//! Built only with `--features sevenz` (see `required-features` on this target).
//! BZip2 and Deflate need `sevenz-codecs` as well.

use std::fs;
use std::path::Path;

use shellzip_core::{
    CompressOpts, ExtractOpts, ListOpts, NopProgress, Overwrite, Password, SevenZOpts, add,
    extract, list, list_with,
};
use zesven::codec::CodecMethod;
use zesven::write::WriteOptions;
use zesven::{ArchivePath, Password as SevenPassword, Writer};

const FIXTURE_PASSWORD: &str = "szip-fixture";

fn password() -> Password {
    Password::new(FIXTURE_PASSWORD).unwrap()
}

fn extract_bytes(archive: &Path, password: Option<Password>) -> tempfile::TempDir {
    let out = tempfile::tempdir().unwrap();
    let mut opts = ExtractOpts::new(out.path());
    opts.overwrite = Overwrite::Always;
    opts.password = password;
    extract(archive, &opts, None, &NopProgress).unwrap();
    out
}

fn write_file(dest: &Path, src: &Path, method: CodecMethod, header: bool, password: Option<&str>) {
    let mut options = WriteOptions::new().method(method).level_clamped(1);
    if let Some(password) = password {
        options = options
            .password(SevenPassword::new(password))
            .encrypt_data(true)
            .encrypt_header(header);
    }
    let mut writer = Writer::create_path(dest).unwrap().options(options);
    writer
        .add_path(src, ArchivePath::new("hello.txt").unwrap())
        .unwrap();
    assert!(writer.finish().unwrap().entries_written >= 1);
}

#[test]
fn add_lists_and_extracts_through_the_core() {
    let tmp = tempfile::tempdir().unwrap();
    let src = tmp.path().join("notes.txt");
    fs::write(&src, b"hello\n").unwrap();
    let dest = tmp.path().join("out.7z");
    let opts = CompressOpts {
        level: 1,
        ..CompressOpts::default()
    };
    add(&dest, &[src], &opts, &NopProgress).unwrap();
    let (_opened, entries) = list(&dest).unwrap();
    assert_eq!(
        entries
            .iter()
            .filter(|entry| entry.kind == shellzip_core::EntryKind::File)
            .count(),
        1
    );
    assert_eq!(entries[0].method.as_deref(), Some("LZMA2"));
    let out = extract_bytes(&dest, None);
    assert_eq!(fs::read(out.path().join("notes.txt")).unwrap(), b"hello\n");
}

#[test]
fn update_and_delete_rewrite_kept_members() {
    let tmp = tempfile::tempdir().unwrap();
    let first = tmp.path().join("keep.txt");
    let second = tmp.path().join("drop.txt");
    fs::write(&first, b"keep").unwrap();
    fs::write(&second, b"drop").unwrap();
    let dest = tmp.path().join("notes.7z");
    add(
        &dest,
        &[first.clone(), second],
        &CompressOpts::default(),
        &NopProgress,
    )
    .unwrap();
    fs::write(&first, b"kept-new").unwrap();
    add(&dest, &[first], &CompressOpts::default(), &NopProgress).unwrap();
    let out = extract_bytes(&dest, None);
    assert_eq!(fs::read(out.path().join("keep.txt")).unwrap(), b"kept-new");
    assert_eq!(fs::read(out.path().join("drop.txt")).unwrap(), b"drop");
    shellzip_core::delete(&dest, &["drop.txt".to_string()], None, &NopProgress).unwrap();
    let out = extract_bytes(&dest, None);
    assert!(!out.path().join("drop.txt").exists());
    assert_eq!(fs::read(out.path().join("keep.txt")).unwrap(), b"kept-new");
}

#[test]
fn updating_an_encrypted_7z_keeps_earlier_members() {
    let tmp = tempfile::tempdir().unwrap();
    let keep = tmp.path().join("keep.txt");
    fs::write(&keep, b"keep").unwrap();
    let dest = tmp.path().join("notes.7z");
    let opts = CompressOpts {
        level: 1,
        password: Some(password()),
        ..CompressOpts::default()
    };
    add(&dest, std::slice::from_ref(&keep), &opts, &NopProgress).unwrap();
    let extra = tmp.path().join("extra.txt");
    fs::write(&extra, b"extra").unwrap();
    add(&dest, std::slice::from_ref(&extra), &opts, &NopProgress).unwrap();
    let out = extract_bytes(&dest, Some(password()));
    assert_eq!(fs::read(out.path().join("keep.txt")).unwrap(), b"keep");
    assert_eq!(fs::read(out.path().join("extra.txt")).unwrap(), b"extra");
}

#[test]
fn lzma2_lists_and_extracts() {
    let tmp = tempfile::tempdir().unwrap();
    let src = tmp.path().join("hello.txt");
    let other = tmp.path().join("other.txt");
    fs::write(&src, b"hello\n").unwrap();
    fs::write(&other, b"other\n").unwrap();
    let plain = tmp.path().join("plain.7z");
    let mut writer = Writer::create_path(&plain).unwrap().options(
        WriteOptions::new()
            .method(CodecMethod::Lzma2)
            .level_clamped(1),
    );
    writer
        .add_path(&src, ArchivePath::new("hello.txt").unwrap())
        .unwrap();
    writer
        .add_path(&other, ArchivePath::new("other.txt").unwrap())
        .unwrap();
    let written = writer.finish().unwrap();
    assert!(written.entries_written >= 2, "{written:?}");

    let (_opened, entries) = list(&plain).unwrap();
    let hello = entries
        .iter()
        .find(|entry| entry.path == "hello.txt")
        .expect("hello.txt");
    assert_eq!(hello.method.as_deref(), Some("LZMA2"));
    assert!(!hello.encrypted);
    let out = extract_bytes(&plain, None);
    assert_eq!(fs::read(out.path().join("hello.txt")).unwrap(), b"hello\n");
    assert_eq!(fs::read(out.path().join("other.txt")).unwrap(), b"other\n");
}

#[test]
fn aes_hides_bytes_and_header_encryption_hides_names() {
    let tmp = tempfile::tempdir().unwrap();
    let src = tmp.path().join("hello.txt");
    fs::write(&src, b"hello\n").unwrap();

    let visible = tmp.path().join("aes.7z");
    write_file(
        &visible,
        &src,
        CodecMethod::Lzma2,
        false,
        Some(FIXTURE_PASSWORD),
    );
    let (_opened, entries) = list(&visible).unwrap();
    assert!(
        entries
            .iter()
            .any(|entry| entry.path == "hello.txt" && entry.encrypted),
        "{entries:?}"
    );
    let missing = extract(
        &visible,
        &ExtractOpts::new(tmp.path().join("denied")),
        None,
        &NopProgress,
    )
    .unwrap_err();
    assert_eq!(missing.exit_code(), 11, "{missing}");
    let out = extract_bytes(&visible, Some(password()));
    assert_eq!(fs::read(out.path().join("hello.txt")).unwrap(), b"hello\n");

    let hidden = tmp.path().join("headers.7z");
    write_file(
        &hidden,
        &src,
        CodecMethod::Lzma2,
        true,
        Some(FIXTURE_PASSWORD),
    );
    let missing = list(&hidden).unwrap_err();
    assert_eq!(missing.exit_code(), 11, "{missing}");
    let wrong = list_with(
        &hidden,
        &ListOpts {
            password: Some(Password::new("wrong-fixture").unwrap()),
            ..ListOpts::default()
        },
    )
    .unwrap_err();
    assert_eq!(wrong.exit_code(), 11, "{wrong}");
    let out = extract_bytes(&hidden, Some(password()));
    assert_eq!(fs::read(out.path().join("hello.txt")).unwrap(), b"hello\n");
}

/// Each archive draws a fresh nonce. A missing or wrong password must stay
/// exit 11 for every one of them, not a decoder error on some ciphertexts.
#[test]
fn header_encryption_is_a_password_error_for_every_nonce() {
    let tmp = tempfile::tempdir().unwrap();
    let src = tmp.path().join("hello.txt");
    fs::write(&src, b"hello\n").unwrap();
    let opts = CompressOpts {
        level: 1,
        threads: 1,
        password: Some(password()),
        sevenz: SevenZOpts {
            encrypt_headers: true,
            ..SevenZOpts::default()
        },
        ..CompressOpts::default()
    };
    for n in 0..12 {
        let dest = tmp.path().join(format!("secret-{n}.7z"));
        add(&dest, std::slice::from_ref(&src), &opts, &NopProgress).unwrap();
        let missing = list(&dest).unwrap_err();
        assert_eq!(missing.exit_code(), 11, "archive {n}: {missing}");
        let wrong = list_with(
            &dest,
            &ListOpts {
                password: Some(Password::new("wrong-fixture").unwrap()),
                ..ListOpts::default()
            },
        )
        .unwrap_err();
        assert_eq!(wrong.exit_code(), 11, "archive {n} wrong password: {wrong}");
    }
}

#[test]
fn committed_sevenz_fixtures_list_when_present() {
    let dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/sevenz");
    let solid = dir.join("solid.7z");
    if !solid.is_file() {
        return;
    }
    let (_opened, entries) = list(&solid).unwrap();
    assert!(
        entries.iter().any(|entry| entry.path.ends_with("a.txt")),
        "{entries:?}"
    );
    let out = extract_bytes(&solid, None);
    assert!(out.path().join("a.txt").is_file() || out.path().join("b.txt").is_file());
}

#[test]
fn seven_zip_reads_archives_we_write_when_installed() {
    let tool = ["7z", "7z.exe"].into_iter().find_map(|name| {
        std::env::var_os("PATH").and_then(|path| {
            std::env::split_paths(&path).find_map(|dir| {
                let candidate = dir.join(name);
                candidate.is_file().then_some(candidate)
            })
        })
    });
    let Some(tool) = tool else {
        return;
    };
    let tmp = tempfile::tempdir().unwrap();
    let src = tmp.path().join("hello.txt");
    fs::write(&src, b"hello\n").unwrap();
    let archive = tmp.path().join("out.7z");
    add(
        &archive,
        &[src],
        &CompressOpts {
            level: 1,
            threads: 1,
            ..CompressOpts::default()
        },
        &NopProgress,
    )
    .unwrap();
    let status = std::process::Command::new(tool)
        .args(["t", archive.to_str().unwrap()])
        .status()
        .unwrap();
    assert!(status.success(), "7z t failed: {status}");
}

#[cfg(feature = "sevenz-codecs")]
#[test]
fn bzip2_and_deflate_round_trip() {
    let tmp = tempfile::tempdir().unwrap();
    let src = tmp.path().join("hello.txt");
    fs::write(&src, b"hello\n").unwrap();
    for (name, method, label) in [
        ("bzip2.7z", CodecMethod::BZip2, "BZip2"),
        ("deflate.7z", CodecMethod::Deflate, "Deflate"),
    ] {
        let archive = tmp.path().join(name);
        write_file(&archive, &src, method, false, None);
        let (_opened, entries) = list(&archive).unwrap();
        assert_eq!(entries[0].method.as_deref(), Some(label), "{name}");
        let out = extract_bytes(&archive, None);
        assert_eq!(fs::read(out.path().join("hello.txt")).unwrap(), b"hello\n");
    }
}

fn fast() -> CompressOpts {
    CompressOpts {
        level: 1,
        threads: 1,
        ..CompressOpts::default()
    }
}

/// Directories whose names sort after a file in a sibling folder, plus mixed case.
///
/// `VMWARE/Worker1` is stored before `VMWARE/CP1/vmware-0.log`, and `a.txt`
/// before `B.txt`. Byte order is the other way around, which is what zesven's
/// deterministic check used to reject.
fn vmware_tree(parent: &Path) -> std::path::PathBuf {
    let root = parent.join("VMWARE");
    fs::create_dir_all(root.join("Worker1")).unwrap();
    fs::create_dir_all(root.join("CP1")).unwrap();
    fs::create_dir_all(root.join("empty")).unwrap();
    fs::write(root.join("Worker1").join("x.txt"), b"worker").unwrap();
    fs::write(root.join("CP1").join("vmware-0.log"), b"log-bytes").unwrap();
    fs::write(root.join("B.txt"), b"upper").unwrap();
    fs::write(root.join("a.txt"), b"lower").unwrap();
    fs::write(root.join("文档.txt"), "你好").unwrap();
    root
}

#[test]
fn nested_folder_adds_in_planner_order() {
    let tmp = tempfile::tempdir().unwrap();
    let root = vmware_tree(tmp.path());
    let dest = tmp.path().join("vmware.7z");
    add(&dest, &[root], &fast(), &NopProgress).unwrap();
    let (_opened, entries) = list(&dest).unwrap();
    let paths: Vec<&str> = entries.iter().map(|entry| entry.path.as_str()).collect();
    assert!(
        paths.iter().any(|path| path.ends_with("vmware-0.log")),
        "{paths:?}"
    );
    assert!(
        paths.iter().any(|path| path.ends_with("文档.txt")),
        "{paths:?}"
    );
    let out = extract_bytes(&dest, None);
    assert_eq!(
        fs::read(out.path().join("VMWARE").join("Worker1").join("x.txt")).unwrap(),
        b"worker"
    );
    assert_eq!(
        fs::read(out.path().join("VMWARE").join("CP1").join("vmware-0.log")).unwrap(),
        b"log-bytes"
    );
    assert_eq!(
        fs::read(out.path().join("VMWARE").join("B.txt")).unwrap(),
        b"upper"
    );
    assert_eq!(
        fs::read(out.path().join("VMWARE").join("a.txt")).unwrap(),
        b"lower"
    );
    assert_eq!(
        fs::read(out.path().join("VMWARE").join("文档.txt")).unwrap(),
        "你好".as_bytes()
    );
    assert!(out.path().join("VMWARE").join("empty").is_dir());
}

#[test]
fn update_adds_a_name_that_sorts_before_kept_members() {
    let tmp = tempfile::tempdir().unwrap();
    let zeta = tmp.path().join("zeta.txt");
    fs::write(&zeta, b"zeta").unwrap();
    let dest = tmp.path().join("notes.7z");
    add(&dest, &[zeta], &fast(), &NopProgress).unwrap();
    let alpha = tmp.path().join("alpha.txt");
    fs::write(&alpha, b"alpha").unwrap();
    add(&dest, &[alpha], &fast(), &NopProgress).unwrap();
    let out = extract_bytes(&dest, None);
    assert_eq!(fs::read(out.path().join("zeta.txt")).unwrap(), b"zeta");
    assert_eq!(fs::read(out.path().join("alpha.txt")).unwrap(), b"alpha");
}

#[test]
fn same_tree_gives_identical_7z_bytes() {
    let tmp = tempfile::tempdir().unwrap();
    let root = vmware_tree(tmp.path());
    let opts = fast();
    let first = tmp.path().join("a.7z");
    let second = tmp.path().join("b.7z");
    add(&first, std::slice::from_ref(&root), &opts, &NopProgress).unwrap();
    add(&second, std::slice::from_ref(&root), &opts, &NopProgress).unwrap();
    assert_eq!(
        fs::read(&first).unwrap(),
        fs::read(&second).unwrap(),
        "two packs of the same tree differed"
    );
}
