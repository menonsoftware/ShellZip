# Graph Report - ShellZip  (2026-10-24)

## Corpus Check
- 80 files · ~58,902 words
- Verdict: corpus is large enough that graph structure adds value.
- Unclassified: 26 file(s) not represented in the graph (top: .mdc 11, .rar 8, (none) 4)

## Summary
- 1392 nodes · 3249 edges · 70 communities (60 shown, 10 thin omitted)
- Extraction: 97% EXTRACTED · 3% INFERRED · 0% AMBIGUOUS · INFERRED: 92 edges (avg confidence: 0.85)
- Token cost: 0 input · 0 output

## Graph Freshness
- Built from commit: `e97d7be5`
- Run `git rev-parse HEAD` and compare to check if the graph is stale.
- Run `graphify update .` after code changes (no API cost).

## Community Hubs (Navigation)
- worker.rs
- Icon
- ops.rs
- policy.rs
- src/sfx.rs
- Password
- stream.rs
- shellzip-sfx/src/main.rs
- walk.rs
- EntryMeta
- archiver/zip.rs
- ShellZipApp
- extractor/mod.rs
- Format
- round_trip.rs
- list.rs
- extractor/tar.rs
- ProgressSink
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
- tests/cli.rs
- view.rs
- Test patterns
- 4. Phased implementation (Cursor task list)
- CompressOpts
- Tier 1 — What can be proven (direct evidence)
- Rust review checklist
- CompWriter
- extractor/zip.rs
- Rust performance
- add_dialog.rs
- Shared
- Rust quality gate
- UpdateMode
- milestone3.rs
- MenuCommand
- prompt.rs
- Icon
- listing.rs
- AddressEvent
- sevenz.rs
- BENCH.md
- Explorer
- NopProgress
- ToolCommand
- ArchiveIndex
- explorer/mod.rs
- path
- milestone2.rs
- motw.rs
- tests/extract.rs
- zip_threads.rs
- copy_chunked
- pathbuf
- NopProgress
- sort_rows
- Target
- Budget

## God Nodes (most connected - your core abstractions)
1. `ProgressSink` - 61 edges
2. `ShellZipApp` - 58 edges
3. `CompressOpts` - 54 edges
4. `Password` - 50 edges
5. `Entry` - 41 edges
6. `Format` - 34 edges
7. `ExtractOpts` - 29 edges
8. `Icon` - 28 edges
9. `Command` - 27 edges
10. `EntryMeta` - 27 edges

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
- 1-file cycle: `crates/shellzip-gui/src/add_dialog.rs -> crates/shellzip-gui/src/add_dialog.rs`
- 1-file cycle: `crates/shellzip-gui/src/explorer/model.rs -> crates/shellzip-gui/src/explorer/model.rs`
- 1-file cycle: `crates/shellzip-gui/src/explorer/listing.rs -> crates/shellzip-gui/src/explorer/listing.rs`
- 1-file cycle: `crates/shellzip-gui/src/icons.rs -> crates/shellzip-gui/src/icons.rs`
- 2-file cycle: `crates/shellzip-core/src/archiver/mod.rs -> crates/shellzip-core/src/archiver/parallel.rs -> crates/shellzip-core/src/archiver/mod.rs`
- 2-file cycle: `crates/shellzip-cli/src/commands/mod.rs -> crates/shellzip-cli/src/commands/sfx.rs -> crates/shellzip-cli/src/commands/mod.rs`
- 2-file cycle: `crates/shellzip-cli/src/cli.rs -> crates/shellzip-cli/src/commands/mod.rs -> crates/shellzip-cli/src/cli.rs`
- 2-file cycle: `crates/shellzip-cli/src/commands/delete.rs -> crates/shellzip-cli/src/commands/mod.rs -> crates/shellzip-cli/src/commands/delete.rs`
- 2-file cycle: `crates/shellzip-cli/src/commands/mod.rs -> crates/shellzip-cli/src/commands/print.rs -> crates/shellzip-cli/src/commands/mod.rs`
- 3-file cycle: `crates/shellzip-core/src/archiver/mod.rs -> crates/shellzip-core/src/archiver/parallel.rs -> crates/shellzip-core/src/archiver/zip.rs -> crates/shellzip-core/src/archiver/mod.rs`
- 3-file cycle: `crates/shellzip-cli/src/cli.rs -> crates/shellzip-cli/src/commands/mod.rs -> crates/shellzip-cli/src/commands/sfx.rs -> crates/shellzip-cli/src/cli.rs`
- 3-file cycle: `crates/shellzip-cli/src/cli.rs -> crates/shellzip-cli/src/commands/mod.rs -> crates/shellzip-cli/src/commands/list.rs -> crates/shellzip-cli/src/cli.rs`
- 3-file cycle: `crates/shellzip-cli/src/cli.rs -> crates/shellzip-cli/src/commands/mod.rs -> crates/shellzip-cli/src/commands/delete.rs -> crates/shellzip-cli/src/cli.rs`
- 3-file cycle: `crates/shellzip-cli/src/cli.rs -> crates/shellzip-cli/src/commands/mod.rs -> crates/shellzip-cli/src/commands/print.rs -> crates/shellzip-cli/src/cli.rs`

