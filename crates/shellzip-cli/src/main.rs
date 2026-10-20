//! `szip` — pack and unpack ZIP, tar, gzip, and zstd archives.

mod cli;
mod commands;
mod ui;

fn main() {
    std::process::exit(cli::run());
}
