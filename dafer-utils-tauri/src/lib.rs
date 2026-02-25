use std::path::PathBuf;
use std::sync::Mutex;

use polars::prelude::*;
use serde::{Deserialize, Serialize};
use tauri::State;
use tauri_plugin_dialog::DialogExt;

use dafer_utils::data_loader;
use dafer_utils::datasource::DataSource;
use dafer_utils::execution;
use dafer_utils::operations::{
    DTypeTag, FillNullStrategy as DaferFillNullStrategy, FilterOp, Operation,
};
use dafer_utils::persistence::PersistentState;
use dafer_utils::query_engine;

// ─── Application State ────────────────────────────────────────────────────────

pub struct AppState {
    pub source: Mutex<Option<DataSource>>,
    pub operations: Mutex<Vec<Operation>>,
    pub redo_stack: Mutex<Vec<Operation>>,
    pub preview_df: Mutex<Option<DataFrame>>,
    pub full_df: Mutex<Option<DataFrame>>,
    pub auto_cast_detected: Mutex<bool>,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            source: Mutex::new(None),
            operations: Mutex::new(Vec::new()),
            redo_stack: Mutex::new(Vec::new()),
            preview_df: Mutex::new(None),
            full_df: Mutex::new(None),
            auto_cast_detected: Mutex::new(false),
        }
    }
}

// ─── Serializable types for frontend communication ────────────────────────────

#[derive(Serialize, Deserialize, Clone)]
pub struct ColumnInfo {
    pub name: String,
    pub dtype: String,
}

#[derive(Serialize, Clone)]
pub struct ColumnStatInfo {
    pub name: String,
    pub dtype: String,
    pub min: Option<String>,
    pub max: Option<String>,
    pub null_count: usize,
    pub error_count: usize,
}

#[derive(Serialize)]
pub struct PreviewResult {
    pub headers: Vec<String>,
    pub dtypes: Vec<String>,
    pub rows: Vec<Vec<String>>,
    pub total_rows: usize,
    pub preview_rows: usize,
    pub stats: Vec<ColumnStatInfo>,
}

#[derive(Serialize)]
pub struct PlotData {
    pub series: Vec<PlotSeries>,
    pub x_is_datetime: bool,
}

#[derive(Serialize)]
pub struct PlotSeries {
    pub name: String,
    pub x: Vec<f64>,
    pub y: Vec<f64>,
}

#[derive(Serialize)]
pub struct HistogramData {
    pub series: Vec<HistogramSeries>,
}

#[derive(Serialize)]
pub struct HistogramSeries {
    pub name: String,
    pub values: Vec<f64>,
}

#[derive(Deserialize)]
pub struct OperationInput {
    pub op_type: String,
    // Filter
    pub column: Option<String>,
    pub filter_op: Option<String>,
    pub value: Option<String>,
    // Sort
    pub descending: Option<bool>,
    // Rename
    pub rename_from: Option<String>,
    pub rename_to: Option<String>,
    // Select
    pub columns: Option<Vec<String>>,
    // Limit
    pub limit: Option<u32>,
    // FillNull
    pub fill_strategy: Option<String>,
    pub fill_value: Option<String>,
    // Cast
    pub cast_dtype: Option<String>,
    // ParseDatetime
    pub datetime_format: Option<String>,
}

// ─── Helper functions ─────────────────────────────────────────────────────────

fn format_cell_value(v: &AnyValue) -> String {
    match v {
        AnyValue::Null => String::new(),
        AnyValue::Float64(f) if f.is_nan() => "NaN".to_string(),
        AnyValue::Float32(f) if f.is_nan() => "NaN".to_string(),
        AnyValue::String(s) => s.to_string(),
        AnyValue::StringOwned(s) => s.to_string(),
        other => {
            let s = other.to_string();
            if s.len() >= 2 && s.starts_with('"') && s.ends_with('"') {
                s[1..s.len() - 1].to_string()
            } else {
                s
            }
        }
    }
}

