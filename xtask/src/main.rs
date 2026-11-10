//! `cargo run -p xtask --release -- bench` times ZIP creation and, when they
//! are installed, 7-Zip and WinRAR on the same corpus.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Instant;

use shellzip_core::{CompressOpts, NopProgress, add};
use shellzip_integrate::EXTENSIONS;

fn main() {
    let command = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "bench".to_string());
    let result = match command.as_str() {
        "bench" => bench(),
        "build-fixtures" => build_fixtures(),
        "package-stubs" => package_stubs(),
        "gen-file-icons" => gen_file_icons(),
        "velopack-stage" => velopack_stage(),
        "velopack-pack" => velopack_pack(),
        "msix-pack" => msix_pack(),
        "msix-register-shell" => register_shell_from_args(),
        "gpg-sign-release" => gpg_sign_release(),
        _ => {
            eprintln!(
                "usage: xtask bench | build-fixtures | package-stubs [stub] | gen-file-icons | velopack-stage | velopack-pack | msix-pack [--install] | msix-register-shell <exe> <dll> <icons> | gpg-sign-release <dir>"
            );
            std::process::exit(2);
        }
    };
    if let Err(err) = result {
        eprintln!("xtask: {err}");
        std::process::exit(1);
    }
}

/// Velopack packId — keep in sync with `shellzip-gui` bundle identifier.
const VELOPACK_ID: &str = "com.shellzip.ShellZip";

fn workspace_version() -> Result<String, Box<dyn std::error::Error>> {
    let text = fs::read_to_string("Cargo.toml")?;
    version_from_workspace_toml(&text)
        .ok_or_else(|| "workspace.package.version not found in Cargo.toml".into())
}

/// Parse `version = "..."` from a `[workspace.package]` table. The key must be the
/// complete token `version` (so `version_foo` / `version_override` do not match).
fn version_from_workspace_toml(text: &str) -> Option<String> {
    let mut in_package = false;
    for line in text.lines() {
        let trimmed = line.trim();
        if trimmed == "[workspace.package]" {
            in_package = true;
            continue;
        }
        if !in_package {
            continue;
        }
        if trimmed.starts_with('[') {
            break;
        }
        let Some(rest) = trimmed.strip_prefix("version") else {
            continue;
        };
        if !matches!(rest.chars().next(), Some(c) if c == '=' || c.is_whitespace()) {
            continue;
        }
        let value = rest.trim().trim_start_matches('=').trim().trim_matches('"');
        if !value.is_empty() {
            return Some(value.to_string());
        }
    }
    None
}

fn host_channel() -> &'static str {
    if cfg!(windows) {
        "win"
    } else if cfg!(target_os = "macos") {
        "osx"
    } else {
        "linux"
    }
}

fn host_exe(name: &str) -> String {
    if cfg!(windows) {
        format!("{name}.exe")
    } else {
        name.to_string()
    }
}

fn copy_tree(src: &Path, dest: &Path) -> Result<(), Box<dyn std::error::Error>> {
    if !src.is_dir() {
        return Err(format!("missing directory {}", src.display()).into());
    }
    fs::create_dir_all(dest)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let from = entry.path();
        let to = dest.join(entry.file_name());
        if from.is_dir() {
            copy_tree(&from, &to)?;
        } else {
            if let Some(parent) = to.parent() {
                fs::create_dir_all(parent)?;
            }
            fs::copy(&from, &to)?;
        }
    }
    Ok(())
}

/// Stage release binaries and format icons into `dist/velopack/<channel>/`.
fn velopack_stage() -> Result<(), Box<dyn std::error::Error>> {
    let channel = std::env::args()
        .nth(2)
        .unwrap_or_else(|| host_channel().to_string());
    let target = std::env::var("CARGO_BUILD_TARGET").unwrap_or_default();
    let release_dir = if target.is_empty() {
        PathBuf::from("target").join("release")
    } else {
        PathBuf::from("target").join(&target).join("release")
    };
    let minisize_dir = if target.is_empty() {
        PathBuf::from("target").join("minisize")
    } else {
        PathBuf::from("target").join(&target).join("minisize")
    };

    let stage = PathBuf::from("dist").join("velopack").join(&channel);
    if stage.exists() {
        fs::remove_dir_all(&stage)?;
    }
    fs::create_dir_all(&stage)?;

    let gui = host_exe("shellzip");
    let cli = host_exe("szip");
    let sfx = host_exe("shellzip-sfx");
    for (src_dir, name) in [
        (&release_dir, gui.as_str()),
        (&release_dir, cli.as_str()),
        (&minisize_dir, sfx.as_str()),
    ] {
        let src = src_dir.join(name);
        if !src.is_file() {
            return Err(format!("missing {}; build release binaries first", src.display()).into());
        }
        fs::copy(&src, stage.join(name))?;
    }

    if cfg!(windows) {
        let dll = release_dir.join("shellzip_shell.dll");
        if dll.is_file() {
            fs::copy(&dll, stage.join("shellzip_shell.dll"))?;
        } else {
            eprintln!(
                "xtask: warning: {} missing; Windows shell menus need the DLL",
                dll.display()
            );
        }
    }

    let icons_src = PathBuf::from("grx").join("shellzip_file_icons");
    let icons_dest = stage.join("icons");
    copy_tree(&icons_src, &icons_dest)?;
    println!("staged {}", stage.display());
    Ok(())
}

