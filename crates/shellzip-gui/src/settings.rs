//! Pack and extract choices stored in a TOML file.
//!
//! The path comes from the `directories` config directory, not the registry.

use shellzip_core::{Overwrite, ZipMethod};

use crate::chrome::PackSettings;

/// Load the saved options, or the defaults when the file is missing.
pub fn load() -> PackSettings {
    let Some(path) = settings_path() else {
        return defaults();
    };
    let Ok(text) = std::fs::read_to_string(path) else {
        return defaults();
    };
    parse(&text).unwrap_or_else(defaults)
}

/// Write `options` when a config directory is available.
pub fn save(options: &PackSettings) -> Result<(), String> {
    let Some(path) = settings_path() else {
        return Ok(());
    };
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|err| format!("create {}: {err}", parent.display()))?;
    }
    let body = serialize(options);
    let tmp = path.with_extension("toml.tmp");
    std::fs::write(&tmp, body).map_err(|err| format!("write {}: {err}", tmp.display()))?;
    std::fs::rename(&tmp, &path).map_err(|err| format!("rename {}: {err}", path.display()))?;
    Ok(())
}

fn defaults() -> PackSettings {
    PackSettings {
        level: 3,
        method: ZipMethod::Deflate,
        overwrite: Overwrite::Never,
        flat: false,
    }
}

fn settings_path() -> Option<std::path::PathBuf> {
    directories::ProjectDirs::from("com", "ShellZip", "shellzip")
        .map(|dirs| dirs.config_dir().join("settings.toml"))
}

fn serialize(options: &PackSettings) -> String {
    format!(
        "\
level = {}
method = \"{}\"
overwrite = \"{}\"
flat = {}
",
        options.level,
        method_name(options.method),
        overwrite_name(options.overwrite),
        options.flat
    )
}

fn parse(text: &str) -> Option<PackSettings> {
    let value: toml::Value = toml::from_str(text).ok()?;
    let table = value.as_table()?;
    let level = table.get("level").and_then(toml::Value::as_integer)?;
    let level = u8::try_from(level).ok()?.min(5);
    let method = match table.get("method").and_then(toml::Value::as_str)? {
        "zstd" => ZipMethod::Zstd,
        _ => ZipMethod::Deflate,
    };
    let overwrite = match table.get("overwrite").and_then(toml::Value::as_str)? {
        "always" => Overwrite::Always,
        "rename" => Overwrite::Rename,
        "ask" => Overwrite::Ask,
        _ => Overwrite::Never,
    };
    let flat = table
        .get("flat")
        .and_then(toml::Value::as_bool)
        .unwrap_or(false);
    Some(PackSettings {
        level,
        method,
        overwrite,
        flat,
    })
}

fn method_name(method: ZipMethod) -> &'static str {
    match method {
        ZipMethod::Deflate => "deflate",
        ZipMethod::Zstd => "zstd",
    }
}

fn overwrite_name(mode: Overwrite) -> &'static str {
    match mode {
        Overwrite::Ask => "ask",
        Overwrite::Always => "always",
        Overwrite::Never => "never",
        Overwrite::Rename => "rename",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trips_the_options_file() {
        let options = PackSettings {
            level: 1,
            method: ZipMethod::Zstd,
            overwrite: Overwrite::Rename,
            flat: true,
        };
        let parsed = parse(&serialize(&options)).expect("settings");
        assert_eq!(parsed, options);
    }
}
