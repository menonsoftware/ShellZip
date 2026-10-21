# Graph Report - ShellZip  (2026-10-21)

## Corpus Check
- 48 files · ~29,665 words
- Verdict: corpus is large enough that graph structure adds value.
- Unclassified: 5 file(s) not represented in the graph (top: (none) 3, .mdc 1, .toml 1)

## Summary
- 802 nodes · 1834 edges · 41 communities (38 shown, 3 thin omitted)
- Extraction: 97% EXTRACTED · 3% INFERRED · 0% AMBIGUOUS · INFERRED: 56 edges (avg confidence: 0.85)
- Token cost: 0 input · 0 output

## Graph Freshness
- Built from commit: `ff79ee31`
- Run `git rev-parse HEAD` and compare to check if the graph is stale.
- Run `graphify update .` after code changes (no API cost).

## Community Hubs (Navigation)
- worker.rs
- Icon
- walk.rs
- extractor/tar.rs
- chrome.rs
- Write
- stream.rs
- policy.rs
- app.rs
- EntryMeta
- archiver/zip.rs
- ShellZipApp
- Ui
- lib.rs
- round_trip.rs
- add.rs
- CompressOpts
- MenuCommand
- Error
- extract.rs
- CompWriter
- theme.rs
- src/cli.rs
- StoreList
- Command
- MethodArg
- shellzip-gui/src/main.rs
- shellzip-core
- model.rs
- explorer/mod.rs
- Row
- 4. Phased implementation (Cursor task list)
- sort_rows
- Tier 1 — What can be proven (direct evidence)
- ArchiveIndex
- Row
- RowKind
- add.rs
- tests/cli.rs
- Overwrite

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
- `create_from_path()` --calls--> `format_from_extension()`  [INFERRED]
  crates/shellzip-core/src/archiver/mod.rs → crates/shellzip-core/src/detect.rs
- `file_type_label()` --calls--> `format_from_extension()`  [INFERRED]
  crates/shellzip-gui/src/explorer/model.rs → crates/shellzip-core/src/detect.rs
- `is_supported_archive()` --calls--> `format_from_extension()`  [INFERRED]
  crates/shellzip-gui/src/explorer/model.rs → crates/shellzip-core/src/detect.rs
- `payload_is_tar()` --calls--> `prefix_is_tar()`  [INFERRED]
  crates/shellzip-core/src/extractor/tar.rs → crates/shellzip-core/src/detect.rs
- `byte_phrase()` --calls--> `format_bytes()`  [INFERRED]
  crates/shellzip-gui/src/explorer/model.rs → crates/shellzip-core/src/entry.rs

## Import Cycles
- 1-file cycle: `crates/shellzip-gui/src/explorer/model.rs -> crates/shellzip-gui/src/explorer/model.rs`
- 1-file cycle: `crates/shellzip-gui/src/explorer/listing.rs -> crates/shellzip-gui/src/explorer/listing.rs`
- 1-file cycle: `crates/shellzip-gui/src/icons.rs -> crates/shellzip-gui/src/icons.rs`
- 3-file cycle: `crates/shellzip-cli/src/cli.rs -> crates/shellzip-cli/src/commands/mod.rs -> crates/shellzip-cli/src/commands/list.rs -> crates/shellzip-cli/src/cli.rs`

## Communities (41 total, 3 thin omitted)

### Community 0 - "worker.rs"
Cohesion: 0.15
Nodes (8): lock(), Outcome, Directory, Failed, Finished, Listed, SharedProgress, spawn()

### Community 1 - "Icon"
Cohesion: 0.08
Nodes (31): ALL, Caret, caret(), Down, Up, Icon, Archive, ArrowUp (+23 more)

### Community 2 - "walk.rs"
Cohesion: 0.09
Nodes (20): EntryKind, Dir, File, Other, Symlink, to_archive_path(), capture(), mode_bits() (+12 more)

### Community 3 - "extractor/tar.rs"
Cohesion: 0.06
Nodes (50): Compression, Gzip, None, Zstd, Entry, Budget, copy_single(), copy_tar_index() (+42 more)

### Community 4 - "chrome.rs"
Cohesion: 0.17
Nodes (11): ZipMethod, Deflate, Zstd, about(), bar_frame(), level_name(), menu_bar(), options_menu() (+3 more)

