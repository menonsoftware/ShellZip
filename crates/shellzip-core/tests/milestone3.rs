//! Milestone 3: ZIP AES-256, RAR extraction, and 7z extraction.
//!
//! The password in these tests is a fixed fixture vector.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use sha2::{Digest, Sha256};
use shellzip_core::{
    CompressOpts, Error, ExtractOpts, ListOpts, NopProgress, Overwrite, Password, UpdateMode, add,
    extract, list, list_with, test,
};

const FIXTURE_PASSWORD: &str = "szip-fixture";

fn password() -> Password {
    Password::new(FIXTURE_PASSWORD).unwrap()
}

fn fixtures_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../fixtures")
}

fn extract_bytes(archive: &Path, password: Option<Password>) -> tempfile::TempDir {
    let out = tempfile::tempdir().unwrap();
    let mut opts = ExtractOpts::new(out.path());
    opts.overwrite = Overwrite::Always;
    opts.password = password;
    extract(archive, &opts, None, &NopProgress).unwrap();
    out
}

#[test]
fn zip_aes_round_trips_and_leaves_names_visible() {
    let tmp = tempfile::tempdir().unwrap();
    let src = tmp.path().join("hello.txt");
    fs::write(&src, b"hello\n").unwrap();
    let archive = tmp.path().join("secret.zip");
    let mut opts = CompressOpts {
        level: 1,
        threads: 4,
        password: Some(password()),
        ..CompressOpts::default()
    };
    add(&archive, &[src], &opts, &NopProgress).unwrap();

    let (_opened, entries) = list(&archive).unwrap();
    assert!(
        entries
            .iter()
            .any(|entry| entry.path.ends_with("hello.txt")),
        "ZIP AES does not hide names: {entries:?}"
    );
    assert!(entries.iter().any(|entry| entry.encrypted));

    let missing = extract(
        &archive,
        &ExtractOpts::new(tmp.path().join("no-password")),
        None,
        &NopProgress,
    )
    .unwrap_err();
    assert!(matches!(missing, Error::Encrypted(_)), "{missing}");
    assert_eq!(missing.exit_code(), 11);

    opts.password = Some(Password::new("wrong-fixture").unwrap());
    let wrong = test(&archive, opts.password.as_ref(), &NopProgress).unwrap_err();
    assert!(matches!(wrong, Error::BadPassword), "{wrong}");
    assert_eq!(wrong.exit_code(), 11);

    let locked = test(&archive, None, &NopProgress).unwrap();
    assert_eq!(locked.exit_code(), 11);
    assert!(!locked.encrypted.is_empty());

    let checked = test(&archive, Some(&password()), &NopProgress).unwrap();
    assert_eq!(checked.exit_code(), 0);

    let out = extract_bytes(&archive, Some(password()));
    let found = fs::read_dir(out.path())
        .unwrap()
        .flatten()
        .find_map(|entry| {
            let path = entry.path();
            path.ends_with("hello.txt").then_some(path)
        })
        .expect("extracted hello.txt");
    assert_eq!(fs::read(found).unwrap(), b"hello\n");
}

#[test]
fn encrypting_an_existing_zip_requires_force() {
    let tmp = tempfile::tempdir().unwrap();
    let src = tmp.path().join("hello.txt");
    fs::write(&src, b"hello\n").unwrap();
    let archive = tmp.path().join("notes.zip");
    add(
        &archive,
        std::slice::from_ref(&src),
        &CompressOpts::default(),
        &NopProgress,
    )
    .unwrap();
    let mut opts = CompressOpts {
        password: Some(password()),
        ..CompressOpts::default()
    };
    let err = add(&archive, std::slice::from_ref(&src), &opts, &NopProgress).unwrap_err();
    assert!(matches!(err, Error::Usage(_)), "{err}");
    opts.force = true;
    opts.update = UpdateMode::Add;
    add(&archive, &[src], &opts, &NopProgress).unwrap();
    let (_opened, entries) = list(&archive).unwrap();
    assert!(
        entries
            .iter()
            .all(|entry| entry.encrypted || entry.kind != shellzip_core::EntryKind::File)
    );
}

#[test]
fn skip_encrypted_members_warn_instead_of_failing() {
    let tmp = tempfile::tempdir().unwrap();
    let src = tmp.path().join("hello.txt");
    fs::write(&src, b"hello\n").unwrap();
    let archive = tmp.path().join("secret.zip");
    let opts = CompressOpts {
        password: Some(password()),
        ..CompressOpts::default()
    };
    add(&archive, &[src], &opts, &NopProgress).unwrap();
    let out = tmp.path().join("out");
    fs::create_dir(&out).unwrap();
    let mut extract_opts = ExtractOpts::new(&out);
    extract_opts.skip_encrypted = true;
    let report = extract(&archive, &extract_opts, None, &NopProgress).unwrap();
    assert_eq!(report.skipped, 1);
    assert!(!report.warnings.is_empty());
    assert!(fs::read_dir(&out).unwrap().next().is_none());
}

