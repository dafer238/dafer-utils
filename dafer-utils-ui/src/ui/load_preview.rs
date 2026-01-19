use crate::enums::{BrowseMode, FileType};
use crate::state::AppState;
use dafer_utils::data_loader::{collect_head, column_stats, scan_csv, scan_parquet, sort_df};
use eframe::egui;
use egui_extras::{Column, TableBuilder};
use std::path::Path;

/// Render the Data Loading & Preview tab content.
/// File type and browse mode are inferred after selection.
pub fn load_preview_tab(ui: &mut egui::Ui, state: &mut AppState) {
    // --- File/folder picker ---
    ui.horizontal(|ui| {
        if let Some(path) = &state.selected_path {
            ui.monospace(path.display().to_string());
        } else {
            ui.label("No path selected");
        }

        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if ui.button("Browse File").clicked() {
                if let Some(file) = rfd::FileDialog::new().pick_file() {
                    state.selected_path = Some(file.clone());
                    state.browse_mode = BrowseMode::File;
                    state.file_type = infer_file_type(&file);
                }
            }
            if ui.button("Browse Folder").clicked() {
                if let Some(folder) = rfd::FileDialog::new().pick_folder() {
                    state.selected_path = Some(folder.clone());
                    state.browse_mode = BrowseMode::Folder;
                    state.file_type = FileType::default();
                }
            }
        });
    });

    ui.separator();

    // --- Show inferred mode and file type ---
    if let Some(_path) = &state.selected_path {
        ui.horizontal(|ui| {
            ui.label(format!(
                "Detected: {} ({:?})",
                match state.browse_mode {
                    BrowseMode::File => "File",
                    BrowseMode::Folder => "Folder",
                },
                state.file_type
            ));
        });
    }

    ui.separator();

    // --- Data preview for supported file types using egui_extras TableBuilder ---
    // State for sorting
    static mut SORT_COLUMN: Option<usize> = None;
    static mut SORT_DESC: bool = false;

    if let Some(path) = &state.selected_path {
        if state.browse_mode == BrowseMode::File {
            let lf_opt = match state.file_type {
                FileType::Csv => scan_csv(path.to_str().unwrap()).ok(),
                FileType::Parquet => scan_parquet(path.to_str().unwrap()).ok(),
                _ => None,
            };
            if let Some(lf) = lf_opt {
                // Collect head for preview
                let mut df =
                    collect_head(&lf, 20).unwrap_or_else(|_| polars::prelude::DataFrame::default());
                let col_stats = column_stats(&df);

                // Sorting logic
                let col_names = df.get_column_names();
                let col_names_owned: Vec<String> =
                    col_names.iter().map(|s| s.to_string()).collect();
                let n_cols = df.width();

                let mut sort_column = unsafe { SORT_COLUMN };
                let mut sort_desc = unsafe { SORT_DESC };

                // Apply sorting before creating the table
                if let Some(idx) = sort_column {
                    if idx < col_names_owned.len() {
                        if let Ok(sorted) = sort_df(&df, &col_names_owned[idx], sort_desc) {
                            df = sorted;
                        }
                    }
                }

                ui.group(|ui| {
                    ui.label("Preview (first 20 rows):");
                    ui.add_space(2.0);

                    TableBuilder::new(ui)
                        .striped(true)
                        .cell_layout(egui::Layout::left_to_right(egui::Align::Min))
                        .columns(Column::auto(), n_cols)
                        .header(18.0, |mut header| {
                            for (i, name) in col_names_owned.iter().enumerate() {
                                header.col(|ui| {
                                    let label = if Some(i) == sort_column {
                                        if sort_desc {
                                            format!("{} ↓", name)
                                        } else {
                                            format!("{} ↑", name)
                                        }
                                    } else {
                                        name.clone()
                                    };
                                    if ui.button(label).clicked() {
                                        if sort_column == Some(i) {
                                            sort_desc = !sort_desc;
                                        } else {
                                            sort_column = Some(i);
                                            sort_desc = false;
                                        }
                                    }
                                });
                            }
                        })
                        .body(|mut body| {
                            for row_idx in 0..df.height() {
                                body.row(13.0, |mut row| {
                                    for col in df.get_columns() {
                                        let val = col
                                            .get(row_idx)
                                            .map(|v| v.to_string())
                                            .unwrap_or_default();
                                        row.col(|ui| {
                                            ui.label(val);
                                        });
                                    }
                                });
                            }
                        });

                    // --- Column stats below table ---
                    ui.add_space(4.0);
                    ui.label("Column descriptions:");
                    TableBuilder::new(ui)
                        .cell_layout(egui::Layout::left_to_right(egui::Align::Min))
                        .columns(Column::auto(), 6)
                        .header(14.0, |mut header| {
                            header.col(|ui| {
                                ui.label("Name");
                            });
                            header.col(|ui| {
                                ui.label("Type");
                            });
                            header.col(|ui| {
                                ui.label("Min");
                            });
                            header.col(|ui| {
                                ui.label("Max");
                            });
                            header.col(|ui| {
                                ui.label("Nulls");
                            });
                            header.col(|ui| {
                                ui.label("Errors");
                            });
                        })
                        .body(|mut body| {
                            for stat in &col_stats {
                                body.row(12.0, |mut row| {
                                    row.col(|ui| {
                                        ui.label(&stat.name);
                                    });
                                    row.col(|ui| {
                                        ui.label(&stat.dtype);
                                    });
                                    row.col(|ui| {
                                        ui.label(stat.min.as_deref().unwrap_or("-"));
                                    });
                                    row.col(|ui| {
                                        ui.label(stat.max.as_deref().unwrap_or("-"));
                                    });
                                    row.col(|ui| {
                                        ui.label(stat.null_count.to_string());
                                    });
                                    row.col(|ui| {
                                        ui.label(stat.error_count.to_string());
                                    });
                                });
                            }
                        });

                    // Save sorting state
                    unsafe {
                        SORT_COLUMN = sort_column;
                        SORT_DESC = sort_desc;
                    }
                });
            } else {
                ui.label("(Unable to preview file or unsupported format)");
            }
        }
    }

    // --- File metadata and info ---
    ui.group(|ui| {
        if let Some(path) = &state.selected_path {
            if let Ok(meta) = std::fs::metadata(path) {
                ui.label(format!(
                    "File size: {} bytes | Path: {}",
                    meta.len(),
                    path.display()
                ));
            }
        }
    });
}

/// Infer file type from extension.
fn infer_file_type(path: &Path) -> FileType {
    match path
        .extension()
        .and_then(|s| s.to_str())
        .unwrap_or("")
        .to_ascii_lowercase()
        .as_str()
    {
        "csv" => FileType::Csv,
        "parquet" => FileType::Parquet,
        "txt" => FileType::Txt,
        "xls" | "xlsx" => FileType::Excel,
        _ => FileType::default(),
    }
}
