# ShellZip FAQ

Short answers to questions that come up when using `szip` or the desktop window. Command details are in [CLI.md](CLI.md). Format limits are in [FORMATS.md](FORMATS.md).

## What is ShellZip?

An archive manager written in Rust: a GUI (`shellzip`), a CLI (`szip`), and a self-extracting stub (`shellzip-sfx`). They share `shellzip-core`. It is inspired by WinRAR’s command surface, not a clone, and it never creates RAR files.

## How is this different from 7-Zip or WinRAR?

ShellZip is a new implementation. It uses Rust crates (`zip`, `zesven`, `tar`, `unrar`, …), not `7zxa.dll` or WinRAR’s engine. The CLI accepts many WinRAR-style switches (`-p`, `-mt`, `-sfx`, `-hp`) so muscle memory still works. ZIP and 7z creation, tar/zstd, a native GUI, and per-user file associations with a shell context menu are in scope. Solid recovery records and volume sets are not.

## Why can’t I create a `.rar`?

The UnRAR licence allows extraction only. Recreating the RAR compressor is prohibited. ShellZip will list, test, and extract RAR 4 and RAR 5. Use ZIP or 7z when you need to pack.

## Which formats can I create?

ZIP, 7z, `.tar`, `.tar.gz` / `.tgz`, `.tar.zst` / `.tzst`, `.gz`, and `.zst`. xz and bzip2 are extract-only, including `.tar.xz` and `.tar.bz2`.

## Why are file names still visible in an encrypted ZIP?

ZIP AES-256 (WinZip AE-2) encrypts file **contents**, not the central directory. That is WinRAR `-p`, not `-hp`. Names, sizes, and timestamps stay readable. 7z can hide names with `--hp` (or the matching checkbox in the add dialog).

## How do I pass a password without putting it on the command line?

```bash
szip x secret.zip -o dest --password          # prompt, no echo
# or
set SZIP_PASSWORD=…                           # Windows
export SZIP_PASSWORD=…                        # Unix
szip x secret.zip -o dest
```

`--password=SECRET` works but shows up in process lists and shell history. The password is never written to `-ilog` or a window title.

## How do I make a self-extracting archive?

```bash
szip a payload.exe src --sfx                  # ZIP payload (default)
szip a payload.exe src --sfx --sfx-format 7z  # 7z / LZMA2 payload
szip s archive.zip -o payload.exe             # wrap an existing ZIP
```

The stub is `shellzip-sfx` next to `szip`, or the path in `SZIP_SFX_STUB` / `--sfx-stub`. On Windows, opening the result shows a folder picker. `-s` is silent, `-y` replaces existing files, `-d DIR` sets the destination.

`make release` copies the size-optimized stub into `target/release`.

## Can I update or delete files inside an existing archive?

Yes for **ZIP** and **7z**:

| Action | CLI | GUI |
|---|---|---|
| Add / replace | `szip a` | Add dialog, “Add and replace files” |
| Add new or newer | `szip u` | “Add and update files” |
| Replace only if newer | `szip f` | “Freshen existing files only” |
| Delete | `szip d archive.zip '*.log'` | Select members, Delete |
| Rename | (library / GUI) | Select one name, Rename |

TAR and single-file streams are rewrite-on-create, not in-place editors. `szip a -y` replaces a ZIP instead of merging.

## What does `szip e` do versus `szip x`?

`x` recreates archive directories. `e` writes every file into one folder (flat extract). The GUI Extract dialog has the same choice as **Path mode → Extract without paths**.

## Why did extract refuse a path?

Entry names go through a whitelist before any create or symlink. Absolute paths, `..`, drive letters, NTFS reserved names (`CON`, `NUL`, …), trailing dots/spaces, and control characters are rejected as `refusing unsafe archive path`. That is zip-slip protection. Symlink targets that leave the destination need `--allow-unsafe-links`.

## What are the 64 GiB and 1_000_000:1 limits?

