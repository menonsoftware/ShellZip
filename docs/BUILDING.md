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

## Platform notes

- **Windows** — GUI and SFX are `windows_subsystem = "windows"`. Icons and manifests come from `winresource` build scripts. `--om` writes `Zone.Identifier`. Build `shellzip-shell` for the Explorer menu.
- **Linux** — install the apt packages above before `make gui`. `shellzip --integrate` installs the desktop file, hicolor MIME icons, and KDE/Nemo/Thunar actions.
- **macOS** — not in the CI matrix yet. Bundle document types and the Finder Sync sources under `platform/macos/`.

## API docs

```bash
make doc
```

`shellzip-core` is the public library surface (`add`, `extract`, `list`, `delete`, `rename`, `Format`, `CompressOpts`, …). The CLI and GUI crates are binaries.
