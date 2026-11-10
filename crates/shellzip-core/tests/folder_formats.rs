//! Nested folders, mixed-case names, and updates for every format we can write.
//!
//! The planner stores directories first, then files grouped by extension
//! (`fs::walk::sort_key`). That is not byte order: `VMWARE/Worker1` comes
//! before `VMWARE/CP1/vmware-0.log`, and `a.txt` comes before `B.txt`.

use std::fs;
use std::path::{Path, PathBuf};

use shellzip_core::{
    CompressOpts, EntryKind, Error, ExtractOpts, NopProgress, Overwrite, add, extract, list, test,
};

struct Case {
    label: &'static str,
    ext: &'static str,
    threads: u16,
    can_update: bool,
}

fn cases() -> Vec<Case> {
    vec![
        Case {
            label: "zip",
            ext: "zip",
            threads: 1,
            can_update: true,
        },
        Case {
            label: "zip-parallel",
            ext: "zip",
            threads: 4,
            can_update: true,
        },
        Case {
            label: "tar",
            ext: "tar",
            threads: 1,
            can_update: false,
        },
        Case {
            label: "tar.gz",
            ext: "tar.gz",
            threads: 1,
            can_update: false,
        },
        Case {
            label: "tgz",
            ext: "tgz",
            threads: 1,
            can_update: false,
        },
        Case {
            label: "tar.zst",
            ext: "tar.zst",
            threads: 1,
            can_update: false,
        },
        Case {
            label: "tzst",
            ext: "tzst",
            threads: 1,
            can_update: false,
        },
        #[cfg(feature = "sevenz")]
        Case {
            label: "7z",
            ext: "7z",
            threads: 1,
            can_update: true,
        },
    ]
}

fn opts(threads: u16) -> CompressOpts {
    CompressOpts {
        level: 1,
        threads,
        ..CompressOpts::default()
    }
}

fn vmware_tree(parent: &Path) -> PathBuf {
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

/// Same key as `fs::walk::sort_key`: directories, then extension, then lowercase path.
fn sort_key(rel: &str, is_dir: bool) -> (u8, String, String) {
    let ext = if is_dir {
        String::new()
    } else {
        rel.rsplit('/')
            .next()
            .and_then(|name| name.rsplit_once('.'))
            .map(|(_, ext)| ext.to_ascii_lowercase())
            .unwrap_or_default()
    };
    (u8::from(!is_dir), ext, rel.to_ascii_lowercase())
}

fn planned_names() -> Vec<String> {
    let mut items = [
        ("VMWARE", true),
        ("VMWARE/Worker1", true),
        ("VMWARE/CP1", true),
        ("VMWARE/empty", true),
        ("VMWARE/Worker1/x.txt", false),
        ("VMWARE/CP1/vmware-0.log", false),
        ("VMWARE/B.txt", false),
        ("VMWARE/a.txt", false),
        ("VMWARE/文档.txt", false),
    ];
    items.sort_by_key(|(rel, is_dir)| sort_key(rel, *is_dir));
    items.into_iter().map(|(rel, _)| rel.to_string()).collect()
}

fn extract_all(archive: &Path, output: &Path) {
    fs::create_dir_all(output).unwrap();
    let mut opts = ExtractOpts::new(output);
    opts.overwrite = Overwrite::Always;
    extract(archive, &opts, None, &NopProgress).unwrap();
}

fn assert_tree(label: &str, output: &Path) {
    let root = output.join("VMWARE");
    assert_eq!(
        fs::read(root.join("Worker1").join("x.txt")).unwrap(),
        b"worker",
        "{label}"
    );
    assert_eq!(
        fs::read(root.join("CP1").join("vmware-0.log")).unwrap(),
        b"log-bytes",
        "{label}"
    );
    assert_eq!(fs::read(root.join("B.txt")).unwrap(), b"upper", "{label}");
    assert_eq!(fs::read(root.join("a.txt")).unwrap(), b"lower", "{label}");
    assert_eq!(
        fs::read(root.join("文档.txt")).unwrap(),
        "你好".as_bytes(),
        "{label}"
    );
    assert!(
        root.join("empty").is_dir(),
        "{label} dropped the empty directory"
    );
}

#[test]
fn nested_folder_round_trips_every_writable_format() {
    let expected = planned_names();
    for case in cases() {
        let tmp = tempfile::tempdir().unwrap();
        let root = vmware_tree(tmp.path());
        let archive = tmp.path().join(format!("bundle.{}", case.ext));
        add(&archive, &[root], &opts(case.threads), &NopProgress)
            .unwrap_or_else(|err| panic!("{}: {err}", case.label));
        let report = test(&archive, None, &NopProgress).unwrap();
        assert_eq!(report.exit_code(), 0, "{}", case.label);
        let (_opened, entries) = list(&archive).unwrap();
        let paths: Vec<String> = entries.iter().map(|entry| entry.path.clone()).collect();
        assert_eq!(paths, expected, "{}", case.label);
        assert!(
            entries
                .iter()
                .any(|entry| { entry.kind == EntryKind::Dir && entry.path.ends_with("empty") }),
            "{}: {paths:?}",
            case.label
        );
        let out = tmp.path().join("out");
        extract_all(&archive, &out);
        assert_tree(case.label, &out);
    }
}

#[test]
fn parallel_zip_round_trips_a_folder_of_spilled_files() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path().join("drivers");
    let dir = root.join("M2V0D17D").join("Packages").join("Drivers");
    fs::create_dir_all(dir.join("empty")).unwrap();
    fs::write(dir.join("note.txt"), b"small").unwrap();
    let names = [
        "Display.A.bin",
        "Display.B.bin",
        "Display.C.bin",
        "Display.D.bin",
    ];
    for (index, name) in names.iter().enumerate() {
        let mut bytes = vec![0u8; 1024 * 1024];
        bytes[0] = b'A' + u8::try_from(index).unwrap();
        fs::write(dir.join(name), &bytes).unwrap();
    }
    let archive = tmp.path().join("drivers.zip");
    add(&archive, &[root], &opts(4), &NopProgress).unwrap_or_else(|err| panic!("zip: {err}"));
    let out = tmp.path().join("out");
    extract_all(&archive, &out);
    let stored = out.join("drivers/M2V0D17D/Packages/Drivers");
    assert_eq!(fs::read(stored.join("note.txt")).unwrap(), b"small");
    assert!(stored.join("empty").is_dir());
    for (index, name) in names.iter().enumerate() {
        let data = fs::read(stored.join(name)).unwrap();
        assert_eq!(data.len(), 1024 * 1024, "{name}");
        assert_eq!(data[0], b'A' + u8::try_from(index).unwrap(), "{name}");
        assert!(data[1..].iter().all(|byte| *byte == 0), "{name}");
    }
}

