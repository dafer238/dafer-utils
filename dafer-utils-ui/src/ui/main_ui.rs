use crate::enums::{BrowseMode, FileType, LoadMode, Theme};
use crate::state::AppState;
use eframe::egui;

pub fn main_ui(ui: &mut egui::Ui, state: &mut AppState) {
    ui.heading("File / Folder Browser");

    ui.horizontal(|ui| {
        ui.radio_value(&mut state.browse_mode, BrowseMode::File, "File");
        ui.radio_value(&mut state.browse_mode, BrowseMode::Folder, "Folder");
    });

    ui.separator();

    ui.horizontal(|ui| {
        if let Some(path) = &state.selected_path {
            ui.monospace(path.display().to_string());
        } else {
            ui.label("No path selected");
        }

        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if ui.button("Browse").clicked() {
                state.selected_path = match state.browse_mode {
                    BrowseMode::File => rfd::FileDialog::new().pick_file(),
                    BrowseMode::Folder => rfd::FileDialog::new().pick_folder(),
                };
            }
        });
    });
    todo!("not name this ui function like the folder");
}