fn format_stat_float(s: &str) -> String {
    if s.contains('.') || s.contains('e') || s.contains('E') {
        if let Ok(f) = s.parse::<f64>() {
            return format!("{:.4}", f);
        }
    }
    s.to_string()
}

fn parse_operation(input: &OperationInput) -> Result<Operation, String> {
    match input.op_type.as_str() {
        "filter" => {
            let col = input.column.as_ref().ok_or("Missing column")?.clone();
            let op_str = input.filter_op.as_ref().ok_or("Missing filter_op")?;
            let filter_op = match op_str.as_str() {
                "=" | "eq" => FilterOp::Eq,
                "!=" | "neq" => FilterOp::Neq,
                ">" | "gt" => FilterOp::Gt,
                ">=" | "gte" => FilterOp::Gte,
                "<" | "lt" => FilterOp::Lt,
                "<=" | "lte" => FilterOp::Lte,
                "contains" => FilterOp::Contains,
                "is_null" => FilterOp::IsNull,
                "is_not_null" => FilterOp::IsNotNull,
                other => return Err(format!("Unknown filter op: {other}")),
            };
            let value = input.value.clone().unwrap_or_default();
            Ok(Operation::Filter {
                column: col,
                op: filter_op,
                value,
            })
        }
        "sort" => {
            let col = input.column.as_ref().ok_or("Missing column")?.clone();
            let desc = input.descending.unwrap_or(false);
            Ok(Operation::Sort {
                column: col,
                descending: desc,
            })
        }
        "drop_column" => {
            let col = input.column.as_ref().ok_or("Missing column")?.clone();
            Ok(Operation::DropColumn(col))
        }
        "rename_column" => {
            let from = input
                .rename_from
                .as_ref()
                .ok_or("Missing rename_from")?
                .clone();
            let to = input.rename_to.as_ref().ok_or("Missing rename_to")?.clone();
            Ok(Operation::RenameColumn { from, to })
        }
        "select_columns" => {
            let cols = input.columns.as_ref().ok_or("Missing columns")?.clone();
            Ok(Operation::SelectColumns(cols))
        }
        "limit" => {
            let n = input.limit.ok_or("Missing limit")?;
            Ok(Operation::Limit(n))
        }
        "fill_null" => {
            let col = input.column.as_ref().ok_or("Missing column")?.clone();
            let strat_str = input
                .fill_strategy
                .as_ref()
                .ok_or("Missing fill_strategy")?;
            let strategy = match strat_str.as_str() {
                "forward" => DaferFillNullStrategy::Forward,
                "backward" => DaferFillNullStrategy::Backward,
                "with_value" => DaferFillNullStrategy::WithValue,
                "mean" => DaferFillNullStrategy::Mean,
                "min" => DaferFillNullStrategy::Min,
                "max" => DaferFillNullStrategy::Max,
                other => return Err(format!("Unknown fill strategy: {other}")),
            };
            let value = if strategy.needs_value() {
                input.fill_value.clone()
            } else {
                None
            };
            Ok(Operation::FillNull {
                column: col,
                strategy,
                value,
            })
        }
        "cast_column" => {
            let col = input.column.as_ref().ok_or("Missing column")?.clone();
            let dtype_str = input.cast_dtype.as_ref().ok_or("Missing cast_dtype")?;
            let dtype = match dtype_str.as_str() {
                "Int32" => DTypeTag::Int32,
                "Int64" => DTypeTag::Int64,
                "Float32" => DTypeTag::Float32,
                "Float64" => DTypeTag::Float64,
                "String" => DTypeTag::Utf8String,
                "Boolean" => DTypeTag::Boolean,
                "Date" => DTypeTag::Date,
                other => return Err(format!("Unknown dtype: {other}")),
            };
            Ok(Operation::CastColumn { column: col, dtype })
        }
        "parse_datetime" => {
            let col = input.column.as_ref().ok_or("Missing column")?.clone();
            let fmt = input
                .datetime_format
                .clone()
                .unwrap_or_else(|| "%Y-%m-%d %H:%M:%S".to_string());
            Ok(Operation::ParseDatetime {
                column: col,
                format: fmt,
            })
        }
        other => Err(format!("Unknown operation type: {other}")),
    }
}

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

