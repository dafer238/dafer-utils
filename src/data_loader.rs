use anyhow::Result;
use polars::prelude::*;

pub fn load_csv(path: &str) -> Result<LazyFrame> {
    Ok(LazyCsvReader::new(path).with_has_header(true).finish()?)
}
