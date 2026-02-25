use crate::enums::ExportFormat;
use crate::state::AppState;
use dafer_utils::execution;
use dafer_utils::operations::*;
use eframe::egui;
use egui_extras::{Column, TableBuilder};

/// Data Modification tab.
///
/// Layout:
/// - Top: three-column toolbar (Pipeline | Operation Builder | Export)
/// - Bottom: table preview of current pipeline result (uses cached strings)
pub fn modify_tab_ui(ui: &mut egui::Ui, state: &mut AppState) {
    if state.source.is_none() {
        ui.vertical_centered(|ui| {
            ui.add_space(40.0);
            ui.heading("No data loaded");
            ui.label("Load a file in the Load & Preview tab first.");
        });
        return;
    }

    let col_names = state.column_names.clone();

    // ── Top Toolbar: 3-column layout ──
    ui.columns(3, |cols| {
        // Column 0: Pipeline
        cols[0].group(|ui| {
            ui.set_min_height(180.0);
            ui.strong("Pipeline");
            ui.separator();
            egui::ScrollArea::vertical()
                .id_salt("pipeline_scroll")
                .max_height(120.0)
                .show(ui, |ui| {
                    if state.operations.is_empty() {
                        ui.label("No operations yet.");
                    } else {
                        let mut remove_idx: Option<usize> = None;
                        for (i, op) in state.operations.iter().enumerate() {
                            ui.horizontal(|ui| {
                                ui.label(format!("{}.", i + 1));
                                ui.label(op.to_string());
                                if ui.small_button("X").clicked() {
                                    remove_idx = Some(i);
                                }
                            });
                        }
                        if let Some(idx) = remove_idx {
                            state.operations.remove(idx);
                            state.redo_stack.clear();
                            state.preview_dirty = true;
                            state.status = "Operation removed".to_string();
                        }
                    }
                });
            ui.horizontal(|ui| {
                if ui.small_button("Undo").clicked() {
                    if let Some(op) = state.operations.pop() {
                        state.redo_stack.push(op);
                        state.preview_dirty = true;
                        state.status = "Undo".to_string();
                    }
                }
                if ui.small_button("Redo").clicked() {
                    if let Some(op) = state.redo_stack.pop() {
                        state.operations.push(op);
                        state.preview_dirty = true;
                        state.status = "Redo".to_string();
                    }
                }
                if ui.small_button("Clear").clicked() {
                    state.operations.clear();
                    state.redo_stack.clear();
                    state.preview_dirty = true;
                    state.status = "Pipeline cleared".to_string();
                }
            });
        });

        // Column 1: Operation Builder
        cols[1].group(|ui| {
            ui.set_min_height(180.0);
            ui.strong("Add Operation");
            ui.separator();
            egui::ScrollArea::vertical()
                .id_salt("op_builder_scroll")
                .max_height(150.0)
                .show(ui, |ui| {
                    egui::ComboBox::from_label("Operation")
                        .selected_text(state.selected_op.to_string())
                        .show_ui(ui, |ui| {
                            for op_type in OperationType::all() {
                                ui.selectable_value(
                                    &mut state.selected_op,
                                    *op_type,
                                    op_type.to_string(),
                                );
                            }
                        });
                    ui.add_space(2.0);

                    match state.selected_op {
                        OperationType::Filter => {
                            render_filter_builder(ui, state, &col_names)
                        }
                        OperationType::Sort => {
                            render_sort_builder(ui, state, &col_names)
                        }
                        OperationType::DropColumn => {
                            render_drop_builder(ui, state, &col_names)
                        }
                        OperationType::RenameColumn => {
                            render_rename_builder(ui, state, &col_names)
                        }
                        OperationType::SelectColumns => {
                            render_select_builder(ui, state, &col_names)
                        }
                        OperationType::Limit => render_limit_builder(ui, state),
                        OperationType::FillNull => {
                            render_fill_null_builder(ui, state, &col_names)
                        }
                        OperationType::CastColumn => {
                            render_cast_builder(ui, state, &col_names)
                        }
                        OperationType::ParseDatetime => {
                            render_parse_datetime_builder(ui, state, &col_names)
                        }
                    }
                });
        });

        // Column 2: Export
        cols[2].group(|ui| {
            ui.set_min_height(180.0);
            ui.strong("Export");
            ui.separator();

            egui::ComboBox::from_label("Format")
                .selected_text(state.export_format.to_string())
                .show_ui(ui, |ui| {
                    ui.selectable_value(
                        &mut state.export_format,
                        ExportFormat::Csv,
                        "CSV",
                    );
                    ui.selectable_value(
                        &mut state.export_format,
                        ExportFormat::Parquet,
                        "Parquet",
                    );
                });

            ui.add_space(4.0);

            if ui.button("Export...").clicked() {
                let ext = match state.export_format {
                    ExportFormat::Csv => "csv",
                    ExportFormat::Parquet => "parquet",
                };
                if let Some(path) = rfd::FileDialog::new()
                    .add_filter("Export file", &[ext])
                    .save_file()
                {
                    if let Some(source) = &state.source {
                        let result = match state.export_format {
                            ExportFormat::Csv => {
                                execution::export_csv(source, &state.operations, &path)
                            }
                            ExportFormat::Parquet => {
                                execution::export_parquet(
                                    source,
                                    &state.operations,
                                    &path,
                                )
                            }
                        };
                        match result {
                            Ok(()) => {
                                state.status =
                                    format!("Exported to {}", path.display());
                            }
                            Err(e) => {
                                state.status = format!("Export error: {}", e);
                            }
                        }
                    }
                }
            }

            ui.add_space(8.0);
            ui.label(
                egui::RichText::new(format!(
                    "{} operations in pipeline",
                    state.operations.len()
                ))
                .small(),
            );
        });
    });

    ui.separator();

    // ── Table Preview (uses cached strings for performance) ──
    if !state.cached_cell_strings.is_empty() && !state.cached_header_names.is_empty() {
        let n_rows = state.cached_cell_strings.len();
        let n_cols = state.cached_header_names.len();

        ui.label(format!("Preview ({n_rows} rows x {n_cols} cols)"));

        let text_height = ui.text_style_height(&egui::TextStyle::Body);
        let row_height = text_height + 4.0;
        let available = ui.available_size();
        let table_height = (available.y - 10.0).max(100.0);
        let header_names: Vec<String> = state.cached_header_names.clone();

        egui::ScrollArea::horizontal()
            .id_salt("modify_hscroll")
            .show(ui, |ui| {
                TableBuilder::new(ui)
                    .id_salt("modify_data_table")
                    .striped(true)
                    .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
                    .columns(Column::initial(100.0).at_least(60.0).clip(true).resizable(true), n_cols)
                    .max_scroll_height(table_height)
                    .header(row_height + 2.0, |mut header| {
                        for name in &header_names {
                            header.col(|ui| {
                                ui.strong(name.as_str());
                            });
                        }
                    })
                    .body(|body| {
                        body.rows(row_height, n_rows, |mut row| {
                            let visual_row = row.index();
                            for col_idx in 0..n_cols {
                                row.col(|ui| {
                                    if !ui.is_rect_visible(ui.max_rect()) {
                                        return;
                                    }
                                    ui.label(
                                        state.cached_cell_strings[visual_row][col_idx].as_str(),
                                    );
                                });
                            }
                        });
                    });
            });
    }
}

