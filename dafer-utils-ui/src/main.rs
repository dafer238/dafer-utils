mod app;
mod enums;
mod state;
mod ui;

fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions::default();
    eframe::run_native(
        "Browser app",
        options,
        Box::new(|_cc| Ok(Box::new(app::MyApp::default()))),
    )
}
