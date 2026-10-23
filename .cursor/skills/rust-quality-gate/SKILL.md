---
name: rust-quality-gate
description: Run and fix the ShellZip Rust quality gate — rustfmt, clippy with -D warnings, workspace tests, rustdoc, optional cargo-deny and cargo-audit. Use before finishing any Rust change, when CI fails on fmt, clippy, or tests, or when the user asks to lint, format, fix warnings, or make CI green.
---

# Rust quality gate

## Workflow

1. Format: `cargo fmt --all`
2. Lint: `cargo clippy --workspace --all-targets -- -D warnings`
3. Test: `cargo test --workspace`
4. Docs, when a `pub` item changed: `cargo doc --workspace --no-deps` with `RUSTDOCFLAGS=-D warnings`
   - PowerShell: `$env:RUSTDOCFLAGS = "-D warnings"; cargo doc --workspace --no-deps`
   - bash: `RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps`
5. After each fix, re-run from step 2 until everything is clean. `make ci` runs steps 1–3.

While iterating, scope to one crate (`-p shellzip-core`) and one test (`cargo test -p shellzip-core <name> -- --nocapture`); finish with `--workspace`.

## Fix clippy, don't silence it

| Lint | Fix |
|---|---|
| `needless_borrow`, `redundant_clone`, `clone_on_copy` | remove the `&` / `.clone()` |
| `unwrap_used`, `expect_used` (also by review) | `?`, `ok_or(Error::...)`, `unwrap_or_else` |
| `too_many_arguments` | group into an options struct (`ExtractOpts`) |
| `large_enum_variant` | `Box` the large payload |
| `cast_possible_truncation`, `cast_sign_loss` | `u64::try_from(x)?` / `usize::try_from(x)?` |
| `needless_pass_by_value` | take `&T` |
| `missing_docs`, `missing_errors_doc` | add `///` and an `# Errors` line |
| `new_without_default` | `impl Default` or `#[derive(Default)]` |
| `module_inception` | rename the inner module |
| `manual_let_else`, `question_mark` | use `let ... else` / `?` |

`#[allow(clippy::...)]` goes on the smallest possible item with a same-line reason, e.g. `#[allow(clippy::struct_excessive_bools)] // mirrors the on-disk flag byte`.

## Fix tests

- Read the first failing assertion only; later failures are usually consequences.
- Windows-only or Unix-only behaviour: `#[cfg(windows)]` / `#[cfg(unix)]` on the test, never `#[ignore]`.
- Flaky `tempdir` cleanup on Windows means a `File` handle is still open when the dir drops; scope the handle in a block.
- Output text changed on purpose? Update the `contains(...)` predicate in `tests/cli.rs` and say so in the summary.

## Optional supply-chain checks

Install once: `cargo install cargo-deny cargo-audit`.

- `cargo deny init` (first time) then `cargo deny check` — licenses (MIT / Apache-2.0 / BSD only), bans, duplicates
- `cargo audit` — RustSec advisories against `Cargo.lock`

## Report

End with one line per step: `fmt clean`, `clippy clean`, `N tests passed (M crates)`, and anything you had to change in a test or `#[allow]` and why.
