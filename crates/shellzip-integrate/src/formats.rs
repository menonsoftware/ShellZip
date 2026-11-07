//! Archive extensions, MIME types, and per-format icon names.

use std::path::Path;

/// One archive format ShellZip registers with the file manager.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ArchiveKind {
    /// File-name suffix without the leading dot (`zip`, `tar.gz`).
    pub extension: &'static str,
    /// Drawing stem under `grx/shellzip_file_icons` (`zip`, `tar.gz`).
    pub icon: &'static str,
    /// Primary freedesktop MIME type.
    pub mime: &'static str,
    /// Windows ProgID (`ShellZip.zip`).
    pub prog_id: &'static str,
    /// Friendly type name shown in Explorer / Finder.
    pub type_name: &'static str,
}

/// Formats registered for open-with, icons, and context menus. No `.exe`.
pub const EXTENSIONS: &[ArchiveKind] = &[
    ArchiveKind {
        extension: "zip",
        icon: "zip",
        mime: "application/zip",
        prog_id: "ShellZip.zip",
        type_name: "ZIP Archive",
    },
    ArchiveKind {
        extension: "zipx",
        icon: "zip",
        mime: "application/zip",
        prog_id: "ShellZip.zipx",
        type_name: "ZIPX Archive",
    },
    ArchiveKind {
        extension: "7z",
        icon: "7z",
        mime: "application/x-7z-compressed",
        prog_id: "ShellZip.7z",
        type_name: "7-Zip Archive",
    },
    ArchiveKind {
        extension: "rar",
        icon: "rar",
        mime: "application/vnd.rar",
        prog_id: "ShellZip.rar",
        type_name: "RAR Archive",
    },
    ArchiveKind {
        extension: "tar",
        icon: "tar",
        mime: "application/x-tar",
        prog_id: "ShellZip.tar",
        type_name: "TAR Archive",
    },
    ArchiveKind {
        extension: "gz",
        icon: "gz",
        mime: "application/gzip",
        prog_id: "ShellZip.gz",
        type_name: "GZIP Archive",
    },
    ArchiveKind {
        extension: "tgz",
        icon: "tar.gz",
        mime: "application/x-compressed-tar",
        prog_id: "ShellZip.tgz",
        type_name: "Gzipped TAR Archive",
    },
    ArchiveKind {
        extension: "bz2",
        icon: "bz2",
        mime: "application/x-bzip2",
        prog_id: "ShellZip.bz2",
        type_name: "BZIP2 Archive",
    },
    ArchiveKind {
        extension: "tbz2",
        icon: "tar.bz2",
        mime: "application/x-bzip-compressed-tar",
        prog_id: "ShellZip.tbz2",
        type_name: "Bzipped TAR Archive",
    },
    ArchiveKind {
        extension: "xz",
        icon: "xz",
        mime: "application/x-xz",
        prog_id: "ShellZip.xz",
        type_name: "XZ Archive",
    },
    ArchiveKind {
        extension: "txz",
        icon: "tar.xz",
        mime: "application/x-xz-compressed-tar",
        prog_id: "ShellZip.txz",
        type_name: "XZed TAR Archive",
    },
    ArchiveKind {
        extension: "zst",
        icon: "zst",
        mime: "application/zstd",
        prog_id: "ShellZip.zst",
        type_name: "Zstandard Archive",
    },
    ArchiveKind {
        extension: "tzst",
        icon: "tar.zst",
        mime: "application/x-zstd-compressed-tar",
        prog_id: "ShellZip.tzst",
        type_name: "Zstd TAR Archive",
    },
];

/// Unique MIME types for the `.desktop` file and `xdg-mime default`.
pub const MIME_TYPES: &[&str] = &[
    "application/zip",
    "application/x-7z-compressed",
    "application/vnd.rar",
    "application/x-rar-compressed",
    "application/gzip",
    "application/x-bzip2",
    "application/x-xz",
    "application/zstd",
    "application/x-tar",
    "application/x-compressed-tar",
    "application/x-bzip-compressed-tar",
    "application/x-xz-compressed-tar",
    "application/x-zstd-compressed-tar",
];

/// Drawing stem used by `grx/shellzip_file_icons`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct FormatIcon(pub &'static str);

/// Icon for a bare extension token without the leading dot.
pub fn icon_for_extension(ext: &str) -> Option<FormatIcon> {
    let lower = ext.to_ascii_lowercase();
    EXTENSIONS
        .iter()
        .find(|kind| kind.extension == lower)
        .map(|kind| FormatIcon(kind.icon))
}

/// Icon for a file path, honouring compound suffixes such as `.tar.gz`.
pub fn icon_for_path(path: &Path) -> Option<FormatIcon> {
    let name = path.file_name()?.to_str()?.to_ascii_lowercase();
    if name.ends_with(".tar.gz") {
        return Some(FormatIcon("tar.gz"));
    }
    if name.ends_with(".tar.bz2") {
        return Some(FormatIcon("tar.bz2"));
    }
    if name.ends_with(".tar.xz") {
        return Some(FormatIcon("tar.xz"));
    }
    if name.ends_with(".tar.zst") {
        return Some(FormatIcon("tar.zst"));
    }
    let ext = path.extension()?.to_str()?;
    icon_for_extension(ext)
}

/// Registered kind for a path, using the last suffix for ProgID lookups.
pub fn kind_for_path(path: &Path) -> Option<&'static ArchiveKind> {
    let ext = path.extension()?.to_str()?.to_ascii_lowercase();
    EXTENSIONS.iter().find(|kind| kind.extension == ext)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn maps_extensions_to_drawings() {
        let cases: &[(&str, &str)] = &[
            ("zip", "zip"),
            ("zipx", "zip"),
            ("7z", "7z"),
            ("rar", "rar"),
            ("tar", "tar"),
            ("gz", "gz"),
            ("tgz", "tar.gz"),
            ("bz2", "bz2"),
            ("tbz2", "tar.bz2"),
            ("xz", "xz"),
            ("txz", "tar.xz"),
            ("zst", "zst"),
            ("tzst", "tar.zst"),
        ];
        for (ext, icon) in cases {
            assert_eq!(icon_for_extension(ext).map(|i| i.0), Some(*icon), "{ext}");
        }
        assert!(icon_for_extension("exe").is_none());
    }

    #[test]
    fn compound_paths_use_tar_drawings() {
        assert_eq!(
            icon_for_path(&PathBuf::from("backup.tar.gz")).map(|i| i.0),
            Some("tar.gz")
        );
        assert_eq!(
            icon_for_path(&PathBuf::from("a.TAR.BZ2")).map(|i| i.0),
            Some("tar.bz2")
        );
        assert_eq!(
            icon_for_path(&PathBuf::from("plain.gz")).map(|i| i.0),
            Some("gz")
        );
        assert_eq!(
            icon_for_path(&PathBuf::from("pack.zipx")).map(|i| i.0),
            Some("zip")
        );
    }
}
