//! Directory walks, include/exclude masks, and stable archive ordering.

use std::path::{Path, PathBuf};

use globset::{GlobBuilder, GlobSet, GlobSetBuilder};
use walkdir::WalkDir;

use crate::entry::{EntryKind, EntryMeta, to_archive_path};
use crate::error::Error;
use crate::fs::metadata::capture;
use crate::options::CompressOpts;

/// One filesystem object that will become an archive member.
#[derive(Debug, Clone)]
pub struct Planned {
    /// Source path. `None` for a parent directory synthesized from a child.
    pub source: Option<PathBuf>,
    /// Archive-relative path using `/`, without a trailing slash.
    pub rel: String,
    /// File, directory, or symlink.
    pub kind: EntryKind,
    /// Size and timestamps.
    pub meta: EntryMeta,
    /// Symlink target, when `kind` is [`EntryKind::Symlink`].
    pub link_target: Option<String>,
}

/// Expand `inputs` into a deterministic member list.
pub fn plan(
    inputs: &[PathBuf],
    opts: &CompressOpts,
    skip: Option<&Path>,
) -> Result<Vec<Planned>, Error> {
    if inputs.is_empty() {
        return Err(Error::NoFiles);
    }
    let filters = Filters::new(&opts.includes, &opts.excludes)?;
    let skip = skip.and_then(canonicalish);
    let mut items = Vec::new();
    for input in inputs {
        if !input.exists() {
            return Err(Error::io(
                format!("open {}", input.display()),
                std::io::Error::new(std::io::ErrorKind::NotFound, "not found"),
            ));
        }
        let input_meta = std::fs::symlink_metadata(input)
            .map_err(|err| Error::io(format!("stat {}", input.display()), err))?;
        if input_meta.is_file() || (input_meta.file_type().is_symlink() && !opts.follow_symlinks) {
            push_one(
                input,
                input,
                true,
                opts,
                &filters,
                skip.as_deref(),
                &mut items,
            )?;
            continue;
        }
        let walk = WalkDir::new(input)
            .follow_links(opts.follow_symlinks)
            .max_depth(if opts.recurse { usize::MAX } else { 1 });
        for entry in walk {
            let entry = entry?;
            let path = entry.path();
            let is_root = path == input.as_path();
            push_one(
                input,
                path,
                is_root,
                opts,
                &filters,
                skip.as_deref(),
                &mut items,
            )?;
        }
    }
    synthesize_parents(&mut items);
    if items.is_empty() {
        return Err(Error::NoFiles);
    }
    items.sort_by_key(sort_key);
    collapse_duplicates(items)
}

fn collapse_duplicates(items: Vec<Planned>) -> Result<Vec<Planned>, Error> {
    let mut collapsed: Vec<Planned> = Vec::with_capacity(items.len());
    for item in items {
        if let Some(prev) = collapsed.last_mut()
            && prev.rel == item.rel
        {
            if prev.source.is_none() && item.source.is_some() {
                *prev = item;
                continue;
            }
            if item.source.is_none() {
                continue;
            }
            return Err(Error::Usage(format!(
                "duplicate archive path `{}`",
                prev.rel
            )));
        }
        collapsed.push(item);
    }
    Ok(collapsed)
}

fn push_one(
    root: &Path,
    path: &Path,
    is_root: bool,
    opts: &CompressOpts,
    filters: &Filters,
    skip: Option<&Path>,
    out: &mut Vec<Planned>,
) -> Result<(), Error> {
    if skip.is_some_and(|skip| canonicalish(path).as_deref() == Some(skip)) {
        return Ok(());
    }
    let meta_fs = std::fs::symlink_metadata(path)
        .map_err(|err| Error::io(format!("stat {}", path.display()), err))?;
    let is_link = meta_fs.file_type().is_symlink() && !opts.follow_symlinks;
    let kind = if is_link {
        EntryKind::Symlink
    } else if meta_fs.is_dir() {
        EntryKind::Dir
    } else if meta_fs.is_file() {
        EntryKind::File
    } else {
        return Ok(());
    };

    let rel = archive_rel(root, path, is_root, opts.ep1, kind)?;
    if rel.is_empty() {
        return Ok(());
    }
    if blocked(filters, &rel) {
        return Ok(());
    }
    if kind != EntryKind::Dir && !filters.included(&rel) {
        return Ok(());
    }
    if kind == EntryKind::Dir && !filters.includes_dirs() && !filters.included(&rel) {
        // Directories are kept only when they match, or when there is no include
        // list. Parents of matched files are added later.
        if !opts.includes.is_empty() {
            return Ok(());
        }
    }

    let link_target = if kind == EntryKind::Symlink {
        Some(
            std::fs::read_link(path)
                .map_err(|err| Error::io(format!("read link {}", path.display()), err))?
                .to_string_lossy()
                .into_owned(),
        )
    } else {
        None
    };
    let meta = capture(path, kind)?;
    out.push(Planned {
        source: Some(path.to_path_buf()),
        rel,
        kind,
        meta,
        link_target,
    });
    Ok(())
}

