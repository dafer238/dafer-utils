//! Query Engine: builds a LazyFrame from a DataSource and a sequence of Operations.
//!
//! The engine never materializes data unless explicitly asked (preview/execute).
//! All transformations are applied lazily via Polars logical plan.

use anyhow::Result;
use polars::prelude::*;

use crate::datasource::DataSource;
use crate::operations::{FillNullStrategy, FilterOp, Operation};

/// Build a LazyFrame by scanning the source and applying all operations in order.
pub fn build_lazy(source: &DataSource, operations: &[Operation]) -> Result<LazyFrame> {
    let mut lf = source.scan().map_err(|e| anyhow::anyhow!("{}", e))?;
    for op in operations {
        lf = apply_operation(lf, op)?;
    }
    Ok(lf)
}

/// Collect a limited preview (N rows) from the full pipeline.
/// This is fast because .limit(N) is pushed down into the logical plan.
pub fn preview(source: &DataSource, operations: &[Operation], n: u32) -> Result<DataFrame> {
    let lf = build_lazy(source, operations)?;
    lf.limit(n).collect().map_err(|e| anyhow::anyhow!("{}", e))
}

/// Get schema information (column names + data types) from the pipeline
/// without collecting any data.
pub fn schema_info(source: &DataSource, operations: &[Operation]) -> Result<Vec<(String, String)>> {
    let mut lf = build_lazy(source, operations)?;
    let schema = lf.collect_schema().map_err(|e| anyhow::anyhow!("{}", e))?;
    Ok(schema
        .iter()
        .map(|(name, dtype)| (name.to_string(), format!("{}", dtype)))
        .collect())
}

/// Execute the full pipeline and collect all results into a DataFrame.
/// Use with caution for large datasets — prefer streaming export instead.
pub fn execute(source: &DataSource, operations: &[Operation]) -> Result<DataFrame> {
    let lf = build_lazy(source, operations)?;
    lf.collect().map_err(|e| anyhow::anyhow!("{}", e))
}

// ─── Operation Application ───────────────────────────────────────────────────

/// Apply a single Operation to a LazyFrame, returning the transformed LazyFrame.
fn apply_operation(lf: LazyFrame, op: &Operation) -> Result<LazyFrame> {
    match op {
        Operation::Filter {
            column,
            op: filter_op,
            value,
        } => {
            let expr = build_filter_expr(column, filter_op, value);
            Ok(lf.filter(expr))
        }

        Operation::Sort { column, descending } => Ok(lf.sort(
            [column.as_str()],
            SortMultipleOptions {
                descending: vec![*descending],
                ..Default::default()
            },
        )),

        Operation::DropColumn(col_name) => {
            // Select all columns except the one to drop
            let schema = lf
                .clone()
                .collect_schema()
                .map_err(|e| anyhow::anyhow!("{}", e))?;
            let exprs: Vec<Expr> = schema
                .iter_names()
                .filter(|name| name.as_str() != col_name.as_str())
                .map(|name| col(name.clone()))
                .collect();
            if exprs.is_empty() {
                anyhow::bail!("Cannot drop all columns");
            }
            Ok(lf.select(exprs))
        }

        Operation::RenameColumn { from, to } => Ok(lf.rename([from.as_str()], [to.as_str()], true)),

        Operation::SelectColumns(columns) => {
            let exprs: Vec<Expr> = columns.iter().map(|c| col(c.as_str())).collect();
            Ok(lf.select(exprs))
        }

        Operation::Limit(n) => Ok(lf.limit(*n)),

        Operation::FillNull {
            column,
            strategy,
            value,
        } => {
            let fill_expr = match strategy {
                FillNullStrategy::Forward => {
                    // Forward fill: replace nulls with previous non-null values
                    // Use shift-based approach as Expr::forward_fill may not be available
                    col(column.as_str()).fill_null(col(column.as_str()).shift(lit(1)))
                }
                FillNullStrategy::Backward => {
                    // Backward fill: replace nulls with next non-null values
                    col(column.as_str()).fill_null(col(column.as_str()).shift(lit(-1)))
                }
                FillNullStrategy::WithValue => {
                    let v = value.as_deref().unwrap_or("");
                    let lit_val = parse_literal(v);
                    col(column.as_str()).fill_null(lit_val)
                }
                FillNullStrategy::Mean => {
                    col(column.as_str()).fill_null(col(column.as_str()).mean())
                }
                FillNullStrategy::Min => col(column.as_str()).fill_null(col(column.as_str()).min()),
                FillNullStrategy::Max => col(column.as_str()).fill_null(col(column.as_str()).max()),
            };
            Ok(lf.with_columns([fill_expr]))
        }

        Operation::CastColumn { column, dtype } => {
            let target = dtype.to_polars();
            Ok(lf.with_columns([col(column.as_str()).cast(target)]))
        }

        Operation::ParseDatetime { column, format } => {
            // Cast String column to Datetime using the user-specified format.
            // Uses str().to_datetime() with non-strict parsing.
            let options = StrptimeOptions {
                format: Some(format.clone().into()),
                strict: false,
                exact: true,
                ..Default::default()
            };
            Ok(lf.with_columns([col(column.as_str()).str().to_datetime(
                Some(TimeUnit::Microseconds),
                None,
                options,
                lit("null"),
            )]))
        }
    }
}

// ─── Helpers ──────────────────────────────────────────────────────────────────

/// Build a Polars filter expression from a column name, operator, and value string.
fn build_filter_expr(column: &str, op: &FilterOp, value: &str) -> Expr {
    let c = col(column);

    match op {
        FilterOp::IsNull => return c.is_null(),
        FilterOp::IsNotNull => return c.is_not_null(),
        _ => {}
    }

    let lit_val = parse_literal(value);

    match op {
        FilterOp::Eq => c.eq(lit_val),
        FilterOp::Neq => c.neq(lit_val),
        FilterOp::Gt => c.gt(lit_val),
        FilterOp::Gte => c.gt_eq(lit_val),
        FilterOp::Lt => c.lt(lit_val),
        FilterOp::Lte => c.lt_eq(lit_val),
        FilterOp::Contains => c.str().contains(lit(value.to_string()), false),
        FilterOp::IsNull | FilterOp::IsNotNull => unreachable!(),
    }
}

/// Parse a string value into a Polars literal expression.
/// Tries integer → float → bool → string (in that order).
fn parse_literal(value: &str) -> Expr {
    if let Ok(n) = value.parse::<i64>() {
        lit(n)
    } else if let Ok(n) = value.parse::<f64>() {
        lit(n)
    } else if let Ok(b) = value.parse::<bool>() {
        lit(b)
    } else {
        lit(value.to_string())
    }
}
