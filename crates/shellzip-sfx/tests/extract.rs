//! The stub extracts the ZIP appended to a copy of itself.

use std::fs;
use std::io;
use std::process::{Command, Output};
use std::thread;
use std::time::Duration;

use shellzip_core::{CompressOpts, NopProgress, Password, SfxPayload, add, build};

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
    build(&sfx, &stub_bin(), &zip_path, SfxPayload::Zip, &NopProgress).unwrap();

    let dest = tmp.path().join("out");
    fs::create_dir(&dest).unwrap();
    let output = run_stub(&sfx, &["-s", "-y", "-d", dest.to_str().unwrap()], &[]);
    assert!(output.status.success(), "stub exit {}", output.status);
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
    let output = run_stub(
        &sfx,
        &["-s", "-y", "-d", dest.to_str().unwrap()],
        &[("SZIP_PASSWORD", "sfx-test-password")],
    );
    assert!(output.status.success(), "stub exit {}", output.status);
    assert_eq!(fs::read(dest.join("hello.txt")).unwrap(), b"secret-sfx");
}

#[test]
fn silent_encrypted_stub_fails_without_a_password() {
    let tmp = tempfile::tempdir().unwrap();
    let sfx = encrypted_sfx(tmp.path());
    let dest = tmp.path().join("out");
    fs::create_dir(&dest).unwrap();
    let output = run_stub(
        &sfx,
        &["-s", "-y", "-d", dest.to_str().unwrap()],
        &[("CI", "1")],
    );
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
    let output = run_stub(
        &sfx,
        &["-s", "-y", "-d", dest.to_str().unwrap()],
        &[("CI", "1"), ("SZIP_PASSWORD", "wrong-password")],
    );
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
    build(&sfx, &stub_bin(), &zip_path, SfxPayload::Zip, &NopProgress).unwrap();
    sfx
}

#[test]
fn stub_extracts_its_appended_7z() {
    let tmp = tempfile::tempdir().unwrap();
    let src = tmp.path().join("hello.txt");
    fs::write(&src, b"hello-7z").unwrap();
    let archive = tmp.path().join("in.7z");
    add(
        &archive,
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
    build(
        &sfx,
        &stub_bin(),
        &archive,
        SfxPayload::SevenZ,
        &NopProgress,
    )
    .unwrap();
    let dest = tmp.path().join("out");
    fs::create_dir(&dest).unwrap();
    let output = run_stub(&sfx, &["-s", "-y", "-d", dest.to_str().unwrap()], &[]);
    assert!(output.status.success(), "stub exit {}", output.status);
    assert_eq!(fs::read(dest.join("hello.txt")).unwrap(), b"hello-7z");
}

#[test]
fn stub_extracts_a_header_encrypted_7z_with_the_password() {
    let tmp = tempfile::tempdir().unwrap();
    let src = tmp.path().join("hello.txt");
    fs::write(&src, b"hidden-7z").unwrap();
    let archive = tmp.path().join("in.7z");
    let mut opts = CompressOpts {
        level: 1,
        threads: 1,
        password: Some(Password::new("sfx-test-password").unwrap()),
        ..CompressOpts::default()
    };
    opts.sevenz.encrypt_headers = true;
    add(&archive, &[src], &opts, &NopProgress).unwrap();
    let sfx = tmp.path().join("package.exe");
    build(
        &sfx,
        &stub_bin(),
        &archive,
        SfxPayload::SevenZ,
        &NopProgress,
    )
    .unwrap();
    let dest = tmp.path().join("out");
    fs::create_dir(&dest).unwrap();
    let output = run_stub(
        &sfx,
        &["-s", "-y", "-d", dest.to_str().unwrap()],
        &[("SZIP_PASSWORD", "sfx-test-password")],
    );
    assert!(output.status.success(), "stub exit {}", output.status);
    assert_eq!(fs::read(dest.join("hello.txt")).unwrap(), b"hidden-7z");
}

/// Run the package, retrying the Linux `ETXTBSY` window after a just-written exec.
fn run_stub(sfx: &std::path::Path, args: &[&str], envs: &[(&str, &str)]) -> Output {
    let mut last_err = None;
    for _ in 0..16 {
        let mut cmd = Command::new(sfx);
        cmd.args(args).env_remove("SZIP_PASSWORD");
        for (key, value) in envs {
            cmd.env(key, value);
        }
        match cmd.output() {
            Ok(output) => return output,
            Err(err) if err.kind() == io::ErrorKind::ExecutableFileBusy => {
                last_err = Some(err);
                thread::sleep(Duration::from_millis(10));
            }
            Err(err) => panic!("run {}: {err}", sfx.display()),
        }
    }
    panic!(
        "run {}: {}",
        sfx.display(),
        last_err.expect("ETXTBSY retry")
    );
}

#[cfg(unix)]
fn install_dialog(dir: &std::path::Path) -> std::path::PathBuf {
    use std::os::unix::fs::PermissionsExt;

    let bin = dir.join("bin");
    fs::create_dir(&bin).unwrap();
    let program = bin.join("zenity");
    fs::write(
        &program,
        "\
#!/bin/sh
printf '%s\\n' \"$@\" >> \"$ZENITY_LOG\"
if [ \"${SFX_TEST_CANCEL:-}\" = 1 ]; then
    exit 1
fi
for arg in \"$@\"; do
    case \"$arg\" in
        --file-selection)
            printf '%s\\n' \"$SFX_TEST_DEST\"
            exit 0
            ;;
        --password)
            printf '%s\\n' \"${SFX_TEST_PASSWORD:-}\"
            exit 0
            ;;
    esac
done
exit 0
",
    )
    .unwrap();
    fs::set_permissions(&program, fs::Permissions::from_mode(0o755)).unwrap();
    bin
}

