//! `szip` end to end, including archives written by the OS tools.

use std::fs;
use std::io::Write;
use std::process::Command;

use predicates::prelude::*;
use predicates::str::contains;

fn szip() -> assert_cmd::Command {
    let mut cmd = assert_cmd::Command::cargo_bin("szip").unwrap();
    cmd.env_remove("SZIP_PASSWORD");
    cmd
}

#[test]
fn add_list_test_extract_round_trip() {
    let tmp = tempfile::tempdir().unwrap();
    let src = tmp.path().join("src");
    fs::create_dir(&src).unwrap();
    fs::write(src.join("hello.txt"), b"hello").unwrap();
    let archive = tmp.path().join("notes.zip");
    szip()
        .args([
            "a",
            "-m",
            "1",
            archive.to_str().unwrap(),
            src.to_str().unwrap(),
        ])
        .assert()
        .success()
        .stdout(contains("created"));
    szip()
        .args(["l", archive.to_str().unwrap()])
        .assert()
        .success()
        .stdout(contains("hello.txt"));
    szip()
        .args(["t", archive.to_str().unwrap()])
        .assert()
        .success()
        .stdout(contains("OK"));
    let out = tmp.path().join("out");
    szip()
        .args([
            "x",
            "-y",
            "-o",
            out.to_str().unwrap(),
            archive.to_str().unwrap(),
        ])
        .assert()
        .success();
    assert_eq!(fs::read(out.join("src/hello.txt")).unwrap(), b"hello");
}

#[test]
fn add_to_an_existing_zip_keeps_previous_files() {
    let tmp = tempfile::tempdir().unwrap();
    let first = tmp.path().join("first.txt");
    let second = tmp.path().join("second.txt");
    fs::write(&first, b"one").unwrap();
    fs::write(&second, b"two").unwrap();
    let archive = tmp.path().join("notes.zip");
    szip()
        .args([
            "a",
            "-m",
            "1",
            archive.to_str().unwrap(),
            first.to_str().unwrap(),
        ])
        .assert()
        .success()
        .stdout(contains("created"));
    szip()
        .args([
            "a",
            "-m",
            "1",
            archive.to_str().unwrap(),
            second.to_str().unwrap(),
        ])
        .assert()
        .success()
        .stdout(contains("updated"));
    szip()
        .args(["l", archive.to_str().unwrap()])
        .assert()
        .success()
        .stdout(contains("first.txt").and(contains("second.txt")));
}

#[test]
fn zip_slip_exits_nonzero() {
    let tmp = tempfile::tempdir().unwrap();
    let archive = tmp.path().join("slip.zip");
    let file = fs::File::create(&archive).unwrap();
    let mut writer = zip::ZipWriter::new(file);
    writer
        .start_file("../evil.txt", zip::write::SimpleFileOptions::default())
        .unwrap();
    writer.write_all(b"pwned").unwrap();
    writer.finish().unwrap();
    szip()
        .args([
            "x",
            "-y",
            "-o",
            tmp.path().join("out").to_str().unwrap(),
            archive.to_str().unwrap(),
        ])
        .assert()
        .code(7);
    assert!(!tmp.path().join("evil.txt").exists());
}

#[test]
fn usage_and_help_exit_codes() {
    szip().assert().code(7);
    szip().arg("--help").assert().success();
}

#[test]
fn reads_a_tar_from_the_system_tool() {
    let tmp = tempfile::tempdir().unwrap();
    let dir = tmp.path().join("tree");
    fs::create_dir(&dir).unwrap();
    fs::write(dir.join("a.txt"), b"tar-bytes").unwrap();
    let archive = tmp.path().join("os.tar");
    let status = Command::new("tar")
        .args([
            "-cf",
            archive.to_str().unwrap(),
            "-C",
            dir.to_str().unwrap(),
            "a.txt",
        ])
        .status()
        .expect("tar");
    assert!(status.success(), "tar exited {status}");
    szip()
        .args(["t", archive.to_str().unwrap()])
        .assert()
        .success();
    let out = tmp.path().join("out");
    szip()
        .args([
            "x",
            "-y",
            "-o",
            out.to_str().unwrap(),
            archive.to_str().unwrap(),
        ])
        .assert()
        .success();
    assert_eq!(fs::read(out.join("a.txt")).unwrap(), b"tar-bytes");
}

