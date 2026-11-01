# Graph Report - ShellZip  (2026-11-01)

## Corpus Check
- 105 files · ~238,978 words
- Verdict: corpus is large enough that graph structure adds value.
- Unclassified: 31 file(s) not represented in the graph (top: .mdc 11, .rar 8, (none) 4)

## Summary
- 2402 nodes · 5900 edges · 99 communities (88 shown, 11 thin omitted)
- Extraction: 97% EXTRACTED · 3% INFERRED · 0% AMBIGUOUS · INFERRED: 164 edges (avg confidence: 0.85)
- Token cost: 0 input · 0 output

## Graph Freshness
- Built from commit: `ee31bf5a`
- Run `git rev-parse HEAD` and compare to check if the graph is stale.
- Run `graphify update .` after code changes (no API cost).

## Community Hubs (Navigation)
- worker.rs
- icons.rs
- ops.rs
- policy.rs
- src/sfx.rs
- Password
- stream.rs
- shellzip-sfx/src/main.rs
- src/entry.rs
- setup.rs
- archiver/zip.rs
- JumpTarget
- Icon
- walk.rs
- progress.rs
- report_dialog.rs
- extractor/tar.rs
- Entry
- Error
- extract.rs
- Scaffold templates
- ShellZipApp
- src/cli.rs
- .retry_with_password
- Command
- xtask/src/main.rs
- shellzip-gui/src/main.rs
- shellzip-core
- model.rs
- milestone2.rs
- view_dialog.rs
- Test patterns
- 4. Phased implementation (Cursor task list)
- .shell_locked
- Tier 1 — What can be proven (direct evidence)
- Rust review checklist
- archiver/tar.rs
- MenuCommand
- Rust performance
- shell_icons/mod.rs
- settings.rs
- Rust quality gate
- setup_template
- info_dialog.rs
- Format
- prompt.rs
- tree.rs
- OpKind
- extractor/zip.rs
- add.rs
- BENCH.md
- about.rs
- Password
- ToolCommand
- tests/cli.rs
- super
- ReportFormat
- listing.rs
- HWND
- run
- BarProgress
- copy_chunked
- prepare_window
- sevenz/entry.rs
- EntryMeta
- milestone3.rs
- paint_banner
- ToolIcon
- add_dialog.rs
- extract_dialog.rs
- sevenz.rs
- view.rs
- Shared
- round_trip.rs
- SetupState
- Place
- folder_formats.rs
- ReportSort
- NopProgress
- app.rs
- Column
- tests/extract.rs
- SortColumn
- theme.rs
- rename_summary.rs
- chrome.rs
- crypto.rs
- CompressOpts
- Outcome
- Target
- RowKind
- pathbuf
- extractor/sevenz/mod.rs
- ArchiveIndex

## God Nodes (most connected - your core abstractions)
1. `ShellZipApp` - 113 edges
2. `ProgressSink` - 80 edges
3. `Password` - 73 edges
4. `CompressOpts` - 67 edges
5. `Entry` - 48 edges
6. `Icon` - 48 edges
7. `Format` - 42 edges
8. `EntryMeta` - 38 edges
9. `ExtractOpts` - 32 edges
10. `FolderTree` - 32 edges

## Surprising Connections (you probably didn't know these)
- `compress_opts()` --calls--> `format_from_extension()`  [INFERRED]
  crates/shellzip-cli/src/cli.rs → crates/shellzip-core/src/detect.rs
- `add()` --calls--> `pack()`  [INFERRED]
  crates/shellzip-cli/src/commands/add.rs → crates/shellzip-gui/src/add_dialog.rs
- `flags_text()` --references--> `Entry`  [EXTRACTED]
  crates/shellzip-cli/src/commands/list.rs → crates/shellzip-core/src/entry.rs
- `test()` --calls--> `test_with()`  [INFERRED]
  crates/shellzip-cli/src/commands/test.rs → crates/shellzip-core/src/lib.rs