## Communities (70 total, 10 thin omitted)

### Community 0 - "worker.rs"
Cohesion: 0.12
Nodes (8): lock(), Outcome, Directory, Failed, Finished, Listed, SharedProgress, spawn()

### Community 1 - "Icon"
Cohesion: 0.13
Nodes (17): ALL, Caret, caret(), Down, Up, MASTER_POINTS, paint(), paint_caret() (+9 more)

### Community 2 - "ops.rs"
Cohesion: 0.10
Nodes (31): member_key(), add(), add_inner(), add_prompted(), add_sfx(), add_stdin(), build_globs(), Counting (+23 more)

### Community 3 - "policy.rs"
Cohesion: 0.16
Nodes (21): Destination, destination(), Skip, Write, exists(), partial_path(), renamed(), stage_path() (+13 more)

### Community 4 - "src/sfx.rs"
Cohesion: 0.09
Nodes (32): SfxArgs, default_output(), existing(), find_stub(), has_trailer(), sfx(), apply_delta(), build() (+24 more)

### Community 5 - "Password"
Cohesion: 0.05
Nodes (37): List, debug_hides_the_password(), Password, archive_at(), copy_rar_index(), copy_staged(), dos_time(), entry_from() (+29 more)

### Community 6 - "stream.rs"
Cohesion: 0.10
Nodes (9): commit_output(), Kind, Gzip, Zstd, SingleArchiver, Stream, Gzip, Idle (+1 more)

### Community 7 - "shellzip-sfx/src/main.rs"
Cohesion: 0.11
Nodes (8): Ask, Bars, byte_style(), sink(), ConflictChoice, Overwrite, Rename, Skip

### Community 8 - "walk.rs"
Cohesion: 0.08
Nodes (22): EntryKind, Dir, File, Other, Symlink, format_attrs(), format_blake(), to_archive_path() (+14 more)

### Community 9 - "EntryMeta"
Cohesion: 0.19
Nodes (6): ArchiveWriter, dir_name(), header_for(), TarArchiver, unix_secs(), EntryMeta

### Community 10 - "archiver/zip.rs"
Cohesion: 0.18
Nodes (7): apply_comment(), method_for(), options_for(), rel_name(), target_text(), ZipArchiver, dos_parts()

### Community 11 - "ShellZipApp"
Cohesion: 0.08
Nodes (11): decide_member(), find_stub(), folder_title(), is_sfx_name(), Place, Archive, Folder, ShellZipApp (+3 more)

### Community 12 - "extractor/mod.rs"
Cohesion: 0.11
Nodes (13): ArchiveReader, only_first(), Opened, resolve(), unsupported(), Verify, list(), list_with() (+5 more)

### Community 13 - "Format"
Cohesion: 0.10
Nodes (26): creatable_extensions_round_trip(), detect_bytes(), detect_file(), detect_header(), find_embedded(), find_subslice(), Format, Bzip2 (+18 more)

### Community 14 - "round_trip.rs"
Cohesion: 0.26
Nodes (16): adding_to_an_existing_zip_keeps_the_other_files(), byte_cap_stops_extraction(), existing_tar_is_left_in_place_without_force(), extract_all(), gzip_rejects_a_second_file(), opts_fast(), overwrite_skip_and_rename(), pack_tree() (+8 more)

### Community 15 - "list.rs"
Cohesion: 0.23
Nodes (7): ArchiveArgs, ListArgs, display_name(), flags_text(), list(), list_technical(), test()

