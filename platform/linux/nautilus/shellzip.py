# ShellZip Nautilus extension.
#
# Licensed under the GNU General Public License v2.0 or later because it
# loads into Nautilus. It is not part of the ShellZip Rust crates (which
# stay MIT). `shellzip --integrate` writes a copy of this file with an
# absolute executable path. Manual install:
#
#   mkdir -p ~/.local/share/nautilus-python/extensions
#   cp shellzip.py ~/.local/share/nautilus-python/extensions/
#   nautilus -q
#
# Requires the `python3-nautilus` / `nautilus-python` package.
# Nautilus 49+ loads GI 4.1 before extensions run; 43–48 use 4.0; older Files uses 3.0.
#
# SPDX-License-Identifier: GPL-2.0-or-later

import gi

# Request the already-loaded version first. require_version raises when Files
# has imported a different Nautilus GI version.
for _version in ("4.1", "4.0", "3.0"):
    try:
        gi.require_version("Nautilus", _version)
        break
    except ValueError:
        continue
else:
    raise ImportError("Nautilus typelib not found")

from gi.repository import GObject, Nautilus
import os
import subprocess

# --integrate replaces this assignment with the absolute GUI path.
SHELLZIP = "shellzip"

ARCHIVE_SUFFIXES = (
    ".tar.gz",
    ".tar.bz2",
    ".tar.xz",
    ".tar.zst",
    ".tgz",
    ".tbz2",
    ".txz",
    ".tzst",
    ".zipx",
    ".zip",
    ".7z",
    ".rar",
    ".tar",
    ".gz",
    ".bz2",
    ".xz",
    ".zst",
)


def _file_manager_disabled():
    """True after --unintegrate while the packaged copy of this file remains."""
    marker = os.path.join(
        os.path.expanduser("~"), ".config", "shellzip", "file-manager-disabled"
    )
    return os.path.isfile(marker)


def _is_archive(path):
    lower = path.lower()
    return any(lower.endswith(suffix) for suffix in ARCHIVE_SUFFIXES)


def _stem(name):
    lower = name.lower()
    for suffix in ARCHIVE_SUFFIXES:
        if lower.endswith(suffix):
            stem = name[: -len(suffix)]
            return stem or "archive"
    base, _dot, _ext = name.rpartition(".")
    return base or "archive"


def _item(name, label, icon=None):
    kwargs = {"name": name, "label": label}
    if icon:
        kwargs["icon"] = icon
    return Nautilus.MenuItem(**kwargs)


class ShellZipExtension(GObject.GObject, Nautilus.MenuProvider):
    def get_file_items(self, *args):
        if _file_manager_disabled():
            return []
        files = args[-1]
        paths = []
        for file_info in files:
            path = file_info.get_location().get_path()
            if path:
                paths.append(path)
        if not paths:
            return []

        archives = [path for path in paths if _is_archive(path)]
        submenu = Nautilus.Menu()

        add_zip = "Add to ZIP"
        add_7z = "Add to 7z"
        if len(paths) == 1:
            stem = os.path.splitext(os.path.basename(paths[0].rstrip(os.sep)))[0]
            if stem:
                add_zip = f"Add to {stem}.zip"
                add_7z = f"Add to {stem}.7z"

        for label, flag in (
            ("Add to archive...", "--add"),
            (add_zip, "--add-zip"),
            (add_7z, "--add-7z"),
        ):
            item = _item(f"ShellZip::{label}", label)
            item.connect("activate", self._run, flag, paths)
            submenu.append_item(item)

        if archives:
            if len(archives) == 1:
                named = f"Extract Here ({_stem(os.path.basename(archives[0]))})"
            else:
                named = "Extract Here (separate folders)"
            for label, flag in (
                ("Open", None),
                ("Extract to…", "--extract-to"),
                ("Extract Here", "--extract-here"),
                (named, "--extract-here-named"),
            ):
                item = _item(f"ShellZip::{label}", label)
                item.connect("activate", self._run, flag, archives)
                submenu.append_item(item)

        root = _item("ShellZip::Root", "ShellZip", icon="shellzip")
        root.set_submenu(submenu)
        return [root]

    def _run(self, _menu, flag, paths):
        cmd = [SHELLZIP]
        if flag:
            cmd.append(flag)
        cmd.extend(paths)
        subprocess.Popen(cmd)