#[cfg(unix)]
fn dialog_envs(
    bin: &std::path::Path,
    dest: &std::path::Path,
    log: &std::path::Path,
    password: Option<&str>,
    cancel: bool,
) -> Vec<(String, String)> {
    let mut envs = vec![
        ("SZIP_SFX_DIALOG".to_string(), "1".to_string()),
        ("PATH".to_string(), bin.display().to_string()),
        ("DISPLAY".to_string(), ":1".to_string()),
        ("SFX_TEST_DEST".to_string(), dest.display().to_string()),
        ("ZENITY_LOG".to_string(), log.display().to_string()),
    ];
    if let Some(password) = password {
        envs.push(("SFX_TEST_PASSWORD".to_string(), password.to_string()));
    }
    if cancel {
        envs.push(("SFX_TEST_CANCEL".to_string(), "1".to_string()));
    }
    envs
}

#[cfg(unix)]
fn run_dialog(sfx: &std::path::Path, envs: &[(String, String)]) -> std::process::Output {
    let borrowed: Vec<(&str, &str)> = envs
        .iter()
        .map(|(key, value)| (key.as_str(), value.as_str()))
        .collect();
    run_stub(sfx, &[], &borrowed)
}

#[cfg(unix)]
#[test]
fn desktop_dialog_extracts_into_the_chosen_folder() {
    let tmp = tempfile::tempdir().unwrap();
    let src = tmp.path().join("note.txt");
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
    let sfx = tmp.path().join("package.sfx");
    build(&sfx, &stub_bin(), &zip_path, SfxPayload::Zip, &NopProgress).unwrap();
    let bin = install_dialog(tmp.path());
    let dest = tmp.path().join("out");
    fs::create_dir(&dest).unwrap();
    let log = tmp.path().join("zenity.log");
    let envs = dialog_envs(&bin, &dest, &log, None, false);
    let output = run_dialog(&sfx, &envs);
    assert!(output.status.success(), "stub exit {}", output.status);
    assert_eq!(fs::read(dest.join("note.txt")).unwrap(), b"hello-sfx");
    let recorded = fs::read_to_string(&log).unwrap();
    assert!(recorded.contains("--file-selection"), "{recorded}");
    assert!(recorded.contains("--info"), "{recorded}");
    assert!(!recorded.contains("--password"), "{recorded}");
}

#[cfg(unix)]
#[test]
fn desktop_dialog_cancel_writes_nothing() {
    let tmp = tempfile::tempdir().unwrap();
    let src = tmp.path().join("note.txt");
    fs::write(&src, b"hello-sfx").unwrap();
    let zip_path = tmp.path().join("in.zip");
    add(
        &zip_path,
        std::slice::from_ref(&src),
        &CompressOpts {
            level: 0,
            threads: 1,
            ..CompressOpts::default()
        },
        &NopProgress,
    )
    .unwrap();
    fs::remove_file(&src).unwrap();
    let sfx = tmp.path().join("package.sfx");
    build(&sfx, &stub_bin(), &zip_path, SfxPayload::Zip, &NopProgress).unwrap();
    let bin = install_dialog(tmp.path());
    let dest = tmp.path().join("out");
    fs::create_dir(&dest).unwrap();
    let log = tmp.path().join("zenity.log");
    let envs = dialog_envs(&bin, &dest, &log, None, true);
    let output = run_dialog(&sfx, &envs);
    assert!(output.status.success(), "stub exit {}", output.status);
    assert!(!src.exists());
    assert!(!dest.join("note.txt").exists());
    let recorded = fs::read_to_string(&log).unwrap();
    assert!(recorded.contains("--file-selection"), "{recorded}");
    assert!(!recorded.contains("--info"), "{recorded}");
}

#[cfg(unix)]
#[test]
fn desktop_dialog_asks_for_an_encrypted_archive() {
    let tmp = tempfile::tempdir().unwrap();
    let sfx = encrypted_sfx(tmp.path());
    let bin = install_dialog(tmp.path());
    let dest = tmp.path().join("out");
    fs::create_dir(&dest).unwrap();
    let log = tmp.path().join("zenity.log");
    let envs = dialog_envs(&bin, &dest, &log, Some("sfx-test-password"), false);
    let output = run_dialog(&sfx, &envs);
    assert!(output.status.success(), "stub exit {}", output.status);
    assert_eq!(fs::read(dest.join("hello.txt")).unwrap(), b"secret-sfx");
    let recorded = fs::read_to_string(&log).unwrap();
    assert!(recorded.contains("--password"), "{recorded}");
    assert!(!recorded.contains("sfx-test-password"), "{recorded}");
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
