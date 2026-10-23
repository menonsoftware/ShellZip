---
name: rust-code-review
description: Review Rust changes in the ShellZip workspace for correctness, error handling, security of untrusted archive input, ownership and performance, concurrency, tests, and API docs. Use when the user asks for a review, audit, or second look at Rust code, a diff, a branch, a PR, or a crate, or asks whether Rust code is safe or idiomatic.
---

# Rust code review

## Steps

1. Get the change set: `git diff` for uncommitted work, `git diff main...HEAD` for a branch, or the files the user named.
2. For each changed file, walk [checklist.md](checklist.md) top to bottom: Correctness → Errors → Security → Ownership & performance → Concurrency → Tests → API & docs → Manifest.
3. Run the gate yourself; do not trust the description:
   `cargo clippy --workspace --all-targets -- -D warnings` then `cargo test --workspace`.
4. Check the project invariants in `.cursor/rules/rust.mdc`: UI-free core, no `unwrap` outside tests, `safe_path` before any filesystem write, streaming through a fixed buffer, new formats behind `ArchiveReader` / `ArchiveWriter`.
5. If a finding depends on code outside the diff, open that file. Do not guess at callers.

## Output format

Group by severity. Each finding: `path:line`, the problem in one sentence, the concrete fix (code when short).

- 🔴 **Must fix** — bugs; panics reachable from untrusted input; path escape; secret in logs or errors; `unsafe`; unbounded memory; new `Error` variant without `exit_code` mapping; data race or lock held across I/O
- 🟡 **Should fix** — unnecessary `clone`; `io::Error` without context; `as` cast on header values; new behaviour without a test; `#[allow]` without reason; public item without docs
- 🟢 **Nit** — naming, doc wording, import order, anything rustfmt already accepts

Close with: commands run and their result, and a one-line verdict (mergeable / mergeable after must-fixes / needs rework).

## Stance

- Point at the fix, not just the problem.
- One finding per root cause; don't repeat the same clone/unwrap note ten times — count them and cite one.
- Don't review generated code (`Cargo.lock`) or formatting rustfmt already enforces.