### Community 16 - "extractor/tar.rs"
Cohesion: 0.11
Nodes (34): Compression, Bzip2, Gzip, None, Xz, Zstd, Entry, mark_extracted() (+26 more)

### Community 17 - "ProgressSink"
Cohesion: 0.09
Nodes (14): tune(), CreatableFormat, add(), add_prompted(), add_sfx(), add_stdin(), delete(), extract() (+6 more)

### Community 18 - "Error"
Cohesion: 0.17
Nodes (11): Error, Archive, BadChecksum, BadPassword, Encrypted, Interrupted, Io, NoFiles (+3 more)

### Community 19 - "extract.rs"
Cohesion: 0.22
Nodes (9): ExtractArgs, Overwrite, overwrite_mode(), OverwriteArg, Always, Ask, Rename, Skip (+1 more)

### Community 20 - "Scaffold templates"
Cohesion: 0.10
Nodes (19): Criterion benchmark, Example, Extra binary in an existing crate, Finish checklist, Integration test file, New module in an existing crate, New `szip` subcommand, New workspace crate (+11 more)

### Community 21 - "chrome.rs"
Cohesion: 0.17
Nodes (13): about(), bar_frame(), caret_button(), confirm_delete(), dismiss_button(), icon_button(), info_window(), menu_bar() (+5 more)

### Community 22 - "src/cli.rs"
Cohesion: 0.14
Nodes (8): Cli, dispatch(), init_tracing(), normalize_args(), password_from_env(), prompt_password(), run(), take_password()

### Community 23 - "theme.rs"
Cohesion: 0.16
Nodes (16): ADD, apply(), CANVAS, DANGER, EXTRACT, FOLDER, HOVER, INK (+8 more)

### Community 24 - "Command"
Cohesion: 0.15
Nodes (12): Command, Add, Complete, Delete, Extract, ExtractFlat, Freshen, ListTechnical (+4 more)

### Community 25 - "xtask/src/main.rs"
Cohesion: 0.17
Nodes (21): bench(), build_fixtures(), collect_files(), crc32(), find_rar(), find_tool(), FIXTURE_PASSWORD, main() (+13 more)

### Community 27 - "shellzip-core"
Cohesion: 0.40
Nodes (5): shellzip-cli, shellzip-core, shellzip-gui, shellzip-sfx, xtask

### Community 29 - "model.rs"
Cohesion: 0.15
Nodes (16): format_bytes(), format_time(), archive_prefix(), byte_phrase(), dir_partial(), extension(), file_partial(), file_type_label() (+8 more)

### Community 30 - "tests/cli.rs"
Cohesion: 0.19
Nodes (15): add_list_test_extract_round_trip(), add_sfx_writes_a_listable_package(), add_to_an_existing_zip_keeps_previous_files(), help_explains_that_zip_encryption_leaves_names_visible(), log_file_receives_debug_events(), rar_and_7z_are_not_created(), reads_a_tar_from_the_system_tool(), reads_a_zip_from_compress_archive() (+7 more)

### Community 31 - "view.rs"
Cohesion: 0.19
Nodes (8): format_crc(), cell_text(), crc_text(), header_clicked(), modified_text(), packed_text(), show(), size_text()

### Community 32 - "Test patterns"
Cohesion: 0.11
Nodes (17): CLI end-to-end with `assert_cmd` (`tests/cli.rs`), Counting fake for a trait, In-memory reader/writer, Integration round-trip (`tests/<scenario>.rs`), Interop with OS tools, Platform-gated test, Property test with `proptest`, Snapshot test with `insta` (+9 more)

### Community 33 - "4. Phased implementation (Cursor task list)"
Cohesion: 0.12
Nodes (15): 1. Scope & feature‑parity matrix, 2. Recommended crate ecosystem, 3. Project structure, 4. Phased implementation (Cursor task list), 5. Testing & verification strategy, 6. Licensing notes, Design principles carried over from WinRAR, Milestone 0 — Skeleton (½ day) (+7 more)

### Community 34 - "CompressOpts"
Cohesion: 0.27
Nodes (8): add_zip(), compress_file(), MEMORY_LIMIT, merge(), Piece, Memory, piece_path(), Temp