// ─── Operation Builders ───────────────────────────────────────────────────────

fn render_filter_builder(ui: &mut egui::Ui, state: &mut AppState, col_names: &[String]) {
    column_combo(ui, "Column", &mut state.filter_column, col_names);

    egui::ComboBox::from_label("Operator")
        .selected_text(state.filter_op.to_string())
        .show_ui(ui, |ui| {
            for op in FilterOp::all() {
                ui.selectable_value(&mut state.filter_op, op.clone(), op.to_string());
            }
        });

    if state.filter_op.needs_value() {
        ui.horizontal(|ui| {
            ui.label("Value:");
            ui.text_edit_singleline(&mut state.filter_value);
        });
    }

    if ui.button("Apply Filter").clicked() && !state.filter_column.is_empty() {
        let op = Operation::Filter {
            column: state.filter_column.clone(),
            op: state.filter_op.clone(),
            value: state.filter_value.clone(),
        };
        apply_op(state, op);
    }
}

fn render_sort_builder(ui: &mut egui::Ui, state: &mut AppState, col_names: &[String]) {
    column_combo(ui, "Column", &mut state.sort_op_column, col_names);
    ui.checkbox(&mut state.sort_op_descending, "Descending");

    if ui.button("Apply Sort").clicked() && !state.sort_op_column.is_empty() {
        let op = Operation::Sort {
            column: state.sort_op_column.clone(),
            descending: state.sort_op_descending,
        };
        apply_op(state, op);
    }
}

fn render_drop_builder(ui: &mut egui::Ui, state: &mut AppState, col_names: &[String]) {
    column_combo(ui, "Column to drop", &mut state.drop_column, col_names);

    if ui.button("Drop Column").clicked() && !state.drop_column.is_empty() {
        let op = Operation::DropColumn(state.drop_column.clone());
        apply_op(state, op);
    }
}

fn render_rename_builder(ui: &mut egui::Ui, state: &mut AppState, col_names: &[String]) {
    column_combo(ui, "From", &mut state.rename_from, col_names);
    ui.horizontal(|ui| {
        ui.label("To:");
        ui.text_edit_singleline(&mut state.rename_to);
    });

    if ui.button("Rename Column").clicked()
        && !state.rename_from.is_empty()
        && !state.rename_to.is_empty()
    {
        let op = Operation::RenameColumn {
            from: state.rename_from.clone(),
            to: state.rename_to.clone(),
        };
        apply_op(state, op);
    }
}

