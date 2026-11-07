# Windows shell integration

## Binaries

| File | Role |
|---|---|
| `shellzip.exe` | GUI; receives Open / `--extract-*` verbs |
| `shellzip_shell.dll` | Explorer `IContextMenu` + `IExtractIcon` / `IPersistFile` |
| `grx/shellzip_file_icons/ico/*.ico` | Per-format document icons |

Build:

```bat
cargo build --release -p shellzip-gui
cargo build --release -p shellzip-shell
```

Copy `shellzip_shell.dll` next to `shellzip.exe` (same folder as `target/release`).

## Register

```bat
shellzip --integrate
```

Writes per-user ProgIDs under `HKCU\Software\Classes`, copies icons to `%LOCALAPPDATA%\ShellZip\icons`, and registers the DLL CLSID `{A7C3E5B1-4D2F-4A91-9E6C-8B1F0D2E3A45}` for context menus and compound-extension icons. No administrator rights. Does not rewrite `UserChoice` hashes; if Windows keeps another default, use Settings → Default apps.

## Windows 11 sparse package

[msix/ShellZipSparse.md](msix/ShellZipSparse.md) describes an optional sparse MSIX so `IExplorerCommand` items appear on the compact context menu. The classic `IContextMenu` handler already appears under **Show more options** (and on Windows 10) after `--integrate`.
