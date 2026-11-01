# Graph Report - ShellZip  (2026-11-01)

## Corpus Check
- 112 files · ~245,817 words
- Verdict: corpus is large enough that graph structure adds value.
- Unclassified: 32 file(s) not represented in the graph (top: .mdc 11, .rar 8, (none) 5)

## Summary
- 2517 nodes · 6079 edges · 107 communities (95 shown, 12 thin omitted)
- Extraction: 97% EXTRACTED · 3% INFERRED · 0% AMBIGUOUS · INFERRED: 164 edges (avg confidence: 0.85)
- Token cost: 0 input · 0 output

## Graph Freshness
- Built from commit: `2da48d16`
- Run `git rev-parse HEAD` and compare to check if the graph is stale.
- Run `graphify update .` after code changes (no API cost).

## Community Hubs (Navigation)
- worker.rs
- Color32
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
- ProgressSink
- report_dialog.rs
- extractor/tar.rs
- Entry
- Error
- extract.rs
- Scaffold templates
- ShellZipApp
- src/cli.rs
- tree.rs
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
- FolderTree
- Outcome
- extractor/zip.rs
- add.rs
- docs/README.md
- view.rs
- Password
- ToolCommand
- tests/cli.rs
- NodeId
- TreeFocus
- icons.rs
- HWND
- OpKind
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
- ExtractUpdate
- pathbuf
- copy_limited
- app.rs
- folder_formats.rs
- TestReport
- ShellZipApp
- NopProgress
- rename_dialog.rs
- Commands
- tests/extract.rs
- ShellZip desktop app
- theme.rs
- rename_summary.rs
- chrome.rs
- ShellZip
- tinted
- CompressOpts
- SetupState
- PreviewBody
- super
- szip command-line reference
- extractor/sevenz/mod.rs
- ShellZip documentation
- test.rs
- ToolbarSize
- commands/sfx.rs
- write
- Overwrite
- appearance.rs

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
- `zip_comment()` --calls--> `detect_file()`  [INFERRED]
  crates/shellzip-core/src/ops.rs → crates/shellzip-core/src/detect.rs
- `compress_opts()` --calls--> `format_from_extension()`  [INFERRED]
  crates/shellzip-cli/src/cli.rs → crates/shellzip-core/src/detect.rs
- `add()` --calls--> `pack()`  [INFERRED]
  crates/shellzip-cli/src/commands/add.rs → crates/shellzip-gui/src/add_dialog.rs
- `flags_text()` --references--> `Entry`  [EXTRACTED]
  crates/shellzip-cli/src/commands/list.rs → crates/shellzip-core/src/entry.rs
- `sfx()` --calls--> `convert()`  [INFERRED]
  crates/shellzip-cli/src/commands/sfx.rs → crates/shellzip-core/src/sfx.rs

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

## Communities (107 total, 12 thin omitted)

### Community 0 - "worker.rs"
Cohesion: 0.07
Nodes (27): bar(), bytes_line(), format_speed(), fraction(), job_fraction(), ProgressCommand, Background, Cancel (+19 more)

### Community 1 - "Color32"
Cohesion: 0.29
Nodes (10): Caret, caret(), Down, Up, paint(), paint_caret(), paint_expander(), paint_tool() (+2 more)

### Community 2 - "ops.rs"
Cohesion: 0.09
Nodes (36): member_key(), add(), add_inner(), add_prompted(), add_sfx(), add_stdin(), build_globs(), checked_member_name() (+28 more)

### Community 3 - "policy.rs"
Cohesion: 0.11
Nodes (34): admit_existing(), archive_is_newer(), BudgetWriter<'_, W>, Destination, destination(), Skip, Write, disk_mtime() (+26 more)

### Community 4 - "src/sfx.rs"
Cohesion: 0.07
Nodes (40): AES_METHOD, decoder_rejected(), explain_open(), header_uses_aes(), MAX_MEMORY, open_streaming(), apply_delta(), build() (+32 more)

### Community 5 - "Password"
Cohesion: 0.07
Nodes (28): archive_at(), copy_rar_index(), copy_staged(), DECODE_IN_MEMORY, dos_time(), entry_from(), extract_rar(), first_volume() (+20 more)

### Community 6 - "stream.rs"
Cohesion: 0.12
Nodes (8): Kind, Gzip, Zstd, SingleArchiver, Stream, Gzip, Idle, Zstd

