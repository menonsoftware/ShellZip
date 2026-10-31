# Graph Report - ShellZip  (2026-10-31)

## Corpus Check
- 92 files · ~80,433 words
- Verdict: corpus is large enough that graph structure adds value.
- Unclassified: 26 file(s) not represented in the graph (top: .mdc 11, .rar 8, (none) 4)

## Summary
- 1915 nodes · 4671 edges · 84 communities (71 shown, 13 thin omitted)
- Extraction: 97% EXTRACTED · 3% INFERRED · 0% AMBIGUOUS · INFERRED: 139 edges (avg confidence: 0.85)
- Token cost: 0 input · 0 output

## Graph Freshness
- Built from commit: `10ce661d`
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
- ProgressSink
- Format
- round_trip.rs
- list.rs
- extractor/tar.rs
- Shared
- Error
- extract.rs
- Scaffold templates
- explorer/mod.rs
- src/cli.rs
- theme.rs
- Command
- xtask/src/main.rs
- shellzip-gui/src/main.rs
- shellzip-core
- model.rs
- archiver/mod.rs
- app.rs
- Test patterns
- 4. Phased implementation (Cursor task list)
- CompressOpts
- Tier 1 — What can be proven (direct evidence)
- Rust review checklist
- archiver/tar.rs
- extractor/zip.rs
- Rust performance
- shell_icons/mod.rs
- extract_dialog.rs
- Rust quality gate
- run
- path
- MenuCommand
- prompt.rs
- tree.rs
- listing.rs
- extractor/sevenz/mod.rs
- Icon
- BENCH.md
- view.rs
- setup_template
- ToolCommand
- tests/cli.rs
- CrcWriter
- .on_menu
- milestone2.rs
- HWND
- SetupState
- BarProgress
- copy_chunked
- prepare_window
- sevenz/entry.rs
- EntryMeta
- milestone3.rs
- DrawItem
- .retry_with_password
- add_dialog.rs
- EntryKind
- sevenz.rs
- copy_sevenz_index
- StoreList
- shellzip-sfx/build.rs
- ProgressSink
- write
- Entry
- Shared
- CompressOpts
- chrome.rs
- super

## God Nodes (most connected - your core abstractions)
1. `ShellZipApp` - 78 edges
2. `Password` - 66 edges
3. `CompressOpts` - 65 edges
4. `ProgressSink` - 64 edges
5. `Entry` - 43 edges
6. `EntryMeta` - 38 edges
7. `Format` - 36 edges
8. `FolderTree` - 31 edges
9. `ExtractOpts` - 30 edges
10. `Icon` - 29 edges

## Surprising Connections (you probably didn't know these)
- `zip_comment()` --calls--> `detect_file()`  [INFERRED]
  crates/shellzip-core/src/ops.rs → crates/shellzip-core/src/detect.rs
- `compress_opts()` --calls--> `format_from_extension()`  [INFERRED]
  crates/shellzip-cli/src/cli.rs → crates/shellzip-core/src/detect.rs
- `flags_text()` --references--> `Entry`  [EXTRACTED]
  crates/shellzip-cli/src/commands/list.rs → crates/shellzip-core/src/entry.rs
- `xtask` --depends_on--> `shellzip-core`  [EXTRACTED]
  xtask/Cargo.toml → crates/shellzip-core/Cargo.toml
- `create_from_path()` --calls--> `format_from_extension()`  [INFERRED]
  crates/shellzip-core/src/archiver/mod.rs → crates/shellzip-core/src/detect.rs

