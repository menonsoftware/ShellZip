# ShellZip — Rust Archive Manager MVP Plan

Derived from the WinRAR 7.23 analysis in `REA_WinRAR_Report.md`. Design choices that copy an *observed* WinRAR trait are marked **[observed]**; everything else is our own decision.

---

## 1. Scope & feature‑parity matrix

### Design principles carried over from WinRAR

| WinRAR trait | Evidence | ShellZip decision |
|---|---|---|
| One statically linked core, three front‑ends (GUI / CLI / SFX / shell ext) **[observed]** | identical import profiles & strings across `Rar.exe`, `UnRAR.exe`, `*.SFX` | One `shellzip-core` library crate; `cli`, `gui`, `sfx-stub` are thin binaries over it |
| SFX = unsigned stub PE ‖ appended archive **[observed]** | byte‑identical 480,768‑byte prefix, `Rar!` at stub size | `sfx-stub` binary + archive + 24‑byte trailer; stub finds its own tail |
| Console tool is RAR‑only; ZIP creation is GUI‑only **[observed]** | `rar a x.zip` → RAR5 | Our CLI creates ZIP/TAR/ZST from day 1 (we are not bound by that split) |
| Foreign formats delegated to a decoder plug‑in (`7zxa.dll`) **[observed]** | exports `CreateDecoder…`, string only in `WinRAR.exe` | `format::Backend` trait; 7z/RAR via separate crates behind Cargo features |
| Shell extension is a thin launcher for the GUI **[inferred]** | 3 ADVAPI imports, `ShellExecuteExW`, `winrar.exe` string | Out of MVP; documented as Phase 5 (`IExplorerCommand` via `windows` crate) |
| Hand‑rolled worker threads, `-mt1..64` **[observed]** | events/semaphores/SRW, no ThreadPool API | `rayon` pool sized by `--threads` |
| CRC32 default, BLAKE2sp optional `-htb` **[observed]** | `rar lt` output | `crc32fast` always; `blake2` for `lt`‑style listing & verify |
| Header encryption `-hp` vs data‑only `-p` **[observed]** | `Details: encrypted headers`, exit 11 | ZIP AES‑256 encrypts data only; header encryption = documented *non‑goal* (ZIP cannot) |
| Zip‑slip / MOTW hardening (`Zone.Identifier`, `-om`) **[observed strings]** | UTF‑16 strings | Mandatory `enclosed_name` checks; optional MOTW ADS write on Windows |
| Store already‑compressed types `-ms` (list in manual) **[observed]** | `Rar.txt` | Same default extension list → `Stored` method |
| Auto dictionary shrink for small inputs **[observed]** | `-md=128k` for 16 KB input | For zstd: `--long` window chosen from total input size |

### Parity matrix

| Capability | WinRAR | ShellZip MVP | Notes |
|---|---|---|---|
| Create ZIP (Deflate / Stored / Zstd method 93) | GUI yes | **M1** | `zip` crate; Zstd method optional |
| Create `.tar`, `.tar.gz`, `.tar.zst` | no (extract only) | **M1** | `tar` + `flate2` + `zstd` |
| Extract ZIP / TAR / GZ / ZST / XZ / BZ2 | yes | **M1** (ZIP, TAR, GZ, ZST), XZ/BZ2 **M2** | `xz2`, `bzip2` feature‑gated |
| Extract and create 7z | via `7zxa.dll` (extract) | **done** (feature `sevenz`) | `zesven`: LZMA2 and AES in `sevenz`; Deflate, BZip2, PPMd, and Zstd in `sevenz-codecs` |
| Extract RAR4/RAR5 (incl. password, solid, multivolume) | yes | **M3** (feature `rar`) | `unrar` crate (binds UnRAR C++; license = extraction only) |
| **Create RAR** | yes | **Never** | RAR encoder is proprietary; UnRAR licence forbids re‑creating the RAR compressor |
| Password protection | AES‑256, `-p`/`-hp` | **M3** ZIP AES‑256 (AE‑2) | `zip` feature `aes-crypto` |
| List / technical list (`l`, `lt`) incl. hashes | yes | **M1** (`l`), **M2** (`lt` with CRC32/BLAKE2) | |
| Test archive (`t`) | yes | **M1** | |
| Update / freshen / delete (`u`,`f`,`d`) | yes | **M2** (ZIP only, rewrite strategy) | TAR streams are append‑only → rewrite |
| Multithreaded compression `-mt` | yes | **M2** | per‑entry parallel ZIP; zstd internal MT |
| Streaming from stdin / to stdout (`-si`, `p`) | yes | **M2** | TAR/ZST only (ZIP needs seek) |
| Progress + ETA | yes | **M1** | `indicatif` |
| Solid archives, recovery record, volumes `-v`, `-rr`, `-rv`, `-oi`, `-qo` | yes | **No** | out of MVP; note TAR.ZST is inherently "solid" |
| SFX creation (`-sfx`) | yes | **M4** (ZIP‑based, Windows + Linux stubs) | |
| GUI manager | yes | **M4** (optional, `eframe`) | |
| NTFS streams / ACLs / hard links (`-os -ow -oh`) | yes | **No** (symlinks only) | |
| Explorer shell extension | yes | **Phase 5 (post‑MVP)** | |

