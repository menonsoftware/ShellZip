# Graph Report - ShellZip  (2026-10-25)

## Corpus Check
- 88 files · ~69,645 words
- Verdict: corpus is large enough that graph structure adds value.
- Unclassified: 26 file(s) not represented in the graph (top: .mdc 11, .rar 8, (none) 4)

## Summary
- 1668 nodes · 3933 edges · 85 communities (69 shown, 16 thin omitted)
- Extraction: 97% EXTRACTED · 3% INFERRED · 0% AMBIGUOUS · INFERRED: 123 edges (avg confidence: 0.85)
- Token cost: 0 input · 0 output

## Graph Freshness
- Built from commit: `b1c95329`
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
- MethodArg
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
- CompressOpts
- app.rs
- Test patterns
- 4. Phased implementation (Cursor task list)
- CompressOpts
- Tier 1 — What can be proven (direct evidence)
- Rust review checklist
- archiver/tar.rs
- extractor/zip.rs
- Rust performance
- stream_one
- delete.rs
- Rust quality gate
- Shared
- milestone3.rs
- MenuCommand
- prompt.rs
- Icon
- listing.rs
- Password
- write
- BENCH.md
- Explorer
- setup_template
- ToolCommand
- RowKind
- SfxFormatArg
- path
- milestone2.rs
- HWND
- SetupState
- BarProgress
- copy_chunked
- prepare_window
- sevenz/entry.rs
- EntryMeta
- tests/cli.rs
- DrawItem
- ExtractOpts
- add_dialog.rs
- .go_up
- sevenz.rs
- folder_formats.rs
- tests/extract.rs
- zip_threads.rs
- Opened
- UpdateMode
- NopProgress
- run
- sevenz/options.rs
- ArchiveReader
- Budget
- super

## God Nodes (most connected - your core abstractions)
1. `ShellZipApp` - 67 edges
2. `Password` - 66 edges
3. `CompressOpts` - 65 edges
4. `ProgressSink` - 64 edges
5. `Entry` - 43 edges
6. `EntryMeta` - 37 edges
7. `Format` - 36 edges
8. `ExtractOpts` - 29 edges
9. `Icon` - 28 edges
10. `Command` - 27 edges

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
- 1-file cycle: `crates/shellzip-sfx/src/setup.rs -> crates/shellzip-sfx/src/setup.rs`
- 1-file cycle: `crates/shellzip-gui/src/add_dialog.rs -> crates/shellzip-gui/src/add_dialog.rs`
- 1-file cycle: `crates/shellzip-gui/src/explorer/model.rs -> crates/shellzip-gui/src/explorer/model.rs`
- 1-file cycle: `crates/shellzip-gui/src/explorer/listing.rs -> crates/shellzip-gui/src/explorer/listing.rs`
- 1-file cycle: `crates/shellzip-gui/src/icons.rs -> crates/shellzip-gui/src/icons.rs`
- 2-file cycle: `crates/shellzip-core/src/archiver/mod.rs -> crates/shellzip-core/src/archiver/parallel.rs -> crates/shellzip-core/src/archiver/mod.rs`
- 2-file cycle: `crates/shellzip-cli/src/commands/mod.rs -> crates/shellzip-cli/src/commands/sfx.rs -> crates/shellzip-cli/src/commands/mod.rs`
- 2-file cycle: `crates/shellzip-cli/src/commands/delete.rs -> crates/shellzip-cli/src/commands/mod.rs -> crates/shellzip-cli/src/commands/delete.rs`
- 2-file cycle: `crates/shellzip-cli/src/commands/mod.rs -> crates/shellzip-cli/src/commands/print.rs -> crates/shellzip-cli/src/commands/mod.rs`
- 3-file cycle: `crates/shellzip-core/src/archiver/mod.rs -> crates/shellzip-core/src/archiver/parallel.rs -> crates/shellzip-core/src/archiver/zip.rs -> crates/shellzip-core/src/archiver/mod.rs`
- 3-file cycle: `crates/shellzip-cli/src/cli.rs -> crates/shellzip-cli/src/commands/mod.rs -> crates/shellzip-cli/src/commands/sfx.rs -> crates/shellzip-cli/src/cli.rs`
- 3-file cycle: `crates/shellzip-cli/src/cli.rs -> crates/shellzip-cli/src/commands/mod.rs -> crates/shellzip-cli/src/commands/list.rs -> crates/shellzip-cli/src/cli.rs`
- 3-file cycle: `crates/shellzip-cli/src/cli.rs -> crates/shellzip-cli/src/commands/mod.rs -> crates/shellzip-cli/src/commands/delete.rs -> crates/shellzip-cli/src/cli.rs`
- 3-file cycle: `crates/shellzip-cli/src/cli.rs -> crates/shellzip-cli/src/commands/mod.rs -> crates/shellzip-cli/src/commands/print.rs -> crates/shellzip-cli/src/cli.rs`

