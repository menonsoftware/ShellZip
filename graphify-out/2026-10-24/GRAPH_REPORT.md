# Graph Report - ShellZip  (2026-10-24)

## Corpus Check
- 81 files · ~61,712 words
- Verdict: corpus is large enough that graph structure adds value.
- Unclassified: 26 file(s) not represented in the graph (top: .mdc 11, .rar 8, (none) 4)

## Summary
- 1496 nodes · 3480 edges · 70 communities (61 shown, 9 thin omitted)
- Extraction: 97% EXTRACTED · 3% INFERRED · 0% AMBIGUOUS · INFERRED: 91 edges (avg confidence: 0.85)
- Token cost: 0 input · 0 output

## Graph Freshness
- Built from commit: `8371e7bd`
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
- setup.rs
- archiver/zip.rs
- ShellZipApp
- CrcWriter
- Format
- round_trip.rs
- list.rs
- extractor/tar.rs
- CompressOpts
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
- UpdateMode
- view.rs
- Test patterns
- 4. Phased implementation (Cursor task list)
- CompressOpts
- Tier 1 — What can be proven (direct evidence)
- Rust review checklist
- archiver/tar.rs
- extractor/zip.rs
- Rust performance
- add_dialog.rs
- print.rs
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
- setup_template
- ToolCommand
- Row
- explorer/mod.rs
- path
- milestone2.rs
- HWND
- SetupState
- BarProgress
- copy_chunked
- prepare_window
- RowKind
- sort_rows
- MethodArg
- DrawItem

## God Nodes (most connected - your core abstractions)
1. `ProgressSink` - 63 edges
2. `ShellZipApp` - 58 edges
3. `CompressOpts` - 54 edges
4. `Password` - 53 edges
5. `Entry` - 41 edges
6. `Format` - 34 edges
7. `ExtractOpts` - 29 edges
8. `Icon` - 28 edges
9. `Command` - 27 edges
10. `EntryMeta` - 27 edges

## Surprising Connections (you probably didn't know these)
- `zip_comment()` --calls--> `detect_file()`  [INFERRED]
  crates/shellzip-core/src/ops.rs → crates/shellzip-core/src/detect.rs
- `flags_text()` --references--> `Entry`  [EXTRACTED]
  crates/shellzip-cli/src/commands/list.rs → crates/shellzip-core/src/entry.rs
- `xtask` --depends_on--> `shellzip-core`  [EXTRACTED]
  xtask/Cargo.toml → crates/shellzip-core/Cargo.toml
- `create_from_path()` --calls--> `format_from_extension()`  [INFERRED]
  crates/shellzip-core/src/archiver/mod.rs → crates/shellzip-core/src/detect.rs
- `compress_file()` --calls--> `method_for()`  [INFERRED]
  crates/shellzip-core/src/archiver/parallel.rs → crates/shellzip-core/src/archiver/zip.rs

## Import Cycles
- 1-file cycle: `crates/shellzip-sfx/src/setup.rs -> crates/shellzip-sfx/src/setup.rs`
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

## Communities (70 total, 9 thin omitted)

### Community 0 - "worker.rs"
Cohesion: 0.12
Nodes (8): lock(), Outcome, Directory, Failed, Finished, Listed, SharedProgress, spawn()

### Community 1 - "Icon"
Cohesion: 0.13
Nodes (17): ALL, Caret, caret(), Down, Up, MASTER_POINTS, paint(), paint_caret() (+9 more)

### Community 2 - "ops.rs"
Cohesion: 0.05
Nodes (48): member_key(), add(), add_prompted(), add_sfx(), add_stdin(), delete(), list_with(), add() (+40 more)

### Community 3 - "policy.rs"
Cohesion: 0.16
Nodes (20): Destination, destination(), Skip, Write, exists(), partial_path(), renamed(), stage_path() (+12 more)

### Community 4 - "src/sfx.rs"
Cohesion: 0.09
Nodes (33): SfxArgs, default_output(), existing(), find_stub(), has_trailer(), sfx(), extract(), apply_delta() (+25 more)

### Community 5 - "Password"
Cohesion: 0.13
Nodes (22): Password, archive_at(), copy_rar_index(), copy_staged(), dos_time(), entry_from(), extract_rar(), first_volume() (+14 more)

### Community 6 - "stream.rs"
Cohesion: 0.13
Nodes (8): commit_output(), Kind, Gzip, Zstd, Stream, Gzip, Idle, Zstd

