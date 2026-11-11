//! Text writers for desktop entries, service menus, and registry plans.

use crate::formats::{EXTENSIONS, MIME_TYPES};
use crate::menu::{MenuAction, action_flag, action_label, linux_named_label};

/// Application icon name installed under `hicolor/*/apps/`.
pub const APP_ICON_NAME: &str = "shellzip";

const NAUTILUS_PYTHON_TEMPLATE: &str = include_str!("../../../platform/linux/nautilus/shellzip.py");

/// Freedesktop `.desktop` file body. `exec_path` is the `shellzip` binary.
pub fn desktop_entry(exec_path: &str) -> String {
    let mimes = MIME_TYPES.join(";");
    let exe = quote_desktop_exec(exec_path);
    let named = linux_named_label();
    format!(
        "\
[Desktop Entry]
Type=Application
Name=ShellZip
Comment=Open, test, extract, and create archives
Exec={exe} %F
Icon={icon}
Terminal=false
Categories=Utility;Archiving;
MimeType={mimes};
StartupNotify=true
StartupWMClass=shellzip
Actions=ExtractTo;ExtractHere;ExtractHereNamed;

[Desktop Action ExtractTo]
Name=Extract to…
Exec={exe} --extract-to %F

[Desktop Action ExtractHere]
Name=Extract Here
Exec={exe} --extract-here %F

[Desktop Action ExtractHereNamed]
Name={named}
Exec={exe} --extract-here-named %F
",
        icon = APP_ICON_NAME,
    )
}

/// KDE Dolphin extract/open service menu (archive MIME types only).
pub fn kde_servicemenu(exec_path: &str) -> String {
    kde_service(
        exec_path,
        MIME_TYPES.join(";"),
        &[
            MenuAction::Open,
            MenuAction::ExtractTo,
            MenuAction::ExtractHere,
            MenuAction::ExtractHereNamed,
        ],
    )
}

/// KDE Dolphin add service menu (every file and directory).
pub fn kde_servicemenu_add(exec_path: &str) -> String {
    kde_service(
        exec_path,
        "all/all;inode/directory".into(),
        &[MenuAction::Add, MenuAction::AddZip, MenuAction::Add7z],
    )
}

/// KDE service-menu files as `(file_name, contents)` pairs.
pub fn kde_servicemenus(exec_path: &str) -> Vec<(String, String)> {
    vec![
        (
            "shellzip-extract.desktop".to_string(),
            kde_servicemenu(exec_path),
        ),
        (
            "shellzip-add.desktop".to_string(),
            kde_servicemenu_add(exec_path),
        ),
    ]
}

fn kde_service(exec_path: &str, mimes: String, actions: &[MenuAction]) -> String {
    let exe = quote_desktop_exec(exec_path);
    let action_ids = actions
        .iter()
        .map(|action| kde_action_id(*action))
        .collect::<Vec<_>>()
        .join(";");
    let mut body = format!(
        "\
[Desktop Entry]
Type=Service
ServiceTypes=KonqPopupMenu/Plugin
MimeType={mimes};
Actions={action_ids};
X-KDE-Priority=TopLevel
X-KDE-Submenu=ShellZip
Icon={icon}
",
        icon = APP_ICON_NAME,
    );
    for action in actions {
        let (name, icon, flag) = kde_action_fields(*action);
        let exec = if flag.is_empty() {
            format!("{exe} %F")
        } else {
            format!("{exe} {flag} %F")
        };
        body.push_str(&format!(
            "
[Desktop Action {id}]
Name={name}
Icon={icon}
Exec={exec}
",
            id = kde_action_id(*action),
        ));
    }
    body
}

fn kde_action_id(action: MenuAction) -> &'static str {
    match action {
        MenuAction::Add => "Add",
        MenuAction::AddZip => "AddZip",
        MenuAction::Add7z => "Add7z",
        MenuAction::Open => "Open",
        MenuAction::ExtractTo => "ExtractTo",
        MenuAction::ExtractHere => "ExtractHere",
        MenuAction::ExtractHereNamed => "ExtractHereNamed",
    }
}