fn render_select_builder(ui: &mut egui::Ui, state: &mut AppState, col_names: &[String]) {
    if state.select_checks.len() != col_names.len() {
        state.select_checks = vec![true; col_names.len()];
    }

    ui.label("Select columns to keep:");
    ui.horizontal_wrapped(|ui| {
        for (i, name) in col_names.iter().enumerate() {
            if i < state.select_checks.len() {
                ui.checkbox(&mut state.select_checks[i], name);
            }
        }
    });

    ui.horizontal(|ui| {
        if ui.button("Select All").clicked() {
            state.select_checks.fill(true);
        }
        if ui.button("Deselect All").clicked() {
            state.select_checks.fill(false);
        }
    });

    if ui.button("Apply Selection").clicked() {
        let selected: Vec<String> = col_names
            .iter()
            .enumerate()
            .filter(|(i, _)| state.select_checks.get(*i).copied().unwrap_or(false))
            .map(|(_, name)| name.clone())
            .collect();
        if !selected.is_empty() {
            let op = Operation::SelectColumns(selected);
            apply_op(state, op);
        }
    }
}

fn render_limit_builder(ui: &mut egui::Ui, state: &mut AppState) {
    ui.horizontal(|ui| {
        ui.label("Max rows:");
        ui.add(egui::DragValue::new(&mut state.limit_n).range(1..=u32::MAX));
    });

    if ui.button("Apply Limit").clicked() {
        let op = Operation::Limit(state.limit_n);
        apply_op(state, op);
    }
}

fn render_fill_null_builder(ui: &mut egui::Ui, state: &mut AppState, col_names: &[String]) {
    column_combo(ui, "Column", &mut state.fill_column, col_names);

    egui::ComboBox::from_label("Strategy")
        .selected_text(state.fill_strategy.to_string())
        .show_ui(ui, |ui| {
            for s in FillNullStrategy::all() {
                ui.selectable_value(&mut state.fill_strategy, s.clone(), s.to_string());
            }
        });

    if state.fill_strategy.needs_value() {
        ui.horizontal(|ui| {
            ui.label("Fill value:");
            ui.text_edit_singleline(&mut state.fill_value);
        });
    }

    if ui.button("Apply Fill Null").clicked() && !state.fill_column.is_empty() {
        let value = if state.fill_strategy.needs_value() {
            Some(state.fill_value.clone())
        } else {
            None
        };
        let op = Operation::FillNull {
            column: state.fill_column.clone(),
            strategy: state.fill_strategy.clone(),
            value,
        };
        apply_op(state, op);
    }
}

fn render_cast_builder(ui: &mut egui::Ui, state: &mut AppState, col_names: &[String]) {
    column_combo(ui, "Column", &mut state.cast_column, col_names);

    egui::ComboBox::from_label("Target type")
        .selected_text(state.cast_dtype.to_string())
        .show_ui(ui, |ui| {
            for dt in DTypeTag::all() {
                ui.selectable_value(&mut state.cast_dtype, dt.clone(), dt.to_string());
            }
        });

    if ui.button("Apply Cast").clicked() && !state.cast_column.is_empty() {
        let op = Operation::CastColumn {
            column: state.cast_column.clone(),
            dtype: state.cast_dtype.clone(),
        };
        apply_op(state, op);
    }
}

fn render_parse_datetime_builder(
    ui: &mut egui::Ui,
    state: &mut AppState,
    col_names: &[String],
) {
    column_combo(ui, "Column", &mut state.datetime_column, col_names);
    ui.horizontal(|ui| {
        ui.label("Format:");
        ui.text_edit_singleline(&mut state.datetime_format);
    });
    ui.label(
        egui::RichText::new("e.g. %Y-%m-%d %H:%M:%S").small().weak(),
    );

    if ui.button("Parse Datetime").clicked() && !state.datetime_column.is_empty() {
        let op = Operation::ParseDatetime {
            column: state.datetime_column.clone(),
            format: state.datetime_format.clone(),
        };
        apply_op(state, op);
    }
}

// ─── Helpers ──────────────────────────────────────────────────────────────────

/// Reusable column name combo box.
fn column_combo(ui: &mut egui::Ui, label: &str, selected: &mut String, col_names: &[String]) {
    egui::ComboBox::from_label(label)
        .selected_text(if selected.is_empty() {
            "(select column)"
        } else {
            selected.as_str()
        })
        .show_ui(ui, |ui| {
            for name in col_names {
                ui.selectable_value(selected, name.clone(), name);
            }
        });
}

/// Apply an operation: push to operations, clear redo, mark preview dirty.
fn apply_op(state: &mut AppState, op: Operation) {
    state.status = format!("Applied: {}", op);
    state.operations.push(op);
    state.redo_stack.clear();
    state.preview_dirty = true;
}
