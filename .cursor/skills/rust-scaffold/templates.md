# Scaffold templates

Replace `<name>` and the placeholder docs. Keep the `//!` and `///` lines; CI treats missing docs on `pub` items as review failures.

## Member crate `Cargo.toml`

```toml
[package]
name = "shellzip-<name>"
version.workspace = true
edition.workspace = true
license.workspace = true
publish.workspace = true
description = "<One line, no trailing period>"

# Binary crates only:
# [[bin]]
# name = "<tool>"
# path = "src/main.rs"

[dependencies]
shellzip-core = { path = "../shellzip-core" }

[dev-dependencies]
tempfile = "3"

[lints]
workspace = true
```

## Library root `src/lib.rs`

```rust
//! <What this crate does, in one or two sentences.>

#![forbid(unsafe_code)]

mod <module>;

pub use <module>::{<PublicType>, <public_fn>};
```

## Binary root `src/main.rs`

```rust
//! `<tool>` — <one line>.

mod cli;

fn main() {
    std::process::exit(cli::run());
}
```

## Module with unit tests `src/<name>.rs`

```rust
//! <What this module owns.>

use crate::error::Error;

/// <What it does and what it returns.>
pub fn <public_fn>(input: &str) -> Result<<Type>, Error> {
    // ...
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn <describes_behaviour>() {
        let cases: &[(&str, bool)] = &[("ok", true), ("", false)];
        for (input, want_ok) in cases {
            assert_eq!(<public_fn>(input).is_ok(), *want_ok, "input {input:?}");
        }
    }
}
```

## CLI subcommand `crates/shellzip-cli/src/commands/<verb>.rs`

```rust
//! `szip <verb>` — <one line>.

use shellzip_core::Error;

use super::Shared;
use crate::cli::<Verb>Args;

/// Run `szip <verb>` and return the process exit code.
pub fn <verb>(args: &<Verb>Args, shared: &Shared) -> Result<i32, Error> {
    let report = shellzip_core::<op>(&args.archive, shared.sink.as_ref())?;
    if shared.quiet < 2 {
        println!("<summary line>");
    }
    Ok(report.exit_code())
}
```

## Integration test `tests/<scenario>.rs`

```rust
//! <Scenario in one line.>

use std::fs;

use shellzip_core::{CompressOpts, NopProgress, add, list};

#[test]
fn <describes_behaviour>() {
    let tmp = tempfile::tempdir().unwrap();
    let src = tmp.path().join("src");
    fs::create_dir(&src).unwrap();
    fs::write(src.join("hello.txt"), b"hello").unwrap();

    let archive = tmp.path().join("out.zip");
    add(&archive, &[src], &CompressOpts::default(), &NopProgress).unwrap();

    let (_opened, entries) = list(&archive).unwrap();
    assert!(entries.iter().any(|e| e.path.ends_with("hello.txt")));
}
```

## CLI end-to-end case (append to `crates/shellzip-cli/tests/cli.rs`)

```rust
#[test]
fn <verb>_reports_<behaviour>() {
    let tmp = tempfile::tempdir().unwrap();
    let archive = tmp.path().join("a.zip");
    // build fixture with szip() or shellzip_core, then:
    szip()
        .args(["<verb>", archive.to_str().unwrap()])
        .assert()
        .code(0)
        .stdout(contains("<expected text>"));
}
```

## Criterion benchmark `benches/<name>.rs`

```rust
//! Throughput of <operation> over a generated corpus.

use criterion::{Criterion, Throughput, criterion_group, criterion_main};

fn bench_<operation>(c: &mut Criterion) {
    let data = vec![0xA5u8; 8 * 1024 * 1024];
    let mut group = c.benchmark_group("<operation>");
    group.throughput(Throughput::Bytes(data.len() as u64));
    group.bench_function("8MiB", |b| {
        b.iter(|| {
            // call the function under test with std::hint::black_box(&data)
        });
    });
    group.finish();
}

criterion_group!(benches, bench_<operation>);
criterion_main!(benches);
```

`Cargo.toml` additions:

```toml
[dev-dependencies]
criterion = { version = "0.5", features = ["html_reports"] }

[[bench]]
name = "<name>"
harness = false
```

## Example `examples/<name>.rs`

```rust
//! List the members of an archive given on the command line.

use std::path::PathBuf;

fn main() -> Result<(), shellzip_core::Error> {
    let path = PathBuf::from(std::env::args().nth(1).unwrap_or_else(|| "sample.zip".into()));
    let (_opened, entries) = shellzip_core::list(&path)?;
    for entry in entries {
        println!("{}", entry.path); // archive-relative name, `/`-separated
    }
    Ok(())
}
```
