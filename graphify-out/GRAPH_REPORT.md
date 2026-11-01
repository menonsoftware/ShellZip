# Graph Report - ShellZip  (2026-11-01)

## Corpus Check
- 112 files · ~245,422 words
- Verdict: corpus is large enough that graph structure adds value.
- Unclassified: 31 file(s) not represented in the graph (top: .mdc 11, .rar 8, (none) 4)

## Summary
- 2513 nodes · 6061 edges · 115 communities (103 shown, 12 thin omitted)
- Extraction: 97% EXTRACTED · 3% INFERRED · 0% AMBIGUOUS · INFERRED: 164 edges (avg confidence: 0.85)
- Token cost: 0 input · 0 output

## Graph Freshness
- Built from commit: `907f153d`
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
- ShellZip FAQ
- setup.rs
- archiver/zip.rs
- parallel.rs
- Icon
- walk.rs
- ProgressReader
- report_dialog.rs
- extractor/tar.rs
- Entry
- Error
- extract.rs
- Scaffold templates
- ShellZipApp
- src/cli.rs
- JumpTarget
- Command
- xtask/src/main.rs
- shellzip-gui/src/main.rs
- shellzip-core
- model.rs
- milestone2.rs
- view_dialog.rs
- Test patterns
- 4. Phased implementation (Cursor task list)
- Building ShellZip
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
- .poll
- extractor/zip.rs
- MethodArg
- docs/README.md
- Explorer
- Password
- ToolCommand
- tests/cli.rs
- super
- summary.rs
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
- Archive formats
- path
- round_trip.rs
- copy_limited
- app.rs
- folder_formats.rs
- TestReport
- Password
- NopProgress
- rename_dialog.rs
- Commands
- tests/extract.rs
- ShellZip desktop app
- theme.rs
- rename_summary.rs
- chrome.rs
- ShellZip
- view.rs
- CompressOpts
- SetupState
- pathbuf
- about.rs
- .on_menu
- szip command-line reference
- ReportFormat
- extractor/sevenz/mod.rs
- ShellZip documentation
- Shared
- list.rs
- ToolbarSize
- commands/sfx.rs
- sevenz/options.rs
- Credit
- ReportSort
- NopProgress
- Column
- write
- Overwrite
- appearance.rs
- MemberAction

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
- `sfx()` --calls--> `convert()`  [INFERRED]
  crates/shellzip-cli/src/commands/sfx.rs → crates/shellzip-core/src/sfx.rs
- `test()` --calls--> `test_with()`  [INFERRED]
  crates/shellzip-cli/src/commands/test.rs → crates/shellzip-core/src/lib.rs

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

## Communities (115 total, 12 thin omitted)

### Community 0 - "worker.rs"
Cohesion: 0.06
Nodes (35): bar(), bytes_line(), format_speed(), fraction(), job_fraction(), ProgressCommand, Background, Cancel (+27 more)

### Community 1 - "icons.rs"
Cohesion: 0.14
Nodes (22): ALL, Caret, caret(), Down, Up, image(), load_bytes(), MASTER_POINTS (+14 more)

### Community 2 - "ops.rs"
Cohesion: 0.12
Nodes (30): Entry, member_key(), to_archive_path(), add(), add_inner(), add_prompted(), add_stdin(), build_globs() (+22 more)

### Community 3 - "policy.rs"
Cohesion: 0.13
Nodes (30): admit_existing(), archive_is_newer(), Destination, destination(), Skip, Write, disk_mtime(), exists() (+22 more)

### Community 4 - "src/sfx.rs"
Cohesion: 0.06
Nodes (44): test_sevenz(), AES_METHOD, decoder_rejected(), explain_open(), header_uses_aes(), MAX_MEMORY, open_streaming(), add_sfx() (+36 more)

### Community 5 - "Password"
Cohesion: 0.14
Nodes (23): stage_path(), archive_at(), copy_rar_index(), copy_staged(), DECODE_IN_MEMORY, dos_time(), entry_from(), extract_rar() (+15 more)