/// Run `vpk pack` against a staged directory (requires .NET 8 + global `vpk`).
fn velopack_pack() -> Result<(), Box<dyn std::error::Error>> {
    let channel = std::env::args()
        .nth(2)
        .unwrap_or_else(|| host_channel().to_string());
    let version = std::env::var("VELOPACK_VERSION")
        .or_else(|_| workspace_version().map_err(|e| e.to_string()))
        .map_err(|e| -> Box<dyn std::error::Error> { e.into() })?;
    let stage = PathBuf::from("dist").join("velopack").join(&channel);
    if !stage.is_dir() {
        return Err(format!(
            "missing {}; run `cargo run -p xtask -- velopack-stage` first",
            stage.display()
        )
        .into());
    }

    let main_exe = host_exe("shellzip");
    let icon = match channel.as_str() {
        "win" => PathBuf::from("crates/shellzip-gui/assets/shellzip.ico"),
        "osx" => PathBuf::from("crates/shellzip-gui/assets/shellzip.icns"),
        _ => PathBuf::from("crates/shellzip-gui/assets/icon.png"),
    };
    if !icon.is_file() {
        return Err(format!("missing icon {}", icon.display()).into());
    }

    let output = PathBuf::from("dist")
        .join("velopack")
        .join(format!("Releases-{channel}"));
    fs::create_dir_all(&output)?;

    let args = vec![
        "pack".into(),
        "-u".into(),
        VELOPACK_ID.into(),
        "-v".into(),
        version.clone(),
        "-p".into(),
        stage.display().to_string(),
        "-e".into(),
        main_exe,
        "--packTitle".into(),
        "ShellZip".into(),
        "-c".into(),
        channel.clone(),
        "-i".into(),
        icon.display().to_string(),
        "-o".into(),
        output.display().to_string(),
    ];

    if channel == "win" {
        eprintln!(
            "xtask: Windows Velopack pack is unsigned (Store MSIX is signed by Partner Center)"
        );
    }

    let status = Command::new("vpk").args(&args).status().map_err(|err| {
        format!(
            "vpk was not found on PATH ({err}); install .NET 8 and `dotnet tool install -g vpk`"
        )
    })?;
    if !status.success() {
        return Err(format!("vpk pack failed with {status}").into());
    }
    println!(
        "packed {VELOPACK_ID} {version} channel={channel} -> {}",
        output.display()
    );

    if channel == "linux" {
        let _ = gpg_sign_dir(&output);
    }
    Ok(())
}

/// GitHub Actions injects missing secrets as empty strings, so `var_os` alone is
/// not enough — treat blank values as unset.
fn env_nonempty(name: &str) -> Option<String> {
    std::env::var(name).ok().and_then(|v| {
        let trimmed = v.trim();
        if trimmed.is_empty() {
            None
        } else {
            Some(trimmed.to_string())
        }
    })
}

/// Password for the local self-signed MSIX cert (dev sideload only).
#[cfg(windows)]
const MSIX_DEV_CERT_PASSWORD: &str = "ShellZip-Dev";

/// Build a Store MSIX from the release Windows exes.
///
/// Pass `--install` to self-sign with a local developer certificate and
/// sideload via `Add-AppxPackage`. Release CI omits that flag so Partner
/// Center can re-sign the unsigned package.
fn msix_pack() -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(not(windows))]
    {
        Err("msix-pack is Windows-only".into())
    }
    #[cfg(windows)]
    {
        let install = std::env::args().skip(2).any(|arg| arg == "--install");
        let (out_msix, publisher) = msix_pack_windows()?;
        if install {
            msix_sign_and_install(&out_msix, &publisher)?;
        }
        Ok(())
    }
}

#[cfg(windows)]
fn msix_pack_windows() -> Result<(PathBuf, String), Box<dyn std::error::Error>> {
    let version = std::env::var("VELOPACK_VERSION")
        .or_else(|_| workspace_version().map_err(|e| e.to_string()))
        .map_err(|e| -> Box<dyn std::error::Error> { e.into() })?;
    let msix_ver = msix_version(&version)?;
    let identity_name = env_nonempty("STORE_IDENTITY_NAME").unwrap_or_else(|| "ShellZip".into());
    let publisher = env_nonempty("STORE_PUBLISHER").unwrap_or_else(|| "CN=ShellZip".into());
    let publisher_display =
        env_nonempty("STORE_PUBLISHER_DISPLAY_NAME").unwrap_or_else(|| "ShellZip".into());

    let target = std::env::var("CARGO_BUILD_TARGET").unwrap_or_default();
    let release_dir = if target.is_empty() {
        PathBuf::from("target").join("release")
    } else {
        PathBuf::from("target").join(&target).join("release")
    };
    let minisize_dir = if target.is_empty() {
        PathBuf::from("target").join("minisize")
    } else {
        PathBuf::from("target").join(&target).join("minisize")
    };

    let root = PathBuf::from("dist").join("msix");
    let payload = root.join("payload");
    // Refresh the payload and previous .msix files, but keep ShellZipDev.*
    // certificates so `msix-pack --install` can reuse them.
    if payload.exists() {
        fs::remove_dir_all(&payload)?;
    }
    if root.is_dir() {
        for entry in fs::read_dir(&root)? {
            let path = entry?.path();
            if path
                .extension()
                .is_some_and(|ext| ext.eq_ignore_ascii_case("msix"))
            {
                fs::remove_file(&path)?;
            }
        }
    }
    let assets = payload.join("Assets");
    fs::create_dir_all(&assets)?;

    for (src_dir, name) in [
        (&release_dir, "shellzip.exe"),
        (&release_dir, "szip.exe"),
        (&release_dir, "shellzip_shell.dll"),
        (&minisize_dir, "shellzip-sfx.exe"),
    ] {
        let src = src_dir.join(name);
        if !src.is_file() {
            return Err(format!(
                "missing {}; build release Windows binaries first",
                src.display()
            )
            .into());
        }
        fs::copy(&src, payload.join(name))?;
    }

    let icon_src = PathBuf::from("crates/shellzip-gui/assets/icon.png");
    write_resized_png(&icon_src, &assets.join("Square44x44Logo.png"), 44)?;
    write_resized_png(&icon_src, &assets.join("Square150x150Logo.png"), 150)?;
    write_resized_png(&icon_src, &assets.join("StoreLogo.png"), 50)?;

    let mime_icons = PathBuf::from("grx")
        .join("shellzip_file_icons")
        .join("linux")
        .join("hicolor")
        .join("256x256")
        .join("mimetypes");
    for kind in EXTENSIONS {
        let mime_file = format!("{}.png", kind.mime.replace('/', "-"));
        let src = mime_icons.join(&mime_file);
        if !src.is_file() {
            return Err(format!("missing file-type icon {}", src.display()).into());
        }
        let dest_name = format!("FileType{}.png", kind.extension.replace('.', "_"));
        fs::copy(&src, assets.join(&dest_name))?;
    }

    let manifest = build_appx_manifest(&identity_name, &publisher, &publisher_display, &msix_ver);
    fs::write(payload.join("AppxManifest.xml"), &manifest)?;

    let tag = if version.starts_with('v') {
        version.clone()
    } else {
        format!("v{version}")
    };
    let out_msix = root.join(format!("ShellZip-{tag}-x64.msix"));
    let makeappx = find_sdk_tool("makeappx.exe")?;
    let status = Command::new(&makeappx)
        .args([
            "pack",
            "/d",
            &payload.display().to_string(),
            "/p",
            &out_msix.display().to_string(),
            "/o",
        ])
        .status()
        .map_err(|err| format!("makeappx failed to start ({err})"))?;
    if !status.success() {
        return Err(format!("makeappx pack failed with {status}").into());
    }
    println!("packed {}", out_msix.display());
    Ok((out_msix, publisher))
}

