use anyhow::Result;
use polars::prelude::*;

pub fn load_csv(path: &str) -> Result<LazyFrame> {
    Ok(LazyCsvReader::new(PlPath::from_str(path))
        .with_has_header(true)
        .finish()?)
}