#[test]
fn passwords_are_rejected_for_tar() {
    let tmp = tempfile::tempdir().unwrap();
    let src = tmp.path().join("hello.txt");
    fs::write(&src, b"hello\n").unwrap();
    let opts = CompressOpts {
        password: Some(password()),
        ..CompressOpts::default()
    };
    let err = add(&tmp.path().join("notes.tar"), &[src], &opts, &NopProgress).unwrap_err();
    assert!(matches!(err, Error::Usage(_)), "{err}");
}

#[test]
fn symlink_targets_outside_the_destination_are_refused() {
    let tmp = tempfile::tempdir().unwrap();
    let archive = tmp.path().join("link.tar");
    {
        let file = fs::File::create(&archive).unwrap();
        let mut builder = tar::Builder::new(file);
        let mut header = tar::Header::new_gnu();
        header.set_path("escape").unwrap();
        header.set_entry_type(tar::EntryType::Symlink);
        header.set_size(0);
        header.set_mode(0o777);
        header.set_mtime(0);
        header.set_link_name("../outside").unwrap();
        header.set_cksum();
        builder.append(&header, std::io::empty()).unwrap();
        builder.finish().unwrap();
    }
    let out = tmp.path().join("out");
    fs::create_dir(&out).unwrap();
    let err = extract(&archive, &ExtractOpts::new(&out), None, &NopProgress).unwrap_err();
    assert!(matches!(err, Error::UnsafePath(_)), "{err}");
    assert!(!tmp.path().join("outside").exists());

    let mut opts = ExtractOpts::new(&out);
    opts.allow_unsafe_links = true;
    match extract(&archive, &opts, None, &NopProgress) {
        Ok(_) => {
            let meta = fs::symlink_metadata(out.join("escape")).unwrap();
            assert!(meta.file_type().is_symlink());
        }
        Err(Error::Io { .. }) => {}
        Err(err) => panic!("{err}"),
    }
}

#[cfg(windows)]
#[test]
fn mark_of_the_web_is_copied_onto_extracted_files() {
    let tmp = tempfile::tempdir().unwrap();
    let src = tmp.path().join("hello.txt");
    fs::write(&src, b"hello\n").unwrap();
    let archive = tmp.path().join("notes.zip");
    add(&archive, &[src], &CompressOpts::default(), &NopProgress).unwrap();
    let zone = "[ZoneTransfer]\r\nZoneId=3\r\n";
    fs::write(format!("{}:Zone.Identifier", archive.display()), zone).unwrap();
    let out = tmp.path().join("out");
    fs::create_dir(&out).unwrap();
    let mut opts = ExtractOpts::new(&out);
    opts.overwrite = Overwrite::Always;
    opts.mark_of_the_web = true;
    extract(&archive, &opts, None, &NopProgress).unwrap();
    let extracted = fs::read_dir(&out)
        .unwrap()
        .flatten()
        .find(|entry| entry.path().extension().is_some_and(|ext| ext == "txt"))
        .unwrap()
        .path();
    let copied = fs::read_to_string(format!("{}:Zone.Identifier", extracted.display())).unwrap();
    assert!(copied.contains("ZoneId=3"), "{copied}");
}

#[cfg(not(windows))]
#[test]
fn mark_of_the_web_is_refused_off_windows() {
    let tmp = tempfile::tempdir().unwrap();
    let archive = tmp.path().join("notes.zip");
    fs::write(&archive, b"not-an-archive").unwrap();
    let mut opts = ExtractOpts::new(tmp.path().join("out"));
    opts.mark_of_the_web = true;
    let err = extract(&archive, &opts, None, &NopProgress).unwrap_err();
    assert!(matches!(err, Error::Usage(_)), "{err}");
}