### Community 7 - "shellzip-sfx/src/main.rs"
Cohesion: 0.07
Nodes (20): Ask, Bars, byte_style(), sink(), ConflictChoice, Overwrite, Rename, Skip (+12 more)

### Community 8 - "ShellZip FAQ"
Cohesion: 0.09
Nodes (22): Can I pipe data in or out?, Can I update or delete files inside an existing archive?, Does ShellZip require `unsafe`?, How do I make a self-extracting archive?, How do I pass a password without putting it on the command line?, How do I run the tests?, How do threads work?, How is this different from 7-Zip or WinRAR? (+14 more)

### Community 9 - "setup.rs"
Cohesion: 0.05
Nodes (13): CB_ADDSTRING, ID_BROWSE, ID_DEST, ID_INTRO, ID_PROGRESS, INTRO, PBM_SETBARCOLOR, PBM_SETPOS (+5 more)

### Community 10 - "archiver/zip.rs"
Cohesion: 0.16
Nodes (12): commit_output(), apply_comment(), copy_renamed(), method_for(), options_for(), rel_name(), rewrite_renamed(), target_text() (+4 more)

### Community 11 - "parallel.rs"
Cohesion: 0.18
Nodes (10): copy_in(), compress_file(), MEMORY_LIMIT, merge(), Piece, Memory, piece_path(), Temp (+2 more)

### Community 12 - "Icon"
Cohesion: 0.08
Nodes (23): Icon, Archive, ArrowUp, Check, ChevronDown, CircleCheck, CircleHelp, File (+15 more)

### Community 13 - "walk.rs"
Cohesion: 0.06
Nodes (29): ArchiveArgs, ListArgs, display_name(), flags_text(), list(), list_technical(), EntryKind, Dir (+21 more)

### Community 14 - "ProgressSink"
Cohesion: 0.08
Nodes (17): count_kept(), keep(), keep_mapped(), mapped_path(), meta_for(), test_sevenz(), test(), test_with() (+9 more)

### Community 15 - "report_dialog.rs"
Cohesion: 0.07
Nodes (58): format_time(), archive(), browse_report(), cmp_opt(), Column, Crc, Modified, Name (+50 more)

### Community 16 - "extractor/tar.rs"
Cohesion: 0.12
Nodes (29): Compression, Bzip2, Gzip, None, Xz, Zstd, copy_limited(), copy_single() (+21 more)

### Community 17 - "Entry"
Cohesion: 0.12
Nodes (13): Entry, ArchiveReader, only_first(), Opened, resolve(), unsupported(), Verify, list() (+5 more)

### Community 18 - "Error"
Cohesion: 0.19
Nodes (14): Error, Archive, BadChecksum, BadPassword, Encrypted, Interrupted, Io, Memory (+6 more)

### Community 19 - "extract.rs"
Cohesion: 0.24
Nodes (9): ExtractArgs, Overwrite, overwrite_mode(), OverwriteArg, Always, Ask, Rename, Skip (+1 more)

### Community 20 - "Scaffold templates"
Cohesion: 0.10
Nodes (19): Criterion benchmark, Example, Extra binary in an existing crate, Finish checklist, Integration test file, New module in an existing crate, New `szip` subcommand, New workspace crate (+11 more)

### Community 22 - "src/cli.rs"
Cohesion: 0.12
Nodes (11): Cli, dispatch(), init_tracing(), normalize_args(), password_from_env(), prompt_password(), run(), SfxFormatArg (+3 more)

### Community 23 - "tree.rs"
Cohesion: 0.15
Nodes (22): best_volume(), cmp_name(), component_len(), drive_root(), drive_root_is_the_filesystem_root(), extend_from(), file_label(), icon_for() (+14 more)

### Community 24 - "Command"
Cohesion: 0.14
Nodes (13): Command, Add, Complete, Delete, Extract, ExtractFlat, Freshen, List (+5 more)

### Community 25 - "xtask/src/main.rs"
Cohesion: 0.15
Nodes (23): bench(), build_fixtures(), build_sevenz_fixtures(), collect_files(), crc32(), find_rar(), find_tool(), FIXTURE_PASSWORD (+15 more)

### Community 26 - "shellzip-gui/src/main.rs"
Cohesion: 0.24
Nodes (7): anstream_fallback(), clear_white_margin(), is_white_margin(), main(), pixel_index(), window_icon(), window_icon_is_square_and_its_corners_are_clear()