#[cfg(windows)]
#[test]
fn reads_a_zip_from_compress_archive() {
    let tmp = tempfile::tempdir().unwrap();
    let file = tmp.path().join("note.txt");
    fs::write(&file, b"from-powershell").unwrap();
    let zip = tmp.path().join("ps.zip");
    let script = format!(
        "Compress-Archive -LiteralPath '{}' -DestinationPath '{}'",
        file.display(),
        zip.display()
    );
    let status = Command::new("powershell")
        .args(["-NoProfile", "-NonInteractive", "-Command", &script])
        .status()
        .expect("powershell");
    assert!(status.success(), "Compress-Archive exited {status}");
    szip().args(["t", zip.to_str().unwrap()]).assert().success();
    let out = tmp.path().join("out");
    szip()
        .args([
            "e",
            "-y",
            "-o",
            out.to_str().unwrap(),
            zip.to_str().unwrap(),
        ])
        .assert()
        .success();
    assert_eq!(fs::read(out.join("note.txt")).unwrap(), b"from-powershell");
}

#[test]
fn stdin_print_update_and_delete() {
    let tmp = tempfile::tempdir().unwrap();
    let archive = tmp.path().join("in.tar.zst");
    szip()
        .args(["a", "--stdin=note.txt", archive.to_str().unwrap()])
        .write_stdin(b"piped")
        .assert()
        .success()
        .stdout(contains("created"));
    szip()
        .args(["p", archive.to_str().unwrap(), "note.txt"])
        .assert()
        .success()
        .stdout(b"piped".as_slice());

    let zip = tmp.path().join("notes.zip");
    let first = tmp.path().join("a.txt");
    let second = tmp.path().join("b.txt");
    fs::write(&first, b"one").unwrap();
    fs::write(&second, b"two").unwrap();
    szip()
        .args([
            "a",
            "-m",
            "0",
            zip.to_str().unwrap(),
            first.to_str().unwrap(),
        ])
        .assert()
        .success();
    szip()
        .args([
            "u",
            "-m",
            "0",
            zip.to_str().unwrap(),
            second.to_str().unwrap(),
        ])
        .assert()
        .success()
        .stdout(contains("updated"));
    szip()
        .args(["d", zip.to_str().unwrap(), "b.txt"])
        .assert()
        .success()
        .stdout(contains("deleted 1"));
    szip()
        .args(["l", zip.to_str().unwrap()])
        .assert()
        .success()
        .stdout(contains("a.txt"))
        .stdout(predicates::str::contains("b.txt").not());
}

#[test]
fn technical_listing_layout() {
    let tmp = tempfile::tempdir().unwrap();
    let file = tmp.path().join("hello.txt");
    fs::write(&file, b"hello").unwrap();
    let archive = tmp.path().join("notes.zip");
    szip()
        .args([
            "a",
            "-m",
            "0",
            archive.to_str().unwrap(),
            file.to_str().unwrap(),
        ])
        .assert()
        .success();
    let output = szip()
        .args(["lt", "--htb", archive.to_str().unwrap()])
        .output()
        .unwrap();
    assert!(output.status.success());
    let text = String::from_utf8(output.stdout).unwrap();
    let body = text.split_once('\n').map(|(_, rest)| rest).unwrap_or(&text);
    insta::with_settings!({
        filters => vec![
            (r"\d{4}-\d{2}-\d{2} \d{2}:\d{2}", "[time]"),
            (r"[dl-][0-7]{4}", "[attrs]"),
        ]
    }, {
        insta::assert_snapshot!(body);
    });
}

#[test]
fn log_file_receives_debug_events() {
    let tmp = tempfile::tempdir().unwrap();
    let file = tmp.path().join("a.txt");
    fs::write(&file, b"a").unwrap();
    let archive = tmp.path().join("a.zip");
    let log = tmp.path().join("run.log");
    szip()
        .args([
            "-v",
            &format!("--ilog={}", log.display()),
            "a",
            "-m",
            "0",
            archive.to_str().unwrap(),
            file.to_str().unwrap(),
        ])
        .assert()
        .success();
    let text = fs::read_to_string(&log).unwrap();
    assert!(text.contains("archive written"), "log was {text:?}");
}

#[test]
fn help_explains_that_zip_encryption_leaves_names_visible() {
    szip()
        .arg("--help")
        .assert()
        .success()
        .stdout(contains("Names stay visible"))
        .stdout(contains("RAR is never created"));
}