### Community 5 - "Write"
Cohesion: 0.13
Nodes (5): Write, Fanout, Tee, copy_chunked(), CrcWriter

### Community 6 - "stream.rs"
Cohesion: 0.11
Nodes (9): commit_output(), Kind, Gzip, Zstd, SingleArchiver, Stream, Gzip, Idle (+1 more)

### Community 7 - "policy.rs"
Cohesion: 0.07
Nodes (27): Ask, Bars, byte_style(), sink(), Destination, destination(), Skip, exists() (+19 more)

### Community 8 - "app.rs"
Cohesion: 0.13
Nodes (11): existing_zip(), folder_title(), ToolCommand, Add, Extract, Open, Test, ADD (+3 more)

### Community 9 - "EntryMeta"
Cohesion: 0.29
Nodes (5): dir_name(), header_for(), TarArchiver, unix_secs(), EntryMeta

### Community 10 - "archiver/zip.rs"
Cohesion: 0.18
Nodes (8): copy_in(), member_key(), method_for(), options_for(), rel_name(), target_text(), ZipArchiver, dos_parts()

### Community 11 - "ShellZipApp"
Cohesion: 0.11
Nodes (7): Place, Archive, Folder, ShellZipApp, spawn_directory(), starting_folder(), is_supported_archive()

### Community 12 - "Ui"
Cohesion: 0.27
Nodes (10): address_bar(), AddressEvent, Jump, Submit, Up, caret_button(), dismiss_button(), icon_button() (+2 more)

### Community 13 - "lib.rs"
Cohesion: 0.06
Nodes (36): Password, reject_unsupported(), detect_bytes(), detect_file(), detect_header(), find_embedded(), find_subslice(), Format (+28 more)

### Community 14 - "round_trip.rs"
Cohesion: 0.14
Nodes (17): NopProgress, ProgressReader<'_, R>, adding_to_an_existing_zip_keeps_the_other_files(), byte_cap_stops_extraction(), existing_tar_is_left_in_place_without_force(), extract_all(), gzip_rejects_a_second_file(), opts_fast() (+9 more)

### Community 15 - "add.rs"
Cohesion: 0.36
Nodes (4): ArchiveArgs, list(), Shared, test()

### Community 16 - "CompressOpts"
Cohesion: 0.16
Nodes (7): ArchiveWriter, create(), create_from_path(), prepare_output(), prepare_temp(), temp_beside(), CompressOpts

### Community 17 - "MenuCommand"
Cohesion: 0.22
Nodes (9): MenuCommand, About, Add, Exit, Extract, Open, SelectAll, SelectNone (+1 more)

### Community 18 - "Error"
Cohesion: 0.19
Nodes (11): Error, Archive, BadChecksum, BadPassword, Encrypted, Interrupted, Io, NoFiles (+3 more)

### Community 19 - "extract.rs"
Cohesion: 0.22
Nodes (9): ExtractArgs, Overwrite, overwrite_mode(), OverwriteArg, Always, Ask, Rename, Skip (+1 more)

### Community 20 - "CompWriter"
Cohesion: 0.21
Nodes (4): CompWriter, Gzip, Plain, Zstd

### Community 21 - "theme.rs"
Cohesion: 0.17
Nodes (15): directories_use_the_yellow_folder(), row_glyph(), selected_glyphs_use_the_selection_ink(), apply(), DANGER, FOLDER, HOVER, INK (+7 more)

### Community 22 - "src/cli.rs"
Cohesion: 0.21
Nodes (4): Cli, dispatch(), init_tracing(), run()

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
Cohesion: 0.17
Nodes (14): format_time(), archive_prefix(), dir_partial(), extension(), file_partial(), file_type_label(), format_modified(), format_modified_is_day_month_year() (+6 more)

### Community 30 - "explorer/mod.rs"
Cohesion: 0.17
Nodes (13): folder_chain(), folder_parent(), Jump, jumps(), JumpTarget, Folder, Prefix, location_text() (+5 more)

### Community 31 - "Row"
Cohesion: 0.11
Nodes (9): SortOrder, Asc, Desc, cell_text(), Explorer, header_clicked(), modified_text(), show() (+1 more)