### Community 27 - "shellzip-core"
Cohesion: 0.40
Nodes (5): shellzip-cli, shellzip-core, shellzip-gui, shellzip-sfx, xtask

### Community 29 - "model.rs"
Cohesion: 0.06
Nodes (45): archive_prefix(), ArchiveIndex, byte_phrase(), compare_column(), compare_name(), compare_option(), dir_partial(), directory_marker_hides_its_own_size() (+37 more)

### Community 30 - "milestone2.rs"
Cohesion: 0.24
Nodes (11): ask_can_skip_replace_or_rename(), freshen_missing_archive_is_not_found(), opts(), parallel_zip_matches_single_thread_bytes(), skip_leaves_existing_members_and_adds_new_ones(), stdin_streams_zst_and_spools_tar(), sync_drops_members_that_were_not_selected(), technical_list_fills_blake2sp() (+3 more)

### Community 31 - "view_dialog.rs"
Cohesion: 0.16
Nodes (12): bytes_len(), cap_writer_stops_at_the_limit(), CapWriter, PREVIEW_CAP, PreviewData, read_disk(), read_disk_stops_at_the_cap(), read_limited() (+4 more)

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
Cohesion: 0.07
Nodes (49): caches_a_raster_from_the_provider(), decode_png(), extension_of(), icon_keys_follow_extension_path_and_folder(), IconCache, IconKey, Computer, Extension (+41 more)

### Community 41 - "settings.rs"
Cohesion: 0.14
Nodes (23): defaults(), delete_name(), DeleteArchive, Always, Ask, Never, ExtractSettings, load() (+15 more)

### Community 42 - "Rust quality gate"
Cohesion: 0.29
Nodes (6): Fix clippy, don't silence it, Fix tests, Optional supply-chain checks, Report, Rust quality gate, Workflow

### Community 43 - "setup_template"
Cohesion: 0.26
Nodes (14): align4(), Control, dword_words(), dword_words_keep_the_template_bytes(), init_common_controls(), push_atom(), push_named(), push_u16() (+6 more)

### Community 44 - "info_dialog.rs"
Cohesion: 0.16
Nodes (20): archive_summary(), ArchiveSummary, comment_tab(), DIALOG_WIDTH, file_name(), help_window(), info_rows(), info_tab() (+12 more)

### Community 45 - "Format"
Cohesion: 0.06
Nodes (51): rewrite_renamed(), CreatableFormat, detect_bytes(), detect_file(), detect_header(), elf_stub_with_trailer_is_sfx(), find_embedded(), find_subslice() (+43 more)

### Community 46 - "prompt.rs"
Cohesion: 0.07
Nodes (49): accept_password(), ask(), ask_console(), ask_gui(), attach_console(), attach_parent(), Choice, All (+41 more)

### Community 47 - "FolderTree"
Cohesion: 0.12
Nodes (15): computer_label(), FolderTree, Job, Dirs, Volumes, KnownFolder, Msg, Dirs (+7 more)

### Community 48 - "Outcome"
Cohesion: 0.11
Nodes (13): preview_outcome(), JobToken, Outcome, Directory, Extracted, Failed, Finished, Listed (+5 more)

### Community 49 - "extractor/zip.rs"
Cohesion: 0.14
Nodes (11): annotate(), entry_from(), extract_zip_index(), hash_capped(), host_label(), list_zip(), method_name(), open_zip() (+3 more)

### Community 50 - "add.rs"
Cohesion: 0.09
Nodes (24): AddArgs, compress_opts(), DeleteArgs, MethodArg, Bzip2, Deflate, Lzma, Lzma2 (+16 more)

### Community 52 - "view.rs"
Cohesion: 0.05
Nodes (42): format_crc(), BANNER, banner_fills_the_dialog_width(), banner_size(), DIALOG_WIDTH, identity(), license(), MIT_LICENSE (+34 more)

### Community 54 - "Password"
Cohesion: 0.08
Nodes (26): add(), add_prompted(), add_sfx(), add_stdin(), delete(), list_with(), rename(), write_member() (+18 more)

### Community 55 - "ToolCommand"
Cohesion: 0.20
Nodes (10): ToolCommand, About, Add, Delete, Extract, Info, Open, Sfx (+2 more)

