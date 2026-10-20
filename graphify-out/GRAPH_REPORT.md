# Graph Report - ShellZip  (2026-10-21)

## Corpus Check
- 47 files · ~28,696 words
- Verdict: corpus is large enough that graph structure adds value.
- Unclassified: 5 file(s) not represented in the graph (top: (none) 3, .mdc 1, .toml 1)

## Summary
- 775 nodes · 1772 edges · 50 communities (46 shown, 4 thin omitted)
- Extraction: 97% EXTRACTED · 3% INFERRED · 0% AMBIGUOUS · INFERRED: 55 edges (avg confidence: 0.85)
- Token cost: 0 input · 0 output

## Graph Freshness
- Built from commit: `0c9f68b5`
- Run `git rev-parse HEAD` and compare to check if the graph is stale.
- Run `graphify update .` after code changes (no API cost).

## Community Hubs (Navigation)
- worker.rs
- Icon
- walk.rs
- extractor/tar.rs
- Write
- policy.rs
- stream.rs
- ui.rs
- extractor/zip.rs
- EntryMeta
- archiver/zip.rs
- ShellZipApp
- ops.rs
- lib.rs
- round_trip.rs
- add.rs
- prepare_output
- ProgressSink
- Error
- extract.rs
- CompWriter
- chrome.rs
- src/cli.rs
- view.rs
- Command
- MethodArg
- shellzip-gui/src/main.rs
- shellzip-core
- model.rs
- explorer/mod.rs
- Row
- sort_rows
- 4. Phased implementation (Cursor task list)
- Budget
- Tier 1 — What can be proven (direct evidence)
- CompressOpts
- ArchiveIndex
- MenuCommand
- app.rs
- listing.rs
- RowKind
- Target
- add.rs
- tests/cli.rs
- Overwrite
- ToolCommand
- SortOrder
- Partial
- ZipMethod

## God Nodes (most connected - your core abstractions)
1. `ShellZipApp` - 45 edges
2. `ProgressSink` - 38 edges
3. `Format` - 30 edges
4. `CompressOpts` - 30 edges
5. `Row` - 28 edges
6. `Entry` - 26 edges
7. `Icon` - 26 edges
8. `EntryMeta` - 25 edges
9. `Compression` - 21 edges
10. `ExtractOpts` - 21 edges

## Surprising Connections (you probably didn't know these)
- `byte_phrase()` --calls--> `format_bytes()`  [INFERRED]
  crates/shellzip-gui/src/explorer/model.rs → crates/shellzip-core/src/entry.rs
- `file_type_label()` --calls--> `format_from_extension()`  [INFERRED]
  crates/shellzip-gui/src/explorer/model.rs → crates/shellzip-core/src/detect.rs
- `is_supported_archive()` --calls--> `format_from_extension()`  [INFERRED]
  crates/shellzip-gui/src/explorer/model.rs → crates/shellzip-core/src/detect.rs
- `payload_is_tar()` --calls--> `prefix_is_tar()`  [INFERRED]
  crates/shellzip-core/src/extractor/tar.rs → crates/shellzip-core/src/detect.rs
- `extract_regular()` --calls--> `destination()`  [INFERRED]
  crates/shellzip-core/src/extractor/tar.rs → crates/shellzip-core/src/extractor/policy.rs

## Import Cycles
- 1-file cycle: `crates/shellzip-gui/src/explorer/model.rs -> crates/shellzip-gui/src/explorer/model.rs`
- 1-file cycle: `crates/shellzip-gui/src/explorer/listing.rs -> crates/shellzip-gui/src/explorer/listing.rs`
- 3-file cycle: `crates/shellzip-cli/src/cli.rs -> crates/shellzip-cli/src/commands/mod.rs -> crates/shellzip-cli/src/commands/list.rs -> crates/shellzip-cli/src/cli.rs`

## Communities (50 total, 4 thin omitted)

### Community 0 - "worker.rs"
Cohesion: 0.15
Nodes (8): lock(), Outcome, Directory, Failed, Finished, Listed, SharedProgress, spawn()

### Community 1 - "Icon"
Cohesion: 0.12
Nodes (21): ALL, Icon, Archive, ArrowUp, CircleCheck, File, FileArchive, Folder (+13 more)

### Community 2 - "walk.rs"
Cohesion: 0.09
Nodes (20): EntryKind, Dir, File, Other, Symlink, to_archive_path(), capture(), mode_bits() (+12 more)

### Community 3 - "extractor/tar.rs"
Cohesion: 0.15
Nodes (26): Compression, Gzip, None, Zstd, Entry, copy_single(), copy_tar_index(), dir_path() (+18 more)

### Community 4 - "Write"
Cohesion: 0.15
Nodes (5): Write, Fanout, Tee, copy_chunked(), CrcWriter

### Community 5 - "policy.rs"
Cohesion: 0.17
Nodes (19): Destination, destination(), Skip, exists(), partial_path(), renamed(), symlink_destination(), write_atomic() (+11 more)