## Communities (85 total, 16 thin omitted)

### Community 0 - "worker.rs"
Cohesion: 0.07
Nodes (20): Overwrite, Always, Ask, Never, Rename, overwrite_name(), lock(), Outcome (+12 more)

### Community 1 - "Icon"
Cohesion: 0.17
Nodes (15): ALL, Caret, caret(), Down, Up, MASTER_POINTS, paint(), paint_caret() (+7 more)

### Community 2 - "ops.rs"
Cohesion: 0.13
Nodes (21): member_key(), build_globs(), Counting, delete(), extract(), filter_is_restrictive(), find_member(), fresh_name() (+13 more)

### Community 3 - "policy.rs"
Cohesion: 0.16
Nodes (20): Destination, destination(), Skip, Write, exists(), renamed(), symlink_destination(), ensure_dir() (+12 more)

### Community 4 - "src/sfx.rs"
Cohesion: 0.06
Nodes (44): SfxArgs, default_output(), existing(), find_stub(), has_trailer(), sfx(), AES_METHOD, decoder_rejected() (+36 more)

### Community 5 - "Password"
Cohesion: 0.15
Nodes (19): stage_path(), archive_at(), copy_rar_index(), copy_staged(), dos_time(), entry_from(), extract_rar(), first_volume() (+11 more)

### Community 6 - "stream.rs"
Cohesion: 0.11
Nodes (9): commit_output(), Kind, Gzip, Zstd, SingleArchiver, Stream, Gzip, Idle (+1 more)

### Community 7 - "shellzip-sfx/src/main.rs"
Cohesion: 0.07
Nodes (20): Ask, Bars, byte_style(), sink(), ConflictChoice, Overwrite, Rename, Skip (+12 more)

### Community 8 - "walk.rs"
Cohesion: 0.07
Nodes (24): EntryKind, Dir, File, Other, Symlink, format_attrs(), capture(), mode_bits() (+16 more)

### Community 9 - "setup.rs"
Cohesion: 0.06
Nodes (11): CB_ADDSTRING, ID_BROWSE, ID_DEST, ID_INTRO, ID_PROGRESS, INTRO, PBM_SETPOS, PBM_SETRANGE32 (+3 more)

### Community 10 - "archiver/zip.rs"
Cohesion: 0.18
Nodes (8): apply_comment(), method_for(), options_for(), rel_name(), target_text(), ZipArchiver, dos_parts(), EntryMeta

### Community 12 - "CrcWriter"
Cohesion: 0.13
Nodes (4): Fanout, blake2sp_matches_the_reference_vector(), Blake2spWriter, CrcWriter

### Community 13 - "Format"
Cohesion: 0.11
Nodes (25): creatable_extensions_round_trip(), detect_bytes(), detect_file(), detect_header(), find_embedded(), find_subslice(), Format, Bzip2 (+17 more)

### Community 14 - "round_trip.rs"
Cohesion: 0.26
Nodes (16): adding_to_an_existing_zip_keeps_the_other_files(), byte_cap_stops_extraction(), existing_tar_is_left_in_place_without_force(), extract_all(), gzip_rejects_a_second_file(), opts_fast(), overwrite_skip_and_rename(), pack_tree() (+8 more)

### Community 15 - "list.rs"
Cohesion: 0.21
Nodes (8): ArchiveArgs, ListArgs, display_name(), flags_text(), list(), list_technical(), test(), format_blake()

### Community 16 - "extractor/tar.rs"
Cohesion: 0.12
Nodes (29): Compression, Bzip2, Gzip, None, Xz, Zstd, Entry, copy_single() (+21 more)

### Community 17 - "MethodArg"
Cohesion: 0.25
Nodes (7): MethodArg, Bzip2, Deflate, Lzma, Lzma2, Ppmd, Zstd

### Community 18 - "Error"
Cohesion: 0.21
Nodes (12): Error, Archive, BadChecksum, BadPassword, Encrypted, Interrupted, Io, NoFiles (+4 more)