### Community 56 - "tests/cli.rs"
Cohesion: 0.14
Nodes (22): add_list_test_extract_round_trip(), add_sfx_writes_a_listable_package(), add_to_an_existing_zip_keeps_previous_files(), help_explains_that_zip_encryption_leaves_names_visible(), log_file_receives_debug_events(), max_size_stops_extract_and_test(), rar_is_not_created(), reads_a_tar_from_the_system_tool() (+14 more)

### Community 57 - "NodeId"
Cohesion: 0.20
Nodes (14): Builder, NodeId, Archive, Computer, Desktop, Dir, Home, Libraries (+6 more)

### Community 58 - "TreeFocus"
Cohesion: 0.19
Nodes (15): archive_prefixes_hang_under_the_archive(), expands_home_and_lists_siblings(), expands_to_the_current_folder_and_lists_siblings(), failed_spawn_does_not_queue_folder_reads(), host_namespace(), layout_rows(), LayoutInput, lists_known_folders_before_drives_and_shell_places() (+7 more)

### Community 60 - "icons.rs"
Cohesion: 0.15
Nodes (9): ALL, dark_toolbar_strokes_are_light(), MASTER_POINTS, recolor_dark(), SIZE_CARET, SIZE_ROW, SIZE_STATUS, tool_bytes() (+1 more)

### Community 61 - "HWND"
Cohesion: 0.21
Nodes (19): browse_initialized(), enable_window(), EnableWindow(), finish_if_ready(), on_browse(), on_command(), on_extract(), paint_banner() (+11 more)

### Community 62 - "OpKind"
Cohesion: 0.18
Nodes (12): listed_effect(), ListedEffect, Browse, Finish, Operation, OpKind, Add, Delete (+4 more)

### Community 65 - "prepare_window"
Cohesion: 0.32
Nodes (8): add_choice(), browse_folder(), prepare_window(), set_text(), set_window_text(), style_progress(), wide_os(), wide_str()

### Community 66 - "sevenz/entry.rs"
Cohesion: 0.10
Nodes (18): coder_dictionary(), coder_label(), entry_from(), entry_key(), file(), folder_dictionary(), folder_label(), host_os() (+10 more)

### Community 67 - "EntryMeta"
Cohesion: 0.12
Nodes (8): archive_path(), CloneMeta, EntryMeta, seven_meta(), SevenZArchiver, spool(), Spooled, write_file_body()

### Community 68 - "milestone3.rs"
Cohesion: 0.17
Nodes (16): assert_matches_unrar(), encrypting_an_existing_zip_requires_force(), extract_bytes(), find_unrar(), FIXTURE_PASSWORD, fixtures_root(), password(), passwords_are_rejected_for_tar() (+8 more)

### Community 69 - "paint_banner"
Cohesion: 0.47
Nodes (4): blit_banner(), DrawItem, fill_band(), fitted()

### Community 70 - "ToolIcon"
Cohesion: 0.12
Nodes (14): TOOL_ALL, toolbar_svgs_carry_the_brand_colors(), ToolIcon, About, Add, Delete, Extract, Info (+6 more)

### Community 71 - "add_dialog.rs"
Cohesion: 0.07
Nodes (51): creatable_formats(), ZipMethod, Deflate, Zstd, AddAction, Close, Submit, AddDialog (+43 more)

### Community 72 - "extract_dialog.rs"
Cohesion: 0.12
Nodes (33): advanced_tab(), begin_new_folder(), colors(), confirm_delete_archive(), confirm_new_folder(), destination_uses_the_archive_name_without_its_extension(), DIALOG_WIDTH, display_path() (+25 more)

### Community 73 - "sevenz.rs"
Cohesion: 0.13
Nodes (19): codec_method(), write_options(), add_lists_and_extracts_through_the_core(), aes_hides_bytes_and_header_encryption_hides_names(), bzip2_and_deflate_round_trip(), committed_sevenz_fixtures_list_when_present(), extract_bytes(), fast() (+11 more)

### Community 74 - "Archive formats"
Cohesion: 0.18
Nodes (11): 7z, Archive formats, Capability matrix, Creation methods, Detection versus extension, Encryption, Feature flags (`shellzip-core`), Safety (+3 more)

### Community 75 - "ExtractUpdate"
Cohesion: 0.20
Nodes (10): ExtractUpdate, Freshen, Replace, Update, ExtractAction, Close, Save, Submit (+2 more)

