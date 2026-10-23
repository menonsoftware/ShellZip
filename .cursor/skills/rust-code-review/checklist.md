# Rust review checklist

Walk each section for every changed file. Items marked **[core]** apply to `shellzip-core`; **[bin]** to `szip` and `shellzip`.

## 1. Correctness

- [ ] Logic handles empty input, zero-length entries, missing parents, and the last iteration of every loop.
- [ ] Integer math on sizes/offsets/counts from an archive uses `checked_*` or `try_from`; no `as` between widths.
- [ ] Unicode: paths compared with `Path` components, not byte offsets into `String`; no `.len()` used as a character count.
- [ ] Windows vs Unix: separators normalised (`\` → `/`) before validation; long-path prefix applied on write; case-insensitive comparison where NTFS semantics apply.
- [ ] Early returns and `?` do not skip cleanup (temp files, encoder `finish()`).
- [ ] No behaviour change hidden behind a feature flag that defaults on.

## 2. Error handling

- [ ] No `unwrap` / `expect` / `panic!` / `unreachable!` outside `#[cfg(test)]`.
- [ ] **[core]** Returns `shellzip_core::Error`; new variant has `exit_code()` mapping and the module-doc table is updated.
- [ ] OS errors wrapped with `Error::io(context, source)`; context names the action and path.
- [ ] Error `Display` text is user-safe: no passwords, no `{:?}` of internal structs.
- [ ] `From` impls for third-party errors live in `error.rs`.
- [ ] **[bin]** Errors are printed as `szip: {err}` and the process exits with `err.exit_code()`; GUI jobs return `Outcome::Failed`.
- [ ] Poisoned locks recovered with `unwrap_or_else(|p| p.into_inner())`.

## 3. Security

- [ ] Every entry name → `safe_rel` → `safe_join` before create/write/chmod/symlink; symlink targets → `symlink_stays_inside`.
- [ ] Declared sizes validated against caps (per entry, total, ratio) before allocation or write.
- [ ] No `read_to_end` / `Vec<u8>` of an entry; fixed buffer streaming.
- [ ] No `std::process::Command` fed by user strings; no shell interpolation.
- [ ] Files created with `create_new` or temp + rename; directories via `ensure_dir`.
- [ ] Permission bits sanitised (no setuid/setgid/world-writable from archive metadata).
- [ ] Passwords only in `Password`; absent from logs, `tracing` fields, `Debug`, errors, and test fixtures committed to the repo.
- [ ] No hard-coded secrets, tokens, or real credentials in examples.
- [ ] `unsafe` absent (workspace forbids it); any FFI crate addition reviewed for its own unsafe surface.

## 4. Ownership and performance

- [ ] Parameters are `&str` / `&Path` / `&[T]` / `&dyn Trait` unless stored.
- [ ] No `clone()` that a borrow would satisfy; no clone in a loop without a comment.
- [ ] No `collect()` between iterator stages that could stay lazy.
- [ ] Buffers allocated once outside loops; `BufReader` / `BufWriter` around files.
- [ ] `format!` not used in hot loops; `write!` into a reused `String`.
- [ ] `#[inline]` only with a profile-backed reason.
- [ ] No premature optimisation that obscures intent without a measured win.

## 5. Concurrency

- [ ] Types crossing threads are `Send + Sync`; `ProgressSink` impls are cheap and non-blocking.
- [ ] No `Mutex` guard held across I/O, channel send, or another lock.
- [ ] Atomics use the weakest sufficient `Ordering` (counters: `Relaxed`).
- [ ] Cancellation (`AtomicBool`) is polled per buffer and yields `Error::Interrupted`.
- [ ] **[bin GUI]** No archive I/O on the `eframe` update path; results carry a `generation` tag.
- [ ] Parallel ZIP path produces byte-identical output to the serial path; in-flight work is bounded.

## 6. Tests

- [ ] New behaviour has a test at the right layer (unit / integration / `tests/cli.rs`).
- [ ] Bug fix includes the test that failed before the fix.
- [ ] Test names describe behaviour; no `test_` prefix; one behaviour per test.
- [ ] Fixtures built in `tempfile::tempdir()`; committed fixtures have SHA-256 verification.
- [ ] Platform-specific tests gated with `#[cfg]`, not `#[ignore]`.
- [ ] Edge cases present where relevant: empty, unicode, zip-slip names, overwrite modes, wrong password → 11.
- [ ] Fakes go through traits; no global state introduced to make something testable.

## 7. API and docs

- [ ] Every new `pub` item has a `///` doc; new file has a `//!` header.
- [ ] Public surface re-exported from `lib.rs`; internal modules stay private.
- [ ] Option structs implement `Default`; enums used instead of boolean flags.
- [ ] Newtypes for values with invariants.
- [ ] Breaking change to `shellzip-core` API is called out and both front ends updated.

## 8. Manifest and tooling

- [ ] New dependency: justified, license compatible, `default-features = false` with explicit features, placed in `[dev-dependencies]` if test-only.
- [ ] `Cargo.lock` updated by cargo, not by hand.
- [ ] Member crate inherits workspace `package` fields and `[lints] workspace = true`.
- [ ] CI (`ci.yml`) and `make ci` still match; new system libs added to the Linux apt list.
- [ ] `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace` all pass locally.
