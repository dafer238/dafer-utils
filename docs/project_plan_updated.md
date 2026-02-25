Roust Data Science Desktop Application
High-Performance Data Viewer & Editor
1. Project Goal

Build a fully Rust-based desktop data science application optimized for:

Maximum performance (speed + memory efficiency)

Lazy evaluation using Polars

Large dataset handling (millions to billions of rows)

Interactive data editing via transformation graph

Exporting and plotting results

Persistent state management

This is not a spreadsheet clone.
It is a query engine with a GUI layer.

2. Core Design Philosophy

All data sources are treated as queryable lazy sources, not eagerly loaded tables.

Architecture model:

DataSource → LazyFrame → Logical Plan → Preview / Execution

All transformations generate new lazy plans.
No in-place row mutation.

The UI edits transformations, not data.

3. Core Technology Stack
Backend

polars (lazy + streaming execution)

arrow (columnar memory model)

rayon (parallelism, already used by Polars)

calamine (Excel reading)

duckdb (optional SQL federated execution)

serde + rkyv or bincode (state serialization)

Frontend

egui

eframe

egui_extras::TableBuilder (virtualized table rendering)

Plotting

egui_plot (interactive lightweight plotting)

plotters (optional advanced static plotting)

4. High-Level Architecture
/core
  datasource.rs
  query_engine.rs
  transformations.rs
  state.rs
  execution.rs

/ui
  app.rs
  table_view.rs
  plot_view.rs
  sidebar.rs
  history.rs

/cache
  parquet_cache.rs

Separation of concerns:

UI never accesses raw DataFrames

UI interacts only with QueryEngine

QueryEngine owns LazyFrame and transformations

5. Data Loading Strategy
Supported Formats

CSV

TSV

Parquet

IPC / Arrow

NDJSON

Excel (converted internally)

SQL databases

Rules

Always prefer lazy scanning:

LazyCsvReader

scan_parquet

scan_ipc

scan_ndjson

Never eagerly load large datasets

Enable streaming mode when collecting

Limit schema inference row count

Allow manual schema override

Excel Strategy

Load with calamine

Convert to Arrow batches

Cache as Parquet for reuse

SQL Strategy

Stream query results in batches

Convert to Arrow RecordBatch

Wrap as LazyFrame

6. Memory Efficiency Rules

Never:

Clone DataFrames unnecessarily

Convert entire frames to Vec<Vec<String>>

Materialize full dataset for preview

Always:

Use columnar Arrow memory

Limit preview rows

Render only visible rows/columns

Use streaming collect for large execution

7. Query Engine Design
App State
struct AppState {
    source: DataSource,
    operations: Vec<Operation>,
    preview_cache: Option<DataFrame>,
}
Operation Model
enum Operation {
    Filter(Expression),
    WithColumn(Expression),
    DropColumn(String),
    Rename { from: String, to: String },
    Sort { column: String, descending: bool },
}

LazyFrame is rebuilt from:

source → apply operations in order → LazyFrame
8. Editing Model

User edits do NOT mutate data.

Editing generates transformations:

Example (cell edit):

with_column(
    when(col("id").eq(lit(row_id)))
    .then(lit(new_value))
    .otherwise(col("column"))
)

Benefits:

Immutability

Undo/redo support

Deterministic behavior

No memory duplication

9. Preview System

Preview strategy:

Use .limit(N) (e.g., 1000 rows)

Recompute only when logical plan changes

Cache preview by logical plan hash

Execute in background worker thread

Send result to UI via channel

Never block UI thread.

10. Execution Modes
Preview Mode

Limited rows

Fast feedback

Streaming enabled

Full Execution Mode

Streaming collect

Export to:

Parquet

CSV

Excel

Or send to plot pipeline

Use:

.collect(streaming = true)

Or:

.sink_parquet(path)
11. Table Rendering (Critical Performance Area)

Must implement virtualization.

Render only:

visible_row_range
visible_column_range

Never:

Pre-render entire dataset

