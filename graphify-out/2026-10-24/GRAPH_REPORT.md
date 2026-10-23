# Graph Report - ShellZip  (2026-10-24)

## Corpus Check
- 64 files · ~39,398 words
- Verdict: corpus is large enough that graph structure adds value.
- Unclassified: 15 file(s) not represented in the graph (top: .mdc 11, (none) 3, .toml 1)

## Summary
- 1001 nodes · 2233 edges · 46 communities (44 shown, 2 thin omitted)
- Extraction: 97% EXTRACTED · 3% INFERRED · 0% AMBIGUOUS · INFERRED: 72 edges (avg confidence: 0.85)
- Token cost: 0 input · 0 output

## Graph Freshness
- Built from commit: `10d9f11d`
- Run `git rev-parse HEAD` and compare to check if the graph is stale.
- Run `graphify update .` after code changes (no API cost).

## Community Hubs (Navigation)
- worker.rs
- Icon
- ops.rs
- extractor/tar.rs
- policy.rs
- Write
- stream.rs
- ui.rs
- extractor/mod.rs
- EntryMeta
- archiver/zip.rs
- ShellZipApp
- milestone2.rs
- Format
- round_trip.rs
- add.rs
- CompressOpts
- lib.rs
- Error
- extract.rs
- Scaffold templates
- chrome.rs
- src/cli.rs
- theme.rs
- Command
- MethodArg
- shellzip-gui/src/main.rs
- shellzip-core
- model.rs
- app.rs
- Row
- Test patterns
- 4. Phased implementation (Cursor task list)
- parallel.rs
- Tier 1 — What can be proven (direct evidence)
- Rust review checklist
- CompWriter
- Entry
- Rust performance
- Overwrite
- write
- Rust quality gate
- UpdateMode
- tests/cli.rs
- MenuCommand

## God Nodes (most connected - your core abstractions)
1. `ProgressSink` - 47 edges
2. `CompressOpts` - 46 edges
3. `ShellZipApp` - 45 edges
4. `Entry` - 32 edges
5. `Format` - 31 edges
6. `EntryMeta` - 26 edges
7. `Icon` - 26 edges
8. `Compression` - 24 edges
9. `Write` - 24 edges
10. `Command` - 23 edges

## Surprising Connections (you probably didn't know these)
- `xtask` --depends_on--> `shellzip-core`  [EXTRACTED]
  xtask/Cargo.toml → crates/shellzip-core/Cargo.toml
- `create_from_path()` --calls--> `format_from_extension()`  [INFERRED]
  crates/shellzip-core/src/archiver/mod.rs → crates/shellzip-core/src/detect.rs
- `compress_file()` --calls--> `copy_in()`  [INFERRED]
  crates/shellzip-core/src/archiver/parallel.rs → crates/shellzip-core/src/archiver/mod.rs
- `compress_file()` --calls--> `method_for()`  [INFERRED]
  crates/shellzip-core/src/archiver/parallel.rs → crates/shellzip-core/src/archiver/zip.rs
- `compress_file()` --calls--> `options_for()`  [INFERRED]
  crates/shellzip-core/src/archiver/parallel.rs → crates/shellzip-core/src/archiver/zip.rs

## Import Cycles
- 1-file cycle: `crates/shellzip-gui/src/explorer/model.rs -> crates/shellzip-gui/src/explorer/model.rs`
- 1-file cycle: `crates/shellzip-gui/src/explorer/listing.rs -> crates/shellzip-gui/src/explorer/listing.rs`
- 1-file cycle: `crates/shellzip-gui/src/icons.rs -> crates/shellzip-gui/src/icons.rs`
- 2-file cycle: `crates/shellzip-core/src/archiver/mod.rs -> crates/shellzip-core/src/archiver/parallel.rs -> crates/shellzip-core/src/archiver/mod.rs`
- 2-file cycle: `crates/shellzip-cli/src/commands/delete.rs -> crates/shellzip-cli/src/commands/mod.rs -> crates/shellzip-cli/src/commands/delete.rs`
- 3-file cycle: `crates/shellzip-core/src/archiver/mod.rs -> crates/shellzip-core/src/archiver/parallel.rs -> crates/shellzip-core/src/archiver/zip.rs -> crates/shellzip-core/src/archiver/mod.rs`
- 3-file cycle: `crates/shellzip-cli/src/cli.rs -> crates/shellzip-cli/src/commands/mod.rs -> crates/shellzip-cli/src/commands/list.rs -> crates/shellzip-cli/src/cli.rs`
- 3-file cycle: `crates/shellzip-cli/src/cli.rs -> crates/shellzip-cli/src/commands/mod.rs -> crates/shellzip-cli/src/commands/delete.rs -> crates/shellzip-cli/src/cli.rs`
- 3-file cycle: `crates/shellzip-cli/src/cli.rs -> crates/shellzip-cli/src/commands/mod.rs -> crates/shellzip-cli/src/commands/print.rs -> crates/shellzip-cli/src/cli.rs`