- `xtask` --depends_on--> `shellzip-core`  [EXTRACTED]
  xtask/Cargo.toml → crates/shellzip-core/Cargo.toml

## Import Cycles
- 1-file cycle: `crates/shellzip-core/src/archiver/parallel.rs -> crates/shellzip-core/src/archiver/parallel.rs`
- 1-file cycle: `crates/shellzip-sfx/src/setup.rs -> crates/shellzip-sfx/src/setup.rs`
- 1-file cycle: `crates/shellzip-gui/src/add_dialog.rs -> crates/shellzip-gui/src/add_dialog.rs`
- 1-file cycle: `crates/shellzip-gui/src/explorer/model.rs -> crates/shellzip-gui/src/explorer/model.rs`
- 1-file cycle: `crates/shellzip-gui/src/chrome.rs -> crates/shellzip-gui/src/chrome.rs`
- 1-file cycle: `crates/shellzip-gui/src/about.rs -> crates/shellzip-gui/src/about.rs`
- 1-file cycle: `crates/shellzip-gui/src/credits.rs -> crates/shellzip-gui/src/credits.rs`
- 1-file cycle: `crates/shellzip-gui/src/icons.rs -> crates/shellzip-gui/src/icons.rs`
- 1-file cycle: `crates/shellzip-gui/src/explorer/listing.rs -> crates/shellzip-gui/src/explorer/listing.rs`
- 1-file cycle: `crates/shellzip-gui/src/main.rs -> crates/shellzip-gui/src/main.rs`
- 1-file cycle: `crates/shellzip-gui/src/shell_icons/platform.rs -> crates/shellzip-gui/src/shell_icons/platform.rs`
- 2-file cycle: `crates/shellzip-core/src/archiver/mod.rs -> crates/shellzip-core/src/archiver/parallel.rs -> crates/shellzip-core/src/archiver/mod.rs`
- 2-file cycle: `crates/shellzip-gui/src/chrome.rs -> crates/shellzip-gui/src/settings.rs -> crates/shellzip-gui/src/chrome.rs`
- 2-file cycle: `crates/shellzip-cli/src/commands/mod.rs -> crates/shellzip-cli/src/commands/test.rs -> crates/shellzip-cli/src/commands/mod.rs`
- 2-file cycle: `crates/shellzip-cli/src/commands/mod.rs -> crates/shellzip-cli/src/commands/sfx.rs -> crates/shellzip-cli/src/commands/mod.rs`
- 2-file cycle: `crates/shellzip-cli/src/commands/add.rs -> crates/shellzip-cli/src/commands/mod.rs -> crates/shellzip-cli/src/commands/add.rs`
- 2-file cycle: `crates/shellzip-cli/src/commands/delete.rs -> crates/shellzip-cli/src/commands/mod.rs -> crates/shellzip-cli/src/commands/delete.rs`
- 2-file cycle: `crates/shellzip-cli/src/commands/mod.rs -> crates/shellzip-cli/src/commands/print.rs -> crates/shellzip-cli/src/commands/mod.rs`
- 3-file cycle: `crates/shellzip-core/src/archiver/mod.rs -> crates/shellzip-core/src/archiver/parallel.rs -> crates/shellzip-core/src/archiver/zip.rs -> crates/shellzip-core/src/archiver/mod.rs`
- 3-file cycle: `crates/shellzip-cli/src/cli.rs -> crates/shellzip-cli/src/commands/mod.rs -> crates/shellzip-cli/src/commands/test.rs -> crates/shellzip-cli/src/cli.rs`

## Communities (99 total, 11 thin omitted)

### Community 0 - "worker.rs"
Cohesion: 0.06
Nodes (34): bar(), bytes_line(), format_speed(), fraction(), job_fraction(), ProgressCommand, Background, Cancel (+26 more)

### Community 1 - "icons.rs"
Cohesion: 0.16
Nodes (22): ALL, Caret, caret(), Down, Up, image(), load_bytes(), MASTER_POINTS (+14 more)