fn archive_rel(
    root: &Path,
    path: &Path,
    is_root: bool,
    ep1: bool,
    kind: EntryKind,
) -> Result<String, Error> {
    if root.is_file() || (is_root && kind != EntryKind::Dir) {
        let name = path
            .file_name()
            .ok_or_else(|| Error::Usage(format!("missing file name for {}", path.display())))?;
        return Ok(name.to_string_lossy().replace('\\', "/"));
    }
    let inside = path.strip_prefix(root).unwrap_or(path);
    let mut rel = String::new();
    if !ep1 && let Some(name) = root.file_name() {
        rel.push_str(&name.to_string_lossy().replace('\\', "/"));
    }
    let rest = to_archive_path(inside);
    if !rest.is_empty() {
        if !rel.is_empty() {
            rel.push('/');
        }
        rel.push_str(&rest);
    }
    if is_root && ep1 {
        return Ok(String::new());
    }
    Ok(rel)
}

fn synthesize_parents(items: &mut Vec<Planned>) {
    let mut extra = Vec::new();
    let known: std::collections::BTreeSet<String> =
        items.iter().map(|item| item.rel.clone()).collect();
    for item in items.iter() {
        if item.kind == EntryKind::Dir {
            continue;
        }
        let mut acc = String::new();
        let parts: Vec<&str> = item.rel.split('/').collect();
        if parts.len() < 2 {
            continue;
        }
        for part in &parts[..parts.len() - 1] {
            if !acc.is_empty() {
                acc.push('/');
            }
            acc.push_str(part);
            if !known.contains(&acc) && !extra.iter().any(|e: &Planned| e.rel == acc) {
                extra.push(Planned {
                    source: None,
                    rel: acc.clone(),
                    kind: EntryKind::Dir,
                    meta: EntryMeta {
                        size: 0,
                        modified: item.meta.modified,
                        mode: 0o755,
                    },
                    link_target: None,
                });
            }
        }
    }
    items.extend(extra);
}

fn sort_key(item: &Planned) -> (u8, String, String) {
    let ext = if item.kind == EntryKind::Dir {
        String::new()
    } else {
        item.rel
            .rsplit('/')
            .next()
            .and_then(|name| name.rsplit_once('.'))
            .map(|(_, ext)| ext.to_ascii_lowercase())
            .unwrap_or_default()
    };
    let group = u8::from(item.kind != EntryKind::Dir);
    (group, ext, item.rel.to_ascii_lowercase())
}

fn canonicalish(path: &Path) -> Option<PathBuf> {
    std::fs::canonicalize(path).ok().or_else(|| {
        if path.is_absolute() {
            Some(path.to_path_buf())
        } else {
            std::env::current_dir().ok().map(|cwd| cwd.join(path))
        }
    })
}

struct Filters {
    include: Option<GlobSet>,
    exclude: GlobSet,
}

impl Filters {
    fn new(includes: &[String], excludes: &[String]) -> Result<Self, Error> {
        Ok(Self {
            include: if includes.is_empty() {
                None
            } else {
                Some(build_set(includes)?)
            },
            exclude: build_set(excludes)?,
        })
    }

    fn includes_dirs(&self) -> bool {
        self.include.is_none()
    }

    fn excluded(&self, rel: &str) -> bool {
        hits(&self.exclude, rel)
    }

    fn included(&self, rel: &str) -> bool {
        self.include.as_ref().is_none_or(|set| hits(set, rel))
    }
}

fn blocked(filters: &Filters, rel: &str) -> bool {
    let mut acc = String::new();
    for part in rel.split('/') {
        if !acc.is_empty() {
            acc.push('/');
        }
        acc.push_str(part);
        if filters.excluded(&acc) {
            return true;
        }
    }
    false
}

fn hits(set: &GlobSet, rel: &str) -> bool {
    let name = rel.rsplit('/').next().unwrap_or(rel);
    set.is_match(rel) || set.is_match(name)
}

fn build_set(patterns: &[String]) -> Result<GlobSet, Error> {
    let mut builder = GlobSetBuilder::new();
    for pattern in patterns {
        let glob = GlobBuilder::new(pattern)
            .case_insensitive(cfg!(windows))
            .literal_separator(true)
            .build()
            .map_err(|err| Error::Usage(format!("invalid mask `{pattern}`: {err}")))?;
        builder.add(glob);
    }
    builder
        .build()
        .map_err(|err| Error::Usage(format!("invalid mask: {err}")))
}