### Community 35 - "Tier 1 — What can be proven (direct evidence)"
Cohesion: 0.20
Nodes (9): 1.1 Directory inventory (32 files, all in one flat directory), 1.2 Separation of concerns, 1.3 Imported Windows APIs grouped by capability, 1.4 CLI surface (from `Rar.exe -?`, `UnRAR.exe`, `Rar.txt`), Evidence ledger (summary), Tier 1 — What can be proven (direct evidence), Tier 2 — What can be inferred (architectural deductions), Tier 3 — What remains hidden (opaque internals) (+1 more)

### Community 36 - "Rust review checklist"
Cohesion: 0.13
Nodes (13): 1. Correctness, 2. Error handling, 3. Security, 4. Ownership and performance, 5. Concurrency, 6. Tests, 7. API and docs, 8. Manifest and tooling (+5 more)

### Community 37 - "CompWriter"
Cohesion: 0.18
Nodes (4): CompWriter, Gzip, Plain, Zstd

### Community 38 - "extractor/zip.rs"
Cohesion: 0.15
Nodes (11): annotate(), entry_from(), extract_zip(), extract_zip_index(), host_label(), list_zip(), method_name(), open_zip() (+3 more)

### Community 39 - "Rust performance"
Cohesion: 0.29
Nodes (6): 1. Baseline, 2. Find the hot spot, 3. Apply the cheapest fix the profile justifies, 4. Binary size, 5. Verify and report, Rust performance

### Community 40 - "add_dialog.rs"
Cohesion: 0.05
Nodes (66): creatable_formats(), Overwrite, Always, Ask, Never, Rename, update_mode_labels_follow_the_add_dialog(), UpdateMode (+58 more)

### Community 41 - "Shared"
Cohesion: 0.33
Nodes (5): DeleteArgs, PrintArgs, delete(), Shared, print_entry()

### Community 42 - "Rust quality gate"
Cohesion: 0.29
Nodes (6): Fix clippy, don't silence it, Fix tests, Optional supply-chain checks, Report, Rust quality gate, Workflow

### Community 43 - "UpdateMode"
Cohesion: 0.21
Nodes (8): AddArgs, compress_opts(), MethodArg, Deflate, Zstd, ZipMethod, add(), pack_sfx()

### Community 44 - "milestone3.rs"
Cohesion: 0.16
Nodes (17): aes_method_with(), assert_matches_unrar(), bzip2_method(), deflate_method(), encrypting_an_existing_zip_requires_force(), extract_bytes(), find_unrar(), FIXTURE_PASSWORD (+9 more)

### Community 45 - "MenuCommand"
Cohesion: 0.18
Nodes (11): MenuCommand, About, Add, Delete, Exit, Extract, Info, Open (+3 more)

### Community 46 - "prompt.rs"
Cohesion: 0.07
Nodes (49): accept_password(), ask(), ask_console(), ask_gui(), attach_console(), attach_parent(), Choice, All (+41 more)

### Community 47 - "Icon"
Cohesion: 0.10
Nodes (19): directories_use_the_yellow_folder(), row_glyph(), selected_glyphs_use_the_selection_ink(), Icon, Archive, ArrowUp, CircleCheck, File (+11 more)

### Community 48 - "listing.rs"
Cohesion: 0.43
Nodes (4): lists_a_folder_and_a_text_file(), read_directory(), row_for(), TempDir

### Community 49 - "AddressEvent"
Cohesion: 0.23
Nodes (13): address_bar(), AddressEvent, Jump, Submit, Up, breadcrumbs(), crumb_label(), parent_menu() (+5 more)

### Community 50 - "sevenz.rs"
Cohesion: 0.17
Nodes (17): copy_sevenz_index(), drain(), encrypted_names(), entry_from(), extract_sevenz(), is_checksum(), list_sevenz(), map_sevenz() (+9 more)

### Community 52 - "Explorer"
Cohesion: 0.12
Nodes (11): SortColumn, Crc, Modified, Name, Packed, Size, Type, SortOrder (+3 more)

### Community 54 - "NopProgress"
Cohesion: 0.15
Nodes (7): test(), test(), TestReport, ensure_running(), ProgressReader, ProgressReader<'a, R>, ProgressSink

### Community 55 - "ToolCommand"
Cohesion: 0.33
Nodes (6): ToolCommand, Add, Delete, Extract, Info, Test

### Community 56 - "ArchiveIndex"
Cohesion: 0.43
Nodes (4): ArchiveIndex, directory_marker_hides_its_own_size(), entry(), index_builds_nested_folders()

