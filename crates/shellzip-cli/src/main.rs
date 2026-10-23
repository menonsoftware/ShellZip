//! `szip` — pack and unpack ZIP, tar, gzip, zstd, xz, and bzip2 archives.

mod cli;
mod commands;
mod ui;

fn main() {
    std::process::exit(cli::run());
}