/// Create (or reuse) a self-signed cert, sign the MSIX, and sideload it.
#[cfg(windows)]
fn msix_sign_and_install(msix: &Path, publisher: &str) -> Result<(), Box<dyn std::error::Error>> {
    let cert_dir = msix
        .parent()
        .ok_or("msix path has no parent directory")?
        .to_path_buf();
    let (pfx, cer) = ensure_msix_dev_certificate(publisher, &cert_dir)?;
    trust_msix_dev_certificate(&cer)?;
    sign_msix(msix, &pfx)?;
    add_appx_package(msix)?;
    // Server 2019 / Windows 10 1809 have no PackagedCom, so the manifest's
    // IExplorerCommand never reaches Explorer. Register the classic menu.
    stop_explorer();
    if let Err(err) = register_classic_explorer_menu() {
        start_explorer();
        return Err(err);
    }
    start_explorer();
    println!("installed {}", msix.display());
    Ok(())
}

/// Register the classic `IContextMenu` against the installed package binaries.
#[cfg(windows)]
fn register_classic_explorer_menu() -> Result<(), Box<dyn std::error::Error>> {
    let install = installed_package_dir()?;
    let exe = install.join("shellzip.exe");
    let dll = install.join("shellzip_shell.dll");
    let icons = std::env::current_dir()
        .unwrap_or_else(|_| PathBuf::from("."))
        .join("grx")
        .join("shellzip_file_icons");
    if !exe.is_file() || !dll.is_file() {
        return Err(format!(
            "installed package is missing shellzip.exe or shellzip_shell.dll under {}",
            install.display()
        )
        .into());
    }
    match register_shell_at(&exe, &dll, &icons) {
        Ok(()) => Ok(()),
        Err(err) if err.to_lowercase().contains("administrator") => {
            elevate_shell_registration(&exe, &dll, &icons)
        }
        Err(err) => Err(err.into()),
    }
}

/// `msix-register-shell <exe> <dll> <icons>` — elevated half of menu registration.
fn register_shell_from_args() -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(not(windows))]
    {
        Err("msix-register-shell is Windows-only".into())
    }
    #[cfg(windows)]
    {
        let exe = std::env::args()
            .nth(2)
            .ok_or("usage: xtask msix-register-shell <exe> <dll> <icons>")?;
        let dll = std::env::args()
            .nth(3)
            .ok_or("usage: xtask msix-register-shell <exe> <dll> <icons>")?;
        let icons = std::env::args()
            .nth(4)
            .ok_or("usage: xtask msix-register-shell <exe> <dll> <icons>")?;
        register_shell_at(Path::new(&exe), Path::new(&dll), Path::new(&icons))?;
        Ok(())
    }
}

#[cfg(windows)]
fn register_shell_at(exe: &Path, dll: &Path, icons: &Path) -> Result<(), String> {
    let paths = shellzip_integrate::IntegratePaths {
        exe: exe.to_path_buf(),
        icons_source: icons.to_path_buf(),
        shell_dll: Some(dll.to_path_buf()),
    };
    shellzip_integrate::integrate(&paths).map_err(|err| err.to_string())?;
    println!("registered classic Explorer menu for {}", exe.display());
    Ok(())
}

#[cfg(windows)]
fn elevate_shell_registration(
    exe: &Path,
    dll: &Path,
    icons: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    let xtask = std::env::current_exe()?;
    let script = format!(
        r#"
$ErrorActionPreference = 'Stop'
$p = Start-Process -FilePath {xtask} -Verb RunAs -Wait -PassThru -ArgumentList @(
  'msix-register-shell',
  {exe},
  {dll},
  {icons}
)
if ($null -eq $p -or $p.ExitCode -ne 0) {{
  throw 'could not register the Explorer menu (approve the UAC prompt)'
}}
"#,
        xtask = ps_single_quoted(&xtask.to_string_lossy()),
        exe = ps_single_quoted(&exe.to_string_lossy()),
        dll = ps_single_quoted(&dll.to_string_lossy()),
        icons = ps_single_quoted(&icons.to_string_lossy()),
    );
    run_powershell(&script).map_err(|err| -> Box<dyn std::error::Error> {
        format!("classic Explorer menu registration failed ({err})").into()
    })?;
    println!("registered classic Explorer menu for {}", exe.display());
    Ok(())
}

#[cfg(windows)]
fn installed_package_dir() -> Result<PathBuf, Box<dyn std::error::Error>> {
    let identity = env_nonempty("STORE_IDENTITY_NAME").unwrap_or_else(|| "ShellZip".into());
    let script = format!(
        r#"
$ErrorActionPreference = 'Stop'
$identity = {identity}
$pkg = Get-AppxPackage | Where-Object {{ $_.Name -eq $identity }} | Select-Object -First 1
if (-not $pkg) {{ throw "package $identity is not installed" }}
Write-Output $pkg.InstallLocation
"#,
        identity = ps_single_quoted(&identity),
    );
    let output = Command::new("powershell")
        .args([
            "-NoProfile",
            "-NonInteractive",
            "-ExecutionPolicy",
            "Bypass",
            "-Command",
        ])
        .arg(script)
        .output()?;
    if !output.status.success() {
        let detail = String::from_utf8_lossy(&output.stderr);
        return Err(format!("could not locate installed package ({detail})").into());
    }
    let location = String::from_utf8_lossy(&output.stdout);
    let location = location.lines().last().unwrap_or("").trim();
    if location.is_empty() {
        return Err("installed package location was empty".into());
    }
    Ok(PathBuf::from(location))
}

/// Stop Explorer so it releases `shellzip_shell.dll` before we replace it.
#[cfg(windows)]
fn stop_explorer() {
    let _ = run_powershell(
        r#"
$ErrorActionPreference = 'SilentlyContinue'
Stop-Process -Name explorer -Force
"#,
    );
}