## Communities (46 total, 2 thin omitted)

### Community 0 - "worker.rs"
Cohesion: 0.14
Nodes (8): lock(), Outcome, Directory, Failed, Finished, Listed, SharedProgress, spawn()

### Community 1 - "Icon"
Cohesion: 0.08
Nodes (31): ALL, Caret, caret(), Down, Up, Icon, Archive, ArrowUp (+23 more)

### Community 2 - "ops.rs"
Cohesion: 0.06
Nodes (45): format_from_extension(), member_key(), to_archive_path(), capture(), archive_rel(), blocked(), build_set(), canonicalish() (+37 more)

### Community 3 - "extractor/tar.rs"
Cohesion: 0.14
Nodes (25): Compression, Bzip2, Gzip, None, Xz, Zstd, copy_single(), copy_tar_index() (+17 more)

### Community 4 - "policy.rs"
Cohesion: 0.09
Nodes (30): Budget, Destination, destination(), Skip, exists(), partial_path(), renamed(), symlink_destination() (+22 more)

### Community 5 - "Write"
Cohesion: 0.11
Nodes (5): Fanout, Tee, blake2sp_matches_the_reference_vector(), Blake2spWriter, CrcWriter

### Community 6 - "stream.rs"
Cohesion: 0.09
Nodes (12): commit_output(), Kind, Gzip, Zstd, SingleArchiver, Stream, Gzip, Idle (+4 more)

### Community 7 - "ui.rs"
Cohesion: 0.11
Nodes (8): Ask, Bars, byte_style(), sink(), ConflictChoice, Overwrite, Rename, Skip

### Community 8 - "extractor/mod.rs"
Cohesion: 0.12
Nodes (11): Password, reject_unsupported(), ArchiveReader, only_first(), unsupported(), Verify, open(), test() (+3 more)

### Community 9 - "EntryMeta"
Cohesion: 0.29
Nodes (5): dir_name(), header_for(), TarArchiver, unix_secs(), EntryMeta

### Community 10 - "archiver/zip.rs"
Cohesion: 0.18
Nodes (7): copy_in(), method_for(), options_for(), rel_name(), target_text(), ZipArchiver, dos_parts()

### Community 11 - "ShellZipApp"
Cohesion: 0.11
Nodes (6): Place, Archive, Folder, ShellZipApp, spawn_directory(), starting_folder()

### Community 12 - "milestone2.rs"
Cohesion: 0.07
Nodes (21): ListArgs, display_name(), list_technical(), EntryKind, Dir, File, Other, Symlink (+13 more)

### Community 13 - "Format"
Cohesion: 0.11
Nodes (22): detect_bytes(), detect_file(), detect_header(), find_embedded(), find_subslice(), Format, Bzip2, Gzip (+14 more)

### Community 14 - "round_trip.rs"
Cohesion: 0.26
Nodes (15): adding_to_an_existing_zip_keeps_the_other_files(), byte_cap_stops_extraction(), existing_tar_is_left_in_place_without_force(), extract_all(), gzip_rejects_a_second_file(), opts_fast(), overwrite_skip_and_rename(), pack_tree() (+7 more)

### Community 15 - "add.rs"
Cohesion: 0.29
Nodes (6): ArchiveArgs, DeleteArgs, delete(), list(), Shared, test()

### Community 16 - "CompressOpts"
Cohesion: 0.20
Nodes (6): ArchiveWriter, create(), create_from_path(), prepare_output(), prepare_temp(), temp_beside()

