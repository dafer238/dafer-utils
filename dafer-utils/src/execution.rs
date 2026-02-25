//! Execution module: handles exporting pipeline results to files.
//!
//! Exports collect the full pipeline (no row limit) and write to disk.
//! For very large datasets, consider streaming exports (sink_parquet/sink_csv)
//! which can be added as a future optimization.

use std::path::Path;

use anyhow::Result;
use polars::prelude::*;

use crate::datasource::DataSource;
use crate::operations::Operation;
use crate::query_engine;

/// Export the full pipeline result as a CSV file.
/// Uses the `csv` crate for writing to avoid requiring extra Polars feature flags.
pub fn export_csv(source: &DataSource, operations: &[Operation], path: &Path) -> Result<()> {
    let df = query_engine::execute(source, operations)?;
    let file = std::fs::File::create(path)?;
    let mut writer = csv::Writer::from_writer(file);

    // Write header
    let headers: Vec<String> = df
        .get_column_names()
        .iter()
        .map(|s| s.to_string())
        .collect();
    writer.write_record(&headers)?;

    // Write rows
    for i in 0..df.height() {
        let row: Vec<String> = df
            .get_columns()
            .iter()
            .map(|col| col.get(i).map(|v| format_any_value(&v)).unwrap_or_default())
            .collect();
        writer.write_record(&row)?;
    }

    writer.flush()?;
    Ok(())
}

/// Export the full pipeline result as a Parquet file.
/// Uses Polars' built-in ParquetWriter (columnar, compressed, schema-preserving).
pub fn export_parquet(source: &DataSource, operations: &[Operation], path: &Path) -> Result<()> {
    let mut df = query_engine::execute(source, operations)?;
    let file = std::fs::File::create(path)?;
    ParquetWriter::new(file)
        .finish(&mut df)
        .map_err(|e| anyhow::anyhow!("{}", e))?;
    Ok(())
}

/// Format an AnyValue for CSV output.
/// Null values become empty strings (standard CSV convention).
fn format_any_value(v: &AnyValue) -> String {
    match v {
        AnyValue::Null => String::new(),
        other => other.to_string(),
    }
}
