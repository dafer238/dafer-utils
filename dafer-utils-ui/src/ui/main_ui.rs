use crate::state::AppState;
use eframe::egui::{self, Frame, RichText};

use crate::ui::load_preview::load_preview_tab;
use crate::ui::modify::modify_tab_ui;
use crate::ui::palette::gruvbox_material::GruvboxMaterial;
use crate::ui::visualize::visualize_tab_ui;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MainTab {
    LoadPreview,
    Modify,
    Visualize,
}

impl MainTab {
    pub fn emoji(&self) -> &'static str {
        match self {
            MainTab::LoadPreview => "📂",
            MainTab::Modify => "⛭",
            MainTab::Visualize => "📊",
        }
    }
}

impl MainTab {
    pub fn all() -> [MainTab; 3] {
        [MainTab::LoadPreview, MainTab::Modify, MainTab::Visualize]
    }
}

static mut SELECTED_TAB: MainTab = MainTab::LoadPreview;

pub fn main_ui(ctx: &egui::Context, state: &mut AppState) {
    let mut selected_tab = unsafe { SELECTED_TAB };

    // Top menu bar
    egui::TopBottomPanel::top("menu_bar")
        .frame(
            Frame::new()
                .fill(GruvboxMaterial::bg(230))
                .inner_margin(2.0),
        )
        .show(ctx, |ui| {
            egui::MenuBar::new().ui(ui, |ui| {
                ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
                    ui.menu_button("File", |ui| {
                        let _ = ui.button(RichText::new("Open..."));
                        let _ = ui.button(RichText::new("Save"));
                        ui.separator();
                        let _ = ui.button(RichText::new("Exit"));
                    });
                    ui.menu_button("Edit", |ui| {
                        let _ = ui.button(RichText::new("Undo"));
                        let _ = ui.button(RichText::new("Redo"));
                        ui.separator();
                        let _ = ui.button(RichText::new("Preferences"));
                    });
                    ui.menu_button("About", |ui| {
                        let _ = ui.button(RichText::new("About dafer-utils"));
                        let _ = ui.button(RichText::new("Help"));
                    });
                });
            });
        });

    egui::TopBottomPanel::bottom("status_bar")
        .frame(
            Frame::new()
                .fill(GruvboxMaterial::bg(230))
                .inner_margin(2.0),
        )
        .show(ctx, |ui| {
            let msg = state.status_message.as_deref().unwrap_or("Ready");
            ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
                ui.label(
                    egui::RichText::new(msg)
                        .small()
                        .color(GruvboxMaterial::fg(255)),
                );
            });
        });

    let _side_panel = egui::SidePanel::left("tab_bar")
        .frame(
            Frame::new()
                .fill(GruvboxMaterial::bg(240))
                .inner_margin(2.0),
        )
        // .resizable(true)
        .max_width(30.0)
        .show(ctx, |ui| {
            ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
                for tab in MainTab::all() {
                    let selected = selected_tab == tab;
                    let label = tab.emoji();
                    let response = ui.selectable_label(selected, label);
                    if response.clicked() {
                        selected_tab = tab;
                    }
                }
            });
        });

    // println!("{:?}", _side_panel.response.rect.width());

    unsafe {
        SELECTED_TAB = selected_tab;
    }

    egui::CentralPanel::default().show(ctx, |ui| match selected_tab {
        MainTab::LoadPreview => load_preview_tab(ui, state),
        MainTab::Modify => modify_tab_ui(ui, state),
        MainTab::Visualize => visualize_tab_ui(ui, state),
    });
}
