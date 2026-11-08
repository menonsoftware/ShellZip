# Building ShellZip

ShellZip is a Cargo workspace. The Makefile wraps the same commands CI runs.

## Requirements

- **Rust** stable, edition 2024. `rust-toolchain.toml` pins `stable` with `rustfmt` and `clippy`. Workspace `rust-version` is **1.88**.
- **Cargo** (comes with rustup)
- **make** (GNU Make on Unix; on Windows, Git Bash or a Make from chocolatey/msys works). You can call `cargo` directly instead.

Linux desktop builds need GTK and window-system headers (same list as `.github/workflows/ci.yml`):

```text
g++ pkg-config libgtk-3-dev libx11-dev libxcb1-dev
libxkbcommon-dev libxkbcommon-x11-dev libwayland-dev libssl-dev
libxcb-render0-dev libxcb-shape0-dev libxcb-xfixes0-dev
```

## Quick commands

| Target | What it runs |
|---|---|
| `make` / `make build` | `cargo build --workspace` |
| `make release` | Release `szip`, `shellzip`, and the minisize SFX stub copied into `target/release` |
| `make build-cli` / `make build-gui` / `make build-sfx` | One crate |
| `make run ARGS="l a.zip"` | `cargo run --bin szip -- …` |
| `make gui` | Open the desktop window |
| `make explore` | Open `target/release` in the file manager |
| `make test` | Workspace tests, then core with `rar,sevenz-codecs,sevenz-parallel` |
| `make clippy` | `clippy --workspace --all-targets -- -D warnings` |
| `make fmt` / `make fmt-check` | rustfmt |
| `make deny` | `cargo deny check` |
| `make ci` | fmt-check + clippy + test + deny + SFX-subset clippy |
| `make install` / `make install-gui` | `cargo install` the CLI or GUI |
| `make bench` | Release ZIP timings into `docs/BENCH.md` |
| `make velopack-stage` | Stage GUI/CLI/SFX (+ shell DLL) and icons under `dist/velopack/` |
| `make velopack-pack` | Stage then run `vpk pack` (GPG on Linux when env is set) |
| `make msix` | Windows: build release binaries and pack an unsigned Store MSIX under `dist/msix/` |
| `make doc` | `cargo doc --workspace --no-deps --open` |
| `make clean` | `cargo clean` |

Extra Cargo flags: `make build CARGO_FLAGS="--offline"`.

Without Make:

```bash
cargo build --workspace
cargo build --release -p shellzip-cli -p shellzip-gui
cargo build -p shellzip-sfx --profile minisize
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo fmt --all
```

## Workspace members

```
crates/shellzip-core    library
crates/shellzip-cli     bin `szip`
crates/shellzip-gui     bin `shellzip`
crates/shellzip-sfx     bin `shellzip-sfx`
xtask                   developer binary
```

Default members are the four `crates/*` packages. `xtask` is built only when invoked.

## Profiles

Defined only in the root `Cargo.toml`:

```toml
[profile.release]
lto = true
codegen-units = 1
strip = true

[profile.minisize]      # SFX stub
inherits = "release"
opt-level = "z"
panic = "abort"
```

`make release` builds the stub with `minisize` and copies it next to the release CLI/GUI so `szip a --sfx` can find it.

## Cargo features

`shellzip-core` features:

| Feature | Who enables it |
|---|---|
| `rar` | `szip`, `shellzip` |
| `sevenz` | SFX stub; implied by the next two |
| `sevenz-codecs` | `szip`, `shellzip` |
| `sevenz-parallel` | `szip`, `shellzip` |

Do not hand-edit `Cargo.lock`. Add dependencies with `cargo add <crate> -p <member>` and `default-features = false`. Licences must be MIT / Apache-2.0 / BSD-compatible (`deny.toml`).

## Quality gate

Before finishing Rust changes, run the same gate as CI:

```bash
cargo fmt --all
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

`make ci` also runs `cargo deny check` and clippy on `shellzip-core` with `--no-default-features --features sevenz` (the SFX subset).

Warnings are errors. `#[allow(…)]` needs a same-line comment saying why. `unsafe_code = "forbid"` at the workspace; the SFX crate uses `deny` so Win32 dialog FFI can compile.

CI (`.github/workflows/ci.yml`) runs on `windows-latest` and `ubuntu-latest`, `fail-fast: false`, toolchain `dtolnay/rust-toolchain@stable`.

## Tests

- Unit tests live next to the code (`#[cfg(test)]`)
- Integration tests: `crates/<crate>/tests/*.rs`
- CLI end-to-end: `crates/shellzip-cli/tests/cli.rs` (`assert_cmd`)
- Fixtures under `fixtures/` are committed binaries (for example RAR samples) with checksums

```bash
cargo test -p shellzip-core
cargo test -p shellzip-cli
cargo test -p shellzip-core --features rar,sevenz-codecs,sevenz-parallel
```

## Benchmarks

```bash
make bench
# or
cargo run --release -p xtask -- bench
```

Times ZIP creation at `-m5` with 1 and 8 threads against a 32 MiB text corpus, and 7z when `7z` is on `PATH`. Results overwrite [BENCH.md](BENCH.md). Criterion benches live under `crates/shellzip-core/benches/`.

## File icons

