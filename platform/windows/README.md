# Windows shell integration

## Binaries

| File | Role |
|---|---|
| `shellzip.exe` | GUI; receives Open / `--add*` / `--extract-*` verbs |
| `shellzip_shell.dll` | Explorer cascaded `IContextMenu` + `IExtractIcon` / `IPersistFile` |
| `grx/shellzip_file_icons/ico/*.ico` | Per-format document icons |

Build:

```bat
cargo build --release -p shellzip-gui
cargo build --release -p shellzip-shell
```

Copy `shellzip_shell.dll` next to `shellzip.exe` (same folder as `target/release`).

## Register (Velopack / portable)

```bat
shellzip --integrate
```

Writes per-user ProgIDs under `HKCU\Software\Classes`, copies icons to `%LOCALAPPDATA%\ShellZip\icons`, copies `shellzip_shell.dll` to `%LOCALAPPDATA%\ShellZip\`, stores `HKCU\Software\ShellZip\ExePath` (so menu verbs can find `shellzip.exe` after the DLL is relocated), and registers the CLSID `{A7C3E5B1-4D2F-4A91-9E6C-8B1F0D2E3A45}` for a cascaded **ShellZip** context menu on every file and folder. Machine registration of the COM server may require elevation. Does not rewrite `UserChoice` hashes; if Windows keeps another default, use Settings → Default apps.

After a cargo rebuild, run `shellzip --integrate` again so `ExePath` and the copied DLL match the new binaries.

## Velopack install

Release CI stages `shellzip.exe`, `szip.exe`, `shellzip-sfx.exe`, `shellzip_shell.dll`, and `icons/` (from `grx/shellzip_file_icons`), then runs `vpk pack` with `packId` `com.shellzip.ShellZip` and channel `win`. Install/update hooks call `--integrate`; uninstall hooks call `--unintegrate`. The Velopack Setup stays **unsigned**.

## Microsoft Store MSIX

Release CI also packs `ShellZip-<tag>-x64.msix` (`make msix` locally). The package includes `shellzip.exe`, `szip.exe`, and `shellzip-sfx.exe` only — not `shellzip_shell.dll`. Archive types and static Extract verbs are declared in `AppxManifest.xml`. Settings live under `%LOCALAPPDATA%\ShellZip\settings.toml`. The Store build skips Velopack hooks and the Integrate menu; updates come from the Store.

Use either the Store package or the Velopack Setup, not both, so associations are not registered twice. See [docs/BUILDING.md](../../docs/BUILDING.md) for Partner Center secrets.

## SmartScreen and Defender

The Velopack Setup and portable zip are **unsigned**. SmartScreen flags an unknown publisher. Defender may also score `shellzip.exe`, `shellzip-sfx.exe`, or `shellzip_shell.dll` as a trojan because of the same shapes: stripped release PEs, the SFX overlay trailer, the in-proc Explorer DLL plus HKLM `InprocServer32` / Approved registration from `--integrate`, and Velopack’s update download. That is a false positive on those heuristics, not evidence of injection or Run-key persistence. Prefer the Store install when available; otherwise allow the path in Defender or build locally with `make release`.

## Windows 11 sparse package

[msix/ShellZipSparse.md](msix/ShellZipSparse.md) describes a future sparse MSIX so `IExplorerCommand` items appear on the compact context menu. The classic `IContextMenu` handler already appears under **Show more options** (and on Windows 10) after `--integrate`. The Store package currently ships associations and static verbs only.