fn kde_action_fields(action: MenuAction) -> (&'static str, &'static str, &'static str) {
    let icon = match action {
        MenuAction::Add | MenuAction::AddZip | MenuAction::Add7z => APP_ICON_NAME,
        MenuAction::Open => APP_ICON_NAME,
        MenuAction::ExtractTo | MenuAction::ExtractHere => "archive-extract",
        MenuAction::ExtractHereNamed => "folder-new",
    };
    (linux_action_name(action), icon, action_flag(action))
}

fn linux_action_name(action: MenuAction) -> &'static str {
    match action {
        MenuAction::ExtractHereNamed => linux_named_label(),
        other => action_label(other),
    }
}

/// One Nemo `.nemo_action` file body for `action`.
pub fn nemo_action(exec_path: &str, action: MenuAction, name: &str) -> String {
    let archives_only = matches!(
        action,
        MenuAction::Open
            | MenuAction::ExtractTo
            | MenuAction::ExtractHere
            | MenuAction::ExtractHereNamed
    );
    nemo_action_body(exec_path, name, action_flag(action), archives_only)
}

fn nemo_action_body(exec_path: &str, name: &str, flag: &str, archives_only: bool) -> String {
    let exe = quote_desktop_exec(exec_path);
    let command = if flag.is_empty() {
        format!("{exe} %F")
    } else {
        format!("{exe} {flag} %F")
    };
    let mime_line = if archives_only {
        format!("Mimetypes={};\n", MIME_TYPES.join(";"))
    } else {
        String::new()
    };
    format!(
        "\
[Nemo Action]
Active=true
Name={name}
Comment=ShellZip: {name}
Exec={command}
Selection=any
Extensions=any;dir;
Icon-Name={icon}
{mime_line}",
        icon = APP_ICON_NAME,
    )
}

/// All Nemo action files as `(file_name, contents)` pairs.
pub fn nemo_actions(exec_path: &str) -> Vec<(String, String)> {
    vec![
        nemo_file(exec_path, "shellzip-add.nemo_action", MenuAction::Add),
        nemo_file(
            exec_path,
            "shellzip-add-zip.nemo_action",
            MenuAction::AddZip,
        ),
        nemo_file(exec_path, "shellzip-add-7z.nemo_action", MenuAction::Add7z),
        nemo_file(exec_path, "shellzip-open.nemo_action", MenuAction::Open),
        nemo_file(
            exec_path,
            "shellzip-extract-to.nemo_action",
            MenuAction::ExtractTo,
        ),
        nemo_file(
            exec_path,
            "shellzip-extract-here.nemo_action",
            MenuAction::ExtractHere,
        ),
        nemo_file(
            exec_path,
            "shellzip-extract-here-named.nemo_action",
            MenuAction::ExtractHereNamed,
        ),
    ]
}

fn nemo_file(exec_path: &str, file_name: &str, action: MenuAction) -> (String, String) {
    (
        file_name.to_string(),
        nemo_action(exec_path, action, linux_action_name(action)),
    )
}

/// PCManFM / libfm action files as `(file_name, contents)` pairs.
pub fn pcmanfm_actions(exec_path: &str) -> Vec<(String, String)> {
    vec![
        pcmanfm_file(exec_path, "shellzip-add.desktop", MenuAction::Add, false),
        pcmanfm_file(
            exec_path,
            "shellzip-add-zip.desktop",
            MenuAction::AddZip,
            false,
        ),
        pcmanfm_file(
            exec_path,
            "shellzip-add-7z.desktop",
            MenuAction::Add7z,
            false,
        ),
        pcmanfm_file(exec_path, "shellzip-open.desktop", MenuAction::Open, true),
        pcmanfm_file(
            exec_path,
            "shellzip-extract-to.desktop",
            MenuAction::ExtractTo,
            true,
        ),
        pcmanfm_file(
            exec_path,
            "shellzip-extract-here.desktop",
            MenuAction::ExtractHere,
            true,
        ),
        pcmanfm_file(
            exec_path,
            "shellzip-extract-here-named.desktop",
            MenuAction::ExtractHereNamed,
            true,
        ),
    ]
}