### Community 19 - "extract.rs"
Cohesion: 0.22
Nodes (9): ExtractArgs, Overwrite, overwrite_mode(), OverwriteArg, Always, Ask, Rename, Skip (+1 more)

### Community 20 - "Scaffold templates"
Cohesion: 0.10
Nodes (19): Criterion benchmark, Example, Extra binary in an existing crate, Finish checklist, Integration test file, New module in an existing crate, New `szip` subcommand, New workspace crate (+11 more)

### Community 21 - "chrome.rs"
Cohesion: 0.13
Nodes (22): about(), address_bar(), AddressEvent, Jump, Submit, Up, bar_frame(), breadcrumbs() (+14 more)

### Community 22 - "src/cli.rs"
Cohesion: 0.15
Nodes (8): Cli, dispatch(), init_tracing(), normalize_args(), password_from_env(), prompt_password(), run(), take_password()

### Community 23 - "theme.rs"
Cohesion: 0.18
Nodes (16): ADD, apply(), CANVAS, DANGER, EXTRACT, FOLDER, HOVER, INK (+8 more)

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
Nodes (31): format_bytes(), format_time(), archive_prefix(), ArchiveIndex, byte_phrase(), compare_column(), compare_name(), compare_option() (+23 more)

### Community 30 - "CompressOpts"
Cohesion: 0.10
Nodes (20): CreatableFormat, add(), add_prompted(), add_sfx(), add_stdin(), delete(), write_member(), add() (+12 more)

### Community 31 - "app.rs"
Cohesion: 0.14
Nodes (11): decide_member(), find_stub(), folder_title(), list_archive(), needs_password(), PasswordAsk, Place, Archive (+3 more)

### Community 32 - "Test patterns"
Cohesion: 0.11
Nodes (17): CLI end-to-end with `assert_cmd` (`tests/cli.rs`), Counting fake for a trait, In-memory reader/writer, Integration round-trip (`tests/<scenario>.rs`), Interop with OS tools, Platform-gated test, Property test with `proptest`, Snapshot test with `insta` (+9 more)

### Community 33 - "4. Phased implementation (Cursor task list)"
Cohesion: 0.12
Nodes (15): 1. Scope & feature‑parity matrix, 2. Recommended crate ecosystem, 3. Project structure, 4. Phased implementation (Cursor task list), 5. Testing & verification strategy, 6. Licensing notes, Design principles carried over from WinRAR, Milestone 0 — Skeleton (½ day) (+7 more)

### Community 34 - "CompressOpts"
Cohesion: 0.21
Nodes (9): copy_in(), add_zip(), compress_file(), MEMORY_LIMIT, merge(), Piece, Memory, piece_path() (+1 more)

### Community 35 - "Tier 1 — What can be proven (direct evidence)"
Cohesion: 0.20
Nodes (9): 1.1 Directory inventory (32 files, all in one flat directory), 1.2 Separation of concerns, 1.3 Imported Windows APIs grouped by capability, 1.4 CLI surface (from `Rar.exe -?`, `UnRAR.exe`, `Rar.txt`), Evidence ledger (summary), Tier 1 — What can be proven (direct evidence), Tier 2 — What can be inferred (architectural deductions), Tier 3 — What remains hidden (opaque internals) (+1 more)

### Community 36 - "Rust review checklist"
Cohesion: 0.13
Nodes (13): 1. Correctness, 2. Error handling, 3. Security, 4. Ownership and performance, 5. Concurrency, 6. Tests, 7. API and docs, 8. Manifest and tooling (+5 more)

### Community 37 - "archiver/tar.rs"
Cohesion: 0.12
Nodes (8): CompWriter, Gzip, Plain, Zstd, dir_name(), header_for(), TarArchiver, unix_secs()

### Community 38 - "extractor/zip.rs"
Cohesion: 0.14
Nodes (13): partial_path(), write_atomic(), annotate(), entry_from(), extract_zip(), extract_zip_index(), host_label(), list_zip() (+5 more)

### Community 39 - "Rust performance"
Cohesion: 0.29
Nodes (6): 1. Baseline, 2. Find the hot spot, 3. Apply the cheapest fix the profile justifies, 4. Binary size, 5. Verify and report, Rust performance

### Community 40 - "stream_one"
Cohesion: 0.36
Nodes (5): add_stdin(), Counting<'_, R>, spool_stdin(), Spooled, stream_one()

### Community 42 - "Rust quality gate"
Cohesion: 0.29
Nodes (6): Fix clippy, don't silence it, Fix tests, Optional supply-chain checks, Report, Rust quality gate, Workflow

