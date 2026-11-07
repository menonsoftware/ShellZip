import Cocoa
import FinderSync

/// Finder Sync extension that launches `shellzip` with Open / Extract verbs.
/// Built only on macOS; enable the extension in System Settings → Extensions.
class ShellZipFinderSync: FIFinderSync {

    override init() {
        super.init()
        FIFinderSyncController.default().directoryURLs = [URL(fileURLWithPath: "/")]
    }

    override func menu(for menuKind: FIMenuKind) -> NSMenu {
        let menu = NSMenu(title: "ShellZip")
        menu.addItem(withTitle: "Open", action: #selector(openArchives(_:)), keyEquivalent: "")
        menu.addItem(withTitle: "Extract to…", action: #selector(extractTo(_:)), keyEquivalent: "")
        menu.addItem(withTitle: "Extract Here", action: #selector(extractHere(_:)), keyEquivalent: "")
        let named = namedLabel()
        menu.addItem(withTitle: named, action: #selector(extractHereNamed(_:)), keyEquivalent: "")
        return menu
    }

    @objc func openArchives(_ sender: AnyObject?) {
        launch(flag: nil)
    }

    @objc func extractTo(_ sender: AnyObject?) {
        launch(flag: "--extract-to")
    }

    @objc func extractHere(_ sender: AnyObject?) {
        launch(flag: "--extract-here")
    }

    @objc func extractHereNamed(_ sender: AnyObject?) {
        launch(flag: "--extract-here-named")
    }

    private func selectedURLs() -> [URL] {
        FIFinderSyncController.default().selectedItemURLs() ?? []
    }

    private func namedLabel() -> String {
        let urls = selectedURLs()
        if urls.count == 1, let name = urls[0].deletingPathExtension().lastPathComponent as String? {
            // Prefer stripping compound suffixes the way ShellZip does.
            let stem = folderStem(urls[0].lastPathComponent)
            return "Extract Here (\(stem))"
        }
        if urls.count > 1 {
            return "Extract Here (separate folders)"
        }
        return "Extract Here (folder)"
    }

    private func folderStem(_ name: String) -> String {
        let lower = name.lowercased()
        for suffix in [".tar.gz", ".tar.bz2", ".tar.xz", ".tar.zst", ".tgz", ".tbz2", ".txz", ".tzst",
                       ".zipx", ".zip", ".7z", ".rar", ".tar", ".gz", ".bz2", ".xz", ".zst"] {
            if lower.hasSuffix(suffix) {
                let cut = name.count - suffix.count
                let stem = String(name.prefix(cut))
                return stem.isEmpty ? "archive" : stem
            }
        }
        return (name as NSString).deletingPathExtension
    }

    private func launch(flag: String?) {
        guard let exe = shellzipURL() else { return }
        let urls = selectedURLs()
        var args: [String] = []
        if let flag = flag {
            args.append(flag)
        }
        args.append(contentsOf: urls.map { $0.path })
        let task = Process()
        task.executableURL = exe
        task.arguments = args
        try? task.run()
    }

    private func shellzipURL() -> URL? {
        if let bundled = Bundle.main.bundleURL
            .deletingLastPathComponent() // PlugIns
            .deletingLastPathComponent() // Contents
            .appendingPathComponent("MacOS/shellzip") as URL?,
           FileManager.default.isExecutableFile(atPath: bundled.path) {
            return bundled
        }
        return URL(fileURLWithPath: "/usr/local/bin/shellzip")
    }
}
