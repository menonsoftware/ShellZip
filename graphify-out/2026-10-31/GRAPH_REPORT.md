# Graph Report - ShellZip  (2026-10-31)

## Corpus Check
- 92 files · ~81,310 words
- Verdict: corpus is large enough that graph structure adds value.
- Unclassified: 26 file(s) not represented in the graph (top: .mdc 11, .rar 8, (none) 4)

## Summary
- 1934 nodes · 4718 edges · 84 communities (73 shown, 11 thin omitted)
- Extraction: 97% EXTRACTED · 3% INFERRED · 0% AMBIGUOUS · INFERRED: 139 edges (avg confidence: 0.85)
- Token cost: 0 input · 0 output

## Graph Freshness
- Built from commit: `cbbb1cd9`
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
- app.rs
- src/cli.rs
- prepare_output
- Command
- xtask/src/main.rs
- shellzip-gui/src/main.rs
- shellzip-core
- model.rs
- ArchiveWriter
- Overwrite
- Test patterns
- 4. Phased implementation (Cursor task list)
- CompressOpts
- Tier 1 — What can be proven (direct evidence)
- Rust review checklist
- archiver/tar.rs
- path
- Rust performance
- shell_icons/mod.rs
- settings.rs
- Rust quality gate
- setup_template
- MenuCommand
- prompt.rs
- tree.rs
- listing.rs
- extractor/sevenz/mod.rs
- BENCH.md
- view.rs
- ToolCommand
- tests/cli.rs
- CrcWriter
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
- Password
- add_dialog.rs
- extract_dialog.rs
- sevenz.rs
- copy_sevenz_index
- Overwrite
- folder_formats.rs
- ProgressSink
- tests/extract.rs
- zip_threads.rs
- Entry
- SevenZMethod
- CompressOpts
- RowKind
- chrome.rs
- MethodArg
- stream_one
- Tab

## God Nodes (most connected - your core abstractions)
1. `ShellZipApp` - 78 edges
2. `Password` - 66 edges
3. `CompressOpts` - 65 edges
4. `ProgressSink` - 64 edges
5. `Entry` - 43 edges
6. `EntryMeta` - 38 edges
7. `Format` - 36 edges
8. `Icon` - 35 edges
9. `FolderTree` - 31 edges
10. `ExtractOpts` - 30 edges

## Surprising Connections (you probably didn't know these)
- `compress_opts()` --calls--> `format_from_extension()`  [INFERRED]
  crates/shellzip-cli/src/cli.rs → crates/shellzip-core/src/detect.rs
- `flags_text()` --references--> `Entry`  [EXTRACTED]
  crates/shellzip-cli/src/commands/list.rs → crates/shellzip-core/src/entry.rs
- `xtask` --depends_on--> `shellzip-core`  [EXTRACTED]
  xtask/Cargo.toml → crates/shellzip-core/Cargo.toml
- `create_from_path()` --calls--> `format_from_extension()`  [INFERRED]
  crates/shellzip-core/src/archiver/mod.rs → crates/shellzip-core/src/detect.rs
- `compress_file()` --calls--> `method_for()`  [INFERRED]
  crates/shellzip-core/src/archiver/parallel.rs → crates/shellzip-core/src/archiver/zip.rs

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

## Communities (84 total, 11 thin omitted)

### Community 0 - "worker.rs"
Cohesion: 0.11
Nodes (12): PasswordAsk, lock(), Outcome, Directory, Extracted, Failed, Finished, Listed (+4 more)

### Community 1 - "Icon"
Cohesion: 0.06
Nodes (39): ALL, Caret, caret(), Down, Right, Up, Icon, Archive (+31 more)

### Community 2 - "ops.rs"
Cohesion: 0.12
Nodes (26): member_key(), add_inner(), add_prompted(), add_sfx(), build_globs(), Counting, delete(), extract() (+18 more)

### Community 3 - "policy.rs"
Cohesion: 0.12
Nodes (32): admit_existing(), archive_is_newer(), Destination, destination(), Skip, Write, disk_mtime(), exists() (+24 more)

### Community 4 - "src/sfx.rs"
Cohesion: 0.06
Nodes (45): SfxArgs, default_output(), existing(), find_stub(), has_trailer(), sfx(), AES_METHOD, decoder_rejected() (+37 more)

### Community 5 - "Password"
Cohesion: 0.16
Nodes (18): archive_at(), copy_rar_index(), copy_staged(), dos_time(), entry_from(), extract_rar(), first_volume(), hash_file() (+10 more)