## Import Cycles
- 1-file cycle: `crates/shellzip-core/src/archiver/parallel.rs -> crates/shellzip-core/src/archiver/parallel.rs`
- 1-file cycle: `crates/shellzip-sfx/src/setup.rs -> crates/shellzip-sfx/src/setup.rs`
- 1-file cycle: `crates/shellzip-gui/src/add_dialog.rs -> crates/shellzip-gui/src/add_dialog.rs`
- 1-file cycle: `crates/shellzip-gui/src/explorer/model.rs -> crates/shellzip-gui/src/explorer/model.rs`
- 1-file cycle: `crates/shellzip-gui/src/chrome.rs -> crates/shellzip-gui/src/chrome.rs`
- 1-file cycle: `crates/shellzip-gui/src/explorer/listing.rs -> crates/shellzip-gui/src/explorer/listing.rs`
- 1-file cycle: `crates/shellzip-gui/src/icons.rs -> crates/shellzip-gui/src/icons.rs`
- 1-file cycle: `crates/shellzip-gui/src/shell_icons/platform.rs -> crates/shellzip-gui/src/shell_icons/platform.rs`
- 2-file cycle: `crates/shellzip-core/src/archiver/mod.rs -> crates/shellzip-core/src/archiver/parallel.rs -> crates/shellzip-core/src/archiver/mod.rs`
- 2-file cycle: `crates/shellzip-cli/src/commands/mod.rs -> crates/shellzip-cli/src/commands/sfx.rs -> crates/shellzip-cli/src/commands/mod.rs`
- 2-file cycle: `crates/shellzip-cli/src/commands/delete.rs -> crates/shellzip-cli/src/commands/mod.rs -> crates/shellzip-cli/src/commands/delete.rs`
- 2-file cycle: `crates/shellzip-cli/src/commands/mod.rs -> crates/shellzip-cli/src/commands/print.rs -> crates/shellzip-cli/src/commands/mod.rs`
- 3-file cycle: `crates/shellzip-core/src/archiver/mod.rs -> crates/shellzip-core/src/archiver/parallel.rs -> crates/shellzip-core/src/archiver/zip.rs -> crates/shellzip-core/src/archiver/mod.rs`
- 3-file cycle: `crates/shellzip-cli/src/cli.rs -> crates/shellzip-cli/src/commands/mod.rs -> crates/shellzip-cli/src/commands/sfx.rs -> crates/shellzip-cli/src/cli.rs`
- 3-file cycle: `crates/shellzip-cli/src/cli.rs -> crates/shellzip-cli/src/commands/mod.rs -> crates/shellzip-cli/src/commands/list.rs -> crates/shellzip-cli/src/cli.rs`
- 3-file cycle: `crates/shellzip-cli/src/cli.rs -> crates/shellzip-cli/src/commands/mod.rs -> crates/shellzip-cli/src/commands/delete.rs -> crates/shellzip-cli/src/cli.rs`
- 3-file cycle: `crates/shellzip-cli/src/cli.rs -> crates/shellzip-cli/src/commands/mod.rs -> crates/shellzip-cli/src/commands/print.rs -> crates/shellzip-cli/src/cli.rs`

## Communities (84 total, 13 thin omitted)

### Community 0 - "worker.rs"
Cohesion: 0.11
Nodes (10): lock(), Outcome, Directory, Extracted, Failed, Finished, Listed, NeedPassword (+2 more)

### Community 1 - "Icon"
Cohesion: 0.13
Nodes (18): ALL, Caret, caret(), Down, Right, Up, MASTER_POINTS, paint() (+10 more)

### Community 2 - "ops.rs"
Cohesion: 0.10
Nodes (28): member_key(), to_archive_path(), add_stdin(), build_globs(), Counting, Counting<'_, R>, delete(), extract() (+20 more)

### Community 3 - "policy.rs"
Cohesion: 0.12
Nodes (32): admit_existing(), archive_is_newer(), Destination, destination(), Skip, Write, disk_mtime(), exists() (+24 more)

### Community 4 - "src/sfx.rs"
Cohesion: 0.07
Nodes (42): SfxArgs, default_output(), existing(), find_stub(), has_trailer(), sfx(), AES_METHOD, decoder_rejected() (+34 more)

### Community 5 - "Password"
Cohesion: 0.12
Nodes (21): Password, stage_path(), archive_at(), copy_rar_index(), copy_staged(), dos_time(), entry_from(), extract_rar() (+13 more)

### Community 6 - "stream.rs"
Cohesion: 0.12
Nodes (8): Kind, Gzip, Zstd, SingleArchiver, Stream, Gzip, Idle, Zstd

### Community 7 - "shellzip-sfx/src/main.rs"
Cohesion: 0.07
Nodes (20): Ask, Bars, byte_style(), sink(), ConflictChoice, Overwrite, Rename, Skip (+12 more)

### Community 8 - "walk.rs"
Cohesion: 0.19
Nodes (13): capture(), archive_rel(), blocked(), build_set(), canonicalish(), collapse_duplicates(), Filters, hits() (+5 more)

### Community 9 - "setup.rs"
Cohesion: 0.06
Nodes (11): CB_ADDSTRING, ID_BROWSE, ID_DEST, ID_INTRO, ID_PROGRESS, INTRO, PBM_SETPOS, PBM_SETRANGE32 (+3 more)