### Community 76 - "pathbuf"
Cohesion: 0.09
Nodes (31): copy_zone(), MAX_ZONE_BYTES, propagate(), zone_path(), adding_to_an_existing_zip_keeps_the_other_files(), byte_cap_stops_extraction(), existing_tar_is_left_in_place_without_force(), extract_all() (+23 more)

### Community 77 - "copy_limited"
Cohesion: 0.22
Nodes (6): archive_packed_len(), Budget, BudgetWriter, ratio_cap_rejects_a_member_that_expands_too_far(), ratio_error(), zero_ratio_disables_the_check()

### Community 78 - "app.rs"
Cohesion: 0.07
Nodes (25): clear_history_forgets_opened_archives_and_destinations(), decide_member(), folder_still_shown(), folder_title(), forget_history(), is_sfx_name(), PasswordAsk, PendingArchiveDelete (+17 more)

### Community 79 - "folder_formats.rs"
Cohesion: 0.35
Nodes (12): assert_tree(), Case, cases(), extract_all(), nested_folder_round_trips_every_writable_format(), opts(), parallel_zip_round_trips_a_folder_of_spilled_files(), planned_names() (+4 more)

### Community 80 - "TestReport"
Cohesion: 0.29
Nodes (5): TestReport, bad_names(), Results, summary_line(), TestDialog

### Community 81 - "ShellZipApp"
Cohesion: 0.13
Nodes (5): Password, find_stub(), list_archive(), needs_password(), ShellZipApp

### Community 83 - "rename_dialog.rs"
Cohesion: 0.18
Nodes (8): reject_new_name(), RenameAction, Cancel, Submit, RenameDialog, RenameTarget, Disk, Member

### Community 84 - "Commands"
Cohesion: 0.20
Nodes (10): `a` — add, Commands, `complete` — shell completions, `d` — delete, `l` / `lt` — list, `p` — print, `s` — self-extractor, `t` — test (+2 more)

### Community 85 - "tests/extract.rs"
Cohesion: 0.31
Nodes (10): built_stub_contains_the_shellzip_icon(), encrypted_sfx(), run_stub(), silent_encrypted_stub_fails_without_a_password(), silent_encrypted_stub_rejects_a_wrong_password(), stub_bin(), stub_extracts_a_header_encrypted_7z_with_the_password(), stub_extracts_an_encrypted_zip_when_the_password_is_set() (+2 more)

### Community 86 - "ShellZip desktop app"
Cohesion: 0.20
Nodes (10): About, Adding files, Extracting, Limitations, Linux desktop entry, Opening archives, Options, Other commands (+2 more)

### Community 87 - "theme.rs"
Cohesion: 0.23
Nodes (8): apply(), apply_installs_both_themes(), DARK, LIGHT, Palette, palette(), RUST, visuals()

### Community 88 - "rename_summary.rs"
Cohesion: 0.13
Nodes (9): Cancel, fingerprint(), names(), rename_cancel_leaves_the_archive_unchanged(), rename_directory_rewrites_children(), rename_rejects_an_existing_destination(), rename_rejects_parent_directory_names(), stored_zip() (+1 more)

### Community 89 - "chrome.rs"
Cohesion: 0.09
Nodes (37): address_bar(), AddressEvent, Jump, Refresh, Submit, Up, bar_frame(), breadcrumbs() (+29 more)

### Community 90 - "ShellZip"
Cohesion: 0.20
Nodes (10): Build and run, Command line, Crates, Desktop app, Documentation, Features, Format support, Licence (+2 more)

### Community 91 - "tinted"
Cohesion: 0.50
Nodes (7): image(), load_bytes(), sized_texture(), tinted(), tool_image(), tool_texture(), warm()

### Community 92 - "CompressOpts"
Cohesion: 0.11
Nodes (12): ArchiveWriter, create(), create_from_path(), prepare_output(), prepare_temp(), temp_beside(), add_zip(), rewrite_renamed() (+4 more)

### Community 93 - "SetupState"
Cohesion: 0.14
Nodes (11): Banner, banner_pixels_are_bgra(), ComInit, decode_banner(), destroy_icon(), finish_worker(), install_icons(), load_icon() (+3 more)

### Community 94 - "PreviewBody"
Cohesion: 0.32
Nodes (5): classify(), PreviewBody, Binary, Text, ViewDialog

### Community 97 - "szip command-line reference"
Cohesion: 0.29
Nodes (7): Compression levels, Examples, Exit codes, Global options, Logging, Password and header encryption, szip command-line reference