fn pcmanfm_file(
    exec_path: &str,
    file_name: &str,
    action: MenuAction,
    archives_only: bool,
) -> (String, String) {
    let exe = quote_desktop_exec(exec_path);
    let flag = action_flag(action);
    let name = linux_action_name(action);
    let exec = if flag.is_empty() {
        format!("{exe} %F")
    } else {
        format!("{exe} {flag} %F")
    };
    let mimes = if archives_only {
        format!("{};", MIME_TYPES.join(";"))
    } else {
        "all/all;inode/directory;".into()
    };
    let body = format!(
        "\
[Desktop Entry]
Type=Action
Name={name}
Icon={icon}
Profiles=profile-zero;

[X-Action-Profile profile-zero]
Exec={exec}
Name={name}
MimeTypes={mimes}
",
        icon = APP_ICON_NAME,
    );
    (file_name.to_string(), body)
}

/// Nautilus Python extension with `exec_path` baked in.
pub fn nautilus_python_extension(exec_path: &str) -> String {
    NAUTILUS_PYTHON_TEMPLATE.replace(
        "SHELLZIP = \"shellzip\"",
        &format!("SHELLZIP = {}", python_string(exec_path)),
    )
}

/// Nautilus / Caja script files as `(file_name, contents)` pairs.
pub fn nautilus_scripts(exec_path: &str) -> Vec<(String, String)> {
    vec![
        script_file(exec_path, "Add to archive...", MenuAction::Add),
        script_file(exec_path, "Add to ZIP", MenuAction::AddZip),
        script_file(exec_path, "Add to 7z", MenuAction::Add7z),
        script_file(exec_path, "Open", MenuAction::Open),
        script_file(exec_path, "Extract to...", MenuAction::ExtractTo),
        script_file(exec_path, "Extract Here", MenuAction::ExtractHere),
        script_file(
            exec_path,
            "Extract Here (folder)",
            MenuAction::ExtractHereNamed,
        ),
    ]
}

fn script_file(exec_path: &str, file_name: &str, action: MenuAction) -> (String, String) {
    let exe = posix_single_quote(exec_path);
    let flag = action_flag(action);
    let body = if flag.is_empty() {
        format!("#!/bin/sh\nexec {exe} \"$@\"\n")
    } else {
        format!("#!/bin/sh\nexec {exe} {flag} \"$@\"\n")
    };
    (file_name.to_string(), body)
}

/// Thunar custom-actions XML snippet (not merged into `uca.xml`).
pub fn thunar_uca(exec_path: &str) -> String {
    format!(
        "\
<?xml version=\"1.0\" encoding=\"UTF-8\"?>
<actions>
{}
</actions>
",
        thunar_action_xml(exec_path)
    )
}

/// Merge ShellZip actions into an existing Thunar `uca.xml` body.
///
/// Replaces any previous ShellZip actions (matched by `unique-id` or name).
pub fn merge_thunar_uca(existing: &str, exec_path: &str) -> String {
    let mut kept = Vec::new();
    for block in thunar_action_blocks(existing) {
        if !is_shellzip_thunar_action(&block) {
            kept.push(block);
        }
    }
    let mut inner = String::new();
    for block in &kept {
        inner.push_str(block);
        inner.push('\n');
    }
    inner.push_str(&thunar_action_xml(exec_path));
    format!("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<actions>\n{inner}</actions>\n")
}

/// Drop ShellZip actions from a Thunar `uca.xml` body.
pub fn strip_thunar_shellzip_actions(existing: &str) -> String {
    let mut inner = String::new();
    for block in thunar_action_blocks(existing) {
        if !is_shellzip_thunar_action(&block) {
            inner.push_str(&block);
            inner.push('\n');
        }
    }
    format!("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<actions>\n{inner}</actions>\n")
}

