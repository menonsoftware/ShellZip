# ShellZip Nautilus extension — optional.
#
# This file is licensed under the GNU General Public License v2.0 or later
# because it loads into Nautilus. It is not part of the ShellZip Rust crates
# (which stay MIT). Install manually:
#
#   mkdir -p ~/.local/share/nautilus-python/extensions
#   cp shellzip.py ~/.local/share/nautilus-python/extensions/
#   nautilus -q
#
# SPDX-License-Identifier: GPL-2.0-or-later

from gi.repository import Nautilus, GObject
import subprocess
import os


def _stem(name: str) -> str:
    lower = name.lower()
    for suffix in (
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
    ):
        if lower.endswith(suffix):
            stem = name[: -len(suffix)]
            return stem or "archive"
    base, _dot, _ext = name.rpartition(".")
    return base or "archive"


class ShellZipExtension(GObject.GObject, Nautilus.MenuProvider):
    def get_file_items(self, *args):
        files = args[-1]
        paths = [f.get_location().get_path() for f in files if not f.is_directory()]
        if not paths:
            return []
        if len(paths) == 1:
            named = f"Extract Here ({_stem(os.path.basename(paths[0]))})"
        else:
            named = "Extract Here (separate folders)"

        submenu = Nautilus.Menu()
        for label, flag in (
            ("Open", None),
            ("Extract to…", "--extract-to"),
            ("Extract Here", "--extract-here"),
            (named, "--extract-here-named"),
        ):
            item = Nautilus.MenuItem(name=f"ShellZip::{label}", label=label)
            item.connect("activate", self._run, flag, paths)
            submenu.append_item(item)

        root = Nautilus.MenuItem(name="ShellZip::Root", label="ShellZip")
        root.set_submenu(submenu)
        return [root]

    def _run(self, _menu, flag, paths):
        cmd = ["shellzip"]
        if flag:
            cmd.append(flag)
        cmd.extend(paths)
        subprocess.Popen(cmd)