### Community 6 - "stream.rs"
Cohesion: 0.11
Nodes (9): commit_output(), Kind, Gzip, Zstd, SingleArchiver, Stream, Gzip, Idle (+1 more)

### Community 7 - "shellzip-sfx/src/main.rs"
Cohesion: 0.05
Nodes (22): Ask, Bars, byte_style(), sink(), ConflictChoice, Overwrite, Rename, Skip (+14 more)

### Community 8 - "ShellZip FAQ"
Cohesion: 0.09
Nodes (22): Can I pipe data in or out?, Can I update or delete files inside an existing archive?, Does ShellZip require `unsafe`?, How do I make a self-extracting archive?, How do I pass a password without putting it on the command line?, How do I run the tests?, How do threads work?, How is this different from 7-Zip or WinRAR? (+14 more)

### Community 9 - "setup.rs"
Cohesion: 0.05
Nodes (13): CB_ADDSTRING, ID_BROWSE, ID_DEST, ID_INTRO, ID_PROGRESS, INTRO, PBM_SETBARCOLOR, PBM_SETPOS (+5 more)

### Community 10 - "archiver/zip.rs"
Cohesion: 0.16
Nodes (10): apply_comment(), copy_renamed(), method_for(), options_for(), rel_name(), rewrite_renamed(), target_text(), zip_dest_name() (+2 more)

### Community 11 - "parallel.rs"
Cohesion: 0.18
Nodes (11): copy_in(), add_zip(), compress_file(), MEMORY_LIMIT, merge(), Piece, Memory, piece_path() (+3 more)

### Community 12 - "Icon"
Cohesion: 0.08
Nodes (23): Icon, Archive, ArrowUp, Check, ChevronDown, CircleCheck, CircleHelp, File (+15 more)

### Community 13 - "walk.rs"
Cohesion: 0.12
Nodes (21): EntryKind, Dir, File, Other, Symlink, format_attrs(), capture(), mode_bits() (+13 more)

### Community 14 - "ProgressReader"
Cohesion: 0.40
Nodes (3): ProgressReader, ProgressReader<'a, R>, reader_and_ensure_running_honor_pause()

### Community 15 - "report_dialog.rs"
Cohesion: 0.14
Nodes (31): format_time(), report_entry(), archive(), cmp_opt(), columns(), DIALOG_WIDTH, display_width(), escape_html() (+23 more)

### Community 16 - "extractor/tar.rs"
Cohesion: 0.12
Nodes (28): Compression, Bzip2, Gzip, None, Xz, Zstd, copy_single(), copy_tar_index() (+20 more)

### Community 17 - "Entry"
Cohesion: 0.14
Nodes (9): ArchiveReader, only_first(), Opened, resolve(), unsupported(), Verify, list(), open() (+1 more)

### Community 18 - "Error"
Cohesion: 0.19
Nodes (14): Error, Archive, BadChecksum, BadPassword, Encrypted, Interrupted, Io, Memory (+6 more)

### Community 19 - "extract.rs"
Cohesion: 0.18
Nodes (10): ExtractArgs, LimitArgs, Overwrite, overwrite_mode(), OverwriteArg, Always, Ask, Rename (+2 more)

### Community 20 - "Scaffold templates"
Cohesion: 0.10
Nodes (19): Criterion benchmark, Example, Extra binary in an existing crate, Finish checklist, Integration test file, New module in an existing crate, New `szip` subcommand, New workspace crate (+11 more)

### Community 22 - "src/cli.rs"
Cohesion: 0.14
Nodes (8): Cli, dispatch(), init_tracing(), normalize_args(), password_from_env(), prompt_password(), run(), take_password()

### Community 23 - "JumpTarget"
Cohesion: 0.22
Nodes (11): AddressEvent, Jump, Refresh, Submit, Up, breadcrumbs(), crumb_label(), Jump (+3 more)

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
Cohesion: 0.06
Nodes (46): archive_prefix(), ArchiveIndex, byte_phrase(), compare_column(), compare_name(), compare_option(), dir_partial(), directory_marker_hides_its_own_size() (+38 more)

