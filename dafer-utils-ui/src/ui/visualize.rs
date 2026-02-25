use crate::enums::PlotType;
use crate::state::AppState;
use chrono::DateTime;
use eframe::egui;
use egui_plot::{Bar, BarChart, GridMark, Legend, Line, Plot, PlotPoints, Points};
use polars::prelude::*;

/// Data Visualization tab.
///
/// - Select X column and multiple Y columns for multi-series plots
/// - Supports Scatter, Line, Bar, Histogram plot types
/// - Each Y column gets its own colored series
/// - Data is extracted from the cached preview DataFrame
pub fn visualize_tab_ui(ui: &mut egui::Ui, state: &mut AppState) {
    if state.source.is_none() {
        ui.vertical_centered(|ui| {
            ui.add_space(40.0);
            ui.heading("No data loaded");
            ui.label("Load a file in the Load & Preview tab first.");
        });
        return;
    }

    if state.preview_df.is_none() {
        ui.label("Preview not available. Load data first.");
        return;
    }

    // ── Plot Configuration ──
    ui.horizontal(|ui| {
        // Plot type
        egui::ComboBox::from_label("Plot type")
            .selected_text(state.plot_type.to_string())
            .show_ui(ui, |ui| {
                for pt in PlotType::all() {
                    if ui
                        .selectable_value(&mut state.plot_type, *pt, pt.to_string())
                        .changed()
                    {
                        state.plot_dirty = true;
                    }
                }
            });

        // X column
        let col_names = state.column_names.clone();
        egui::ComboBox::from_label("X")
            .selected_text(if state.plot_x.is_empty() {
                "(select)"
            } else {
                state.plot_x.as_str()
            })
            .show_ui(ui, |ui| {
                for name in &col_names {
                    if ui
                        .selectable_value(&mut state.plot_x, name.clone(), name)
                        .changed()
                    {
                        state.plot_dirty = true;
                    }
                }
            });

        // Histogram bins
        if state.plot_type == PlotType::Histogram {
            ui.label("Bins:");
            if ui
                .add(egui::DragValue::new(&mut state.histogram_bins).range(5..=200))
                .changed()
            {
                state.plot_dirty = true;
            }
        }
    });

    // ── Y Column(s) selection ──
    {
        let col_names = state.column_names.clone();
        ui.horizontal_wrapped(|ui| {
            ui.label("Y series:");

            // Show existing Y columns with remove buttons
            let mut to_remove: Option<usize> = None;
            for (i, y_col) in state.plot_y_columns.iter().enumerate() {
                ui.group(|ui| {
                    ui.horizontal(|ui| {
                        ui.label(y_col);
                        if ui.small_button("X").clicked() {
                            to_remove = Some(i);
                        }
                    });
                });
            }
            if let Some(idx) = to_remove {
                state.plot_y_columns.remove(idx);
                state.plot_dirty = true;
            }

            // Add new Y column
            ui.menu_button("+ Add Y", |ui| {
                egui::ScrollArea::vertical()
                    .max_height(300.0)
                    .show(ui, |ui| {
                        for name in &col_names {
                            if !state.plot_y_columns.contains(name) {
                                if ui.button(name).clicked() {
                                    state.plot_y_columns.push(name.clone());
                                    state.plot_dirty = true;
                                    ui.close();
                                }
                            }
                        }
                    });
            });
        });
    }

    ui.separator();

    // ── Recompute plot data if dirty ──
    if state.plot_dirty {
        recompute_plot_data(state);
    }

    // ── Reset Zoom ──
    if ui.button("Reset Zoom").clicked() {
        state.plot_reset_counter += 1;
    }

    let plot_height = (ui.available_height() - 10.0).max(200.0);

    // ── Render Plot ──
    if state.plot_type == PlotType::Histogram {
        render_histogram(ui, state, plot_height);
        return;
    }

    if state.plot_multi_data.is_empty() {
        ui.label("Select valid X and Y columns (must be numeric) to plot.");
        return;
    }

    // Compute data bounds for axis auto-fit
    let x_label = state.plot_x.clone();
    let bounds = compute_plot_bounds(&state.plot_multi_data);
    let rc = state.plot_reset_counter;
    let x_is_dt = state.plot_x_is_datetime;

    match state.plot_type {
        PlotType::Scatter => {
            let mut plot = Plot::new(format!("scatter_{rc}"))
                .height(plot_height)
                .x_axis_label(&x_label)
                .show_axes([true, true])
                .show_grid([true, true])
                .legend(Legend::default());
            if x_is_dt {
                plot = plot.x_axis_formatter(datetime_axis_formatter);
                plot = plot.label_formatter(datetime_label_formatter);
            }
            if let Some((x0, x1, y0, y1)) = bounds {
                let xm = (x1 - x0).abs().max(0.1) * 0.05;
                let ym = (y1 - y0).abs().max(0.1) * 0.05;
                plot = plot.include_x(x0 - xm).include_x(x1 + xm)
                           .include_y(y0 - ym).include_y(y1 + ym);
            }
            plot.show(ui, |plot_ui| {
                for (name, data) in &state.plot_multi_data {
                    let points = Points::new(
                        name.as_str(),
                        PlotPoints::new(data.clone()),
                    )
                    .radius(3.0);
                    plot_ui.points(points);
                }
            });
        }
        PlotType::Line => {
            let mut plot = Plot::new(format!("line_{rc}"))
                .height(plot_height)
                .x_axis_label(&x_label)
                .show_axes([true, true])
                .show_grid([true, true])
                .legend(Legend::default());
            if x_is_dt {
                plot = plot.x_axis_formatter(datetime_axis_formatter);
                plot = plot.label_formatter(datetime_label_formatter);
            }
            if let Some((x0, x1, y0, y1)) = bounds {
                let xm = (x1 - x0).abs().max(0.1) * 0.05;
                let ym = (y1 - y0).abs().max(0.1) * 0.05;
                plot = plot.include_x(x0 - xm).include_x(x1 + xm)
                           .include_y(y0 - ym).include_y(y1 + ym);
            }
            plot.show(ui, |plot_ui| {
                for (name, data) in &state.plot_multi_data {
                    let mut sorted_data = data.clone();
                    sorted_data.sort_by(|a, b| {
                        a[0].partial_cmp(&b[0]).unwrap_or(std::cmp::Ordering::Equal)
                    });
                    let line = Line::new(name.as_str(), PlotPoints::new(sorted_data));
                    plot_ui.line(line);
                }
            });
        }
        PlotType::Bar => {
            let mut plot = Plot::new(format!("bar_{rc}"))
                .height(plot_height)
                .x_axis_label(&x_label)
                .show_axes([true, true])
                .show_grid([true, true])
                .legend(Legend::default());
            if x_is_dt {
                plot = plot.x_axis_formatter(datetime_axis_formatter);
                plot = plot.label_formatter(datetime_label_formatter);
            }
            if let Some((x0, x1, y0, y1)) = bounds {
                let xm = (x1 - x0).abs().max(0.1) * 0.05;
                let ym = (y1 - y0).abs().max(0.1) * 0.05;
                plot = plot.include_x(x0 - xm).include_x(x1 + xm)
                           .include_y(y0 - ym).include_y(y1 + ym);
            }
            plot.show(ui, |plot_ui| {
                for (name, data) in &state.plot_multi_data {
                    let bars: Vec<Bar> = data
                        .iter()
                        .map(|[x, y]| Bar::new(*x, *y).width(0.8))
                        .collect();
                    plot_ui.bar_chart(BarChart::new(name.as_str(), bars));
                }
            });
        }
        PlotType::Histogram => {} // handled above
    }
}
/// Compute data bounds across all plot series.
fn compute_plot_bounds(data: &[(String, Vec<[f64; 2]>)]) -> Option<(f64, f64, f64, f64)> {
    let mut x_min = f64::INFINITY;
    let mut x_max = f64::NEG_INFINITY;
    let mut y_min = f64::INFINITY;
    let mut y_max = f64::NEG_INFINITY;
    for (_, points) in data {
        for &[x, y] in points {
            x_min = x_min.min(x);
            x_max = x_max.max(x);
            y_min = y_min.min(y);
            y_max = y_max.max(y);
        }
    }
    if x_min.is_finite() && x_max.is_finite() && y_min.is_finite() && y_max.is_finite() {
        Some((x_min, x_max, y_min, y_max))
    } else {
        None
    }
}
// ─── DateTime Axis Formatting ─────────────────────────────────────────────────