/// Start Explorer after the classic menu registration is in place.
#[cfg(windows)]
fn start_explorer() {
    if let Err(err) = run_powershell(
        r#"
$ErrorActionPreference = 'SilentlyContinue'
Start-Process explorer.exe
"#,
    ) {
        eprintln!("xtask: warning: could not start Explorer ({err}); start it from Task Manager");
    } else {
        println!("restarted Explorer");
    }
}

/// Ensure `ShellZipDev.pfx` / `.cer` exist for `publisher` under `dir`.
#[cfg(windows)]
fn ensure_msix_dev_certificate(
    publisher: &str,
    dir: &Path,
) -> Result<(PathBuf, PathBuf), Box<dyn std::error::Error>> {
    fs::create_dir_all(dir)?;
    let pfx = dir.join("ShellZipDev.pfx");
    let cer = dir.join("ShellZipDev.cer");
    let subject_file = dir.join("ShellZipDev.subject");
    let reuse = pfx.is_file()
        && cer.is_file()
        && subject_file
            .is_file()
            .then(|| fs::read_to_string(&subject_file).ok())
            .flatten()
            .is_some_and(|saved| saved.trim() == publisher);
    if reuse {
        println!("reusing {}", pfx.display());
        return Ok((pfx, cer));
    }

    // Subject must match AppxManifest Identity Publisher exactly.
    let script = format!(
        r#"
$ErrorActionPreference = 'Stop'
$subject = {subject}
$password = ConvertTo-SecureString -String {password} -Force -AsPlainText
$pfx = {pfx}
$cer = {cer}
Get-ChildItem Cert:\CurrentUser\My |
  Where-Object {{ $_.Subject -eq $subject -and $_.FriendlyName -eq 'ShellZip Dev' }} |
  Remove-Item -Force -ErrorAction SilentlyContinue
$cert = New-SelfSignedCertificate `
  -Type Custom `
  -Subject $subject `
  -KeyUsage DigitalSignature `
  -FriendlyName 'ShellZip Dev' `
  -CertStoreLocation 'Cert:\CurrentUser\My' `
  -TextExtension @(
    '2.5.29.37={{text}}1.3.6.1.5.5.7.3.3',
    '2.5.29.19={{text}}'
  )
Export-PfxCertificate -Cert $cert -FilePath $pfx -Password $password | Out-Null
Export-Certificate -Cert $cert -FilePath $cer | Out-Null
"#,
        subject = ps_single_quoted(publisher),
        password = ps_single_quoted(MSIX_DEV_CERT_PASSWORD),
        pfx = ps_single_quoted(&pfx.to_string_lossy()),
        cer = ps_single_quoted(&cer.to_string_lossy()),
    );
    run_powershell(&script).map_err(|err| {
        format!("failed to create MSIX developer certificate ({err}). Enable PowerShell script execution for this user.")
    })?;
    fs::write(&subject_file, publisher)?;
    println!("created {}", pfx.display());
    Ok((pfx, cer))
}

/// Import the public cert into LocalMachine TrustedPeople for sideloading.
///
/// AppX deployment validates package signatures against the machine store, so
/// CurrentUser TrustedPeople is not enough (HRESULT 0x800B0109). Elevates via
/// UAC when the current process cannot write LocalMachine.
#[cfg(windows)]
fn trust_msix_dev_certificate(cer: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let cer = absolute_path(cer)?;
    let cer_lit = ps_single_quoted(&cer.to_string_lossy());
    // Elevated child must be a single -Command string; embed the path literally.
    let elevated = format!(
        "$ErrorActionPreference='Stop'; Import-Certificate -FilePath {cer_lit} -CertStoreLocation Cert:\\LocalMachine\\TrustedPeople | Out-Null"
    );
    let script = format!(
        r#"
$ErrorActionPreference = 'Stop'
$cer = {cer_lit}
$store = 'Cert:\LocalMachine\TrustedPeople'
$leaf = New-Object System.Security.Cryptography.X509Certificates.X509Certificate2 $cer
$existing = Get-ChildItem $store -ErrorAction SilentlyContinue |
  Where-Object {{ $_.Thumbprint -eq $leaf.Thumbprint }}
if ($existing) {{
  Write-Output 'certificate already trusted'
  exit 0
}}
try {{
  Import-Certificate -FilePath $cer -CertStoreLocation $store | Out-Null
}} catch {{
  $p = Start-Process -FilePath powershell.exe -Verb RunAs -Wait -PassThru -ArgumentList @(
    '-NoProfile',
    '-ExecutionPolicy', 'Bypass',
    '-Command',
    {elevated}
  )
  if ($null -eq $p -or $p.ExitCode -ne 0) {{
    throw 'could not import into LocalMachine\TrustedPeople (approve the UAC prompt, or run make msix from an elevated shell)'
  }}
}}
"#,
        cer_lit = cer_lit,
        elevated = ps_single_quoted(&elevated),
    );
    run_powershell(&script).map_err(|err| {
        format!("failed to trust MSIX developer certificate in LocalMachine\\TrustedPeople ({err})")
    })?;
    println!("trusted {}", cer.display());
    Ok(())
}

/// Sign `msix` with SignTool using the local developer PFX.
#[cfg(windows)]
fn sign_msix(msix: &Path, pfx: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let msix = absolute_path(msix)?;
    let pfx = absolute_path(pfx)?;
    let signtool = find_sdk_tool("signtool.exe")?;
    let status = Command::new(&signtool)
        .args([
            "sign",
            "/fd",
            "SHA256",
            "/f",
            &pfx.to_string_lossy(),
            "/p",
            MSIX_DEV_CERT_PASSWORD,
            "/v",
        ])
        .arg(&msix)
        .status()
        .map_err(|err| format!("signtool failed to start ({err})"))?;
    if !status.success() {
        return Err(format!("signtool sign failed with {status}").into());
    }
    println!("signed {}", msix.display());
    Ok(())
}

