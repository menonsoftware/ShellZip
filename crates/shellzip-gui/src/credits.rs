//! Crates linked into the ShellZip window, with the license each one declares.
//!
//! Built from `cargo tree -p shellzip-gui -e normal` for Windows, Linux, and
//! macOS. Workspace crates are omitted; they use the ShellZip license.
//! `unrar_sys` vendors the UnRAR sources, so its line is the clarified
//! expression from `deny.toml`, not the MIT-only metadata of the wrapper.
//!
//! Regenerate with `python xtask/gen_credits.py`.

/// One third-party crate linked into this program.
pub struct Credit {
    /// Crate name.
    pub name: &'static str,
    /// Version resolved in `Cargo.lock`.
    pub version: &'static str,
    /// SPDX license expression.
    pub license: &'static str,
}

/// Packages linked into the build for this operating system.
pub fn credits() -> &'static [Credit] {
    #[cfg(windows)]
    {
        WINDOWS
    }
    #[cfg(target_os = "linux")]
    {
        LINUX
    }
    #[cfg(target_os = "macos")]
    {
        MACOS
    }
    #[cfg(not(any(windows, target_os = "linux", target_os = "macos")))]
    {
        LINUX
    }
}

#[cfg_attr(not(windows), allow(dead_code))] // listed in the About box on Windows
pub static WINDOWS: &[Credit] = &[
    Credit {
        name: "ab_glyph",
        version: "0.2.32",
        license: "Apache-2.0",
    },
    Credit {
        name: "ab_glyph_rasterizer",
        version: "0.1.10",
        license: "Apache-2.0",
    },
    Credit {
        name: "accesskit",
        version: "0.19.0",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "accesskit_consumer",
        version: "0.28.0",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "accesskit_windows",
        version: "0.27.0",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "accesskit_winit",
        version: "0.27.0",
        license: "Apache-2.0",
    },
    Credit {
        name: "adler2",
        version: "2.0.1",
        license: "0BSD OR MIT OR Apache-2.0",
    },
    Credit {
        name: "aes",
        version: "0.8.4",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "ahash",
        version: "0.8.12",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "aho-corasick",
        version: "1.1.5",
        license: "Unlicense OR MIT",
    },
    Credit {
        name: "anyhow",
        version: "1.0.104",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "arboard",
        version: "3.7.0",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "arrayref",
        version: "0.3.9",
        license: "BSD-2-Clause",
    },
    Credit {
        name: "arrayvec",
        version: "0.7.8",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "base64",
        version: "0.22.1",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "base64",
        version: "0.23.1",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "bitflags",
        version: "1.3.2",
        license: "MIT/Apache-2.0",
    },
    Credit {
        name: "bitflags",
        version: "2.13.2",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "blake2s_simd",
        version: "1.0.5",
        license: "MIT",
    },
    Credit {
        name: "block-buffer",
        version: "0.10.4",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "block-buffer",
        version: "0.12.1",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "block-padding",
        version: "0.3.3",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "bstr",
        version: "1.13.1",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "bumpalo",
        version: "3.20.3",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "bytemuck",
        version: "1.25.2",
        license: "Zlib OR Apache-2.0 OR MIT",
    },
    Credit {
        name: "bytemuck_derive",
        version: "1.12.1",
        license: "Zlib OR Apache-2.0 OR MIT",
    },
    Credit {
        name: "byteorder-lite",
        version: "0.1.0",
        license: "Unlicense OR MIT",
    },
    Credit {
        name: "bytes",
        version: "1.12.1",
        license: "MIT",
    },
    Credit {
        name: "bzip2",
        version: "0.6.1",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "cbc",
        version: "0.1.2",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "cfg-if",
        version: "1.0.5",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "chacha20",
        version: "0.10.2",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "chrono",
        version: "0.4.45",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "cipher",
        version: "0.4.4",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "clipboard-win",
        version: "5.4.3",
        license: "BSL-1.0",
    },
    Credit {
        name: "color_quant",
        version: "1.1.0",
        license: "MIT",
    },
    Credit {
        name: "const-oid",
        version: "0.10.2",
        license: "Apache-2.0 OR MIT",
    },
    Credit {
        name: "constant_time_eq",
        version: "0.3.1",
        license: "CC0-1.0 OR MIT-0 OR Apache-2.0",
    },
    Credit {
        name: "constant_time_eq",
        version: "0.4.2",
        license: "CC0-1.0 OR MIT-0 OR Apache-2.0",
    },
    Credit {
        name: "cpufeatures",
        version: "0.2.17",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "cpufeatures",
        version: "0.3.1",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "crc32fast",
        version: "1.5.3",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "crc64fast",
        version: "1.1.0",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "crossbeam-deque",
        version: "0.8.8",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "crossbeam-epoch",
        version: "0.9.21",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "crossbeam-utils",
        version: "0.8.23",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "crypto-common",
        version: "0.1.7",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "crypto-common",
        version: "0.2.2",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "cursor-icon",
        version: "1.2.0",
        license: "MIT OR Apache-2.0 OR Zlib",
    },
    Credit {
        name: "data-url",
        version: "0.3.2",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "deranged",
        version: "0.5.9",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "derivative",
        version: "2.2.0",
        license: "MIT/Apache-2.0",
    },
    Credit {
        name: "digest",
        version: "0.10.7",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "digest",
        version: "0.11.3",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "directories",
        version: "6.0.0",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "dirs-sys",
        version: "0.5.0",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "displaydoc",
        version: "0.2.7",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "document-features",
        version: "0.2.12",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "dpi",
        version: "0.1.2",
        license: "Apache-2.0 AND MIT",
    },
    Credit {
        name: "ecolor",
        version: "0.32.3",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "eframe",
        version: "0.32.3",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "egui",
        version: "0.32.3",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "egui-winit",
        version: "0.32.3",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "egui_extras",
        version: "0.32.3",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "egui_glow",
        version: "0.32.3",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "ehttp",
        version: "0.5.0",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "either",
        version: "1.19.0",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "emath",
        version: "0.32.3",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "enum-map",
        version: "2.7.3",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "enum-map-derive",
        version: "0.17.0",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "epaint",
        version: "0.32.3",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "epaint_default_fonts",
        version: "0.32.3",
        license: "(MIT OR Apache-2.0) AND OFL-1.1 AND Ubuntu-font-1.0",
    },
    Credit {
        name: "equivalent",
        version: "1.0.3",
        license: "Apache-2.0 OR MIT",
    },
    Credit {
        name: "error-code",
        version: "3.4.0",
        license: "BSL-1.0",
    },
    Credit {
        name: "fdeflate",
        version: "0.3.7",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "file_icon_provider",
        version: "1.0.2",
        license: "MIT",
    },
    Credit {
        name: "filetime",
        version: "0.2.29",
        license: "MIT/Apache-2.0",
    },
    Credit {
        name: "flate2",
        version: "1.1.10",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "float-cmp",
        version: "0.9.0",
        license: "MIT",
    },
    Credit {
        name: "foldhash",
        version: "0.1.5",
        license: "Zlib",
    },
    Credit {
        name: "form_urlencoded",
        version: "1.2.2",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "generic-array",
        version: "0.14.7",
        license: "MIT",
    },
    Credit {
        name: "getrandom",
        version: "0.2.17",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "getrandom",
        version: "0.3.4",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "getrandom",
        version: "0.4.3",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "gif",
        version: "0.14.2",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "glob",
        version: "0.3.4",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "globset",
        version: "0.4.20",
        license: "Unlicense OR MIT",
    },
    Credit {
        name: "glow",
        version: "0.16.0",
        license: "MIT OR Apache-2.0 OR Zlib",
    },
    Credit {
        name: "glutin",
        version: "0.32.3",
        license: "Apache-2.0",
    },
    Credit {
        name: "glutin-winit",
        version: "0.5.0",
        license: "MIT",
    },
    Credit {
        name: "glutin_egl_sys",
        version: "0.7.1",
        license: "Apache-2.0",
    },
    Credit {
        name: "glutin_wgl_sys",
        version: "0.6.1",
        license: "Apache-2.0",
    },
    Credit {
        name: "hashbrown",
        version: "0.15.5",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "hashbrown",
        version: "0.17.1",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "hmac",
        version: "0.12.1",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "http",
        version: "1.5.0",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "httparse",
        version: "1.10.1",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "hybrid-array",
        version: "0.4.15",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "icu_collections",
        version: "2.3.0",
        license: "Unicode-3.0",
    },
    Credit {
        name: "icu_locale_core",
        version: "2.3.0",
        license: "Unicode-3.0",
    },
    Credit {
        name: "icu_normalizer",
        version: "2.3.0",
        license: "Unicode-3.0",
    },
    Credit {
        name: "icu_normalizer_data",
        version: "2.3.0",
        license: "Unicode-3.0",
    },
    Credit {
        name: "icu_properties",
        version: "2.3.0",
        license: "Unicode-3.0",
    },
    Credit {
        name: "icu_properties_data",
        version: "2.3.0",
        license: "Unicode-3.0",
    },
    Credit {
        name: "icu_provider",
        version: "2.3.1",
        license: "Unicode-3.0",
    },
    Credit {
        name: "idna",
        version: "1.1.0",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "idna_adapter",
        version: "1.2.2",
        license: "Apache-2.0 OR MIT",
    },
    Credit {
        name: "image",
        version: "0.25.10",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "image-webp",
        version: "0.2.4",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "imagesize",
        version: "0.13.0",
        license: "MIT",
    },
    Credit {
        name: "indexmap",
        version: "2.14.2",
        license: "Apache-2.0 OR MIT",
    },
    Credit {
        name: "inout",
        version: "0.1.4",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "itoa",
        version: "1.0.18",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "kurbo",
        version: "0.11.3",
        license: "Apache-2.0 OR MIT",
    },
    Credit {
        name: "lazy_static",
        version: "1.5.1",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "libbz2-rs-sys",
        version: "0.2.5",
        license: "bzip2-1.0.6",
    },
    Credit {
        name: "libc",
        version: "0.2.191",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "libloading",
        version: "0.8.9",
        license: "ISC",
    },
    Credit {
        name: "litemap",
        version: "0.8.3",
        license: "Unicode-3.0",
    },
    Credit {
        name: "litrs",
        version: "1.0.1",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "lock_api",
        version: "0.4.14",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "log",
        version: "0.4.34",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "lzma-rust2",
        version: "0.15.8",
        license: "Apache-2.0",
    },
    Credit {
        name: "lzma-sys",
        version: "0.1.20",
        license: "MIT/Apache-2.0",
    },
    Credit {
        name: "memchr",
        version: "2.8.3",
        license: "Unlicense OR MIT",
    },
    Credit {
        name: "memoffset",
        version: "0.9.1",
        license: "MIT",
    },
    Credit {
        name: "mime",
        version: "0.3.17",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "mime_guess2",
        version: "2.3.1",
        license: "MIT",
    },
    Credit {
        name: "miniz_oxide",
        version: "0.8.9",
        license: "MIT OR Zlib OR Apache-2.0",
    },
    Credit {
        name: "miniz_oxide",
        version: "0.9.1",
        license: "MIT OR Zlib OR Apache-2.0",
    },
    Credit {
        name: "moxcms",
        version: "0.8.1",
        license: "BSD-3-Clause OR Apache-2.0",
    },
    Credit {
        name: "nohash-hasher",
        version: "0.2.0",
        license: "Apache-2.0 OR MIT",
    },
    Credit {
        name: "normpath",
        version: "1.5.2",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "ntapi",
        version: "0.4.3",
        license: "Apache-2.0 OR MIT",
    },
    Credit {
        name: "num-conv",
        version: "0.2.2",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "num-traits",
        version: "0.2.19",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "once_cell",
        version: "1.21.4",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "option-ext",
        version: "0.2.0",
        license: "MPL-2.0",
    },
    Credit {
        name: "owned_ttf_parser",
        version: "0.25.1",
        license: "Apache-2.0",
    },
    Credit {
        name: "parking_lot",
        version: "0.12.5",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "parking_lot_core",
        version: "0.9.12",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "pbkdf2",
        version: "0.12.2",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "percent-encoding",
        version: "2.3.2",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "pico-args",
        version: "0.5.0",
        license: "MIT",
    },
    Credit {
        name: "pin-project-lite",
        version: "0.2.17",
        license: "Apache-2.0 OR MIT",
    },
    Credit {
        name: "png",
        version: "0.17.16",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "png",
        version: "0.18.1",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "potential_utf",
        version: "0.1.6",
        license: "Unicode-3.0",
    },
    Credit {
        name: "powerfmt",
        version: "0.2.1",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "ppmd-rust",
        version: "1.5.2",
        license: "CC0-1.0 OR MIT-0",
    },
    Credit {
        name: "proc-macro2",
        version: "1.0.107",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "profiling",
        version: "1.0.18",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "pxfm",
        version: "0.1.30",
        license: "BSD-3-Clause OR Apache-2.0",
    },
    Credit {
        name: "quick-error",
        version: "2.0.1",
        license: "MIT/Apache-2.0",
    },
    Credit {
        name: "quote",
        version: "1.0.47",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "rand",
        version: "0.10.3",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "rand_core",
        version: "0.10.1",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "raw-window-handle",
        version: "0.6.2",
        license: "MIT OR Apache-2.0 OR Zlib",
    },
    Credit {
        name: "rayon",
        version: "1.12.0",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "rayon-core",
        version: "1.13.0",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "regex",
        version: "1.13.1",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "regex-automata",
        version: "0.4.18",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "regex-syntax",
        version: "0.8.11",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "resvg",
        version: "0.45.1",
        license: "Apache-2.0 OR MIT",
    },
    Credit {
        name: "rfd",
        version: "0.15.4",
        license: "MIT",
    },
    Credit {
        name: "rgb",
        version: "0.8.53",
        license: "MIT",
    },
    Credit {
        name: "ring",
        version: "0.17.14",
        license: "Apache-2.0 AND ISC",
    },
    Credit {
        name: "roxmltree",
        version: "0.20.0",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "rustls",
        version: "0.23.46",
        license: "Apache-2.0 OR ISC OR MIT",
    },
    Credit {
        name: "rustls-pki-types",
        version: "1.15.1",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "rustls-webpki",
        version: "0.103.16",
        license: "ISC",
    },
    Credit {
        name: "same-file",
        version: "1.0.6",
        license: "Unlicense/MIT",
    },
    Credit {
        name: "scopeguard",
        version: "1.2.0",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "semver",
        version: "1.0.28",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "serde",
        version: "1.0.229",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "serde_core",
        version: "1.0.229",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "serde_derive",
        version: "1.0.229",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "serde_json",
        version: "1.0.154",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "serde_spanned",
        version: "1.1.2",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "sha1",
        version: "0.10.7",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "sha1",
        version: "0.11.0",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "sha2",
        version: "0.10.9",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "sha2",
        version: "0.11.1",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "simd-adler32",
        version: "0.3.10",
        license: "MIT",
    },
    Credit {
        name: "simplecss",
        version: "0.2.2",
        license: "Apache-2.0 OR MIT",
    },
    Credit {
        name: "siphasher",
        version: "1.0.4",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "smallvec",
        version: "1.16.3",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "smol_str",
        version: "0.2.2",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "stable_deref_trait",
        version: "1.2.1",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "static_assertions",
        version: "1.1.0",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "strict-num",
        version: "0.1.1",
        license: "MIT",
    },
    Credit {
        name: "subtle",
        version: "2.6.1",
        license: "BSD-3-Clause",
    },
    Credit {
        name: "svgtypes",
        version: "0.15.3",
        license: "Apache-2.0 OR MIT",
    },
    Credit {
        name: "syn",
        version: "1.0.109",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "syn",
        version: "2.0.119",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "syn",
        version: "3.0.9",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "synstructure",
        version: "0.14.0",
        license: "MIT",
    },
    Credit {
        name: "sysinfo",
        version: "0.32.1",
        license: "MIT",
    },
    Credit {
        name: "sysinfo",
        version: "0.40.1",
        license: "MIT",
    },
    Credit {
        name: "systemicons",
        version: "0.9.13",
        license: "MIT",
    },
    Credit {
        name: "tar",
        version: "0.4.46",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "thiserror",
        version: "2.0.21",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "thiserror-impl",
        version: "2.0.21",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "time",
        version: "0.3.55",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "time-core",
        version: "0.1.9",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "tiny-skia",
        version: "0.11.4",
        license: "BSD-3-Clause",
    },
    Credit {
        name: "tiny-skia-path",
        version: "0.11.4",
        license: "BSD-3-Clause",
    },
    Credit {
        name: "tinystr",
        version: "0.8.4",
        license: "Unicode-3.0",
    },
    Credit {
        name: "toml",
        version: "1.1.8+spec-1.1.0",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "toml_datetime",
        version: "1.1.2+spec-1.1.0",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "toml_parser",
        version: "1.1.5+spec-1.1.0",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "toml_writer",
        version: "1.1.3+spec-1.1.0",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "tracing",
        version: "0.1.44",
        license: "MIT",
    },
    Credit {
        name: "tracing-attributes",
        version: "0.1.31",
        license: "MIT",
    },
    Credit {
        name: "tracing-core",
        version: "0.1.36",
        license: "MIT",
    },
    Credit {
        name: "trash",
        version: "5.2.9",
        license: "MIT",
    },
    Credit {
        name: "ttf-parser",
        version: "0.25.1",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "typed-path",
        version: "0.12.4",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "typenum",
        version: "1.20.1",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "unicase",
        version: "2.10.0",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "unicode-ident",
        version: "1.0.27",
        license: "(MIT OR Apache-2.0) AND Unicode-3.0",
    },
    Credit {
        name: "unicode-segmentation",
        version: "1.13.3",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "unrar",
        version: "0.5.8",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "unrar_sys",
        version: "0.5.8",
        license: "(MIT OR Apache-2.0) AND LicenseRef-UnRAR",
    },
    Credit {
        name: "untrusted",
        version: "0.9.0",
        license: "ISC",
    },
    Credit {
        name: "ureq",
        version: "2.12.1",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "ureq",
        version: "3.5.1",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "ureq-proto",
        version: "0.7.1",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "url",
        version: "2.5.8",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "usvg",
        version: "0.45.1",
        license: "Apache-2.0 OR MIT",
    },
    Credit {
        name: "utf8-zero",
        version: "0.8.1",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "utf8_iter",
        version: "1.0.4",
        license: "Apache-2.0 OR MIT",
    },
    Credit {
        name: "uuid",
        version: "1.28.0",
        license: "Apache-2.0 OR MIT",
    },
    Credit {
        name: "velopack",
        version: "1.2.161",
        license: "MIT",
    },
    Credit {
        name: "walkdir",
        version: "2.5.0",
        license: "Unlicense/MIT",
    },
    Credit {
        name: "web-time",
        version: "1.1.0",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "webbrowser",
        version: "1.2.4",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "webpki-roots",
        version: "0.26.11",
        license: "CDLA-Permissive-2.0",
    },
    Credit {
        name: "webpki-roots",
        version: "1.0.10",
        license: "CDLA-Permissive-2.0",
    },
    Credit {
        name: "weezl",
        version: "0.1.12",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "widestring",
        version: "1.2.1",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "winapi",
        version: "0.3.9",
        license: "MIT/Apache-2.0",
    },
    Credit {
        name: "winapi-util",
        version: "0.1.11",
        license: "Unlicense OR MIT",
    },
    Credit {
        name: "windows",
        version: "0.57.0",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "windows",
        version: "0.58.0",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "windows",
        version: "0.61.3",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "windows",
        version: "0.62.2",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "windows-collections",
        version: "0.2.0",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "windows-collections",
        version: "0.3.2",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "windows-core",
        version: "0.57.0",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "windows-core",
        version: "0.58.0",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "windows-core",
        version: "0.61.2",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "windows-core",
        version: "0.62.2",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "windows-future",
        version: "0.2.1",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "windows-future",
        version: "0.3.2",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "windows-icons",
        version: "0.4.0",
        license: "MIT",
    },
    Credit {
        name: "windows-implement",
        version: "0.57.0",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "windows-implement",
        version: "0.58.0",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "windows-implement",
        version: "0.60.2",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "windows-interface",
        version: "0.57.0",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "windows-interface",
        version: "0.58.0",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "windows-interface",
        version: "0.59.3",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "windows-link",
        version: "0.1.3",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "windows-link",
        version: "0.2.1",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "windows-numerics",
        version: "0.2.0",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "windows-numerics",
        version: "0.3.1",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "windows-result",
        version: "0.1.2",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "windows-result",
        version: "0.2.0",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "windows-result",
        version: "0.3.4",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "windows-result",
        version: "0.4.1",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "windows-strings",
        version: "0.1.0",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "windows-strings",
        version: "0.4.2",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "windows-strings",
        version: "0.5.1",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "windows-sys",
        version: "0.52.0",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "windows-sys",
        version: "0.59.0",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "windows-sys",
        version: "0.61.2",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "windows-targets",
        version: "0.52.6",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "windows-threading",
        version: "0.1.0",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "windows-threading",
        version: "0.2.1",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "windows_x86_64_msvc",
        version: "0.52.6",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "winit",
        version: "0.30.13",
        license: "Apache-2.0",
    },
    Credit {
        name: "winnow",
        version: "1.0.4",
        license: "MIT",
    },
    Credit {
        name: "winreg",
        version: "0.55.0",
        license: "MIT",
    },
    Credit {
        name: "writeable",
        version: "0.6.4",
        license: "Unicode-3.0",
    },
    Credit {
        name: "xml",
        version: "1.4.0",
        license: "MIT",
    },
    Credit {
        name: "xmlwriter",
        version: "0.1.0",
        license: "MIT",
    },
    Credit {
        name: "xz2",
        version: "0.1.7",
        license: "MIT/Apache-2.0",
    },
    Credit {
        name: "yoke",
        version: "0.8.3",
        license: "Unicode-3.0",
    },
    Credit {
        name: "yoke-derive",
        version: "0.8.4",
        license: "Unicode-3.0",
    },
    Credit {
        name: "zerocopy",
        version: "0.8.63",
        license: "BSD-2-Clause OR Apache-2.0 OR MIT",
    },
    Credit {
        name: "zerofrom",
        version: "0.1.8",
        license: "Unicode-3.0",
    },
    Credit {
        name: "zerofrom-derive",
        version: "0.1.8",
        license: "Unicode-3.0",
    },
    Credit {
        name: "zeroize",
        version: "1.9.1",
        license: "Apache-2.0 OR MIT",
    },
    Credit {
        name: "zeroize_derive",
        version: "1.5.0",
        license: "Apache-2.0 OR MIT",
    },
    Credit {
        name: "zerotrie",
        version: "0.2.5",
        license: "Unicode-3.0",
    },
    Credit {
        name: "zerovec",
        version: "0.11.8",
        license: "Unicode-3.0",
    },
    Credit {
        name: "zerovec-derive",
        version: "0.11.6",
        license: "Unicode-3.0",
    },
    Credit {
        name: "zesven",
        version: "3.2.0",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "zip",
        version: "2.6.1",
        license: "MIT",
    },
    Credit {
        name: "zip",
        version: "8.6.0",
        license: "MIT",
    },
    Credit {
        name: "zlib-rs",
        version: "0.6.8",
        license: "Zlib",
    },
    Credit {
        name: "zmij",
        version: "1.0.23",
        license: "MIT",
    },
    Credit {
        name: "zopfli",
        version: "0.8.4",
        license: "Apache-2.0",
    },
    Credit {
        name: "zstd",
        version: "0.13.3",
        license: "MIT",
    },
    Credit {
        name: "zstd-safe",
        version: "7.3.0",
        license: "BSD-3-Clause",
    },
    Credit {
        name: "zstd-sys",
        version: "2.1.2+zstd.1.5.7",
        license: "BSD-3-Clause",
    },
];

