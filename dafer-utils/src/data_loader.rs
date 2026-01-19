use anyhow::Result;
use polars::prelude::*;

/// Scan a CSV file as a LazyFrame.
pub fn scan_csv(path: &str) -> Result<LazyFrame, PolarsError> {
    LazyCsvReader::new(PlPath::from_str(path))
        .with_has_header(true)
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
