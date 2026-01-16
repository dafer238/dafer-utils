use anyhow::Result;
use polars::prelude::*;

pub fn scan_csv(path: &str) -> Result<LazyFrame, PolarsError> {
    LazyCsvReader::new(PlPath::from_str(path))
        .with_has_header(true)
        .finish()
}

pub fn scan_parquet(path: &str) -> Result<LazyFrame, PolarsError> {
    LazyFrame::scan_parquet(PlPath::from_str(path), ScanArgsParquet::default())
}
