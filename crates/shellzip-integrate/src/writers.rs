//! Text writers for desktop entries, service menus, and registry plans.

use crate::formats::{EXTENSIONS, MIME_TYPES};
use crate::menu::{MenuAction, action_flag, action_label, linux_named_label};

/// Freedesktop `.desktop` file body. `exec_path` is the `shellzip` binary.
pub fn desktop_entry(exec_path: &str) -> String {
    let mimes = MIME_TYPES.join(";");
    let open = format!("{exec_path} %F");
    let extract_to = format!("{exec_path} --extract-to %F");
    let extract_here = format!("{exec_path} --extract-here %F");
    let extract_named = format!("{exec_path} --extract-here-named %F");
    format!(
        "\
[Desktop Entry]
Type=Application
Name=ShellZip
Comment=Open, test, extract, and create archives
Exec={open}
Icon=shellzip
Terminal=false
Categories=Utility;Archiving;
MimeType={mimes};
StartupNotify=true
Actions=ExtractTo;ExtractHere;ExtractHereNamed;

[Desktop Action ExtractTo]
Name=Extract to…
Exec={extract_to}

[Desktop Action ExtractHere]
Name=Extract Here
Exec={extract_here}

[Desktop Action ExtractHereNamed]
Name={named}
Exec={extract_named}
",
        named = linux_named_label(),
    )
}

/// KDE Dolphin / Gwenview service menu.
pub fn kde_servicemenu(exec_path: &str) -> String {
    let mimes = MIME_TYPES.join(";");
    format!(
        "\
[Desktop Entry]
Type=Service
ServiceTypes=KonqPopupMenu/Plugin
MimeType={mimes};
Actions=Open;ExtractTo;ExtractHere;ExtractHereNamed;
X-KDE-Priority=TopLevel
X-KDE-Submenu=ShellZip

[Desktop Action Open]
Name=Open
Icon=shellzip
Exec={exec_path} %F

[Desktop Action ExtractTo]
Name=Extract to…
Icon=archive-extract
Exec={exec_path} --extract-to %F

[Desktop Action ExtractHere]
Name=Extract Here
Icon=archive-extract
Exec={exec_path} --extract-here %F

[Desktop Action ExtractHereNamed]
Name={named}
Icon=folder-new
Exec={exec_path} --extract-here-named %F
",
        named = linux_named_label(),
    )
}

/// One Nemo `.nemo_action` file body for `action`.
pub fn nemo_action(exec_path: &str, action: MenuAction, name: &str) -> String {
    let flag = action_flag(action);
    let command = if flag.is_empty() {
        format!("{exec_path} %F")
    } else {
        format!("{exec_path} {flag} %F")
    };
    let mimes = MIME_TYPES.join(";");
    format!(
        "\
[Nemo Action]
Active=true
Name={name}
Comment=ShellZip: {name}
Exec={command}
Selection=any
Extensions=any;
Mimetypes={mimes};
"
    )
}

/// All Nemo action files as `(file_name, contents)` pairs.
pub fn nemo_actions(exec_path: &str) -> Vec<(String, String)> {
    vec![
        (
            "shellzip-open.nemo_action".to_string(),
            nemo_action(exec_path, MenuAction::Open, action_label(MenuAction::Open)),
        ),
        (
            "shellzip-extract-to.nemo_action".to_string(),
            nemo_action(
                exec_path,
                MenuAction::ExtractTo,
                action_label(MenuAction::ExtractTo),
            ),
        ),
        (
            "shellzip-extract-here.nemo_action".to_string(),
            nemo_action(
                exec_path,
                MenuAction::ExtractHere,
                action_label(MenuAction::ExtractHere),
            ),
        ),
        (
            "shellzip-extract-here-named.nemo_action".to_string(),
            nemo_action(exec_path, MenuAction::ExtractHereNamed, linux_named_label()),
        ),
    ]
}

/// Thunar custom-actions XML snippet.
pub fn thunar_uca(exec_path: &str) -> String {
    let patterns = EXTENSIONS
        .iter()
        .map(|k| format!("*.{}", k.extension))
        .collect::<Vec<_>>()
        .join(";");
    format!(
        "\
<?xml version=\"1.0\" encoding=\"UTF-8\"?>
<!-- ShellZip Thunar custom actions. Merge into ~/.config/Thunar/uca.xml -->
<actions>
  <action>
    <icon>shellzip</icon>
    <name>ShellZip: Open</name>
    <command>{exec_path} %F</command>
    <description>Open with ShellZip</description>
    <patterns>{patterns}</patterns>
    <directories/>
    <audio-files/>
    <image-files/>
    <other-files/>
    <text-files/>
    <video-files/>
  </action>
  <action>
    <icon>archive-extract</icon>
    <name>ShellZip: Extract to…</name>
    <command>{exec_path} --extract-to %F</command>
    <description>Extract with ShellZip</description>
    <patterns>{patterns}</patterns>
    <other-files/>
  </action>
  <action>
    <icon>archive-extract</icon>
    <name>ShellZip: Extract Here</name>
    <command>{exec_path} --extract-here %F</command>
    <description>Extract here with ShellZip</description>
    <patterns>{patterns}</patterns>
    <other-files/>
  </action>
  <action>
    <icon>folder-new</icon>
    <name>ShellZip: {named}</name>
    <command>{exec_path} --extract-here-named %F</command>
    <description>Extract into a new folder</description>
    <patterns>{patterns}</patterns>
    <other-files/>
  </action>
</actions>
",
        named = linux_named_label(),
    )
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
    out.push_str(
        "\
*\\shellex\\ContextMenuHandlers\\ShellZip = {CLSID}
Directory\\shellex\\ContextMenuHandlers\\ShellZip = {CLSID}
Approved shell extension list += ShellZip
",
    );
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
    }

    #[test]
    fn registry_plan_mentions_progids() {
        let plan = windows_registry_plan(r"C:\ShellZip\shellzip.exe", r"C:\icons");
        assert!(plan.contains("ShellZip.zip"));
        assert!(plan.contains(r"C:\icons\zip.ico,0"));
        assert!(plan.contains(r"C:\icons\tar.gz.ico,0"));
        assert!(plan.contains(r"*\shellex\ContextMenuHandlers\ShellZip"));
        assert!(plan.contains(r"Directory\shellex\ContextMenuHandlers\ShellZip"));
    }

    #[test]
    fn plist_fragment_has_types() {
        let plist = macos_document_types();
        assert!(plist.contains("com.shellzip.zip"));
        assert!(plist.contains("CFBundleTypeIconFile"));
    }
}
