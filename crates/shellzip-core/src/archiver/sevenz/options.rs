//! Turn [`CompressOpts`] into a `zesven` [`WriteOptions`].

use zesven::Password as SevenPassword;
use zesven::codec::CodecMethod;
use zesven::resources::{MemoryLimit, Threads};
use zesven::write::WriteOptions;

use crate::error::Error;
use crate::options::{CompressOpts, SevenZMethod};

/// Compression settings for one 7z archive.
///
/// # Errors
///
/// Returns [`Error::Unsupported`] when `opts.sevenz.method` needs `sevenz-codecs`
/// and this build does not have it, and [`Error::Usage`] when header encryption
/// was requested without a password.
pub(super) fn write_options(opts: &CompressOpts) -> Result<WriteOptions, Error> {
    if opts.sevenz.encrypt_headers && opts.password.is_none() {
        return Err(Error::Usage(
            "encrypting 7z file names needs a password".to_string(),
        ));
    }
    let method = if opts.level == 0 {
        CodecMethod::Copy
    } else {
        codec_method(opts.sevenz.method)?
    };
    let mut options = WriteOptions::new()
        .method(method)
        .level_clamped(opts.sevenz_level())
        .threads(Threads::count_or_single(opts.thread_count()))
        .memory_limit(MemoryLimit::Auto)
        .deterministic(true);
    if opts.sevenz.solid && opts.level > 0 {
        options = options.solid();
    }
    if let Some(comment) = opts.comment.as_ref().filter(|text| !text.is_empty()) {
        options = options.comment(comment.clone());
    }
    if let Some(password) = &opts.password {
        options = options
            .password(SevenPassword::new(password.expose()))
            .encrypt_data(true)
            .encrypt_header(opts.sevenz.encrypt_headers);
    }
    Ok(options)
}

fn codec_method(method: SevenZMethod) -> Result<CodecMethod, Error> {
    let enabled = match method {
        SevenZMethod::Lzma2 | SevenZMethod::Lzma => true,
        SevenZMethod::Ppmd | SevenZMethod::Bzip2 | SevenZMethod::Deflate | SevenZMethod::Zstd => {
            cfg!(feature = "sevenz-codecs")
        }
    };
    if !enabled {
        return Err(Error::Unsupported {
            format: "7Z".to_string(),
            detail: format!("{} needs the `sevenz-codecs` feature", method.label()),
        });
    }
    Ok(match method {
        SevenZMethod::Lzma2 => CodecMethod::Lzma2,
        SevenZMethod::Lzma => CodecMethod::Lzma,
        SevenZMethod::Ppmd => CodecMethod::PPMd,
        SevenZMethod::Bzip2 => CodecMethod::BZip2,
        SevenZMethod::Deflate => CodecMethod::Deflate,
        SevenZMethod::Zstd => CodecMethod::Zstd,
    })
}