### Community 30 - "milestone2.rs"
Cohesion: 0.29
Nodes (11): ask_can_skip_replace_or_rename(), freshen_missing_archive_is_not_found(), opts(), parallel_zip_matches_single_thread_bytes(), skip_leaves_existing_members_and_adds_new_ones(), stdin_streams_zst_and_spools_tar(), sync_drops_members_that_were_not_selected(), technical_list_fills_blake2sp() (+3 more)

### Community 31 - "view_dialog.rs"
Cohesion: 0.12
Nodes (17): bytes_len(), cap_writer_stops_at_the_limit(), CapWriter, classify(), PREVIEW_CAP, PreviewBody, Binary, Text (+9 more)

### Community 32 - "Test patterns"
Cohesion: 0.11
Nodes (17): CLI end-to-end with `assert_cmd` (`tests/cli.rs`), Counting fake for a trait, In-memory reader/writer, Integration round-trip (`tests/<scenario>.rs`), Interop with OS tools, Platform-gated test, Property test with `proptest`, Snapshot test with `insta` (+9 more)

### Community 33 - "4. Phased implementation (Cursor task list)"
Cohesion: 0.13
Nodes (15): 1. Scope & feature‑parity matrix, 2. Recommended crate ecosystem, 3. Project structure, 4. Phased implementation (Cursor task list), 5. Testing & verification strategy, 6. Licensing notes, Design principles carried over from WinRAR, Milestone 0 — Skeleton (½ day) (+7 more)

### Community 34 - "Building ShellZip"
Cohesion: 0.18
Nodes (11): API docs, Benchmarks, Building ShellZip, Cargo features, Platform notes, Profiles, Quality gate, Quick commands (+3 more)

### Community 35 - "Tier 1 — What can be proven (direct evidence)"
Cohesion: 0.22
Nodes (9): 1.1 Directory inventory (32 files, all in one flat directory), 1.2 Separation of concerns, 1.3 Imported Windows APIs grouped by capability, 1.4 CLI surface (from `Rar.exe -?`, `UnRAR.exe`, `Rar.txt`), Evidence ledger (summary), Tier 1 — What can be proven (direct evidence), Tier 2 — What can be inferred (architectural deductions), Tier 3 — What remains hidden (opaque internals) (+1 more)

### Community 36 - "Rust review checklist"
Cohesion: 0.13
Nodes (13): 1. Correctness, 2. Error handling, 3. Security, 4. Ownership and performance, 5. Concurrency, 6. Tests, 7. API and docs, 8. Manifest and tooling (+5 more)

### Community 37 - "archiver/tar.rs"
Cohesion: 0.12
Nodes (8): CompWriter, Gzip, Plain, Zstd, dir_name(), header_for(), TarArchiver, unix_secs()

### Community 38 - "MenuCommand"
Cohesion: 0.09
Nodes (24): menu_bar(), MenuCommand, About, Add, ClearHistory, Delete, Exit, Extract (+16 more)

### Community 39 - "Rust performance"
Cohesion: 0.29
Nodes (6): 1. Baseline, 2. Find the hot spot, 3. Apply the cheapest fix the profile justifies, 4. Binary size, 5. Verify and report, Rust performance

### Community 40 - "shell_icons/mod.rs"
Cohesion: 0.06
Nodes (49): caches_a_raster_from_the_provider(), decode_png(), extension_of(), icon_keys_follow_extension_path_and_folder(), IconCache, IconKey, Computer, Extension (+41 more)

### Community 41 - "settings.rs"
Cohesion: 0.11
Nodes (29): ExtractUpdate, Freshen, Replace, Update, defaults(), delete_name(), DeleteArchive, Always (+21 more)

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
Cohesion: 0.10
Nodes (26): creatable_extensions_round_trip(), detect_bytes(), detect_file(), detect_header(), find_embedded(), find_subslice(), Format, Bzip2 (+18 more)

### Community 46 - "prompt.rs"
Cohesion: 0.07
Nodes (49): accept_password(), ask(), ask_console(), ask_gui(), attach_console(), attach_parent(), Choice, All (+41 more)