---

## 2. Recommended crate ecosystem

| Layer | Crate | Why |
|---|---|---|
| ZIP | `zip` (features: `deflate`, `zstd`, `aes-crypto`, `time`) | Read/write, AES‑256 AE‑2, handles prepended data (needed for SFX), `enclosed_name()` for zip‑slip |
| Deflate | `flate2` (default `miniz_oxide`, or `zlib-rs` backend for speed) | `.gz`, ZIP deflate |
| TAR | `tar` | Streaming builder/reader, preserves mode/mtime/symlinks |
| Zstd | `zstd` (feature `zstdmt`) | Built‑in multithreading, long‑distance matching (`--long`) ≈ WinRAR `-mcl` |
| XZ / BZ2 (M2) | `xz2`, `bzip2` | Extract parity with WinRAR's foreign‑format list |
| RAR (M3) | `unrar` | Safe wrapper over UnRAR; extract/list/test only |
| 7z | `zesven` | Pure Rust 7z reader and writer (LZMA/LZMA2/AES by default; more codecs and threads behind features). Replaces `sevenz-rust2`. |
| Format sniffing | own `detect.rs` (magic bytes) + `infer` as fallback | WinRAR sniffs by signature, not extension **[observed: `rar lt out.zip` → RAR 5]** |
| CLI | `clap` (derive) + `clap_complete` | `rar`‑like `command + switches` grammar |
| Walk / masks | `walkdir`, `globset`, `ignore` (for `.gitignore` awareness, optional) | `-r`, `-x<mask>`, `-n<mask>` |
| Parallelism | `rayon` | per‑entry compression pool (`--threads`, default = available_parallelism) |
| Hashing | `crc32fast`, `blake2` (BLAKE2sp), `sha2` | listing/verify parity with `-ht` |
| Crypto (non‑ZIP envelope, optional) | `aes-gcm`, `argon2` / `pbkdf2`, `hmac`, `zeroize` | only if we add encrypted `.tar.zst` envelope later; ZIP AES comes from `zip` |
| Progress / terminal | `indicatif`, `console`, `anstream` | bars, ETA, colour‑safe |
| Errors / logging | `thiserror` (core), `anyhow` (bins), `tracing` + `tracing-subscriber` | `-ilog`‑style file log via `tracing-appender` |
| FS metadata | `filetime`, `same-file`, `tempfile` | restore mtime; atomic writes (temp + rename) |
| Memory‑mapped read (M2) | `memmap2` | large file hashing/verification |
| Windows specifics | `windows-sys` (MOTW ADS, long paths), `winres`/`embed-resource` (icon + manifest) | parity with `-om`, `dpiAware` manifest **[observed]** |
| GUI (M4) | `eframe`/`egui`, `rfd` (native dialogs), `egui_extras` (tables) | small, cross‑platform, drag‑drop support |
| Testing | `assert_cmd`, `predicates`, `tempfile`, `proptest`, `insta` (listing snapshots) | end‑to‑end CLI tests |

---