Pre-convert all values to strings

Instead:

Access Series directly

Format values on-demand

Use egui_extras::TableBuilder

12. Handling Large Datasets

If dataset exceeds RAM:

Use streaming execution

Push filters before collect

Avoid full materialization

Encourage Parquet caching

Memory-map when possible

Optional:

Auto-offer conversion to Parquet cache for CSV

13. Plotting Strategy

Never plot raw large datasets.

Before plotting:

Aggregate

Downsample

Bin

Use quantiles or grouping

Plot only processed data.

Avoid rendering millions of points.

14. State Persistence

Persist:

Data source configuration

Operation list

UI state

Cached metadata

Schema

Use:

rkyv (zero-copy deserialization preferred)

or bincode

Never serialize entire DataFrames unless explicitly exporting.

15. Undo / Redo

Maintain operation history stack:

Vec<Operation>

Undo:

Remove last operation

Rebuild LazyFrame

Redo:

Reapply operation

Do not snapshot full DataFrames.

16. Performance Best Practices

Let Polars manage threading

Avoid excessive async

Use background worker thread for execution

Cache schema and column statistics

Hash logical plan to avoid unnecessary recompute

Avoid string-heavy transformations

Avoid cloning Series

17. Internal Normalization Format

Internally normalize data to:

Arrow

Parquet

Parquet is the preferred cache format:

Columnar

Compressed

Schema-preserving

Fast reload

18. Advanced Optimization (Optional Phase)

Logical plan diffing

Column statistics caching (min, max, null count)

Approximate distinct counts (HyperLogLog)

Query plan visualization

Incremental recomputation

19. Non-Goals

Spreadsheet-level formula engine

Row-by-row mutation model

Web-based UI

Python bindings

20. Core Principle Summary

This application is:

A lazy analytical engine

A transformation graph editor

A virtualized data viewport

A high-performance desktop tool

It is NOT:

A spreadsheet clone

An in-memory data blob renderer

Everything must remain:

Columnar

Lazy

Streaming

Immutable

Parallelust Data Science Desktop Application
High-Performance Data Viewer & Editor
1. Project Goal

Build a fully Rust-based desktop data science application optimized for:

Maximum performance (speed + memory efficiency)

Lazy evaluation using Polars

Large dataset handling (millions to billions of rows)

Interactive data editing via transformation graph

Exporting and plotting results

Persistent state management

This is not a spreadsheet clone.
It is a query engine with a GUI layer.

2. Core Design Philosophy

All data sources are treated as queryable lazy sources, not eagerly loaded tables.

Architecture model:

DataSource → LazyFrame → Logical Plan → Preview / Execution

All transformations generate new lazy plans.
No in-place row mutation.

The UI edits transformations, not data.

3. Core Technology Stack
Backend

polars (lazy + streaming execution)

arrow (columnar memory model)

rayon (parallelism, already used by Polars)

calamine (Excel reading)

duckdb (optional SQL federated execution)

serde + rkyv or bincode (state serialization)

Frontend

egui

eframe

egui_extras::TableBuilder (virtualized table rendering)

Plotting

egui_plot (interactive lightweight plotting)

plotters (optional advanced static plotting)

4. High-Level Architecture
/core
  datasource.rs
  query_engine.rs
  transformations.rs
  state.rs
  execution.rs

/ui
  app.rs
  table_view.rs
  plot_view.rs
  sidebar.rs
  history.rs

/cache
  parquet_cache.rs

Separation of concerns:

UI never accesses raw DataFrames

UI interacts only with QueryEngine

QueryEngine owns LazyFrame and transformations

5. Data Loading Strategy
Supported Formats

CSV

TSV

Parquet

IPC / Arrow

NDJSON

Excel (converted internally)

SQL databases

Rules

Always prefer lazy scanning:

LazyCsvReader

scan_parquet

scan_ipc

scan_ndjson

Never eagerly load large datasets

Enable streaming mode when collecting

Limit schema inference row count