Extract and test stop after **64 GiB** of uncompressed output, or when uncompressed bytes exceed packed bytes by **1_000_000:1**. Those caps exist so a zip bomb cannot fill the disk. Override with `--max-size` and `--max-ratio`. `--max-ratio 0` disables the ratio cap.

## Why was my JPEG stored instead of compressed?

ZIP stores already-compressed types by default (the WinRAR `-ms` list): `7z`, `jpg`, `png` is **not** on that list, but `jpeg`, `jpg`, `mp3`, `zip`, `gz`, `zst`, `rar`, … are. Override with `--store-ext jpg;zip` or compress everything with `--no-store-list`.

## How do threads work?

`--threads N` (or `-mtN`) sizes the ZIP per-entry pool and zstd’s internal threads. Default is the CPU count, capped at 64. 7z parallel compression is on in `szip` and `shellzip`. Workers compress ZIP members independently; one thread merges them in order, so packed bytes match a single-thread run.

## Where does the GUI store settings?

A `settings.toml` under the platform config directory from the `directories` crate (`com` / `ShellZip` / `shellzip`):

| OS | Typical path |
|---|---|
| Windows | `%APPDATA%\ShellZip\shellzip\config\settings.toml` |
| Linux | `~/.config/shellzip/settings.toml` |
| macOS | `~/Library/Application Support/com.ShellZip.shellzip/settings.toml` |

Theme, toolbar size, pack/extract defaults, recent archives, and recent destinations are stored there. Passwords are not.

## Is there an Explorer / Finder context menu?

Yes. **Options → Integrate with the file manager** (or `shellzip --integrate`) registers per-user associations and menus. The ShellZip submenu offers Open, Extract to…, Extract Here, and Extract Here (name). Handlers start `shellzip`; they do not extract inside the file manager. Remove with **Remove integration** or `shellzip --unintegrate`.

- **Windows** — `shellzip_shell.dll` (`IContextMenu`, icon handler). Place the DLL next to `shellzip.exe`. See [platform/windows/README.md](../platform/windows/README.md).
- **Linux** — desktop actions, KDE service menu, Nemo actions, Thunar snippet; optional GPL Nautilus script under [platform/linux/nautilus/](../platform/linux/nautilus/).
- **macOS** — document types and a Finder Sync Swift source under [platform/macos/](../platform/macos/).

## What do the exit codes mean?

They follow the RAR convention:

| Code | Meaning |
|---:|---|
| 0 | Success |
| 1 | Warning |
| 2 | Fatal / bad archive |
| 3 | Checksum mismatch |
| 5 | Write failure |
| 6 | Open / not found |
| 7 | Usage, unsafe path, or clap error |
| 8 | Memory |
| 9 | Could not create output |
| 10 | No files matched |
| 11 | Missing or wrong password |
| 255 | Interrupted |

## Can I pipe data in or out?

Yes, with limits:

- `--stdin` / `-siNAME` packs one member from stdin. `.gz` and `.zst` stream. TAR and ZIP go through a temporary file because their headers store sizes.
- `szip p archive [member]` writes one member to stdout.
- ZIP is not a pipe destination: local headers need seek.

## Why is the SFX stub larger than a tiny installer?

The stub embeds enough of `shellzip-core` to extract ZIP and 7z LZMA2/AES, plus a Windows setup window. It is built with `--profile minisize` (`opt-level = "z"`). On x86_64 Windows the current stub is under the 2.5 MiB cap documented in [BENCH.md](BENCH.md).

## Does ShellZip require `unsafe`?

The workspace forbids `unsafe` in the library, CLI, and GUI. The SFX stub opts out only for Win32 dialog FFI and still denies `unsafe` everywhere else.

## Is this on crates.io?

No. `publish = false` on the workspace. Build from this repository with Cargo or the Makefile.

## How do I run the tests?

```bash
make test          # workspace + optional format features
make ci            # fmt, clippy, tests, cargo deny
```

See [BUILDING.md](BUILDING.md).