## 3. Project structure

```
ShellZip/
├── Cargo.toml                      # [workspace] members = crates/*, resolver = "2"
├── rust-toolchain.toml             # stable, pinned
├── .cursor/rules/rust.mdc          # conventions: no unwrap in core, clippy -D warnings, doc every pub fn
├── docs/                           # this plan + REA report
├── fixtures/                       # committed test archives (zip/tar/rar5/rar4/7z, sfx) + SHA256SUMS
├── crates/
│   ├── shellzip-core/              # lib: everything format‑ and IO‑related, no UI
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── error.rs            # Error enum (thiserror) + exit‑code mapping (rar‑like: 0 ok, 1 warn, 2 fatal, 3 bad crc, 10 no files, 11 bad password…)
│   │       ├── detect.rs           # magic‑byte sniffing → Format { Zip, Tar{comp}, Gz, Zst, Xz, Bz2, Rar4, Rar5, SevenZ, Sfx{inner} }
│   │       ├── entry.rs            # Entry { path, kind, size, mtime, mode, crc32/blake2, encrypted }
│   │       ├── options.rs          # CompressOpts { level 0..5, threads, store_exts, solid_hint }, ExtractOpts { overwrite, keep_broken, flat }
│   │       ├── progress.rs         # ProgressSink trait (CLI = indicatif, GUI = channel, tests = noop)
│   │       ├── fs/
│   │       │   ├── walk.rs         # walkdir + globset include/exclude, base‑path stripping (‑ep1 semantics)
│   │       │   ├── metadata.rs     # mtime/mode/symlink capture & restore (filetime)
│   │       │   └── safe_path.rs    # zip‑slip, abs‑path, `..`, NTFS reserved names, long‑path prefix
│   │       ├── archiver/
│   │       │   ├── mod.rs          # trait ArchiveWriter { add_file, add_dir, add_symlink, finish }
│   │       │   ├── zip.rs          # ZipWriter impl, per‑entry method pick (store list), AES option
│   │       │   ├── tar.rs          # tar::Builder over Box<dyn Write>; gz/zst/none wrappers
│   │       │   └── parallel.rs     # rayon pipeline: compress N entries to temp, raw‑copy into ZipWriter in order
│   │       ├── extractor/
│   │       │   ├── mod.rs          # trait ArchiveReader { entries(), extract(entry, sink), test() }
│   │       │   ├── zip.rs, tar.rs, rar.rs (feature), sevenz.rs (feature)
│   │       │   └── policy.rs       # overwrite modes (‑o+/‑o‑/‑or rename), MOTW propagation
│   │       ├── crypto.rs           # password prompt abstraction, zeroizing buffers, ZIP AES config
│   │       ├── hash.rs             # crc32 / blake2sp / sha256 helpers, streaming
│   │       └── sfx.rs              # SfxTrailer { magic, archive_offset, archive_len, version }, build(stub, archive) & locate()
│   ├── shellzip-cli/               # bin `szip`
│   │   └── src/
│   │       ├── main.rs
│   │       ├── cli.rs              # clap: szip <cmd> [switches] <archive> [files…]; cmd ∈ a,x,e,l,lt,t,u,f,d,s(sfx)
│   │       ├── commands/{add,extract,list,test,update,sfx}.rs
│   │       └── ui.rs               # indicatif sinks, -idq/-inul quiet levels, -y assume‑yes
│   ├── shellzip-sfx/               # bin stub: minimal, no clap; reads own exe, extracts ZIP tail, optional silent mode
│   │   ├── build.rs                # winres: icon + manifest (asInvoker, dpiAware, common‑controls v6) [observed in Default.SFX]
│   │   └── src/main.rs
│   └── shellzip-gui/               # optional eframe app
│       └── src/{main.rs, app.rs, panels/{toolbar,file_list,status}.rs, worker.rs}
└── xtask/                          # cargo xtask: build‑fixtures (uses Rar.exe/7z if present), release packaging
```

Key API shape (core):

