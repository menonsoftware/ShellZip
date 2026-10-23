# Test patterns

All snippets follow workspace conventions: behaviour-named tests, `unwrap` allowed in tests, fixtures in `tempfile::tempdir()`.

## Unit, table-driven

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_io_kinds_to_rar_exit_codes() {
        use std::io::{Error as IoError, ErrorKind};
        let cases = [
            (ErrorKind::NotFound, 6),
            (ErrorKind::PermissionDenied, 5),
            (ErrorKind::Interrupted, 255),
            (ErrorKind::Other, 2),
        ];
        for (kind, want) in cases {
            let err = Error::io("ctx", IoError::from(kind));
            assert_eq!(err.exit_code(), want, "kind {kind:?}");
        }
    }
}
```

## Unit, error path with `matches!`

```rust
#[test]
fn rejects_parent_dir_component() {
    let err = safe_rel("a/../b").unwrap_err();
    assert!(matches!(err, Error::UnsafePath(_)), "got {err:?}");
}
```

## Integration round-trip (`tests/<scenario>.rs`)

```rust
use std::fs;
use std::path::Path;

use shellzip_core::{CompressOpts, ExtractOpts, NopProgress, Overwrite, add, extract, test};

fn sample_tree(root: &Path) {
    fs::create_dir_all(root.join("sub")).unwrap();
    fs::write(root.join("sub/hello.txt"), b"hello").unwrap();
    fs::write(root.join("empty.bin"), b"").unwrap();
    fs::write(root.join("文件.txt"), "你好").unwrap();
}

#[test]
fn round_trips_every_format() {
    for ext in ["zip", "tar", "tar.gz", "tar.zst"] {
        let tmp = tempfile::tempdir().unwrap();
        let src = tmp.path().join("src");
        fs::create_dir(&src).unwrap();
        sample_tree(&src);

        let archive = tmp.path().join(format!("bundle.{ext}"));
        add(&archive, &[src.clone()], &CompressOpts::default(), &NopProgress).unwrap();
        test(&archive, &NopProgress).unwrap();

        let out = tmp.path().join("out");
        let mut opts = ExtractOpts::new(&out);
        opts.overwrite = Overwrite::Always;
        extract(&archive, &opts, None, &NopProgress).unwrap();

        assert_eq!(fs::read(out.join("src/sub/hello.txt")).unwrap(), b"hello", "{ext}");
        assert_eq!(fs::read(out.join("src/文件.txt")).unwrap(), "你好".as_bytes(), "{ext}");
    }
}
```

## CLI end-to-end with `assert_cmd` (`tests/cli.rs`)

```rust
use predicates::str::contains;

fn szip() -> assert_cmd::Command {
    assert_cmd::Command::cargo_bin("szip").unwrap()
}

#[test]
fn test_command_reports_bad_checksum_with_exit_3() {
    let tmp = tempfile::tempdir().unwrap();
    let archive = tmp.path().join("corrupt.zip");
    // ... write a ZIP then flip a payload byte ...
    szip()
        .args(["t", archive.to_str().unwrap()])
        .assert()
        .code(3)
        .stderr(contains("checksum mismatch"));
}

#[test]
fn usage_error_exits_7() {
    szip().args(["x"]).assert().code(7);
}
```

Environment and cwd: `.env("SZIP_PASSWORD", "x")`, `.current_dir(tmp.path())`.

## Property test with `proptest`

`cargo add proptest -p shellzip-core --dev`

```rust
#[cfg(test)]
mod prop {
    use proptest::prelude::*;

    use super::*;

    proptest! {
        #[test]
        fn safe_rel_never_yields_parent_or_root(name in "\\PC{1,64}") {
            if let Ok(rel) = safe_rel(&name) {
                prop_assert!(rel.components().all(|c| matches!(c, std::path::Component::Normal(_))));
            }
        }
    }
}
```

## Snapshot test with `insta`

`cargo add insta -p shellzip-cli --dev --features filters` and `cargo install cargo-insta`.

```rust
#[test]
fn technical_listing_layout() {
    let output = szip().args(["lt", "fixtures/sample.zip"]).output().unwrap();
    let text = String::from_utf8(output.stdout).unwrap();
    insta::with_settings!({ filters => vec![(r"\d{4}-\d{2}-\d{2} \d{2}:\d{2}", "[time]")] }, {
        insta::assert_snapshot!(text);
    });
}
```

Review snapshots with `cargo insta review`; commit the `.snap` files beside the test.

## Counting fake for a trait

```rust
use std::sync::atomic::{AtomicU64, Ordering};

use shellzip_core::ProgressSink;

#[derive(Default)]
struct CountingSink {
    bytes: AtomicU64,
    entries: AtomicU64,
}

impl ProgressSink for CountingSink {
    fn begin(&self, _label: &str, _total: u64) {}
    fn begin_entry(&self, _name: &str, _bytes: u64) {
        self.entries.fetch_add(1, Ordering::Relaxed);
    }
    fn advance(&self, bytes: u64) {
        self.bytes.fetch_add(bytes, Ordering::Relaxed);
    }
    fn end_entry(&self) {}
    fn message(&self, _text: &str) {}
}

#[test]
fn extract_reports_every_byte() {
    let sink = CountingSink::default();
    // ... extract with &sink ...
    assert_eq!(sink.bytes.load(Ordering::Relaxed), expected_total);
}
```

## In-memory reader/writer

```rust
use std::io::Cursor;

let mut input = Cursor::new(b"payload".to_vec());
let mut output = Cursor::new(Vec::new());
copy_with_hash(&mut input, &mut output).unwrap();
assert_eq!(output.into_inner(), b"payload");
```

## Platform-gated test

```rust
#[cfg(unix)]
#[test]
fn restores_symlink_inside_destination() { /* ... */ }

#[cfg(windows)]
#[test]
fn rejects_reserved_device_names() { /* ... */ }
```

## Interop with OS tools

```rust
#[test]
fn reads_tar_written_by_bsdtar() {
    let Ok(tar) = which_tar() else { return; }; // skip silently when the tool is absent
    // run std::process::Command::new(tar) to build the fixture, then list()/extract() it
}
```

Skipping when the external tool is missing is acceptable here because the test is about interop, not about our code's presence.
