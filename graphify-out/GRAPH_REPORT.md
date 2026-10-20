# Graph Report - ShellZip  (2026-10-21)

## Corpus Check
- cluster-only mode — file stats not available

## Summary
- 571 nodes · 1287 edges · 29 communities (27 shown, 2 thin omitted)
- Extraction: 97% EXTRACTED · 3% INFERRED · 0% AMBIGUOUS · INFERRED: 39 edges (avg confidence: 0.85)
- Token cost: 0 input · 0 output

## Community Hubs (Navigation)
- app.rs
- panels.rs
- walk.rs
- extractor/tar.rs
- ProgressSink
- policy.rs
- stream.rs
- ui.rs
- extractor/mod.rs
- EntryMeta
- archiver/zip.rs
- Write
- ops.rs
- detect.rs
- round_trip.rs
- add.rs
- CompressOpts
- lib.rs
- Error
- extract.rs
- CompWriter
- Format
- src/cli.rs
- tests/cli.rs
- Command
- MethodArg
- shellzip-gui/src/main.rs
- shellzip-core

## God Nodes (most connected - your core abstractions)
1. `ProgressSink` - 38 edges
2. `CompressOpts` - 30 edges
3. `Format` - 30 edges
4. `Entry` - 25 edges
5. `EntryMeta` - 25 edges
6. `ExtractOpts` - 21 edges
7. `Compression` - 21 edges
8. `Icon` - 20 edges
9. `Write` - 20 edges
10. `ShellZipApp` - 19 edges

## Surprising Connections (you probably didn't know these)
- `resolve()` --calls--> `detect_file()`  [INFERRED]
  crates/shellzip-core/src/extractor/mod.rs → crates/shellzip-core/src/detect.rs
- `extract_regular()` --calls--> `destination()`  [INFERRED]
  crates/shellzip-core/src/extractor/tar.rs → crates/shellzip-core/src/extractor/policy.rs
- `extract_regular()` --calls--> `write_atomic()`  [INFERRED]
  crates/shellzip-core/src/extractor/tar.rs → crates/shellzip-core/src/extractor/policy.rs
- `extract_single()` --calls--> `destination()`  [INFERRED]
  crates/shellzip-core/src/extractor/tar.rs → crates/shellzip-core/src/extractor/policy.rs
- `extract_single()` --calls--> `write_atomic()`  [INFERRED]
  crates/shellzip-core/src/extractor/tar.rs → crates/shellzip-core/src/extractor/policy.rs

## Import Cycles
- 2-file cycle: `crates/shellzip-cli/src/commands/mod.rs -> crates/shellzip-cli/src/commands/test.rs -> crates/shellzip-cli/src/commands/mod.rs`
- 2-file cycle: `crates/shellzip-cli/src/commands/add.rs -> crates/shellzip-cli/src/commands/mod.rs -> crates/shellzip-cli/src/commands/add.rs`
- 2-file cycle: `crates/shellzip-cli/src/commands/extract.rs -> crates/shellzip-cli/src/commands/mod.rs -> crates/shellzip-cli/src/commands/extract.rs`
- 3-file cycle: `crates/shellzip-cli/src/cli.rs -> crates/shellzip-cli/src/commands/mod.rs -> crates/shellzip-cli/src/commands/test.rs -> crates/shellzip-cli/src/cli.rs`
- 3-file cycle: `crates/shellzip-cli/src/cli.rs -> crates/shellzip-cli/src/commands/mod.rs -> crates/shellzip-cli/src/commands/list.rs -> crates/shellzip-cli/src/cli.rs`
- 3-file cycle: `crates/shellzip-cli/src/cli.rs -> crates/shellzip-cli/src/commands/mod.rs -> crates/shellzip-cli/src/commands/add.rs -> crates/shellzip-cli/src/cli.rs`
- 3-file cycle: `crates/shellzip-cli/src/cli.rs -> crates/shellzip-cli/src/commands/mod.rs -> crates/shellzip-cli/src/commands/extract.rs -> crates/shellzip-cli/src/cli.rs`