### Community 17 - "lib.rs"
Cohesion: 0.16
Nodes (10): dir_path(), extract_regular(), extract_tar(), extract(), DEFAULT_STORE_EXTS, ExtractOpts, ExtractReport, MAX_THREADS (+2 more)

### Community 18 - "Error"
Cohesion: 0.19
Nodes (11): Error, Archive, BadChecksum, BadPassword, Encrypted, Interrupted, Io, NoFiles (+3 more)

### Community 19 - "extract.rs"
Cohesion: 0.22
Nodes (9): ExtractArgs, Overwrite, overwrite_mode(), OverwriteArg, Always, Ask, Rename, Skip (+1 more)

### Community 20 - "Scaffold templates"
Cohesion: 0.10
Nodes (19): Criterion benchmark, Example, Extra binary in an existing crate, Finish checklist, Integration test file, New module in an existing crate, New `szip` subcommand, New workspace crate (+11 more)

### Community 21 - "chrome.rs"
Cohesion: 0.16
Nodes (14): about(), address_bar(), AddressEvent, Jump, Submit, Up, bar_frame(), caret_button() (+6 more)

### Community 22 - "src/cli.rs"
Cohesion: 0.15
Nodes (5): Cli, dispatch(), init_tracing(), normalize_args(), run()

### Community 23 - "theme.rs"
Cohesion: 0.17
Nodes (15): directories_use_the_yellow_folder(), row_glyph(), selected_glyphs_use_the_selection_ink(), apply(), DANGER, FOLDER, HOVER, INK (+7 more)

### Community 24 - "Command"
Cohesion: 0.15
Nodes (12): Command, Add, Complete, Delete, Extract, ExtractFlat, Freshen, List (+4 more)

### Community 25 - "MethodArg"
Cohesion: 0.50
Nodes (4): MethodArg, Deflate, Zstd, ZipMethod

### Community 27 - "shellzip-core"
Cohesion: 0.50
Nodes (4): shellzip-cli, shellzip-core, shellzip-gui, xtask

### Community 29 - "model.rs"
Cohesion: 0.06
Nodes (54): format_time(), lists_a_folder_and_a_text_file(), read_directory(), row_for(), TempDir, archive_prefix(), ArchiveIndex, compare_column() (+46 more)

### Community 30 - "app.rs"
Cohesion: 0.14
Nodes (11): existing_zip(), folder_title(), ToolCommand, Add, Extract, Open, Test, ADD (+3 more)

### Community 31 - "Row"
Cohesion: 0.10
Nodes (14): SortColumn, Modified, Name, Size, Type, SortOrder, Asc, Desc (+6 more)

### Community 32 - "Test patterns"
Cohesion: 0.11
Nodes (17): CLI end-to-end with `assert_cmd` (`tests/cli.rs`), Counting fake for a trait, In-memory reader/writer, Integration round-trip (`tests/<scenario>.rs`), Interop with OS tools, Platform-gated test, Property test with `proptest`, Snapshot test with `insta` (+9 more)

### Community 33 - "4. Phased implementation (Cursor task list)"
Cohesion: 0.12
Nodes (15): 1. Scope & feature‑parity matrix, 2. Recommended crate ecosystem, 3. Project structure, 4. Phased implementation (Cursor task list), 5. Testing & verification strategy, 6. Licensing notes, Design principles carried over from WinRAR, Milestone 0 — Skeleton (½ day) (+7 more)

### Community 34 - "parallel.rs"
Cohesion: 0.22
Nodes (8): add_zip(), compress_file(), MEMORY_LIMIT, merge(), Piece, Memory, piece_path(), Temp

### Community 35 - "Tier 1 — What can be proven (direct evidence)"
Cohesion: 0.20
Nodes (9): 1.1 Directory inventory (32 files, all in one flat directory), 1.2 Separation of concerns, 1.3 Imported Windows APIs grouped by capability, 1.4 CLI surface (from `Rar.exe -?`, `UnRAR.exe`, `Rar.txt`), Evidence ledger (summary), Tier 1 — What can be proven (direct evidence), Tier 2 — What can be inferred (architectural deductions), Tier 3 — What remains hidden (opaque internals) (+1 more)

### Community 36 - "Rust review checklist"
Cohesion: 0.13
Nodes (13): 1. Correctness, 2. Error handling, 3. Security, 4. Ownership and performance, 5. Concurrency, 6. Tests, 7. API and docs, 8. Manifest and tooling (+5 more)