### Community 6 - "stream.rs"
Cohesion: 0.12
Nodes (9): commit_output(), Kind, Gzip, Zstd, SingleArchiver, Stream, Gzip, Idle (+1 more)

### Community 7 - "ui.rs"
Cohesion: 0.10
Nodes (8): Ask, Bars, byte_style(), sink(), ConflictChoice, Overwrite, Rename, Skip

### Community 8 - "extractor/zip.rs"
Cohesion: 0.24
Nodes (9): annotate(), entry_from(), extract_zip(), extract_zip_index(), list_zip(), method_name(), open_zip(), test_zip() (+1 more)

### Community 9 - "EntryMeta"
Cohesion: 0.18
Nodes (6): ArchiveWriter, dir_name(), header_for(), TarArchiver, unix_secs(), EntryMeta

### Community 10 - "archiver/zip.rs"
Cohesion: 0.22
Nodes (7): copy_in(), method_for(), options_for(), rel_name(), target_text(), ZipArchiver, dos_parts()

### Community 11 - "ShellZipApp"
Cohesion: 0.11
Nodes (9): Place, Archive, Folder, ShellZipApp, spawn_directory(), starting_folder(), folder_parent(), is_supported_archive() (+1 more)

### Community 12 - "ops.rs"
Cohesion: 0.26
Nodes (5): build_globs(), extract(), filter_is_restrictive(), name_filter(), ExtractOpts

### Community 13 - "lib.rs"
Cohesion: 0.06
Nodes (34): Password, reject_unsupported(), detect_bytes(), detect_file(), detect_header(), find_embedded(), find_subslice(), Format (+26 more)

### Community 14 - "round_trip.rs"
Cohesion: 0.14
Nodes (15): NopProgress, ProgressReader<'_, R>, byte_cap_stops_extraction(), extract_all(), gzip_rejects_a_second_file(), opts_fast(), overwrite_skip_and_rename(), pack_tree() (+7 more)

### Community 15 - "add.rs"
Cohesion: 0.47
Nodes (4): ArchiveArgs, list(), Shared, test()

### Community 17 - "ProgressSink"
Cohesion: 0.16
Nodes (6): test(), test(), TestReport, ProgressReader, ProgressReader<'a, R>, ProgressSink

### Community 18 - "Error"
Cohesion: 0.19
Nodes (11): Error, Archive, BadChecksum, BadPassword, Encrypted, Interrupted, Io, NoFiles (+3 more)

### Community 19 - "extract.rs"
Cohesion: 0.22
Nodes (9): ExtractArgs, Overwrite, overwrite_mode(), OverwriteArg, Always, Ask, Rename, Skip (+1 more)

### Community 20 - "CompWriter"
Cohesion: 0.19
Nodes (4): CompWriter, Gzip, Plain, Zstd

### Community 21 - "chrome.rs"
Cohesion: 0.17
Nodes (12): about(), address_bar(), bar_frame(), dismiss_button(), icon_button(), level_name(), menu_bar(), options_menu() (+4 more)

### Community 22 - "src/cli.rs"
Cohesion: 0.21
Nodes (4): Cli, dispatch(), init_tracing(), run()

### Community 23 - "view.rs"
Cohesion: 0.16
Nodes (14): ADD, apply(), CANVAS, DANGER, FOLDER, HOVER, INK, LINE (+6 more)

### Community 24 - "Command"
Cohesion: 0.25
Nodes (7): Command, Add, Complete, Extract, ExtractFlat, List, Test

### Community 25 - "MethodArg"
Cohesion: 0.50
Nodes (4): MethodArg, Deflate, Zstd, ZipMethod

### Community 27 - "shellzip-core"
Cohesion: 0.67
Nodes (3): shellzip-cli, shellzip-core, shellzip-gui

### Community 29 - "model.rs"
Cohesion: 0.14
Nodes (14): format_time(), archive_prefix(), byte_phrase(), extension(), file_type_label(), format_modified(), format_modified_is_day_month_year(), named() (+6 more)

### Community 30 - "explorer/mod.rs"
Cohesion: 0.31
Nodes (7): folder_chain(), Jump, jumps(), JumpTarget, Folder, Prefix, path_under()

### Community 31 - "Row"
Cohesion: 0.16
Nodes (7): Row, cell_text(), Explorer, glyph(), modified_text(), show(), size_text()

### Community 32 - "sort_rows"
Cohesion: 0.18
Nodes (11): compare_column(), compare_name(), compare_option(), rank(), sort_keeps_parent_and_folders_above_files(), sort_rows(), SortColumn, Modified (+3 more)

### Community 33 - "4. Phased implementation (Cursor task list)"
Cohesion: 0.12
Nodes (15): 1. Scope & feature‑parity matrix, 2. Recommended crate ecosystem, 3. Project structure, 4. Phased implementation (Cursor task list), 5. Testing & verification strategy, 6. Licensing notes, Design principles carried over from WinRAR, Milestone 0 — Skeleton (½ day) (+7 more)