#[test]
fn zip_password_round_trip_and_exit_11() {
    let tmp = tempfile::tempdir().unwrap();
    let file = tmp.path().join("hello.txt");
    fs::write(&file, b"hello\n").unwrap();
    let archive = tmp.path().join("secret.zip");
    szip()
        .args([
            "a",
            "-m",
            "1",
            "-pszip-fixture",
            archive.to_str().unwrap(),
            file.to_str().unwrap(),
        ])
        .assert()
        .success();
    szip()
        .args(["l", archive.to_str().unwrap()])
        .assert()
        .success()
        .stdout(contains("hello.txt"));
    szip()
        .args([
            "x",
            "-y",
            "-o",
            tmp.path().join("missing").to_str().unwrap(),
            archive.to_str().unwrap(),
        ])
        .assert()
        .code(11)
        .stderr(contains("password required"));
    szip()
        .args([
            "x",
            "-y",
            "-o",
            tmp.path().join("wrong").to_str().unwrap(),
            "-pwrong-fixture",
            archive.to_str().unwrap(),
        ])
        .assert()
        .code(11)
        .stderr(contains("incorrect password"));
    let out = tmp.path().join("out");
    szip()
        .args([
            "x",
            "-y",
            "-o",
            out.to_str().unwrap(),
            "-pszip-fixture",
            archive.to_str().unwrap(),
        ])
        .assert()
        .success();
    assert_eq!(fs::read(out.join("hello.txt")).unwrap(), b"hello\n");
    szip()
        .args([
            "x",
            "-y",
            "--mes",
            "-o",
            tmp.path().join("skip").to_str().unwrap(),
            archive.to_str().unwrap(),
        ])
        .assert()
        .code(1)
        .stderr(contains("skipped encrypted"));
}

#[test]
fn password_from_the_environment_is_not_written_to_the_log() {
    let tmp = tempfile::tempdir().unwrap();
    let file = tmp.path().join("hello.txt");
    fs::write(&file, b"hello\n").unwrap();
    let archive = tmp.path().join("secret.zip");
    let log = tmp.path().join("run.log");
    let mut cmd = assert_cmd::Command::cargo_bin("szip").unwrap();
    cmd.env("SZIP_PASSWORD", "szip-log-secret")
        .args([
            "-v",
            &format!("--ilog={}", log.display()),
            "a",
            "-m",
            "0",
            archive.to_str().unwrap(),
            file.to_str().unwrap(),
        ])
        .assert()
        .success()
        .stderr(contains("szip-log-secret").not());
    let text = fs::read_to_string(&log).unwrap();
    assert!(!text.contains("szip-log-secret"), "{text}");
    let out = tmp.path().join("out");
    let mut extract = assert_cmd::Command::cargo_bin("szip").unwrap();
    extract
        .env("SZIP_PASSWORD", "szip-log-secret")
        .args([
            "x",
            "-y",
            "-o",
            out.to_str().unwrap(),
            archive.to_str().unwrap(),
        ])
        .assert()
        .success();
    assert_eq!(fs::read(out.join("hello.txt")).unwrap(), b"hello\n");
}

#[test]
fn rar_is_not_created() {
    let tmp = tempfile::tempdir().unwrap();
    let file = tmp.path().join("hello.txt");
    fs::write(&file, b"hello\n").unwrap();
    szip()
        .args([
            "a",
            "-m",
            "0",
            tmp.path().join("notes.rar").to_str().unwrap(),
            file.to_str().unwrap(),
        ])
        .assert()
        .failure()
        .stderr(contains("not"));
}

#[test]
fn sevenz_add_list_test_extract_round_trip() {
    let tmp = tempfile::tempdir().unwrap();
    let file = tmp.path().join("hello.txt");
    fs::write(&file, b"hello\n").unwrap();
    let archive = tmp.path().join("notes.7z");
    szip()
        .args([
            "a",
            "-m",
            "1",
            archive.to_str().unwrap(),
            file.to_str().unwrap(),
        ])
        .assert()
        .success()
        .stdout(contains("created"));
    szip()
        .args(["l", archive.to_str().unwrap()])
        .assert()
        .success()
        .stdout(contains("7Z"))
        .stdout(contains("hello.txt"));
    szip()
        .args(["t", archive.to_str().unwrap()])
        .assert()
        .success();
    let out = tmp.path().join("out");
    szip()
        .args([
            "x",
            "-y",
            "-o",
            out.to_str().unwrap(),
            archive.to_str().unwrap(),
        ])
        .assert()
        .success();
    assert_eq!(fs::read(out.join("hello.txt")).unwrap(), b"hello\n");
}