Allow manual schema override

Excel Strategy

Load with calamine

Convert to Arrow batches

Cache as Parquet for reuse

SQL Strategy

Stream query results in batches

Convert to Arrow RecordBatch

Wrap as LazyFrame

6. Memory Efficiency Rules

Never:

Clone DataFrames unnecessarily

Convert entire frames to Vec<Vec<String>>

Materialize full dataset for preview

Always:

Use columnar Arrow memory

Limit preview rows

Render only visible rows/columns

Use streaming collect for large execution

7. Query Engine Design
App State
struct AppState {
    source: DataSource,
    operations: Vec<Operation>,
    preview_cache: Option<DataFrame>,
}
Operation Model
enum Operation {
    Filter(Expression),
    WithColumn(Expression),
    DropColumn(String),
    Rename { from: String, to: String },
    Sort { column: String, descending: bool },
}

LazyFrame is rebuilt from:

source → apply operations in order → LazyFrame
8. Editing Model

User edits do NOT mutate data.

Editing generates transformations:

Example (cell edit):

with_column(
    when(col("id").eq(lit(row_id)))
    .then(lit(new_value))
    .otherwise(col("column"))
)

Benefits:

Immutability

Undo/redo support

Deterministic behavior

No memory duplication

9. Preview System

Preview strategy:

Use .limit(N) (e.g., 1000 rows)

Recompute only when logical plan changes

Cache preview by logical plan hash

Execute in background worker thread

Send result to UI via channel

Never block UI thread.

10. Execution Modes
Preview Mode

Limited rows

Fast feedback

Streaming enabled

Full Execution Mode

Streaming collect

Export to:

Parquet

CSV

Excel

Or send to plot pipeline

Use:

.collect(streaming = true)

Or:

.sink_parquet(path)
11. Table Rendering (Critical Performance Area)

Must implement virtualization.

Render only:

visible_row_range
visible_column_range

Never:

Pre-render entire dataset

Pre-convert all values to strings

Instead:

Access Series directly

Format values on-demand

Use egui_extras::TableBuilder

12. Handling Large Datasets

If dataset exceeds RAM:

Use streaming execution

Push filters before collect

Avoid full materialization

Encourage Parquet caching

Memory-map when possible

Optional:

Auto-offer conversion to Parquet cache for CSV

13. Plotting Strategy

Never plot raw large datasets.

Before plotting:

Aggregate

Downsample

Bin

Use quantiles or grouping

Plot only processed data.

Avoid rendering millions of points.

14. State Persistence

Persist:

Data source configuration

Operation list

UI state

Cached metadata

Schema

Use:

rkyv (zero-copy deserialization preferred)

or bincode

Never serialize entire DataFrames unless explicitly exporting.

15. Undo / Redo

Maintain operation history stack:

Vec<Operation>

Undo:

Remove last operation

Rebuild LazyFrame

Redo:

Reapply operation

Do not snapshot full DataFrames.

16. Performance Best Practices

Let Polars manage threading

Avoid excessive async

Use background worker thread for execution

Cache schema and column statistics

Hash logical plan to avoid unnecessary recompute

Avoid string-heavy transformations

Avoid cloning Series

17. Internal Normalization Format

Internally normalize data to:

Arrow

Parquet

Parquet is the preferred cache format:

Columnar

Compressed

Schema-preserving

Fast reload

18. Advanced Optimization (Optional Phase)

Logical plan diffing

Column statistics caching (min, max, null count)

Approximate distinct counts (HyperLogLog)

Query plan visualization

Incremental recomputation

19. Non-Goals

Spreadsheet-level formula engine

Row-by-row mutation model

Web-based UI

Python bindings

20. Core Principle Summary

This application is:

A lazy analytical engine

A transformation graph editor

A virtualized data viewport

A high-performance desktop tool

It is NOT:

A spreadsheet clone

An in-memory data blob renderer

Everything must remain:

Columnar

Lazy

Streaming

Immutable

Parallel