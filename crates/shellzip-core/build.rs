fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    let windows = std::env::var("CARGO_CFG_TARGET_OS").ok().as_deref() == Some("windows");
    let rar = std::env::var_os("CARGO_FEATURE_RAR").is_some();
    if windows && rar {
        // UnRAR calls the registry and CryptoAPI. Its build script does not
        // pull in advapi32, so the final link fails on MSVC without this.
        println!("cargo:rustc-link-lib=advapi32");
    }
}