/// Sideload a signed MSIX with Add-AppxPackage.
///
/// Same-version packages with different content are rejected (0x80073CFB), so
/// remove any existing ShellZip Appx package for this user first.
#[cfg(windows)]
fn add_appx_package(msix: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let msix = absolute_path(msix)?;
    let identity = env_nonempty("STORE_IDENTITY_NAME").unwrap_or_else(|| "ShellZip".into());
    let script = format!(
        r#"
$ErrorActionPreference = 'Stop'
$identity = {identity}
Get-AppxPackage |
  Where-Object {{ $_.Name -eq $identity -or $_.Name -like ($identity + '_*') -or $_.Name -like ('*' + $identity + '*') }} |
  ForEach-Object {{
    Write-Output ('removing ' + $_.PackageFullName)
    Remove-AppxPackage -Package $_.PackageFullName
  }}
Add-AppxPackage -Path {msix} -ForceUpdateFromAnyVersion -ForceApplicationShutdown
"#,
        identity = ps_single_quoted(&identity),
        msix = ps_single_quoted(&msix.to_string_lossy()),
    );
    run_powershell(&script).map_err(|err| {
        format!(
            "Add-AppxPackage failed ({err}). Ensure the developer certificate is in LocalMachine\\TrustedPeople and Developer Mode (or sideloading) is enabled."
        )
    })?;
    Ok(())
}

/// Absolute path without the Win32 `\\?\` prefix (PowerShell rejects it).
#[cfg(windows)]
fn absolute_path(path: &Path) -> Result<PathBuf, Box<dyn std::error::Error>> {
    let absolute =
        fs::canonicalize(path).map_err(|err| format!("canonicalize {}: {err}", path.display()))?;
    let text = absolute.to_string_lossy();
    if let Some(rest) = text.strip_prefix(r"\\?\") {
        Ok(PathBuf::from(rest))
    } else {
        Ok(absolute)
    }
}

/// Quote a string for PowerShell single-quoted literals.
#[cfg(windows)]
fn ps_single_quoted(value: &str) -> String {
    format!("'{}'", value.replace('\'', "''"))
}

/// Run a PowerShell script and surface stdout/stderr on failure.
#[cfg(windows)]
fn run_powershell(script: &str) -> Result<(), Box<dyn std::error::Error>> {
    let output = Command::new("powershell")
        .args([
            "-NoProfile",
            "-NonInteractive",
            "-ExecutionPolicy",
            "Bypass",
            "-Command",
        ])
        .arg(script)
        .output()
        .map_err(|err| format!("powershell failed to start ({err})"))?;
    if output.status.success() {
        return Ok(());
    }
    let stderr = String::from_utf8_lossy(&output.stderr);
    let stdout = String::from_utf8_lossy(&output.stdout);
    let detail = [stderr.trim(), stdout.trim()]
        .into_iter()
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join("\n");
    if detail.is_empty() {
        Err(format!("powershell exited with {}", output.status).into())
    } else {
        Err(detail.into())
    }
}

/// `1.2.3` → `1.2.3.0` for the Appx Identity Version attribute.
fn msix_version(semver: &str) -> Result<String, Box<dyn std::error::Error>> {
    let trimmed = semver.trim().trim_start_matches('v');
    let mut parts = trimmed.split('.');
    let major = parts.next().ok_or("empty version")?;
    let minor = parts.next().unwrap_or("0");
    let patch = parts.next().unwrap_or("0");
    if major.is_empty()
        || !major.chars().all(|c| c.is_ascii_digit())
        || !minor.chars().all(|c| c.is_ascii_digit())
        || !patch.chars().all(|c| c.is_ascii_digit())
    {
        return Err(format!("invalid semver for MSIX: {semver}").into());
    }
    Ok(format!("{major}.{minor}.{patch}.0"))
}

#[cfg(windows)]
fn write_resized_png(src: &Path, dest: &Path, size: u32) -> Result<(), Box<dyn std::error::Error>> {
    let image = image::open(src)?;
    let resized = image.resize_exact(size, size, image::imageops::FilterType::Lanczos3);
    resized.save(dest)?;
    Ok(())
}

