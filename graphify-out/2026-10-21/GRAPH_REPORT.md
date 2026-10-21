# Graph Report - ShellZip  (2026-10-21)

## Corpus Check
- 48 files · ~29,058 words
- Verdict: corpus is large enough that graph structure adds value.
- Unclassified: 5 file(s) not represented in the graph (top: (none) 3, .mdc 1, .toml 1)

## Summary
- 792 nodes · 1809 edges · 44 communities (39 shown, 5 thin omitted)
- Extraction: 97% EXTRACTED · 3% INFERRED · 0% AMBIGUOUS · INFERRED: 55 edges (avg confidence: 0.85)
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
- Write
- extractor/zip.rs
- stream.rs
- ui.rs
- Entry
- EntryMeta
- archiver/zip.rs
- ShellZipApp
- lib.rs
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
- SingleArchiver
- Command
- MethodArg
- shellzip-gui/src/main.rs
- shellzip-core
- model.rs
- app.rs
- Row
- Explorer
- 4. Phased implementation (Cursor task list)
- sort_rows
- Tier 1 — What can be proven (direct evidence)
- commit_output
- ArchiveIndex
- listing.rs
- RowKind
- Target
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
- `file_type_label()` --calls--> `format_from_extension()`  [INFERRED]
  crates/shellzip-gui/src/explorer/model.rs → crates/shellzip-core/src/detect.rs
- `is_supported_archive()` --calls--> `format_from_extension()`  [INFERRED]
  crates/shellzip-gui/src/explorer/model.rs → crates/shellzip-core/src/detect.rs
- `byte_phrase()` --calls--> `format_bytes()`  [INFERRED]
  crates/shellzip-gui/src/explorer/model.rs → crates/shellzip-core/src/entry.rs
- `destination()` --calls--> `ensure_dir()`  [INFERRED]
  crates/shellzip-core/src/extractor/policy.rs → crates/shellzip-core/src/fs/safe_path.rs
- `destination()` --calls--> `safe_join()`  [INFERRED]
  crates/shellzip-core/src/extractor/policy.rs → crates/shellzip-core/src/fs/safe_path.rs

## Import Cycles
- 1-file cycle: `crates/shellzip-gui/src/explorer/model.rs -> crates/shellzip-gui/src/explorer/model.rs`
- 1-file cycle: `crates/shellzip-gui/src/explorer/listing.rs -> crates/shellzip-gui/src/explorer/listing.rs`
- 3-file cycle: `crates/shellzip-cli/src/cli.rs -> crates/shellzip-cli/src/commands/mod.rs -> crates/shellzip-cli/src/commands/list.rs -> crates/shellzip-cli/src/cli.rs`

## Communities (44 total, 5 thin omitted)

### Community 0 - "worker.rs"
Cohesion: 0.14
Nodes (8): lock(), Outcome, Directory, Failed, Finished, Listed, SharedProgress, spawn()

### Community 1 - "Icon"
Cohesion: 0.08
Nodes (31): ALL, Caret, caret(), Down, Up, Icon, Archive, ArrowUp (+23 more)

### Community 2 - "walk.rs"
Cohesion: 0.08
Nodes (20): EntryKind, Dir, File, Other, Symlink, to_archive_path(), capture(), mode_bits() (+12 more)

### Community 3 - "extractor/tar.rs"
Cohesion: 0.14
Nodes (25): Compression, Gzip, None, Zstd, prefix_is_tar(), copy_single(), copy_tar_index(), dir_path() (+17 more)

### Community 4 - "Write"
Cohesion: 0.22
Nodes (6): Destination, destination(), Skip, Write, Fanout, copy_chunked()

### Community 5 - "extractor/zip.rs"
Cohesion: 0.07
Nodes (28): Budget, exists(), partial_path(), renamed(), symlink_destination(), write_atomic(), annotate(), entry_from() (+20 more)

### Community 6 - "stream.rs"
Cohesion: 0.18
Nodes (7): Kind, Gzip, Zstd, Stream, Gzip, Idle, Zstd

### Community 7 - "ui.rs"
Cohesion: 0.07
Nodes (10): Ask, Bars, byte_style(), sink(), ConflictChoice, Overwrite, Rename, Skip (+2 more)

### Community 8 - "Entry"
Cohesion: 0.12
Nodes (12): Password, reject_unsupported(), Entry, ArchiveReader, only_first(), Opened, unsupported(), Verify (+4 more)

### Community 9 - "EntryMeta"
Cohesion: 0.22
Nodes (6): ArchiveWriter, dir_name(), header_for(), TarArchiver, unix_secs(), EntryMeta

### Community 10 - "archiver/zip.rs"
Cohesion: 0.26
Nodes (6): method_for(), options_for(), rel_name(), target_text(), ZipArchiver, dos_parts()

### Community 11 - "ShellZipApp"
Cohesion: 0.11
Nodes (6): ShellZipApp, spawn_directory(), starting_folder(), folder_parent(), is_supported_archive(), location_text()

### Community 12 - "lib.rs"
Cohesion: 0.17
Nodes (14): add(), extract(), test(), add(), build_globs(), extract(), filter_is_restrictive(), name_filter() (+6 more)

### Community 13 - "lib.rs"
Cohesion: 0.09
Nodes (24): create(), create_from_path(), detect_bytes(), detect_file(), detect_header(), find_embedded(), find_subslice(), Format (+16 more)