## Communities (29 total, 2 thin omitted)

### Community 0 - "app.rs"
Cohesion: 0.07
Nodes (17): Overwrite, Always, Ask, Never, Rename, ZipMethod, Deflate, Zstd (+9 more)

### Community 1 - "panels.rs"
Cohesion: 0.07
Nodes (32): format_time(), Icon, Archive, CircleCheck, File, FileArchive, Folder, FolderDown (+24 more)

### Community 2 - "walk.rs"
Cohesion: 0.09
Nodes (20): EntryKind, Dir, File, Other, Symlink, to_archive_path(), capture(), mode_bits() (+12 more)

### Community 3 - "extractor/tar.rs"
Cohesion: 0.12
Nodes (28): Compression, Gzip, None, Zstd, prefix_is_tar(), Entry, Budget, copy_single() (+20 more)

### Community 4 - "ProgressSink"
Cohesion: 0.09
Nodes (14): annotate(), entry_from(), extract_zip(), extract_zip_index(), list_zip(), method_name(), open_zip(), test_zip() (+6 more)

### Community 5 - "policy.rs"
Cohesion: 0.12
Nodes (20): Destination, destination(), Skip, exists(), partial_path(), renamed(), symlink_destination(), write_atomic() (+12 more)

### Community 6 - "stream.rs"
Cohesion: 0.11
Nodes (9): commit_output(), Kind, Gzip, Zstd, SingleArchiver, Stream, Gzip, Idle (+1 more)

### Community 7 - "ui.rs"
Cohesion: 0.10
Nodes (8): Ask, Bars, byte_style(), sink(), ConflictChoice, Overwrite, Rename, Skip

### Community 8 - "extractor/mod.rs"
Cohesion: 0.13
Nodes (8): Password, reject_unsupported(), only_first(), Opened, resolve(), unsupported(), Verify, list()

### Community 9 - "EntryMeta"
Cohesion: 0.20
Nodes (6): ArchiveWriter, dir_name(), header_for(), TarArchiver, unix_secs(), EntryMeta

### Community 10 - "archiver/zip.rs"
Cohesion: 0.23
Nodes (6): method_for(), options_for(), rel_name(), target_text(), ZipArchiver, dos_parts()

### Community 11 - "Write"
Cohesion: 0.13
Nodes (5): Write, Fanout, Tee, copy_chunked(), CrcWriter

### Community 12 - "ops.rs"
Cohesion: 0.20
Nodes (8): add(), add(), build_globs(), extract(), filter_is_restrictive(), name_filter(), AddReport, ExtractOpts

### Community 13 - "detect.rs"
Cohesion: 0.23
Nodes (9): detect_bytes(), detect_file(), detect_header(), find_embedded(), find_subslice(), is_tar_block(), pe_stub_with_rar5_tail(), pe_stub_with_zip_tail() (+1 more)

### Community 14 - "round_trip.rs"
Cohesion: 0.25
Nodes (13): byte_cap_stops_extraction(), extract_all(), gzip_rejects_a_second_file(), opts_fast(), overwrite_skip_and_rename(), pack_tree(), round_trip_zip_tar_and_compressed_tar(), sample_tree() (+5 more)

### Community 15 - "add.rs"
Cohesion: 0.24
Nodes (8): AddArgs, ArchiveArgs, compress_opts(), add(), list(), Shared, test(), format_bytes()

### Community 16 - "CompressOpts"
Cohesion: 0.20
Nodes (7): copy_in(), create(), create_from_path(), prepare_output(), temp_beside(), format_from_extension(), CompressOpts

### Community 17 - "lib.rs"
Cohesion: 0.17
Nodes (9): ArchiveReader, extract(), list(), open(), test(), open(), test(), DEFAULT_STORE_EXTS (+1 more)

