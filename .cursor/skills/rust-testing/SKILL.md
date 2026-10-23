---
name: rust-testing
description: Write and organize Rust tests for the ShellZip workspace — unit tests, table-driven cases, integration tests, assert_cmd CLI tests, proptest property tests, insta snapshots, and trait-based fakes. Use when adding tests, raising coverage, reproducing a bug with a failing test, or when the user asks how to test a Rust function, command, or archive format.
---

# Rust testing

## Choose the layer

| Testing... | Layer | Location |
|---|---|---|
| a private fn or one module | unit | `#[cfg(test)] mod tests` in the same file |
| public API across modules | integration | `crates/<crate>/tests/<scenario>.rs` |
| `szip` arguments, exit codes, output | CLI e2e | `crates/shellzip-cli/tests/cli.rs` |
| "never escapes dest", "always round-trips" | property | `proptest` inside a unit or integration test |
| `l` / `lt` listing text | snapshot | `insta` in an integration test |

## Workflow

1. Reproduce first: write the failing test, run `cargo test -p <crate> <name>`, confirm it fails for the right reason.
2. Implement; re-run that one test; then `cargo test --workspace`.
3. Name by behaviour (`rejects_escape_and_reserved`), one behaviour per test, no `test_` prefix.
4. Table-driven tests put the case in the assertion message so a failure names its input.
5. Fixtures are built inside `tempfile::tempdir()`. Commit a binary under `fixtures/` only when an external tool made it (WinRAR, 7z, bsdtar, `Compress-Archive`); add its SHA-256 to `fixtures/SHA256SUMS` and verify in the test.

Copy-paste patterns for every layer: [patterns.md](patterns.md).

## Edge cases archive code must cover

- Empty file, empty directory, archive with zero entries
- Unicode names, names with spaces, paths longer than 248 chars (Windows long-path prefix)
- Zip-slip: `../evil.txt`, `/etc/passwd`, `C:\evil`, `dir/CON.txt`, trailing dot/space, symlink pointing outside dest
- Overwrite modes always / never / rename, with and without `-y`
- Wrong or missing password → exit code 11
- Interruption mid-copy → `Error::Interrupted` and no partial output file
- Interop: extract an archive written by the OS tool; the OS tool reads ours

## Fakes over mocks

- `ProgressSink` → `NopProgress`, or a counting fake when you assert on progress
- `Read` / `Write` → `std::io::Cursor<Vec<u8>>`
- Prompts → a closure `ConflictDecider`
- No seam? Introduce a trait at the boundary first. Reach for `mockall` only if a hand-written fake would be unwieldy.
- Never add global state to make something testable.

## Running

- `cargo test -p <crate> <name> -- --nocapture` while iterating
- `cargo test --workspace` before finishing
- `cargo test --doc -p shellzip-core` when you add runnable doc examples