### Community 7 - "shellzip-sfx/src/main.rs"
Cohesion: 0.07
Nodes (20): Ask, Bars, byte_style(), sink(), ConflictChoice, Overwrite, Rename, Skip (+12 more)

### Community 8 - "walk.rs"
Cohesion: 0.08
Nodes (22): dos_parts(), EntryKind, Dir, File, Other, Symlink, format_attrs(), to_archive_path() (+14 more)

### Community 9 - "setup.rs"
Cohesion: 0.06
Nodes (11): CB_ADDSTRING, ID_BROWSE, ID_DEST, ID_INTRO, ID_PROGRESS, INTRO, PBM_SETPOS, PBM_SETRANGE32 (+3 more)

### Community 10 - "archiver/zip.rs"
Cohesion: 0.18
Nodes (7): apply_comment(), method_for(), options_for(), rel_name(), target_text(), ZipArchiver, EntryMeta

### Community 11 - "ShellZipApp"
Cohesion: 0.08
Nodes (11): decide_member(), find_stub(), folder_title(), is_sfx_name(), Place, Archive, Folder, ShellZipApp (+3 more)

### Community 12 - "CrcWriter"
Cohesion: 0.14
Nodes (4): Tee, blake2sp_matches_the_reference_vector(), Blake2spWriter, CrcWriter

### Community 13 - "Format"
Cohesion: 0.06
Nodes (36): creatable_extensions_round_trip(), CreatableFormat, detect_bytes(), detect_file(), detect_header(), find_embedded(), find_subslice(), Format (+28 more)

### Community 14 - "round_trip.rs"
Cohesion: 0.15
Nodes (22): copy_zone(), MAX_ZONE_BYTES, propagate(), zone_path(), adding_to_an_existing_zip_keeps_the_other_files(), byte_cap_stops_extraction(), existing_tar_is_left_in_place_without_force(), extract_all() (+14 more)

### Community 15 - "list.rs"
Cohesion: 0.21
Nodes (8): ArchiveArgs, ListArgs, display_name(), flags_text(), list(), list_technical(), test(), format_blake()

### Community 16 - "extractor/tar.rs"
Cohesion: 0.12
Nodes (26): Compression, Bzip2, Gzip, None, Xz, Zstd, Entry, copy_single() (+18 more)

### Community 17 - "CompressOpts"
Cohesion: 0.15
Nodes (3): SingleArchiver, tune(), CompressOpts

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
Nodes (13): about(), bar_frame(), caret_button(), confirm_delete(), dismiss_button(), icon_button(), info_window(), menu_bar() (+5 more)

### Community 22 - "src/cli.rs"
Cohesion: 0.10
Nodes (9): Cli, dispatch(), init_tracing(), normalize_args(), password_from_env(), prompt_password(), run(), take_password() (+1 more)

### Community 23 - "theme.rs"
Cohesion: 0.17
Nodes (17): List, ADD, apply(), CANVAS, DANGER, EXTRACT, FOLDER, HOVER (+9 more)

### Community 24 - "Command"
Cohesion: 0.15
Nodes (12): Command, Add, Complete, Delete, Extract, ExtractFlat, Freshen, ListTechnical (+4 more)

### Community 25 - "xtask/src/main.rs"
Cohesion: 0.06
Nodes (44): add_list_test_extract_round_trip(), add_sfx_writes_a_listable_package(), add_to_an_existing_zip_keeps_previous_files(), help_explains_that_zip_encryption_leaves_names_visible(), log_file_receives_debug_events(), rar_and_7z_are_not_created(), reads_a_tar_from_the_system_tool(), reads_a_zip_from_compress_archive() (+36 more)

### Community 27 - "shellzip-core"
Cohesion: 0.40
Nodes (5): shellzip-cli, shellzip-core, shellzip-gui, shellzip-sfx, xtask

### Community 29 - "model.rs"
Cohesion: 0.15
Nodes (16): format_bytes(), format_time(), archive_prefix(), byte_phrase(), dir_partial(), extension(), file_partial(), file_type_label() (+8 more)

### Community 30 - "UpdateMode"
Cohesion: 0.12
Nodes (13): DEFAULT_STORE_EXTS, MAX_THREADS, shrinks_zstd_window_for_small_inputs(), StoreList, update_mode_labels_follow_the_add_dialog(), UpdateMode, Add, .ALL (+5 more)

### Community 31 - "view.rs"
Cohesion: 0.21
Nodes (8): format_crc(), cell_text(), crc_text(), header_clicked(), modified_text(), packed_text(), show(), size_text()

