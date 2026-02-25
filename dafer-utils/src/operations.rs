use serde::{Deserialize, Serialize};
use std::fmt;

use polars::prelude::DataType;

// ─── Filter Operator ──────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum FilterOp {
    Eq,
    Neq,
    Gt,
    Gte,
    Lt,
    Lte,
    Contains,
    IsNull,
    IsNotNull,
}

impl FilterOp {
    pub fn all() -> &'static [FilterOp] {
        &[
            FilterOp::Eq,
            FilterOp::Neq,
            FilterOp::Gt,
            FilterOp::Gte,
            FilterOp::Lt,
            FilterOp::Lte,
            FilterOp::Contains,
            FilterOp::IsNull,
            FilterOp::IsNotNull,
        ]
    }

    /// Returns true if this operator requires a value input.
    pub fn needs_value(&self) -> bool {
        !matches!(self, FilterOp::IsNull | FilterOp::IsNotNull)
    }
}

impl fmt::Display for FilterOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FilterOp::Eq => write!(f, "="),
            FilterOp::Neq => write!(f, "≠"),
            FilterOp::Gt => write!(f, ">"),
            FilterOp::Gte => write!(f, "≥"),
            FilterOp::Lt => write!(f, "<"),
            FilterOp::Lte => write!(f, "≤"),
            FilterOp::Contains => write!(f, "contains"),
            FilterOp::IsNull => write!(f, "is null"),
            FilterOp::IsNotNull => write!(f, "is not null"),
        }
    }
}

impl Default for FilterOp {
    fn default() -> Self {
        FilterOp::Eq
    }
}

// ─── Fill Null Strategy ───────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum FillNullStrategy {
    Forward,
    Backward,
    WithValue,
    Mean,
    Min,
    Max,
}

impl FillNullStrategy {
    pub fn all() -> &'static [FillNullStrategy] {
        &[
            FillNullStrategy::Forward,
            FillNullStrategy::Backward,
            FillNullStrategy::WithValue,
            FillNullStrategy::Mean,
            FillNullStrategy::Min,
            FillNullStrategy::Max,
        ]
    }

    pub fn needs_value(&self) -> bool {
        matches!(self, FillNullStrategy::WithValue)
    }
}

impl fmt::Display for FillNullStrategy {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FillNullStrategy::Forward => write!(f, "Forward Fill"),
            FillNullStrategy::Backward => write!(f, "Backward Fill"),
            FillNullStrategy::WithValue => write!(f, "With Value"),
            FillNullStrategy::Mean => write!(f, "Mean"),
            FillNullStrategy::Min => write!(f, "Min"),
            FillNullStrategy::Max => write!(f, "Max"),
        }
    }
}

impl Default for FillNullStrategy {
    fn default() -> Self {
        FillNullStrategy::Forward
    }
}

// ─── Data Type Tag (serializable representation of Polars DataType) ───────────

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum DTypeTag {
    Int32,
    Int64,
    Float32,
    Float64,
    Utf8String,
    Boolean,
    Date,
}

impl DTypeTag {
    pub fn to_polars(&self) -> DataType {
        match self {
            DTypeTag::Int32 => DataType::Int32,
            DTypeTag::Int64 => DataType::Int64,
            DTypeTag::Float32 => DataType::Float32,
            DTypeTag::Float64 => DataType::Float64,
            DTypeTag::Utf8String => DataType::String,
            DTypeTag::Boolean => DataType::Boolean,
            DTypeTag::Date => DataType::Date,
        }
    }

    pub fn all() -> &'static [DTypeTag] {
        &[
            DTypeTag::Int32,
            DTypeTag::Int64,
            DTypeTag::Float32,
            DTypeTag::Float64,
            DTypeTag::Utf8String,
            DTypeTag::Boolean,
            DTypeTag::Date,
        ]
    }
}

impl fmt::Display for DTypeTag {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DTypeTag::Int32 => write!(f, "Int32"),
            DTypeTag::Int64 => write!(f, "Int64"),
            DTypeTag::Float32 => write!(f, "Float32"),
            DTypeTag::Float64 => write!(f, "Float64"),
            DTypeTag::Utf8String => write!(f, "String"),
            DTypeTag::Boolean => write!(f, "Boolean"),
            DTypeTag::Date => write!(f, "Date"),
        }
    }
}

impl Default for DTypeTag {
    fn default() -> Self {
        DTypeTag::Float64
    }
}

// ─── Operation ────────────────────────────────────────────────────────────────
//
// Operations are serializable descriptions of transformations.
// The query engine converts these to Polars lazy expressions.
// No Polars Expr is stored directly — this ensures serializability and undo/redo.

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Operation {
    Filter {
        column: String,
        op: FilterOp,
        value: String,
    },
    Sort {
        column: String,
        descending: bool,
    },
    DropColumn(String),
    RenameColumn {
        from: String,
        to: String,
    },
    SelectColumns(Vec<String>),
    Limit(u32),
    FillNull {
        column: String,
        strategy: FillNullStrategy,
        value: Option<String>,
    },
    CastColumn {
        column: String,
        dtype: DTypeTag,
    },
    ParseDatetime {
        column: String,
        format: String,
    },
}

impl fmt::Display for Operation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Operation::Filter { column, op, value } => {
                if op.needs_value() {
                    write!(f, "Filter: {} {} {}", column, op, value)
                } else {
                    write!(f, "Filter: {} {}", column, op)
                }
            }
            Operation::Sort { column, descending } => {
                write!(
                    f,
                    "Sort: {} {}",
                    column,
                    if *descending { "DESC" } else { "ASC" }
                )
            }
            Operation::DropColumn(col) => write!(f, "Drop: {}", col),
            Operation::RenameColumn { from, to } => write!(f, "Rename: {} → {}", from, to),
            Operation::SelectColumns(cols) => write!(f, "Select: {}", cols.join(", ")),
            Operation::Limit(n) => write!(f, "Limit: {}", n),
            Operation::FillNull {
                column, strategy, ..
            } => {
                write!(f, "FillNull: {} ({})", column, strategy)
            }
            Operation::CastColumn { column, dtype } => {
                write!(f, "Cast: {} → {}", column, dtype)
            }
            Operation::ParseDatetime { column, format } => {
                write!(f, "ParseDatetime: {} ({})", column, format)
            }
        }
    }
}

// ─── Operation Type (for UI selection) ────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum OperationType {
    #[default]
    Filter,
    Sort,
    DropColumn,
    RenameColumn,
    SelectColumns,
    Limit,
    FillNull,
    CastColumn,
    ParseDatetime,
}

impl OperationType {
    pub fn all() -> &'static [OperationType] {
        &[
            OperationType::Filter,
            OperationType::Sort,
            OperationType::DropColumn,
            OperationType::RenameColumn,
            OperationType::SelectColumns,
            OperationType::Limit,
            OperationType::FillNull,
            OperationType::CastColumn,
            OperationType::ParseDatetime,
        ]
    }
}

impl fmt::Display for OperationType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            OperationType::Filter => write!(f, "Filter"),
            OperationType::Sort => write!(f, "Sort"),
            OperationType::DropColumn => write!(f, "Drop Column"),
            OperationType::RenameColumn => write!(f, "Rename Column"),
            OperationType::SelectColumns => write!(f, "Select Columns"),
            OperationType::Limit => write!(f, "Limit Rows"),
            OperationType::FillNull => write!(f, "Fill Null"),
            OperationType::CastColumn => write!(f, "Cast Column Type"),
            OperationType::ParseDatetime => write!(f, "Parse Datetime"),
        }
    }
}