#[test]
fn sevenz_header_encryption_hides_names_and_exits_11() {
    let tmp = tempfile::tempdir().unwrap();
    let file = tmp.path().join("hello.txt");
    fs::write(&file, b"hello\n").unwrap();
    let archive = tmp.path().join("secret.7z");
    szip()
        .args([
            "a",
            "-m",
            "1",
            "-hpsecret",
            archive.to_str().unwrap(),
            file.to_str().unwrap(),
        ])
        .assert()
        .success();
    szip()
        .args(["l", archive.to_str().unwrap()])
        .assert()
        .code(11)
        .stdout(contains("hello.txt").not());
    szip()
        .args(["l", "-pwrong", archive.to_str().unwrap()])
        .assert()
        .code(11);
    szip()
        .args(["l", "-psecret", archive.to_str().unwrap()])
        .assert()
        .success()
        .stdout(contains("hello.txt"));
}

#[test]
fn sevenz_update_freshen_delete_rewrite() {
    let tmp = tempfile::tempdir().unwrap();
    let keep = tmp.path().join("keep.txt");
    let extra = tmp.path().join("extra.txt");
    fs::write(&keep, b"old").unwrap();
    fs::write(&extra, b"extra").unwrap();
    let archive = tmp.path().join("notes.7z");
    szip()
        .args([
            "a",
            "-m",
            "0",
            archive.to_str().unwrap(),
            keep.to_str().unwrap(),
        ])
        .assert()
        .success();
    fs::write(&keep, b"new").unwrap();
    let later = std::time::SystemTime::now() + std::time::Duration::from_secs(10);
    fs::File::options()
        .write(true)
        .open(&keep)
        .unwrap()
        .set_modified(later)
        .unwrap();
    szip()
        .args([
            "f",
            archive.to_str().unwrap(),
            keep.to_str().unwrap(),
            extra.to_str().unwrap(),
        ])
        .assert()
        .success();
    let out = tmp.path().join("out");
    szip()
        .args([
            "x",
            "-y",
            "-o",
            out.to_str().unwrap(),
            archive.to_str().unwrap(),
        ])
        .assert()
        .success();
    assert_eq!(fs::read(out.join("keep.txt")).unwrap(), b"new");
    assert!(
        !out.join("extra.txt").exists(),
        "freshen must not add files"
    );
    szip()
        .args(["d", archive.to_str().unwrap(), "keep.txt"])
        .assert()
        .success()
        .stdout(contains("deleted"));
    szip()
        .args(["l", archive.to_str().unwrap()])
        .assert()
        .success()
        .stdout(contains("keep.txt").not());
}

#[test]
fn sevenz_method_on_zip_is_a_usage_error() {
    let tmp = tempfile::tempdir().unwrap();
    let file = tmp.path().join("hello.txt");
    fs::write(&file, b"hello\n").unwrap();
    szip()
        .args([
            "a",
            "--method",
            "lzma2",
            tmp.path().join("notes.zip").to_str().unwrap(),
            file.to_str().unwrap(),
        ])
        .assert()
        .failure()
        .stderr(contains("7z method"));
}

#[test]
fn sevenz_technical_listing() {
    let tmp = tempfile::tempdir().unwrap();
    let file = tmp.path().join("hello.txt");
    fs::write(&file, b"hello").unwrap();
    let archive = tmp.path().join("notes.7z");
    szip()
        .args([
            "a",
            "-m",
            "0",
            "--no-solid",
            archive.to_str().unwrap(),
            file.to_str().unwrap(),
        ])
        .assert()
        .success();
    let output = szip()
        .args(["lt", "--htb", archive.to_str().unwrap()])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let text = String::from_utf8(output.stdout).unwrap();
    let body = text.split_once('\n').map(|(_, rest)| rest).unwrap_or(&text);
    insta::with_settings!({
        filters => vec![
            (r"\d{4}-\d{2}-\d{2} \d{2}:\d{2}", "[time]"),
            (r"[dl-][0-7]{4}", "[attrs]"),
            (r"[0-9a-f]{64}", "[blake]"),
        ]
    }, {
        insta::assert_snapshot!(body);
    });
}