### Community 2 - "ops.rs"
Cohesion: 0.09
Nodes (39): member_key(), add(), add_inner(), add_prompted(), add_sfx(), add_stdin(), build_globs(), checked_member_name() (+31 more)

### Community 3 - "policy.rs"
Cohesion: 0.09
Nodes (38): admit_existing(), archive_is_newer(), archive_packed_len(), BudgetWriter, BudgetWriter<'_, W>, Destination, destination(), Skip (+30 more)

### Community 4 - "src/sfx.rs"
Cohesion: 0.06
Nodes (46): SfxArgs, default_output(), existing(), find_stub(), has_trailer(), sfx(), AES_METHOD, decoder_rejected() (+38 more)

### Community 5 - "Password"
Cohesion: 0.13
Nodes (19): Password, archive_at(), copy_rar_index(), copy_staged(), dos_time(), entry_from(), extract_rar(), first_volume() (+11 more)

### Community 6 - "stream.rs"
Cohesion: 0.11
Nodes (9): commit_output(), Kind, Gzip, Zstd, SingleArchiver, Stream, Gzip, Idle (+1 more)

### Community 7 - "shellzip-sfx/src/main.rs"
Cohesion: 0.07
Nodes (20): Ask, Bars, byte_style(), sink(), ConflictChoice, Overwrite, Rename, Skip (+12 more)

### Community 8 - "src/entry.rs"
Cohesion: 0.15
Nodes (9): ArchiveArgs, ListArgs, display_name(), flags_text(), list(), list_technical(), dos_parts(), format_attrs() (+1 more)

### Community 9 - "setup.rs"
Cohesion: 0.05
Nodes (11): CB_ADDSTRING, ID_BROWSE, ID_DEST, ID_INTRO, ID_PROGRESS, INTRO, PBM_SETPOS, PBM_SETRANGE32 (+3 more)

### Community 10 - "archiver/zip.rs"
Cohesion: 0.09
Nodes (20): copy_in(), add_zip(), compress_file(), MEMORY_LIMIT, merge(), Piece, Memory, piece_path() (+12 more)

### Community 11 - "JumpTarget"
Cohesion: 0.21
Nodes (13): AddressEvent, Jump, Refresh, Submit, Up, breadcrumbs(), crumb_label(), folder_chain() (+5 more)

### Community 12 - "Icon"
Cohesion: 0.08
Nodes (23): Icon, Archive, ArrowUp, Check, ChevronDown, CircleCheck, CircleHelp, File (+15 more)

### Community 13 - "walk.rs"
Cohesion: 0.10
Nodes (20): EntryKind, Dir, File, Other, Symlink, to_archive_path(), capture(), mode_bits() (+12 more)

### Community 14 - "progress.rs"
Cohesion: 0.09
Nodes (11): ProgressWrite, test_sevenz(), delete(), rename(), CountWait, ensure_running(), ProgressReader, ProgressReader<'a, R> (+3 more)

### Community 15 - "report_dialog.rs"
Cohesion: 0.15
Nodes (30): format_time(), archive(), cmp_opt(), columns(), DIALOG_WIDTH, display_width(), escape_html(), escape_markdown() (+22 more)

### Community 16 - "extractor/tar.rs"
Cohesion: 0.12
Nodes (26): Compression, Bzip2, Gzip, None, Xz, Zstd, copy_single(), copy_tar_index() (+18 more)

### Community 17 - "Entry"
Cohesion: 0.13
Nodes (10): Entry, ArchiveReader, only_first(), Opened, resolve(), unsupported(), Verify, list() (+2 more)

### Community 18 - "Error"
Cohesion: 0.19
Nodes (14): Error, Archive, BadChecksum, BadPassword, Encrypted, Interrupted, Io, Memory (+6 more)

### Community 19 - "extract.rs"
Cohesion: 0.17
Nodes (10): ExtractArgs, LimitArgs, Overwrite, overwrite_mode(), OverwriteArg, Always, Ask, Rename (+2 more)

