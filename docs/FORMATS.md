# Archive formats

ShellZip identifies containers from **magic bytes**, then falls back to the file extension when creating. A ShellZip trailer after any stub, or a PE image with an archive appended after the stub, is reported as SFX.

## Capability matrix

| Format | Create | List | Test | Extract | Update / delete / rename | Password |
|---|---|---|---|---|---|---|
| ZIP | yes | yes | yes | yes | yes | AES-256 AE-2, contents only |
| 7z | yes (`sevenz`) | yes | yes | yes | yes | AES-256; optional header encryption |
| TAR | yes | yes | yes | yes | no | no |
| TAR.GZ / TGZ | yes | yes | yes | yes | no | no |
| TAR.ZST / TZST | yes | yes | yes | yes | no | no |
| TAR.XZ / TXZ | no | yes | yes | yes | no | no |
| TAR.BZ2 / TBZ2 | no | yes | yes | yes | no | no |
| GZIP | yes | yes | yes | yes | no | no |
| ZSTD | yes | yes | yes | yes | no | no |
| XZ | no | yes | yes | yes | no | no |
| BZIP2 | no | yes | yes | yes | no | no |
| RAR 4 / RAR 5 | **never** | yes (`rar`) | yes | yes | no | yes |
| SFX (ZIP inner) | yes | yes | yes | yes | unwrap with `szip s` | same as ZIP |
| SFX (7z inner) | yes | yes | yes | yes | unwrap with `szip s` | same as 7z |

`szip` and `shellzip` enable `rar`, `sevenz-codecs`, and `sevenz-parallel`. The SFX stub enables `sevenz` only (LZMA2 and AES).

## Creation methods

### ZIP

- **Deflate** — default
- **Zstd** — ZIP method 93 (`--method zstd`)
- **Stored** — level `0`, or members whose extension is on the store list

Default store extensions (WinRAR `-ms`):

```text
7z ace arj bz2 cab gz jpeg jpg lha lz lzh mp3 rar taz
tbz2 tgz txz zst xz z zip zipx tzst
```

`--store-ext jpg;zip` replaces the list. `--no-store-list` compresses everything.

Parallel ZIP: workers compress entries to temporary pieces; one writer merges them in input order. Packed bytes match `--threads 1`.

### 7z

| Method | Notes |
|---|---|
| LZMA2 | Default. Available in a `sevenz`-only build (SFX stub) |
| LZMA | Original LZMA |
| PPMd, BZip2, Deflate, Zstd | Need `sevenz-codecs` (`szip` / `shellzip`) |

Solid archives are the default (`--solid`). `--no-solid` / `-s-` writes one block per file. Solid is closer to one compressed stream; a change to one file rewrites more of the archive.

7z update and rename recompress kept members. ZIP can copy compressed payloads through.

### TAR and streams

TAR preserves mode, mtime, and symlinks. `.tar.gz` and `.tar.zst` wrap the tar stream. `.gz` and `.zst` are single-file containers. xz and bzip2 are decode-only.

`--stdin` streams into `.gz` / `.zst`. TAR and ZIP stage a temporary file because headers record the size.

## Encryption

| Format | Algorithm | Names hidden |
|---|---|---|
| ZIP | AES-256, WinZip AE-2 (`zip` crate `aes-crypto`) | no |
| 7z | AES-256 (`zesven`) | yes, with `--hp` |
| RAR | whatever the archive uses | as stored; extract only |

TAR, gzip, zstd, xz, and bzip2 do not take a password. A password on those formats is a usage error.

Passwords live in the `Password` newtype and are wiped on drop. They do not appear in `Debug`, logs, error text, or window titles.

## Self-extracting packages

Layout:

```text
[ shellzip-sfx stub ][ ZIP or 7z payload ][ 24-byte trailer ]
```

The stub locates the trailer, then extracts the payload. A 7z SFX is limited to LZMA2 so the stub can open it. Create with `szip a --sfx` or **SFX** in the GUI; unwrap with `szip s`. The package must stay under 4 GiB so Windows can load the PE. The Linux stub is the same layout and extracts on its own.

Stub switches: `-s` silent, `-y` replace, `-d DIR` destination. On Windows without a console, a setup window chooses the folder. On Linux without a terminal, zenity or kdialog chooses the folder; if neither is installed, files are written beside the program. Silent mode reads `SZIP_PASSWORD` and does not prompt.

## Detection versus extension

`detect_file` / `detect_bytes` look at signatures (`PK\x03\x04`, `Rar!`, `7z\xBC\xAF\x27\x1C`, gzip/zstd/xz/bzip2 magic, ustar, ShellZip trailer, PE-plus-payload). `format_from_extension` is used when **creating**, so `szip a out.zip …` writes ZIP even if `out.zip` does not exist yet.

When **reading**, content wins. That matches the observed WinRAR behaviour: `rar lt out.zip` can report RAR 5 if the bytes are RAR 5.

## Safety

Archive bytes, names, and declared sizes are untrusted.

**Paths.** Every entry name goes through `safe_rel` then `safe_join` before create, write, permissions, or symlink. Only normal path components are accepted. Rejected: absolute paths, drive letters, `..`, control characters, NTFS reserved names, trailing dots and spaces. Symlink targets must stay inside the destination unless `--allow-unsafe-links`.

**Zip bombs.** Extract and test default to 64 GiB uncompressed (`--max-size`) and a 1_000_000:1 ratio (`--max-ratio`). Streaming uses a fixed buffer; an entry is never `read_to_end` into a `Vec`.

**Mark of the Web.** `--om` / `--motw` copies the archive’s `Zone.Identifier` alternate data stream onto extracted files on Windows.

**Permissions.** Only safe mode bits are restored. setuid, setgid, and world-writable bits from an archive are not applied.

**Atomic writes.** Updates rewrite to a temporary file and rename. Cancellation (`Error::Interrupted`) leaves the original archive in place.

## Feature flags (`shellzip-core`)

| Feature | Effect |
|---|---|
| `rar` | Extract / list / test RAR 4 and RAR 5 via `unrar` |
| `sevenz` | Read and write 7z (LZMA, LZMA2, AES) |
| `sevenz-codecs` | Adds Deflate, BZip2, PPMd, Zstd on top of `sevenz` |
| `sevenz-parallel` | Multi-threaded 7z compression |

Features are additive. A `sevenz`-only build still reads LZMA2 and AES, which is what the SFX stub needs.