#[test]
fn sfx_wrap_list_test_extract_and_unwrap() {
    let tmp = tempfile::tempdir().unwrap();
    let src = tmp.path().join("hello.txt");
    fs::write(&src, b"hello-sfx").unwrap();
    let zip = tmp.path().join("notes.zip");
    szip()
        .args(["a", "-m", "0", zip.to_str().unwrap(), src.to_str().unwrap()])
        .assert()
        .success();
    let stub = tmp.path().join("stub.bin");
    fs::write(&stub, b"MZ-test-stub").unwrap();
    let sfx = tmp.path().join("notes.exe");
    szip()
        .args([
            "s",
            "--sfx-stub",
            stub.to_str().unwrap(),
            zip.to_str().unwrap(),
            "-o",
            sfx.to_str().unwrap(),
        ])
        .assert()
        .success()
        .stdout(contains("created"));
    szip()
        .args(["l", sfx.to_str().unwrap()])
        .assert()
        .success()
        .stdout(contains("hello.txt"));
    szip()
        .args(["t", sfx.to_str().unwrap()])
        .assert()
        .success()
        .stdout(contains("OK"));
    let out = tmp.path().join("out");
    szip()
        .args([
            "x",
            "-y",
            "-o",
            out.to_str().unwrap(),
            sfx.to_str().unwrap(),
        ])
        .assert()
        .success();
    assert_eq!(fs::read(out.join("hello.txt")).unwrap(), b"hello-sfx");
    let back = tmp.path().join("back.zip");
    szip()
        .args(["s", sfx.to_str().unwrap(), "-o", back.to_str().unwrap()])
        .assert()
        .success()
        .stdout(contains("extracted"));
    szip()
        .args(["l", back.to_str().unwrap()])
        .assert()
        .success()
        .stdout(contains("hello.txt"));
}

#[test]
fn sfx_7z_wrap_list_test_extract_and_unwrap() {
    let tmp = tempfile::tempdir().unwrap();
    let src = tmp.path().join("hello.txt");
    fs::write(&src, b"hello-7z").unwrap();
    let archive = tmp.path().join("notes.7z");
    szip()
        .args([
            "a",
            "-m",
            "0",
            archive.to_str().unwrap(),
            src.to_str().unwrap(),
        ])
        .assert()
        .success();
    let stub = tmp.path().join("stub.bin");
    fs::write(&stub, b"MZ-test-stub").unwrap();
    let sfx = tmp.path().join("notes.exe");
    szip()
        .args([
            "s",
            "--sfx-stub",
            stub.to_str().unwrap(),
            archive.to_str().unwrap(),
            "-o",
            sfx.to_str().unwrap(),
        ])
        .assert()
        .success()
        .stdout(contains("created"));
    szip()
        .args(["l", sfx.to_str().unwrap()])
        .assert()
        .success()
        .stdout(contains("hello.txt"));
    szip().args(["t", sfx.to_str().unwrap()]).assert().success();
    let out = tmp.path().join("out");
    szip()
        .args([
            "x",
            "-y",
            "-o",
            out.to_str().unwrap(),
            sfx.to_str().unwrap(),
        ])
        .assert()
        .success();
    assert_eq!(fs::read(out.join("hello.txt")).unwrap(), b"hello-7z");
    let back = tmp.path().join("back.7z");
    szip()
        .args(["s", sfx.to_str().unwrap(), "-o", back.to_str().unwrap()])
        .assert()
        .success()
        .stdout(contains("extracted"));
    szip()
        .args(["l", back.to_str().unwrap()])
        .assert()
        .success()
        .stdout(contains("hello.txt"));
}

#[test]
fn add_sfx_writes_a_listable_package() {
    let tmp = tempfile::tempdir().unwrap();
    let src = tmp.path().join("note.txt");
    fs::write(&src, b"note").unwrap();
    let stub = tmp.path().join("stub.bin");
    fs::write(&stub, b"MZ-stub").unwrap();
    let dest = tmp.path().join("pack.exe");
    szip()
        .args([
            "a",
            "-m",
            "0",
            "--sfx",
            "--sfx-stub",
            stub.to_str().unwrap(),
            dest.to_str().unwrap(),
            src.to_str().unwrap(),
        ])
        .assert()
        .success()
        .stdout(contains("created"));
    szip()
        .args(["l", dest.to_str().unwrap()])
        .assert()
        .success()
        .stdout(contains("note.txt"));
}