### Community 20 - "Scaffold templates"
Cohesion: 0.10
Nodes (19): Criterion benchmark, Example, Extra binary in an existing crate, Finish checklist, Integration test file, New module in an existing crate, New `szip` subcommand, New workspace crate (+11 more)

### Community 22 - "src/cli.rs"
Cohesion: 0.12
Nodes (11): Cli, dispatch(), init_tracing(), normalize_args(), password_from_env(), prompt_password(), run(), SfxFormatArg (+3 more)

### Community 23 - ".retry_with_password"
Cohesion: 0.16
Nodes (5): decide_member(), find_stub(), list_archive(), needs_password(), Extract

### Community 24 - "Command"
Cohesion: 0.14
Nodes (13): Command, Add, Complete, Delete, Extract, ExtractFlat, Freshen, List (+5 more)

### Community 25 - "xtask/src/main.rs"
Cohesion: 0.16
Nodes (23): bench(), build_fixtures(), build_sevenz_fixtures(), collect_files(), crc32(), find_rar(), find_tool(), FIXTURE_PASSWORD (+15 more)

### Community 26 - "shellzip-gui/src/main.rs"
Cohesion: 0.24
Nodes (7): anstream_fallback(), clear_white_margin(), is_white_margin(), main(), pixel_index(), window_icon(), window_icon_is_square_and_its_corners_are_clear()

### Community 27 - "shellzip-core"
Cohesion: 0.40
Nodes (5): shellzip-cli, shellzip-core, shellzip-gui, shellzip-sfx, xtask

### Community 29 - "model.rs"
Cohesion: 0.14
Nodes (15): archive_prefix(), byte_phrase(), dir_partial(), file_partial(), format_modified(), format_modified_is_day_month_year(), format_size(), insert() (+7 more)

### Community 30 - "milestone2.rs"
Cohesion: 0.29
Nodes (11): ask_can_skip_replace_or_rename(), freshen_missing_archive_is_not_found(), opts(), parallel_zip_matches_single_thread_bytes(), skip_leaves_existing_members_and_adds_new_ones(), stdin_streams_zst_and_spools_tar(), sync_drops_members_that_were_not_selected(), technical_list_fills_blake2sp() (+3 more)

### Community 31 - "view_dialog.rs"
Cohesion: 0.11
Nodes (17): bytes_len(), cap_writer_stops_at_the_limit(), CapWriter, classify(), PREVIEW_CAP, PreviewBody, Binary, Text (+9 more)

### Community 32 - "Test patterns"
Cohesion: 0.11
Nodes (17): CLI end-to-end with `assert_cmd` (`tests/cli.rs`), Counting fake for a trait, In-memory reader/writer, Integration round-trip (`tests/<scenario>.rs`), Interop with OS tools, Platform-gated test, Property test with `proptest`, Snapshot test with `insta` (+9 more)

### Community 33 - "4. Phased implementation (Cursor task list)"
Cohesion: 0.12
Nodes (15): 1. Scope & feature‑parity matrix, 2. Recommended crate ecosystem, 3. Project structure, 4. Phased implementation (Cursor task list), 5. Testing & verification strategy, 6. Licensing notes, Design principles carried over from WinRAR, Milestone 0 — Skeleton (½ day) (+7 more)

### Community 34 - ".shell_locked"
Cohesion: 0.23
Nodes (3): spawn_directory(), folder_parent(), location_text()

### Community 35 - "Tier 1 — What can be proven (direct evidence)"
Cohesion: 0.20
Nodes (9): 1.1 Directory inventory (32 files, all in one flat directory), 1.2 Separation of concerns, 1.3 Imported Windows APIs grouped by capability, 1.4 CLI surface (from `Rar.exe -?`, `UnRAR.exe`, `Rar.txt`), Evidence ledger (summary), Tier 1 — What can be proven (direct evidence), Tier 2 — What can be inferred (architectural deductions), Tier 3 — What remains hidden (opaque internals) (+1 more)

