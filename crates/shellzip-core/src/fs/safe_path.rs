//! Destination paths that cannot escape the output directory.

use std::path::{Component, Path, PathBuf};

use crate::error::Error;

const RESERVED: &[&str] = &["CON", "PRN", "AUX", "NUL"];

/// Validate an archive member name and return its relative path.
pub fn safe_rel(raw: &str) -> Result<PathBuf, Error> {
    if raw.is_empty() || raw.chars().any(char::is_control) {
        return Err(Error::UnsafePath(raw.to_string()));
    }
    let norm = raw.replace('\\', "/");
    if norm.starts_with('/') || norm.starts_with("//") {
        return Err(Error::UnsafePath(raw.to_string()));
    }
    let bytes = norm.as_bytes();
    if bytes.len() >= 2 && bytes[0].is_ascii_alphabetic() && bytes[1] == b':' {
        return Err(Error::UnsafePath(raw.to_string()));
    }

    let mut parts = Vec::new();
    for component in norm.split('/') {
        if component.is_empty() || component == "." {
            continue;
        }
        if component == ".."
            || component.ends_with(' ')
            || component.ends_with('.')
            || is_reserved(component)
        {
            return Err(Error::UnsafePath(raw.to_string()));
        }
        parts.push(component.to_string());
    }
    if parts.is_empty() {
        return Err(Error::UnsafePath(raw.to_string()));
    }
    let mut path = PathBuf::new();
    for part in parts {
        path.push(part);
    }
    Ok(path)
}

/// Join `rel` onto `dest` and reject any result that is not inside `dest`.
pub fn safe_join(dest: &Path, rel: &Path) -> Result<PathBuf, Error> {
    for component in rel.components() {
        if !matches!(component, Component::Normal(_)) {
            return Err(Error::UnsafePath(rel.display().to_string()));
        }
    }
    let root = lexical_absolute(dest)?;
    let mut out = root.clone();
    for component in rel.components() {
        out.push(component);
    }
    let out = lexical_absolute(&out)?;
    if !is_inside(&root, &out) {
        return Err(Error::UnsafePath(rel.display().to_string()));
    }
    Ok(extend_long_path(out))
}

/// True when a symlink target, resolved against `link_parent`, stays inside `dest`.
pub fn symlink_stays_inside(dest: &Path, link_path: &Path, target: &str) -> Result<(), Error> {
    if target.is_empty() || target.chars().any(char::is_control) {
        return Err(Error::UnsafePath(target.to_string()));
    }
    let target_path = Path::new(target);
    if target_path.is_absolute()
        || target.starts_with('/')
        || target.starts_with('\\')
        || looks_like_drive(target)
    {
        return Err(Error::UnsafePath(target.to_string()));
    }
    let parent = link_path.parent().unwrap_or(dest);
    let mut combined = lexical_absolute(parent)?;
    for component in target_path.components() {
        match component {
            Component::Normal(part) => combined.push(part),
            Component::CurDir => {}
            Component::ParentDir => {
                if !combined.pop() {
                    return Err(Error::UnsafePath(target.to_string()));
                }
            }
            _ => return Err(Error::UnsafePath(target.to_string())),
        }
    }
    let root = lexical_absolute(dest)?;
    if !is_inside(&root, &combined) {
        return Err(Error::UnsafePath(target.to_string()));
    }
    Ok(())
}

/// Create `dir` and every missing parent, refusing to traverse an existing symlink.
pub fn ensure_dir(dir: &Path) -> Result<(), Error> {
    let abs = lexical_absolute(dir)?;
    let mut current = PathBuf::new();
    for component in abs.components() {
        current.push(component);
        match std::fs::symlink_metadata(&current) {
            Ok(meta) if meta.file_type().is_symlink() => {
                return Err(Error::UnsafePath(format!(
                    "refusing to follow symlink {}",
                    current.display()
                )));
            }
            Ok(meta) if meta.is_dir() => {}
            Ok(_) => {
                return Err(Error::Usage(format!(
                    "{} exists and is not a directory",
                    current.display()
                )));
            }
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => {
                std::fs::create_dir(extend_long_path(current.clone())).map_err(|err| {
                    Error::io(format!("create directory {}", current.display()), err)
                })?;
            }
            Err(err) => {
                return Err(Error::io(format!("inspect {}", current.display()), err));
            }
        }
    }
    Ok(())
}