Master SVGs live in `grx/shellzip_file_icons/svg/`. Rasterize committed multi-size packs with:

```bash
make gen-file-icons
# or
cargo run -p xtask -- gen-file-icons
```

That writes Vista-compatible `.ico` (BMP ≤64 + PNG 256 for Explorer Medium/Large/Extra Large), full Retina `.icns` under `icns/` and `macos/`, and freedesktop MIME PNGs under `linux/hicolor/{16..512}/`.

## Shell integration

```bash
make install-shell          # cargo install shellzip, build the Windows DLL, then --integrate
shellzip --integrate        # per-user associations + menus
shellzip --unintegrate
```

Crates: `shellzip-integrate` (tables and installers), `shellzip-shell` (Windows Explorer cdylib). Platform notes: [platform/windows](../platform/windows), [platform/macos](../platform/macos), [platform/linux/nautilus](../platform/linux/nautilus).

Velopack installs call integrate automatically (Windows install/update/uninstall FastCallbacks; first-run and post-update restart on Linux/macOS — Velopack has no uninstall hook on those platforms). Portable zip/`cargo install` users still use **Options → Integrate** or `--integrate`. Packaged layouts ship format icons next to the binary as `icons/` (see `make velopack-stage`).

## Velopack packaging

Installers and auto-updates use [Velopack](https://github.com/velopack/velopack) (`packId` `com.shellzip.ShellZip`). Channels are `win` / `osx` / `linux`. Feeds live on GitHub Releases.

```bash
# Needs .NET 8 SDK and: dotnet tool install -g vpk
make release                # or release-cli/gui/sfx/shell as needed
make velopack-pack          # stages dist/velopack/<channel>/ then vpk pack
```

Tag releases still upload portable `ShellZip-<tag>-<target>.zip|.tar.gz` archives; prefer the Velopack Setup / AppImage / `.app` for GUI installs with updates. Windows also packs `ShellZip-<tag>-x64.msix` for Partner Center.

### Windows Store MSIX

`make msix` (or the Windows leg of `release.yml`) runs `xtask msix-pack`. The package is full-trust desktop (`shellzip.exe`, `szip.exe`, `shellzip-sfx.exe`), declares archive file associations and static Extract verbs, and stays **unsigned** on GitHub Releases. Partner Center re-signs it after `msstore publish`.

| Env / secret | Purpose |
|---|---|
| `STORE_IDENTITY_NAME` | Package Identity `Name` (must match Partner Center) |
| `STORE_PUBLISHER` | Package Identity `Publisher` (`CN=…` exactly as Partner Center shows) |
| `STORE_PUBLISHER_DISPLAY_NAME` | Must match Partner Center publisher display name (for example `Menonsoftware`) |
| `STORE_APP_ID` | Store product id for `msstore publish` |
| `STORE_TENANT_ID` / `STORE_SELLER_ID` / `STORE_CLIENT_ID` / `STORE_CLIENT_SECRET` | Entra app that Partner Center granted Manager rights |

Upload only the `.msix` from `dist/msix/` (not the `payload/` folder). Partner Center re-signs it. Local defaults (`CN=ShellZip` / display name `ShellZip`) are for layout checks only — a Store submission must be packed with the real identity env vars set. `runFullTrust` needs a restricted-capability note / approval in Partner Center the first time.

If any publish secret is empty, CI skips the Store upload and still ships Velopack. Reserve the app name and complete the first listing in Partner Center before the CLI can publish updates.

Windows Velopack Setup and the portable zip stay unsigned on purpose (no Authenticode signer in CI).

### Linux GPG verification

After `vpk pack`, CI detach-signs the AppImage (and related artifacts) and publishes `SHA256SUMS` / `*.asc` when these secrets are set: `GPG_PRIVATE_KEY`, `GPG_KEY_ID`, optional `GPG_PASSPHRASE`. Publish the matching public key as `docs/signing-key.asc` and record its fingerprint here when the release key is created.

```bash
gpg --import docs/signing-key.asc
sha256sum -c SHA256SUMS
gpg --verify SHA256SUMS.asc SHA256SUMS
gpg --verify ShellZip-*.AppImage.asc ShellZip-*.AppImage
```

### macOS

Velopack packs an `.app` on `macos-latest`, but Apple Developer ID signing and notarization are not wired yet. Users may need to right-click → Open the first time (Gatekeeper).

## Platform notes

- **Windows** — GUI and SFX are `windows_subsystem = "windows"`. Icons and manifests come from `winresource` build scripts. `--om` writes `Zone.Identifier`. Build `shellzip-shell` for the Explorer menu on Velopack / portable installs. Store installs use the MSIX associations instead.
- **Linux** — install the apt packages above before `make gui`. `shellzip --integrate` installs the desktop file, hicolor MIME icons, and KDE/Nemo/Thunar actions. Distribution is a Velopack AppImage (`chmod +x`); verify with GPG when signatures are published.
- **macOS** — release matrix builds `aarch64-apple-darwin`. Bundle document types and the Finder Sync sources under `platform/macos/`. Notarization is deferred.

## API docs

```bash
make doc
```

`shellzip-core` is the public library surface (`add`, `extract`, `list`, `delete`, `rename`, `Format`, `CompressOpts`, …). The CLI and GUI crates are binaries.