### Community 18 - "Error"
Cohesion: 0.19
Nodes (11): Error, Archive, BadChecksum, BadPassword, Encrypted, Interrupted, Io, NoFiles (+3 more)

### Community 19 - "extract.rs"
Cohesion: 0.22
Nodes (9): ExtractArgs, Overwrite, overwrite_mode(), OverwriteArg, Always, Ask, Rename, Skip (+1 more)

### Community 20 - "CompWriter"
Cohesion: 0.19
Nodes (4): CompWriter, Gzip, Plain, Zstd

### Community 21 - "Format"
Cohesion: 0.15
Nodes (11): Format, Bzip2, Gzip, Rar4, Rar5, SevenZ, Sfx, Tar (+3 more)

### Community 22 - "src/cli.rs"
Cohesion: 0.21
Nodes (4): Cli, dispatch(), init_tracing(), run()

### Community 23 - "tests/cli.rs"
Cohesion: 0.36
Nodes (6): add_list_test_extract_round_trip(), reads_a_tar_from_the_system_tool(), reads_a_zip_from_compress_archive(), szip(), usage_and_help_exit_codes(), zip_slip_exits_nonzero()

### Community 24 - "Command"
Cohesion: 0.25
Nodes (7): Command, Add, Complete, Extract, ExtractFlat, List, Test

### Community 25 - "MethodArg"
Cohesion: 0.50
Nodes (4): MethodArg, Deflate, Zstd, ZipMethod

### Community 27 - "shellzip-core"
Cohesion: 0.67
Nodes (3): shellzip-cli, shellzip-core, shellzip-gui

## Knowledge Gaps
- **65 isolated node(s):** `Always`, `Ask`, `Never`, `Rename`, `Deflate` (+60 more)
  These have ≤1 connection - possible missing edges. (Counts symbols only; 179 node(s) total have ≤1 connection when file, concept and rationale nodes are included.)
- **2 thin communities (<3 nodes) omitted from report** — run `graphify query` to explore isolated nodes.

## Suggested Questions
_Questions this graph is uniquely positioned to answer:_

- **Why does `Error` connect `Error` to `app.rs`, `walk.rs`, `extractor/tar.rs`, `ProgressSink`, `policy.rs`, `stream.rs`, `ui.rs`, `extractor/mod.rs`, `EntryMeta`, `archiver/zip.rs`, `Write`, `ops.rs`, `detect.rs`, `round_trip.rs`, `add.rs`, `CompressOpts`, `lib.rs`, `extract.rs`, `src/cli.rs`?**
  _High betweenness centrality (0.181) - this node is a cross-community bridge._
- **What connects `Always`, `Ask`, `Never` to the rest of the system?**
  _65 weakly-connected nodes found - possible documentation gaps or missing edges._
- **Should `app.rs` be split into smaller, more focused modules?**
  _Cohesion score 0.06588235294117648 - nodes in this community are weakly interconnected._
- **Why does `ProgressSink` connect `ProgressSink` to `app.rs`, `extractor/tar.rs`, `ui.rs`, `extractor/mod.rs`, `ops.rs`, `add.rs`, `lib.rs`?**
  _High betweenness centrality (0.095) - this node is a cross-community bridge._
- **Should `panels.rs` be split into smaller, more focused modules?**
  _Cohesion score 0.07373737373737374 - nodes in this community are weakly interconnected._
- **Why does `CompressOpts` connect `CompressOpts` to `app.rs`, `walk.rs`, `policy.rs`, `stream.rs`, `EntryMeta`, `archiver/zip.rs`, `ops.rs`, `round_trip.rs`, `add.rs`, `lib.rs`, `CompWriter`, `src/cli.rs`?**
  _High betweenness centrality (0.095) - this node is a cross-community bridge._
- **Should `walk.rs` be split into smaller, more focused modules?**
  _Cohesion score 0.08902439024390243 - nodes in this community are weakly interconnected._