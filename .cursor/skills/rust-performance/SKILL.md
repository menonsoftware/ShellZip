---
name: rust-performance
description: Benchmark, profile, and optimize Rust code or binary size in the ShellZip workspace — criterion benchmarks, flamegraphs, allocation and streaming fixes, thread scaling, and release-profile tuning (LTO, strip, opt-level z). Use when something is slow, memory-heavy, or too large, or when the user asks to benchmark, profile, optimize, or shrink a binary.
disable-model-invocation: true
---

# Rust performance

Rule: measure → change one thing → measure again. No optimization lands without a before/after number in the summary.

## 1. Baseline

- Micro: a criterion bench (scaffold with `rust-scaffold`), then
  `cargo bench -p <crate> -- --save-baseline before`
- Macro: time the release binary on a fixed corpus
  - PowerShell: `cargo build --release; Measure-Command { .\target\release\szip.exe a out.zip corpus }`
  - bash: `/usr/bin/time -v target/release/szip a out.zip corpus/` (prints max RSS)
- Record wall time, MB/s, peak RSS, thread count, corpus size. Numbers go in the PR description or `docs/BENCH.md`.

## 2. Find the hot spot

- `cargo install flamegraph`, then
  `cargo flamegraph -p shellzip-cli --bin szip -- a out.zip corpus/`
  Linux needs `perf`; Windows uses DTrace/ETW and may need an elevated shell.
- Keep symbols: `CARGO_PROFILE_RELEASE_DEBUG=1` for the profiling run (don't commit `debug = true` to the release profile).
- Allocation pressure: `cargo clippy -p <crate> -- -W clippy::perf -W clippy::pedantic` surfaces `needless_collect`, `redundant_clone`, `large_types_passed_by_value`; for exact counts use `dhat` (`cargo add dhat --dev`) behind a `dhat-heap` feature.
- Thread scaling: run the same job with `--threads 1 2 4 8` and tabulate; sub-linear scaling means a serial section or lock contention.

## 3. Apply the cheapest fix the profile justifies

1. Streaming / buffering: fixed buffer, `BufReader` / `BufWriter`, hash-while-copy adapters, encoders finished once.
2. Allocation: reuse buffers, `&str` params, drop intermediate `collect()`, `with_capacity`.
3. Data structure: `HashMap` lookup instead of linear `Vec` scan; `BTreeMap` only when order matters.
4. Parallelism: `rayon` per-entry with bounded in-flight pieces; zstd `multithread(n)`; verify output is byte-identical to the serial path.
5. Compiler: `#[inline]` on tiny cross-crate helpers that the flamegraph shows as separate frames. Nothing else.

Each step: re-run the bench, keep only changes that move the number.

## 4. Binary size

- Root `[profile.release]`: `lto = true`, `codegen-units = 1`, `strip = true`; `panic = "abort"` only for binaries that don't rely on unwinding (not the GUI).
- `[profile.minisize]` inheriting release with `opt-level = "z"` for the SFX stub.
- `cargo install cargo-bloat`, then `cargo bloat --release -p <crate> -n 30`. Trim features (`default-features = false`) before removing dependencies.
- Confirm with `ls -l` / `Get-Item` on the binary before and after.

## 5. Verify and report

- `cargo bench -p <crate> -- --baseline before`
- `cargo test --workspace` and the quality gate still pass; round-trip tests prove output is unchanged.
- Summary: metric, before, after, what changed, what you tried and reverted.