/// Format X axis ticks as datetime strings. Adapts resolution based on visible range.
fn datetime_axis_formatter(mark: GridMark, range: &std::ops::RangeInclusive<f64>) -> String {
    let secs = mark.value as i64;
    let ndt = match DateTime::from_timestamp(secs, 0) {
        Some(dt) => dt.naive_utc(),
        None => return format!("{}", mark.value),
    };
    let span = range.end() - range.start();
    if span < 60.0 {
        ndt.format("%H:%M:%S").to_string()
    } else if span < 3600.0 {
        ndt.format("%H:%M:%S").to_string()
    } else if span < 86400.0 {
        ndt.format("%m-%d %H:%M").to_string()
    } else if span < 86400.0 * 90.0 {
        ndt.format("%Y-%m-%d").to_string()
    } else {
        ndt.format("%Y-%m").to_string()
    }
}

/// Label formatter for hover tooltip when X is datetime.
fn datetime_label_formatter(name: &str, point: &egui_plot::PlotPoint) -> String {
    let secs = point.x as i64;
    let x_str = DateTime::from_timestamp(secs, 0)
        .map(|dt| dt.naive_utc().format("%Y-%m-%d %H:%M:%S").to_string())
        .unwrap_or_else(|| format!("{:.2}", point.x));
    if name.is_empty() {
        format!("x = {x_str}\ny = {:.4}", point.y)
    } else {
        format!("{name}\nx = {x_str}\ny = {:.4}", point.y)
    }
}

