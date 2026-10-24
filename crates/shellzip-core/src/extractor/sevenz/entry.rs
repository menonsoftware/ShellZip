//! Map a 7z header onto ShellZip [`Entry`] values.
//!
//! Method names come from the folder coders. `zesven` keeps the folder index
//! of each file crate-private, so the mapping follows the 7z rule: files with
//! a stream take the next unpack stream, and each folder owns a run of them.

use std::collections::HashMap;

use zesven::Entry as SevenEntry;
use zesven::format::parser::ArchiveHeader;
use zesven::format::streams::{Coder, Folder};

use crate::entry::{Entry, EntryKind};

/// Label, dictionary size, packed size, and whether the folder holds more than one file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct MethodInfo {
    /// Coders joined with `+`, in header order.
    pub label: String,
    /// LZMA or LZMA2 dictionary, when the coder properties record one.
    pub dictionary: Option<u64>,
    /// Packed size when this folder holds one file and one packed stream.
    pub packed_size: Option<u64>,
    /// The folder compresses more than one file together.
    pub solid: bool,
}

/// One label per archive path, without a trailing slash.
pub(super) fn methods_by_name(header: &ArchiveHeader) -> HashMap<String, MethodInfo> {
    let folders = header.folders();
    let counts = stream_counts(header, folders.len());
    let packed = packed_sizes(header);
    let mut folder_for_stream = Vec::new();
    for (index, count) in counts.iter().copied().enumerate() {
        for _ in 0..count {
            folder_for_stream.push(index);
        }
    }
    let mut stream = 0usize;
    let mut out = HashMap::new();
    for file in header.entries() {
        if !file.has_stream {
            continue;
        }
        let Some(folder_index) = folder_for_stream.get(stream).copied() else {
            stream += 1;
            continue;
        };
        stream += 1;
        let Some(folder) = folders.get(folder_index) else {
            continue;
        };
        let streams_here = counts.get(folder_index).copied().unwrap_or(1);
        out.insert(
            entry_key(&file.name),
            MethodInfo {
                label: folder_label(folder),
                dictionary: folder_dictionary(folder),
                packed_size: single_packed(folder, streams_here, &packed),
                solid: streams_here > 1,
            },
        );
    }
    out
}

/// ShellZip entry for one listed member. `method` is missing for directories.
pub(super) fn entry_from(
    file: &SevenEntry,
    method: Option<&MethodInfo>,
    archive_solid: bool,
) -> Entry {
    let path = entry_key(file.path.as_str());
    let kind = if file.is_symlink {
        EntryKind::Symlink
    } else if file.is_directory {
        EntryKind::Dir
    } else if file.is_anti {
        EntryKind::Other
    } else {
        EntryKind::File
    };
    Entry {
        path,
        kind,
        size: Some(file.size),
        packed_size: method.and_then(|info| info.packed_size),
        modified: file.modified(),
        mode: file.unix_mode(),
        crc32: file.crc32,
        method: method
            .map(|info| info.label.clone())
            .or_else(|| (kind == EntryKind::Dir || file.size == 0).then(|| "-".to_string())),
        host_os: host_os(file),
        blake2sp: None,
        link_target: None,
        encrypted: file.is_encrypted,
        solid: method.map(|info| info.solid).unwrap_or(archive_solid),
        dictionary: method.and_then(|info| info.dictionary),
    }
}

fn stream_counts(header: &ArchiveHeader, folders: usize) -> Vec<u64> {
    header
        .substreams_info
        .as_ref()
        .map(|info| info.num_unpack_streams_in_folders.clone())
        .filter(|counts| !counts.is_empty())
        .unwrap_or_else(|| vec![1; folders])
}

fn packed_sizes(header: &ArchiveHeader) -> Vec<u64> {
    header
        .pack_info
        .as_ref()
        .map(|info| info.pack_sizes.clone())
        .unwrap_or_default()
}

fn single_packed(folder: &Folder, streams: u64, packed: &[u64]) -> Option<u64> {
    if streams != 1 || folder.packed_streams.len() != 1 {
        return None;
    }
    let index = usize::try_from(folder.packed_streams[0]).ok()?;
    packed.get(index).copied()
}