#[cfg(feature = "rar")]
#[test]
fn rar_fixtures_match_their_checksums_and_extract() {
    let root = fixtures_root();
    let sums = fs::read_to_string(root.join("SHA256SUMS")).unwrap_or_else(|_| {
        panic!("missing fixtures/SHA256SUMS; run cargo run -p xtask -- build-fixtures")
    });
    assert!(!sums.trim().is_empty());
    for line in sums.lines() {
        let (hash, name) = line
            .split_once("  ")
            .unwrap_or_else(|| panic!("bad checksum line {line}"));
        let bytes = fs::read(root.join(name)).unwrap_or_else(|err| panic!("{name}: {err}"));
        let got = format!("{:x}", Sha256::digest(bytes));
        assert_eq!(got, hash, "{name}");
    }

    let dir = root.join("rar");
    for name in ["store.rar", "rar4.rar", "blake.rar"] {
        let out = extract_bytes(&dir.join(name), None);
        assert_eq!(fs::read(out.path().join("hello.txt")).unwrap(), b"hello\n");
        assert_matches_unrar(&dir.join(name), None);
    }

    let (_opened, solid) = list(&dir.join("solid.rar")).unwrap();
    assert!(
        solid
            .iter()
            .any(|entry| entry.path.ends_with("a.txt") && entry.solid)
    );
    assert!(
        solid
            .iter()
            .any(|entry| entry.path.ends_with("b.txt") && entry.solid)
    );
    let solid_out = extract_bytes(&dir.join("solid.rar"), None);
    assert_eq!(
        fs::read(solid_out.path().join("a.txt")).unwrap(),
        b"alpha\n"
    );
    assert_eq!(
        fs::read(solid_out.path().join("b.txt")).unwrap(),
        b"bravo\n"
    );
    assert_matches_unrar(&dir.join("solid.rar"), None);

    let (_opened, locked) = list(&dir.join("password.rar")).unwrap();
    assert!(
        locked
            .iter()
            .any(|entry| entry.encrypted && entry.path.ends_with("hello.txt"))
    );
    let missing = list_with(&dir.join("headers.rar"), &ListOpts::default()).unwrap_err();
    assert!(matches!(missing, Error::Encrypted(_)), "{missing}");
    assert_eq!(missing.exit_code(), 11);
    let wrong = list_with(
        &dir.join("headers.rar"),
        &ListOpts {
            password: Some(Password::new("wrong-fixture").unwrap()),
            ..ListOpts::default()
        },
    )
    .unwrap_err();
    assert_eq!(wrong.exit_code(), 11);

    let out = extract_bytes(&dir.join("password.rar"), Some(password()));
    assert_eq!(fs::read(out.path().join("hello.txt")).unwrap(), b"hello\n");
    let headers = extract_bytes(&dir.join("headers.rar"), Some(password()));
    assert_eq!(
        fs::read(headers.path().join("hello.txt")).unwrap(),
        b"hello\n"
    );
    assert_matches_unrar(&dir.join("password.rar"), Some(FIXTURE_PASSWORD));
    assert_matches_unrar(&dir.join("headers.rar"), Some(FIXTURE_PASSWORD));

    let parts: Vec<_> = fs::read_dir(&dir)
        .unwrap()
        .flatten()
        .map(|entry| entry.path())
        .filter(|path| {
            path.file_name()
                .and_then(|name| name.to_str())
                .is_some_and(|name| name.starts_with("volumes.part"))
        })
        .collect();
    assert!(
        parts.len() >= 2,
        "expected multivolume parts, found {parts:?}"
    );
    let first = parts
        .iter()
        .find(|path| {
            let name = path.file_name().unwrap().to_str().unwrap();
            name.contains("part1.") || name.contains("part01.")
        })
        .unwrap();
    let volume_out = extract_bytes(first, None);
    assert_eq!(
        fs::read(volume_out.path().join("big.bin")).unwrap(),
        vec![0xA5; 30_000]
    );
    let last = parts.iter().find(|path| *path != first).unwrap();
    let from_later = extract_bytes(last, None);
    assert_eq!(
        fs::read(from_later.path().join("big.bin")).unwrap(),
        vec![0xA5; 30_000]
    );
    assert_matches_unrar(first, None);
}

#[cfg(feature = "rar")]
fn assert_matches_unrar(archive: &Path, password: Option<&str>) {
    let Some(tool) = find_unrar() else {
        return;
    };
    let tmp = tempfile::tempdir().unwrap();
    let ours = tmp.path().join("ours");
    let theirs = tmp.path().join("theirs");
    fs::create_dir_all(&ours).unwrap();
    fs::create_dir_all(&theirs).unwrap();
    let mut opts = ExtractOpts::new(&ours);
    opts.overwrite = Overwrite::Always;
    opts.password = password.map(|value| Password::new(value).unwrap());
    extract(archive, &opts, None, &NopProgress).unwrap();
    let mut args = vec!["x".into(), "-y".into(), "-inul".into()];
    match password {
        Some(value) => args.push(format!("-p{value}")),
        None => args.push("-p-".into()),
    }
    args.push(archive.display().to_string());
    args.push(format!("{}{}", theirs.display(), std::path::MAIN_SEPARATOR));
    let status = Command::new(&tool).args(&args).status().unwrap();
    assert!(status.success(), "UnRAR status {status}");
    for entry in fs::read_dir(&ours).unwrap().flatten() {
        if !entry.file_type().unwrap().is_file() {
            continue;
        }
        let name = entry.file_name();
        let left = fs::read(entry.path()).unwrap();
        let right = fs::read(theirs.join(&name)).unwrap_or_else(|err| panic!("{name:?}: {err}"));
        assert_eq!(left, right, "{name:?}");
    }
}