#[cfg_attr(not(target_os = "linux"), allow(dead_code))] // listed in the About box on Linux
pub static LINUX: &[Credit] = &[
    Credit {
        name: "ab_glyph",
        version: "0.2.32",
        license: "Apache-2.0",
    },
    Credit {
        name: "ab_glyph_rasterizer",
        version: "0.1.10",
        license: "Apache-2.0",
    },
    Credit {
        name: "accesskit",
        version: "0.19.0",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "accesskit_atspi_common",
        version: "0.12.0",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "accesskit_consumer",
        version: "0.28.0",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "accesskit_unix",
        version: "0.15.0",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "accesskit_winit",
        version: "0.27.0",
        license: "Apache-2.0",
    },
    Credit {
        name: "adler2",
        version: "2.0.1",
        license: "0BSD OR MIT OR Apache-2.0",
    },
    Credit {
        name: "aes",
        version: "0.8.4",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "ahash",
        version: "0.8.12",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "aho-corasick",
        version: "1.1.5",
        license: "Unlicense OR MIT",
    },
    Credit {
        name: "anyhow",
        version: "1.0.104",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "arboard",
        version: "3.7.0",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "arrayref",
        version: "0.3.9",
        license: "BSD-2-Clause",
    },
    Credit {
        name: "arrayvec",
        version: "0.7.8",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "as-raw-xcb-connection",
        version: "1.0.1",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "ashpd",
        version: "0.11.1",
        license: "MIT",
    },
    Credit {
        name: "async-broadcast",
        version: "0.7.2",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "async-channel",
        version: "2.5.0",
        license: "Apache-2.0 OR MIT",
    },
    Credit {
        name: "async-executor",
        version: "1.14.0",
        license: "Apache-2.0 OR MIT",
    },
    Credit {
        name: "async-fs",
        version: "2.2.0",
        license: "Apache-2.0 OR MIT",
    },
    Credit {
        name: "async-io",
        version: "2.6.0",
        license: "Apache-2.0 OR MIT",
    },
    Credit {
        name: "async-lock",
        version: "3.4.2",
        license: "Apache-2.0 OR MIT",
    },
    Credit {
        name: "async-net",
        version: "2.0.0",
        license: "Apache-2.0 OR MIT",
    },
    Credit {
        name: "async-process",
        version: "2.5.0",
        license: "Apache-2.0 OR MIT",
    },
    Credit {
        name: "async-recursion",
        version: "1.2.0",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "async-signal",
        version: "0.2.14",
        license: "Apache-2.0 OR MIT",
    },
    Credit {
        name: "async-task",
        version: "4.7.1",
        license: "Apache-2.0 OR MIT",
    },
    Credit {
        name: "async-trait",
        version: "0.1.92",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "atomic-waker",
        version: "1.1.2",
        license: "Apache-2.0 OR MIT",
    },
    Credit {
        name: "atspi",
        version: "0.25.0",
        license: "Apache-2.0 OR MIT",
    },
    Credit {
        name: "atspi-common",
        version: "0.9.0",
        license: "Apache-2.0 OR MIT",
    },
    Credit {
        name: "atspi-connection",
        version: "0.9.0",
        license: "Apache-2.0 OR MIT",
    },
    Credit {
        name: "atspi-proxies",
        version: "0.9.0",
        license: "Apache-2.0 OR MIT",
    },
    Credit {
        name: "base64",
        version: "0.22.1",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "base64",
        version: "0.23.1",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "bitflags",
        version: "1.3.2",
        license: "MIT/Apache-2.0",
    },
    Credit {
        name: "bitflags",
        version: "2.13.2",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "blake2s_simd",
        version: "1.0.5",
        license: "MIT",
    },
    Credit {
        name: "block-buffer",
        version: "0.10.4",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "block-buffer",
        version: "0.12.1",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "block-padding",
        version: "0.3.3",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "blocking",
        version: "1.7.0",
        license: "Apache-2.0 OR MIT",
    },
    Credit {
        name: "bstr",
        version: "1.13.1",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "bumpalo",
        version: "3.20.3",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "bytemuck",
        version: "1.25.2",
        license: "Zlib OR Apache-2.0 OR MIT",
    },
    Credit {
        name: "bytemuck_derive",
        version: "1.12.1",
        license: "Zlib OR Apache-2.0 OR MIT",
    },
    Credit {
        name: "byteorder-lite",
        version: "0.1.0",
        license: "Unlicense OR MIT",
    },
    Credit {
        name: "bytes",
        version: "1.12.1",
        license: "MIT",
    },
    Credit {
        name: "bzip2",
        version: "0.6.1",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "calloop",
        version: "0.13.0",
        license: "MIT",
    },
    Credit {
        name: "calloop",
        version: "0.14.5",
        license: "MIT",
    },
    Credit {
        name: "calloop-wayland-source",
        version: "0.3.0",
        license: "MIT",
    },
    Credit {
        name: "calloop-wayland-source",
        version: "0.4.1",
        license: "MIT",
    },
    Credit {
        name: "cbc",
        version: "0.1.2",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "cfg-if",
        version: "1.0.5",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "chacha20",
        version: "0.10.2",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "chrono",
        version: "0.4.45",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "cipher",
        version: "0.4.4",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "color_quant",
        version: "1.1.0",
        license: "MIT",
    },
    Credit {
        name: "concurrent-queue",
        version: "2.5.0",
        license: "Apache-2.0 OR MIT",
    },
    Credit {
        name: "const-oid",
        version: "0.10.2",
        license: "Apache-2.0 OR MIT",
    },
    Credit {
        name: "constant_time_eq",
        version: "0.3.1",
        license: "CC0-1.0 OR MIT-0 OR Apache-2.0",
    },
    Credit {
        name: "constant_time_eq",
        version: "0.4.2",
        license: "CC0-1.0 OR MIT-0 OR Apache-2.0",
    },
    Credit {
        name: "cpufeatures",
        version: "0.2.17",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "cpufeatures",
        version: "0.3.1",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "crc32fast",
        version: "1.5.3",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "crc64fast",
        version: "1.1.0",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "crossbeam-deque",
        version: "0.8.8",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "crossbeam-epoch",
        version: "0.9.21",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "crossbeam-utils",
        version: "0.8.23",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "crypto-common",
        version: "0.1.7",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "crypto-common",
        version: "0.2.2",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "cursor-icon",
        version: "1.2.0",
        license: "MIT OR Apache-2.0 OR Zlib",
    },
    Credit {
        name: "data-url",
        version: "0.3.2",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "deranged",
        version: "0.5.9",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "derivative",
        version: "2.2.0",
        license: "MIT/Apache-2.0",
    },
    Credit {
        name: "digest",
        version: "0.10.7",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "digest",
        version: "0.11.3",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "directories",
        version: "6.0.0",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "dirs",
        version: "5.0.1",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "dirs-next",
        version: "2.0.0",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "dirs-sys",
        version: "0.4.1",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "dirs-sys",
        version: "0.5.0",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "dirs-sys-next",
        version: "0.1.2",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "displaydoc",
        version: "0.2.7",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "dlib",
        version: "0.5.3",
        license: "MIT",
    },
    Credit {
        name: "document-features",
        version: "0.2.12",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "downcast-rs",
        version: "1.2.1",
        license: "MIT/Apache-2.0",
    },
    Credit {
        name: "dpi",
        version: "0.1.2",
        license: "Apache-2.0 AND MIT",
    },
    Credit {
        name: "ecolor",
        version: "0.32.3",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "eframe",
        version: "0.32.3",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "egui",
        version: "0.32.3",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "egui-winit",
        version: "0.32.3",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "egui_extras",
        version: "0.32.3",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "egui_glow",
        version: "0.32.3",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "ehttp",
        version: "0.5.0",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "either",
        version: "1.19.0",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "emath",
        version: "0.32.3",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "endi",
        version: "1.1.1",
        license: "MIT",
    },
    Credit {
        name: "enum-map",
        version: "2.7.3",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "enum-map-derive",
        version: "0.17.0",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "enumflags2",
        version: "0.7.12",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "enumflags2_derive",
        version: "0.7.12",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "epaint",
        version: "0.32.3",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "epaint_default_fonts",
        version: "0.32.3",
        license: "(MIT OR Apache-2.0) AND OFL-1.1 AND Ubuntu-font-1.0",
    },
    Credit {
        name: "equivalent",
        version: "1.0.3",
        license: "Apache-2.0 OR MIT",
    },
    Credit {
        name: "errno",
        version: "0.3.14",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "event-listener",
        version: "5.4.2",
        license: "Apache-2.0 OR MIT",
    },
    Credit {
        name: "event-listener-strategy",
        version: "0.5.4",
        license: "Apache-2.0 OR MIT",
    },
    Credit {
        name: "fastrand",
        version: "2.5.0",
        license: "Apache-2.0 OR MIT",
    },
    Credit {
        name: "fdeflate",
        version: "0.3.7",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "filetime",
        version: "0.2.29",
        license: "MIT/Apache-2.0",
    },
    Credit {
        name: "flate2",
        version: "1.1.10",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "float-cmp",
        version: "0.9.0",
        license: "MIT",
    },
    Credit {
        name: "foldhash",
        version: "0.1.5",
        license: "Zlib",
    },
    Credit {
        name: "font-types",
        version: "0.12.7",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "fontconfig-parser",
        version: "0.5.8",
        license: "MIT",
    },
    Credit {
        name: "fontdb",
        version: "0.24.0",
        license: "MIT",
    },
    Credit {
        name: "form_urlencoded",
        version: "1.2.2",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "freedesktop-icons",
        version: "0.4.0",
        license: "MIT",
    },
    Credit {
        name: "futures-channel",
        version: "0.3.34",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "futures-core",
        version: "0.3.34",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "futures-io",
        version: "0.3.34",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "futures-lite",
        version: "2.6.1",
        license: "Apache-2.0 OR MIT",
    },
    Credit {
        name: "futures-macro",
        version: "0.3.34",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "futures-task",
        version: "0.3.34",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "futures-util",
        version: "0.3.34",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "generic-array",
        version: "0.14.7",
        license: "MIT",
    },
    Credit {
        name: "gethostname",
        version: "1.1.0",
        license: "Apache-2.0",
    },
    Credit {
        name: "getrandom",
        version: "0.2.17",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "getrandom",
        version: "0.3.4",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "getrandom",
        version: "0.4.3",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "gif",
        version: "0.14.2",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "glob",
        version: "0.3.4",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "globset",
        version: "0.4.20",
        license: "Unlicense OR MIT",
    },
    Credit {
        name: "glow",
        version: "0.16.0",
        license: "MIT OR Apache-2.0 OR Zlib",
    },
    Credit {
        name: "glutin",
        version: "0.32.3",
        license: "Apache-2.0",
    },
    Credit {
        name: "glutin-winit",
        version: "0.5.0",
        license: "MIT",
    },
    Credit {
        name: "glutin_egl_sys",
        version: "0.7.1",
        license: "Apache-2.0",
    },
    Credit {
        name: "glutin_glx_sys",
        version: "0.6.1",
        license: "Apache-2.0",
    },
    Credit {
        name: "harfrust",
        version: "0.12.0",
        license: "MIT",
    },
    Credit {
        name: "hashbrown",
        version: "0.15.5",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "hashbrown",
        version: "0.17.1",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "hex",
        version: "0.4.3",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "hmac",
        version: "0.12.1",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "http",
        version: "1.5.0",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "httparse",
        version: "1.10.1",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "hybrid-array",
        version: "0.4.15",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "iana-time-zone",
        version: "0.1.65",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "icu_collections",
        version: "2.3.0",
        license: "Unicode-3.0",
    },
    Credit {
        name: "icu_locale_core",
        version: "2.3.0",
        license: "Unicode-3.0",
    },
    Credit {
        name: "icu_normalizer",
        version: "2.3.0",
        license: "Unicode-3.0",
    },
    Credit {
        name: "icu_normalizer_data",
        version: "2.3.0",
        license: "Unicode-3.0",
    },
    Credit {
        name: "icu_properties",
        version: "2.3.0",
        license: "Unicode-3.0",
    },
    Credit {
        name: "icu_properties_data",
        version: "2.3.0",
        license: "Unicode-3.0",
    },
    Credit {
        name: "icu_provider",
        version: "2.3.1",
        license: "Unicode-3.0",
    },
    Credit {
        name: "idna",
        version: "1.1.0",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "idna_adapter",
        version: "1.2.2",
        license: "Apache-2.0 OR MIT",
    },
    Credit {
        name: "image",
        version: "0.25.10",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "image-webp",
        version: "0.2.4",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "imagesize",
        version: "0.13.0",
        license: "MIT",
    },
    Credit {
        name: "imagesize",
        version: "0.15.0",
        license: "MIT",
    },
    Credit {
        name: "indexmap",
        version: "2.14.2",
        license: "Apache-2.0 OR MIT",
    },
    Credit {
        name: "ini_core",
        version: "0.2.0",
        license: "MIT",
    },
    Credit {
        name: "inout",
        version: "0.1.4",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "itoa",
        version: "1.0.18",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "kurbo",
        version: "0.11.3",
        license: "Apache-2.0 OR MIT",
    },
    Credit {
        name: "kurbo",
        version: "0.13.1",
        license: "Apache-2.0 OR MIT",
    },
    Credit {
        name: "lazy_static",
        version: "1.5.1",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "libbz2-rs-sys",
        version: "0.2.5",
        license: "bzip2-1.0.6",
    },
    Credit {
        name: "libc",
        version: "0.2.191",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "libloading",
        version: "0.8.9",
        license: "ISC",
    },
    Credit {
        name: "linux-raw-sys",
        version: "0.12.1",
        license: "Apache-2.0 WITH LLVM-exception OR Apache-2.0 OR MIT",
    },
    Credit {
        name: "linux-raw-sys",
        version: "0.4.15",
        license: "Apache-2.0 WITH LLVM-exception OR Apache-2.0 OR MIT",
    },
    Credit {
        name: "litemap",
        version: "0.8.3",
        license: "Unicode-3.0",
    },
    Credit {
        name: "litrs",
        version: "1.0.1",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "lock_api",
        version: "0.4.14",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "log",
        version: "0.4.34",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "lzma-rust2",
        version: "0.15.8",
        license: "Apache-2.0",
    },
    Credit {
        name: "lzma-sys",
        version: "0.1.20",
        license: "MIT/Apache-2.0",
    },
    Credit {
        name: "memchr",
        version: "2.8.3",
        license: "Unlicense OR MIT",
    },
    Credit {
        name: "memmap2",
        version: "0.9.11",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "memoffset",
        version: "0.9.1",
        license: "MIT",
    },
    Credit {
        name: "mime",
        version: "0.3.17",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "mime_guess2",
        version: "2.3.1",
        license: "MIT",
    },
    Credit {
        name: "minimal-lexical",
        version: "0.2.1",
        license: "MIT/Apache-2.0",
    },
    Credit {
        name: "miniz_oxide",
        version: "0.8.9",
        license: "MIT OR Zlib OR Apache-2.0",
    },
    Credit {
        name: "miniz_oxide",
        version: "0.9.1",
        license: "MIT OR Zlib OR Apache-2.0",
    },
    Credit {
        name: "moxcms",
        version: "0.8.1",
        license: "BSD-3-Clause OR Apache-2.0",
    },
    Credit {
        name: "nohash-hasher",
        version: "0.2.0",
        license: "Apache-2.0 OR MIT",
    },
    Credit {
        name: "nom",
        version: "7.1.3",
        license: "MIT",
    },
    Credit {
        name: "normpath",
        version: "1.5.2",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "num-conv",
        version: "0.2.2",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "num-traits",
        version: "0.2.19",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "once_cell",
        version: "1.21.4",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "option-ext",
        version: "0.2.0",
        license: "MPL-2.0",
    },
    Credit {
        name: "ordered-stream",
        version: "0.2.0",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "owned_ttf_parser",
        version: "0.25.1",
        license: "Apache-2.0",
    },
    Credit {
        name: "parking",
        version: "2.2.1",
        license: "Apache-2.0 OR MIT",
    },
    Credit {
        name: "parking_lot",
        version: "0.12.5",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "parking_lot_core",
        version: "0.9.12",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "pbkdf2",
        version: "0.12.2",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "percent-encoding",
        version: "2.3.2",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "pico-args",
        version: "0.5.0",
        license: "MIT",
    },
    Credit {
        name: "pin-project-lite",
        version: "0.2.17",
        license: "Apache-2.0 OR MIT",
    },
    Credit {
        name: "piper",
        version: "0.2.5",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "png",
        version: "0.17.16",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "png",
        version: "0.18.1",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "polling",
        version: "3.11.0",
        license: "Apache-2.0 OR MIT",
    },
    Credit {
        name: "pollster",
        version: "0.4.0",
        license: "Apache-2.0/MIT",
    },
    Credit {
        name: "polycool",
        version: "0.4.0",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "potential_utf",
        version: "0.1.6",
        license: "Unicode-3.0",
    },
    Credit {
        name: "powerfmt",
        version: "0.2.1",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "ppmd-rust",
        version: "1.5.2",
        license: "CC0-1.0 OR MIT-0",
    },
    Credit {
        name: "ppv-lite86",
        version: "0.2.21",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "proc-macro-crate",
        version: "3.5.0",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "proc-macro2",
        version: "1.0.107",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "profiling",
        version: "1.0.18",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "pxfm",
        version: "0.1.30",
        license: "BSD-3-Clause OR Apache-2.0",
    },
    Credit {
        name: "quick-error",
        version: "2.0.1",
        license: "MIT/Apache-2.0",
    },
    Credit {
        name: "quick-xml",
        version: "0.41.0",
        license: "MIT",
    },
    Credit {
        name: "quote",
        version: "1.0.47",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "rand",
        version: "0.10.3",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "rand",
        version: "0.9.5",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "rand_chacha",
        version: "0.9.0",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "rand_core",
        version: "0.10.1",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "rand_core",
        version: "0.9.5",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "raw-window-handle",
        version: "0.6.2",
        license: "MIT OR Apache-2.0 OR Zlib",
    },
    Credit {
        name: "rayon",
        version: "1.12.0",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "rayon-core",
        version: "1.13.0",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "read-fonts",
        version: "0.41.0",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "regex",
        version: "1.13.1",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "regex-automata",
        version: "0.4.18",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "regex-syntax",
        version: "0.8.11",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "resvg",
        version: "0.45.1",
        license: "Apache-2.0 OR MIT",
    },
    Credit {
        name: "resvg",
        version: "0.49.0",
        license: "Apache-2.0 OR MIT",
    },
    Credit {
        name: "rfd",
        version: "0.15.4",
        license: "MIT",
    },
    Credit {
        name: "rgb",
        version: "0.8.53",
        license: "MIT",
    },
    Credit {
        name: "ring",
        version: "0.17.14",
        license: "Apache-2.0 AND ISC",
    },
    Credit {
        name: "roxmltree",
        version: "0.20.0",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "roxmltree",
        version: "0.21.2",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "rustix",
        version: "0.38.44",
        license: "Apache-2.0 WITH LLVM-exception OR Apache-2.0 OR MIT",
    },
    Credit {
        name: "rustix",
        version: "1.1.5",
        license: "Apache-2.0 WITH LLVM-exception OR Apache-2.0 OR MIT",
    },
    Credit {
        name: "rustls",
        version: "0.23.46",
        license: "Apache-2.0 OR ISC OR MIT",
    },
    Credit {
        name: "rustls-pki-types",
        version: "1.15.1",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "rustls-webpki",
        version: "0.103.16",
        license: "ISC",
    },
    Credit {
        name: "same-file",
        version: "1.0.6",
        license: "Unlicense/MIT",
    },
    Credit {
        name: "scoped-tls",
        version: "1.0.1",
        license: "MIT/Apache-2.0",
    },
    Credit {
        name: "scopeguard",
        version: "1.2.0",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "sctk-adwaita",
        version: "0.10.1",
        license: "MIT",
    },
    Credit {
        name: "semver",
        version: "1.0.28",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "serde",
        version: "1.0.229",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "serde_core",
        version: "1.0.229",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "serde_derive",
        version: "1.0.229",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "serde_json",
        version: "1.0.154",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "serde_repr",
        version: "0.1.21",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "serde_spanned",
        version: "1.1.2",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "sha1",
        version: "0.10.7",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "sha1",
        version: "0.11.0",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "sha2",
        version: "0.10.9",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "sha2",
        version: "0.11.1",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "signal-hook-registry",
        version: "1.4.8",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "simd-adler32",
        version: "0.3.10",
        license: "MIT",
    },
    Credit {
        name: "simplecss",
        version: "0.2.2",
        license: "Apache-2.0 OR MIT",
    },
    Credit {
        name: "siphasher",
        version: "1.0.4",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "skrifa",
        version: "0.44.0",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "slab",
        version: "0.4.12",
        license: "MIT",
    },
    Credit {
        name: "slotmap",
        version: "1.1.1",
        license: "Zlib",
    },
    Credit {
        name: "smallvec",
        version: "1.16.3",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "smithay-client-toolkit",
        version: "0.19.2",
        license: "MIT",
    },
    Credit {
        name: "smithay-client-toolkit",
        version: "0.20.0",
        license: "MIT",
    },
    Credit {
        name: "smithay-clipboard",
        version: "0.7.3",
        license: "MIT",
    },
    Credit {
        name: "smol_str",
        version: "0.2.2",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "stable_deref_trait",
        version: "1.2.1",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "static_assertions",
        version: "1.1.0",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "strict-num",
        version: "0.1.1",
        license: "MIT",
    },
    Credit {
        name: "subtle",
        version: "2.6.1",
        license: "BSD-3-Clause",
    },
    Credit {
        name: "svgtypes",
        version: "0.15.3",
        license: "Apache-2.0 OR MIT",
    },
    Credit {
        name: "svgtypes",
        version: "0.16.1",
        license: "Apache-2.0 OR MIT",
    },
    Credit {
        name: "syn",
        version: "1.0.109",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "syn",
        version: "2.0.119",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "syn",
        version: "3.0.9",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "synstructure",
        version: "0.14.0",
        license: "MIT",
    },
    Credit {
        name: "sysinfo",
        version: "0.32.1",
        license: "MIT",
    },
    Credit {
        name: "sysinfo",
        version: "0.40.1",
        license: "MIT",
    },
    Credit {
        name: "tar",
        version: "0.4.46",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "thiserror",
        version: "1.0.69",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "thiserror",
        version: "2.0.21",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "thiserror-impl",
        version: "1.0.69",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "thiserror-impl",
        version: "2.0.21",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "time",
        version: "0.3.55",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "time-core",
        version: "0.1.9",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "tiny-skia",
        version: "0.11.4",
        license: "BSD-3-Clause",
    },
    Credit {
        name: "tiny-skia",
        version: "0.12.0",
        license: "BSD-3-Clause",
    },
    Credit {
        name: "tiny-skia-path",
        version: "0.11.4",
        license: "BSD-3-Clause",
    },
    Credit {
        name: "tiny-skia-path",
        version: "0.12.0",
        license: "BSD-3-Clause",
    },
    Credit {
        name: "tinystr",
        version: "0.8.4",
        license: "Unicode-3.0",
    },
    Credit {
        name: "tinyvec",
        version: "1.13.3",
        license: "Zlib OR Apache-2.0 OR MIT",
    },
    Credit {
        name: "toml",
        version: "1.1.8+spec-1.1.0",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "toml_datetime",
        version: "1.1.2+spec-1.1.0",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "toml_edit",
        version: "0.25.17+spec-1.1.0",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "toml_parser",
        version: "1.1.5+spec-1.1.0",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "toml_writer",
        version: "1.1.3+spec-1.1.0",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "tracing",
        version: "0.1.44",
        license: "MIT",
    },
    Credit {
        name: "tracing-attributes",
        version: "0.1.31",
        license: "MIT",
    },
    Credit {
        name: "tracing-core",
        version: "0.1.36",
        license: "MIT",
    },
    Credit {
        name: "trash",
        version: "5.2.9",
        license: "MIT",
    },
    Credit {
        name: "ttf-parser",
        version: "0.25.1",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "typed-path",
        version: "0.12.4",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "typenum",
        version: "1.20.1",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "unicase",
        version: "2.10.0",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "unicode-bidi",
        version: "0.3.18",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "unicode-ident",
        version: "1.0.27",
        license: "(MIT OR Apache-2.0) AND Unicode-3.0",
    },
    Credit {
        name: "unicode-script",
        version: "0.5.8",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "unicode-segmentation",
        version: "1.13.3",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "unicode-vo",
        version: "0.1.0",
        license: "MIT/Apache-2.0",
    },
    Credit {
        name: "unrar",
        version: "0.5.8",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "unrar_sys",
        version: "0.5.8",
        license: "(MIT OR Apache-2.0) AND LicenseRef-UnRAR",
    },
    Credit {
        name: "untrusted",
        version: "0.9.0",
        license: "ISC",
    },
    Credit {
        name: "ureq",
        version: "2.12.1",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "ureq",
        version: "3.5.1",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "ureq-proto",
        version: "0.7.1",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "url",
        version: "2.5.8",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "urlencoding",
        version: "2.1.3",
        license: "MIT",
    },
    Credit {
        name: "usvg",
        version: "0.45.1",
        license: "Apache-2.0 OR MIT",
    },
    Credit {
        name: "usvg",
        version: "0.49.0",
        license: "Apache-2.0 OR MIT",
    },
    Credit {
        name: "utf8-zero",
        version: "0.8.1",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "utf8_iter",
        version: "1.0.4",
        license: "Apache-2.0 OR MIT",
    },
    Credit {
        name: "uuid",
        version: "1.28.0",
        license: "Apache-2.0 OR MIT",
    },
    Credit {
        name: "velopack",
        version: "1.2.161",
        license: "MIT",
    },
    Credit {
        name: "wait-timeout",
        version: "0.2.1",
        license: "MIT/Apache-2.0",
    },
    Credit {
        name: "waitpid-any",
        version: "0.3.0",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "walkdir",
        version: "2.5.0",
        license: "Unlicense/MIT",
    },
    Credit {
        name: "wayland-backend",
        version: "0.3.17",
        license: "MIT",
    },
    Credit {
        name: "wayland-client",
        version: "0.31.15",
        license: "MIT",
    },
    Credit {
        name: "wayland-csd-frame",
        version: "0.3.0",
        license: "MIT",
    },
    Credit {
        name: "wayland-cursor",
        version: "0.31.14",
        license: "MIT",
    },
    Credit {
        name: "wayland-protocols",
        version: "0.32.13",
        license: "MIT",
    },
    Credit {
        name: "wayland-protocols-experimental",
        version: "20250721.0.1",
        license: "MIT",
    },
    Credit {
        name: "wayland-protocols-misc",
        version: "0.3.12",
        license: "MIT",
    },
    Credit {
        name: "wayland-protocols-plasma",
        version: "0.3.12",
        license: "MIT",
    },
    Credit {
        name: "wayland-protocols-wlr",
        version: "0.3.12",
        license: "MIT",
    },
    Credit {
        name: "wayland-scanner",
        version: "0.31.11",
        license: "MIT",
    },
    Credit {
        name: "wayland-sys",
        version: "0.31.11",
        license: "MIT",
    },
    Credit {
        name: "web-time",
        version: "1.1.0",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "webbrowser",
        version: "1.2.4",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "webpki-roots",
        version: "0.26.11",
        license: "CDLA-Permissive-2.0",
    },
    Credit {
        name: "webpki-roots",
        version: "1.0.10",
        license: "CDLA-Permissive-2.0",
    },
    Credit {
        name: "weezl",
        version: "0.1.12",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "widestring",
        version: "1.2.1",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "winit",
        version: "0.30.13",
        license: "Apache-2.0",
    },
    Credit {
        name: "winnow",
        version: "1.0.4",
        license: "MIT",
    },
    Credit {
        name: "writeable",
        version: "0.6.4",
        license: "Unicode-3.0",
    },
    Credit {
        name: "x11-dl",
        version: "2.21.0",
        license: "MIT",
    },
    Credit {
        name: "x11rb",
        version: "0.13.2",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "x11rb-protocol",
        version: "0.13.2",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "xattr",
        version: "1.6.1",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "xcursor",
        version: "0.3.11",
        license: "MIT",
    },
    Credit {
        name: "xdg",
        version: "2.5.2",
        license: "Apache-2.0 OR MIT",
    },
    Credit {
        name: "xdg-mime",
        version: "0.4.0",
        license: "Apache-2.0",
    },
    Credit {
        name: "xkbcommon-dl",
        version: "0.4.2",
        license: "MIT",
    },
    Credit {
        name: "xkeysym",
        version: "0.2.1",
        license: "MIT OR Apache-2.0 OR Zlib",
    },
    Credit {
        name: "xml",
        version: "1.4.0",
        license: "MIT",
    },
    Credit {
        name: "xmlwriter",
        version: "0.1.0",
        license: "MIT",
    },
    Credit {
        name: "xz2",
        version: "0.1.7",
        license: "MIT/Apache-2.0",
    },
    Credit {
        name: "yoke",
        version: "0.8.3",
        license: "Unicode-3.0",
    },
    Credit {
        name: "yoke-derive",
        version: "0.8.4",
        license: "Unicode-3.0",
    },
    Credit {
        name: "zbus",
        version: "5.19.0",
        license: "MIT",
    },
    Credit {
        name: "zbus-lockstep",
        version: "0.5.2",
        license: "MIT",
    },
    Credit {
        name: "zbus-lockstep-macros",
        version: "0.5.2",
        license: "MIT",
    },
    Credit {
        name: "zbus_macros",
        version: "5.19.0",
        license: "MIT",
    },
    Credit {
        name: "zbus_names",
        version: "4.3.5",
        license: "MIT",
    },
    Credit {
        name: "zbus_xml",
        version: "5.2.1",
        license: "MIT",
    },
    Credit {
        name: "zcheapstr",
        version: "1.1.0",
        license: "MIT",
    },
    Credit {
        name: "zerocopy",
        version: "0.8.63",
        license: "BSD-2-Clause OR Apache-2.0 OR MIT",
    },
    Credit {
        name: "zerofrom",
        version: "0.1.8",
        license: "Unicode-3.0",
    },
    Credit {
        name: "zerofrom-derive",
        version: "0.1.8",
        license: "Unicode-3.0",
    },
    Credit {
        name: "zeroize",
        version: "1.9.1",
        license: "Apache-2.0 OR MIT",
    },
    Credit {
        name: "zeroize_derive",
        version: "1.5.0",
        license: "Apache-2.0 OR MIT",
    },
    Credit {
        name: "zerotrie",
        version: "0.2.5",
        license: "Unicode-3.0",
    },
    Credit {
        name: "zerovec",
        version: "0.11.8",
        license: "Unicode-3.0",
    },
    Credit {
        name: "zerovec-derive",
        version: "0.11.6",
        license: "Unicode-3.0",
    },
    Credit {
        name: "zesven",
        version: "3.2.0",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "zip",
        version: "2.6.1",
        license: "MIT",
    },
    Credit {
        name: "zip",
        version: "8.6.0",
        license: "MIT",
    },
    Credit {
        name: "zlib-rs",
        version: "0.6.8",
        license: "Zlib",
    },
    Credit {
        name: "zmij",
        version: "1.0.23",
        license: "MIT",
    },
    Credit {
        name: "zopfli",
        version: "0.8.4",
        license: "Apache-2.0",
    },
    Credit {
        name: "zstd",
        version: "0.13.3",
        license: "MIT",
    },
    Credit {
        name: "zstd-safe",
        version: "7.3.0",
        license: "BSD-3-Clause",
    },
    Credit {
        name: "zstd-sys",
        version: "2.1.2+zstd.1.5.7",
        license: "BSD-3-Clause",
    },
    Credit {
        name: "zune-core",
        version: "0.5.3",
        license: "MIT OR Apache-2.0 OR Zlib",
    },
    Credit {
        name: "zune-jpeg",
        version: "0.5.15",
        license: "MIT OR Apache-2.0 OR Zlib",
    },
    Credit {
        name: "zvariant",
        version: "5.15.1",
        license: "MIT",
    },
    Credit {
        name: "zvariant_derive",
        version: "5.15.1",
        license: "MIT",
    },
    Credit {
        name: "zvariant_utils",
        version: "4.2.0",
        license: "MIT",
    },
];

