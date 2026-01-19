mod app;
mod enums;
mod state;
mod ui;

fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions::default();
    eframe::run_native(
        "Data handling utils",
        options,
        Box::new(|_cc| Ok(Box::new(app::MyApp::default()))),
    )
}
