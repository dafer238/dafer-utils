use std::path::PathBuf;

use polars::prelude::*;
use serde::{Deserialize, Serialize};

use crate::data_loader;

/// Supported data source types (CSV and Parquet for now).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum DataSourceType {
    Csv,
    Parquet,
}

impl std::fmt::Display for DataSourceType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DataSourceType::Csv => write!(f, "CSV"),
            DataSourceType::Parquet => write!(f, "Parquet"),
        }
    }
}

/// Represents a data source file with its type.
/// Immutable reference to the source — all transformations build on top of this.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataSource {
    pub path: PathBuf,
    pub source_type: DataSourceType,
    /// Auto-detected String columns that are actually numeric.
    /// Set after the first preview collection; applied during scan.
    #[serde(default)]
    pub auto_numeric_cols: Vec<String>,
}

impl DataSource {
    /// Create a DataSource from a file path, inferring the type from the extension.
    /// Returns `None` if the extension is not recognized.
    pub fn from_path(path: PathBuf) -> Option<Self> {
        let ext = path.extension()?.to_str()?.to_ascii_lowercase();
        let source_type = match ext.as_str() {
            "csv" | "tsv" => DataSourceType::Csv,
            "parquet" | "pq" => DataSourceType::Parquet,
            _ => return None,
        };
        Some(Self {
            path,
            source_type,
            auto_numeric_cols: Vec::new(),
        })
    }

    /// Scan the source as a LazyFrame (lazy evaluation — no data is loaded yet).
    /// Automatically applies numeric casts for detected numeric String columns.
    pub fn scan(&self) -> Result<LazyFrame, PolarsError> {
        let path_str = self.path.to_str().unwrap_or_default();
        let mut lf = match self.source_type {
            DataSourceType::Csv => data_loader::scan_csv(path_str)?,
            DataSourceType::Parquet => data_loader::scan_parquet(path_str)?,
        };
        // Auto-cast detected numeric String columns to Float64
        if !self.auto_numeric_cols.is_empty() {
            let exprs: Vec<Expr> = self
                .auto_numeric_cols
                .iter()
                .map(|name| col(name.as_str()).cast(DataType::Float64))
                .collect();
            lf = lf.with_columns(exprs);
        }
        Ok(lf)
    }
}