### Community 10 - "archiver/zip.rs"
Cohesion: 0.17
Nodes (7): apply_comment(), method_for(), options_for(), rel_name(), target_text(), ZipArchiver, dos_parts()

### Community 11 - "ShellZipApp"
Cohesion: 0.11
Nodes (5): Place, Archive, Folder, ShellZipApp, is_supported_archive()

### Community 12 - "ProgressSink"
Cohesion: 0.13
Nodes (4): NopProgress, ProgressReader, ProgressReader<'a, R>, ProgressReader<'_, R>

### Community 13 - "Format"
Cohesion: 0.10
Nodes (26): creatable_extensions_round_trip(), creatable_formats(), CreatableFormat, detect_bytes(), detect_file(), detect_header(), find_embedded(), find_subslice() (+18 more)

### Community 14 - "round_trip.rs"
Cohesion: 0.26
Nodes (16): adding_to_an_existing_zip_keeps_the_other_files(), byte_cap_stops_extraction(), existing_tar_is_left_in_place_without_force(), extract_all(), gzip_rejects_a_second_file(), opts_fast(), overwrite_skip_and_rename(), pack_tree() (+8 more)

### Community 15 - "list.rs"
Cohesion: 0.21
Nodes (8): ArchiveArgs, ListArgs, display_name(), flags_text(), list(), list_technical(), test(), format_blake()

### Community 16 - "extractor/tar.rs"
Cohesion: 0.12
Nodes (27): Compression, Bzip2, Gzip, None, Xz, Zstd, prefix_is_tar(), Entry (+19 more)

### Community 17 - "Shared"
Cohesion: 0.12
Nodes (15): AddArgs, compress_opts(), MethodArg, Bzip2, Deflate, Lzma, Lzma2, Ppmd (+7 more)

### Community 18 - "Error"
Cohesion: 0.19
Nodes (12): Error, Archive, BadChecksum, BadPassword, Encrypted, Interrupted, Io, NoFiles (+4 more)

### Community 19 - "extract.rs"
Cohesion: 0.22
Nodes (9): ExtractArgs, Overwrite, overwrite_mode(), OverwriteArg, Always, Ask, Rename, Skip (+1 more)

### Community 20 - "Scaffold templates"
Cohesion: 0.10
Nodes (19): Criterion benchmark, Example, Extra binary in an existing crate, Finish checklist, Integration test file, New module in an existing crate, New `szip` subcommand, New workspace crate (+11 more)

### Community 21 - "explorer/mod.rs"
Cohesion: 0.20
Nodes (9): archive_prefix(), folder_chain(), folder_parent(), jumps(), Folder, Prefix, location_text(), normalize_sep() (+1 more)

### Community 22 - "src/cli.rs"
Cohesion: 0.14
Nodes (8): Cli, dispatch(), init_tracing(), normalize_args(), password_from_env(), prompt_password(), run(), take_password()

### Community 23 - "theme.rs"
Cohesion: 0.18
Nodes (16): ADD, apply(), CANVAS, DANGER, EXTRACT, FOLDER, HOVER, INK (+8 more)

### Community 24 - "Command"
Cohesion: 0.14
Nodes (13): Command, Add, Complete, Delete, Extract, ExtractFlat, Freshen, List (+5 more)

### Community 25 - "xtask/src/main.rs"
Cohesion: 0.17
Nodes (22): bench(), build_fixtures(), collect_files(), crc32(), find_rar(), find_tool(), FIXTURE_PASSWORD, main() (+14 more)

### Community 27 - "shellzip-core"
Cohesion: 0.40
Nodes (5): shellzip-cli, shellzip-core, shellzip-gui, shellzip-sfx, xtask

### Community 29 - "model.rs"
Cohesion: 0.08
Nodes (39): format_bytes(), format_time(), ArchiveIndex, byte_phrase(), compare_column(), compare_name(), compare_option(), dir_partial() (+31 more)

### Community 30 - "archiver/mod.rs"
Cohesion: 0.17
Nodes (7): ArchiveWriter, copy_in(), create(), create_from_path(), prepare_output(), prepare_temp(), temp_beside()

### Community 31 - "app.rs"
Cohesion: 0.12
Nodes (14): decide_member(), folder_title(), is_sfx_name(), PasswordAsk, PendingArchiveDelete, spawn_directory(), starting_folder(), ExtractJob (+6 more)

