use eframe::egui;

use crate::enums::{BrowseMode, FileType, LoadMode, Theme};

use crate::state::AppState;
use crate::ui::main_ui::main_ui;

pub struct MyApp {
    pub state: AppState,
}

impl Default for MyApp {
    fn default() -> Self {
        Self {
            state: AppState {
                theme: Theme::default(),
                browse_mode: BrowseMode::default(),
                file_type: FileType::default(),
                load_mode: LoadMode::default(),
                selected_path: None,
            },
        }
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            // scale everything by 20% (1.2x)
            ctx.set_zoom_factor(1.2);

            main_ui(ui, &mut self.state);
        });
    }
}