### Community 6 - "stream.rs"
Cohesion: 0.11
Nodes (9): commit_output(), Kind, Gzip, Zstd, SingleArchiver, Stream, Gzip, Idle (+1 more)

### Community 7 - "shellzip-sfx/src/main.rs"
Cohesion: 0.07
Nodes (20): Ask, Bars, byte_style(), sink(), ConflictChoice, Overwrite, Rename, Skip (+12 more)

### Community 8 - "walk.rs"
Cohesion: 0.13
Nodes (21): EntryKind, Dir, File, Other, Symlink, format_attrs(), to_archive_path(), capture() (+13 more)

### Community 9 - "setup.rs"
Cohesion: 0.06
Nodes (11): CB_ADDSTRING, ID_BROWSE, ID_DEST, ID_INTRO, ID_PROGRESS, INTRO, PBM_SETPOS, PBM_SETRANGE32 (+3 more)

### Community 10 - "archiver/zip.rs"
Cohesion: 0.17
Nodes (7): apply_comment(), method_for(), options_for(), rel_name(), target_text(), ZipArchiver, dos_parts()

### Community 11 - "ShellZipApp"
Cohesion: 0.07
Nodes (9): decide_member(), list_archive(), needs_password(), Place, Archive, Folder, ShellZipApp, spawn_directory() (+1 more)

### Community 13 - "Format"
Cohesion: 0.10
Nodes (25): creatable_extensions_round_trip(), detect_bytes(), detect_file(), detect_header(), find_embedded(), find_subslice(), Format, Bzip2 (+17 more)

### Community 14 - "round_trip.rs"
Cohesion: 0.26
Nodes (16): adding_to_an_existing_zip_keeps_the_other_files(), byte_cap_stops_extraction(), existing_tar_is_left_in_place_without_force(), extract_all(), gzip_rejects_a_second_file(), opts_fast(), overwrite_skip_and_rename(), pack_tree() (+8 more)

### Community 15 - "list.rs"
Cohesion: 0.21
Nodes (8): ArchiveArgs, ListArgs, display_name(), flags_text(), list(), list_technical(), test(), format_blake()

### Community 16 - "extractor/tar.rs"
Cohesion: 0.11
Nodes (28): Compression, Bzip2, Gzip, None, Xz, Zstd, prefix_is_tar(), Entry (+20 more)

### Community 17 - "Shared"
Cohesion: 0.20
Nodes (10): AddArgs, compress_opts(), DeleteArgs, SfxFormatArg, SevenZ, Zip, add(), pack_sfx() (+2 more)

### Community 18 - "Error"
Cohesion: 0.12
Nodes (15): PrintArgs, print_entry(), tune(), Error, Archive, BadChecksum, BadPassword, Encrypted (+7 more)

### Community 19 - "extract.rs"
Cohesion: 0.22
Nodes (9): ExtractArgs, Overwrite, overwrite_mode(), OverwriteArg, Always, Ask, Rename, Skip (+1 more)

### Community 20 - "Scaffold templates"
Cohesion: 0.10
Nodes (19): Criterion benchmark, Example, Extra binary in an existing crate, Finish checklist, Integration test file, New module in an existing crate, New `szip` subcommand, New workspace crate (+11 more)

### Community 21 - "app.rs"
Cohesion: 0.12
Nodes (21): find_stub(), folder_title(), is_sfx_name(), PendingArchiveDelete, starting_folder(), AddressEvent, Jump, Submit (+13 more)

### Community 22 - "src/cli.rs"
Cohesion: 0.14
Nodes (8): Cli, dispatch(), init_tracing(), normalize_args(), password_from_env(), prompt_password(), run(), take_password()

### Community 23 - "prepare_output"
Cohesion: 0.48
Nodes (3): prepare_output(), prepare_temp(), temp_beside()

### Community 24 - "Command"
Cohesion: 0.14
Nodes (13): Command, Add, Complete, Delete, Extract, ExtractFlat, Freshen, List (+5 more)

### Community 25 - "xtask/src/main.rs"
Cohesion: 0.15
Nodes (23): bench(), build_fixtures(), build_sevenz_fixtures(), collect_files(), crc32(), find_rar(), find_tool(), FIXTURE_PASSWORD (+15 more)

### Community 27 - "shellzip-core"
Cohesion: 0.40
Nodes (5): shellzip-cli, shellzip-core, shellzip-gui, shellzip-sfx, xtask

### Community 29 - "model.rs"
Cohesion: 0.08
Nodes (35): format_bytes(), format_time(), ArchiveIndex, byte_phrase(), compare_column(), compare_name(), compare_option(), dir_partial() (+27 more)

