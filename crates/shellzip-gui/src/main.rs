//! ShellZip desktop window.
//!
//! The Windows executable does not open a console window.

#![cfg_attr(all(not(test), target_os = "windows"), windows_subsystem = "windows")]

mod about;
mod add_dialog;
mod app;
mod chrome;
mod credits;
mod explorer;
mod extract_dialog;
mod icons;
mod info_dialog;
mod launch;
mod package;
mod progress_bar;
mod progress_dialog;
mod rename_dialog;
mod report_dialog;
mod settings;
mod sfx_dialog;
mod shell_icons;
mod test_dialog;
mod theme;
mod update;
mod view_dialog;
mod worker;

fn main() -> std::process::ExitCode {
    // Store packages update through Partner Center; Velopack must not write next to the exe.
    if !package::is_packaged() {
        run_velopack_startup();
    }

    let mut argv = std::env::args();
    let _program = argv.next();
    match launch::parse(argv) {
        launch::Startup::Exit(code) => code,
        launch::Startup::Run(action) => {
            run_window(action);
            std::process::ExitCode::SUCCESS
        }
    }
}

/// Register install/uninstall hooks and apply pending updates before the window opens.
///
/// Velopack FastCallbacks (`on_after_install_*` / `on_before_uninstall_*`) exist only on
/// Windows. On Linux and macOS, integrate on first run and again after an update restart;
/// there is no uninstall hook (deleting the AppImage / `.app` does not invoke Velopack).
fn run_velopack_startup() {
    let mut app = velopack::VelopackApp::build();
    #[cfg(target_os = "windows")]
    {
        app = app
            .on_after_install_fast_callback(|_| {
                let _ = launch::run_integrate();
            })
            .on_after_update_fast_callback(|_| {
                let _ = launch::run_integrate();
            })
            .on_before_uninstall_fast_callback(|_| {
                let _ = launch::run_unintegrate();
            });
    }
    app = app
        .on_first_run(|_| {
            let _ = launch::run_integrate();
        })
        .on_restarted(|_| {
            let _ = launch::run_integrate();
        });
    app.run();
}

fn run_window(action: launch::LaunchAction) {
    let mut viewport = egui::ViewportBuilder::default()
        .with_inner_size([1080.0, 720.0])
        .with_min_inner_size([820.0, 480.0])
        .with_title("ShellZip");
    if let Some(icon) = window_icon() {
        viewport = viewport.with_icon(icon);
    }
    let options = eframe::NativeOptions {
        viewport,
        ..Default::default()
    };
    if let Err(err) = eframe::run_native(
        "ShellZip",
        options,
        Box::new(move |cc| Ok(Box::new(app::ShellZipApp::new(cc, action)))),
    ) {
        anstream_fallback(&err);
        std::process::exit(1);
    }
}

fn anstream_fallback(err: &eframe::Error) {
    eprintln!("shellzip: {err}");
}

/// Title-bar and taskbar icon. The Windows file icon is embedded by `build.rs`.
fn window_icon() -> Option<egui::IconData> {
    let image = image::load_from_memory(include_bytes!("../assets/icon.png")).ok()?;
    let mut rgba = image.into_rgba8();
    clear_white_margin(&mut rgba);
    let (width, height) = rgba.dimensions();
    Some(egui::IconData {
        rgba: rgba.into_raw(),
        width,
        height,
    })
}

/// The source tile is a rounded square on an opaque white matte. Flood-fill that
/// matte from the edges so the title bar shows the window color instead of a white box.
fn clear_white_margin(image: &mut image::RgbaImage) {
    let width = image.width();
    let height = image.height();
    if width == 0 || height == 0 {
        return;
    }
    let cells = usize::try_from(width)
        .ok()
        .and_then(|w| usize::try_from(height).ok().and_then(|h| w.checked_mul(h)))
        .unwrap_or(0);
    if cells == 0 {
        return;
    }
    let mut seen = vec![false; cells];
    let mut pending = Vec::new();
    for x in 0..width {
        pending.push((x, 0));
        pending.push((x, height - 1));
    }
    for y in 0..height {
        pending.push((0, y));
        pending.push((width - 1, y));
    }
    while let Some((x, y)) = pending.pop() {
        let Some(index) = pixel_index(width, x, y) else {
            continue;
        };
        if seen.get(index).copied().unwrap_or(true) {
            continue;
        }
        seen[index] = true;
        if !is_white_margin(*image.get_pixel(x, y)) {
            continue;
        }
        image.put_pixel(x, y, image::Rgba([0, 0, 0, 0]));
        if x > 0 {
            pending.push((x - 1, y));
        }
        if y > 0 {
            pending.push((x, y - 1));
        }
        if x + 1 < width {
            pending.push((x + 1, y));
        }
        if y + 1 < height {
            pending.push((x, y + 1));
        }
    }
}

fn pixel_index(width: u32, x: u32, y: u32) -> Option<usize> {
    usize::try_from(y)
        .ok()?
        .checked_mul(usize::try_from(width).ok()?)?
        .checked_add(usize::try_from(x).ok()?)
}

/// Light, near-gray pixels are the white matte or its anti-aliased edge.
fn is_white_margin(pixel: image::Rgba<u8>) -> bool {
    let [red, green, blue, alpha] = pixel.0;
    if alpha == 0 {
        return true;
    }
    let max = red.max(green).max(blue);
    let min = red.min(green).min(blue);
    min >= 96 && max - min <= 18
}

#[cfg(test)]
mod tests {
    use super::window_icon;

    #[test]
    fn window_icon_is_square_and_its_corners_are_clear() {
        let icon = window_icon().expect("icon png");
        assert_eq!(icon.width, 256);
        assert_eq!(icon.height, 256);
        let pixels = usize::try_from(icon.width).unwrap() * usize::try_from(icon.height).unwrap();
        assert_eq!(icon.rgba.len(), pixels * 4);
        let alpha = |x: u32, y: u32| {
            let index = (usize::try_from(y).unwrap() * usize::try_from(icon.width).unwrap()
                + usize::try_from(x).unwrap())
                * 4
                + 3;
            icon.rgba[index]
        };
        assert_eq!(alpha(0, 0), 0, "top-left matte");
        assert_eq!(alpha(icon.width - 1, 0), 0, "top-right matte");
        assert_eq!(alpha(0, icon.height - 1), 0, "bottom-left matte");
        assert_eq!(alpha(8, 8), 0, "white padding");
        assert_eq!(alpha(icon.width / 2, icon.height / 2), 255, "shell");
    }

    #[test]
    fn windows_icon_file_is_an_icon() {
        let bytes = include_bytes!("../assets/shellzip.ico");
        assert_eq!(&bytes[..4], &[0, 0, 1, 0]);
        let count = u16::from_le_bytes([bytes[4], bytes[5]]);
        assert!(count >= 5, "icon sizes: {count}");
    }

    #[test]
    fn macos_icon_file_is_icns() {
        let bytes = include_bytes!("../assets/shellzip.icns");
        assert_eq!(&bytes[..4], b"icns");
    }
}