// ─── Tauri Commands ───────────────────────────────────────────────────────────

#[tauri::command]
fn open_file(path: String, state: State<AppState>) -> Result<String, String> {
    let pb = PathBuf::from(&path);
    let ds = DataSource::from_path(pb).ok_or_else(|| format!("Unsupported file: {path}"))?;
    *state.source.lock().unwrap() = Some(ds);
    *state.operations.lock().unwrap() = Vec::new();
    *state.redo_stack.lock().unwrap() = Vec::new();
    *state.auto_cast_detected.lock().unwrap() = false;
    *state.preview_df.lock().unwrap() = None;
    *state.full_df.lock().unwrap() = None;
    Ok(format!("Loaded: {path}"))
}

#[tauri::command]
fn get_preview(state: State<AppState>) -> Result<PreviewResult, String> {
    let source_guard = state.source.lock().unwrap();
    let source = source_guard.as_ref().ok_or("No file loaded")?;
    let ops = state.operations.lock().unwrap().clone();

    // Preview (200 rows)
    let df = query_engine::preview(source, &ops, 200).map_err(|e| format!("Preview error: {e}"))?;

    // Auto-detect numeric string columns on first load
    {
        let mut detected = state.auto_cast_detected.lock().unwrap();
        if !*detected {
            let numeric_cols = data_loader::detect_numeric_string_columns(&df);
            if !numeric_cols.is_empty() {
                drop(source_guard);
                let mut src_guard = state.source.lock().unwrap();
                if let Some(ref mut src) = *src_guard {
                    src.auto_numeric_cols = numeric_cols;
                }
                *detected = true;
                drop(src_guard);
                // Re-run preview with casts applied
                let source_guard2 = state.source.lock().unwrap();
                let source2 = source_guard2.as_ref().ok_or("No file loaded")?;
                let df2 = query_engine::preview(source2, &ops, 200)
                    .map_err(|e| format!("Preview error: {e}"))?;
                return build_preview_result(source2, &ops, df2, &state);
            }
            *detected = true;
        }
    }

    build_preview_result(source, &ops, df, &state)
}

fn build_preview_result(
    source: &DataSource,
    ops: &[Operation],
    df: DataFrame,
    state: &State<AppState>,
) -> Result<PreviewResult, String> {
    let headers: Vec<String> = df
        .get_column_names()
        .iter()
        .map(|s| s.to_string())
        .collect();
    let dtypes: Vec<String> = df.dtypes().iter().map(|d| format!("{d}")).collect();
    let preview_rows = df.height();

    // Build row strings
    let columns = df.get_columns();
    let n_rows = df.height();
    let n_cols = df.width();
    let mut rows: Vec<Vec<String>> = Vec::with_capacity(n_rows);
    for row_idx in 0..n_rows {
        let mut row_strs = Vec::with_capacity(n_cols);
        for col_s in columns {
            let val: String = col_s
                .get(row_idx)
                .map(|v| format_cell_value(&v))
                .unwrap_or_default();
            row_strs.push(val);
        }
        rows.push(row_strs);
    }

    // Full dataset for stats
    let (stats, total_rows) = match query_engine::execute(source, ops) {
        Ok(full) => {
            let stats: Vec<ColumnStatInfo> = data_loader::column_stats(&full)
                .into_iter()
                .map(|s| ColumnStatInfo {
                    name: s.name,
                    dtype: s.dtype,
                    min: s.min.map(|v| format_stat_float(&v)),
                    max: s.max.map(|v| format_stat_float(&v)),
                    null_count: s.null_count,
                    error_count: s.error_count,
                })
                .collect();
            let total = full.height();
            *state.full_df.lock().unwrap() = Some(full);
            (stats, total)
        }
        Err(_) => {
            let stats: Vec<ColumnStatInfo> = data_loader::column_stats(&df)
                .into_iter()
                .map(|s| ColumnStatInfo {
                    name: s.name,
                    dtype: s.dtype,
                    min: s.min.map(|v| format_stat_float(&v)),
                    max: s.max.map(|v| format_stat_float(&v)),
                    null_count: s.null_count,
                    error_count: s.error_count,
                })
                .collect();
            *state.full_df.lock().unwrap() = None;
            (stats, preview_rows)
        }
    };

    *state.preview_df.lock().unwrap() = Some(df);

    Ok(PreviewResult {
        headers,
        dtypes,
        rows,
        total_rows,
        preview_rows,
        stats,
    })
}