#[cfg(feature = "rar")]
fn find_unrar() -> Option<PathBuf> {
    for candidate in [
        r"C:\Program Files\WinRAR\UnRAR.exe",
        r"C:\Program Files\WinRAR\unrar.exe",
        r"C:\Program Files (x86)\WinRAR\UnRAR.exe",
    ] {
        let path = PathBuf::from(candidate);
        if path.is_file() {
            return Some(path);
        }
    }
    std::env::var_os("PATH").and_then(|path| {
        std::env::split_paths(&path).find_map(|dir| {
            ["UnRAR.exe", "unrar.exe", "unrar"]
                .into_iter()
                .map(|name| dir.join(name))
                .find(|candidate| candidate.is_file())
        })
    })
}

#[cfg(feature = "sevenz")]
#[test]
fn sevenz_lzma2_bzip2_deflate_and_aes_round_trip() {
    let tmp = tempfile::tempdir().unwrap();
    let src = tmp.path().join("tree");
    fs::create_dir(&src).unwrap();
    fs::write(src.join("hello.txt"), b"hello\n").unwrap();
    fs::write(src.join("other.txt"), b"other\n").unwrap();

    let plain = tmp.path().join("plain.7z");
    sevenz_rust2::compress_to_path(&src, &plain).unwrap();
    let (_opened, entries) = list(&plain).unwrap();
    assert!(
        entries
            .iter()
            .any(|entry| entry.path.ends_with("hello.txt")),
        "{entries:?}"
    );
    let out = extract_bytes(&plain, None);
    assert_eq!(fs::read(out.path().join("hello.txt")).unwrap(), b"hello\n");
    assert_eq!(fs::read(out.path().join("other.txt")).unwrap(), b"other\n");

    for (name, method) in [
        ("bzip2.7z", bzip2_method()),
        ("deflate.7z", deflate_method()),
    ] {
        let archive = tmp.path().join(name);
        write_sevenz(&archive, &src.join("hello.txt"), vec![method], false, None);
        let out = extract_bytes(&archive, None);
        assert_eq!(fs::read(out.path().join("hello.txt")).unwrap(), b"hello\n");
    }

    let visible = tmp.path().join("aes.7z");
    write_sevenz(
        &visible,
        &src.join("hello.txt"),
        vec![lzma2_method()],
        false,
        Some(FIXTURE_PASSWORD),
    );
    let (_opened, entries) = list(&visible).unwrap();
    assert!(
        entries
            .iter()
            .any(|entry| entry.path.ends_with("hello.txt") && entry.encrypted)
    );
    let missing = extract(
        &visible,
        &ExtractOpts::new(tmp.path().join("denied")),
        None,
        &NopProgress,
    )
    .unwrap_err();
    assert_eq!(missing.exit_code(), 11);
    let out = extract_bytes(&visible, Some(password()));
    assert_eq!(fs::read(out.path().join("hello.txt")).unwrap(), b"hello\n");

    let hidden = tmp.path().join("headers.7z");
    write_sevenz(
        &hidden,
        &src.join("hello.txt"),
        vec![lzma2_method()],
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

#[cfg(feature = "sevenz")]
fn write_sevenz(
    dest: &Path,
    src: &Path,
    methods: Vec<sevenz_rust2::EncoderConfiguration>,
    encrypt_header: bool,
    password: Option<&str>,
) {
    let mut writer = sevenz_rust2::ArchiveWriter::create(dest).unwrap();
    let mut chain = methods;
    if let Some(password) = password {
        chain.insert(0, aes_method_with(password));
    }
    if !chain.is_empty() {
        writer.set_content_methods(chain);
    }
    writer.set_encrypt_header(encrypt_header && password.is_some());
    writer
        .push_archive_entry(
            sevenz_rust2::ArchiveEntry::from_path(src, "hello.txt".to_string()),
            Some(fs::File::open(src).unwrap()),
        )
        .unwrap();
    writer.finish().unwrap();
}

#[cfg(feature = "sevenz")]
fn aes_method_with(password: &str) -> sevenz_rust2::EncoderConfiguration {
    sevenz_rust2::encoder_options::AesEncoderOptions::new(sevenz_rust2::Password::new(password))
        .into()
}

#[cfg(feature = "sevenz")]
fn lzma2_method() -> sevenz_rust2::EncoderConfiguration {
    sevenz_rust2::EncoderMethod::LZMA2.into()
}

#[cfg(feature = "sevenz")]
fn bzip2_method() -> sevenz_rust2::EncoderConfiguration {
    sevenz_rust2::encoder_options::Bzip2Options::from_level(1).into()
}

#[cfg(feature = "sevenz")]
fn deflate_method() -> sevenz_rust2::EncoderConfiguration {
    sevenz_rust2::encoder_options::DeflateOptions::from_level(1).into()
}