### Community 14 - "round_trip.rs"
Cohesion: 0.14
Nodes (15): CompressOpts, StoreList, byte_cap_stops_extraction(), extract_all(), gzip_rejects_a_second_file(), opts_fast(), overwrite_skip_and_rename(), pack_tree() (+7 more)

### Community 15 - "add.rs"
Cohesion: 0.36
Nodes (4): ArchiveArgs, list(), Shared, test()

### Community 17 - "ProgressSink"
Cohesion: 0.22
Nodes (3): ProgressReader, ProgressReader<'a, R>, ProgressSink

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
Cohesion: 0.05
Nodes (55): ZipMethod, Deflate, Zstd, about(), address_bar(), AddressEvent, Jump, Submit (+47 more)

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
Nodes (14): format_bytes(), format_time(), byte_phrase(), dir_partial(), extension(), file_partial(), file_type_label(), format_modified() (+6 more)

### Community 30 - "app.rs"
Cohesion: 0.16
Nodes (10): folder_title(), archive_prefix(), folder_chain(), jumps(), JumpTarget, Folder, Prefix, normalize_sep() (+2 more)

### Community 31 - "Row"
Cohesion: 0.16
Nodes (9): named(), Row, totals(), totals_match_the_status_sentence(), cell_text(), header_clicked(), modified_text(), show() (+1 more)

### Community 32 - "Explorer"
Cohesion: 0.14
Nodes (9): SortColumn, Modified, Name, Size, Type, SortOrder, Asc, Desc (+1 more)

### Community 33 - "4. Phased implementation (Cursor task list)"
Cohesion: 0.12
Nodes (15): 1. Scope & feature‑parity matrix, 2. Recommended crate ecosystem, 3. Project structure, 4. Phased implementation (Cursor task list), 5. Testing & verification strategy, 6. Licensing notes, Design principles carried over from WinRAR, Milestone 0 — Skeleton (½ day) (+7 more)

### Community 34 - "sort_rows"
Cohesion: 0.40
Nodes (5): compare_column(), compare_name(), compare_option(), sort_keeps_parent_and_folders_above_files(), sort_rows()

### Community 35 - "Tier 1 — What can be proven (direct evidence)"
Cohesion: 0.20
Nodes (9): 1.1 Directory inventory (32 files, all in one flat directory), 1.2 Separation of concerns, 1.3 Imported Windows APIs grouped by capability, 1.4 CLI surface (from `Rar.exe -?`, `UnRAR.exe`, `Rar.txt`), Evidence ledger (summary), Tier 1 — What can be proven (direct evidence), Tier 2 — What can be inferred (architectural deductions), Tier 3 — What remains hidden (opaque internals) (+1 more)

### Community 37 - "ArchiveIndex"
Cohesion: 0.27
Nodes (7): Place, Archive, Folder, ArchiveIndex, directory_marker_hides_its_own_size(), entry(), index_builds_nested_folders()

### Community 40 - "listing.rs"
Cohesion: 0.43
Nodes (4): lists_a_folder_and_a_text_file(), read_directory(), row_for(), TempDir

### Community 41 - "RowKind"
Cohesion: 0.29
Nodes (7): rank(), RowKind, Archive, Directory, File, Parent, Symlink

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

## Knowledge Gaps
- **98 isolated node(s):** `shellzip-cli`, `Add`, `Extract`, `ExtractFlat`, `List` (+93 more)
  These have ≤1 connection - possible missing edges or undocumented components. (Counts symbols only; 262 node(s) total have ≤1 connection when file, concept and rationale nodes are included.)
- **5 thin communities (<3 nodes) omitted from report** — run `graphify query` to explore isolated nodes.

## Suggested Questions
_Questions this graph is uniquely positioned to answer:_

- **Why does `Error` connect `Error` to `worker.rs`, `walk.rs`, `extractor/tar.rs`, `extractor/zip.rs`, `stream.rs`, `ui.rs`, `Entry`, `EntryMeta`, `archiver/zip.rs`, `add.rs`, `lib.rs`, `lib.rs`, `round_trip.rs`, `add.rs`, `extract.rs`, `src/cli.rs`?**
  _High betweenness centrality (0.102) - this node is a cross-community bridge._
- **What connects `shellzip-cli`, `Add`, `Extract` to the rest of the system?**
  _98 weakly-connected nodes found - possible documentation gaps or missing edges._
- **Should `worker.rs` be split into smaller, more focused modules?**
  _Cohesion score 0.13852813852813853 - nodes in this community are weakly interconnected._
- **Why does `ProgressSink` connect `ProgressSink` to `worker.rs`, `extractor/tar.rs`, `extractor/zip.rs`, `ui.rs`, `Entry`, `lib.rs`, `add.rs`, `chrome.rs`?**
  _High betweenness centrality (0.068) - this node is a cross-community bridge._
- **Should `Icon` be split into smaller, more focused modules?**
  _Cohesion score 0.08246225319396051 - nodes in this community are weakly interconnected._
- **Why does `ShellZipApp` connect `ShellZipApp` to `Explorer`, `worker.rs`, `ArchiveIndex`, `chrome.rs`, `app.rs`, `Row`?**
  _High betweenness centrality (0.066) - this node is a cross-community bridge._
- **Should `walk.rs` be split into smaller, more focused modules?**
  _Cohesion score 0.0824524312896406 - nodes in this community are weakly interconnected._