fn is_reserved(component: &str) -> bool {
    let stem = component.split('.').next().unwrap_or(component);
    let upper = stem.to_ascii_uppercase();
    if RESERVED.contains(&upper.as_str()) {
        return true;
    }
    let bytes = upper.as_bytes();
    bytes.len() == 4
        && (upper.starts_with("COM") || upper.starts_with("LPT"))
        && bytes[3].is_ascii_digit()
        && bytes[3] != b'0'
}

fn looks_like_drive(text: &str) -> bool {
    let bytes = text.as_bytes();
    bytes.len() >= 2 && bytes[0].is_ascii_alphabetic() && bytes[1] == b':'
}

fn is_inside(root: &Path, path: &Path) -> bool {
    let root_comps: Vec<_> = root.components().collect();
    let path_comps: Vec<_> = path.components().collect();
    if path_comps.len() < root_comps.len() {
        return false;
    }
    root_comps.iter().zip(path_comps.iter()).all(|(a, b)| {
        if cfg!(windows) {
            a.as_os_str()
                .to_string_lossy()
                .eq_ignore_ascii_case(&b.as_os_str().to_string_lossy())
        } else {
            a == b
        }
    })
}

pub(crate) fn lexical_absolute(path: &Path) -> Result<PathBuf, Error> {
    let abs = if path.is_absolute() {
        path.to_path_buf()
    } else {
        std::env::current_dir()
            .map_err(|err| Error::io("current directory", err))?
            .join(path)
    };
    let mut out = PathBuf::new();
    for component in abs.components() {
        match component {
            Component::Prefix(prefix) => out.push(prefix.as_os_str()),
            Component::RootDir => out.push(component),
            Component::CurDir => {}
            Component::ParentDir => {
                if !out.pop() {
                    return Err(Error::UnsafePath(path.display().to_string()));
                }
            }
            Component::Normal(part) => out.push(part),
        }
    }
    Ok(out)
}

pub(crate) fn extend_long_path(path: PathBuf) -> PathBuf {
    #[cfg(windows)]
    {
        let text = path.to_string_lossy().replace('/', "\\");
        if text.len() >= 248 && !text.starts_with(r"\\?\") {
            let extended = if let Some(rest) = text.strip_prefix(r"\\") {
                format!(r"\\?\UNC\{rest}")
            } else {
                format!(r"\\?\{text}")
            };
            return PathBuf::from(extended);
        }
    }
    path
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_escape_and_reserved() {
        assert!(safe_rel("../evil.txt").is_err());
        assert!(safe_rel("foo/../../evil.txt").is_err());
        assert!(safe_rel(r"C:\evil").is_err());
        assert!(safe_rel("/etc/passwd").is_err());
        assert!(safe_rel("dir/CON.txt").is_err());
        assert!(safe_rel("dir/file.").is_err());
        assert!(safe_rel("dir/file ").is_err());
        assert!(safe_rel("ok/file.txt").is_ok());
        assert!(safe_rel(r"ok\nested\file.txt").is_ok());
    }

    #[test]
    fn join_stays_inside() {
        let dest = std::env::temp_dir().join("shellzip-safe-join");
        let rel = safe_rel("a/b.txt").unwrap();
        let joined = safe_join(&dest, &rel).unwrap();
        let root = lexical_absolute(&dest).unwrap();
        assert!(is_inside(&root, &lexical_absolute(&joined).unwrap()));
    }
}
