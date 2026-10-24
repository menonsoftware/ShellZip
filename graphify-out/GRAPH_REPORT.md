# Graph Report - ShellZip  (2026-10-24)

## Corpus Check
- 70 files · ~46,645 words
- Verdict: corpus is large enough that graph structure adds value.
- Unclassified: 24 file(s) not represented in the graph (top: .mdc 11, .rar 8, (none) 4)

## Summary
- 1117 nodes · 2609 edges · 54 communities (49 shown, 5 thin omitted)
- Extraction: 97% EXTRACTED · 3% INFERRED · 0% AMBIGUOUS · INFERRED: 80 edges (avg confidence: 0.85)
- Token cost: 0 input · 0 output

## Graph Freshness
- Built from commit: `a04de21f`
- Run `git rev-parse HEAD` and compare to check if the graph is stale.
- Run `graphify update .` after code changes (no API cost).

## Community Hubs (Navigation)
- worker.rs
- Icon
- ops.rs
- extractor/tar.rs
- super
- rar.rs
- stream.rs
- ui.rs
- extractor/mod.rs
- EntryMeta
- archiver/zip.rs
- ShellZipApp
- walk.rs
- Format
- round_trip.rs
- list.rs
- CompressOpts
- lib.rs
- Error
- extract.rs
- Scaffold templates
- chrome.rs
- src/cli.rs
- theme.rs
- Command
- xtask/src/main.rs
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
- milestone2.rs
- stream_one
- ProgressSink
- CompressOpts
- sevenz.rs
- BENCH.md
- MethodArg

## God Nodes (most connected - your core abstractions)
1. `ProgressSink` - 54 edges
2. `CompressOpts` - 47 edges
3. `Password` - 46 edges
4. `ShellZipApp` - 45 edges
5. `Entry` - 41 edges
6. `Format` - 32 edges
7. `Write` - 31 edges
8. `ExtractOpts` - 29 edges
9. `EntryMeta` - 27 edges
10. `Icon` - 26 edges

## Surprising Connections (you probably didn't know these)
- `flags_text()` --references--> `Entry`  [EXTRACTED]
  crates/shellzip-cli/src/commands/list.rs → crates/shellzip-core/src/entry.rs
- `xtask` --depends_on--> `shellzip-core`  [EXTRACTED]
  xtask/Cargo.toml → crates/shellzip-core/Cargo.toml
- `create_from_path()` --calls--> `format_from_extension()`  [INFERRED]
  crates/shellzip-core/src/archiver/mod.rs → crates/shellzip-core/src/detect.rs
- `compress_file()` --calls--> `copy_in()`  [INFERRED]
  crates/shellzip-core/src/archiver/parallel.rs → crates/shellzip-core/src/archiver/mod.rs
- `compress_file()` --calls--> `method_for()`  [INFERRED]
  crates/shellzip-core/src/archiver/parallel.rs → crates/shellzip-core/src/archiver/zip.rs

## Import Cycles
- 1-file cycle: `crates/shellzip-gui/src/explorer/model.rs -> crates/shellzip-gui/src/explorer/model.rs`
- 1-file cycle: `crates/shellzip-gui/src/explorer/listing.rs -> crates/shellzip-gui/src/explorer/listing.rs`
- 1-file cycle: `crates/shellzip-gui/src/icons.rs -> crates/shellzip-gui/src/icons.rs`
- 2-file cycle: `crates/shellzip-core/src/archiver/mod.rs -> crates/shellzip-core/src/archiver/parallel.rs -> crates/shellzip-core/src/archiver/mod.rs`
- 2-file cycle: `crates/shellzip-cli/src/cli.rs -> crates/shellzip-cli/src/commands/mod.rs -> crates/shellzip-cli/src/cli.rs`
- 2-file cycle: `crates/shellzip-cli/src/commands/delete.rs -> crates/shellzip-cli/src/commands/mod.rs -> crates/shellzip-cli/src/commands/delete.rs`
- 2-file cycle: `crates/shellzip-cli/src/commands/mod.rs -> crates/shellzip-cli/src/commands/print.rs -> crates/shellzip-cli/src/commands/mod.rs`
- 3-file cycle: `crates/shellzip-core/src/archiver/mod.rs -> crates/shellzip-core/src/archiver/parallel.rs -> crates/shellzip-core/src/archiver/zip.rs -> crates/shellzip-core/src/archiver/mod.rs`
- 3-file cycle: `crates/shellzip-cli/src/cli.rs -> crates/shellzip-cli/src/commands/mod.rs -> crates/shellzip-cli/src/commands/list.rs -> crates/shellzip-cli/src/cli.rs`
- 3-file cycle: `crates/shellzip-cli/src/cli.rs -> crates/shellzip-cli/src/commands/mod.rs -> crates/shellzip-cli/src/commands/delete.rs -> crates/shellzip-cli/src/cli.rs`
- 3-file cycle: `crates/shellzip-cli/src/cli.rs -> crates/shellzip-cli/src/commands/mod.rs -> crates/shellzip-cli/src/commands/print.rs -> crates/shellzip-cli/src/cli.rs`