### Community 47 - "tree.rs"
Cohesion: 0.06
Nodes (66): archive_prefixes_hang_under_the_archive(), best_volume(), Builder, cmp_name(), component_len(), computer_label(), drive_root(), drive_root_is_the_filesystem_root() (+58 more)

### Community 48 - ".poll"
Cohesion: 0.27
Nodes (3): folder_still_shown(), preview_outcome(), JobToken

### Community 49 - "extractor/zip.rs"
Cohesion: 0.13
Nodes (14): partial_path(), write_atomic(), annotate(), entry_from(), extract_zip(), extract_zip_index(), hash_capped(), host_label() (+6 more)

### Community 50 - "MethodArg"
Cohesion: 0.16
Nodes (14): AddArgs, compress_opts(), MethodArg, Bzip2, Deflate, Lzma, Lzma2, Ppmd (+6 more)

### Community 52 - "Explorer"
Cohesion: 0.11
Nodes (13): SortColumn, Crc, Modified, Name, Packed, Size, Type, SortOrder (+5 more)

### Community 54 - "Password"
Cohesion: 0.07
Nodes (23): CreatableFormat, ProgressWrite, add(), add_prompted(), add_stdin(), delete(), extract(), list_with() (+15 more)

### Community 55 - "ToolCommand"
Cohesion: 0.20
Nodes (10): ToolCommand, About, Add, Delete, Extract, Info, Open, Sfx (+2 more)

### Community 56 - "tests/cli.rs"
Cohesion: 0.15
Nodes (22): add_list_test_extract_round_trip(), add_sfx_writes_a_listable_package(), add_to_an_existing_zip_keeps_previous_files(), help_explains_that_zip_encryption_leaves_names_visible(), log_file_receives_debug_events(), max_size_stops_extract_and_test(), rar_is_not_created(), reads_a_tar_from_the_system_tool() (+14 more)

### Community 57 - "super"
Cohesion: 0.17
Nodes (3): blake2sp_matches_the_reference_vector(), Blake2spWriter, CrcWriter

### Community 58 - "summary.rs"
Cohesion: 0.14
Nodes (25): ABSENT, add_bytes(), archive_summary(), comment_of(), FOUR_GIB, host_label(), host_line(), kind_label() (+17 more)

### Community 60 - "listing.rs"
Cohesion: 0.36
Nodes (6): lists_a_folder_and_a_text_file(), read_directory(), read_subdirs(), row_for(), subdirs_skip_files(), TempDir

### Community 61 - "HWND"
Cohesion: 0.21
Nodes (19): browse_initialized(), enable_window(), EnableWindow(), finish_if_ready(), on_browse(), on_command(), on_extract(), paint_banner() (+11 more)

### Community 62 - "run"
Cohesion: 0.22
Nodes (8): ComInit, dword_words(), dword_words_keep_the_template_bytes(), finish_worker(), init_common_controls(), run(), show(), windows_accepts_the_setup_dialog_template()

### Community 65 - "prepare_window"
Cohesion: 0.32
Nodes (8): add_choice(), browse_folder(), prepare_window(), set_text(), set_window_text(), style_progress(), wide_os(), wide_str()

### Community 66 - "sevenz/entry.rs"
Cohesion: 0.10
Nodes (18): coder_dictionary(), coder_label(), entry_from(), entry_key(), file(), folder_dictionary(), folder_label(), host_os() (+10 more)

### Community 67 - "EntryMeta"
Cohesion: 0.11
Nodes (15): archive_path(), CloneMeta, EntryMeta, seven_meta(), SevenZArchiver, spool(), Spooled, write_file_body() (+7 more)

### Community 68 - "milestone3.rs"
Cohesion: 0.17
Nodes (16): assert_matches_unrar(), encrypting_an_existing_zip_requires_force(), extract_bytes(), find_unrar(), FIXTURE_PASSWORD, fixtures_root(), password(), passwords_are_rejected_for_tar() (+8 more)

### Community 69 - "paint_banner"
Cohesion: 0.28
Nodes (7): Banner, banner_pixels_are_bgra(), blit_banner(), decode_banner(), DrawItem, fill_band(), fitted()

