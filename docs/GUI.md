# ShellZip desktop app

`shellzip` is the graphical archive manager. It is an eframe / egui window over the same `shellzip-core` library as `szip`. Archive I/O runs on a worker thread; the UI thread only paints snapshots.

```bash
make gui
make gui-release
shellzip                      # empty window, folder explorer
shellzip D:\archives\app.zip  # open that archive
```

On Windows the binary is a GUI subsystem program (no console). Minimum window size is 820×480; the default is 1080×720.

## Window layout

- **Menu bar** — File, Commands, Options, Help
- **Toolbar** — Open, Add, Extract To, Test, View, Delete, SFX, Info, About
- **Address bar** — current folder or archive path, Up, Refresh, parent dropdown
- **Folder tree** — drives and directories (and the open archive’s folders)
- **File list** — name, size, packed, type, modified, with shell icons where the OS provides them
- **Status line** — selection and job progress

Drag a folder onto the window to start an add. Drop an archive to open it.

## Opening archives

**File → Open…**, the Open toolbar button, or a path on the command line. Detection uses magic bytes, so a misnamed file still opens as the real format.

Recent archives appear under **File**. **File → Clear history** forgets remembered archives and extract destinations.

While a ZIP or 7z is open, the address bar navigates inside it. `..` and Up leave the archive and return to the parent folder.

## Adding files

**File → Add to archive…** (or **Add to this archive…** when a ZIP/7z is open) opens a two-page dialog:

**General**

- Archive name and creatable format (ZIP default, then 7z, TAR, TAR.GZ, TAR.ZST, GZIP, ZSTD)
- Update mode: add and replace, add and update, freshen, ask, skip, synchronize
- Compression level 0–5, ZIP method (Deflate / Zstd), 7z method (LZMA2 default)
- Solid 7z (on by default), password, 7z encrypt file names
- Create SFX (ZIP or 7z payload)
- Run in background (progress window hidden; the file list stays usable)

**Comment**

- ZIP comment only. Leave the box empty after editing to clear an existing comment.

OK starts a worker job. A progress dialog reports bytes and the current name unless background is selected.

## Extracting

**Extract To** opens a dialog modeled on a classic extract window.

**General** — destination, update (replace / update / freshen), overwrite (ask / replace / skip / rename), folder tree

**Advanced** — path mode (relative to the open archive folder, full paths, or no paths), restore modification times, keep broken files

**Options** — delete the archive after a clean full extract, optionally via the recycle bin; open the destination in the file manager; background extract

**Save** writes extract defaults to settings without starting a job.

## Self-extractor

**SFX** opens a dialog when a ZIP, 7z, or ShellZip self-extractor is open. Choose the output path. Wrapping always uses the progress window; unwrapping may **Start in background** like Extract. The stub must sit next to the ShellZip executable, or `SZIP_SFX_STUB` must point at it. Windows cannot run a self-extractor of 4 GiB or larger; keep very large archives as ZIP or 7z.

## Other commands

| Command | What it does |
|---|---|
| Test | Checksums and sizes; results in a report dialog |
| View | Preview one selected file in a window |
| Delete | Remove selected ZIP or 7z members (confirm first) |
| Rename | Rename one selected file or folder prefix inside a ZIP or 7z |
| SFX | Self-extractor dialog (see above) |
| Info | Header summary without decoding payloads |
| Generate report | HTML, Markdown, or text listing of the open archive |
| Select all / none / invert | File-list selection |

RAR archives can be opened, listed, tested, extracted, viewed, and reported. They cannot be added to, deleted from, or renamed.

## Options

- **Theme** — light, dark, or follow the operating system
- **Toolbar size** — normal (24 px), large (48 px), or extra large (64 px)

Settings persist in `settings.toml` (see [FAQ.md](FAQ.md#where-does-the-gui-store-settings)). Pack level, methods, solid flag, extract overwrite/update, SFX background, report format, recent paths, and theme are stored. Passwords are not.

## About

**Help → About** shows the project licence, the UnRAR extraction notice, and a table of every third-party crate linked into the GUI for this OS (regenerated from `cargo tree`; see `xtask/gen_credits.py`).

## File manager integration

**Options → Integrate with the file manager** asks for confirmation, then registers ShellZip for the current user (`shellzip --integrate` / `make install-shell` skip the dialog):

- Default / Open With handlers for zip, zipx, 7z, rar, tar, gz, tgz, bz2, tbz2, xz, txz, zst, tzst
- Per-format document icons from `grx/shellzip_file_icons`
- Context menu: Open, Extract to…, Extract Here, Extract Here (archive folder name)

**Options → Remove file manager integration** asks for confirmation before clearing those registrations.

Shell verbs:

```text
shellzip archive.zip
shellzip --extract-to archive.zip
shellzip --extract-here archive.zip
shellzip --extract-here-named archive.zip
shellzip --integrate
shellzip --unintegrate
```

Extract Here uses the saved overwrite policy (Ask by default) and the password prompt when needed. Linux also installs `scripts/shellzip.desktop` MIME actions; see [platform/](../platform/) for OS-specific notes.

## Limitations

- TAR and single-file streams are not edited in place
- The SFX stub extracts ZIP and 7z LZMA2/AES only; extra 7z codecs need `szip` or `shellzip`
- Self-extractors must stay under 4 GiB so Windows can start them
- Preview is for a single selected file, not a hex editor
- `.exe` self-extractors are never claimed as a document type