### Community 43 - "Shared"
Cohesion: 0.24
Nodes (7): AddArgs, compress_opts(), PrintArgs, add(), pack_sfx(), Shared, print_entry()

### Community 44 - "milestone3.rs"
Cohesion: 0.18
Nodes (11): assert_matches_unrar(), encrypting_an_existing_zip_requires_force(), extract_bytes(), find_unrar(), FIXTURE_PASSWORD, fixtures_root(), password(), passwords_are_rejected_for_tar() (+3 more)

### Community 45 - "MenuCommand"
Cohesion: 0.18
Nodes (11): MenuCommand, About, Add, Delete, Exit, Extract, Info, Open (+3 more)

### Community 46 - "prompt.rs"
Cohesion: 0.07
Nodes (49): accept_password(), ask(), ask_console(), ask_gui(), attach_console(), attach_parent(), Choice, All (+41 more)

### Community 47 - "Icon"
Cohesion: 0.09
Nodes (21): directories_use_the_yellow_folder(), row_glyph(), selected_glyphs_use_the_selection_ink(), Icon, Archive, ArrowUp, CircleCheck, File (+13 more)

### Community 48 - "listing.rs"
Cohesion: 0.43
Nodes (4): lists_a_folder_and_a_text_file(), read_directory(), row_for(), TempDir

### Community 49 - "Password"
Cohesion: 0.09
Nodes (14): Password, copy_sevenz_index(), ProgressWrite, ProgressWrite<'_, W>, Tee, test_sevenz(), open_streaming(), test() (+6 more)

### Community 52 - "Explorer"
Cohesion: 0.09
Nodes (19): format_crc(), SortColumn, Crc, Modified, Name, Packed, Size, Type (+11 more)

### Community 54 - "setup_template"
Cohesion: 0.47
Nodes (9): align4(), Control, push_atom(), push_named(), push_u16(), push_u32(), push_wide(), setup_dialog_has_browse_and_a_progress_bar() (+1 more)

### Community 55 - "ToolCommand"
Cohesion: 0.33
Nodes (6): ToolCommand, Add, Delete, Extract, Info, Test

### Community 56 - "RowKind"
Cohesion: 0.12
Nodes (19): extension(), file_type_label(), named(), Row, RowKind, Archive, Directory, File (+11 more)

### Community 57 - "SfxFormatArg"
Cohesion: 0.67
Nodes (3): SfxFormatArg, SevenZ, Zip

### Community 58 - "path"
Cohesion: 0.19
Nodes (6): ArchiveWriter, create(), create_from_path(), prepare_output(), prepare_temp(), temp_beside()

### Community 60 - "milestone2.rs"
Cohesion: 0.22
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
Nodes (19): coder_dictionary(), coder_label(), entry_from(), entry_key(), file(), folder_dictionary(), folder_label(), host_os() (+11 more)

### Community 67 - "EntryMeta"
Cohesion: 0.11
Nodes (11): archive_path(), CloneMeta, EntryMeta, seven_meta(), SevenZArchiver, spool(), Spooled, write_file_body() (+3 more)

### Community 68 - "tests/cli.rs"
Cohesion: 0.16
Nodes (21): add_list_test_extract_round_trip(), add_sfx_writes_a_listable_package(), add_to_an_existing_zip_keeps_previous_files(), help_explains_that_zip_encryption_leaves_names_visible(), log_file_receives_debug_events(), rar_is_not_created(), reads_a_tar_from_the_system_tool(), reads_a_zip_from_compress_archive() (+13 more)

### Community 70 - "ExtractOpts"
Cohesion: 0.15
Nodes (12): only_first(), resolve(), unsupported(), Verify, mark_extracted(), finish_staged(), extract_sevenz(), ExtractOne (+4 more)

### Community 71 - "add_dialog.rs"
Cohesion: 0.06
Nodes (65): creatable_formats(), SevenZMethod, Bzip2, Deflate, Lzma, Lzma2, Ppmd, Zstd (+57 more)

### Community 72 - ".go_up"
Cohesion: 0.25
Nodes (3): is_sfx_name(), starting_folder(), is_supported_archive()

### Community 73 - "sevenz.rs"
Cohesion: 0.22
Nodes (16): add_lists_and_extracts_through_the_core(), aes_hides_bytes_and_header_encryption_hides_names(), bzip2_and_deflate_round_trip(), committed_sevenz_fixtures_list_when_present(), extract_bytes(), fast(), FIXTURE_PASSWORD, header_encryption_is_a_password_error_for_every_nonce() (+8 more)