### Community 31 - "Overwrite"
Cohesion: 0.33
Nodes (6): Overwrite, Always, Ask, Never, Rename, overwrite_name()

### Community 32 - "Test patterns"
Cohesion: 0.11
Nodes (17): CLI end-to-end with `assert_cmd` (`tests/cli.rs`), Counting fake for a trait, In-memory reader/writer, Integration round-trip (`tests/<scenario>.rs`), Interop with OS tools, Platform-gated test, Property test with `proptest`, Snapshot test with `insta` (+9 more)

### Community 33 - "4. Phased implementation (Cursor task list)"
Cohesion: 0.12
Nodes (15): 1. Scope & feature‑parity matrix, 2. Recommended crate ecosystem, 3. Project structure, 4. Phased implementation (Cursor task list), 5. Testing & verification strategy, 6. Licensing notes, Design principles carried over from WinRAR, Milestone 0 — Skeleton (½ day) (+7 more)

### Community 34 - "CompressOpts"
Cohesion: 0.16
Nodes (10): copy_in(), compress_file(), MEMORY_LIMIT, merge(), Piece, Memory, piece_path(), Temp (+2 more)

### Community 35 - "Tier 1 — What can be proven (direct evidence)"
Cohesion: 0.20
Nodes (9): 1.1 Directory inventory (32 files, all in one flat directory), 1.2 Separation of concerns, 1.3 Imported Windows APIs grouped by capability, 1.4 CLI surface (from `Rar.exe -?`, `UnRAR.exe`, `Rar.txt`), Evidence ledger (summary), Tier 1 — What can be proven (direct evidence), Tier 2 — What can be inferred (architectural deductions), Tier 3 — What remains hidden (opaque internals) (+1 more)

### Community 36 - "Rust review checklist"
Cohesion: 0.13
Nodes (13): 1. Correctness, 2. Error handling, 3. Security, 4. Ownership and performance, 5. Concurrency, 6. Tests, 7. API and docs, 8. Manifest and tooling (+5 more)

### Community 37 - "archiver/tar.rs"
Cohesion: 0.12
Nodes (8): CompWriter, Gzip, Plain, Zstd, dir_name(), header_for(), TarArchiver, unix_secs()

### Community 38 - "path"
Cohesion: 0.09
Nodes (8): copy_zone(), MAX_ZONE_BYTES, propagate(), zone_path(), DEFAULT_STORE_EXTS, MAX_THREADS, main(), windows_sdk_bin()

### Community 39 - "Rust performance"
Cohesion: 0.29
Nodes (6): 1. Baseline, 2. Find the hot spot, 3. Apply the cheapest fix the profile justifies, 4. Binary size, 5. Verify and report, Rust performance

### Community 40 - "shell_icons/mod.rs"
Cohesion: 0.06
Nodes (49): caches_a_raster_from_the_provider(), decode_png(), extension_of(), icon_keys_follow_extension_path_and_folder(), IconCache, IconKey, Computer, Extension (+41 more)

### Community 41 - "settings.rs"
Cohesion: 0.15
Nodes (25): ZipMethod, Deflate, Zstd, PackSettings, defaults(), delete_name(), DeleteArchive, Always (+17 more)

### Community 42 - "Rust quality gate"
Cohesion: 0.29
Nodes (6): Fix clippy, don't silence it, Fix tests, Optional supply-chain checks, Report, Rust quality gate, Workflow

### Community 43 - "setup_template"
Cohesion: 0.18
Nodes (16): align4(), ComInit, Control, dword_words(), finish_worker(), init_common_controls(), push_atom(), push_named() (+8 more)

### Community 45 - "MenuCommand"
Cohesion: 0.17
Nodes (12): menu_bar(), MenuCommand, About, Add, Delete, Exit, Extract, Info (+4 more)

### Community 46 - "prompt.rs"
Cohesion: 0.07
Nodes (49): accept_password(), ask(), ask_console(), ask_gui(), attach_console(), attach_parent(), Choice, All (+41 more)

### Community 47 - "tree.rs"
Cohesion: 0.06
Nodes (65): archive_prefixes_hang_under_the_archive(), best_volume(), Builder, cmp_name(), component_len(), computer_label(), drive_root(), drive_root_is_the_filesystem_root() (+57 more)

### Community 48 - "listing.rs"
Cohesion: 0.36
Nodes (6): lists_a_folder_and_a_text_file(), read_directory(), read_subdirs(), row_for(), subdirs_skip_files(), TempDir

