use crate::enums::MainTab;
use crate::state::AppState;
use eframe::egui::{self, Frame, RichText};

use crate::ui::load_preview::load_preview_tab;
use crate::ui::modify::modify_tab_ui;
use crate::ui::palette::gruvbox_material::GruvboxMaterial;
use crate::ui::visualize::visualize_tab_ui;

use dafer_utils::persistence::PersistentState;

/// Main UI layout: menu bar + vertical tab bar + central panel + status bar.
pub fn main_ui(ctx: &egui::Context, state: &mut AppState) {
    // ── Top Menu Bar ──
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
                        if ui.button(RichText::new("Open...")).clicked() {
                            if let Some(file) = rfd::FileDialog::new()
                                .add_filter("Data files", &["csv", "tsv", "parquet", "pq"])
                                .pick_file()
                            {
                                open_file(state, file);
                            }
                            ui.close();
                        }
                        ui.separator();
                        if ui.button(RichText::new("Save State...")).clicked() {
                            if let Some(path) = rfd::FileDialog::new()
                                .add_filter("State file", &["dfr"])
                                .save_file()
                            {
                                let persistent = PersistentState {
                                    source: state.source.clone(),
                                    operations: state.operations.clone(),
                                };
                                match persistent.save(&path) {
                                    Ok(()) => state.status = "State saved".to_string(),
                                    Err(e) => {
                                        state.status = format!("Save error: {}", e)
                                    }
                                }
                            }
                            ui.close();
                        }
                        if ui.button(RichText::new("Load State...")).clicked() {
                            if let Some(path) = rfd::FileDialog::new()
                                .add_filter("State file", &["dfr"])
                                .pick_file()
                            {
                                match PersistentState::load(&path) {
                                    Ok(persistent) => {
                                        state.source = persistent.source;
                                        state.operations = persistent.operations;
                                        state.redo_stack.clear();
                                        state.preview_dirty = true;
                                        state.status = "State loaded".to_string();
                                    }
                                    Err(e) => {
                                        state.status = format!("Load error: {}", e)
                                    }
                                }
                            }
                            ui.close();
                        }
                        ui.separator();
                        if ui.button(RichText::new("Exit")).clicked() {
                            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                        }
                    });
                    ui.menu_button("Edit", |ui| {
                        if ui.button(RichText::new("Undo")).clicked() {
                            undo(state);
                            ui.close();
                        }
                        if ui.button(RichText::new("Redo")).clicked() {
                            redo(state);
                            ui.close();
                        }
                        ui.separator();
                        if ui.button(RichText::new("Clear Pipeline")).clicked() {
                            state.operations.clear();
                            state.redo_stack.clear();
                            state.preview_dirty = true;
                            state.status = "Pipeline cleared".to_string();
                            ui.close();
                        }
                    });
                    ui.menu_button("About", |ui| {
                        let _ = ui.button(RichText::new("dafer-utils v0.1.0"));
                        let _ = ui.button(RichText::new("Rust Data Science Desktop App"));
                    });
                });
            });
        });

    // ── Bottom Status Bar ──
    egui::TopBottomPanel::bottom("status_bar")
        .frame(
            Frame::new()
                .fill(GruvboxMaterial::bg(230))
                .inner_margin(2.0),
        )
        .show(ctx, |ui| {
            ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
                ui.label(
                    egui::RichText::new(&state.status)
                        .small()
                        .color(GruvboxMaterial::fg(255)),
                );
                // Show pipeline ops count on the right
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if !state.operations.is_empty() {
                        ui.label(
                            egui::RichText::new(format!(
                                "Pipeline: {} ops",
                                state.operations.len()
                            ))
                            .small()
                            .color(GruvboxMaterial::fg3(200)),
                        );
                    }
                });
            });
        });

    // ── Left Tab Bar (vertical) ──
    egui::SidePanel::left("tab_bar")
        .frame(
            Frame::new()
                .fill(GruvboxMaterial::bg(240))
                .inner_margin(2.0),
        )
        .max_width(30.0)
        .show(ctx, |ui| {
            ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
                for tab in MainTab::all() {
                    let selected = state.selected_tab == tab;
                    let label = tab.emoji();
                    let response = ui.selectable_label(selected, label);
                    if response.clicked() {
                        state.selected_tab = tab;
                    }
                }
            });
        });

    // ── Central Panel (tab content) ──
    egui::CentralPanel::default().show(ctx, |ui| match state.selected_tab {
        MainTab::LoadPreview => load_preview_tab(ui, state),
        MainTab::Modify => modify_tab_ui(ui, state),
        MainTab::Visualize => visualize_tab_ui(ui, state),
    });
}

// ─── Helpers ──────────────────────────────────────────────────────────────────

/// Open a data file, set it as the source, and trigger preview.
fn open_file(state: &mut AppState, path: std::path::PathBuf) {
    if let Some(ds) = dafer_utils::datasource::DataSource::from_path(path.clone()) {
        state.source = Some(ds);
        state.operations.clear();
        state.redo_stack.clear();
        state.preview_dirty = true;
        state.sort_column = None;
        state.sort_descending = false;
        state.auto_cast_detected = false;
        state.selected_cell = None;
        state.selected_row = None;
        state.selected_col = None;
        state.plot_y_columns.clear();
        state.plot_multi_data.clear();
        state.status = format!("Loaded: {}", path.display());
    } else {
        state.status = format!("Unsupported file: {}", path.display());
    }
}

/// Undo: pop last operation and push it onto redo stack.
fn undo(state: &mut AppState) {
    if let Some(op) = state.operations.pop() {
        state.redo_stack.push(op);
        state.preview_dirty = true;
        state.status = "Undo".to_string();
    }
}

/// Redo: pop from redo stack and push onto operations.
fn redo(state: &mut AppState) {
    if let Some(op) = state.redo_stack.pop() {
        state.operations.push(op);
        state.preview_dirty = true;
        state.status = "Redo".to_string();
    }
}