### Community 57 - "explorer/mod.rs"
Cohesion: 0.12
Nodes (15): folder_chain(), folder_parent(), is_supported_archive(), named(), path_under(), Row, RowKind, Archive (+7 more)

### Community 58 - "path"
Cohesion: 0.24
Nodes (6): copy_in(), create(), create_from_path(), prepare_output(), prepare_temp(), temp_beside()

### Community 60 - "milestone2.rs"
Cohesion: 0.22
Nodes (11): ask_can_skip_replace_or_rename(), freshen_missing_archive_is_not_found(), opts(), parallel_zip_matches_single_thread_bytes(), skip_leaves_existing_members_and_adds_new_ones(), stdin_streams_zst_and_spools_tar(), sync_drops_members_that_were_not_selected(), technical_list_fills_blake2sp() (+3 more)

### Community 61 - "motw.rs"
Cohesion: 0.60
Nodes (4): copy_zone(), MAX_ZONE_BYTES, propagate(), zone_path()

### Community 62 - "tests/extract.rs"
Cohesion: 0.33
Nodes (7): built_stub_contains_the_shellzip_icon(), encrypted_sfx(), silent_encrypted_stub_fails_without_a_password(), silent_encrypted_stub_rejects_a_wrong_password(), stub_bin(), stub_extracts_an_encrypted_zip_when_the_password_is_set(), stub_extracts_its_appended_zip()

### Community 67 - "sort_rows"
Cohesion: 0.33
Nodes (6): compare_column(), compare_name(), compare_option(), rank(), sort_keeps_parent_and_folders_above_files(), sort_rows()

### Community 68 - "Target"
Cohesion: 0.40
Nodes (5): Target, ArchiveDir, ArchiveMember, Parent, Path

## Knowledge Gaps
- **136 isolated node(s):** `shellzip-cli`, `Add`, `Update`, `Freshen`, `Delete` (+131 more)
  These have ≤1 connection - possible missing edges or undocumented components. (Counts symbols only; 425 node(s) total have ≤1 connection when file, concept and rationale nodes are included.)
- **10 thin communities (<3 nodes) omitted from report** — run `graphify query` to explore isolated nodes.

## Suggested Questions
_Questions this graph is uniquely positioned to answer:_

- **Why does `Error` connect `Error` to `worker.rs`, `ops.rs`, `policy.rs`, `src/sfx.rs`, `Password`, `stream.rs`, `shellzip-sfx/src/main.rs`, `walk.rs`, `EntryMeta`, `archiver/zip.rs`, `extractor/mod.rs`, `Format`, `round_trip.rs`, `list.rs`, `extractor/tar.rs`, `ProgressSink`, `extract.rs`, `src/cli.rs`, `CompressOpts`, `extractor/zip.rs`, `Shared`, `UpdateMode`, `milestone3.rs`, `prompt.rs`, `sevenz.rs`, `path`, `motw.rs`?**
  _High betweenness centrality (0.098) - this node is a cross-community bridge._
- **What connects `shellzip-cli`, `Add`, `Update` to the rest of the system?**
  _136 weakly-connected nodes found - possible documentation gaps or missing edges._
- **Should `worker.rs` be split into smaller, more focused modules?**
  _Cohesion score 0.12 - nodes in this community are weakly interconnected._
- **Why does `CompressOpts` connect `ProgressSink` to `CompressOpts`, `ops.rs`, `CompWriter`, `stream.rs`, `Password`, `walk.rs`, `EntryMeta`, `archiver/zip.rs`, `UpdateMode`, `add_dialog.rs`, `round_trip.rs`, `src/cli.rs`, `path`, `milestone2.rs`?**
  _High betweenness centrality (0.068) - this node is a cross-community bridge._
- **Should `Icon` be split into smaller, more focused modules?**
  _Cohesion score 0.1349206349206349 - nodes in this community are weakly interconnected._
- **Why does `ProgressSink` connect `NopProgress` to `worker.rs`, `CompressOpts`, `ops.rs`, `NopProgress`, `Password`, `extractor/zip.rs`, `shellzip-sfx/src/main.rs`, `Shared`, `extractor/mod.rs`, `extractor/tar.rs`, `ProgressSink`, `sevenz.rs`, `theme.rs`?**
  _High betweenness centrality (0.067) - this node is a cross-community bridge._
- **Should `ops.rs` be split into smaller, more focused modules?**
  _Cohesion score 0.1048780487804878 - nodes in this community are weakly interconnected._