//! The stub extracts the ZIP appended to a copy of itself.

use std::fs;
use std::process::Command;

use shellzip_core::{CompressOpts, NopProgress, Password, add, build};

#[test]
fn stub_extracts_its_appended_zip() {
    let tmp = tempfile::tempdir().unwrap();
    let src = tmp.path().join("hello.txt");
    fs::write(&src, b"hello-sfx").unwrap();
    let zip_path = tmp.path().join("in.zip");
    add(
        &zip_path,
        &[src],
        &CompressOpts {
            level: 0,
            threads: 1,
            ..CompressOpts::default()
        },
        &NopProgress,
    )
    .unwrap();

    let sfx = tmp.path().join("package.exe");
    build(&sfx, &stub_bin(), &zip_path).unwrap();

    let dest = tmp.path().join("out");
    fs::create_dir(&dest).unwrap();
    let status = Command::new(&sfx)
        .args(["-s", "-y", "-d", dest.to_str().unwrap()])
        .status()
        .unwrap();
    assert!(status.success(), "stub exit {status}");
    assert_eq!(fs::read(dest.join("hello.txt")).unwrap(), b"hello-sfx");
}

#[cfg(windows)]
#[test]
fn built_stub_contains_the_shellzip_icon() {
    let exe = fs::read(stub_bin()).unwrap();
    let icon = include_bytes!("../assets/shellzip.ico");
    let png = icon
        .windows(8)
        .position(|window| window == [0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A])
        .expect("png inside the icon");
    let sample = &icon[png..png + 32];
    assert!(
        exe.windows(sample.len()).any(|window| window == sample),
        "the SFX stub was built without its icon"
    );
}

#[test]
fn stub_extracts_an_encrypted_zip_when_the_password_is_set() {
    let tmp = tempfile::tempdir().unwrap();
    let sfx = encrypted_sfx(tmp.path());
    let dest = tmp.path().join("out");
    fs::create_dir(&dest).unwrap();
    let status = Command::new(&sfx)
        .args(["-s", "-y", "-d", dest.to_str().unwrap()])
        .env("SZIP_PASSWORD", "sfx-test-password")
        .status()
        .unwrap();
    assert!(status.success(), "stub exit {status}");
    assert_eq!(fs::read(dest.join("hello.txt")).unwrap(), b"secret-sfx");
}

#[test]
fn silent_encrypted_stub_fails_without_a_password() {
    let tmp = tempfile::tempdir().unwrap();
    let sfx = encrypted_sfx(tmp.path());
    let dest = tmp.path().join("out");
    fs::create_dir(&dest).unwrap();
    let output = Command::new(&sfx)
        .args(["-s", "-y", "-d", dest.to_str().unwrap()])
        .env("CI", "1")
        .env_remove("SZIP_PASSWORD")
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(11));
    let err = String::from_utf8_lossy(&output.stderr);
    assert!(err.contains("password"), "{err}");
    assert!(!dest.join("hello.txt").exists());
}

#[test]
fn silent_encrypted_stub_rejects_a_wrong_password() {
    let tmp = tempfile::tempdir().unwrap();
    let sfx = encrypted_sfx(tmp.path());
    let dest = tmp.path().join("out");
    fs::create_dir(&dest).unwrap();
    let output = Command::new(&sfx)
        .args(["-s", "-y", "-d", dest.to_str().unwrap()])
        .env("CI", "1")
        .env("SZIP_PASSWORD", "wrong-password")
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(11), "{output:?}");
    assert!(!dest.join("hello.txt").exists());
}

fn encrypted_sfx(dir: &std::path::Path) -> std::path::PathBuf {
    let src = dir.join("hello.txt");
    fs::write(&src, b"secret-sfx").unwrap();
    let zip_path = dir.join("in.zip");
    add(
        &zip_path,
        &[src],
        &CompressOpts {
            level: 0,
            threads: 1,
            password: Some(Password::new("sfx-test-password").unwrap()),
            ..CompressOpts::default()
        },
        &NopProgress,
    )
    .unwrap();
    let sfx = dir.join("package.exe");
    build(&sfx, &stub_bin(), &zip_path).unwrap();
    sfx
}

fn stub_bin() -> std::path::PathBuf {
    if let Ok(path) = std::env::var("CARGO_BIN_EXE_shellzip_sfx") {
        return std::path::PathBuf::from(path);
    }
    let exe = std::env::current_exe().expect("test executable");
    let name = if cfg!(windows) {
        "shellzip-sfx.exe"
    } else {
        "shellzip-sfx"
    };
    exe.parent()
        .and_then(|deps| deps.parent())
        .map(|dir| dir.join(name))
        .expect("target directory")
}