fn xml_escape(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

/// Store-package `IExplorerCommand` CLSID (must match `shellzip-shell`).
const EXPLORER_COMMAND_CLSID: &str = "B8D4F6C2-5E3A-4B02-AF7D-9C2E1F3A4B56";

/// Generate `AppxManifest.xml` for the full-trust desktop package.
fn build_appx_manifest(
    identity_name: &str,
    publisher: &str,
    publisher_display: &str,
    version: &str,
) -> String {
    let mut associations = String::new();
    for kind in EXTENSIONS {
        let logo = format!("Assets\\FileType{}.png", kind.extension.replace('.', "_"));
        let display = xml_escape(kind.type_name);
        associations.push_str(&format!(
            r#"
        <uap:Extension Category="windows.fileTypeAssociation">
          <uap:FileTypeAssociation Name="{name}">
            <uap:DisplayName>{display}</uap:DisplayName>
            <uap:Logo>{logo}</uap:Logo>
            <uap:SupportedFileTypes>
              <uap:FileType>.{ext}</uap:FileType>
            </uap:SupportedFileTypes>
            <uap2:SupportedVerbs>
              <uap3:Verb Id="ExtractTo" Parameters="--extract-to &quot;%1&quot;">Extract to…</uap3:Verb>
              <uap3:Verb Id="ExtractHere" Parameters="--extract-here &quot;%1&quot;">Extract Here</uap3:Verb>
              <uap3:Verb Id="ExtractHereNamed" Parameters="--extract-here-named &quot;%1&quot;">Extract Here (name)</uap3:Verb>
            </uap2:SupportedVerbs>
          </uap:FileTypeAssociation>
        </uap:Extension>"#,
            name = kind.extension.replace('.', "_"),
            display = display,
            logo = logo,
            ext = kind.extension,
        ));
    }

    let context_menus = format!(
        r#"
        <com:Extension Category="windows.comServer">
          <com:ComServer>
            <com:SurrogateServer AppId="{clsid}" DisplayName="ShellZip">
              <com:Class Id="{clsid}" Path="shellzip_shell.dll" ThreadingModel="STA" />
            </com:SurrogateServer>
          </com:ComServer>
        </com:Extension>
        <desktop4:Extension Category="windows.fileExplorerContextMenus">
          <desktop4:FileExplorerContextMenus>
            <desktop5:ItemType Type="*">
              <desktop5:Verb Id="0ShellZipFile" Clsid="{clsid}" />
            </desktop5:ItemType>
            <desktop5:ItemType Type="Directory">
              <desktop5:Verb Id="0ShellZipDir" Clsid="{clsid}" />
            </desktop5:ItemType>
            <desktop5:ItemType Type="Directory\Background">
              <desktop5:Verb Id="0ShellZipBackground" Clsid="{clsid}" />
            </desktop5:ItemType>
          </desktop4:FileExplorerContextMenus>
        </desktop4:Extension>"#,
        clsid = EXPLORER_COMMAND_CLSID,
    );

    format!(
        r#"<?xml version="1.0" encoding="utf-8"?>
<Package
  xmlns="http://schemas.microsoft.com/appx/manifest/foundation/windows10"
  xmlns:uap="http://schemas.microsoft.com/appx/manifest/uap/windows10"
  xmlns:uap2="http://schemas.microsoft.com/appx/manifest/uap/windows10/2"
  xmlns:uap3="http://schemas.microsoft.com/appx/manifest/uap/windows10/3"
  xmlns:uap5="http://schemas.microsoft.com/appx/manifest/uap/windows10/5"
  xmlns:com="http://schemas.microsoft.com/appx/manifest/com/windows10"
  xmlns:desktop4="http://schemas.microsoft.com/appx/manifest/desktop/windows10/4"
  xmlns:desktop5="http://schemas.microsoft.com/appx/manifest/desktop/windows10/5"
  xmlns:rescap="http://schemas.microsoft.com/appx/manifest/foundation/windows10/restrictedcapabilities"
  IgnorableNamespaces="uap uap2 uap3 uap5 com desktop4 desktop5 rescap">
  <Identity
    Name="{identity}"
    Publisher="{publisher}"
    Version="{version}"
    ProcessorArchitecture="x64" />
  <Properties>
    <DisplayName>ShellZip</DisplayName>
    <PublisherDisplayName>{publisher_display}</PublisherDisplayName>
    <Logo>Assets\StoreLogo.png</Logo>
    <Description>Archive manager for ZIP, 7z, RAR, TAR, and related formats.</Description>
  </Properties>
  <Dependencies>
    <TargetDeviceFamily Name="Windows.Desktop" MinVersion="10.0.17763.0" MaxVersionTested="10.0.26100.0" />
  </Dependencies>
  <Resources>
    <Resource Language="en-us" />
  </Resources>
  <Applications>
    <Application Id="ShellZip" Executable="shellzip.exe" EntryPoint="Windows.FullTrustApplication">
      <uap:VisualElements
        DisplayName="ShellZip"
        Description="ShellZip archive manager"
        BackgroundColor="transparent"
        Square150x150Logo="Assets\Square150x150Logo.png"
        Square44x44Logo="Assets\Square44x44Logo.png" />
      <Extensions>
        <uap5:Extension Category="windows.appExecutionAlias">
          <uap5:AppExecutionAlias>
            <uap5:ExecutionAlias Alias="shellzip.exe" />
          </uap5:AppExecutionAlias>
        </uap5:Extension>{associations}{context_menus}
      </Extensions>
    </Application>
    <Application Id="Szip" Executable="szip.exe" EntryPoint="Windows.FullTrustApplication">
      <uap:VisualElements
        DisplayName="szip"
        Description="ShellZip command-line tool"
        BackgroundColor="transparent"
        Square150x150Logo="Assets\Square150x150Logo.png"
        Square44x44Logo="Assets\Square44x44Logo.png" />
      <Extensions>
        <uap5:Extension Category="windows.appExecutionAlias">
          <uap5:AppExecutionAlias>
            <uap5:ExecutionAlias Alias="szip.exe" />
          </uap5:AppExecutionAlias>
        </uap5:Extension>
      </Extensions>
    </Application>
  </Applications>
  <Capabilities>
    <rescap:Capability Name="runFullTrust" />
  </Capabilities>
</Package>
"#,
        identity = xml_escape(identity_name),
        publisher = xml_escape(publisher),
        publisher_display = xml_escape(publisher_display),
        version = version,
        associations = associations,
        context_menus = context_menus,
    )
}

/// Locate `makeappx.exe` / `signtool.exe` on PATH or under the Windows Kits.
#[cfg(windows)]
fn find_sdk_tool(name: &str) -> Result<PathBuf, Box<dyn std::error::Error>> {
    if let Ok(path) = which_on_path(name) {
        return Ok(path);
    }
    let kits = PathBuf::from(r"C:\Program Files (x86)\Windows Kits\10\bin");
    if kits.is_dir() {
        let mut versions: Vec<_> = fs::read_dir(&kits)?
            .filter_map(|entry| entry.ok())
            .filter(|entry| entry.path().is_dir())
            .map(|entry| entry.path())
            .collect();
        versions.sort();
        versions.reverse();
        for version in versions {
            let candidate = version.join("x64").join(name);
            if candidate.is_file() {
                return Ok(candidate);
            }
        }
    }
    Err(format!("{name} not found; install the Windows 10 SDK").into())
}

#[cfg(windows)]
fn which_on_path(name: &str) -> Result<PathBuf, Box<dyn std::error::Error>> {
    let path = std::env::var_os("PATH").ok_or("PATH is not set")?;
    for dir in std::env::split_paths(&path) {
        let candidate = dir.join(name);
        if candidate.is_file() {
            return Ok(candidate);
        }
    }
    Err(format!("{name} not on PATH").into())
}

/// Detach-sign release artifacts in a directory (`SHA256SUMS` + `.asc`).
fn gpg_sign_release() -> Result<(), Box<dyn std::error::Error>> {
    let dir = std::env::args()
        .nth(2)
        .map(PathBuf::from)
        .ok_or("usage: xtask gpg-sign-release <dir>")?;
    gpg_sign_dir(&dir)
}

fn gpg_sign_dir(dir: &Path) -> Result<(), Box<dyn std::error::Error>> {
    if env_nonempty("GPG_PRIVATE_KEY").is_none() && env_nonempty("GPG_KEY_ID").is_none() {
        eprintln!(
            "xtask: warning: skipping GPG signing (set GPG_PRIVATE_KEY / GPG_KEY_ID to enable)"
        );
        return Ok(());
    }
    if !dir.is_dir() {
        return Err(format!("missing {}", dir.display()).into());
    }

    let mut files = Vec::new();
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        let name = entry.file_name().to_string_lossy().to_string();
        if name == "SHA256SUMS" || name.ends_with(".asc") {
            continue;
        }
        files.push(path);
    }
    files.sort();
    if files.is_empty() {
        return Err(format!("no files to sign in {}", dir.display()).into());
    }

    let sums_path = dir.join("SHA256SUMS");
    let mut sums = String::new();
    for path in &files {
        let bytes = fs::read(path)?;
        let digest = sha256_hex(&bytes);
        let name = path
            .file_name()
            .and_then(|n| n.to_str())
            .ok_or("non-utf8 file name")?;
        sums.push_str(&format!("{digest}  {name}\n"));
    }
    fs::write(&sums_path, &sums)?;

    let key_id = std::env::var("GPG_KEY_ID").unwrap_or_default();
    sign_with_gpg(&sums_path, &key_id)?;
    for path in &files {
        if path
            .extension()
            .is_some_and(|ext| ext == "AppImage" || ext == "nupkg" || ext == "zip")
            || path
                .file_name()
                .is_some_and(|n| n.to_string_lossy().contains("AppImage"))
        {
            sign_with_gpg(path, &key_id)?;
        }
    }
    println!("gpg-signed {}", dir.display());
    Ok(())
}