#[cfg_attr(not(target_os = "macos"), allow(dead_code))] // listed in the About box on macOS
pub static MACOS: &[Credit] = &[
    Credit {
        name: "ab_glyph",
        version: "0.2.32",
        license: "Apache-2.0",
    },
    Credit {
        name: "ab_glyph_rasterizer",
        version: "0.1.10",
        license: "Apache-2.0",
    },
    Credit {
        name: "accesskit",
        version: "0.19.0",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "accesskit_consumer",
        version: "0.28.0",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "accesskit_macos",
        version: "0.20.0",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "accesskit_winit",
        version: "0.27.0",
        license: "Apache-2.0",
    },
    Credit {
        name: "adler2",
        version: "2.0.1",
        license: "0BSD OR MIT OR Apache-2.0",
    },
    Credit {
        name: "aes",
        version: "0.8.4",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "ahash",
        version: "0.8.12",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "aho-corasick",
        version: "1.1.5",
        license: "Unlicense OR MIT",
    },
    Credit {
        name: "anyhow",
        version: "1.0.104",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "arboard",
        version: "3.7.0",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "arrayref",
        version: "0.3.9",
        license: "BSD-2-Clause",
    },
    Credit {
        name: "arrayvec",
        version: "0.7.8",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "base64",
        version: "0.22.1",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "base64",
        version: "0.23.1",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "bitflags",
        version: "1.3.2",
        license: "MIT/Apache-2.0",
    },
    Credit {
        name: "bitflags",
        version: "2.13.2",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "blake2s_simd",
        version: "1.0.5",
        license: "MIT",
    },
    Credit {
        name: "block",
        version: "0.1.6",
        license: "MIT",
    },
    Credit {
        name: "block-buffer",
        version: "0.10.4",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "block-buffer",
        version: "0.12.1",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "block-padding",
        version: "0.3.3",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "block2",
        version: "0.5.1",
        license: "MIT",
    },
    Credit {
        name: "block2",
        version: "0.6.2",
        license: "MIT",
    },
    Credit {
        name: "bstr",
        version: "1.13.1",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "bumpalo",
        version: "3.20.3",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "bytemuck",
        version: "1.25.2",
        license: "Zlib OR Apache-2.0 OR MIT",
    },
    Credit {
        name: "bytemuck_derive",
        version: "1.12.1",
        license: "Zlib OR Apache-2.0 OR MIT",
    },
    Credit {
        name: "byteorder-lite",
        version: "0.1.0",
        license: "Unlicense OR MIT",
    },
    Credit {
        name: "bytes",
        version: "1.12.1",
        license: "MIT",
    },
    Credit {
        name: "bzip2",
        version: "0.6.1",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "cbc",
        version: "0.1.2",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "cfg-if",
        version: "1.0.5",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "cgl",
        version: "0.3.2",
        license: "MIT / Apache-2.0",
    },
    Credit {
        name: "chacha20",
        version: "0.10.2",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "chrono",
        version: "0.4.45",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "cipher",
        version: "0.4.4",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "cocoa",
        version: "0.26.1",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "cocoa-foundation",
        version: "0.2.1",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "color_quant",
        version: "1.1.0",
        license: "MIT",
    },
    Credit {
        name: "const-oid",
        version: "0.10.2",
        license: "Apache-2.0 OR MIT",
    },
    Credit {
        name: "constant_time_eq",
        version: "0.3.1",
        license: "CC0-1.0 OR MIT-0 OR Apache-2.0",
    },
    Credit {
        name: "constant_time_eq",
        version: "0.4.2",
        license: "CC0-1.0 OR MIT-0 OR Apache-2.0",
    },
    Credit {
        name: "core-foundation",
        version: "0.10.1",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "core-foundation",
        version: "0.9.4",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "core-foundation-sys",
        version: "0.8.7",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "core-graphics",
        version: "0.23.2",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "core-graphics",
        version: "0.24.0",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "core-graphics-types",
        version: "0.1.3",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "core-graphics-types",
        version: "0.2.0",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "cpufeatures",
        version: "0.2.17",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "cpufeatures",
        version: "0.3.1",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "crc32fast",
        version: "1.5.3",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "crc64fast",
        version: "1.1.0",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "crossbeam-deque",
        version: "0.8.8",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "crossbeam-epoch",
        version: "0.9.21",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "crossbeam-utils",
        version: "0.8.23",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "crypto-common",
        version: "0.1.7",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "crypto-common",
        version: "0.2.2",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "cursor-icon",
        version: "1.2.0",
        license: "MIT OR Apache-2.0 OR Zlib",
    },
    Credit {
        name: "data-url",
        version: "0.3.2",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "deranged",
        version: "0.5.9",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "derivative",
        version: "2.2.0",
        license: "MIT/Apache-2.0",
    },
    Credit {
        name: "digest",
        version: "0.10.7",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "digest",
        version: "0.11.3",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "directories",
        version: "6.0.0",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "dirs-sys",
        version: "0.5.0",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "dispatch",
        version: "0.2.0",
        license: "MIT",
    },
    Credit {
        name: "dispatch2",
        version: "0.3.1",
        license: "Zlib OR Apache-2.0 OR MIT",
    },
    Credit {
        name: "displaydoc",
        version: "0.2.7",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "document-features",
        version: "0.2.12",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "dpi",
        version: "0.1.2",
        license: "Apache-2.0 AND MIT",
    },
    Credit {
        name: "ecolor",
        version: "0.32.3",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "eframe",
        version: "0.32.3",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "egui",
        version: "0.32.3",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "egui-winit",
        version: "0.32.3",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "egui_extras",
        version: "0.32.3",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "egui_glow",
        version: "0.32.3",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "ehttp",
        version: "0.5.0",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "either",
        version: "1.19.0",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "emath",
        version: "0.32.3",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "enum-map",
        version: "2.7.3",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "enum-map-derive",
        version: "0.17.0",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "epaint",
        version: "0.32.3",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "epaint_default_fonts",
        version: "0.32.3",
        license: "(MIT OR Apache-2.0) AND OFL-1.1 AND Ubuntu-font-1.0",
    },
    Credit {
        name: "equivalent",
        version: "1.0.3",
        license: "Apache-2.0 OR MIT",
    },
    Credit {
        name: "errno",
        version: "0.3.14",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "fax",
        version: "0.2.7",
        license: "MIT",
    },
    Credit {
        name: "fdeflate",
        version: "0.3.7",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "file_icon_provider",
        version: "1.0.2",
        license: "MIT",
    },
    Credit {
        name: "filetime",
        version: "0.2.29",
        license: "MIT/Apache-2.0",
    },
    Credit {
        name: "flate2",
        version: "1.1.10",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "float-cmp",
        version: "0.9.0",
        license: "MIT",
    },
    Credit {
        name: "foldhash",
        version: "0.1.5",
        license: "Zlib",
    },
    Credit {
        name: "foreign-types",
        version: "0.5.0",
        license: "MIT/Apache-2.0",
    },
    Credit {
        name: "foreign-types-macros",
        version: "0.2.4",
        license: "MIT/Apache-2.0",
    },
    Credit {
        name: "foreign-types-shared",
        version: "0.3.1",
        license: "MIT/Apache-2.0",
    },
    Credit {
        name: "form_urlencoded",
        version: "1.2.2",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "generic-array",
        version: "0.14.7",
        license: "MIT",
    },
    Credit {
        name: "getrandom",
        version: "0.2.17",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "getrandom",
        version: "0.3.4",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "getrandom",
        version: "0.4.3",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "gif",
        version: "0.14.2",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "glob",
        version: "0.3.4",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "globset",
        version: "0.4.20",
        license: "Unlicense OR MIT",
    },
    Credit {
        name: "glow",
        version: "0.16.0",
        license: "MIT OR Apache-2.0 OR Zlib",
    },
    Credit {
        name: "glutin",
        version: "0.32.3",
        license: "Apache-2.0",
    },
    Credit {
        name: "glutin-winit",
        version: "0.5.0",
        license: "MIT",
    },
    Credit {
        name: "half",
        version: "2.7.1",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "hashbrown",
        version: "0.15.5",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "hashbrown",
        version: "0.17.1",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "hmac",
        version: "0.12.1",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "http",
        version: "1.5.0",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "httparse",
        version: "1.10.1",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "hybrid-array",
        version: "0.4.15",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "iana-time-zone",
        version: "0.1.65",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "icu_collections",
        version: "2.3.0",
        license: "Unicode-3.0",
    },
    Credit {
        name: "icu_locale_core",
        version: "2.3.0",
        license: "Unicode-3.0",
    },
    Credit {
        name: "icu_normalizer",
        version: "2.3.0",
        license: "Unicode-3.0",
    },
    Credit {
        name: "icu_normalizer_data",
        version: "2.3.0",
        license: "Unicode-3.0",
    },
    Credit {
        name: "icu_properties",
        version: "2.3.0",
        license: "Unicode-3.0",
    },
    Credit {
        name: "icu_properties_data",
        version: "2.3.0",
        license: "Unicode-3.0",
    },
    Credit {
        name: "icu_provider",
        version: "2.3.1",
        license: "Unicode-3.0",
    },
    Credit {
        name: "idna",
        version: "1.1.0",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "idna_adapter",
        version: "1.2.2",
        license: "Apache-2.0 OR MIT",
    },
    Credit {
        name: "image",
        version: "0.25.10",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "image-webp",
        version: "0.2.4",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "imagesize",
        version: "0.13.0",
        license: "MIT",
    },
    Credit {
        name: "indexmap",
        version: "2.14.2",
        license: "Apache-2.0 OR MIT",
    },
    Credit {
        name: "inout",
        version: "0.1.4",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "itoa",
        version: "1.0.18",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "kurbo",
        version: "0.11.3",
        license: "Apache-2.0 OR MIT",
    },
    Credit {
        name: "lazy_static",
        version: "1.5.1",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "libbz2-rs-sys",
        version: "0.2.5",
        license: "bzip2-1.0.6",
    },
    Credit {
        name: "libc",
        version: "0.2.191",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "libloading",
        version: "0.8.9",
        license: "ISC",
    },
    Credit {
        name: "litemap",
        version: "0.8.3",
        license: "Unicode-3.0",
    },
    Credit {
        name: "litrs",
        version: "1.0.1",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "lock_api",
        version: "0.4.14",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "log",
        version: "0.4.34",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "lzma-rust2",
        version: "0.15.8",
        license: "Apache-2.0",
    },
    Credit {
        name: "lzma-sys",
        version: "0.1.20",
        license: "MIT/Apache-2.0",
    },
    Credit {
        name: "malloc_buf",
        version: "0.0.6",
        license: "MIT",
    },
    Credit {
        name: "memchr",
        version: "2.8.3",
        license: "Unlicense OR MIT",
    },
    Credit {
        name: "memoffset",
        version: "0.9.1",
        license: "MIT",
    },
    Credit {
        name: "mime",
        version: "0.3.17",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "mime_guess2",
        version: "2.3.1",
        license: "MIT",
    },
    Credit {
        name: "miniz_oxide",
        version: "0.8.9",
        license: "MIT OR Zlib OR Apache-2.0",
    },
    Credit {
        name: "miniz_oxide",
        version: "0.9.1",
        license: "MIT OR Zlib OR Apache-2.0",
    },
    Credit {
        name: "moxcms",
        version: "0.8.1",
        license: "BSD-3-Clause OR Apache-2.0",
    },
    Credit {
        name: "nohash-hasher",
        version: "0.2.0",
        license: "Apache-2.0 OR MIT",
    },
    Credit {
        name: "normpath",
        version: "1.5.2",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "num-conv",
        version: "0.2.2",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "num-traits",
        version: "0.2.19",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "objc",
        version: "0.2.7",
        license: "MIT",
    },
    Credit {
        name: "objc-sys",
        version: "0.3.5",
        license: "MIT",
    },
    Credit {
        name: "objc2",
        version: "0.5.3",
        license: "MIT",
    },
    Credit {
        name: "objc2",
        version: "0.6.5",
        license: "MIT",
    },
    Credit {
        name: "objc2-app-kit",
        version: "0.2.2",
        license: "MIT",
    },
    Credit {
        name: "objc2-app-kit",
        version: "0.3.2",
        license: "Zlib OR Apache-2.0 OR MIT",
    },
    Credit {
        name: "objc2-cloud-kit",
        version: "0.3.2",
        license: "Zlib OR Apache-2.0 OR MIT",
    },
    Credit {
        name: "objc2-core-data",
        version: "0.3.2",
        license: "Zlib OR Apache-2.0 OR MIT",
    },
    Credit {
        name: "objc2-core-foundation",
        version: "0.3.2",
        license: "Zlib OR Apache-2.0 OR MIT",
    },
    Credit {
        name: "objc2-core-graphics",
        version: "0.3.2",
        license: "Zlib OR Apache-2.0 OR MIT",
    },
    Credit {
        name: "objc2-core-image",
        version: "0.3.2",
        license: "Zlib OR Apache-2.0 OR MIT",
    },
    Credit {
        name: "objc2-core-text",
        version: "0.3.2",
        license: "Zlib OR Apache-2.0 OR MIT",
    },
    Credit {
        name: "objc2-core-video",
        version: "0.3.2",
        license: "Zlib OR Apache-2.0 OR MIT",
    },
    Credit {
        name: "objc2-encode",
        version: "4.1.0",
        license: "MIT",
    },
    Credit {
        name: "objc2-foundation",
        version: "0.2.2",
        license: "MIT",
    },
    Credit {
        name: "objc2-foundation",
        version: "0.3.2",
        license: "MIT",
    },
    Credit {
        name: "objc2-io-kit",
        version: "0.3.2",
        license: "Zlib OR Apache-2.0 OR MIT",
    },
    Credit {
        name: "objc2-quartz-core",
        version: "0.3.2",
        license: "Zlib OR Apache-2.0 OR MIT",
    },
    Credit {
        name: "objc2-uniform-type-identifiers",
        version: "0.3.2",
        license: "Zlib OR Apache-2.0 OR MIT",
    },
    Credit {
        name: "once_cell",
        version: "1.21.4",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "option-ext",
        version: "0.2.0",
        license: "MPL-2.0",
    },
    Credit {
        name: "owned_ttf_parser",
        version: "0.25.1",
        license: "Apache-2.0",
    },
    Credit {
        name: "parking_lot",
        version: "0.12.5",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "parking_lot_core",
        version: "0.9.12",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "pbkdf2",
        version: "0.12.2",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "percent-encoding",
        version: "2.3.2",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "pico-args",
        version: "0.5.0",
        license: "MIT",
    },
    Credit {
        name: "pin-project-lite",
        version: "0.2.17",
        license: "Apache-2.0 OR MIT",
    },
    Credit {
        name: "png",
        version: "0.17.16",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "png",
        version: "0.18.1",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "potential_utf",
        version: "0.1.6",
        license: "Unicode-3.0",
    },
    Credit {
        name: "powerfmt",
        version: "0.2.1",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "ppmd-rust",
        version: "1.5.2",
        license: "CC0-1.0 OR MIT-0",
    },
    Credit {
        name: "proc-macro2",
        version: "1.0.107",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "profiling",
        version: "1.0.18",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "pxfm",
        version: "0.1.30",
        license: "BSD-3-Clause OR Apache-2.0",
    },
    Credit {
        name: "quick-error",
        version: "2.0.1",
        license: "MIT/Apache-2.0",
    },
    Credit {
        name: "quote",
        version: "1.0.47",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "rand",
        version: "0.10.3",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "rand_core",
        version: "0.10.1",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "raw-window-handle",
        version: "0.6.2",
        license: "MIT OR Apache-2.0 OR Zlib",
    },
    Credit {
        name: "rayon",
        version: "1.12.0",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "rayon-core",
        version: "1.13.0",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "regex",
        version: "1.13.1",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "regex-automata",
        version: "0.4.18",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "regex-syntax",
        version: "0.8.11",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "resvg",
        version: "0.45.1",
        license: "Apache-2.0 OR MIT",
    },
    Credit {
        name: "rfd",
        version: "0.15.4",
        license: "MIT",
    },
    Credit {
        name: "rgb",
        version: "0.8.53",
        license: "MIT",
    },
    Credit {
        name: "ring",
        version: "0.17.14",
        license: "Apache-2.0 AND ISC",
    },
    Credit {
        name: "roxmltree",
        version: "0.20.0",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "rustix",
        version: "1.1.5",
        license: "Apache-2.0 WITH LLVM-exception OR Apache-2.0 OR MIT",
    },
    Credit {
        name: "rustls",
        version: "0.23.46",
        license: "Apache-2.0 OR ISC OR MIT",
    },
    Credit {
        name: "rustls-pki-types",
        version: "1.15.1",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "rustls-webpki",
        version: "0.103.16",
        license: "ISC",
    },
    Credit {
        name: "same-file",
        version: "1.0.6",
        license: "Unlicense/MIT",
    },
    Credit {
        name: "scopeguard",
        version: "1.2.0",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "semver",
        version: "1.0.28",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "serde",
        version: "1.0.229",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "serde_core",
        version: "1.0.229",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "serde_derive",
        version: "1.0.229",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "serde_json",
        version: "1.0.154",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "serde_spanned",
        version: "1.1.2",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "sha1",
        version: "0.10.7",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "sha1",
        version: "0.11.0",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "sha2",
        version: "0.10.9",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "sha2",
        version: "0.11.1",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "simd-adler32",
        version: "0.3.10",
        license: "MIT",
    },
    Credit {
        name: "simplecss",
        version: "0.2.2",
        license: "Apache-2.0 OR MIT",
    },
    Credit {
        name: "siphasher",
        version: "1.0.4",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "smallvec",
        version: "1.16.3",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "smol_str",
        version: "0.2.2",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "stable_deref_trait",
        version: "1.2.1",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "static_assertions",
        version: "1.1.0",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "strict-num",
        version: "0.1.1",
        license: "MIT",
    },
    Credit {
        name: "subtle",
        version: "2.6.1",
        license: "BSD-3-Clause",
    },
    Credit {
        name: "svgtypes",
        version: "0.15.3",
        license: "Apache-2.0 OR MIT",
    },
    Credit {
        name: "syn",
        version: "1.0.109",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "syn",
        version: "2.0.119",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "syn",
        version: "3.0.9",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "synstructure",
        version: "0.14.0",
        license: "MIT",
    },
    Credit {
        name: "sysinfo",
        version: "0.32.1",
        license: "MIT",
    },
    Credit {
        name: "sysinfo",
        version: "0.40.1",
        license: "MIT",
    },
    Credit {
        name: "systemicons",
        version: "0.9.13",
        license: "MIT",
    },
    Credit {
        name: "tar",
        version: "0.4.46",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "thiserror",
        version: "2.0.21",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "thiserror-impl",
        version: "2.0.21",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "tiff",
        version: "0.11.3",
        license: "MIT",
    },
    Credit {
        name: "time",
        version: "0.3.55",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "time-core",
        version: "0.1.9",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "tiny-skia",
        version: "0.11.4",
        license: "BSD-3-Clause",
    },
    Credit {
        name: "tiny-skia-path",
        version: "0.11.4",
        license: "BSD-3-Clause",
    },
    Credit {
        name: "tinystr",
        version: "0.8.4",
        license: "Unicode-3.0",
    },
    Credit {
        name: "toml",
        version: "1.1.8+spec-1.1.0",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "toml_datetime",
        version: "1.1.2+spec-1.1.0",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "toml_parser",
        version: "1.1.5+spec-1.1.0",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "toml_writer",
        version: "1.1.3+spec-1.1.0",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "tracing",
        version: "0.1.44",
        license: "MIT",
    },
    Credit {
        name: "tracing-attributes",
        version: "0.1.31",
        license: "MIT",
    },
    Credit {
        name: "tracing-core",
        version: "0.1.36",
        license: "MIT",
    },
    Credit {
        name: "trash",
        version: "5.2.9",
        license: "MIT",
    },
    Credit {
        name: "ttf-parser",
        version: "0.25.1",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "typed-path",
        version: "0.12.4",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "typenum",
        version: "1.20.1",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "unicase",
        version: "2.10.0",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "unicode-ident",
        version: "1.0.27",
        license: "(MIT OR Apache-2.0) AND Unicode-3.0",
    },
    Credit {
        name: "unicode-segmentation",
        version: "1.13.3",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "unrar",
        version: "0.5.8",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "unrar_sys",
        version: "0.5.8",
        license: "(MIT OR Apache-2.0) AND LicenseRef-UnRAR",
    },
    Credit {
        name: "untrusted",
        version: "0.9.0",
        license: "ISC",
    },
    Credit {
        name: "ureq",
        version: "2.12.1",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "ureq",
        version: "3.5.1",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "ureq-proto",
        version: "0.7.1",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "url",
        version: "2.5.8",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "usvg",
        version: "0.45.1",
        license: "Apache-2.0 OR MIT",
    },
    Credit {
        name: "utf8-zero",
        version: "0.8.1",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "utf8_iter",
        version: "1.0.4",
        license: "Apache-2.0 OR MIT",
    },
    Credit {
        name: "uuid",
        version: "1.28.0",
        license: "Apache-2.0 OR MIT",
    },
    Credit {
        name: "velopack",
        version: "1.2.161",
        license: "MIT",
    },
    Credit {
        name: "wait-timeout",
        version: "0.2.1",
        license: "MIT/Apache-2.0",
    },
    Credit {
        name: "waitpid-any",
        version: "0.3.0",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "walkdir",
        version: "2.5.0",
        license: "Unlicense/MIT",
    },
    Credit {
        name: "web-time",
        version: "1.1.0",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "webbrowser",
        version: "1.2.4",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "webpki-roots",
        version: "0.26.11",
        license: "CDLA-Permissive-2.0",
    },
    Credit {
        name: "webpki-roots",
        version: "1.0.10",
        license: "CDLA-Permissive-2.0",
    },
    Credit {
        name: "weezl",
        version: "0.1.12",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "widestring",
        version: "1.2.1",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "winit",
        version: "0.30.13",
        license: "Apache-2.0",
    },
    Credit {
        name: "winnow",
        version: "1.0.4",
        license: "MIT",
    },
    Credit {
        name: "writeable",
        version: "0.6.4",
        license: "Unicode-3.0",
    },
    Credit {
        name: "xattr",
        version: "1.6.1",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "xml",
        version: "1.4.0",
        license: "MIT",
    },
    Credit {
        name: "xmlwriter",
        version: "0.1.0",
        license: "MIT",
    },
    Credit {
        name: "xz2",
        version: "0.1.7",
        license: "MIT/Apache-2.0",
    },
    Credit {
        name: "yoke",
        version: "0.8.3",
        license: "Unicode-3.0",
    },
    Credit {
        name: "yoke-derive",
        version: "0.8.4",
        license: "Unicode-3.0",
    },
    Credit {
        name: "zerocopy",
        version: "0.8.63",
        license: "BSD-2-Clause OR Apache-2.0 OR MIT",
    },
    Credit {
        name: "zerocopy-derive",
        version: "0.8.63",
        license: "BSD-2-Clause OR Apache-2.0 OR MIT",
    },
    Credit {
        name: "zerofrom",
        version: "0.1.8",
        license: "Unicode-3.0",
    },
    Credit {
        name: "zerofrom-derive",
        version: "0.1.8",
        license: "Unicode-3.0",
    },
    Credit {
        name: "zeroize",
        version: "1.9.1",
        license: "Apache-2.0 OR MIT",
    },
    Credit {
        name: "zeroize_derive",
        version: "1.5.0",
        license: "Apache-2.0 OR MIT",
    },
    Credit {
        name: "zerotrie",
        version: "0.2.5",
        license: "Unicode-3.0",
    },
    Credit {
        name: "zerovec",
        version: "0.11.8",
        license: "Unicode-3.0",
    },
    Credit {
        name: "zerovec-derive",
        version: "0.11.6",
        license: "Unicode-3.0",
    },
    Credit {
        name: "zesven",
        version: "3.2.0",
        license: "MIT OR Apache-2.0",
    },
    Credit {
        name: "zip",
        version: "2.6.1",
        license: "MIT",
    },
    Credit {
        name: "zip",
        version: "8.6.0",
        license: "MIT",
    },
    Credit {
        name: "zlib-rs",
        version: "0.6.8",
        license: "Zlib",
    },
    Credit {
        name: "zmij",
        version: "1.0.23",
        license: "MIT",
    },
    Credit {
        name: "zopfli",
        version: "0.8.4",
        license: "Apache-2.0",
    },
    Credit {
        name: "zstd",
        version: "0.13.3",
        license: "MIT",
    },
    Credit {
        name: "zstd-safe",
        version: "7.3.0",
        license: "BSD-3-Clause",
    },
    Credit {
        name: "zstd-sys",
        version: "2.1.2+zstd.1.5.7",
        license: "BSD-3-Clause",
    },
    Credit {
        name: "zune-core",
        version: "0.5.3",
        license: "MIT OR Apache-2.0 OR Zlib",
    },
    Credit {
        name: "zune-jpeg",
        version: "0.5.15",
        license: "MIT OR Apache-2.0 OR Zlib",
    },
];