### Community 74 - "folder_formats.rs"
Cohesion: 0.36
Nodes (11): assert_tree(), Case, cases(), extract_all(), nested_folder_round_trips_every_writable_format(), opts(), planned_names(), single_stream_rejects_a_folder() (+3 more)

### Community 75 - "tests/extract.rs"
Cohesion: 0.33
Nodes (9): built_stub_contains_the_shellzip_icon(), encrypted_sfx(), silent_encrypted_stub_fails_without_a_password(), silent_encrypted_stub_rejects_a_wrong_password(), stub_bin(), stub_extracts_a_header_encrypted_7z_with_the_password(), stub_extracts_an_encrypted_zip_when_the_password_is_set(), stub_extracts_its_appended_7z() (+1 more)

### Community 77 - "Opened"
Cohesion: 0.27
Nodes (6): Opened, list(), list_with(), list(), list_with(), ListOpts

### Community 78 - "UpdateMode"
Cohesion: 0.22
Nodes (8): UpdateMode, Add, .ALL, Ask, Freshen, Skip, Sync, Update

### Community 80 - "run"
Cohesion: 0.25
Nodes (7): ComInit, dword_words(), finish_worker(), init_common_controls(), run(), show(), windows_accepts_the_setup_dialog_template()

### Community 82 - "ArchiveReader"
Cohesion: 0.33
Nodes (3): ArchiveReader, open(), open()

## Knowledge Gaps
- **163 isolated node(s):** `shellzip-cli`, `Add`, `Update`, `Freshen`, `Delete` (+158 more)
  These have ≤1 connection - possible missing edges or undocumented components. (Counts symbols only; 519 node(s) total have ≤1 connection when file, concept and rationale nodes are included.)
- **16 thin communities (<3 nodes) omitted from report** — run `graphify query` to explore isolated nodes.

## Suggested Questions
_Questions this graph is uniquely positioned to answer:_

- **Why does `Error` connect `Error` to `worker.rs`, `ops.rs`, `policy.rs`, `src/sfx.rs`, `Password`, `stream.rs`, `shellzip-sfx/src/main.rs`, `walk.rs`, `setup.rs`, `archiver/zip.rs`, `Format`, `round_trip.rs`, `list.rs`, `extractor/tar.rs`, `extract.rs`, `src/cli.rs`, `CompressOpts`, `app.rs`, `CompressOpts`, `archiver/tar.rs`, `extractor/zip.rs`, `delete.rs`, `Shared`, `milestone3.rs`, `prompt.rs`, `Password`, `write`, `path`, `EntryMeta`, `ExtractOpts`, `folder_formats.rs`, `sevenz/options.rs`?**
  _High betweenness centrality (0.155) - this node is a cross-community bridge._
- **What connects `shellzip-cli`, `Add`, `Update` to the rest of the system?**
  _163 weakly-connected nodes found - possible documentation gaps or missing edges._
- **Should `worker.rs` be split into smaller, more focused modules?**
  _Cohesion score 0.07396870554765292 - nodes in this community are weakly interconnected._
- **Why does `CompressOpts` connect `CompressOpts` to `ops.rs`, `stream.rs`, `walk.rs`, `archiver/zip.rs`, `round_trip.rs`, `src/cli.rs`, `CompressOpts`, `archiver/tar.rs`, `stream_one`, `Shared`, `Password`, `write`, `path`, `milestone2.rs`, `EntryMeta`, `add_dialog.rs`, `sevenz.rs`, `folder_formats.rs`, `UpdateMode`, `sevenz/options.rs`?**
  _High betweenness centrality (0.063) - this node is a cross-community bridge._
- **Should `ops.rs` be split into smaller, more focused modules?**
  _Cohesion score 0.13105413105413105 - nodes in this community are weakly interconnected._
- **Why does `ProgressSink` connect `Password` to `worker.rs`, `CompressOpts`, `ops.rs`, `Password`, `ExtractOpts`, `shellzip-sfx/src/main.rs`, `extractor/zip.rs`, `stream_one`, `setup.rs`, `Shared`, `NopProgress`, `extractor/tar.rs`, `ArchiveReader`, `theme.rs`, `CompressOpts`, `BarProgress`?**
  _High betweenness centrality (0.061) - this node is a cross-community bridge._
- **Should `src/sfx.rs` be split into smaller, more focused modules?**
  _Cohesion score 0.06105834464043419 - nodes in this community are weakly interconnected._