fn sha256_hex(bytes: &[u8]) -> String {
    use sha2::{Digest, Sha256};
    let digest = Sha256::digest(bytes);
    digest.iter().map(|byte| format!("{byte:02x}")).collect()
}

fn sign_with_gpg(path: &Path, key_id: &str) -> Result<(), Box<dyn std::error::Error>> {
    let out = PathBuf::from(format!("{}.asc", path.display()));
    let mut cmd = Command::new("gpg");
    cmd.args([
        "--batch",
        "--yes",
        "--pinentry-mode",
        "loopback",
        "--detach-sign",
        "--armor",
        "-o",
    ]);
    cmd.arg(&out);
    if !key_id.is_empty() {
        cmd.args(["--local-user", key_id]);
    }
    if let Ok(pass) = std::env::var("GPG_PASSPHRASE") {
        cmd.args(["--passphrase", &pass]);
    }
    cmd.arg(path);
    let status = cmd
        .status()
        .map_err(|err| format!("gpg not found ({err})"))?;
    if !status.success() {
        return Err(format!("gpg --detach-sign failed for {}", path.display()).into());
    }
    Ok(())
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

/// Drawing stems under `grx/shellzip_file_icons/svg/`.
const FILE_ICON_NAMES: &[&str] = &[
    "zip", "7z", "rar", "tar", "gz", "bz2", "xz", "zst", "tar.gz", "tar.bz2", "tar.xz", "tar.zst",
];

/// Shell-standard ICO sizes. Explorer Medium/Large/Extra Large need these;
/// Vista+ wants BMP/DIB below 256 and PNG only for the 256 entry.
const ICO_SIZES: &[u32] = &[16, 24, 32, 48, 64, 256];

/// Freedesktop hicolor raster sizes used by GNOME/KDE/Nautilus zoom levels.
const LINUX_SIZES: &[u32] = &[16, 22, 24, 32, 48, 64, 128, 256, 512];

/// All PNG edges we rasterize once and reuse for ICO / ICNS / hicolor.
const RASTER_SIZES: &[u32] = &[16, 20, 22, 24, 32, 40, 48, 64, 96, 128, 256, 512, 1024];

/// ICNS OSType → pixel edge. Includes 1x and @2x slots so Finder scales cleanly.
const ICNS_SLOTS: &[([u8; 4], u32)] = &[
    (*b"icp4", 16),
    (*b"icp5", 32),
    (*b"icp6", 64),
    (*b"ic07", 128),
    (*b"ic08", 256),
    (*b"ic09", 512),
    (*b"ic10", 1024),
    (*b"ic11", 32),  // 16@2x
    (*b"ic12", 64),  // 32@2x
    (*b"ic13", 256), // 128@2x
    (*b"ic14", 512), // 256@2x
];

/// Drawing stem → freedesktop MIME icon basename (no extension).
fn linux_mime_basenames(stem: &str) -> &'static [&'static str] {
    match stem {
        "zip" => &["application-zip"],
        "7z" => &["application-x-7z-compressed"],
        "rar" => &["application-vnd.rar", "application-x-rar-compressed"],
        "tar" => &["application-x-tar"],
        "gz" => &["application-gzip"],
        "bz2" => &["application-x-bzip2"],
        "xz" => &["application-x-xz"],
        "zst" => &["application-zstd"],
        "tar.gz" => &["application-x-compressed-tar"],
        "tar.bz2" => &["application-x-bzip-compressed-tar"],
        "tar.xz" => &["application-x-xz-compressed-tar"],
        "tar.zst" => &["application-x-zstd-compressed-tar"],
        _ => &[],
    }
}

/// Rasterize `grx/shellzip_file_icons/svg` into multi-size platform icon packs.
///
/// - Windows `.ico`: BMP/DIB for ≤64, PNG for 256 (Explorer Medium/Large/XL).
/// - macOS `.icns`: classic + retina slots through 1024.
/// - Linux `hicolor`: PNG trees for every common zoom size.
fn gen_file_icons() -> Result<(), Box<dyn std::error::Error>> {
    let root = PathBuf::from("grx").join("shellzip_file_icons");
    let svg_dir = root.join("svg");
    let png_dir = root.join("png");
    let ico_dir = root.join("ico");
    let icns_dir = root.join("icns");
    let macos_dir = root.join("macos");
    let hicolor = root.join("linux").join("hicolor");
    fs::create_dir_all(&png_dir)?;
    fs::create_dir_all(&ico_dir)?;
    fs::create_dir_all(&icns_dir)?;
    fs::create_dir_all(&macos_dir)?;

    for name in FILE_ICON_NAMES {
        let mut rasters: std::collections::BTreeMap<u32, Vec<u8>> =
            std::collections::BTreeMap::new();
        for &size in RASTER_SIZES {
            let svg_name = if size <= 24 {
                format!("{name}_small.svg")
            } else {
                format!("{name}.svg")
            };
            let png_bytes = raster_svg(&svg_dir.join(&svg_name), size)?;
            fs::write(png_dir.join(format!("{name}_{size}.png")), &png_bytes)?;
            rasters.insert(size, png_bytes);
        }

        let mut ico_dir_entry = ico::IconDir::new(ico::ResourceType::Icon);
        for &size in ICO_SIZES {
            let png_bytes = rasters.get(&size).ok_or("missing ico raster")?;
            let image = image::load_from_memory(png_bytes)?.into_rgba8();
            let icon_image = ico::IconImage::from_rgba_data(size, size, image.into_raw());
            // Soft SVG shadows make `encode()` pick PNG for every size; Explorer's
            // Medium/Large/Extra Large paths expect classic DIB below 256.
            let entry = if size >= 256 {
                ico::IconDirEntry::encode_as_png(&icon_image)?
            } else {
                ico::IconDirEntry::encode_as_bmp(&icon_image)?
            };
            ico_dir_entry.add_entry(entry);
        }
        let ico_path = ico_dir.join(format!("{name}.ico"));
        let mut ico_file = fs::File::create(&ico_path)?;
        ico_dir_entry.write(&mut ico_file)?;

        let mut icns_entries: Vec<([u8; 4], Vec<u8>)> = Vec::new();
        for &(ostype, size) in ICNS_SLOTS {
            let png_bytes = rasters.get(&size).ok_or("missing icns raster")?.clone();
            icns_entries.push((ostype, png_bytes));
        }
        write_icns(&icns_dir.join(format!("{name}.icns")), &icns_entries)?;
        write_icns(&macos_dir.join(format!("{name}.icns")), &icns_entries)?;

        for &size in LINUX_SIZES {
            let png_bytes = rasters.get(&size).ok_or("missing linux raster")?;
            let dir = hicolor.join(format!("{size}x{size}")).join("mimetypes");
            fs::create_dir_all(&dir)?;
            for mime in linux_mime_basenames(name) {
                fs::write(dir.join(format!("{mime}.png")), png_bytes)?;
            }
        }
        println!("wrote {name}.ico / {name}.icns + hicolor");
    }
    Ok(())
}