// ─── Data Extraction ──────────────────────────────────────────────────────────

/// Recompute multi-series plot data from the preview DataFrame.
fn recompute_plot_data(state: &mut AppState) {
    state.plot_dirty = false;
    state.plot_x_is_datetime = false;

    if state.plot_type == PlotType::Histogram {
        // Histogram only needs Y/X data, handled inline
        return;
    }

    state.plot_multi_data.clear();

    if state.plot_x.is_empty() || state.plot_y_columns.is_empty() {
        return;
    }

    // Use full dataset for plotting (fall back to preview if unavailable)
    let plot_df = state.full_df.as_ref().or(state.preview_df.as_ref());
    if let Some(df) = plot_df {
        // Check if X column is datetime/date type
        let x_vals = if let Ok(series) = df.column(&state.plot_x) {
            match series.dtype() {
                DataType::Datetime(tu, _) => {
                    state.plot_x_is_datetime = true;
                    let divisor = match tu {
                        TimeUnit::Nanoseconds => 1_000_000_000.0,
                        TimeUnit::Microseconds => 1_000_000.0,
                        TimeUnit::Milliseconds => 1_000.0,
                    };
                    if let Ok(casted) = series.cast(&DataType::Float64) {
                        if let Ok(ca) = casted.f64() {
                            ca.into_no_null_iter().map(|v| v / divisor).collect()
                        } else {
                            vec![]
                        }
                    } else {
                        vec![]
                    }
                }
                DataType::Date => {
                    state.plot_x_is_datetime = true;
                    if let Ok(casted) = series.cast(&DataType::Float64) {
                        if let Ok(ca) = casted.f64() {
                            ca.into_no_null_iter().map(|v| v * 86400.0).collect()
                        } else {
                            vec![]
                        }
                    } else {
                        vec![]
                    }
                }
                _ => extract_f64_column(df, &state.plot_x),
            }
        } else {
            vec![]
        };

        for y_col in &state.plot_y_columns.clone() {
            let y_vals = extract_f64_column(df, y_col);
            let data: Vec<[f64; 2]> = x_vals
                .iter()
                .zip(y_vals.iter())
                .map(|(&x, &y)| [x, y])
                .collect();
            if !data.is_empty() {
                state.plot_multi_data.push((y_col.clone(), data));
            }
        }
    }
}