#[test]
fn parallel_zip_deflates_a_large_incompressible_file() {
    // Spilled pieces (>= 1 MiB) must survive deflate + raw merge without
    // truncating the stream ("incomplete deflate stream" on test/extract).
    const SIZE: usize = 4 * 1024 * 1024;
    let tmp = tempfile::tempdir().unwrap();
    let source = tmp.path().join("blob.bin");
    let mut bytes = vec![0u8; SIZE];
    for (index, byte) in bytes.iter_mut().enumerate() {
        *byte = (index.wrapping_mul(1103515245).wrapping_add(12345) >> 16) as u8;
    }
    fs::write(&source, &bytes).unwrap();
    let archive = tmp.path().join("blob.zip");
    add(&archive, &[source], &opts(4), &NopProgress).unwrap_or_else(|err| panic!("zip: {err}"));
    test(&archive, None, &NopProgress).unwrap_or_else(|err| panic!("test: {err}"));
    let out = tmp.path().join("out");
    extract_all(&archive, &out);
    assert_eq!(fs::read(out.join("blob.bin")).unwrap(), bytes);
}

#[test]
fn update_adds_an_earlier_name_beside_a_kept_member() {
    for case in cases().into_iter().filter(|case| case.can_update) {
        let tmp = tempfile::tempdir().unwrap();
        let zeta = tmp.path().join("zeta.txt");
        fs::write(&zeta, b"zeta").unwrap();
        let archive = tmp.path().join(format!("notes.{}", case.ext));
        let opts = opts(case.threads);
        add(&archive, &[zeta], &opts, &NopProgress)
            .unwrap_or_else(|err| panic!("{}: {err}", case.label));
        let alpha = tmp.path().join("alpha.txt");
        fs::write(&alpha, b"alpha").unwrap();
        add(&archive, &[alpha], &opts, &NopProgress)
            .unwrap_or_else(|err| panic!("{}: {err}", case.label));
        let out = tmp.path().join("out");
        extract_all(&archive, &out);
        assert_eq!(
            fs::read(out.join("zeta.txt")).unwrap(),
            b"zeta",
            "{}",
            case.label
        );
        assert_eq!(
            fs::read(out.join("alpha.txt")).unwrap(),
            b"alpha",
            "{}",
            case.label
        );
    }
}

#[test]
fn single_stream_rejects_a_folder() {
    for ext in ["gz", "zst"] {
        let tmp = tempfile::tempdir().unwrap();
        let root = vmware_tree(tmp.path());
        let archive = tmp.path().join(format!("bundle.{ext}"));
        let err = add(&archive, &[root], &opts(1), &NopProgress).unwrap_err();
        let Error::Usage(text) = &err else {
            panic!("{ext}: expected a usage error, got {err}");
        };
        assert!(
            text.contains("directory"),
            "{ext} should say a folder does not fit a single stream: {text}"
        );
        assert!(
            text.to_ascii_lowercase().contains("tar"),
            "{ext} should point at a tar wrapper: {text}"
        );
        assert!(!archive.exists(), "{ext} left a partial archive");
    }
}
