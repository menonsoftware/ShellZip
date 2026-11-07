# szip command-line reference

`szip` is the ShellZip console tool. Grammar:

```text
szip [global options] <command> [command options] <archive> [files…]
```

WinRAR-style short switches are rewritten before clap parses them (`-pSECRET` → `--password=SECRET`, `-mt8` → `--threads=8`, and so on). `szip --help` and `szip <command> --help` are always current.

## Global options

| Switch | Meaning |
|---|---|
| `-v`, `--verbose` | More logging. Repeat for a wider trace |
| `-q`, `--quiet` | Hide progress. Repeat to hide the summary. `l` and `lt` still print |
| `--ilog FILE`, `-ilog=FILE` | Append a tracing log to `FILE` |
| `--ierr`, `-ierr` | Also write that log to stderr |
| `--password [SECRET]`, `-p`, `-pSECRET` | Password for ZIP AES-256, RAR, and 7z |

Bare `--password` / `-p` prompts without echo. Otherwise `SZIP_PASSWORD` is used. The value is never written to the log or a window title.

## Commands

### `a` — add

```bash
szip a archive.zip src [more…]
szip a archive.7z src -m5 --method lzma2
szip a -y archive.zip src          # replace the archive instead of merging a ZIP
```

| Switch | Meaning |
|---|---|
| `-m`, `--level 0..5` | Compression level. Default `3`. `0` stores |
| `--method` | `deflate` or `zstd` (ZIP); `lzma2`, `lzma`, `ppmd`, `bzip2` (7z). Deflate and Zstd also work for 7z |
| `--ep1` | Drop the top directory name from stored paths |
| `--no-recurse` | Do not descend into directories |
| `--follow-symlinks` | Store the target of a symlink |
| `-x`, `--exclude GLOB` | Exclude archive-relative glob. Repeatable |
| `-n`, `--include GLOB` | Only pack matches. Repeatable |
| `--store-ext LIST` | Semicolon-separated extensions stored uncompressed (replaces the default list) |
| `--no-store-list` | Compress every ZIP member |
| `-y`, `--yes` | Replace an existing archive. Only valid with `a` |
| `--threads N`, `-mtN` | Workers for ZIP entries and zstd. Default = CPU count, cap 64 |
| `--long LOG2`, `-long=LOG2` | Zstd window log, 10–31. Default follows total input size |
| `--sfx`, `-sfx` | Wrap the new archive as a self-extractor. Destination is the executable path |
| `--sfx-stub PATH` | Stub binary for `--sfx` |
| `--sfx-format zip\|7z` | Payload inside the SFX. Default `zip`. 7z is LZMA2 only |
| `--hp`, `-hp` | Encrypt 7z names and metadata. Requires a password. ZIP cannot hide names |
| `--solid`, `-s` | One solid 7z block (default) |
| `--no-solid`, `-s-` | One 7z block per file |
| `--stdin[=NAME]`, `-siNAME` | Read one member from stdin |

An existing ZIP keeps its other files unless `-y` is passed. `u` and `f` share these switches except `-y`.

### `u` / `f` — update / freshen

ZIP and 7z only.

- `u` adds new members and replaces a member when the source is newer
- `f` replaces only members that already exist and are older than the source

### `d` — delete

```bash
szip d archive.zip '*.log' dir/old.txt
```

Rewrites a ZIP or 7z, dropping members that match a name or glob.

### `x` / `e` — extract

```bash
szip x archive.zip -o dest --yes
szip e archive.zip -o dest --or     # flatten; rename on collision
```

`x` recreates directories. `e` writes every file into `--output`.

