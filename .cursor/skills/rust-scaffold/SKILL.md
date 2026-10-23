---
name: rust-scaffold
description: Scaffold new Rust units in the ShellZip Cargo workspace — a module, a workspace crate, an extra binary, a szip CLI subcommand, an integration test file, a criterion benchmark, or an example — wired into Cargo.toml, lib.rs or main.rs, and CI. Use when adding or creating a new crate, module, command, bench, example, or test file.
---

# Rust scaffold

Pick the unit below, follow its steps, then run the `rust-quality-gate` skill. Templates for each file are in [templates.md](templates.md).

## New module in an existing crate

1. Create `src/<name>.rs`. If it will have children, create `src/<name>/mod.rs` plus `src/<name>/<child>.rs`.
2. First line is `//! <what this file owns>`.
3. Add `mod <name>;` to `lib.rs` / `main.rs`. Re-export the public items with `pub use <name>::{...};` so callers never spell the module path.
4. Add `#[cfg(test)] mod tests { use super::*; ... }` with at least one behaviour test.

## New workspace crate

1. `cargo new --lib crates/shellzip-<name>` (drop `--lib` for a binary crate).
2. Replace the generated `Cargo.toml` with the member template (workspace inheritance, `[lints] workspace = true`).
3. Add `"crates/shellzip-<name>"` to `[workspace] members`, and to `default-members` if it should build by default.
4. `lib.rs` starts with `//!` crate docs and `#![forbid(unsafe_code)]`.
5. Needs new Linux system libraries? Add them to the apt list in `.github/workflows/ci.yml`.
6. Add a `make build-<name>` / `test-<name>` target if the crate is a front end; document it in `help`.

## Extra binary in an existing crate

1. `src/bin/<tool>.rs` with `fn main() { std::process::exit(run()); }`.
2. `[[bin]] name = "<tool>" path = "src/bin/<tool>.rs"` in that crate's `Cargo.toml`.

## New `szip` subcommand

1. `crates/shellzip-cli/src/commands/<verb>.rs` exposing `pub fn <verb>(args: &<Verb>Args, shared: &Shared) -> Result<i32, Error>`.
2. Register `mod <verb>;` in `commands/mod.rs`; add the `Command::<Verb>` variant with clap derive in `cli.rs`; route it in `dispatch`.
3. Archive logic goes into `shellzip-core` (`ops.rs` or a new module), never into the command file.
4. Add an end-to-end case to `tests/cli.rs` asserting stdout text and exit code.
5. Update `szip --help` wording in the clap doc comments; that text is the user manual.

## Integration test file

`crates/<crate>/tests/<scenario>.rs`; public API only; fixtures via `tempfile::tempdir()`.

## Criterion benchmark

1. `cargo add criterion -p <crate> --dev --features html_reports`.
2. `benches/<name>.rs` and `[[bench]] name = "<name>" harness = false` in `Cargo.toml`.
3. `cargo bench -p <crate>`; HTML report in `target/criterion/`.

## Example

`crates/<crate>/examples/<name>.rs` for library crates. CI compiles it through `--all-targets`, so it must stay warning-free.

## Finish checklist

- [ ] `//!` file doc and `///` on every `pub` item
- [ ] No `unwrap` / `expect` outside tests
- [ ] Registered in `Cargo.toml`, `lib.rs` / `main.rs`, workspace members, CI, Makefile as applicable
- [ ] `cargo clippy --workspace --all-targets -- -D warnings` and `cargo test --workspace` pass
