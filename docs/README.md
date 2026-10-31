# ShellZip documentation

User and developer pages for this repository. Start at the root [README.md](../README.md).

## User guides

| Page | Contents |
|---|---|
| [FAQ.md](FAQ.md) | Common questions (formats, passwords, SFX, settings, exit codes) |
| [CLI.md](CLI.md) | `szip` commands, switches, examples |
| [GUI.md](GUI.md) | Desktop window, dialogs, toolbar, settings |
| [FORMATS.md](FORMATS.md) | What each format can do, encryption, safety |

## Building and internals

| Page | Contents |
|---|---|
| [BUILDING.md](BUILDING.md) | Toolchain, Makefile, features, CI, tests |
| [BENCH.md](BENCH.md) | Generated release timings (`make bench`) |
| [Rust_MVP_Plan.md](Rust_MVP_Plan.md) | Original milestone plan and crate layout |
| [REA_WinRAR_Report.md](REA_WinRAR_Report.md) | Read-only notes on WinRAR 7.23 that informed the CLI |

`cargo doc --workspace --no-deps` (or `make doc`) builds rustdoc for the crates.