### Community 32 - "Test patterns"
Cohesion: 0.11
Nodes (17): CLI end-to-end with `assert_cmd` (`tests/cli.rs`), Counting fake for a trait, In-memory reader/writer, Integration round-trip (`tests/<scenario>.rs`), Interop with OS tools, Platform-gated test, Property test with `proptest`, Snapshot test with `insta` (+9 more)

### Community 33 - "4. Phased implementation (Cursor task list)"
Cohesion: 0.12
Nodes (15): 1. Scope & feature‑parity matrix, 2. Recommended crate ecosystem, 3. Project structure, 4. Phased implementation (Cursor task list), 5. Testing & verification strategy, 6. Licensing notes, Design principles carried over from WinRAR, Milestone 0 — Skeleton (½ day) (+7 more)

### Community 34 - "CompressOpts"
Cohesion: 0.18
Nodes (10): add_zip(), compress_file(), MEMORY_LIMIT, merge(), Piece, Memory, piece_path(), Temp (+2 more)

### Community 35 - "Tier 1 — What can be proven (direct evidence)"
Cohesion: 0.20
Nodes (9): 1.1 Directory inventory (32 files, all in one flat directory), 1.2 Separation of concerns, 1.3 Imported Windows APIs grouped by capability, 1.4 CLI surface (from `Rar.exe -?`, `UnRAR.exe`, `Rar.txt`), Evidence ledger (summary), Tier 1 — What can be proven (direct evidence), Tier 2 — What can be inferred (architectural deductions), Tier 3 — What remains hidden (opaque internals) (+1 more)

### Community 36 - "Rust review checklist"
Cohesion: 0.13
Nodes (13): 1. Correctness, 2. Error handling, 3. Security, 4. Ownership and performance, 5. Concurrency, 6. Tests, 7. API and docs, 8. Manifest and tooling (+5 more)

### Community 37 - "archiver/tar.rs"
Cohesion: 0.11
Nodes (9): commit_output(), CompWriter, Gzip, Plain, Zstd, dir_name(), header_for(), TarArchiver (+1 more)

### Community 38 - "extractor/zip.rs"
Cohesion: 0.15
Nodes (11): annotate(), entry_from(), extract_zip(), extract_zip_index(), host_label(), list_zip(), method_name(), open_zip() (+3 more)

### Community 39 - "Rust performance"
Cohesion: 0.29
Nodes (6): 1. Baseline, 2. Find the hot spot, 3. Apply the cheapest fix the profile justifies, 4. Binary size, 5. Verify and report, Rust performance

### Community 40 - "shell_icons/mod.rs"
Cohesion: 0.07
Nodes (49): caches_a_raster_from_the_provider(), decode_png(), extension_of(), icon_keys_follow_extension_path_and_folder(), IconCache, IconKey, Computer, Extension (+41 more)

### Community 41 - "extract_dialog.rs"
Cohesion: 0.06
Nodes (67): ExtractUpdate, Freshen, Replace, Update, Overwrite, Always, Ask, Never (+59 more)

### Community 42 - "Rust quality gate"
Cohesion: 0.29
Nodes (6): Fix clippy, don't silence it, Fix tests, Optional supply-chain checks, Report, Rust quality gate, Workflow

### Community 43 - "run"
Cohesion: 0.25
Nodes (7): ComInit, dword_words(), finish_worker(), init_common_controls(), run(), show(), windows_accepts_the_setup_dialog_template()

### Community 44 - "path"
Cohesion: 0.12
Nodes (4): copy_zone(), MAX_ZONE_BYTES, propagate(), zone_path()

### Community 45 - "MenuCommand"
Cohesion: 0.17
Nodes (12): menu_bar(), MenuCommand, About, Add, Delete, Exit, Extract, Info (+4 more)

### Community 46 - "prompt.rs"
Cohesion: 0.07
Nodes (49): accept_password(), ask(), ask_console(), ask_gui(), attach_console(), attach_parent(), Choice, All (+41 more)

### Community 47 - "tree.rs"
Cohesion: 0.06
Nodes (67): tree_row(), JumpTarget, archive_prefixes_hang_under_the_archive(), best_volume(), Builder, cmp_name(), component_len(), computer_label() (+59 more)

### Community 48 - "listing.rs"
Cohesion: 0.36
Nodes (6): lists_a_folder_and_a_text_file(), read_directory(), read_subdirs(), row_for(), subdirs_skip_files(), TempDir

