//! `cargo run -p xtask --release -- bench` times ZIP creation and, when they
//! are installed, 7-Zip and WinRAR on the same corpus.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Instant;

use shellzip_core::{CompressOpts, NopProgress, add};

fn main() {
    let command = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "bench".to_string());
    if command != "bench" {
        eprintln!("usage: xtask bench");
        std::process::exit(2);
    }
    if let Err(err) = bench() {
        eprintln!("xtask: {err}");
        std::process::exit(1);
    }
}

fn bench() -> Result<(), Box<dyn std::error::Error>> {
    let tmp = std::env::temp_dir().join(format!("szip-bench-{}", std::process::id()));
    let _ = fs::remove_dir_all(&tmp);
    let src = tmp.join("src");
    fs::create_dir_all(&src)?;
    for index in 0..16u32 {
        fs::write(
            src.join(format!("file-{index:02}.txt")),
            payload(index, 2 * 1024 * 1024),
        )?;
    }
    let bytes = fs::read_dir(&src)?.try_fold(0u64, |total, entry| {
        let entry = entry?;
        Ok::<u64, std::io::Error>(total + entry.metadata()?.len())
    })?;

    let mut rows = Vec::new();
    for threads in [1u16, 8] {
        let dest = tmp.join(format!("szip-{threads}.zip"));
        let elapsed = time_szip(&src, &dest, threads)?;
        let packed = fs::metadata(&dest)?.len();
        rows.push(Row {
            tool: format!("szip --threads {threads}"),
            format: "zip".to_string(),
            elapsed,
            packed,
        });
    }
    if let Some(tool) = find_tool("7z") {
        let dest = tmp.join("7z.zip");
        if let Some(elapsed) = time_external(
            &tool,
            &[
                "a",
                "-tzip",
                "-mx=5",
                "-bso0",
                "-bsp0",
                &dest.display().to_string(),
                &src.display().to_string(),
            ],
        ) {
            let packed = fs::metadata(&dest).map(|meta| meta.len()).unwrap_or(0);
            rows.push(Row {
                tool: "7z a -tzip -mx=5".to_string(),
                format: "zip".to_string(),
                elapsed,
                packed,
            });
        }
    }
    if let Some(tool) = find_tool("Rar").or_else(|| find_tool("rar")) {
        let dest = tmp.join("rar.rar");
        if let Some(elapsed) = time_external(
            &tool,
            &[
                "a",
                "-m5",
                "-inul",
                &dest.display().to_string(),
                &src.display().to_string(),
            ],
        ) {
            let packed = fs::metadata(&dest).map(|meta| meta.len()).unwrap_or(0);
            rows.push(Row {
                tool: "Rar a -m5".to_string(),
                format: "rar".to_string(),
                elapsed,
                packed,
            });
        }
    }

    let single = rows.iter().find(|row| row.tool.contains("threads 1"));
    let multi = rows.iter().find(|row| row.tool.contains("threads 8"));
    let speedup = match (single, multi) {
        (Some(single), Some(multi)) if multi.elapsed.as_secs_f64() > 0.0 => {
            single.elapsed.as_secs_f64() / multi.elapsed.as_secs_f64()
        }
        _ => 0.0,
    };
    let markdown = render(&rows, bytes, speedup);
    let doc = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../docs/BENCH.md");
    fs::write(&doc, &markdown)?;
    println!("{markdown}");
    println!("wrote {}", doc.display());
    let _ = fs::remove_dir_all(&tmp);
    Ok(())
}

struct Row {
    tool: String,
    format: String,
    elapsed: std::time::Duration,
    packed: u64,
}

fn time_szip(src: &Path, dest: &Path, threads: u16) -> Result<std::time::Duration, std::io::Error> {
    let _ = fs::remove_file(dest);
    let opts = CompressOpts {
        level: 5,
        threads,
        ..CompressOpts::default()
    };
    let start = Instant::now();
    add(dest, &[src.to_path_buf()], &opts, &NopProgress).map_err(std::io::Error::other)?;
    Ok(start.elapsed())
}

fn time_external(tool: &Path, args: &[&str]) -> Option<std::time::Duration> {
    let start = Instant::now();
    let status = Command::new(tool).args(args).status().ok()?;
    if status.success() {
        Some(start.elapsed())
    } else {
        None
    }
}

fn find_tool(name: &str) -> Option<PathBuf> {
    let path = std::env::var_os("PATH")?;
    for dir in std::env::split_paths(&path) {
        let plain = dir.join(name);
        if plain.is_file() {
            return Some(plain);
        }
        let exe = dir.join(format!("{name}.exe"));
        if exe.is_file() {
            return Some(exe);
        }
    }
    None
}

fn payload(index: u32, len: usize) -> Vec<u8> {
    const WORDS: &[&str] = &[
        "alpha", "bravo", "charlie", "delta", "echo", "foxtrot", "golf", "hotel", "india",
        "juliet", "kilo", "lima", "mike", "november", "oscar", "papa", "quebec", "romeo", "sierra",
        "tango", "uniform", "victor", "whiskey", "xray", "yankee", "zulu",
    ];
    let mut state = 0x1234_0000u32 ^ index.wrapping_mul(0x9E37_79B9);
    let mut out = String::with_capacity(len);
    while out.len() < len {
        state = state.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
        let words = (state % 8) + 2;
        out.push_str("member ");
        out.push_str(&index.to_string());
        out.push(' ');
        for _ in 0..words {
            state = state.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
            out.push_str(WORDS[(state as usize) % WORDS.len()]);
            out.push(' ');
        }
        out.push('\n');
    }
    let mut bytes = out.into_bytes();
    bytes.truncate(len);
    bytes
}

fn render(rows: &[Row], bytes: u64, speedup: f64) -> String {
    let mut out = String::new();
    out.push_str("# ShellZip benchmarks\n\n");
    out.push_str(
        "Generated by `cargo run --release -p xtask -- bench`. The corpus is 16 text files of 2 MiB. \
         `szip` uses `-m5` (deflate level 9). \
         7-Zip and WinRAR are included only when they are on `PATH`. ZIP creation with `--threads 8` \
         is compared with `--threads 1`. Workers compress entries in parallel and one thread merges \
         them in order, so the ratio tracks how much of the time is spent in deflate.\n\n",
    );
    out.push_str(&format!("Uncompressed input: {bytes} bytes.\n\n"));
    out.push_str("| Tool | Format | Seconds | MB/s | Packed bytes |\n");
    out.push_str("| --- | --- | ---: | ---: | ---: |\n");
    for row in rows {
        let secs = row.elapsed.as_secs_f64();
        let mb_s = if secs > 0.0 {
            (bytes as f64 / 1_000_000.0) / secs
        } else {
            0.0
        };
        out.push_str(&format!(
            "| {} | {} | {secs:.3} | {mb_s:.1} | {} |\n",
            row.tool, row.format, row.packed
        ));
    }
    out.push_str(&format!(
        "\n`--threads 8` / `--threads 1` speedup: {speedup:.2}×. \
         Matching packed sizes mean the parallel merge wrote the same compressed bytes as one thread.\n"
    ));
    out
}
