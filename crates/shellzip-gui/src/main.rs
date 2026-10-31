//! ShellZip desktop window.
//!
//! The Windows executable does not open a console window.

#![cfg_attr(all(not(test), target_os = "windows"), windows_subsystem = "windows")]

mod add_dialog;
mod app;
mod chrome;
mod explorer;
mod extract_dialog;
mod icons;
mod settings;
mod shell_icons;
mod theme;
mod worker;

fn main() {
    let launch = std::env::args()
        .nth(1)
        .map(std::path::PathBuf::from)
        .filter(|path| {
            path.as_os_str()
                .to_str()
                .is_none_or(|text| !text.starts_with('-'))
        });
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1080.0, 720.0])
            .with_min_inner_size([760.0, 480.0])
            .with_title("ShellZip"),
        ..Default::default()
    };
    if let Err(err) = eframe::run_native(
        "ShellZip",
        options,
        Box::new(move |cc| Ok(Box::new(app::ShellZipApp::new(cc, launch)))),
    ) {
        anstream_fallback(&err);
        std::process::exit(1);
    }
}

fn anstream_fallback(err: &eframe::Error) {
    eprintln!("shellzip: {err}");
}