### Community 70 - "ToolIcon"
Cohesion: 0.10
Nodes (18): dark_toolbar_strokes_are_light(), recolor_dark(), TOOL_ALL, tool_bytes(), tool_cache_key(), toolbar_svgs_carry_the_brand_colors(), ToolIcon, About (+10 more)

### Community 71 - "add_dialog.rs"
Cohesion: 0.05
Nodes (68): creatable_formats(), SevenZMethod, Bzip2, Deflate, Lzma, Lzma2, Ppmd, Zstd (+60 more)

### Community 72 - "extract_dialog.rs"
Cohesion: 0.11
Nodes (37): advanced_tab(), begin_new_folder(), colors(), confirm_delete_archive(), confirm_new_folder(), destination_uses_the_archive_name_without_its_extension(), DIALOG_WIDTH, display_path() (+29 more)

### Community 73 - "sevenz.rs"
Cohesion: 0.20
Nodes (17): add_lists_and_extracts_through_the_core(), aes_hides_bytes_and_header_encryption_hides_names(), bzip2_and_deflate_round_trip(), committed_sevenz_fixtures_list_when_present(), extract_bytes(), fast(), FIXTURE_PASSWORD, header_encryption_is_a_password_error_for_every_nonce() (+9 more)

### Community 74 - "Archive formats"
Cohesion: 0.18
Nodes (11): 7z, Archive formats, Capability matrix, Creation methods, Detection versus extension, Encryption, Feature flags (`shellzip-core`), Safety (+3 more)

### Community 75 - "path"
Cohesion: 0.12
Nodes (5): dos_parts(), copy_zone(), MAX_ZONE_BYTES, propagate(), zone_path()

### Community 76 - "round_trip.rs"
Cohesion: 0.25
Nodes (17): adding_to_an_existing_zip_keeps_the_other_files(), byte_cap_stops_extraction(), existing_tar_is_left_in_place_without_force(), extract_all(), gzip_rejects_a_second_file(), opts_fast(), overwrite_skip_and_rename(), pack_tree() (+9 more)

### Community 77 - "copy_limited"
Cohesion: 0.17
Nodes (8): archive_packed_len(), Budget, BudgetWriter, BudgetWriter<'_, W>, copy_limited(), ratio_cap_rejects_a_member_that_expands_too_far(), ratio_error(), zero_ratio_disables_the_check()

### Community 78 - "app.rs"
Cohesion: 0.06
Nodes (34): clear_history_forgets_opened_archives_and_destinations(), decide_member(), find_stub(), folder_title(), forget_history(), is_sfx_name(), listed_effect(), ListedEffect (+26 more)

### Community 79 - "folder_formats.rs"
Cohesion: 0.35
Nodes (12): assert_tree(), Case, cases(), extract_all(), nested_folder_round_trips_every_writable_format(), opts(), parallel_zip_round_trips_a_folder_of_spilled_files(), planned_names() (+4 more)

### Community 80 - "TestReport"
Cohesion: 0.18
Nodes (8): TestReport, bad_names(), Results, summary_line(), TestCommand, Cancel, Close, TestDialog

### Community 81 - "Password"
Cohesion: 0.17
Nodes (4): debug_hides_the_password(), Password, list_archive(), needs_password()

### Community 83 - "rename_dialog.rs"
Cohesion: 0.18
Nodes (8): reject_new_name(), RenameAction, Cancel, Submit, RenameDialog, RenameTarget, Disk, Member

### Community 84 - "Commands"
Cohesion: 0.20
Nodes (10): `a` — add, Commands, `complete` — shell completions, `d` — delete, `l` / `lt` — list, `p` — print, `s` — self-extractor, `t` — test (+2 more)

### Community 85 - "tests/extract.rs"
Cohesion: 0.33
Nodes (9): built_stub_contains_the_shellzip_icon(), encrypted_sfx(), silent_encrypted_stub_fails_without_a_password(), silent_encrypted_stub_rejects_a_wrong_password(), stub_bin(), stub_extracts_a_header_encrypted_7z_with_the_password(), stub_extracts_an_encrypted_zip_when_the_password_is_set(), stub_extracts_its_appended_7z() (+1 more)

