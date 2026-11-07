# macOS packaging

## App bundle

1. Build `shellzip` (`cargo build --release -p shellzip-gui`).
2. Create `ShellZip.app` with `cargo bundle` (from `cargo-bundle`) or a hand-rolled layout:
   - `Contents/MacOS/shellzip`
   - `Contents/Resources/shellzip.icns` (application icon)
   - `Contents/Resources/*.icns` from `grx/shellzip_file_icons/macos/` (document icons; multi-size / Retina)
3. Merge [Info.plist.document-types.fragment.xml](Info.plist.document-types.fragment.xml) into `CFBundleDocumentTypes`.
4. Set `CFBundleIdentifier` to `com.shellzip.ShellZip` (see `crates/shellzip-gui/Cargo.toml` metadata).

## Finder Sync

[FinderSync/ShellZipFinderSync.swift](FinderSync/ShellZipFinderSync.swift) is the context-menu extension. Build it as an appex inside the app bundle on a Mac; Windows and Linux CI do not compile Swift. Enable the extension under **System Settings → Privacy & Security → Extensions → Added Extensions**.

## Integrate

From the built app or a PATH install:

```bash
shellzip --integrate
shellzip --unintegrate
```

`--integrate` copies document-type metadata and icns files and asks Launch Services to refresh.