/// Extract a column as Vec<f64>, casting to float. Non-numeric/null values are skipped.
fn extract_f64_column(df: &DataFrame, col_name: &str) -> Vec<f64> {
    if let Ok(series) = df.column(col_name) {
        if let Ok(casted) = series.cast(&DataType::Float64) {
            if let Ok(ca) = casted.f64() {
                return ca.into_no_null_iter().collect();
            }
        }
    }
    vec![]
}

/// Render histogram(s) for selected columns (supports multi-series overlay).
fn render_histogram(ui: &mut egui::Ui, state: &AppState, plot_height: f32) {
    // Use Y columns if available, else fall back to X
    let columns: Vec<String> = if !state.plot_y_columns.is_empty() {
        state.plot_y_columns.clone()
    } else if !state.plot_x.is_empty() {
        vec![state.plot_x.clone()]
    } else {
        ui.label("Select columns for the histogram (use Y series or X).");
        return;
    };

    // Use full dataset for histogram (fall back to preview if unavailable)
    let df = match state.full_df.as_ref().or(state.preview_df.as_ref()) {
        Some(df) => df,
        None => {
            ui.label("No data.");
            return;
        }
    };

    let mut all_series: Vec<(String, Vec<Bar>)> = Vec::new();
    let mut global_x_min = f64::INFINITY;
    let mut global_x_max = f64::NEG_INFINITY;
    let mut global_y_max = 0.0f64;

    for col_name in &columns {
        let values = extract_f64_column(df, col_name);
        if values.is_empty() {
            continue;
        }
        let (centers, counts, bin_width) = compute_histogram(&values, state.histogram_bins);
        if let (Some(&first), Some(&last)) = (centers.first(), centers.last()) {
            global_x_min = global_x_min.min(first - bin_width);
            global_x_max = global_x_max.max(last + bin_width);
        }
        let max_count = counts.iter().copied().fold(0.0f64, f64::max);
        global_y_max = global_y_max.max(max_count);
        let bars: Vec<Bar> = centers
            .iter()
            .zip(counts.iter())
            .map(|(&c, &count)| Bar::new(c, count).width(bin_width * 0.95))
            .collect();
        all_series.push((col_name.clone(), bars));
    }

    if all_series.is_empty() {
        ui.label("No numeric data in selected columns.");
        return;
    }

    let rc = state.plot_reset_counter;
    let mut plot = Plot::new(format!("histogram_{rc}"))
        .height(plot_height)
        .y_axis_label("Count")
        .show_axes([true, true])
        .show_grid([true, true])
        .legend(Legend::default());
    if global_x_min.is_finite() && global_x_max.is_finite() {
        plot = plot
            .include_x(global_x_min)
            .include_x(global_x_max)
            .include_y(0.0)
            .include_y(global_y_max * 1.05);
    }
    plot.show(ui, |plot_ui| {
        for (name, bars) in all_series {
            plot_ui.bar_chart(BarChart::new(name, bars));
        }
    });
}

// ─── Histogram Computation ────────────────────────────────────────────────────

/// Compute histogram bins: returns (bin_centers, counts, bin_width).
fn compute_histogram(values: &[f64], n_bins: usize) -> (Vec<f64>, Vec<f64>, f64) {
    if values.is_empty() || n_bins == 0 {
        return (vec![], vec![], 0.0);
    }

    let min = values.iter().copied().fold(f64::INFINITY, f64::min);
    let max = values.iter().copied().fold(f64::NEG_INFINITY, f64::max);

    if (max - min).abs() < f64::EPSILON {
        return (vec![min], vec![values.len() as f64], 1.0);
    }

    let bin_width = (max - min) / n_bins as f64;
    let mut counts = vec![0.0f64; n_bins];

    for &v in values {
        let bin = ((v - min) / bin_width).floor() as usize;
        let bin = bin.min(n_bins - 1); // clamp last value to last bin
        counts[bin] += 1.0;
    }

    let centers: Vec<f64> = (0..n_bins)
        .map(|i| min + (i as f64 + 0.5) * bin_width)
        .collect();

    (centers, counts, bin_width)
}
