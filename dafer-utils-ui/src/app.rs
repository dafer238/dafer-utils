use eframe::egui;
use std::cmp::Ordering;

use dafer_utils::data_loader;
use dafer_utils::query_engine;
use polars::prelude::AnyValue;

use crate::state::AppState;
use crate::ui::main_ui::main_ui;

pub struct MyApp {
    pub state: AppState,
}

impl Default for MyApp {
    fn default() -> Self {
        Self {
            state: AppState::default(),
        }
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        ctx.set_zoom_factor(1.2);

        // On-demand repaint: only repaint when UI interaction happens.
        // This eliminates the continuous 90fps loop that made tables slow.

        // Recompute preview when pipeline changes (once per dirty flag)
        if self.state.preview_dirty {
            self.recompute_preview();
        }

        // Rebuild table string cache when sort or data changes
        if self.state.table_cache_dirty {
            self.rebuild_table_cache();
        }

        main_ui(ctx, &mut self.state);
    }
}

impl MyApp {
    /// Recompute the preview DataFrame from the current source + operations.
    fn recompute_preview(&mut self) {
        let state = &mut self.state;

        if let Some(source) = &state.source {
            match query_engine::preview(source, &state.operations, state.preview_rows) {
                Ok(df) => {
                    // Auto-detect numeric String columns on first load
                    if !state.auto_cast_detected {
                        let numeric_cols = data_loader::detect_numeric_string_columns(&df);
                        if !numeric_cols.is_empty() {
                            if let Some(ref mut src) = state.source {
                                src.auto_numeric_cols = numeric_cols;
                            }
                            state.auto_cast_detected = true;
                            // Re-collect with casts applied next frame
                            state.preview_dirty = true;
                            state.status = "Auto-detected numeric columns, re-loading...".into();
                            return;
                        }
                        state.auto_cast_detected = true;
                    }

                    // Cache column metadata
                    state.column_names = df
                        .get_column_names()
                        .iter()
                        .map(|s| s.to_string())
                        .collect();
                    state.column_dtypes =
                        df.dtypes().iter().map(|d| format!("{}", d)).collect();
                    state.row_count = Some(df.height());

                    // Compute stats and plot data from the FULL dataset
                    match query_engine::execute(source, &state.operations) {
                        Ok(full) => {
                            state.column_stats = data_loader::column_stats(&full);
                            // Format min/max floats to 4 decimal places for display
                            for stat in &mut state.column_stats {
                                stat.min = stat.min.take().map(|s| format_stat_float(&s));
                                stat.max = stat.max.take().map(|s| format_stat_float(&s));
                            }
                            state.full_df = Some(full);
                        }
                        Err(_) => {
                            state.column_stats = data_loader::column_stats(&df);
                            for stat in &mut state.column_stats {
                                stat.min = stat.min.take().map(|s| format_stat_float(&s));
                                stat.max = stat.max.take().map(|s| format_stat_float(&s));
                            }
                            state.full_df = None;
                        }
                    }

                    if state.select_checks.len() != state.column_names.len() {
                        state.select_checks = vec![true; state.column_names.len()];
                    }

                    let total_rows = state.full_df.as_ref().map(|f| f.height()).unwrap_or(df.height());
                    state.preview_df = Some(df);
                    state.plot_dirty = true;
                    state.table_cache_dirty = true;
                    state.status = format!(
                        "Showing {} of {} rows x {} columns",
                        state.row_count.unwrap_or(0),
                        total_rows,
                        state.column_names.len()
                    );
                }
                Err(e) => {
                    state.status = format!("Preview error: {}", e);
                    state.preview_df = None;
                    state.cached_cell_strings.clear();
                    state.cached_header_names.clear();
                }
            }
        } else {
            state.preview_df = None;
            state.full_df = None;
            state.column_names.clear();
            state.column_dtypes.clear();
            state.column_stats.clear();
            state.row_count = None;
            state.cached_cell_strings.clear();
            state.cached_header_names.clear();
            state.status = "No file loaded".to_string();
        }

        state.preview_dirty = false;
    }

    /// Build the pre-computed string grid from the preview DataFrame.
    /// Applies visual sort if active. This runs once per sort/data change.
    fn rebuild_table_cache(&mut self) {
        let state = &mut self.state;
        state.table_cache_dirty = false;

        let Some(ref df) = state.preview_df else {
            state.cached_cell_strings.clear();
            state.cached_header_names.clear();
            return;
        };

        let n_rows = df.height();
        let n_cols = df.width();

        // Build header names
        state.cached_header_names = df
            .get_column_names()
            .iter()
            .map(|s| s.to_string())
            .collect();

        // Build cell strings [row][col]
        let columns = df.get_columns();
        let mut grid: Vec<Vec<String>> = Vec::with_capacity(n_rows);
        for row_idx in 0..n_rows {
            let mut row_strs = Vec::with_capacity(n_cols);
            for col_s in columns {
                let val = col_s
                    .get(row_idx)
                    .map(|v| format_cell_value(&v))
                    .unwrap_or_default();
                row_strs.push(val);
            }
            grid.push(row_strs);
        }

        // Apply visual sort if active
        if let Some(ref sort_col) = state.sort_column {
            if let Some(col_idx) = state
                .cached_header_names
                .iter()
                .position(|n| n == sort_col)
            {
                let descending = state.sort_descending;
                grid.sort_by(|a, b| {
                    let ord = natural_cmp(&a[col_idx], &b[col_idx]);
                    if descending {
                        ord.reverse()
                    } else {
                        ord
                    }
                });
            }
        }

        state.cached_cell_strings = grid;
    }
}

// ─── Helpers ──────────────────────────────────────────────────────────────────

/// Format an AnyValue for table display.
/// Null → empty, NaN → "NaN", strings → unquoted.
fn format_cell_value(v: &AnyValue) -> String {
    match v {
        AnyValue::Null => String::new(),
        AnyValue::Float64(f) if f.is_nan() => "NaN".to_string(),
        AnyValue::Float32(f) if f.is_nan() => "NaN".to_string(),
        AnyValue::String(s) => s.to_string(),
        AnyValue::StringOwned(s) => s.to_string(),
        other => {
            let s = other.to_string();
            // Strip Polars quote artifacts
            if s.len() >= 2 && s.starts_with('"') && s.ends_with('"') {
                s[1..s.len() - 1].to_string()
            } else {
                s
            }
        }
    }
}

/// Numeric-aware string comparison for natural sorting.
fn natural_cmp(a: &str, b: &str) -> Ordering {
    match (a.parse::<f64>(), b.parse::<f64>()) {
        (Ok(an), Ok(bn)) => an.partial_cmp(&bn).unwrap_or(Ordering::Equal),
        _ => a.cmp(b),
    }
}

/// Format a stat value: if it parses as a float, display with 4 decimal places.
/// Otherwise return as-is (integers, strings, dates, etc.).
fn format_stat_float(s: &str) -> String {
    if s.contains('.') || s.contains('e') || s.contains('E') {
        if let Ok(f) = s.parse::<f64>() {
            return format!("{:.4}", f);
        }
    }
    s.to_string()
}