### Community 32 - "Test patterns"
Cohesion: 0.11
Nodes (17): CLI end-to-end with `assert_cmd` (`tests/cli.rs`), Counting fake for a trait, In-memory reader/writer, Integration round-trip (`tests/<scenario>.rs`), Interop with OS tools, Platform-gated test, Property test with `proptest`, Snapshot test with `insta` (+9 more)

### Community 33 - "4. Phased implementation (Cursor task list)"
Cohesion: 0.12
Nodes (15): 1. Scope & feature‑parity matrix, 2. Recommended crate ecosystem, 3. Project structure, 4. Phased implementation (Cursor task list), 5. Testing & verification strategy, 6. Licensing notes, Design principles carried over from WinRAR, Milestone 0 — Skeleton (½ day) (+7 more)

### Community 34 - "CompressOpts"
Cohesion: 0.23
Nodes (9): copy_in(), add_zip(), compress_file(), MEMORY_LIMIT, merge(), Piece, Memory, piece_path() (+1 more)

### Community 35 - "Tier 1 — What can be proven (direct evidence)"
Cohesion: 0.20
Nodes (9): 1.1 Directory inventory (32 files, all in one flat directory), 1.2 Separation of concerns, 1.3 Imported Windows APIs grouped by capability, 1.4 CLI surface (from `Rar.exe -?`, `UnRAR.exe`, `Rar.txt`), Evidence ledger (summary), Tier 1 — What can be proven (direct evidence), Tier 2 — What can be inferred (architectural deductions), Tier 3 — What remains hidden (opaque internals) (+1 more)

### Community 36 - "Rust review checklist"
Cohesion: 0.13
Nodes (13): 1. Correctness, 2. Error handling, 3. Security, 4. Ownership and performance, 5. Concurrency, 6. Tests, 7. API and docs, 8. Manifest and tooling (+5 more)

### Community 37 - "archiver/tar.rs"
Cohesion: 0.11
Nodes (8): CompWriter, Gzip, Plain, Zstd, dir_name(), header_for(), TarArchiver, unix_secs()

### Community 38 - "extractor/zip.rs"
Cohesion: 0.12
Nodes (20): mark_extracted(), write_atomic(), finish_staged(), dir_path(), extract_regular(), extract_single(), extract_tar(), annotate() (+12 more)

### Community 39 - "Rust performance"
Cohesion: 0.29
Nodes (6): 1. Baseline, 2. Find the hot spot, 3. Apply the cheapest fix the profile justifies, 4. Binary size, 5. Verify and report, Rust performance

### Community 40 - "add_dialog.rs"
Cohesion: 0.06
Nodes (57): creatable_formats(), Overwrite, Always, Ask, Never, Rename, ZipMethod, Deflate (+49 more)

### Community 42 - "Rust quality gate"
Cohesion: 0.29
Nodes (6): Fix clippy, don't silence it, Fix tests, Optional supply-chain checks, Report, Rust quality gate, Workflow

### Community 43 - "UpdateMode"
Cohesion: 0.24
Nodes (7): AddArgs, compress_opts(), DeleteArgs, add(), pack_sfx(), delete(), Shared

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
Cohesion: 0.14
Nodes (18): Budget, copy_sevenz_index(), drain(), encrypted_names(), entry_from(), extract_sevenz(), is_checksum(), list_sevenz() (+10 more)

### Community 52 - "Explorer"
Cohesion: 0.12
Nodes (11): SortColumn, Crc, Modified, Name, Packed, Size, Type, SortOrder (+3 more)

### Community 54 - "setup_template"
Cohesion: 0.18
Nodes (16): align4(), ComInit, Control, dword_words(), finish_worker(), init_common_controls(), push_atom(), push_named() (+8 more)

### Community 55 - "ToolCommand"
Cohesion: 0.33
Nodes (6): ToolCommand, Add, Delete, Extract, Info, Test

### Community 56 - "Row"
Cohesion: 0.21
Nodes (10): ArchiveIndex, directory_marker_hides_its_own_size(), entry(), index_builds_nested_folders(), Row, Target, ArchiveDir, ArchiveMember (+2 more)

### Community 57 - "explorer/mod.rs"
Cohesion: 0.18
Nodes (7): folder_chain(), folder_parent(), named(), path_under(), totals(), totals_label(), totals_match_the_status_sentence()