fn folder_label(folder: &Folder) -> String {
    if folder.coders.is_empty() {
        return "-".to_string();
    }
    folder
        .coders
        .iter()
        .map(coder_label)
        .collect::<Vec<_>>()
        .join("+")
}

fn folder_dictionary(folder: &Folder) -> Option<u64> {
    folder.coders.iter().find_map(coder_dictionary)
}

/// Short name for a coder id. Unknown ids are hex.
pub(super) fn coder_label(coder: &Coder) -> String {
    method_name(&coder.method_id)
}

fn coder_dictionary(coder: &Coder) -> Option<u64> {
    let props = coder.properties.as_deref()?;
    match coder.method_id.as_slice() {
        [0x21] => lzma2_dictionary(props),
        [0x03, 0x01, 0x01] => lzma_dictionary(props),
        _ => None,
    }
}

/// LZMA2 dictionary from the one-byte property. 40 means the full 4 GiB − 1 range.
fn lzma2_dictionary(props: &[u8]) -> Option<u64> {
    let prop = *props.first()?;
    if prop > 40 {
        return None;
    }
    if prop == 40 {
        return Some(u64::from(u32::MAX));
    }
    let shift = u32::from(prop / 2 + 11);
    Some(u64::from(2 | (prop & 1)) << shift)
}

/// LZMA dictionary is the little-endian `u32` after the properties byte.
fn lzma_dictionary(props: &[u8]) -> Option<u64> {
    let bytes: [u8; 4] = props.get(1..5)?.try_into().ok()?;
    Some(u64::from(u32::from_le_bytes(bytes)))
}

fn method_name(id: &[u8]) -> String {
    match id {
        [0x00] => "Copy".to_string(),
        [0x21] => "LZMA2".to_string(),
        [0x03, 0x01, 0x01] => "LZMA".to_string(),
        [0x03, 0x04, 0x01] => "PPMd".to_string(),
        [0x04, 0x01, 0x08] => "Deflate".to_string(),
        [0x04, 0x02, 0x02] => "BZip2".to_string(),
        [0x04, 0xF7, 0x11, 0x01] => "Zstd".to_string(),
        [0x04, 0xF7, 0x11, 0x04] => "LZ4".to_string(),
        [0x04, 0xF7, 0x11, 0x02] => "Brotli".to_string(),
        [0x06, 0xF1, 0x07, 0x01] => "AES".to_string(),
        [0x03, 0x03, 0x01, 0x03] => "BCJ".to_string(),
        [0x03, 0x03, 0x05, 0x01] => "ARM".to_string(),
        [0x0A] => "ARM64".to_string(),
        [0x03, 0x03, 0x07, 0x01] => "ARMT".to_string(),
        [0x03, 0x03, 0x02, 0x05] => "PPC".to_string(),
        [0x03, 0x03, 0x08, 0x05] => "SPARC".to_string(),
        [0x03, 0x03, 0x04, 0x01] => "IA64".to_string(),
        [0x0B] => "RISCV".to_string(),
        [0x03, 0x03, 0x01, 0x1B] => "BCJ2".to_string(),
        [0x03] => "Delta".to_string(),
        other => {
            let hex = other
                .iter()
                .map(|byte| format!("{byte:02X}"))
                .collect::<String>();
            format!("0x{hex}")
        }
    }
}

fn host_os(file: &SevenEntry) -> Option<String> {
    if file.unix_mode().is_some() {
        Some("Unix".to_string())
    } else if file.attributes.is_some() {
        Some("Windows".to_string())
    } else {
        None
    }
}

pub(super) fn entry_key(name: &str) -> String {
    name.trim_end_matches(['/', '\\']).replace('\\', "/")
}

/// Borrowed view of the fields the extract loop needs after the iterator advances.
pub(crate) struct Member {
    pub name: String,
    pub size: u64,
    pub is_directory: bool,
    pub is_symlink: bool,
    pub is_anti: bool,
    pub encrypted: bool,
    pub modified: Option<std::time::SystemTime>,
}