```rust
pub trait ArchiveWriter {
    fn add_file(&mut self, rel: &Path, src: &mut dyn Read, meta: &EntryMeta) -> Result<()>;
    fn add_dir(&mut self, rel: &Path, meta: &EntryMeta) -> Result<()>;
    fn add_symlink(&mut self, rel: &Path, target: &Path, meta: &EntryMeta) -> Result<()>;
    fn finish(self: Box<Self>) -> Result<u64>; // bytes written
}

pub trait ArchiveReader {
    fn format(&self) -> Format;
    fn entries(&mut self) -> Result<Vec<Entry>>;
    fn extract_entry(&mut self, idx: usize, sink: &mut dyn Write, pw: Option<&Password>) -> Result<Verify>;
    fn test(&mut self, progress: &dyn ProgressSink, pw: Option<&Password>) -> Result<TestReport>;
}

pub fn open(path: &Path) -> Result<Box<dyn ArchiveReader>>;       // sniffs, handles SFX overlay
pub fn create(path: &Path, fmt: Format, opts: &CompressOpts) -> Result<Box<dyn ArchiveWriter>>;
```

---

## 4. Phased implementation (Cursor task list)

Each milestone ends with `cargo clippy --all-targets -D warnings`, `cargo test --workspace`, and a tagged commit. Fixture archives live in `fixtures/` and are verified by SHA‑256 in tests.

### Milestone 0 — Skeleton (½ day)

1. `cargo new --lib crates/shellzip-core`, `cargo new crates/shellzip-cli`; workspace `Cargo.toml`, `rust-toolchain.toml`, `.cursor/rules/rust.mdc` (no `unwrap` in core, `#![deny(unsafe_code)]` in core, error enum with exit codes).
2. `error.rs` with rar‑compatible exit codes (0,1,2,3,5,6,7,8,9,10,11,255) **[observed: exit 11 for bad password]**.
3. `detect.rs` + unit tests for magic bytes: `PK\x03\x04`, `PK\x05\x06` (empty zip), `\x1f\x8b`, `(\xb5/\xfd`, `\xfd7zXZ`, `BZh`, `Rar!\x1a\x07\x00`, `Rar!\x1a\x07\x01\x00`, `7z\xbc\xaf\x27\x1c`, ustar at offset 257, and "PE + search tail" for SFX.
4. CI: GitHub Actions matrix (windows‑latest, ubuntu‑latest), `cargo fmt --check`, clippy, test.

### Milestone 1 — Core ZIP/TAR pack & unpack CLI (3–4 days)

Goal: `szip a`, `szip x`, `szip e`, `szip l`, `szip t` for ZIP, TAR, TAR.GZ, TAR.ZST, GZ, ZST.

1. `fs/walk.rs`: recursive walk (`-r` default on for dirs), include/exclude globs (`-x`, `-n`), `-ep1`‑style base stripping, deterministic ordering (sort by extension then name — cheap copy of `RarFiles.lst` idea, improves zstd/solid ratio).
2. `archiver/zip.rs`: `ZipWriter<BufWriter<File>>`; level map `-m0`→Stored, `-m1..5`→Deflate 1/3/6/7/9 (or zstd levels 1/3/7/12/19 with `--method zstd`); store‑list `-ms` default = `7z ace arj bz2 cab gz jpeg jpg lha lz lzh mp3 rar taz tbz2 tgz txz zst xz z zip zipx tzst` **[observed in Rar.txt]**; mtime, unix mode, symlinks (`-ol`).
3. `archiver/tar.rs`: `tar::Builder` over `Box<dyn Write>` = `File | GzEncoder | zstd::Encoder`; `finish()` flushes encoder.
4. `extractor/{zip,tar}.rs`: entries list, extract with `safe_path.rs` (reject absolute, `..`, drive letters, NTFS reserved names, trailing dots/spaces on Windows), overwrite policy `-o+ -o- -or`, `-y`, restore mtime, create parents, atomic write (temp + rename), `-kb` keep broken.
5. `test` command = decode every entry to a `HashWriter`, compare CRC32 (ZIP) / byte count (TAR).
6. `progress.rs` + `indicatif` multi‑bar (overall bytes + current file), `-idq`, `-inul`.
7. Tests: `assert_cmd` round‑trips (add → list → test → extract → byte compare) on a generated tree with nested dirs, empty dirs, unicode names, 0‑byte file, symlink (unix); zip‑slip fixtures (`../evil.txt`, `C:\evil`) must be rejected; interop: extract a ZIP produced by `Compress-Archive`/`Expand-Archive`, and a TAR produced by the OS `tar.exe` (bsdtar ships on Win10 1803+).

