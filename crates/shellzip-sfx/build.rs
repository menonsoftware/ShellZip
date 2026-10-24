//! Embed the ShellZip icon in the Windows SFX stub.

use std::path::PathBuf;

fn main() {
    println!("cargo:rerun-if-changed=assets/shellzip.ico");
    if std::env::var("CARGO_CFG_TARGET_OS").ok().as_deref() != Some("windows") {
        return;
    }
    let mut res = winresource::WindowsResource::new();
    res.set_icon("assets/shellzip.ico");
    res.set("FileDescription", "ShellZip self-extractor");
    res.set("ProductName", "ShellZip");
    if let Some(toolkit) = windows_sdk_bin() {
        res.set_toolkit_path(toolkit.to_str().unwrap_or(""));
    }
    if let Err(err) = res.compile() {
        panic!("embed the ShellZip icon in the SFX stub: {err}");
    }
}

/// Directory that contains `rc.exe`, newest Windows SDK first.
fn windows_sdk_bin() -> Option<PathBuf> {
    let root = PathBuf::from(r"C:\Program Files (x86)\Windows Kits\10\bin");
    let mut versions = std::fs::read_dir(&root)
        .ok()?
        .filter_map(Result::ok)
        .collect::<Vec<_>>();
    versions.sort_by_key(|entry| std::cmp::Reverse(entry.file_name()));
    versions.into_iter().find_map(|entry| {
        let bin = entry.path().join("x64");
        bin.join("rc.exe").is_file().then_some(bin)
    })
}