### Community 37 - "CompWriter"
Cohesion: 0.19
Nodes (4): CompWriter, Gzip, Plain, Zstd

### Community 38 - "Entry"
Cohesion: 0.29
Nodes (7): Entry, Opened, list(), list_with(), list(), list_with(), ListOpts

### Community 39 - "Rust performance"
Cohesion: 0.29
Nodes (6): 1. Baseline, 2. Find the hot spot, 3. Apply the cheapest fix the profile justifies, 4. Binary size, 5. Verify and report, Rust performance

### Community 40 - "Overwrite"
Cohesion: 0.18
Nodes (12): Overwrite, Always, Ask, Never, Rename, ZipMethod, Deflate, Zstd (+4 more)

### Community 41 - "write"
Cohesion: 0.20
Nodes (3): PrintArgs, print_entry(), tune()

### Community 42 - "Rust quality gate"
Cohesion: 0.29
Nodes (6): Fix clippy, don't silence it, Fix tests, Optional supply-chain checks, Report, Rust quality gate, Workflow

### Community 43 - "UpdateMode"
Cohesion: 0.27
Nodes (7): AddArgs, compress_opts(), add(), UpdateMode, Add, Freshen, Update

### Community 44 - "tests/cli.rs"
Cohesion: 0.07
Nodes (20): add_list_test_extract_round_trip(), add_to_an_existing_zip_keeps_previous_files(), log_file_receives_debug_events(), reads_a_tar_from_the_system_tool(), reads_a_zip_from_compress_archive(), stdin_print_update_and_delete(), szip(), technical_listing_layout() (+12 more)

### Community 45 - "MenuCommand"
Cohesion: 0.22
Nodes (9): MenuCommand, About, Add, Exit, Extract, Open, SelectAll, SelectNone (+1 more)

## Knowledge Gaps
- **109 isolated node(s):** `shellzip-cli`, `Add`, `Update`, `Freshen`, `Delete` (+104 more)
  These have ≤1 connection - possible missing edges or undocumented components. (Counts symbols only; 351 node(s) total have ≤1 connection when file, concept and rationale nodes are included.)
- **2 thin communities (<3 nodes) omitted from report** — run `graphify query` to explore isolated nodes.

## Suggested Questions
_Questions this graph is uniquely positioned to answer:_

- **Why does `Error` connect `Error` to `worker.rs`, `ops.rs`, `extractor/tar.rs`, `policy.rs`, `stream.rs`, `ui.rs`, `extractor/mod.rs`, `EntryMeta`, `archiver/zip.rs`, `milestone2.rs`, `Format`, `round_trip.rs`, `add.rs`, `CompressOpts`, `lib.rs`, `extract.rs`, `src/cli.rs`, `parallel.rs`, `write`, `UpdateMode`?**
  _High betweenness centrality (0.074) - this node is a cross-community bridge._
- **What connects `shellzip-cli`, `Add`, `Update` to the rest of the system?**
  _109 weakly-connected nodes found - possible documentation gaps or missing edges._
- **Should `worker.rs` be split into smaller, more focused modules?**
  _Cohesion score 0.14285714285714285 - nodes in this community are weakly interconnected._
- **Why does `CompressOpts` connect `ops.rs` to `parallel.rs`, `CompWriter`, `stream.rs`, `Overwrite`, `EntryMeta`, `archiver/zip.rs`, `UpdateMode`, `write`, `milestone2.rs`, `round_trip.rs`, `CompressOpts`, `lib.rs`, `src/cli.rs`?**
  _High betweenness centrality (0.071) - this node is a cross-community bridge._
- **Should `Icon` be split into smaller, more focused modules?**
  _Cohesion score 0.07973421926910298 - nodes in this community are weakly interconnected._
- **Why does `Entry` connect `Entry` to `worker.rs`, `ops.rs`, `extractor/tar.rs`, `policy.rs`, `extractor/mod.rs`, `ShellZipApp`, `milestone2.rs`, `lib.rs`, `model.rs`, `app.rs`?**
  _High betweenness centrality (0.051) - this node is a cross-community bridge._
- **Should `ops.rs` be split into smaller, more focused modules?**
  _Cohesion score 0.056842105263157895 - nodes in this community are weakly interconnected._