fn raster_svg(path: &Path, size: u32) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let data = fs::read(path)?;
    let mut options = usvg::Options::default();
    options.fontdb_mut().load_system_fonts();
    let tree = usvg::Tree::from_data(&data, &options)?;
    let mut pixmap = tiny_skia::Pixmap::new(size, size).ok_or("pixmap")?;
    let scale = size as f32 / tree.size().width().max(tree.size().height());
    let transform = tiny_skia::Transform::from_scale(scale, scale);
    resvg::render(&tree, transform, &mut pixmap.as_mut());
    Ok(pixmap.encode_png()?)
}

fn write_icns(
    path: &Path,
    entries: &[([u8; 4], Vec<u8>)],
) -> Result<(), Box<dyn std::error::Error>> {
    let mut body = Vec::new();
    for (ostype, png) in entries {
        body.extend_from_slice(ostype);
        let len = u32::try_from(png.len() + 8)?;
        body.extend_from_slice(&len.to_be_bytes());
        body.extend_from_slice(png);
    }
    let mut file = Vec::new();
    file.extend_from_slice(b"icns");
    let total = u32::try_from(body.len() + 8)?;
    file.extend_from_slice(&total.to_be_bytes());
    file.extend_from_slice(&body);
    fs::write(path, file)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{build_appx_manifest, msix_version, version_from_workspace_toml};

    #[test]
    fn reads_workspace_package_version() {
        let text = r#"
[workspace]
members = ["crates/foo"]

[workspace.package]
edition = "2024"
version = "1.2.3"
license = "MIT"
"#;
        assert_eq!(version_from_workspace_toml(text).as_deref(), Some("1.2.3"));
    }

    #[test]
    fn rejects_version_prefixed_keys() {
        let text = r#"
[workspace.package]
version_override = "9.9.9"
version_foo = "8.8.8"
edition = "2024"
"#;
        assert_eq!(version_from_workspace_toml(text), None);
    }

    #[test]
    fn accepts_version_with_spaces_around_equals() {
        let text = "[workspace.package]\nversion  =  \"0.1.0\"\n";
        assert_eq!(version_from_workspace_toml(text).as_deref(), Some("0.1.0"));
    }

    #[test]
    fn stops_at_next_table() {
        let text = r#"
[workspace.package]
edition = "2024"

[workspace.dependencies]
version = "should-not-win"
"#;
        assert_eq!(version_from_workspace_toml(text), None);
    }

    #[test]
    fn msix_version_appends_revision() {
        assert_eq!(msix_version("1.2.3").unwrap(), "1.2.3.0");
        assert_eq!(msix_version("v0.1.9").unwrap(), "0.1.9.0");
    }

    #[cfg(windows)]
    #[test]
    fn powershell_single_quotes_escape_apostrophes() {
        assert_eq!(super::ps_single_quoted("CN=ShellZip"), "'CN=ShellZip'");
        assert_eq!(super::ps_single_quoted("O'Reilly"), "'O''Reilly'");
    }

    #[test]
    fn appx_manifest_lists_every_extension_and_verbs() {
        let xml = build_appx_manifest("ShellZip", "CN=ShellZip", "ShellZip", "0.1.9.0");
        for kind in shellzip_integrate::EXTENSIONS {
            assert!(
                xml.contains(&format!(".{ext}</uap:FileType>", ext = kind.extension)),
                "missing FileType for {}",
                kind.extension
            );
        }
        assert!(xml.contains("--extract-to"));
        assert!(xml.contains("--extract-here"));
        assert!(xml.contains("--extract-here-named"));
        assert!(xml.contains("runFullTrust"));
        assert!(xml.contains("shellzip_shell.dll"));
        assert!(xml.contains("com:SurrogateServer"));
        assert!(xml.contains(super::EXPLORER_COMMAND_CLSID));
        assert!(xml.contains(r#"Type="*""#));
        assert!(xml.contains(r#"Type="Directory""#));
        assert!(xml.contains(r#"Type="Directory\Background""#));
        assert!(xml.contains(r#"Id="0ShellZipFile""#));
        assert!(xml.contains(r#"Id="0ShellZipDir""#));
        assert!(xml.contains(r#"Id="0ShellZipBackground""#));
        assert!(xml.contains(&format!(
            r#"AppId="{clsid}""#,
            clsid = super::EXPLORER_COMMAND_CLSID
        )));
        assert!(
            xml.contains("IgnorableNamespaces=\"uap uap2 uap3 uap5 com desktop4 desktop5 rescap\"")
        );
        assert!(!xml.contains("A7C3E5B1-4D2F-4A91-9E6C-8B1F0D2E3A45"));
    }
}