## Communities (54 total, 5 thin omitted)

### Community 0 - "worker.rs"
Cohesion: 0.14
Nodes (8): lock(), Outcome, Directory, Failed, Finished, Listed, SharedProgress, spawn()

### Community 1 - "Icon"
Cohesion: 0.08
Nodes (31): ALL, Caret, caret(), Down, Up, Icon, Archive, ArrowUp (+23 more)

### Community 2 - "ops.rs"
Cohesion: 0.19
Nodes (16): member_key(), add(), build_globs(), Counting, delete(), extract(), filter_is_restrictive(), find_member() (+8 more)

### Community 3 - "extractor/tar.rs"
Cohesion: 0.06
Nodes (61): Compression, Bzip2, Gzip, None, Xz, Zstd, Destination, destination() (+53 more)

### Community 5 - "rar.rs"
Cohesion: 0.06
Nodes (30): List, Password, Write, archive_at(), copy_rar_index(), copy_staged(), dos_time(), entry_from() (+22 more)

### Community 6 - "stream.rs"
Cohesion: 0.12
Nodes (9): commit_output(), Kind, Gzip, Zstd, SingleArchiver, Stream, Gzip, Idle (+1 more)

### Community 7 - "ui.rs"
Cohesion: 0.10
Nodes (8): Ask, Bars, byte_style(), sink(), ConflictChoice, Overwrite, Rename, Skip

### Community 8 - "extractor/mod.rs"
Cohesion: 0.20
Nodes (7): ArchiveReader, only_first(), resolve(), unsupported(), Verify, open(), open()

### Community 9 - "EntryMeta"
Cohesion: 0.29
Nodes (5): dir_name(), header_for(), TarArchiver, unix_secs(), EntryMeta

### Community 10 - "archiver/zip.rs"
Cohesion: 0.16
Nodes (7): copy_in(), method_for(), options_for(), rel_name(), target_text(), ZipArchiver, dos_parts()

### Community 11 - "ShellZipApp"
Cohesion: 0.11
Nodes (6): Place, Archive, Folder, ShellZipApp, spawn_directory(), starting_folder()

### Community 12 - "walk.rs"
Cohesion: 0.07
Nodes (25): EntryKind, Dir, File, Other, Symlink, format_attrs(), to_archive_path(), capture() (+17 more)

### Community 13 - "Format"
Cohesion: 0.12
Nodes (22): detect_bytes(), detect_file(), detect_header(), find_embedded(), find_subslice(), Format, Bzip2, format_from_extension() (+14 more)

### Community 14 - "round_trip.rs"
Cohesion: 0.26
Nodes (16): adding_to_an_existing_zip_keeps_the_other_files(), byte_cap_stops_extraction(), existing_tar_is_left_in_place_without_force(), extract_all(), gzip_rejects_a_second_file(), opts_fast(), overwrite_skip_and_rename(), pack_tree() (+8 more)

### Community 15 - "list.rs"
Cohesion: 0.19
Nodes (9): ArchiveArgs, ListArgs, display_name(), flags_text(), list(), list_technical(), test(), format_blake() (+1 more)

### Community 16 - "CompressOpts"
Cohesion: 0.20
Nodes (6): ArchiveWriter, create(), create_from_path(), prepare_output(), prepare_temp(), temp_beside()

### Community 17 - "lib.rs"
Cohesion: 0.21
Nodes (7): delete(), extract(), DEFAULT_STORE_EXTS, DeleteReport, MAX_THREADS, shrinks_zstd_window_for_small_inputs(), StoreList

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
Nodes (8): Cli, dispatch(), init_tracing(), normalize_args(), password_from_env(), prompt_password(), run(), take_password()

### Community 23 - "theme.rs"
Cohesion: 0.17
Nodes (15): directories_use_the_yellow_folder(), row_glyph(), selected_glyphs_use_the_selection_ink(), apply(), DANGER, FOLDER, HOVER, INK (+7 more)

### Community 24 - "Command"
Cohesion: 0.17
Nodes (11): Command, Add, Complete, Delete, Extract, ExtractFlat, Freshen, ListTechnical (+3 more)

### Community 25 - "xtask/src/main.rs"
Cohesion: 0.19
Nodes (19): bench(), build_fixtures(), collect_files(), crc32(), find_rar(), find_tool(), FIXTURE_PASSWORD, main() (+11 more)

### Community 27 - "shellzip-core"
Cohesion: 0.50
Nodes (4): shellzip-cli, shellzip-core, shellzip-gui, xtask

### Community 29 - "model.rs"
Cohesion: 0.05
Nodes (56): format_bytes(), format_time(), lists_a_folder_and_a_text_file(), read_directory(), row_for(), TempDir, archive_prefix(), ArchiveIndex (+48 more)

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
Cohesion: 0.22
Nodes (7): Entry, Opened, list(), list_with(), list(), list_with(), ListOpts

### Community 39 - "Rust performance"
Cohesion: 0.29
Nodes (6): 1. Baseline, 2. Find the hot spot, 3. Apply the cheapest fix the profile justifies, 4. Binary size, 5. Verify and report, Rust performance