### Community 36 - "Rust review checklist"
Cohesion: 0.13
Nodes (13): 1. Correctness, 2. Error handling, 3. Security, 4. Ownership and performance, 5. Concurrency, 6. Tests, 7. API and docs, 8. Manifest and tooling (+5 more)

### Community 37 - "archiver/tar.rs"
Cohesion: 0.12
Nodes (8): CompWriter, Gzip, Plain, Zstd, dir_name(), header_for(), TarArchiver, unix_secs()

### Community 38 - "MenuCommand"
Cohesion: 0.11
Nodes (20): menu_bar(), MenuCommand, About, Add, ClearHistory, Delete, Exit, Extract (+12 more)

### Community 39 - "Rust performance"
Cohesion: 0.29
Nodes (6): 1. Baseline, 2. Find the hot spot, 3. Apply the cheapest fix the profile justifies, 4. Binary size, 5. Verify and report, Rust performance

### Community 40 - "shell_icons/mod.rs"
Cohesion: 0.06
Nodes (49): caches_a_raster_from_the_provider(), decode_png(), extension_of(), icon_keys_follow_extension_path_and_folder(), IconCache, IconKey, Computer, Extension (+41 more)

### Community 41 - "settings.rs"
Cohesion: 0.07
Nodes (38): ExtractUpdate, Freshen, Replace, Update, Overwrite, Always, Ask, Never (+30 more)

### Community 42 - "Rust quality gate"
Cohesion: 0.29
Nodes (6): Fix clippy, don't silence it, Fix tests, Optional supply-chain checks, Report, Rust quality gate, Workflow

### Community 43 - "setup_template"
Cohesion: 0.47
Nodes (9): align4(), Control, push_atom(), push_named(), push_u16(), push_u32(), push_wide(), setup_dialog_has_browse_and_a_progress_bar() (+1 more)

### Community 44 - "info_dialog.rs"
Cohesion: 0.16
Nodes (20): archive_summary(), ArchiveSummary, comment_tab(), DIALOG_WIDTH, file_name(), help_window(), info_rows(), info_tab() (+12 more)

### Community 45 - "Format"
Cohesion: 0.06
Nodes (51): rewrite_renamed(), CreatableFormat, detect_bytes(), detect_file(), detect_header(), find_embedded(), find_subslice(), Format (+43 more)

### Community 46 - "prompt.rs"
Cohesion: 0.07
Nodes (49): accept_password(), ask(), ask_console(), ask_gui(), attach_console(), attach_parent(), Choice, All (+41 more)

### Community 47 - "tree.rs"
Cohesion: 0.06
Nodes (67): read_subdirs(), archive_prefixes_hang_under_the_archive(), best_volume(), Builder, cmp_name(), component_len(), computer_label(), drive_root() (+59 more)

### Community 48 - "OpKind"
Cohesion: 0.20
Nodes (11): listed_effect(), ListedEffect, Browse, Finish, Operation, OpKind, Add, Delete (+3 more)

### Community 49 - "extractor/zip.rs"
Cohesion: 0.12
Nodes (14): Budget, copy_limited(), annotate(), entry_from(), extract_zip(), extract_zip_index(), hash_capped(), host_label() (+6 more)

### Community 50 - "add.rs"
Cohesion: 0.10
Nodes (20): AddArgs, compress_opts(), MethodArg, Bzip2, Deflate, Lzma, Lzma2, Ppmd (+12 more)

### Community 52 - "about.rs"
Cohesion: 0.13
Nodes (18): BANNER, banner_fills_the_dialog_width(), banner_size(), DIALOG_WIDTH, identity(), license(), MIT_LICENSE, notices() (+10 more)

### Community 54 - "Password"
Cohesion: 0.07
Nodes (22): add_prompted(), add_stdin(), extract(), list_with(), test(), test_with(), write_member(), test() (+14 more)

### Community 55 - "ToolCommand"
Cohesion: 0.20
Nodes (10): ToolCommand, About, Add, Delete, Extract, Info, Open, Sfx (+2 more)

