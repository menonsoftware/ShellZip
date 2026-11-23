---
name: optimize-rust
description: >-
  Optimize a Rust hot path in ShellZip with the warp-speed checklist: measure
  first, then algorithms, cache layout, registers, static dispatch, bounds-check
  hoisting, and instruction-level parallelism. Use when the user runs
  /optimize-rust or asks to apply the Rust optimization guide, speed up a
  function, or tune a hot loop.
disable-model-invocation: true
---

# Optimize Rust

Source: [Achieving warp speed with Rust](https://gist.github.com/jFransham/369a86eff00e5f280ed25121454acec1) (jFransham). Apply the rules below. Do not paste the article into the diff.

`rust-performance` owns measurement, criterion benches, flamegraphs, and release-profile size. This command decides what to change. Invoke `rust-performance` for the baseline and the after measurement.

## Scope

Optimize the function, module, or bench the user named. If they named nothing, ask which path is hot and stop. Leave the rest of the workspace alone.

## Procedure

1. **Leave it** when real use is fine and the code is already straightforward. Say so. Readable code stays until a profile shows a hot spot. Weigh the time to change the code against the CPU or wall time the change would save.
2. **Orient.** Run `graphify query` on the hot function before editing. Note callers, allocations, and whether the cost is per call or once per process.
3. **Baseline** with `rust-performance` (`cargo bench -p <crate> -- --save-baseline before`, or a timed release run). Record the metric and the input. Wrap bench inputs in `std::hint::black_box` so the compiler cannot fold the call away. Rerun microbenchmarks and drop a slow first sample. A sub-millisecond delta from one run is not a result.
4. **Skip one-shot work.** Config parsing, startup, and any function that runs once stay as they are, even when that function is the slowest single call. Optimize the code that dominates cumulative runtime (a 2 ms win on a function called 1000 times beats a 1 s win on a function called once).
5. **Change one thing.** Re-measure. Keep the change only when the number moves and behavior stays the same (round-trip tests, byte-identical output). Revert the rest.
6. **Gate.** `cargo fmt --all`, then `cargo clippy --workspace --all-targets -- -D warnings`, then `cargo test --workspace`.

## What to try, cheapest first

Stop at the first change that wins. A micro-opt on top of an `O(n²)` pass is the wrong change.

### 1. Less work

- Fewer passes. Every `collect` walks the whole sequence. Fuse the work into one iterator.
- Fix complexity before tuning constants. Quadratic code hides on small inputs. Use a known algorithm; do not invent one.
- Make the inner loop a pure function. Indirect reads and writes block LLVM and make the bench noisy.

### 2. Cache: data used together sits together

Stack when the size is known without running the program. Heap when it is not (`String`, `Vec`, `HashMap`, `Box<[T]>`, `Box<T>` for a sized `T`, including the `Box` inside a recursive enum).

Replace `Vec<Vec<T>>` and other pointer-per-row layouts with one flat `Vec<T>` plus dimensions. If rows are jagged and fixed after build, store one `Vec<T>` and a list of index ranges and return slices. Keep separate inner `Vec`s only when each row is both jagged and growable on its own.

Use `[[T; N]; M]` when both dimensions are static. Keep at most one pointer hop from the container to the value.

`smallvec` or `arrayvec` only after a profile shows heap traffic on values that are usually short. Add with `cargo add -p <crate>` and `default-features = false`. Both licenses are MIT OR Apache-2.0. Leave unstable `TypedArena` out.

### 3. Registers

Write locals you own. A mutable loop index is fine. Passing `&mut` to that index through several calls is not, unless those calls are already inlined. Straight-line code optimizes more readily than pointer-heavy code.

### 4. Dispatch

Prefer `&impl Trait` or `&mut impl Trait` over `Box<dyn Trait>`. A closed set of variants is an enum, or a generic pair such as `struct Cons<H, T>(H, T)` / `Or<A, B>`, so the call stays static and the values stay contiguous. `dyn Trait` is for an open set. ShellZip boxes `dyn Write` at compression boundaries; leave those boundaries boxed.

### 5. Still hot after the layout change

- **Bounds checks.** Hoist one `assert!(slice.len() >= N)` before a run of indexes so later checks are unreachable:

  ```rust
  fn sum6(xs: &[u8]) -> u8 {
      assert!(xs.len() >= 6);
      xs[0] + xs[1] + xs[2] + xs[3] + xs[4] + xs[5]
  }
  ```

  This workspace forbids `unsafe`. Stop at the `assert!`. Unchecked indexing is out.

- **Loops.** LLVM unrolls ordinary loops. Hand-unroll only to remove work that dominates the body (a division, or a multiply that waits on the previous iteration). When the length is a multiple of `N`, iterate `&[[T; N]]` and operate on one `[T; N]` per step.

- **Constants.** Integer division and branches are much slower when the operand is not a constant. Put `#[inline]` on a tiny helper that callers usually pass a constant, so they can fold it. Also bench the non-constant call, so the win is real for both.

- **Floats** for non-constant division or transcendental math when the precision loss is acceptable. Integer add and multiply stay on integers. The compiler will not drop precision for you.

- **Instruction-level parallelism.** Rewrite a fold so independent multiplies do not wait on the previous add. Independent arithmetic autovectorizes; a loop-carried dependency does not. `rayon` and thread pools stay in `rust-performance`, and their output must match the serial path byte for byte.

- **Inlining.** `#[inline(always)]` stays off. `#[inline]` only on a tiny hot cross-crate function (`next`, `deref`, a const-argument helper) after a bench shows a win. Extra inlining grows the binary and misses the instruction cache.

- **LTO and panic strategy.** Release LTO lives in the workspace `[profile.release]`. Do not add a second copy. `panic = "abort"` is a measured size experiment for a binary that does not unwind. The GUI keeps unwinding.

## Report

```
Target: <function>
Baseline: <metric and input>
Kept: <change and the new number>
Reverted: <tries that did not win>
Left alone: <one-shot or readability-bound code>
```