### Community 40 - "Overwrite"
Cohesion: 0.18
Nodes (12): Overwrite, Always, Ask, Never, Rename, ZipMethod, Deflate, Zstd (+4 more)

### Community 41 - "write"
Cohesion: 0.23
Nodes (5): DeleteArgs, PrintArgs, delete(), Shared, print_entry()

### Community 42 - "Rust quality gate"
Cohesion: 0.29
Nodes (6): Fix clippy, don't silence it, Fix tests, Optional supply-chain checks, Report, Rust quality gate, Workflow

### Community 43 - "UpdateMode"
Cohesion: 0.27
Nodes (7): AddArgs, compress_opts(), add(), UpdateMode, Add, Freshen, Update

### Community 44 - "tests/cli.rs"
Cohesion: 0.05
Nodes (33): add_list_test_extract_round_trip(), add_to_an_existing_zip_keeps_previous_files(), help_explains_that_zip_encryption_leaves_names_visible(), log_file_receives_debug_events(), rar_and_7z_are_not_created(), reads_a_tar_from_the_system_tool(), reads_a_zip_from_compress_archive(), stdin_print_update_and_delete() (+25 more)

### Community 45 - "MenuCommand"
Cohesion: 0.22
Nodes (9): MenuCommand, About, Add, Exit, Extract, Open, SelectAll, SelectNone (+1 more)

### Community 46 - "milestone2.rs"
Cohesion: 0.39
Nodes (7): freshen_missing_archive_is_not_found(), opts(), parallel_zip_matches_single_thread_bytes(), stdin_streams_zst_and_spools_tar(), technical_list_fills_blake2sp(), touch(), update_freshen_and_delete_rewrite_zip()

### Community 47 - "stream_one"
Cohesion: 0.25
Nodes (7): add_stdin(), add_stdin(), Counting<'_, R>, spool_stdin(), Spooled, stream_one(), AddReport

### Community 48 - "ProgressSink"
Cohesion: 0.16
Nodes (6): test(), test(), TestReport, ProgressReader, ProgressReader<'a, R>, ProgressSink

### Community 49 - "CompressOpts"
Cohesion: 0.16
Nodes (3): tune(), add(), CompressOpts

### Community 50 - "sevenz.rs"
Cohesion: 0.16
Nodes (17): Budget, copy_sevenz_index(), drain(), encrypted_names(), entry_from(), extract_sevenz(), is_checksum(), list_sevenz() (+9 more)

### Community 52 - "MethodArg"
Cohesion: 0.50
Nodes (4): MethodArg, Deflate, Zstd, ZipMethod

## Knowledge Gaps
- **108 isolated node(s):** `shellzip-cli`, `Add`, `Update`, `Freshen`, `Delete` (+103 more)
  These have ≤1 connection - possible missing edges or undocumented components. (Counts symbols only; 371 node(s) total have ≤1 connection when file, concept and rationale nodes are included.)
- **5 thin communities (<3 nodes) omitted from report** — run `graphify query` to explore isolated nodes.

## Suggested Questions
_Questions this graph is uniquely positioned to answer:_

- **Why does `Error` connect `Error` to `worker.rs`, `ops.rs`, `extractor/tar.rs`, `rar.rs`, `stream.rs`, `ui.rs`, `extractor/mod.rs`, `EntryMeta`, `archiver/zip.rs`, `walk.rs`, `Format`, `round_trip.rs`, `list.rs`, `CompressOpts`, `lib.rs`, `extract.rs`, `src/cli.rs`, `parallel.rs`, `write`, `UpdateMode`, `tests/cli.rs`, `CompressOpts`, `sevenz.rs`?**
  _High betweenness centrality (0.088) - this node is a cross-community bridge._
- **What connects `shellzip-cli`, `Add`, `Update` to the rest of the system?**
  _108 weakly-connected nodes found - possible documentation gaps or missing edges._
- **Should `worker.rs` be split into smaller, more focused modules?**
  _Cohesion score 0.14285714285714285 - nodes in this community are weakly interconnected._
- **Why does `CompressOpts` connect `CompressOpts` to `parallel.rs`, `ops.rs`, `CompWriter`, `stream.rs`, `rar.rs`, `Overwrite`, `EntryMeta`, `archiver/zip.rs`, `UpdateMode`, `walk.rs`, `milestone2.rs`, `stream_one`, `CompressOpts`, `lib.rs`, `round_trip.rs`, `src/cli.rs`?**
  _High betweenness centrality (0.083) - this node is a cross-community bridge._
- **Should `Icon` be split into smaller, more focused modules?**
  _Cohesion score 0.07973421926910298 - nodes in this community are weakly interconnected._
- **Why does `Overwrite` connect `Overwrite` to `extractor/tar.rs`, `tests/cli.rs`, `milestone2.rs`, `round_trip.rs`, `lib.rs`, `extract.rs`, `chrome.rs`, `src/cli.rs`, `app.rs`?**
  _High betweenness centrality (0.054) - this node is a cross-community bridge._
- **Should `extractor/tar.rs` be split into smaller, more focused modules?**
  _Cohesion score 0.060814687320711415 - nodes in this community are weakly interconnected._