### Community 56 - "tests/cli.rs"
Cohesion: 0.15
Nodes (22): add_list_test_extract_round_trip(), add_sfx_writes_a_listable_package(), add_to_an_existing_zip_keeps_previous_files(), help_explains_that_zip_encryption_leaves_names_visible(), log_file_receives_debug_events(), max_size_stops_extract_and_test(), rar_is_not_created(), reads_a_tar_from_the_system_tool() (+14 more)

### Community 57 - "super"
Cohesion: 0.17
Nodes (3): blake2sp_matches_the_reference_vector(), Blake2spWriter, CrcWriter

### Community 58 - "ReportFormat"
Cohesion: 0.18
Nodes (14): browse_report(), default_report_path(), extension(), help_window(), ReportAction, Close, Save, Wrote (+6 more)

### Community 60 - "listing.rs"
Cohesion: 0.33
Nodes (5): lists_a_folder_and_a_text_file(), read_directory(), row_for(), subdirs_skip_files(), TempDir

### Community 61 - "HWND"
Cohesion: 0.21
Nodes (19): browse_initialized(), enable_window(), EnableWindow(), finish_if_ready(), on_browse(), on_command(), on_extract(), paint_banner() (+11 more)

### Community 62 - "run"
Cohesion: 0.25
Nodes (7): ComInit, dword_words(), finish_worker(), init_common_controls(), run(), show(), windows_accepts_the_setup_dialog_template()

### Community 65 - "prepare_window"
Cohesion: 0.38
Nodes (7): add_choice(), browse_folder(), prepare_window(), set_text(), set_window_text(), wide_os(), wide_str()

### Community 66 - "sevenz/entry.rs"
Cohesion: 0.09
Nodes (18): coder_dictionary(), coder_label(), entry_from(), entry_key(), file(), folder_dictionary(), folder_label(), host_os() (+10 more)

### Community 67 - "EntryMeta"
Cohesion: 0.12
Nodes (14): archive_path(), CloneMeta, EntryMeta, seven_meta(), SevenZArchiver, spool(), Spooled, write_file_body() (+6 more)

### Community 68 - "milestone3.rs"
Cohesion: 0.20
Nodes (11): assert_matches_unrar(), encrypting_an_existing_zip_requires_force(), extract_bytes(), find_unrar(), FIXTURE_PASSWORD, fixtures_root(), password(), passwords_are_rejected_for_tar() (+3 more)

### Community 69 - "paint_banner"
Cohesion: 0.28
Nodes (7): Banner, banner_pixels_are_bgra(), blit_banner(), decode_banner(), DrawItem, fill_band(), fitted()

### Community 70 - "ToolIcon"
Cohesion: 0.10
Nodes (18): dark_toolbar_strokes_are_light(), recolor_dark(), TOOL_ALL, tool_bytes(), tool_cache_key(), toolbar_svgs_carry_the_brand_colors(), ToolIcon, About (+10 more)

### Community 71 - "add_dialog.rs"
Cohesion: 0.06
Nodes (59): creatable_formats(), SevenZMethod, Bzip2, Deflate, Lzma, Lzma2, Ppmd, Zstd (+51 more)

### Community 72 - "extract_dialog.rs"
Cohesion: 0.10
Nodes (38): advanced_tab(), begin_new_folder(), colors(), confirm_delete_archive(), confirm_new_folder(), destination_uses_the_archive_name_without_its_extension(), DIALOG_WIDTH, display_path() (+30 more)

### Community 73 - "sevenz.rs"
Cohesion: 0.13
Nodes (19): codec_method(), write_options(), add_lists_and_extracts_through_the_core(), aes_hides_bytes_and_header_encryption_hides_names(), bzip2_and_deflate_round_trip(), committed_sevenz_fixtures_list_when_present(), extract_bytes(), fast() (+11 more)

### Community 74 - "view.rs"
Cohesion: 0.14
Nodes (14): format_crc(), cell_text(), crc_text(), Explorer, file(), header_clicked(), invert_selection_flips_every_row_except_parent(), modified_text() (+6 more)

