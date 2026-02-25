use anyhow::Result;
use polars::prelude::*;

/// Scan a CSV file as a LazyFrame.
/// Uses a high schema inference length to correctly detect numeric columns
/// even when values are quoted (e.g. "2.124879").
pub fn scan_csv(path: &str) -> Result<LazyFrame, PolarsError> {
    LazyCsvReader::new(PlPath::from_str(path))
        .with_has_header(true)
        .with_infer_schema_length(Some(10000))
        .finish()
}

/// Scan a Parquet file as a LazyFrame.
pub fn scan_parquet(path: &str) -> Result<LazyFrame, PolarsError> {
    LazyFrame::scan_parquet(PlPath::from_str(path), ScanArgsParquet::default())
}

/// Collect the first n rows from a LazyFrame into a DataFrame.
pub fn collect_head(lf: &LazyFrame, n: usize) -> Result<DataFrame, PolarsError> {
    lf.clone().limit(n as u32).collect()
}

/// Get column statistics: min, max, null count, error count (for numeric columns).
pub fn column_stats(df: &DataFrame) -> Vec<ColumnStats> {
    df.iter()
        .map(|series| {
            let dtype = series.dtype();
            let null_count = series.null_count();
            let name = series.name().to_string();
            let (min, max, error_count) = match dtype {
                DataType::Int64 => (
                    series.min::<i64>().ok().flatten().map(|v| v.to_string()),
                    series.max::<i64>().ok().flatten().map(|v| v.to_string()),
                    0,
                ),
                DataType::Int32 => (
                    series.min::<i32>().ok().flatten().map(|v| v.to_string()),
                    series.max::<i32>().ok().flatten().map(|v| v.to_string()),
                    0,
                ),
                DataType::Float64 => (
                    series.min::<f64>().ok().flatten().map(|v| v.to_string()),
                    series.max::<f64>().ok().flatten().map(|v| v.to_string()),
                    0,
                ),
                DataType::Float32 => (
                    series.min::<f32>().ok().flatten().map(|v| v.to_string()),
                    series.max::<f32>().ok().flatten().map(|v| v.to_string()),
                    0,
                ),
                _ => (None, None, 0),
            };
            ColumnStats {
                name,
                dtype: format!("{:?}", dtype),
                min,
                max,
                null_count,
                error_count,
            }
        })
        .collect()
}

/// Sort a DataFrame by a column (ascending/descending).
pub fn sort_df(df: &DataFrame, column: &str, descending: bool) -> Result<DataFrame, PolarsError> {
    use polars::prelude::SortMultipleOptions;
    df.sort(
        &[column.to_string()],
        SortMultipleOptions {
            descending: vec![descending],
            ..Default::default()
        },
    )
}

/// Struct for holding column statistics.
#[derive(Debug, Clone)]
pub struct ColumnStats {
    pub name: String,
    pub dtype: String,
    pub min: Option<String>,
    pub max: Option<String>,
    pub null_count: usize,
    pub error_count: usize,
}

// ─── Auto-detection of numeric String columns ────────────────────────────────

/// Detect String columns that contain primarily numeric values.
/// Returns column names that should be auto-cast to Float64.
pub fn detect_numeric_string_columns(df: &DataFrame) -> Vec<String> {
    df.get_columns()
        .iter()
        .filter(|s| s.dtype() == &DataType::String)
        .filter(|s| is_numeric_string_column(s))
        .map(|s| s.name().to_string())
        .collect()
}

/// Check if a String column contains primarily numeric values by sampling.
fn is_numeric_string_column(col: &Column) -> bool {
    let series = col.as_materialized_series();
    let Ok(str_ca) = series.str() else {
        return false;
    };
    let sample_size = series.len().min(200);
    let mut numeric_count = 0usize;
    let mut non_null_count = 0usize;
    for i in 0..sample_size {
        if let Some(val) = str_ca.get(i) {
            non_null_count += 1;
            let trimmed = val.trim();
            if !trimmed.is_empty() && trimmed.parse::<f64>().is_ok() {
                numeric_count += 1;
            }
        }
    }
    non_null_count > 0 && numeric_count * 10 >= non_null_count * 9
}