### Community 49 - "extractor/sevenz/mod.rs"
Cohesion: 0.14
Nodes (19): Budget, mark_extracted(), restore_entry_time(), symlink_destination(), finish_staged(), Member, extract_sevenz(), ExtractOne (+11 more)

### Community 52 - "view.rs"
Cohesion: 0.09
Nodes (19): format_crc(), SortColumn, Crc, Modified, Name, Packed, Size, Type (+11 more)

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
Cohesion: 0.25
Nodes (16): enable_window(), EnableWindow(), finish_if_ready(), on_command(), on_extract(), paint_progress(), percent(), prepare_folder() (+8 more)

### Community 62 - "SetupState"
Cohesion: 0.21
Nodes (7): browse_initialized(), destroy_icon(), install_icons(), load_icon(), paint_banner(), rgb(), SetupState

### Community 65 - "prepare_window"
Cohesion: 0.36
Nodes (8): add_choice(), browse_folder(), on_browse(), prepare_window(), set_text(), set_window_text(), wide_os(), wide_str()

### Community 66 - "sevenz/entry.rs"
Cohesion: 0.09
Nodes (18): coder_dictionary(), coder_label(), entry_from(), entry_key(), file(), folder_dictionary(), folder_label(), host_os() (+10 more)

### Community 67 - "EntryMeta"
Cohesion: 0.13
Nodes (11): archive_path(), CloneMeta, EntryMeta, seven_meta(), SevenZArchiver, spool(), Spooled, write_file_body() (+3 more)

### Community 68 - "milestone3.rs"
Cohesion: 0.18
Nodes (11): assert_matches_unrar(), encrypting_an_existing_zip_requires_force(), extract_bytes(), find_unrar(), FIXTURE_PASSWORD, fixtures_root(), password(), passwords_are_rejected_for_tar() (+3 more)

### Community 70 - "Password"
Cohesion: 0.11
Nodes (11): debug_hides_the_password(), Password, test_sevenz(), annotate(), extract_zip_index(), list_zip(), open_zip(), test_zip() (+3 more)

### Community 71 - "add_dialog.rs"
Cohesion: 0.07
Nodes (53): creatable_formats(), SevenZOpts, UpdateMode, Add, .ALL, Ask, Freshen, Skip (+45 more)

### Community 72 - "extract_dialog.rs"
Cohesion: 0.12
Nodes (32): advanced_tab(), begin_new_folder(), confirm_delete_archive(), confirm_new_folder(), destination_uses_the_archive_name_without_its_extension(), display_path(), ExtractAction, Close (+24 more)

### Community 73 - "sevenz.rs"
Cohesion: 0.14
Nodes (18): codec_method(), write_options(), add_lists_and_extracts_through_the_core(), aes_hides_bytes_and_header_encryption_hides_names(), bzip2_and_deflate_round_trip(), committed_sevenz_fixtures_list_when_present(), extract_bytes(), fast() (+10 more)

### Community 74 - "copy_sevenz_index"
Cohesion: 0.32
Nodes (3): copy_sevenz_index(), ProgressWrite<'_, W>, Tee

### Community 78 - "Overwrite"
Cohesion: 0.18
Nodes (11): ExtractUpdate, Freshen, Replace, Update, update_name(), ExtractJob, PasswordJob, Delete (+3 more)

### Community 79 - "folder_formats.rs"
Cohesion: 0.35
Nodes (12): assert_tree(), Case, cases(), extract_all(), nested_folder_round_trips_every_writable_format(), opts(), parallel_zip_round_trips_a_folder_of_spilled_files(), planned_names() (+4 more)

### Community 80 - "ProgressSink"
Cohesion: 0.10
Nodes (17): CreatableFormat, ProgressWrite, add(), add_prompted(), add_sfx(), add_stdin(), delete(), extract() (+9 more)

### Community 82 - "tests/extract.rs"
Cohesion: 0.33
Nodes (9): built_stub_contains_the_shellzip_icon(), encrypted_sfx(), silent_encrypted_stub_fails_without_a_password(), silent_encrypted_stub_rejects_a_wrong_password(), stub_bin(), stub_extracts_a_header_encrypted_7z_with_the_password(), stub_extracts_an_encrypted_zip_when_the_password_is_set(), stub_extracts_its_appended_7z() (+1 more)

### Community 84 - "Entry"
Cohesion: 0.14
Nodes (9): ArchiveReader, only_first(), Opened, resolve(), unsupported(), Verify, list(), open() (+1 more)

