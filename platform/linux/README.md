# Linux packaging and shell integration

## Velopack AppImage

Release CI builds an x86_64 AppImage with Velopack (`packId` `com.shellzip.ShellZip`, channel `linux`). Users download the AppImage, run `chmod +x`, and start it. Auto-updates use the same GitHub Releases feed as Windows/macOS.

Stage and pack locally:

```bash
make release
make velopack-pack    # VELOPACK_CHANNEL=linux
```

Format icons ship beside the binary as `icons/` (freedesktop tree from `grx/shellzip_file_icons`). First launch runs file-manager integrate.

### GPG signatures

When release secrets `GPG_PRIVATE_KEY` / `GPG_KEY_ID` are set, CI publishes `SHA256SUMS`, `SHA256SUMS.asc`, and `*.AppImage.asc`. Verify with the public key documented in [docs/BUILDING.md](../../docs/BUILDING.md) (published as `docs/signing-key.asc` once the release key exists).

## Integrate

```bash
shellzip --integrate
shellzip --unintegrate
```

Writes:

- `~/.local/share/applications/shellzip.desktop` (MIME defaults, launcher actions, `StartupWMClass=shellzip`)
- Application icon `~/.local/share/icons/hicolor/*/apps/shellzip.png` (plus archive MIME icons)
- GNOME Files: `~/.local/share/nautilus-python/extensions/shellzip.py` (GPL-2.0+, generated with the absolute GUI path) and `~/.local/share/nautilus/scripts/ShellZip/`
- KDE Dolphin: service menus under `kio/servicemenus` and `kservices5/ServiceMenus` (Add on every file/folder; Open/Extract on archives)
- Nemo, Caja scripts, PCManFM/libfm actions, and Thunar `~/.config/Thunar/uca.xml` (merged, with stable `unique-id`s)

A top-level **ShellZip** submenu in GNOME Files needs `python3-nautilus` (Debian/Ubuntu) or `nautilus-python`. Without that package, the same verbs appear under **Scripts → ShellZip**. Restart the file manager (`nautilus -q`) after integrate.

The `.desktop` `Icon=shellzip` name is a freedesktop theme icon, not an ELF resource. The raw `shellzip` binary in a folder listing still uses the generic executable glyph; the launcher, dock, window, and context-menu icon are ShellZip.

Manual copy of the Nautilus plugin (uses `shellzip` from `PATH`): [nautilus/shellzip.py](nautilus/shellzip.py).