### Community 75 - "Shared"
Cohesion: 0.24
Nodes (7): DeleteArgs, PrintArgs, TestArgs, delete(), Shared, print_entry(), test()

### Community 76 - "round_trip.rs"
Cohesion: 0.25
Nodes (17): adding_to_an_existing_zip_keeps_the_other_files(), byte_cap_stops_extraction(), existing_tar_is_left_in_place_without_force(), extract_all(), gzip_rejects_a_second_file(), opts_fast(), overwrite_skip_and_rename(), pack_tree() (+9 more)

### Community 77 - "SetupState"
Cohesion: 0.28
Nodes (5): destroy_icon(), install_icons(), load_icon(), percent(), SetupState

### Community 78 - "Place"
Cohesion: 0.40
Nodes (3): Place, Archive, Folder

### Community 79 - "folder_formats.rs"
Cohesion: 0.35
Nodes (12): assert_tree(), Case, cases(), extract_all(), nested_folder_round_trips_every_writable_format(), opts(), parallel_zip_round_trips_a_folder_of_spilled_files(), planned_names() (+4 more)

### Community 80 - "ReportSort"
Cohesion: 0.25
Nodes (8): report_sort_name(), ReportSort, Crc, Modified, Name, Original, Packed, Size

### Community 83 - "app.rs"
Cohesion: 0.09
Nodes (19): clear_history_forgets_opened_archives_and_destinations(), folder_still_shown(), folder_title(), forget_history(), is_sfx_name(), PendingArchiveDelete, report_entry(), report_row() (+11 more)

### Community 84 - "Column"
Cohesion: 0.33
Nodes (6): Column, Crc, Modified, Name, Packed, Size

### Community 85 - "tests/extract.rs"
Cohesion: 0.29
Nodes (9): built_stub_contains_the_shellzip_icon(), encrypted_sfx(), silent_encrypted_stub_fails_without_a_password(), silent_encrypted_stub_rejects_a_wrong_password(), stub_bin(), stub_extracts_a_header_encrypted_7z_with_the_password(), stub_extracts_an_encrypted_zip_when_the_password_is_set(), stub_extracts_its_appended_7z() (+1 more)

### Community 86 - "SortColumn"
Cohesion: 0.14
Nodes (15): compare_column(), compare_name(), compare_option(), sort_keeps_parent_and_folders_above_files(), sort_rows(), SortColumn, Crc, Modified (+7 more)

### Community 87 - "theme.rs"
Cohesion: 0.13
Nodes (14): directories_use_the_yellow_folder(), row_glyph(), selected_glyphs_use_the_selection_ink(), apply(), apply_installs_both_themes(), DARK, LIGHT, Palette (+6 more)

### Community 88 - "rename_summary.rs"
Cohesion: 0.13
Nodes (9): Cancel, fingerprint(), names(), rename_cancel_leaves_the_archive_unchanged(), rename_directory_rewrites_children(), rename_rejects_an_existing_destination(), rename_rejects_parent_directory_names(), stored_zip() (+1 more)

### Community 89 - "chrome.rs"
Cohesion: 0.11
Nodes (26): address_bar(), bar_frame(), caret_button(), confirm_delete(), DeleteChoice, Cancel, Delete, Wait (+18 more)

### Community 92 - "CompressOpts"
Cohesion: 0.09
Nodes (12): ArchiveWriter, create(), create_from_path(), prepare_output(), prepare_temp(), temp_beside(), rewrite_renamed(), tune() (+4 more)

### Community 93 - "Outcome"
Cohesion: 0.11
Nodes (15): PasswordAsk, preview_outcome(), JobToken, Outcome, Directory, Extracted, Failed, Finished (+7 more)

### Community 94 - "Target"
Cohesion: 0.25
Nodes (9): named(), Row, Target, ArchiveDir, ArchiveMember, Parent, Path, totals() (+1 more)

### Community 95 - "RowKind"
Cohesion: 0.20
Nodes (10): extension(), file_type_label(), rank(), RowKind, Archive, Directory, File, Parent (+2 more)