fn thunar_action_xml(exec_path: &str) -> String {
    let patterns = archive_glob_patterns();
    let mut out = String::new();
    out.push_str(&thunar_action(
        exec_path,
        "shellzip-add",
        "ShellZip: Add to archive...",
        APP_ICON_NAME,
        "--add",
        "*",
        true,
    ));
    out.push_str(&thunar_action(
        exec_path,
        "shellzip-add-zip",
        "ShellZip: Add to ZIP",
        APP_ICON_NAME,
        "--add-zip",
        "*",
        true,
    ));
    out.push_str(&thunar_action(
        exec_path,
        "shellzip-add-7z",
        "ShellZip: Add to 7z",
        APP_ICON_NAME,
        "--add-7z",
        "*",
        true,
    ));
    out.push_str(&thunar_action(
        exec_path,
        "shellzip-open",
        "ShellZip: Open",
        APP_ICON_NAME,
        "",
        &patterns,
        false,
    ));
    out.push_str(&thunar_action(
        exec_path,
        "shellzip-extract-to",
        "ShellZip: Extract to…",
        "archive-extract",
        "--extract-to",
        &patterns,
        false,
    ));
    out.push_str(&thunar_action(
        exec_path,
        "shellzip-extract-here",
        "ShellZip: Extract Here",
        "archive-extract",
        "--extract-here",
        &patterns,
        false,
    ));
    out.push_str(&thunar_action(
        exec_path,
        "shellzip-extract-here-named",
        &format!("ShellZip: {}", linux_named_label()),
        "folder-new",
        "--extract-here-named",
        &patterns,
        false,
    ));
    out
}

fn thunar_action(
    exec_path: &str,
    unique_id: &str,
    name: &str,
    icon: &str,
    flag: &str,
    patterns: &str,
    add_selection: bool,
) -> String {
    let exe = posix_single_quote(exec_path);
    let command = if flag.is_empty() {
        format!("{exe} %F")
    } else {
        format!("{exe} {flag} %F")
    };
    let types = if add_selection {
        "    <directories/>\n    <audio-files/>\n    <image-files/>\n    <other-files/>\n    <text-files/>\n    <video-files/>\n"
    } else {
        "    <other-files/>\n"
    };
    format!(
        "  <action>\n    <icon>{}</icon>\n    <name>{}</name>\n    <unique-id>{}</unique-id>\n    <command>{}</command>\n    <description>{}</description>\n    <patterns>{}</patterns>\n{}  </action>\n",
        xml_escape(icon),
        xml_escape(name),
        xml_escape(unique_id),
        xml_escape(&command),
        xml_escape(name),
        xml_escape(patterns),
        types,
    )
}

fn thunar_action_blocks(xml: &str) -> Vec<String> {
    let mut blocks = Vec::new();
    let mut rest = xml;
    while let Some(start) = rest.find("<action>") {
        let Some(end_rel) = rest[start..].find("</action>") else {
            break;
        };
        let end = start + end_rel + "</action>".len();
        blocks.push(rest[start..end].to_string());
        rest = &rest[end..];
    }
    blocks
}

fn is_shellzip_thunar_action(block: &str) -> bool {
    block.contains("<unique-id>shellzip-") || block.contains("<name>ShellZip:")
}

fn archive_glob_patterns() -> String {
    EXTENSIONS
        .iter()
        .map(|kind| format!("*.{}", kind.extension))
        .collect::<Vec<_>>()
        .join(";")
}

/// Quote a program path for a freedesktop `Exec=` key.
fn quote_desktop_exec(exe: &str) -> String {
    if exe
        .chars()
        .any(|ch| ch.is_whitespace() || matches!(ch, '"' | '$' | '`' | '\\'))
    {
        let escaped = exe
            .replace('\\', "\\\\")
            .replace('"', "\\\"")
            .replace('$', "\\$")
            .replace('`', "\\`");
        format!("\"{escaped}\"")
    } else {
        exe.to_string()
    }
}

fn posix_single_quote(value: &str) -> String {
    format!("'{}'", value.replace('\'', "'\\''"))
}