### Community 49 - "extractor/sevenz/mod.rs"
Cohesion: 0.18
Nodes (15): Budget, mark_extracted(), restore_entry_time(), finish_staged(), Member, extract_sevenz(), ExtractOne, test_sevenz() (+7 more)

### Community 50 - "Icon"
Cohesion: 0.10
Nodes (19): directories_use_the_yellow_folder(), row_glyph(), selected_glyphs_use_the_selection_ink(), Icon, Archive, ArrowUp, CircleCheck, File (+11 more)

### Community 52 - "view.rs"
Cohesion: 0.09
Nodes (19): format_crc(), SortColumn, Crc, Modified, Name, Packed, Size, Type (+11 more)

### Community 54 - "setup_template"
Cohesion: 0.47
Nodes (9): align4(), Control, push_atom(), push_named(), push_u16(), push_u32(), push_wide(), setup_dialog_has_browse_and_a_progress_bar() (+1 more)

### Community 55 - "ToolCommand"
Cohesion: 0.33
Nodes (6): ToolCommand, Add, Delete, Extract, Info, Test

### Community 56 - "tests/cli.rs"
Cohesion: 0.16
Nodes (21): add_list_test_extract_round_trip(), add_sfx_writes_a_listable_package(), add_to_an_existing_zip_keeps_previous_files(), help_explains_that_zip_encryption_leaves_names_visible(), log_file_receives_debug_events(), rar_is_not_created(), reads_a_tar_from_the_system_tool(), reads_a_zip_from_compress_archive() (+13 more)

### Community 57 - "CrcWriter"
Cohesion: 0.14
Nodes (4): Tee, blake2sp_matches_the_reference_vector(), Blake2spWriter, CrcWriter

### Community 60 - "milestone2.rs"
Cohesion: 0.24
Nodes (11): ask_can_skip_replace_or_rename(), freshen_missing_archive_is_not_found(), opts(), parallel_zip_matches_single_thread_bytes(), skip_leaves_existing_members_and_adds_new_ones(), stdin_streams_zst_and_spools_tar(), sync_drops_members_that_were_not_selected(), technical_list_fills_blake2sp() (+3 more)

### Community 61 - "HWND"
Cohesion: 0.27
Nodes (16): enable_window(), EnableWindow(), finish_if_ready(), on_browse(), on_command(), on_extract(), paint_progress(), prepare_folder() (+8 more)

### Community 62 - "SetupState"
Cohesion: 0.19
Nodes (8): browse_initialized(), destroy_icon(), install_icons(), load_icon(), paint_banner(), percent(), rgb(), SetupState

### Community 65 - "prepare_window"
Cohesion: 0.38
Nodes (7): add_choice(), browse_folder(), prepare_window(), set_text(), set_window_text(), wide_os(), wide_str()

### Community 66 - "sevenz/entry.rs"
Cohesion: 0.09
Nodes (18): coder_dictionary(), coder_label(), entry_from(), entry_key(), file(), folder_dictionary(), folder_label(), host_os() (+10 more)

### Community 67 - "EntryMeta"
Cohesion: 0.13
Nodes (11): archive_path(), CloneMeta, EntryMeta, seven_meta(), SevenZArchiver, spool(), Spooled, write_file_body() (+3 more)

### Community 68 - "milestone3.rs"
Cohesion: 0.07
Nodes (33): bench_zip_threads(), assert_tree(), Case, cases(), extract_all(), nested_folder_round_trips_every_writable_format(), opts(), parallel_zip_round_trips_a_folder_of_spilled_files() (+25 more)

### Community 70 - ".retry_with_password"
Cohesion: 0.36
Nodes (3): find_stub(), list_archive(), needs_password()

### Community 71 - "add_dialog.rs"
Cohesion: 0.06
Nodes (55): SevenZOpts, UpdateMode, Add, .ALL, Ask, Freshen, Skip, Sync (+47 more)

### Community 72 - "EntryKind"
Cohesion: 0.25
Nodes (7): EntryKind, Dir, File, Other, Symlink, format_attrs(), mode_bits()

### Community 73 - "sevenz.rs"
Cohesion: 0.14
Nodes (18): codec_method(), write_options(), add_lists_and_extracts_through_the_core(), aes_hides_bytes_and_header_encryption_hides_names(), bzip2_and_deflate_round_trip(), committed_sevenz_fixtures_list_when_present(), extract_bytes(), fast() (+10 more)