### Community 35 - "Tier 1 — What can be proven (direct evidence)"
Cohesion: 0.20
Nodes (9): 1.1 Directory inventory (32 files, all in one flat directory), 1.2 Separation of concerns, 1.3 Imported Windows APIs grouped by capability, 1.4 CLI surface (from `Rar.exe -?`, `UnRAR.exe`, `Rar.txt`), Evidence ledger (summary), Tier 1 — What can be proven (direct evidence), Tier 2 — What can be inferred (architectural deductions), Tier 3 — What remains hidden (opaque internals) (+1 more)

### Community 36 - "CompressOpts"
Cohesion: 0.13
Nodes (9): create(), create_from_path(), format_from_extension(), add(), add(), AddReport, CompressOpts, DEFAULT_STORE_EXTS (+1 more)

### Community 37 - "ArchiveIndex"
Cohesion: 0.43
Nodes (4): ArchiveIndex, directory_marker_hides_its_own_size(), entry(), index_builds_nested_folders()

### Community 38 - "MenuCommand"
Cohesion: 0.22
Nodes (9): MenuCommand, About, Add, Exit, Extract, Open, SelectAll, SelectNone (+1 more)

### Community 39 - "app.rs"
Cohesion: 0.15
Nodes (8): format_bytes(), folder_title(), AddressEvent, Jump, Submit, Up, EXTRACT, TEST

### Community 40 - "listing.rs"
Cohesion: 0.36
Nodes (4): lists_a_folder_and_a_text_file(), read_directory(), row_for(), TempDir

### Community 41 - "RowKind"
Cohesion: 0.33
Nodes (6): RowKind, Archive, Directory, File, Parent, Symlink

### Community 42 - "Target"
Cohesion: 0.40
Nodes (5): Target, ArchiveDir, ArchiveMember, Parent, Path

### Community 43 - "add.rs"
Cohesion: 0.70
Nodes (3): AddArgs, compress_opts(), add()

### Community 44 - "tests/cli.rs"
Cohesion: 0.36
Nodes (6): add_list_test_extract_round_trip(), reads_a_tar_from_the_system_tool(), reads_a_zip_from_compress_archive(), szip(), usage_and_help_exit_codes(), zip_slip_exits_nonzero()

### Community 45 - "Overwrite"
Cohesion: 0.40
Nodes (5): Overwrite, Always, Ask, Never, Rename

### Community 46 - "ToolCommand"
Cohesion: 0.40
Nodes (5): ToolCommand, Add, Extract, Open, Test

### Community 47 - "SortOrder"
Cohesion: 0.33
Nodes (5): SortOrder, Asc, Desc, header_clicked(), sort_mark()

### Community 48 - "Partial"
Cohesion: 0.60
Nodes (5): dir_partial(), file_partial(), insert(), Partial, upsert()

### Community 49 - "ZipMethod"
Cohesion: 0.67
Nodes (3): ZipMethod, Deflate, Zstd

## Knowledge Gaps
- **96 isolated node(s):** `shellzip-cli`, `Add`, `Extract`, `ExtractFlat`, `List` (+91 more)
  These have ≤1 connection - possible missing edges or undocumented components. (Counts symbols only; 252 node(s) total have ≤1 connection when file, concept and rationale nodes are included.)
- **4 thin communities (<3 nodes) omitted from report** — run `graphify query` to explore isolated nodes.

## Suggested Questions
_Questions this graph is uniquely positioned to answer:_

- **Why does `Error` connect `Error` to `worker.rs`, `walk.rs`, `extractor/tar.rs`, `CompressOpts`, `policy.rs`, `stream.rs`, `app.rs`, `ui.rs`, `EntryMeta`, `archiver/zip.rs`, `add.rs`, `extractor/zip.rs`, `lib.rs`, `ops.rs`, `add.rs`, `round_trip.rs`, `extract.rs`, `src/cli.rs`?**
  _High betweenness centrality (0.106) - this node is a cross-community bridge._
- **What connects `shellzip-cli`, `Add`, `Extract` to the rest of the system?**
  _96 weakly-connected nodes found - possible documentation gaps or missing edges._
- **Should `worker.rs` be split into smaller, more focused modules?**
  _Cohesion score 0.14761904761904762 - nodes in this community are weakly interconnected._
- **Why does `ProgressSink` connect `ProgressSink` to `worker.rs`, `extractor/tar.rs`, `CompressOpts`, `ui.rs`, `app.rs`, `extractor/zip.rs`, `ops.rs`, `lib.rs`, `round_trip.rs`, `add.rs`?**
  _High betweenness centrality (0.070) - this node is a cross-community bridge._
- **Should `Icon` be split into smaller, more focused modules?**
  _Cohesion score 0.11576354679802955 - nodes in this community are weakly interconnected._
- **Why does `ShellZipApp` connect `ShellZipApp` to `worker.rs`, `app.rs`, `chrome.rs`, `model.rs`, `explorer/mod.rs`, `Row`?**
  _High betweenness centrality (0.067) - this node is a cross-community bridge._
- **Should `walk.rs` be split into smaller, more focused modules?**
  _Cohesion score 0.08902439024390243 - nodes in this community are weakly interconnected._