### Community 33 - "4. Phased implementation (Cursor task list)"
Cohesion: 0.12
Nodes (15): 1. Scope & feature‑parity matrix, 2. Recommended crate ecosystem, 3. Project structure, 4. Phased implementation (Cursor task list), 5. Testing & verification strategy, 6. Licensing notes, Design principles carried over from WinRAR, Milestone 0 — Skeleton (½ day) (+7 more)

### Community 34 - "sort_rows"
Cohesion: 0.20
Nodes (11): compare_column(), compare_name(), compare_option(), rank(), sort_keeps_parent_and_folders_above_files(), sort_rows(), SortColumn, Modified (+3 more)

### Community 35 - "Tier 1 — What can be proven (direct evidence)"
Cohesion: 0.20
Nodes (9): 1.1 Directory inventory (32 files, all in one flat directory), 1.2 Separation of concerns, 1.3 Imported Windows APIs grouped by capability, 1.4 CLI surface (from `Rar.exe -?`, `UnRAR.exe`, `Rar.txt`), Evidence ledger (summary), Tier 1 — What can be proven (direct evidence), Tier 2 — What can be inferred (architectural deductions), Tier 3 — What remains hidden (opaque internals) (+1 more)

### Community 37 - "ArchiveIndex"
Cohesion: 0.43
Nodes (4): ArchiveIndex, directory_marker_hides_its_own_size(), entry(), index_builds_nested_folders()

### Community 40 - "Row"
Cohesion: 0.21
Nodes (10): lists_a_folder_and_a_text_file(), read_directory(), row_for(), TempDir, Row, Target, ArchiveDir, ArchiveMember (+2 more)

### Community 41 - "RowKind"
Cohesion: 0.33
Nodes (6): RowKind, Archive, Directory, File, Parent, Symlink

### Community 43 - "add.rs"
Cohesion: 0.43
Nodes (5): AddArgs, compress_opts(), add(), format_bytes(), byte_phrase()

### Community 44 - "tests/cli.rs"
Cohesion: 0.33
Nodes (7): add_list_test_extract_round_trip(), add_to_an_existing_zip_keeps_previous_files(), reads_a_tar_from_the_system_tool(), reads_a_zip_from_compress_archive(), szip(), usage_and_help_exit_codes(), zip_slip_exits_nonzero()

### Community 45 - "Overwrite"
Cohesion: 0.40
Nodes (5): Overwrite, Always, Ask, Never, Rename

## Knowledge Gaps
- **98 isolated node(s):** `shellzip-cli`, `Add`, `Extract`, `ExtractFlat`, `List` (+93 more)
  These have ≤1 connection - possible missing edges or undocumented components. (Counts symbols only; 262 node(s) total have ≤1 connection when file, concept and rationale nodes are included.)
- **3 thin communities (<3 nodes) omitted from report** — run `graphify query` to explore isolated nodes.

## Suggested Questions
_Questions this graph is uniquely positioned to answer:_

- **Why does `Error` connect `Error` to `worker.rs`, `walk.rs`, `extractor/tar.rs`, `Write`, `stream.rs`, `policy.rs`, `EntryMeta`, `archiver/zip.rs`, `add.rs`, `lib.rs`, `round_trip.rs`, `add.rs`, `CompressOpts`, `extract.rs`, `src/cli.rs`?**
  _High betweenness centrality (0.102) - this node is a cross-community bridge._
- **What connects `shellzip-cli`, `Add`, `Extract` to the rest of the system?**
  _98 weakly-connected nodes found - possible documentation gaps or missing edges._
- **Should `worker.rs` be split into smaller, more focused modules?**
  _Cohesion score 0.14761904761904762 - nodes in this community are weakly interconnected._
- **Why does `ProgressSink` connect `extractor/tar.rs` to `worker.rs`, `policy.rs`, `app.rs`, `lib.rs`, `round_trip.rs`, `add.rs`?**
  _High betweenness centrality (0.066) - this node is a cross-community bridge._
- **Should `Icon` be split into smaller, more focused modules?**
  _Cohesion score 0.07973421926910298 - nodes in this community are weakly interconnected._
- **Why does `ShellZipApp` connect `ShellZipApp` to `worker.rs`, `chrome.rs`, `app.rs`, `Row`, `explorer/mod.rs`, `Row`?**
  _High betweenness centrality (0.064) - this node is a cross-community bridge._
- **Should `walk.rs` be split into smaller, more focused modules?**
  _Cohesion score 0.0859465737514518 - nodes in this community are weakly interconnected._