### Community 74 - "copy_sevenz_index"
Cohesion: 0.32
Nodes (3): copy_sevenz_index(), ProgressWrite<'_, W>, Tee

### Community 80 - "ProgressSink"
Cohesion: 0.12
Nodes (16): ProgressWrite, add(), add_prompted(), add_sfx(), add_stdin(), delete(), extract(), list_with() (+8 more)

### Community 84 - "Entry"
Cohesion: 0.14
Nodes (9): ArchiveReader, only_first(), Opened, resolve(), unsupported(), Verify, list(), open() (+1 more)

### Community 85 - "Shared"
Cohesion: 0.33
Nodes (5): DeleteArgs, PrintArgs, delete(), Shared, print_entry()

### Community 86 - "CompressOpts"
Cohesion: 0.14
Nodes (10): add(), add_inner(), add_prompted(), add_sfx(), scratch_archive(), write_sequential(), CompressOpts, SfxPayload (+2 more)

### Community 89 - "chrome.rs"
Cohesion: 0.12
Nodes (22): about(), address_bar(), AddressEvent, Jump, Submit, Up, bar_frame(), breadcrumbs() (+14 more)

## Knowledge Gaps
- **190 isolated node(s):** `shellzip-cli`, `Add`, `Update`, `Freshen`, `Delete` (+185 more)
  These have ≤1 connection - possible missing edges or undocumented components. (Counts symbols only; 557 node(s) total have ≤1 connection when file, concept and rationale nodes are included.)
- **13 thin communities (<3 nodes) omitted from report** — run `graphify query` to explore isolated nodes.

## Suggested Questions
_Questions this graph is uniquely positioned to answer:_

- **Why does `Error` connect `Error` to `worker.rs`, `ops.rs`, `policy.rs`, `src/sfx.rs`, `Password`, `stream.rs`, `shellzip-sfx/src/main.rs`, `walk.rs`, `setup.rs`, `archiver/zip.rs`, `Format`, `round_trip.rs`, `list.rs`, `extractor/tar.rs`, `Shared`, `extract.rs`, `src/cli.rs`, `archiver/mod.rs`, `app.rs`, `CompressOpts`, `archiver/tar.rs`, `extractor/zip.rs`, `path`, `prompt.rs`, `extractor/sevenz/mod.rs`, `EntryMeta`, `milestone3.rs`, `sevenz.rs`, `ProgressSink`, `write`, `Entry`, `Shared`?**
  _High betweenness centrality (0.104) - this node is a cross-community bridge._
- **What connects `shellzip-cli`, `Add`, `Update` to the rest of the system?**
  _190 weakly-connected nodes found - possible documentation gaps or missing edges._
- **Should `worker.rs` be split into smaller, more focused modules?**
  _Cohesion score 0.11396011396011396 - nodes in this community are weakly interconnected._
- **Why does `Password` connect `Password` to `ops.rs`, `src/sfx.rs`, `shellzip-sfx/src/main.rs`, `setup.rs`, `ShellZipApp`, `list.rs`, `src/cli.rs`, `theme.rs`, `app.rs`, `extractor/zip.rs`, `run`, `extractor/sevenz/mod.rs`, `SetupState`, `sevenz/entry.rs`, `.retry_with_password`, `copy_sevenz_index`, `ProgressSink`, `Entry`, `Shared`, `CompressOpts`, `super`?**
  _High betweenness centrality (0.059) - this node is a cross-community bridge._
- **Should `Icon` be split into smaller, more focused modules?**
  _Cohesion score 0.12807881773399016 - nodes in this community are weakly interconnected._
- **Why does `CompressOpts` connect `CompressOpts` to `ops.rs`, `Password`, `stream.rs`, `walk.rs`, `archiver/zip.rs`, `round_trip.rs`, `Shared`, `src/cli.rs`, `archiver/mod.rs`, `CompressOpts`, `archiver/tar.rs`, `milestone2.rs`, `EntryMeta`, `milestone3.rs`, `add_dialog.rs`, `sevenz.rs`, `StoreList`, `ProgressSink`, `write`?**
  _High betweenness centrality (0.058) - this node is a cross-community bridge._
- **Should `ops.rs` be split into smaller, more focused modules?**
  _Cohesion score 0.1036036036036036 - nodes in this community are weakly interconnected._