### Community 99 - "extractor/sevenz/mod.rs"
Cohesion: 0.21
Nodes (12): mark_extracted(), restore_entry_time(), finish_staged(), Member, extract_sevenz(), ExtractOne, ProgressWrite, write_member() (+4 more)

### Community 100 - "ShellZip documentation"
Cohesion: 0.67
Nodes (3): Building and internals, ShellZip documentation, User guides

### Community 101 - "test.rs"
Cohesion: 0.33
Nodes (3): LimitArgs, TestArgs, test()

### Community 103 - "ToolbarSize"
Cohesion: 0.25
Nodes (3): ToolbarSize, ExtraLarge, Large

### Community 104 - "commands/sfx.rs"
Cohesion: 0.46
Nodes (7): SfxArgs, default_output(), existing(), find_stub(), has_trailer(), sfx(), stub_file_names()

### Community 112 - "Overwrite"
Cohesion: 0.33
Nodes (6): Overwrite, Always, Ask, Never, Rename, overwrite_name()

### Community 113 - "appearance.rs"
Cohesion: 0.60
Nodes (3): directories_use_the_yellow_folder(), row_glyph(), selected_glyphs_use_the_selection_ink()

## Knowledge Gaps
- **273 isolated node(s):** `shellzip-cli`, `Add`, `Update`, `Freshen`, `Delete` (+268 more)
  These have ≤1 connection - possible missing edges or undocumented components. (Counts symbols only; 779 node(s) total have ≤1 connection when file, concept and rationale nodes are included.)
- **12 thin communities (<3 nodes) omitted from report** — run `graphify query` to explore isolated nodes.

## Suggested Questions
_Questions this graph is uniquely positioned to answer:_

- **Why does `Error` connect `Error` to `worker.rs`, `ops.rs`, `policy.rs`, `src/sfx.rs`, `Password`, `stream.rs`, `shellzip-sfx/src/main.rs`, `setup.rs`, `archiver/zip.rs`, `parallel.rs`, `walk.rs`, `ProgressSink`, `extractor/tar.rs`, `Entry`, `extract.rs`, `src/cli.rs`, `view_dialog.rs`, `archiver/tar.rs`, `Format`, `prompt.rs`, `extractor/zip.rs`, `add.rs`, `Password`, `EntryMeta`, `milestone3.rs`, `sevenz.rs`, `pathbuf`, `app.rs`, `folder_formats.rs`, `rename_summary.rs`, `CompressOpts`, `extractor/sevenz/mod.rs`, `test.rs`, `commands/sfx.rs`, `write`?**
  _High betweenness centrality (0.076) - this node is a cross-community bridge._
- **What connects `shellzip-cli`, `Add`, `Update` to the rest of the system?**
  _273 weakly-connected nodes found - possible documentation gaps or missing edges._
- **Should `worker.rs` be split into smaller, more focused modules?**
  _Cohesion score 0.0726764500349406 - nodes in this community are weakly interconnected._
- **Why does `Password` connect `ShellZipApp` to `ops.rs`, `src/sfx.rs`, `Password`, `shellzip-sfx/src/main.rs`, `setup.rs`, `walk.rs`, `ProgressSink`, `Entry`, `src/cli.rs`, `view_dialog.rs`, `extractor/zip.rs`, `add.rs`, `Password`, `sevenz/entry.rs`, `app.rs`, `CompressOpts`, `SetupState`, `super`, `.fmt`, `extractor/sevenz/mod.rs`?**
  _High betweenness centrality (0.069) - this node is a cross-community bridge._
- **Should `ops.rs` be split into smaller, more focused modules?**
  _Cohesion score 0.09082125603864734 - nodes in this community are weakly interconnected._
- **Why does `ShellZipApp` connect `ShellZipApp` to `worker.rs`, `report_dialog.rs`, `ShellZipApp`, `model.rs`, `MenuCommand`, `shell_icons/mod.rs`, `settings.rs`, `info_dialog.rs`, `FolderTree`, `Outcome`, `view.rs`, `OpKind`, `add_dialog.rs`, `extract_dialog.rs`, `app.rs`, `TestReport`, `rename_dialog.rs`, `PreviewBody`, `ToolbarSize`?**
  _High betweenness centrality (0.065) - this node is a cross-community bridge._
- **Should `policy.rs` be split into smaller, more focused modules?**
  _Cohesion score 0.10676532769556026 - nodes in this community are weakly interconnected._