| Switch | Meaning |
|---|---|
| `-o`, `--output DIR` | Destination. Default `.` |
| `--overwrite ask\|always\|skip\|rename` | Collision policy. Default `ask` (skip when stdin is not a terminal) |
| `--o+` / `-y` | Replace existing files |
| `--o-` | Leave existing files |
| `--or` | Write `name (n).ext` |
| `--kb`, `--keep-broken` | Keep a partial file when decoding fails |
| `--mes`, `--skip-encrypted` | Skip encrypted members instead of failing |
| `--allow-unsafe-links` | Allow symlink targets outside the destination |
| `--om`, `--motw` | Copy the archive `Zone.Identifier` onto extracted files (Windows) |
| `-n`, `--include GLOB` | Extract only matches |
| `--max-size BYTES` | Stop after this many uncompressed bytes. Default 64 GiB |
| `--max-ratio N` | Uncompressed bytes per packed byte. Default 1_000_000. `0` disables |

### `l` / `lt` — list

```bash
szip l archive.zip
szip lt archive.zip --htb
```

`l` prints names. `lt` prints size, packed, ratio, time, attributes, CRC32, method, and host OS. `--htb` / `-htb` adds a BLAKE2sp-256 of each file payload.

### `p` — print

```bash
szip p archive.zip path/inside.txt > out.txt
```

Writes one member to stdout. Omit the member name when the archive has a single file.

### `t` — test

```bash
szip t archive.zip
```

Checks checksums and declared sizes. Accepts `--max-size` and `--max-ratio`.

### `s` — self-extractor

```bash
szip s archive.zip -o payload.exe
szip s payload.exe -o archive.zip     # unwrap
```

A ZIP is wrapped with `shellzip-sfx` and a 24-byte trailer. An SFX package is unwrapped back to a ZIP. `a --sfx` creates one in a single step. Packages of 4 GiB or larger are refused (Windows cannot start them).

`--sfx-stub PATH` selects the stub when wrapping. Search order otherwise: `shellzip-sfx` beside `szip`, then `SZIP_SFX_STUB`.

### `complete` — shell completions

```bash
szip complete bash
szip complete zsh
szip complete fish
szip complete powershell
```

Writes a completion script to stdout.

## Compression levels

`-m` is 0 through 5, not a 7-Zip `mx` value.

| `-m` | Intent |
|---:|---|
| 0 | Store |
| 1 | Fast |
| 3 | Default |
| 5 | Maximum (ZIP deflate maps to level 9) |

## Password and header encryption

| Format | `--password` | `--hp` |
|---|---|---|
| ZIP | AES-256 AE-2 on contents. Names stay visible | Invalid — ZIP cannot hide names |
| 7z | AES-256 on contents | Also encrypts names and metadata |
| RAR | Needed to list/extract encrypted archives | n/a (extract only) |
| TAR / GZ / ZST / XZ / BZ2 | Not accepted | n/a |

## Exit codes

Mapped from `shellzip_core::Error`, matching the RAR convention:

| Code | When |
|---:|---|
| 0 | Success |
| 1 | Warning |
| 2 | Fatal I/O or invalid archive |
| 3 | Checksum mismatch |
| 5 | Write / permission / disk full |
| 6 | Open / not found |
| 7 | Usage, unsafe path, or clap parse error |
| 8 | Out of memory / decoder budget |
| 9 | Could not create the output |
| 10 | No files matched the filters |
| 11 | Password required or rejected |
| 255 | Interrupted |

## Logging

```bash
szip -v x archive.zip -o dest --ilog run.log --ierr
```

`-v` sets `shellzip_core=debug`. `-vv` sets `debug` for all targets. Without `--ilog`, verbose traces go to stderr.

## Examples

```bash
# Pack a tree, drop the top folder name, eight ZIP workers
szip a backup.zip project --ep1 --threads 8 -m5

# Encrypted 7z with hidden names
szip a vault.7z accounts --password --hp --method lzma2

# Extract, replace, copy MOTW on Windows
szip x incoming.zip -o unpacked --yes --om

# Technical list with content hashes
szip lt incoming.zip --htb

# One-step SFX
szip a setup.exe dist --sfx --sfx-format zip

# Completions
szip complete powershell > szip.ps1
```