impl Member {
    pub(crate) fn from_entry(file: &SevenEntry) -> Self {
        Self {
            name: entry_key(file.path.as_str()),
            size: file.size,
            is_directory: file.is_directory,
            is_symlink: file.is_symlink,
            is_anti: file.is_anti,
            encrypted: file.is_encrypted,
            modified: file.modified(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use zesven::format::files::{ArchiveEntry as HeaderFile, FilesInfo};
    use zesven::format::streams::{BindPair, PackInfo, SubStreamsInfo, UnpackInfo};

    fn coder(id: &[u8]) -> Coder {
        Coder {
            method_id: id.to_vec(),
            num_in_streams: 1,
            num_out_streams: 1,
            properties: None,
        }
    }

    fn folder(id: &[u8], packed_index: u64) -> Folder {
        Folder {
            coders: vec![coder(id)],
            bind_pairs: Vec::<BindPair>::new(),
            packed_streams: vec![packed_index],
            unpack_sizes: vec![4],
            unpack_crc: None,
        }
    }

    fn file(name: &str, has_stream: bool) -> HeaderFile {
        HeaderFile {
            name: name.to_string(),
            is_directory: !has_stream,
            is_anti: false,
            has_stream,
            size: if has_stream { 4 } else { 0 },
            crc: None,
            ctime: None,
            atime: None,
            mtime: None,
            attributes: None,
        }
    }

    #[test]
    fn method_names_cover_the_common_coder_ids() {
        let cases: &[(&[u8], &str)] = &[
            (&[0x21], "LZMA2"),
            (&[0x03, 0x01, 0x01], "LZMA"),
            (&[0x03, 0x04, 0x01], "PPMd"),
            (&[0x04, 0x01, 0x08], "Deflate"),
            (&[0x04, 0x02, 0x02], "BZip2"),
            (&[0x04, 0xF7, 0x11, 0x01], "Zstd"),
            (&[0x06, 0xF1, 0x07, 0x01], "AES"),
            (&[0x03, 0x03, 0x01, 0x03], "BCJ"),
            (&[0x03, 0x03, 0x01, 0x1B], "BCJ2"),
            (&[0x03], "Delta"),
            (&[0x00], "Copy"),
            (&[0x0A], "ARM64"),
            (&[0x0B], "RISCV"),
        ];
        for (id, label) in cases {
            assert_eq!(method_name(id), *label, "{id:?}");
        }
        assert_eq!(method_name(&[0xAB, 0xCD]), "0xABCD");
    }

    #[test]
    fn lzma2_dictionary_follows_the_property_byte() {
        assert_eq!(lzma2_dictionary(&[40]), Some(u64::from(u32::MAX)));
        assert_eq!(lzma2_dictionary(&[0]), Some(1 << 12));
        assert_eq!(lzma2_dictionary(&[41]), None);
        assert_eq!(
            lzma_dictionary(&[0x5D, 0x00, 0x00, 0x00, 0x01]),
            Some(1 << 24)
        );
    }

    #[test]
    fn streams_take_folder_methods_in_order() {
        let header = ArchiveHeader {
            pack_info: Some(PackInfo {
                pack_pos: 0,
                pack_sizes: vec![10, 20],
                pack_crcs: vec![None, None],
            }),
            unpack_info: Some(UnpackInfo {
                folders: vec![folder(&[0x21], 0), folder(&[0x04, 0x01, 0x08], 1)],
            }),
            substreams_info: Some(SubStreamsInfo {
                num_unpack_streams_in_folders: vec![2, 1],
                unpack_sizes: vec![4, 4, 4],
                digests: vec![None, None, None],
            }),
            files_info: Some(FilesInfo {
                entries: vec![
                    file("dir", false),
                    file("a.txt", true),
                    file("b.txt", true),
                    file("c.txt", true),
                ],
                comment: None,
            }),
            header_encrypted: false,
        };
        let methods = methods_by_name(&header);
        assert!(!methods.contains_key("dir"));
        assert_eq!(methods["a.txt"].label, "LZMA2");
        assert!(methods["a.txt"].solid);
        assert_eq!(methods["a.txt"].packed_size, None, "shared folder");
        assert_eq!(methods["b.txt"].label, "LZMA2");
        assert_eq!(methods["c.txt"].label, "Deflate");
        assert!(!methods["c.txt"].solid);
        assert_eq!(methods["c.txt"].packed_size, Some(20));
    }
}