### Community 86 - "ShellZip desktop app"
Cohesion: 0.20
Nodes (10): About, Adding files, Extracting, Limitations, Linux desktop entry, Opening archives, Options, Other commands (+2 more)

### Community 87 - "theme.rs"
Cohesion: 0.23
Nodes (8): apply(), apply_installs_both_themes(), DARK, LIGHT, Palette, palette(), RUST, visuals()

### Community 88 - "rename_summary.rs"
Cohesion: 0.12
Nodes (9): Cancel, fingerprint(), names(), rename_cancel_leaves_the_archive_unchanged(), rename_directory_rewrites_children(), rename_rejects_an_existing_destination(), rename_rejects_parent_directory_names(), stored_zip() (+1 more)

### Community 89 - "chrome.rs"
Cohesion: 0.11
Nodes (26): address_bar(), bar_frame(), caret_button(), confirm_delete(), DeleteChoice, Cancel, Delete, Wait (+18 more)

### Community 90 - "ShellZip"
Cohesion: 0.20
Nodes (10): Build and run, Command line, Crates, Desktop app, Documentation, Features, Format support, Licence (+2 more)

### Community 91 - "view.rs"
Cohesion: 0.23
Nodes (11): format_crc(), cell_text(), crc_text(), header_clicked(), modified_text(), packed_cell(), packed_text(), paint_header_cell() (+3 more)

### Community 92 - "CompressOpts"
Cohesion: 0.12
Nodes (9): ArchiveWriter, create(), create_from_path(), prepare_output(), prepare_temp(), rewrite_renamed(), temp_beside(), rewrite_renamed() (+1 more)

### Community 93 - "SetupState"
Cohesion: 0.28
Nodes (5): destroy_icon(), install_icons(), load_icon(), percent(), SetupState

### Community 94 - "pathbuf"
Cohesion: 0.38
Nodes (4): main(), windows_sdk_bin(), main(), windows_sdk_bin()

### Community 95 - "about.rs"
Cohesion: 0.16
Nodes (11): BANNER, banner_fills_the_dialog_width(), banner_size(), DIALOG_WIDTH, identity(), license(), MIT_LICENSE, notices() (+3 more)

### Community 96 - ".on_menu"
Cohesion: 0.16
Nodes (5): Place, Archive, Folder, report_row(), path_under()

### Community 97 - "szip command-line reference"
Cohesion: 0.29
Nodes (7): Compression levels, Examples, Exit codes, Global options, Logging, Password and header encryption, szip command-line reference

### Community 98 - "ReportFormat"
Cohesion: 0.18
Nodes (14): browse_report(), default_report_path(), extension(), help_window(), ReportAction, Close, Save, Wrote (+6 more)

### Community 99 - "extractor/sevenz/mod.rs"
Cohesion: 0.14
Nodes (14): mark_extracted(), restore_entry_time(), finish_staged(), copy_sevenz_index(), extract_sevenz(), ExtractOne, ProgressWrite<'_, W>, Tee (+6 more)

### Community 100 - "ShellZip documentation"
Cohesion: 0.67
Nodes (3): Building and internals, ShellZip documentation, User guides

### Community 101 - "Shared"
Cohesion: 0.27
Nodes (7): DeleteArgs, PrintArgs, TestArgs, delete(), Shared, print_entry(), test()

### Community 102 - "list.rs"
Cohesion: 0.27
Nodes (7): ArchiveArgs, ListArgs, display_name(), flags_text(), list(), list_technical(), format_blake()

### Community 103 - "ToolbarSize"
Cohesion: 0.22
Nodes (3): ToolbarSize, ExtraLarge, Large

### Community 104 - "commands/sfx.rs"
Cohesion: 0.46
Nodes (7): SfxArgs, default_output(), existing(), find_stub(), has_trailer(), sfx(), stub_file_names()