#[cfg(test)]
mod tests {
    use super::{Credit, LINUX, MACOS, WINDOWS, credits};

    fn assert_credits(rows: &[Credit]) {
        assert!(rows.len() > 50, "{}", rows.len());
        for pair in rows.windows(2) {
            let left = (pair[0].name.to_ascii_lowercase(), pair[0].version);
            let right = (pair[1].name.to_ascii_lowercase(), pair[1].version);
            assert!(
                left <= right,
                "{} {} before {} {}",
                pair[0].name,
                pair[0].version,
                pair[1].name,
                pair[1].version
            );
        }
        for row in rows {
            assert!(!row.license.is_empty(), "{}", row.name);
            assert!(!row.name.starts_with("shellzip"), "{}", row.name);
            assert_ne!(row.license, "UNKNOWN", "{}", row.name);
        }
        let unrar = rows
            .iter()
            .find(|row| row.name == "unrar_sys")
            .expect("unrar_sys");
        assert!(
            unrar.license.contains("LicenseRef-UnRAR"),
            "{}",
            unrar.license
        );
        assert!(
            rows.iter()
                .any(|row| row.name == "option-ext" && row.license == "MPL-2.0")
        );
    }

    #[test]
    fn every_platform_lists_packaged_licenses() {
        assert_credits(WINDOWS);
        assert_credits(LINUX);
        assert_credits(MACOS);
        assert_credits(credits());
    }
}