### Community 85 - "SevenZMethod"
Cohesion: 0.20
Nodes (9): SevenZMethod, Bzip2, Deflate, Lzma, Lzma2, Ppmd, Zstd, update_mode_labels_follow_the_add_dialog() (+1 more)

### Community 86 - "CompressOpts"
Cohesion: 0.14
Nodes (6): create(), create_from_path(), add_zip(), CompressOpts, shrinks_zstd_window_for_small_inputs(), StoreList

### Community 87 - "RowKind"
Cohesion: 0.24
Nodes (9): directories_use_the_yellow_folder(), row_glyph(), selected_glyphs_use_the_selection_ink(), RowKind, Archive, Directory, File, Parent (+1 more)

### Community 89 - "chrome.rs"
Cohesion: 0.09
Nodes (32): about(), address_bar(), bar_frame(), caret_button(), confirm_delete(), dismiss_button(), dropdown_height(), global_rect() (+24 more)

### Community 90 - "MethodArg"
Cohesion: 0.25
Nodes (7): MethodArg, Bzip2, Deflate, Lzma, Lzma2, Ppmd, Zstd

### Community 91 - "stream_one"
Cohesion: 0.36
Nodes (5): add_stdin(), Counting<'_, R>, spool_stdin(), Spooled, stream_one()

### Community 93 - "Tab"
Cohesion: 0.50
Nodes (4): Tab, Advanced, General, Options

## Knowledge Gaps
- **195 isolated node(s):** `shellzip-cli`, `Add`, `Update`, `Freshen`, `Delete` (+190 more)
  These have ≤1 connection - possible missing edges or undocumented components. (Counts symbols only; 566 node(s) total have ≤1 connection when file, concept and rationale nodes are included.)
- **11 thin communities (<3 nodes) omitted from report** — run `graphify query` to explore isolated nodes.

## Suggested Questions
_Questions this graph is uniquely positioned to answer:_

- **Why does `Error` connect `Error` to `worker.rs`, `ops.rs`, `policy.rs`, `src/sfx.rs`, `Password`, `stream.rs`, `shellzip-sfx/src/main.rs`, `walk.rs`, `setup.rs`, `archiver/zip.rs`, `Format`, `round_trip.rs`, `list.rs`, `extractor/tar.rs`, `Shared`, `extract.rs`, `app.rs`, `src/cli.rs`, `CompressOpts`, `archiver/tar.rs`, `path`, `prompt.rs`, `extractor/sevenz/mod.rs`, `EntryMeta`, `milestone3.rs`, `sevenz.rs`, `folder_formats.rs`, `ProgressSink`, `Entry`, `CompressOpts`?**
  _High betweenness centrality (0.103) - this node is a cross-community bridge._
- **What connects `shellzip-cli`, `Add`, `Update` to the rest of the system?**
  _195 weakly-connected nodes found - possible documentation gaps or missing edges._
- **Should `worker.rs` be split into smaller, more focused modules?**
  _Cohesion score 0.10591133004926108 - nodes in this community are weakly interconnected._
- **Why does `Password` connect `Password` to `ops.rs`, `src/sfx.rs`, `Password`, `shellzip-sfx/src/main.rs`, `setup.rs`, `ShellZipApp`, `list.rs`, `Shared`, `app.rs`, `src/cli.rs`, `path`, `setup_template`, `extractor/sevenz/mod.rs`, `SetupState`, `sevenz/entry.rs`, `copy_sevenz_index`, `ProgressSink`, `Entry`, `CompressOpts`, `chrome.rs`?**
  _High betweenness centrality (0.055) - this node is a cross-community bridge._
- **Should `Icon` be split into smaller, more focused modules?**
  _Cohesion score 0.06274509803921569 - nodes in this community are weakly interconnected._
- **Why does `CompressOpts` connect `CompressOpts` to `ops.rs`, `stream.rs`, `walk.rs`, `archiver/zip.rs`, `round_trip.rs`, `Shared`, `Error`, `src/cli.rs`, `prepare_output`, `CompressOpts`, `archiver/tar.rs`, `path`, `settings.rs`, `milestone2.rs`, `EntryMeta`, `Password`, `add_dialog.rs`, `sevenz.rs`, `folder_formats.rs`, `ProgressSink`, `stream_one`?**
  _High betweenness centrality (0.054) - this node is a cross-community bridge._
- **Should `ops.rs` be split into smaller, more focused modules?**
  _Cohesion score 0.12310606060606061 - nodes in this community are weakly interconnected._