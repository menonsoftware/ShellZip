//! ZIP rename and header summary.

use std::fs;
use std::path::{Path, PathBuf};

use shellzip_core::{
    CompressOpts, Error, NopProgress, ProgressSink, add, archive_summary, list, rename,
    write_member, zip_comment,
};

fn stored_zip(dir: &Path, names: &[(&str, &[u8])]) -> PathBuf {
    let src = dir.join("src");
    fs::create_dir(&src).unwrap();
    let mut inputs = Vec::new();
    for (name, bytes) in names {
        let path = src.join(name);
        fs::write(&path, bytes).unwrap();
        inputs.push(path);
    }
    let archive = dir.join("sample.zip");
    let opts = CompressOpts {
        level: 1,
        comment: Some("keep-me".to_string()),
        ..CompressOpts::default()
    };
    add(&archive, &inputs, &opts, &NopProgress).unwrap();
    archive
}

fn names(path: &Path) -> Vec<String> {
    let (_opened, entries) = list(path).unwrap();
    let mut names: Vec<_> = entries.into_iter().map(|entry| entry.path).collect();
    names.sort();
    names
}

fn fingerprint(path: &Path, name: &str) -> (Option<u32>, Option<u64>, Vec<u8>) {
    let (_opened, entries) = list(path).unwrap();
    let entry = entries.iter().find(|entry| entry.path == name).unwrap();
    let mut body = Vec::new();
    write_member(path, Some(name), &mut body, None).unwrap();
    (entry.crc32, entry.packed_size, body)
}

#[test]
fn zip_rename_keeps_payload_bytes_and_new_name() {
    let tmp = tempfile::tempdir().unwrap();
    let payload = b"alpha\0\xffbeta-alpha\0\xffbeta";
    let other = b"keep-me-bytes";
    let archive = stored_zip(tmp.path(), &[("a.bin", payload), ("b.bin", other)]);
    let before = fingerprint(&archive, "a.bin");
    let other_before = fingerprint(&archive, "b.bin");

    let report = rename(&archive, "a.bin", "c.bin", None, &NopProgress).unwrap();

    assert_eq!(report.removed, 0);
    assert!(report.kept >= 2, "kept {}", report.kept);
    assert_eq!(report.path, archive);
    assert!(report.packed_bytes > 0);
    assert_eq!(
        names(&archive),
        vec!["b.bin".to_string(), "c.bin".to_string()]
    );
    assert_eq!(fingerprint(&archive, "c.bin"), before);
    assert_eq!(fingerprint(&archive, "b.bin"), other_before);
    assert_eq!(other_before.2, other);
    assert_eq!(zip_comment(&archive).unwrap(), "keep-me");
}

#[test]
fn rename_rejects_an_existing_destination() {
    let tmp = tempfile::tempdir().unwrap();
    let archive = stored_zip(tmp.path(), &[("a.bin", b"one"), ("b.bin", b"two")]);
    let before = fs::read(&archive).unwrap();

    let err = rename(&archive, "a.bin", "b.bin", None, &NopProgress).unwrap_err();

    assert!(
        matches!(&err, Error::Usage(text) if text.contains("already exists")),
        "{err}"
    );
    assert_eq!(fs::read(&archive).unwrap(), before);
    assert_eq!(
        names(&archive),
        vec!["a.bin".to_string(), "b.bin".to_string()]
    );
}

#[test]
fn rename_rejects_parent_directory_names() {
    let tmp = tempfile::tempdir().unwrap();
    let archive = stored_zip(tmp.path(), &[("a.bin", b"one")]);
    let before = fs::read(&archive).unwrap();

    let err = rename(&archive, "a.bin", "../escaped.bin", None, &NopProgress).unwrap_err();

    assert!(matches!(&err, Error::UnsafePath(_)), "{err}");
    assert_eq!(fs::read(&archive).unwrap(), before);
}

#[test]
fn rename_directory_rewrites_children() {
    let tmp = tempfile::tempdir().unwrap();
    let dir = tmp.path().join("dir");
    fs::create_dir(&dir).unwrap();
    fs::write(dir.join("a.txt"), b"inside").unwrap();
    let archive = tmp.path().join("tree.zip");
    add(
        &archive,
        &[dir],
        &CompressOpts {
            level: 0,
            ..CompressOpts::default()
        },
        &NopProgress,
    )
    .unwrap();

    rename(&archive, "dir", "renamed", None, &NopProgress).unwrap();

    let mut body = Vec::new();
    write_member(&archive, Some("renamed/a.txt"), &mut body, None).unwrap();
    assert_eq!(body, b"inside");
    let listed = names(&archive);
    assert!(
        listed.iter().any(|name| name == "renamed/a.txt"),
        "{listed:?}"
    );
    assert!(
        listed
            .iter()
            .all(|name| name != "dir" && !name.starts_with("dir/")),
        "{listed:?}"
    );
}

#[test]
fn rename_cancel_leaves_the_archive_unchanged() {
    let tmp = tempfile::tempdir().unwrap();
    let archive = stored_zip(tmp.path(), &[("a.bin", b"one")]);
    let before = fs::read(&archive).unwrap();

    let err = rename(&archive, "a.bin", "c.bin", None, &Cancel).unwrap_err();

    assert!(matches!(&err, Error::Interrupted), "{err}");
    assert_eq!(fs::read(&archive).unwrap(), before);
}

#[test]
fn archive_summary_on_stored_zip() {
    let tmp = tempfile::tempdir().unwrap();
    let src = tmp.path().join("hello.txt");
    fs::write(&src, b"hello").unwrap();
    let archive = tmp.path().join("plain.zip");
    add(
        &archive,
        &[src],
        &CompressOpts {
            level: 0,
            ..CompressOpts::default()
        },
        &NopProgress,
    )
    .unwrap();

    let summary = archive_summary(&archive).unwrap();

    assert_eq!(summary.kind, "ZIP archive");
    assert_eq!(summary.version, "1.0 / Store");
    assert_eq!(summary.files, 1);
    assert_eq!(summary.folders, 0);
    assert_eq!(summary.total_size, 5);
    assert_eq!(summary.packed_size, 5);
    assert_eq!(summary.ratio_percent, 100);
    assert_eq!(summary.comment, None);
    assert!(!summary.encrypted);
    assert_eq!(summary.sfx_bytes, 0);
    assert!(
        summary.host_os == "Unix" || summary.host_os == "DOS",
        "host {}",
        summary.host_os
    );
}

struct Cancel;

impl ProgressSink for Cancel {
    fn begin(&self, _label: &str, _total_bytes: u64) {}
    fn begin_entry(&self, _name: &str, _bytes: u64) {}
    fn advance(&self, _bytes: u64) {}
    fn end_entry(&self) {}
    fn message(&self, _text: &str) {}
    fn is_cancelled(&self) -> bool {
        true
    }
}