### Community 98 - "pathbuf"
Cohesion: 0.21
Nodes (8): copy_zone(), MAX_ZONE_BYTES, propagate(), zone_path(), main(), windows_sdk_bin(), main(), windows_sdk_bin()

### Community 99 - "extractor/sevenz/mod.rs"
Cohesion: 0.17
Nodes (15): mark_extracted(), restore_entry_time(), symlink_destination(), finish_staged(), Member, copy_sevenz_index(), extract_sevenz(), ExtractOne (+7 more)

### Community 102 - "ArchiveIndex"
Cohesion: 0.43
Nodes (4): ArchiveIndex, directory_marker_hides_its_own_size(), entry(), index_builds_nested_folders()

## Knowledge Gaps
- **273 isolated node(s):** `shellzip-cli`, `Add`, `Update`, `Freshen`, `Delete` (+268 more)
  These have ≤1 connection - possible missing edges or undocumented components. (Counts symbols only; 703 node(s) total have ≤1 connection when file, concept and rationale nodes are included.)
- **11 thin communities (<3 nodes) omitted from report** — run `graphify query` to explore isolated nodes.

## Suggested Questions
_Questions this graph is uniquely positioned to answer:_

- **Why does `Error` connect `Error` to `worker.rs`, `ops.rs`, `policy.rs`, `src/sfx.rs`, `Password`, `stream.rs`, `shellzip-sfx/src/main.rs`, `src/entry.rs`, `setup.rs`, `archiver/zip.rs`, `walk.rs`, `extractor/tar.rs`, `Entry`, `extract.rs`, `src/cli.rs`, `view_dialog.rs`, `archiver/tar.rs`, `Format`, `prompt.rs`, `extractor/zip.rs`, `add.rs`, `Password`, `EntryMeta`, `milestone3.rs`, `sevenz.rs`, `Shared`, `round_trip.rs`, `folder_formats.rs`, `app.rs`, `rename_summary.rs`, `CompressOpts`, `pathbuf`, `extractor/sevenz/mod.rs`?**
  _High betweenness centrality (0.086) - this node is a cross-community bridge._
- **What connects `shellzip-cli`, `Add`, `Update` to the rest of the system?**
  _273 weakly-connected nodes found - possible documentation gaps or missing edges._
- **Should `worker.rs` be split into smaller, more focused modules?**
  _Cohesion score 0.059227921734531994 - nodes in this community are weakly interconnected._
- **Why does `ShellZipApp` connect `ShellZipApp` to `worker.rs`, `Password`, `report_dialog.rs`, `.retry_with_password`, `view_dialog.rs`, `.shell_locked`, `shell_icons/mod.rs`, `settings.rs`, `info_dialog.rs`, `tree.rs`, `OpKind`, `Password`, `add_dialog.rs`, `extract_dialog.rs`, `view.rs`, `Place`, `.refuse_operation`, `app.rs`, `theme.rs`, `Outcome`?**
  _High betweenness centrality (0.085) - this node is a cross-community bridge._
- **Should `ops.rs` be split into smaller, more focused modules?**
  _Cohesion score 0.09268707482993198 - nodes in this community are weakly interconnected._
- **Why does `Password` connect `Password` to `ops.rs`, `src/sfx.rs`, `shellzip-sfx/src/main.rs`, `src/entry.rs`, `setup.rs`, `progress.rs`, `Entry`, `ShellZipApp`, `src/cli.rs`, `.retry_with_password`, `view_dialog.rs`, `extractor/zip.rs`, `Password`, `run`, `sevenz/entry.rs`, `Shared`, `SetupState`, `.refuse_operation`, `app.rs`, `crypto.rs`, `CompressOpts`, `extractor/sevenz/mod.rs`?**
  _High betweenness centrality (0.071) - this node is a cross-community bridge._
- **Should `policy.rs` be split into smaller, more focused modules?**
  _Cohesion score 0.08653061224489796 - nodes in this community are weakly interconnected._