#[tauri::command]
fn add_operation(input: OperationInput, state: State<AppState>) -> Result<String, String> {
    let op = parse_operation(&input)?;
    let desc = format!("{op}");
    state.operations.lock().unwrap().push(op);
    state.redo_stack.lock().unwrap().clear();
    Ok(desc)
}

#[tauri::command]
fn remove_operation(index: usize, state: State<AppState>) -> Result<(), String> {
    let mut ops = state.operations.lock().unwrap();
    if index < ops.len() {
        ops.remove(index);
        state.redo_stack.lock().unwrap().clear();
        Ok(())
    } else {
        Err("Invalid operation index".into())
    }
}

#[tauri::command]
fn undo_operation(state: State<AppState>) -> Result<Option<String>, String> {
    let mut ops = state.operations.lock().unwrap();
    if let Some(op) = ops.pop() {
        let desc = format!("{op}");
        state.redo_stack.lock().unwrap().push(op);
        Ok(Some(desc))
    } else {
        Ok(None)
    }
}

#[tauri::command]
fn redo_operation(state: State<AppState>) -> Result<Option<String>, String> {
    let mut redo = state.redo_stack.lock().unwrap();
    if let Some(op) = redo.pop() {
        let desc = format!("{op}");
        state.operations.lock().unwrap().push(op);
        Ok(Some(desc))
    } else {
        Ok(None)
    }
}

#[tauri::command]
fn clear_pipeline(state: State<AppState>) -> Result<(), String> {
    state.operations.lock().unwrap().clear();
    state.redo_stack.lock().unwrap().clear();
    Ok(())
}

#[tauri::command]
fn get_operations(state: State<AppState>) -> Vec<String> {
    state
        .operations
        .lock()
        .unwrap()
        .iter()
        .map(|op| format!("{op}"))
        .collect()
}