fn python_string(value: &str) -> String {
    let mut out = String::from("\"");
    for ch in value.chars() {
        match ch {
            '\\' => out.push_str("\\\\"),
            '"' => out.push_str("\\\""),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            ch if ch.is_control() => {
                let code = u32::from(ch);
                if let Ok(byte) = u8::try_from(code) {
                    out.push_str(&format!("\\x{byte:02x}"));
                }
            }
            ch => out.push(ch),
        }
    }
    out.push('"');
    out
}

fn xml_escape(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

/// `CFBundleDocumentTypes` fragment for `Info.plist` (XML array items).
pub fn macos_document_types() -> String {
    let mut out = String::from("<!-- ShellZip document types -->\n");
    for kind in EXTENSIONS {
        let icon = kind.icon.replace('.', "_");
        out.push_str(&format!(
            "\
<dict>
  <key>CFBundleTypeName</key>
  <string>{name}</string>
  <key>CFBundleTypeRole</key>
  <string>Viewer</string>
  <key>CFBundleTypeIconFile</key>
  <string>{icon}</string>
  <key>LSItemContentTypes</key>
  <array>
    <string>com.shellzip.{ext}</string>
  </array>
  <key>CFBundleTypeExtensions</key>
  <array>
    <string>{ext}</string>
  </array>
</dict>
",
            name = kind.type_name,
            ext = kind.extension,
        ));
    }
    out
}

/// Dry-run description of Windows registry keys (no live hive writes).
pub fn windows_registry_plan(exe: &str, icons_dir: &str) -> String {
    let mut out = String::from("# ShellZip Windows association plan (HKCU)\n");
    for kind in EXTENSIONS {
        let icon_file = format!("{icons_dir}\\{}.ico,0", kind.icon);
        out.push_str(&format!(
            "\
ProgID {prog}
  (default) = {name}
  DefaultIcon = {icon}
  shell\\open\\command = \"{exe}\" \"%1\"
OpenWithProgids\\{ext} += {prog}
",
            prog = kind.prog_id,
            name = kind.type_name,
            icon = icon_file,
            ext = kind.extension,
            exe = exe,
        ));
    }
    out.push_str(&format!(
        "\
*\\shellex\\ContextMenuHandlers\\ShellZip = {{CLSID}}
Directory\\shellex\\ContextMenuHandlers\\ShellZip = {{CLSID}}
Software\\ShellZip\\ExePath = {exe}
Approved shell extension list += ShellZip
"
    ));
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn desktop_lists_mimes_and_actions() {
        let text = desktop_entry("/usr/bin/shellzip");
        assert!(text.contains("application/zip"));
        assert!(text.contains("application/x-7z-compressed"));
        assert!(text.contains("--extract-here-named"));
        assert!(text.contains("Extract Here (folder)"));
        assert!(text.contains("StartupWMClass=shellzip"));
        assert!(text.contains("Icon=shellzip"));
    }

    #[test]
    fn desktop_quotes_exec_with_spaces() {
        let text = desktop_entry("/opt/My Apps/shellzip");
        assert!(text.contains("\"/opt/My Apps/shellzip\" %F"));
    }

    #[test]
    fn nautilus_extension_bakes_absolute_exe() {
        let text = nautilus_python_extension("/opt/ShellZip/shellzip");
        assert!(text.contains("SHELLZIP = \"/opt/ShellZip/shellzip\""));
        assert!(!text.contains("SHELLZIP = \"shellzip\""));
        assert!(text.contains("gi.require_version(\"Nautilus\", \"4.0\")"));
        assert!(text.contains("--add"));
        assert!(text.contains("icon=\"shellzip\"") || text.contains("icon='shellzip'"));
    }

    #[test]
    fn kde_menus_split_add_and_extract() {
        let extract = kde_servicemenu("/usr/bin/shellzip");
        assert!(extract.contains("MimeType=application/zip;"));
        assert!(extract.contains("Actions=Open;ExtractTo;ExtractHere;ExtractHereNamed;"));
        assert!(extract.contains("Icon=shellzip"));
        assert!(!extract.contains("Actions=Add;"));

        let add = kde_servicemenu_add("/usr/bin/shellzip");
        assert!(add.contains("MimeType=all/all;inode/directory;"));
        assert!(add.contains("Actions=Add;AddZip;Add7z;"));
        assert!(add.contains("--add-zip"));
    }

    #[test]
    fn nemo_includes_add_actions() {
        let names: Vec<_> = nemo_actions("/usr/bin/shellzip")
            .into_iter()
            .map(|(name, _)| name)
            .collect();
        assert!(names.contains(&"shellzip-add.nemo_action".to_string()));
        assert!(names.contains(&"shellzip-extract-here.nemo_action".to_string()));
        let add = nemo_actions("/usr/bin/shellzip")
            .into_iter()
            .find(|(name, _)| name == "shellzip-add.nemo_action")
            .unwrap()
            .1;
        assert!(!add.contains("Mimetypes="));
        assert!(add.contains("Icon-Name=shellzip"));
        assert!(add.contains("Extensions=any;dir;"));
    }

    #[test]
    fn thunar_merge_is_idempotent_and_uses_unique_ids() {
        let first = merge_thunar_uca("", "/usr/bin/shellzip");
        assert!(first.contains("<unique-id>shellzip-open</unique-id>"));
        assert!(first.contains("<unique-id>shellzip-add</unique-id>"));
        assert!(first.contains("*.zip;"));
        let with_other = merge_thunar_uca(
            "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<actions>\n  <action>\n    <name>Open Terminal Here</name>\n    <unique-id>term-1</unique-id>\n  </action>\n</actions>\n",
            "/usr/bin/shellzip",
        );
        assert!(with_other.contains("Open Terminal Here"));
        let second = merge_thunar_uca(&first, "/opt/ShellZip/shellzip");
        assert_eq!(
            second
                .matches("<unique-id>shellzip-open</unique-id>")
                .count(),
            1
        );
        assert!(second.contains("'/opt/ShellZip/shellzip'"));
        let stripped = strip_thunar_shellzip_actions(&second);
        assert!(!stripped.contains("shellzip-open"));
        assert!(!stripped.contains("ShellZip:"));
    }

    #[test]
    fn nautilus_scripts_exec_quoted_path() {
        let scripts = nautilus_scripts("/opt/My Apps/shellzip");
        let open = scripts.iter().find(|(name, _)| name == "Open").unwrap();
        assert!(open.1.starts_with("#!/bin/sh\n"));
        assert!(open.1.contains("'/opt/My Apps/shellzip'"));
    }

    #[test]
    fn pcmanfm_actions_cover_add_and_extract() {
        let files = pcmanfm_actions("/usr/bin/shellzip");
        assert!(files.iter().any(|(name, _)| name == "shellzip-add.desktop"));
        let extract = files
            .iter()
            .find(|(name, _)| name == "shellzip-extract-here.desktop")
            .unwrap();
        assert!(extract.1.contains("application/zip"));
        let add = files
            .iter()
            .find(|(name, _)| name == "shellzip-add.desktop")
            .unwrap();
        assert!(add.1.contains("all/all;inode/directory;"));
    }

    #[test]
    fn registry_plan_mentions_progids() {
        let plan = windows_registry_plan(r"C:\ShellZip\shellzip.exe", r"C:\icons");
        assert!(plan.contains("ShellZip.zip"));
        assert!(plan.contains(r"C:\icons\zip.ico,0"));
        assert!(plan.contains(r"C:\icons\tar.gz.ico,0"));
        assert!(plan.contains(r"*\shellex\ContextMenuHandlers\ShellZip"));
        assert!(plan.contains(r"Directory\shellex\ContextMenuHandlers\ShellZip"));
        assert!(plan.contains(r"Software\ShellZip\ExePath = C:\ShellZip\shellzip.exe"));
    }

    #[test]
    fn plist_fragment_has_types() {
        let plist = macos_document_types();
        assert!(plist.contains("com.shellzip.zip"));
        assert!(plist.contains("CFBundleTypeIconFile"));
    }
}