### Community 58 - "path"
Cohesion: 0.20
Nodes (6): ArchiveWriter, create(), create_from_path(), prepare_output(), prepare_temp(), temp_beside()

### Community 60 - "milestone2.rs"
Cohesion: 0.22
Nodes (11): ask_can_skip_replace_or_rename(), freshen_missing_archive_is_not_found(), opts(), parallel_zip_matches_single_thread_bytes(), skip_leaves_existing_members_and_adds_new_ones(), stdin_streams_zst_and_spools_tar(), sync_drops_members_that_were_not_selected(), technical_list_fills_blake2sp() (+3 more)

### Community 61 - "HWND"
Cohesion: 0.25
Nodes (16): enable_window(), EnableWindow(), finish_if_ready(), on_command(), on_extract(), paint_progress(), percent(), prepare_folder() (+8 more)

### Community 62 - "SetupState"
Cohesion: 0.21
Nodes (7): browse_initialized(), destroy_icon(), install_icons(), load_icon(), paint_banner(), rgb(), SetupState

### Community 65 - "prepare_window"
Cohesion: 0.36
Nodes (8): add_choice(), browse_folder(), on_browse(), prepare_window(), set_text(), set_window_text(), wide_os(), wide_str()

### Community 66 - "RowKind"
Cohesion: 0.33
Nodes (6): RowKind, Archive, Directory, File, Parent, Symlink

### Community 67 - "sort_rows"
Cohesion: 0.33
Nodes (6): compare_column(), compare_name(), compare_option(), rank(), sort_keeps_parent_and_folders_above_files(), sort_rows()

### Community 68 - "MethodArg"
Cohesion: 0.50
Nodes (4): MethodArg, Deflate, Zstd, ZipMethod

## Knowledge Gaps
- **141 isolated node(s):** `shellzip-cli`, `Add`, `Update`, `Freshen`, `Delete` (+136 more)
  These have ≤1 connection - possible missing edges or undocumented components. (Counts symbols only; 466 node(s) total have ≤1 connection when file, concept and rationale nodes are included.)
- **9 thin communities (<3 nodes) omitted from report** — run `graphify query` to explore isolated nodes.

## Suggested Questions
_Questions this graph is uniquely positioned to answer:_

- **Why does `Error` connect `Error` to `worker.rs`, `ops.rs`, `policy.rs`, `src/sfx.rs`, `Password`, `stream.rs`, `shellzip-sfx/src/main.rs`, `walk.rs`, `setup.rs`, `archiver/zip.rs`, `Format`, `round_trip.rs`, `list.rs`, `extractor/tar.rs`, `CompressOpts`, `extract.rs`, `src/cli.rs`, `UpdateMode`, `CompressOpts`, `archiver/tar.rs`, `extractor/zip.rs`, `print.rs`, `UpdateMode`, `milestone3.rs`, `prompt.rs`, `sevenz.rs`, `path`?**
  _High betweenness centrality (0.102) - this node is a cross-community bridge._
- **What connects `shellzip-cli`, `Add`, `Update` to the rest of the system?**
  _141 weakly-connected nodes found - possible documentation gaps or missing edges._
- **Should `worker.rs` be split into smaller, more focused modules?**
  _Cohesion score 0.12 - nodes in this community are weakly interconnected._
- **Why does `ProgressSink` connect `ops.rs` to `worker.rs`, `CompressOpts`, `src/sfx.rs`, `Password`, `extractor/zip.rs`, `shellzip-sfx/src/main.rs`, `setup.rs`, `UpdateMode`, `Format`, `extractor/tar.rs`, `sevenz.rs`, `theme.rs`, `BarProgress`?**
  _High betweenness centrality (0.069) - this node is a cross-community bridge._
- **Should `Icon` be split into smaller, more focused modules?**
  _Cohesion score 0.1349206349206349 - nodes in this community are weakly interconnected._
- **Why does `CompressOpts` connect `CompressOpts` to `CompressOpts`, `ops.rs`, `src/sfx.rs`, `archiver/tar.rs`, `stream.rs`, `Password`, `walk.rs`, `add_dialog.rs`, `archiver/zip.rs`, `UpdateMode`, `round_trip.rs`, `src/cli.rs`, `path`, `milestone2.rs`, `UpdateMode`?**
  _High betweenness centrality (0.064) - this node is a cross-community bridge._
- **Should `ops.rs` be split into smaller, more focused modules?**
  _Cohesion score 0.05087719298245614 - nodes in this community are weakly interconnected._