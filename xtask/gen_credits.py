#!/usr/bin/env python3
"""Regenerate crates/shellzip-gui/src/credits.rs from cargo tree + metadata."""

from __future__ import annotations

import json
import os
import re
import subprocess
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]


def run(args: list[str]) -> str:
    return subprocess.check_output(args, cwd=ROOT, text=True, stderr=subprocess.DEVNULL)


def license_for(pkg: dict | None) -> str:
    if pkg is None:
        return "UNKNOWN"
    if pkg["name"] == "unrar_sys":
        return "(MIT OR Apache-2.0) AND LicenseRef-UnRAR"
    lic = (pkg.get("license") or "").replace("\n", " ").strip()
    if not lic and pkg.get("license_file"):
        return "SEE LICENSE FILE"
    return lic or "UNKNOWN"


def packages_for_target(meta: dict, target: str) -> list[tuple[str, str, str]]:
    out = run(
        [
            "cargo",
            "tree",
            "-p",
            "shellzip-gui",
            "-e",
            "normal",
            "--target",
            target,
            "--prefix",
            "none",
            "--format",
            "{p}",
        ]
    )
    by_nv = {(p["name"], p["version"]): p for p in meta["packages"]}
    seen: set[tuple[str, str]] = set()
    rows: list[tuple[str, str, str]] = []
    for line in out.splitlines():
        line = line.strip()
        match = re.match(r"^(\S+)\s+v([^\s]+)", line)
        if not match:
            continue
        name, version = match.group(1), match.group(2)
        if name.startswith("shellzip"):
            continue
        key = (name, version)
        if key in seen:
            continue
        seen.add(key)
        rows.append((name, version, license_for(by_nv.get(key))))
    rows.sort(key=lambda row: (row[0].lower(), row[1]))
    return rows


def emit_array(name: str, rows: list[tuple[str, str, str]]) -> str:
    lines = [f"pub static {name}: &[Credit] = &["]
    for crate, version, license in rows:
        lic = license.replace("\\", "\\\\").replace('"', '\\"')
        lines.extend(
            [
                "    Credit {",
                f'        name: "{crate}",',
                f'        version: "{version}",',
                f'        license: "{lic}",',
                "    },",
            ]
        )
    lines.append("];")
    return "\n".join(lines)


def main() -> None:
    os.chdir(ROOT)
    meta = json.loads(run(["cargo", "metadata", "--format-version", "1", "--all-features"]))
    targets = {
        "WINDOWS": "x86_64-pc-windows-msvc",
        "LINUX": "x86_64-unknown-linux-gnu",
        "MACOS": "aarch64-apple-darwin",
    }
    all_rows = {name: packages_for_target(meta, target) for name, target in targets.items()}
    for name, rows in all_rows.items():
        print(f"{name}: {len(rows)} packages")
        unknown = [row for row in rows if row[2] == "UNKNOWN"]
        if unknown:
            print("  UNKNOWN:", unknown)

    cfgs = {
        "WINDOWS": '#[cfg_attr(not(windows), allow(dead_code))] // listed in the About box on Windows\n',
        "LINUX": '#[cfg_attr(not(target_os = "linux"), allow(dead_code))] // listed in the About box on Linux\n',
        "MACOS": '#[cfg_attr(not(target_os = "macos"), allow(dead_code))] // listed in the About box on macOS\n',
    }
    parts = [
        """//! Crates linked into the ShellZip window, with the license each one declares.
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
"""
    ]
    for name in ("WINDOWS", "LINUX", "MACOS"):
        parts.append(cfgs[name] + emit_array(name, all_rows[name]))
        parts.append("")
    parts.append(
        """
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
"""
    )
    out = ROOT / "crates" / "shellzip-gui" / "src" / "credits.rs"
    out.write_text("\n".join(parts), encoding="utf-8")
    print("wrote", out)


if __name__ == "__main__":
    main()