#[tauri::command]
fn get_plot_data(
    x_col: String,
    y_cols: Vec<String>,
    state: State<AppState>,
) -> Result<PlotData, String> {
    let full_guard = state.full_df.lock().unwrap();
    let preview_guard = state.preview_df.lock().unwrap();
    let df = full_guard
        .as_ref()
        .or(preview_guard.as_ref())
        .ok_or("No data loaded")?;

    let mut x_is_datetime = false;

    // Extract X values
    let x_vals: Vec<f64> = if let Ok(series) = df.column(&x_col) {
        match series.dtype() {
            DataType::Datetime(tu, _) => {
                x_is_datetime = true;
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
                x_is_datetime = true;
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
            _ => extract_f64_column(df, &x_col),
        }
    } else {
        vec![]
    };

    let mut series_list = Vec::new();
    for y_col in &y_cols {
        let y_vals = extract_f64_column(df, y_col);
        let len = x_vals.len().min(y_vals.len());
        if len > 0 {
            series_list.push(PlotSeries {
                name: y_col.clone(),
                x: x_vals[..len].to_vec(),
                y: y_vals[..len].to_vec(),
            });
        }
    }

    Ok(PlotData {
        series: series_list,
        x_is_datetime,
    })
}

#[tauri::command]
fn get_histogram_data(
    columns: Vec<String>,
    state: State<AppState>,
) -> Result<HistogramData, String> {
    let full_guard = state.full_df.lock().unwrap();
    let preview_guard = state.preview_df.lock().unwrap();
    let df = full_guard
        .as_ref()
        .or(preview_guard.as_ref())
        .ok_or("No data loaded")?;

    let mut series = Vec::new();
    for col_name in &columns {
        let values = extract_f64_column(df, col_name);
        if !values.is_empty() {
            series.push(HistogramSeries {
                name: col_name.clone(),
                values,
            });
        }
    }

    Ok(HistogramData { series })
}

#[tauri::command]
fn export_data(path: String, format: String, state: State<AppState>) -> Result<String, String> {
    let source_guard = state.source.lock().unwrap();
    let source = source_guard.as_ref().ok_or("No file loaded")?;
    let ops = state.operations.lock().unwrap().clone();
    let pb = PathBuf::from(&path);

    match format.as_str() {
        "csv" => {
            execution::export_csv(source, &ops, &pb).map_err(|e| format!("Export error: {e}"))?
        }
        "parquet" => execution::export_parquet(source, &ops, &pb)
            .map_err(|e| format!("Export error: {e}"))?,
        other => return Err(format!("Unknown export format: {other}")),
    }

    Ok(format!("Exported to {path}"))
}

#[tauri::command]
fn save_state(path: String, state: State<AppState>) -> Result<String, String> {
    let source = state.source.lock().unwrap().clone();
    let operations = state.operations.lock().unwrap().clone();
    let persistent = PersistentState { source, operations };
    persistent
        .save(&PathBuf::from(&path))
        .map_err(|e| format!("Save error: {e}"))?;
    Ok("State saved".to_string())
}

#[tauri::command]
fn load_state(path: String, state: State<AppState>) -> Result<String, String> {
    let persistent =
        PersistentState::load(&PathBuf::from(&path)).map_err(|e| format!("Load error: {e}"))?;
    *state.source.lock().unwrap() = persistent.source;
    *state.operations.lock().unwrap() = persistent.operations;
    state.redo_stack.lock().unwrap().clear();
    *state.auto_cast_detected.lock().unwrap() = false;
    *state.preview_df.lock().unwrap() = None;
    *state.full_df.lock().unwrap() = None;
    Ok("State loaded".to_string())
}

#[tauri::command]
fn get_file_metadata(state: State<AppState>) -> Result<serde_json::Value, String> {
    let source_guard = state.source.lock().unwrap();
    let source = source_guard.as_ref().ok_or("No file loaded")?;
    let path = &source.path;
    let meta = std::fs::metadata(path).map_err(|e| format!("{e}"))?;
    let size = meta.len();
    let size_str = if size > 1_000_000_000 {
        format!("{:.2} GB", size as f64 / 1_000_000_000.0)
    } else if size > 1_000_000 {
        format!("{:.2} MB", size as f64 / 1_000_000.0)
    } else if size > 1_000 {
        format!("{:.1} KB", size as f64 / 1_000.0)
    } else {
        format!("{size} bytes")
    };

    Ok(serde_json::json!({
        "path": path.display().to_string(),
        "size": size_str,
        "source_type": format!("{}", source.source_type),
    }))
}
// ─── Dialog Helpers ────────────────────────────────────────────────

#[tauri::command]
fn pick_data_file(app: tauri::AppHandle) -> Option<String> {
    app.dialog()
        .file()
        .add_filter("Data Files", &["csv", "tsv", "parquet", "pq"])
        .blocking_pick_file()
        .map(|p| p.to_string())
}

#[tauri::command]
fn pick_save_path(app: tauri::AppHandle, ext: String) -> Option<String> {
    let ext_ref = ext.as_str();
    app.dialog()
        .file()
        .add_filter("File", &[ext_ref])
        .blocking_save_file()
        .map(|p| p.to_string())
}
// ─── App Entry Point ──────────────────────────────────────────────────────────

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .manage(AppState::default())
        .invoke_handler(tauri::generate_handler![
            open_file,
            get_preview,
            add_operation,
            remove_operation,
            undo_operation,
            redo_operation,
            clear_pipeline,
            get_operations,
            get_plot_data,
            get_histogram_data,
            export_data,
            save_state,
            load_state,
            get_file_metadata,
            pick_data_file,
            pick_save_path,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