Acceptance: round‑trip a 1 GB tree without excess memory (streaming, < 100 MB RSS), all formats, Windows + Linux CI green.

### Milestone 2 — Streaming, multithreading, update ops (3 days)

1. `archiver/parallel.rs`: rayon pool; each worker compresses one entry to a temp `zip` piece (or in‑memory for < 1 MB), main thread merges in original order using `ZipWriter::raw_copy_file` — this is how we get `-mt`‑style scaling without changing ZIP layout. `--threads N` (default `available_parallelism`, cap 64 **[observed cap]**).
2. zstd MT: `Encoder::multithread(n)` and `--long=<log2>` chosen from total input size (shrink window for small inputs **[observed behaviour]**).
3. Stdin/stdout streaming: `szip a -si[name] out.tar.zst`, `szip p archive entry > file`; document that ZIP needs a seekable output (same restriction class as WinRAR's `-si` notes).
4. `lt` technical listing (name, size, packed, ratio, mtime, attrs, CRC32, optional BLAKE2sp via `-htb`, method, host OS) — snapshot‑tested with `insta`.
5. ZIP `u` (update newer), `f` (freshen existing), `d` (delete masks): implement as *rewrite* (copy untouched entries with `raw_copy_file`, re‑add changed ones), always via temp file + rename.
6. XZ and BZ2 extraction (`xz2`, `bzip2`) behind default features.
7. `tracing` log file `-ilog=<file>`; `-ierr` to stderr.
8. Benchmarks (`criterion` or simple `xtask bench`) vs. `7z.exe`/`Rar.exe` if present on the box — numbers in `docs/BENCH.md`.

Acceptance: 8‑thread ZIP creation ≥ 4× single‑thread on a multi‑file corpus; memory bounded by `threads × chunk`.

### Milestone 3 — RAR/7z extraction & password encryption (3–4 days)

1. Feature `rar`: `extractor/rar.rs` over `unrar` crate — list (`l`/`lt` with host OS, dictionary, solid/encrypted flags), test, extract with password callback, multivolume (`.part1.rar`), header‑encrypted archives (`-hp` → prompt before listing, exit 11 on wrong password **[observed]**). Build fixtures with the installed `Rar.exe` via `xtask build-fixtures` (`-hp`, `-p`, `-s`, `-htb`, `-v10k`, RAR4 via `-ma4`) and commit them.
2. Feature `sevenz`: `extractor/sevenz.rs` over `sevenz-rust2` (LZMA/LZMA2/BZip2/Deflate, AES with password).
3. `crypto.rs`: `Password` newtype with `zeroize`; sources = `-p<pw>`, `-p` (prompt via `rpassword`), `SZIP_PASSWORD` env; ZIP writer uses `zip` `aes-crypto` (AES‑256 AE‑2; PBKDF2‑HMAC‑SHA1 1000 iters is fixed by the WinZip spec). Document clearly: ZIP cannot encrypt file names — equivalent to `-p`, not `-hp`.
4. `-mes` skip‑encrypted behaviour on extract; `-kb`.
5. Security review items: password never in process title/logs; `zeroize` on drop; refuse to extract symlinks pointing outside the destination unless `--allow-unsafe-links`.
6. Optional: Mark‑of‑the‑Web propagation on Windows (`-om`): if the source archive has a `Zone.Identifier` ADS, write the same ADS on extracted files via `windows-sys` **[observed WinRAR feature]**.

Acceptance: every RAR fixture lists/tests/extracts identically to `UnRAR.exe` (compare file hashes); ZIP AES archives round‑trip with WinRAR and 7‑Zip.

### Milestone 4 — SFX stub & optional GUI (4–5 days)

**SFX (2 days)**

1. `crates/shellzip-sfx`: tiny binary (no clap, `panic=abort`, `opt-level="z"`, LTO) that: `current_exe()` → read last 24 bytes → verify `SfxTrailer { magic: b"SZSFX01", archive_offset: u64, archive_len: u64, flags }` → open `File` and wrap in a `Take`/offset reader → `zip::ZipArchive` → extract to `--dest`/cwd; switches `-s` silent, `-d<dir>`, `-y`; on Windows show a minimal dialog (Win32 `MessageBoxW`/`TaskDialog` via `windows-sys`) when run without a console; mirrors `STARTDLG / GETPASSWORD1 / REPLACEFILEDLG` roles **[observed in Default.SFX resources]**.
2. `core/sfx.rs::build()`: copy stub → append ZIP → append trailer; `detect.rs` recognises `MZ` + trailer and `open()` transparently serves the inner ZIP (`zip` crate tolerates prepended data).
3. `szip s archive.zip` ↔ `rar s` convert to/from SFX; `szip a -sfx out.exe …`.
4. Ship prebuilt stubs `szip-sfx-x86_64-pc-windows-msvc.exe`, `-gnu-linux` as `include_bytes!` or alongside the CLI (like `Default.SFX`/`WinCon.SFX` **[observed]**). Stubs stay unsigned until after appending (Store packages are re-signed by Partner Center).
5. Tests: build SFX, run it in a temp dir, compare tree; `szip l out.exe` works.

**GUI (2–3 days, optional)**

1. `eframe` app: toolbar (Add, Extract To, Test, Delete, Info), `egui_extras::TableBuilder` file list (name, size, packed, modified, CRC), breadcrumb navigation inside archive, drag‑and‑drop files onto window (`ctx.input(|i| i.raw.dropped_files)`).
2. `worker.rs`: operations run on a background thread; `ProgressSink` implemented over `std::sync::mpsc`; cancel via `AtomicBool`.
3. Open‑with registration script (Windows: ProgID `ShellZip.Archive`, `shell\open\command`) **[observed WinRAR pattern]**; Linux `.desktop` file.
4. Settings persisted with `directories` + `serde` (TOML) instead of registry.

### Phase 5 — post‑MVP backlog (not scheduled)

Explorer `IExplorerCommand` context menu via `windows` crate + sparse MSIX (same pattern as `RarExtPackage.msix`), multivolume ZIP, encrypted `.tar.zst` envelope (AES‑GCM + Argon2id), recovery data via PAR2 (`par2` crate), solid dedup `-oi`‑style hard‑link detection, language packs. 7z create/extract is in tree via `zesven`, not this backlog.

---

## 5. Testing & verification strategy

| Layer | Tooling | What |
|---|---|---|
| Unit | `cargo test` in core | detect, safe_path, hash, trailer encode/decode, option parsing |
| Property | `proptest` | random trees → add → extract → identical; random path strings never escape dest |
| Integration | `assert_cmd` + `tempfile` | CLI grammar, exit codes, quiet levels, overwrite prompts with `-y` |
| Interop | fixtures from `Rar.exe`, `7z`, `tar.exe`, `Compress-Archive` | we read theirs, they read ours (`UnRAR.exe t`, `7z t`, `Expand-Archive`) |
| Snapshot | `insta` | `l` / `lt` outputs |
| Security | fixtures | zip‑slip, symlink escape, zip bombs (ratio cap + `--max-size`), encrypted‑header RAR without password → exit 11 |
| Perf | `xtask bench` | MB/s and RSS per format/thread count |

---

## 6. Licensing notes

- **RAR creation is excluded by design.** The RAR compressor is proprietary (never published); the UnRAR source licence permits building extractors only and explicitly forbids using it to develop a RAR‑compatible archiver. Shipping `unrar` crate bindings for *extraction* is permitted.
- `zip`, `flate2`, `tar`, `zstd` (BSD/GPL dual → use BSD), `sevenz-rust2` (Apache‑2.0) are compatible with an MIT project; `xz2` links liblzma (public domain), `bzip2` (BSD‑like). Record all with `cargo deny` / `cargo about` in CI.
- SFX stubs we ship are our own code; nothing from WinRAR is copied.
