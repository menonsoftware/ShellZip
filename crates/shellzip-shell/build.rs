//! Wire the module-definition file so COM exports are PRIVATE.

fn main() {
    let target_os = std::env::var("CARGO_CFG_TARGET_OS").unwrap_or_default();
    if target_os != "windows" {
        return;
    }
    let def = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("exports.def");
    println!("cargo:rerun-if-changed={}", def.display());
    // Prefer an absolute path so link.exe finds the file; strip the Win32
    // verbatim prefix that canonicalize() adds (link.exe rejects `\\?\`).
    let def = match def.canonicalize() {
        Ok(path) => {
            let text = path.to_string_lossy();
            if let Some(rest) = text.strip_prefix(r"\\?\") {
                std::path::PathBuf::from(rest)
            } else {
                path
            }
        }
        Err(_) => def,
    };
    println!("cargo:rustc-cdylib-link-arg=/DEF:{}", def.display());
}