### Community 106 - "Credit"
Cohesion: 0.50
Nodes (7): assert_credits(), Credit, credits(), every_platform_lists_packaged_licenses(), LINUX, MACOS, WINDOWS

### Community 108 - "ReportSort"
Cohesion: 0.25
Nodes (8): report_sort_name(), ReportSort, Crc, Modified, Name, Original, Packed, Size

### Community 110 - "Column"
Cohesion: 0.33
Nodes (6): Column, Crc, Modified, Name, Packed, Size

### Community 112 - "Overwrite"
Cohesion: 0.40
Nodes (5): Overwrite, Always, Ask, Never, Rename

### Community 113 - "appearance.rs"
Cohesion: 0.60
Nodes (3): directories_use_the_yellow_folder(), row_glyph(), selected_glyphs_use_the_selection_ink()

### Community 114 - "MemberAction"
Cohesion: 0.50
Nodes (4): MemberAction, Drop, Rename, Write

## Knowledge Gaps
- **273 isolated node(s):** `shellzip-cli`, `Add`, `Update`, `Freshen`, `Delete` (+268 more)
  These have ≤1 connection - possible missing edges or undocumented components. (Counts symbols only; 780 node(s) total have ≤1 connection when file, concept and rationale nodes are included.)
- **12 thin communities (<3 nodes) omitted from report** — run `graphify query` to explore isolated nodes.

## Suggested Questions
_Questions this graph is uniquely positioned to answer:_

- **Why does `Error` connect `Error` to `worker.rs`, `ops.rs`, `policy.rs`, `src/sfx.rs`, `Password`, `stream.rs`, `shellzip-sfx/src/main.rs`, `setup.rs`, `archiver/zip.rs`, `parallel.rs`, `walk.rs`, `extractor/tar.rs`, `Entry`, `extract.rs`, `src/cli.rs`, `view_dialog.rs`, `archiver/tar.rs`, `Format`, `prompt.rs`, `extractor/zip.rs`, `Password`, `summary.rs`, `EntryMeta`, `milestone3.rs`, `path`, `round_trip.rs`, `app.rs`, `folder_formats.rs`, `NopProgress`, `rename_summary.rs`, `CompressOpts`, `extractor/sevenz/mod.rs`, `Shared`, `list.rs`, `commands/sfx.rs`, `sevenz/options.rs`, `write`?**
  _High betweenness centrality (0.087) - this node is a cross-community bridge._
- **What connects `shellzip-cli`, `Add`, `Update` to the rest of the system?**
  _273 weakly-connected nodes found - possible documentation gaps or missing edges._
- **Should `worker.rs` be split into smaller, more focused modules?**
  _Cohesion score 0.060285563194077206 - nodes in this community are weakly interconnected._
- **Why does `ShellZipApp` connect `ShellZipApp` to `worker.rs`, `report_dialog.rs`, `model.rs`, `view_dialog.rs`, `MenuCommand`, `shell_icons/mod.rs`, `settings.rs`, `info_dialog.rs`, `tree.rs`, `.poll`, `Explorer`, `add_dialog.rs`, `extract_dialog.rs`, `app.rs`, `TestReport`, `Password`, `rename_dialog.rs`, `.on_menu`, `ToolbarSize`?**
  _High betweenness centrality (0.070) - this node is a cross-community bridge._
- **Should `icons.rs` be split into smaller, more focused modules?**
  _Cohesion score 0.14393939393939395 - nodes in this community are weakly interconnected._
- **Why does `Password` connect `Password` to `ops.rs`, `src/sfx.rs`, `Password`, `shellzip-sfx/src/main.rs`, `setup.rs`, `Entry`, `ShellZipApp`, `src/cli.rs`, `view_dialog.rs`, `extractor/zip.rs`, `Password`, `run`, `sevenz/entry.rs`, `app.rs`, `CompressOpts`, `SetupState`, `extractor/sevenz/mod.rs`, `Shared`, `list.rs`?**
  _High betweenness centrality (0.062) - this node is a cross-community bridge._
- **Should `ops.rs` be split into smaller, more focused modules?**
  _Cohesion score 0.12307692307692308 - nodes in this community are weakly interconnected._