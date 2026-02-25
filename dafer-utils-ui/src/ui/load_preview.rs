use crate::state::AppState;
use crate::ui::palette::gruvbox_material::GruvboxMaterial;
use dafer_utils::datasource::DataSource;
use eframe::egui;
use egui_extras::{Column, TableBuilder};

/// Data Loading & Preview tab.
///
/// High-performance rendering: uses a pre-computed string cache (built in app.rs)
/// instead of accessing the DataFrame during rendering. Column widths are fixed
/// initial values with resizing, eliminating expensive auto-measurement.
///
/// Features:
/// - File picker (browse for CSV/Parquet)
/// - Virtualized table with pre-computed strings
/// - Clickable column headers for visual sorting
/// - Cell/row/column selection + Ctrl+C copy
/// - Alternate row striping
/// - Column statistics and file metadata
pub fn load_preview_tab(ui: &mut egui::Ui, state: &mut AppState) {
    // ── File Picker ──
    ui.horizontal(|ui| {
        if let Some(source) = &state.source {
            ui.monospace(source.path.display().to_string());
            ui.label(format!("({})", source.source_type));
        } else {
            ui.label("No file loaded");
        }

        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if ui.button("Browse File").clicked() {
                if let Some(file) = rfd::FileDialog::new()
                    .add_filter("Data files", &["csv", "tsv", "parquet", "pq"])
                    .pick_file()
                {
                    if let Some(ds) = DataSource::from_path(file.clone()) {
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
                        state.status = format!("Loaded: {}", file.display());
                    } else {
                        state.status = format!("Unsupported: {}", file.display());
                    }
                }
            }
        });
    });

    ui.separator();

    // ── Handle Ctrl+C Copy ──
    if ui.input(|i| i.modifiers.command && i.key_pressed(egui::Key::C)) {
        copy_selection_to_clipboard(ui, state);
    }

    // ── Preview Table (from pre-computed string cache) ──
    if !state.cached_cell_strings.is_empty() && !state.cached_header_names.is_empty() {
        let n_rows = state.cached_cell_strings.len();
        let n_cols = state.cached_header_names.len();

        ui.label(format!("Preview ({n_rows} rows x {n_cols} cols)"));

        let text_height = ui.text_style_height(&egui::TextStyle::Body);
        let row_height = text_height + 2.0;
        let available = ui.available_size();
        // Limit table to ~50% so column stats remain visible
        let table_height = (available.y * 0.5).max(150.0);

        let header_names: Vec<String> = state.cached_header_names.clone();

        egui::ScrollArea::horizontal()
            .id_salt("preview_hscroll")
            .show(ui, |ui| {
                TableBuilder::new(ui)
                    .id_salt("preview_data_table")
                    .striped(true)
                    .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
                    .columns(Column::initial(100.0).at_least(60.0).clip(true).resizable(true), n_cols)
                    .max_scroll_height(table_height)
                    .header(row_height + 4.0, |mut header| {
                        for (col_idx, name) in header_names.iter().enumerate() {
                            header.col(|ui| {
                                let is_sorted =
                                    state.sort_column.as_deref() == Some(name.as_str());
                                let label = if is_sorted {
                                    if state.sort_descending {
                                        format!("{} v", name)
                                    } else {
                                        format!("{} ^", name)
                                    }
                                } else {
                                    name.clone()
                                };

                                let response = ui.add(
                                    egui::Label::new(
                                        egui::RichText::new(&label)
                                            .strong()
                                            .color(GruvboxMaterial::fg(255)),
                                    )
                                    .sense(egui::Sense::click()),
                                );

                                if response.clicked() {
                                    if is_sorted {
                                        state.sort_descending = !state.sort_descending;
                                    } else {
                                        state.sort_column = Some(name.clone());
                                        state.sort_descending = false;
                                    }
                                    state.table_cache_dirty = true;
                                }

                                if response.secondary_clicked() {
                                    state.selected_col = Some(col_idx);
                                    state.selected_cell = None;
                                    state.selected_row = None;
                                }
                            });
                        }
                    })
                    .body(|body| {
                        let has_selection = state.selected_cell.is_some()
                            || state.selected_row.is_some()
                            || state.selected_col.is_some();
                        body.rows(row_height, n_rows, |mut row| {
                            let visual_row = row.index();
                            for col_idx in 0..n_cols {
                                row.col(|ui| {
                                    // Only render content for visible cells
                                    if !ui.is_rect_visible(ui.max_rect()) {
                                        return;
                                    }
                                    if has_selection {
                                        let is_selected =
                                            state.selected_cell == Some((visual_row, col_idx))
                                                || state.selected_row == Some(visual_row)
                                                || state.selected_col == Some(col_idx);
                                        if is_selected {
                                            ui.painter().rect_filled(
                                                ui.max_rect(),
                                                0.0,
                                                GruvboxMaterial::blue(50),
                                            );
                                        }
                                    }
                                    ui.label(
                                        state.cached_cell_strings[visual_row][col_idx].as_str(),
                                    );
                                });
                            }
                        });
                    });
            });

        ui.add_space(4.0);

        // ── Column Statistics (always visible) ──
        if !state.column_stats.is_empty() {
            ui.strong("Column Statistics");
            ui.separator();
            let stats_height = ui.available_height().max(80.0);
            egui::ScrollArea::horizontal()
                .id_salt("stats_hscroll")
                .show(ui, |ui| {
                    TableBuilder::new(ui)
                        .id_salt("preview_stats_table")
                        .striped(true)
                        .cell_layout(egui::Layout::left_to_right(egui::Align::Min))
                        .columns(Column::initial(80.0).at_least(60.0).clip(true).resizable(true), 6)
                        .max_scroll_height(stats_height)
                        .header(18.0, |mut header| {
                            for label in &["Name", "Type", "Min", "Max", "Nulls", "Errors"] {
                                header.col(|ui| {
                                    ui.strong(*label);
                                });
                            }
                        })
                        .body(|body| {
                            let stats = &state.column_stats;
                            body.rows(16.0, stats.len(), |mut row| {
                                let stat = &stats[row.index()];
                                row.col(|ui| { ui.label(&stat.name); });
                                row.col(|ui| { ui.label(&stat.dtype); });
                                row.col(|ui| { ui.label(stat.min.as_deref().unwrap_or("-")); });
                                row.col(|ui| { ui.label(stat.max.as_deref().unwrap_or("-")); });
                                row.col(|ui| { ui.label(stat.null_count.to_string()); });
                                row.col(|ui| { ui.label(stat.error_count.to_string()); });
                            });
                        });
                });
        }
    } else if state.source.is_some() {
        ui.label("Loading preview...");
    } else {
        ui.vertical_centered(|ui| {
            ui.add_space(40.0);
            ui.heading("Welcome to dafer-utils");
            ui.add_space(8.0);
            ui.label("Click 'Browse File' to load a CSV or Parquet file.");
            ui.label("Or use File > Open from the menu bar.");
        });
    }

    // ── File metadata ──
    if let Some(source) = &state.source {
        ui.add_space(4.0);
        ui.separator();
        if let Ok(meta) = std::fs::metadata(&source.path) {
            let size = meta.len();
            let size_str = if size > 1_000_000_000 {
                format!("{:.2} GB", size as f64 / 1_000_000_000.0)
            } else if size > 1_000_000 {
                format!("{:.2} MB", size as f64 / 1_000_000.0)
            } else if size > 1_000 {
                format!("{:.1} KB", size as f64 / 1_000.0)
            } else {
                format!("{} bytes", size)
            };
            ui.label(format!(
                "File: {} | Size: {} | {} | Pipeline: {} ops",
                source.path.display(),
                size_str,
                source.source_type,
                state.operations.len()
            ));
        }
    }
}

/// Copy the current selection (cell, row, or column) to clipboard.
fn copy_selection_to_clipboard(ui: &egui::Ui, state: &AppState) {
    let text = if let Some((row, col)) = state.selected_cell {
        state
            .cached_cell_strings
            .get(row)
            .and_then(|r| r.get(col))
            .cloned()
    } else if let Some(row) = state.selected_row {
        state.cached_cell_strings.get(row).map(|r| r.join("\t"))
    } else if let Some(col) = state.selected_col {
        let vals: Vec<&str> = state
            .cached_cell_strings
            .iter()
            .filter_map(|r| r.get(col).map(|s| s.as_str()))
            .collect();
        Some(vals.join("\n"))
    } else {
        None
    };

    if let Some(text) = text {
        ui.ctx().copy_text(text);
    }
}
