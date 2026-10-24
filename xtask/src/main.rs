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
    let result = match command.as_str() {
        "bench" => bench(),
        "build-fixtures" => build_fixtures(),
        "sign" => sign(),
        "package-stubs" => package_stubs(),
        _ => {
            eprintln!("usage: xtask bench | build-fixtures | sign <sfx> | package-stubs [stub]");
            std::process::exit(2);
        }
    };
    if let Err(err) = result {
        eprintln!("xtask: {err}");
        std::process::exit(1);
    }
}

fn sign() -> Result<(), Box<dyn std::error::Error>> {
    let path = std::env::args().nth(2).ok_or("usage: xtask sign <sfx>")?;
    let status = Command::new("signtool")
        .args(["sign", "/fd", "SHA256", "/a", &path])
        .status()
        .map_err(|err| {
            format!(
                "signtool was not found on PATH ({err}); the SFX stays unsigned until you sign it"
            )
        })?;
    if status.success() {
        println!("signed {path}");
        Ok(())
    } else {
        Err(format!("signtool failed with {status}").into())
    }
}

fn package_stubs() -> Result<(), Box<dyn std::error::Error>> {
    let built = std::env::args()
        .nth(2)
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            let name = if cfg!(windows) {
                "shellzip-sfx.exe"
            } else {
                "shellzip-sfx"
            };
            PathBuf::from("target").join("minisize").join(name)
        });
    if !built.is_file() {
        return Err(format!(
            "stub {} is missing; run `cargo build -p shellzip-sfx --profile minisize` first",
            built.display()
        )
        .into());
    }
    let shipped = if cfg!(windows) {
        "szip-sfx-x86_64-pc-windows-msvc.exe"
    } else {
        "szip-sfx-x86_64-unknown-linux-gnu"
    };
    let dest_dir = PathBuf::from("dist");
    fs::create_dir_all(&dest_dir)?;
    let dest = dest_dir.join(shipped);
    fs::copy(&built, &dest)?;
    println!("copied {} -> {}", built.display(), dest.display());
    Ok(())
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
    for threads in [1u16, 8] {
        let dest = tmp.join(format!("szip-{threads}.7z"));
        let elapsed = time_pack(&src, &dest, threads, 3)?;
        let packed = fs::metadata(&dest)?.len();
        rows.push(Row {
            tool: format!("szip --threads {threads} -m3"),
            format: "7z".to_string(),
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
        let seven = tmp.join("7z.7z");
        if let Some(elapsed) = time_external(
            &tool,
            &[
                "a",
                "-t7z",
                "-mx=5",
                "-bso0",
                "-bsp0",
                &seven.display().to_string(),
                &src.display().to_string(),
            ],
        ) {
            let packed = fs::metadata(&seven).map(|meta| meta.len()).unwrap_or(0);
            rows.push(Row {
                tool: "7z a -t7z -mx=5".to_string(),
                format: "7z".to_string(),
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
    time_pack(src, dest, threads, 5)
}

fn time_pack(
    src: &Path,
    dest: &Path,
    threads: u16,
    level: u8,
) -> Result<std::time::Duration, std::io::Error> {
    let _ = fs::remove_file(dest);
    let opts = CompressOpts {
        level,
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

/// Fixture password. A test vector, not a credential. Never printed.
const FIXTURE_PASSWORD: &str = "szip-fixture";

fn build_fixtures() -> Result<(), Box<dyn std::error::Error>> {
    let tool = find_rar().ok_or("Rar.exe was not found")?;
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../fixtures");
    let dir = root.join("rar");
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir)?;
    let tmp = std::env::temp_dir().join(format!("szip-fixtures-{}", std::process::id()));
    let _ = fs::remove_dir_all(&tmp);
    fs::create_dir_all(&tmp)?;
    let hello = tmp.join("hello.txt");
    let alpha = tmp.join("a.txt");
    let bravo = tmp.join("b.txt");
    let big = tmp.join("big.bin");
    fs::write(&hello, b"hello\n")?;
    fs::write(&alpha, b"alpha\n")?;
    fs::write(&bravo, b"bravo\n")?;
    fs::write(&big, vec![0xA5; 30_000])?;

    let password = format!("-p{FIXTURE_PASSWORD}");
    let headers = format!("-hp{FIXTURE_PASSWORD}");
    rar_add(
        &tool,
        &dir.join("store.rar"),
        &["-ma5", "-m0", "-ep"],
        &[&hello],
    )?;
    rar_add(
        &tool,
        &dir.join("password.rar"),
        &["-ma5", "-m0", "-ep", &password],
        &[&hello],
    )?;
    rar_add(
        &tool,
        &dir.join("headers.rar"),
        &["-ma5", "-m0", "-ep", &headers],
        &[&hello],
    )?;
    rar_add(
        &tool,
        &dir.join("solid.rar"),
        &["-ma5", "-s", "-m3", "-ep"],
        &[&alpha, &bravo],
    )?;
    // WinRAR 7 dropped `-ma4`. Write a stored RAR 4.x archive directly.
    write_rar4_store(&dir.join("rar4.rar"), "hello.txt", b"hello\n")?;
    rar_add(
        &tool,
        &dir.join("blake.rar"),
        &["-ma5", "-m0", "-ep", "-htb"],
        &[&hello],
    )?;
    rar_add(
        &tool,
        &dir.join("volumes.rar"),
        &["-ma5", "-m0", "-ep", "-v10k"],
        &[&big],
    )?;
    if dir.join("volumes.rar").is_file() && volume_parts(&dir).len() >= 2 {
        fs::remove_file(dir.join("volumes.rar"))?;
    }
    match find_tool("7z") {
        Some(tool) => build_sevenz_fixtures(&tool, &root, &tmp)?,
        None => println!("7z was not on PATH; skipped fixtures/sevenz"),
    }
    write_sums(&root)?;
    let count = fs::read_dir(&dir)?.count();
    println!("wrote {count} RAR fixtures under {}", dir.display());
    let _ = fs::remove_dir_all(&tmp);
    Ok(())
}

fn build_sevenz_fixtures(
    tool: &Path,
    root: &Path,
    tmp: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    let dir = root.join("sevenz");
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir)?;
    let hello = tmp.join("hello.txt");
    let alpha = tmp.join("a.txt");
    let bravo = tmp.join("b.txt");
    let big = tmp.join("big.bin");
    let sevenz =
        |extra: &[&str], dest: &Path, files: &[&Path]| -> Result<(), Box<dyn std::error::Error>> {
            let mut args = vec!["a", "-y", "-bso0", "-bsp0"];
            args.extend_from_slice(extra);
            args.push(dest.to_str().unwrap_or("."));
            for file in files {
                args.push(file.to_str().unwrap_or("."));
            }
            let status = Command::new(tool).args(&args).status()?;
            if status.success() {
                Ok(())
            } else {
                Err(format!("7z failed while writing {}", dest.display()).into())
            }
        };
    sevenz(&["-m0"], &dir.join("store.7z"), &[&hello])?;
    sevenz(
        &["-ms=on", "-mx=5"],
        &dir.join("solid.7z"),
        &[&alpha, &bravo],
    )?;
    sevenz(&["-m0=PPMd"], &dir.join("ppmd.7z"), &[&hello])?;
    sevenz(&["-mf=BCJ2"], &dir.join("bcj2.7z"), &[&big])?;
    sevenz(
        &[&format!("-p{FIXTURE_PASSWORD}"), "-mhe=on", "-m0"],
        &dir.join("headers.7z"),
        &[&hello],
    )?;
    sevenz(&["-v10k", "-m0"], &dir.join("volumes.7z"), &[&big])?;
    println!(
        "wrote 7z fixtures under {} (7z {})",
        dir.display(),
        tool.display()
    );
    Ok(())
}

fn volume_parts(dir: &Path) -> Vec<PathBuf> {
    let mut parts = Vec::new();
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let name = entry.file_name();
            let text = name.to_string_lossy();
            if text.starts_with("volumes.part") {
                parts.push(entry.path());
            }
        }
    }
    parts.sort();
    parts
}

fn rar_add(
    tool: &Path,
    archive: &Path,
    switches: &[&str],
    files: &[&Path],
) -> Result<(), Box<dyn std::error::Error>> {
    let mut args = vec!["a".to_string(), "-y".to_string(), "-inul".to_string()];
    args.extend(switches.iter().map(|item| (*item).to_string()));
    args.push(archive.display().to_string());
    for file in files {
        args.push(file.display().to_string());
    }
    let status = Command::new(tool).args(&args).status()?;
    if status.success() {
        Ok(())
    } else {
        Err(format!("Rar.exe failed while writing {}", archive.display()).into())
    }
}

fn find_rar() -> Option<PathBuf> {
    for candidate in [
        r"C:\Program Files\WinRAR\Rar.exe",
        r"C:\Program Files\WinRAR\rar.exe",
        r"C:\Program Files (x86)\WinRAR\Rar.exe",
    ] {
        let path = PathBuf::from(candidate);
        if path.is_file() {
            return Some(path);
        }
    }
    find_tool("Rar").or_else(|| find_tool("rar"))
}

fn write_sums(root: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let mut files = Vec::new();
    collect_files(root, &mut files)?;
    files.sort();
    let mut body = String::new();
    for file in files {
        if file.file_name().and_then(|name| name.to_str()) == Some("SHA256SUMS") {
            continue;
        }
        let rel = file.strip_prefix(root)?;
        let hash = sha256_file(&file)?;
        let text = rel.to_string_lossy().replace('\\', "/");
        body.push_str(&format!("{hash}  {text}\n"));
    }
    fs::write(root.join("SHA256SUMS"), body)?;
    Ok(())
}

fn collect_files(dir: &Path, out: &mut Vec<PathBuf>) -> Result<(), std::io::Error> {
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            collect_files(&path, out)?;
        } else if path.is_file() {
            out.push(path);
        }
    }
    Ok(())
}

/// Stored RAR 4.x member. WinRAR 7 no longer accepts `-ma4`.
fn write_rar4_store(path: &Path, name: &str, data: &[u8]) -> Result<(), std::io::Error> {
    let name_bytes = name.as_bytes();
    let mut file_header = Vec::new();
    file_header.extend_from_slice(&[0, 0]);
    file_header.push(0x74);
    file_header.extend_from_slice(&0u16.to_le_bytes());
    let header_size = 32 + name_bytes.len();
    file_header.extend_from_slice(&u16::try_from(header_size).unwrap_or(0).to_le_bytes());
    file_header.extend_from_slice(&u32::try_from(data.len()).unwrap_or(0).to_le_bytes());
    file_header.extend_from_slice(&u32::try_from(data.len()).unwrap_or(0).to_le_bytes());
    file_header.push(2);
    file_header.extend_from_slice(&crc32(data).to_le_bytes());
    file_header.extend_from_slice(&0u32.to_le_bytes());
    file_header.push(20);
    file_header.push(0x30);
    file_header.extend_from_slice(&u16::try_from(name_bytes.len()).unwrap_or(0).to_le_bytes());
    file_header.extend_from_slice(&0x20u32.to_le_bytes());
    file_header.extend_from_slice(name_bytes);
    stamp_header_crc(&mut file_header);

    let mut main = vec![0, 0, 0x73, 0, 0, 13, 0, 0, 0, 0, 0, 0, 0];
    stamp_header_crc(&mut main);
    let mut end = vec![0, 0, 0x7b, 0, 0, 7, 0];
    stamp_header_crc(&mut end);

    let mut archive = Vec::new();
    archive.extend_from_slice(b"Rar!\x1a\x07\x00");
    archive.extend_from_slice(&main);
    archive.extend_from_slice(&file_header);
    archive.extend_from_slice(data);
    archive.extend_from_slice(&end);
    fs::write(path, archive)
}

fn stamp_header_crc(header: &mut [u8]) {
    let crc = crc32(&header[2..]) as u16;
    header[0] = crc.to_le_bytes()[0];
    header[1] = crc.to_le_bytes()[1];
}

fn crc32(data: &[u8]) -> u32 {
    let mut crc = 0xffff_ffffu32;
    for &byte in data {
        crc ^= u32::from(byte);
        for _ in 0..8 {
            let mask = (crc & 1).wrapping_neg();
            crc = (crc >> 1) ^ (0xEDB8_8320 & mask);
        }
    }
    !crc
}

fn sha256_file(path: &Path) -> Result<String, std::io::Error> {
    use sha2::{Digest, Sha256};
    let bytes = fs::read(path)?;
    let digest = Sha256::digest(bytes);
    Ok(digest.iter().map(|byte| format!("{byte:02x}")).collect())
}

fn render(rows: &[Row], bytes: u64, speedup: f64) -> String {
    let mut out = String::new();
    out.push_str("# ShellZip benchmarks\n\n");
    out.push_str(
        "Generated by `cargo run --release -p xtask -- bench`. The corpus is 16 text files of 2 MiB. \
         `szip` ZIP uses `-m5` (deflate level 9). `szip` 7z uses `-m3` (LZMA2, solid). \
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
