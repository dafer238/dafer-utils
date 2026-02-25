use polars::prelude::DataFrame;

use dafer_utils::data_loader::ColumnStats;
use dafer_utils::datasource::DataSource;
use dafer_utils::operations::{DTypeTag, FillNullStrategy, FilterOp, Operation, OperationType};

use crate::enums::{ExportFormat, MainTab, PlotType, Theme};

/// Central application state.
///
/// All UI state is stored here — no `static mut` anywhere.
/// The preview DataFrame is cached and rebuilt only when the pipeline changes.
/// A pre-computed string grid is used for high-performance table rendering.
pub struct AppState {
    // ── Theme ──
    pub theme: Theme,

    // ── Navigation ──
    pub selected_tab: MainTab,

    // ── Data Core ──
    pub source: Option<DataSource>,
    pub operations: Vec<Operation>,
    pub redo_stack: Vec<Operation>,

    // ── Preview Cache ──
    pub preview_df: Option<DataFrame>,
    pub full_df: Option<DataFrame>,
    pub preview_rows: u32,
    pub preview_dirty: bool,

    // ── Auto-cast detection ──
    pub auto_cast_detected: bool,

    // ── Schema Info ──
    pub column_names: Vec<String>,
    pub column_dtypes: Vec<String>,
    pub column_stats: Vec<ColumnStats>,
    pub row_count: Option<usize>,

    // ── Table String Cache (performance: pre-computed, no DataFrame access during render) ──
    pub cached_cell_strings: Vec<Vec<String>>,
    pub cached_header_names: Vec<String>,
    pub table_cache_dirty: bool,

    // ── Cell/Row/Column Selection ──
    pub selected_cell: Option<(usize, usize)>,
    pub selected_row: Option<usize>,
    pub selected_col: Option<usize>,

    // ── Visual sort (cosmetic, not part of pipeline) ──
    pub sort_column: Option<String>,
    pub sort_descending: bool,

    // ── Modify Tab: Operation Builder ──
    pub selected_op: OperationType,
    pub filter_column: String,
    pub filter_op: FilterOp,
    pub filter_value: String,
    pub rename_from: String,
    pub rename_to: String,
    pub drop_column: String,
    pub select_checks: Vec<bool>,
    pub cast_column: String,
    pub cast_dtype: DTypeTag,
    pub fill_column: String,
    pub fill_strategy: FillNullStrategy,
    pub fill_value: String,
    pub sort_op_column: String,
    pub sort_op_descending: bool,
    pub limit_n: u32,
    pub datetime_column: String,
    pub datetime_format: String,

    // ── Visualize Tab ──
    pub plot_type: PlotType,
    pub plot_x: String,
    pub plot_y_columns: Vec<String>,
    pub plot_multi_data: Vec<(String, Vec<[f64; 2]>)>,
    pub plot_dirty: bool,
    pub histogram_bins: usize,
    pub plot_x_is_datetime: bool,

    // ── Plot Reset Zoom ──
    pub plot_reset_counter: u64,

    // ── Export ──
    pub export_format: ExportFormat,

    // ── Status ──
    pub status: String,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            theme: Theme::default(),
            selected_tab: MainTab::default(),

            source: None,
            operations: Vec::new(),
            redo_stack: Vec::new(),

            preview_df: None,
            full_df: None,
            preview_rows: 200,
            preview_dirty: false,

            auto_cast_detected: false,

            column_names: Vec::new(),
            column_dtypes: Vec::new(),
            column_stats: Vec::new(),
            row_count: None,

            cached_cell_strings: Vec::new(),
            cached_header_names: Vec::new(),
            table_cache_dirty: false,

            selected_cell: None,
            selected_row: None,
            selected_col: None,

            sort_column: None,
            sort_descending: false,

            selected_op: OperationType::default(),
            filter_column: String::new(),
            filter_op: FilterOp::default(),
            filter_value: String::new(),
            rename_from: String::new(),
            rename_to: String::new(),
            drop_column: String::new(),
            select_checks: Vec::new(),
            cast_column: String::new(),
            cast_dtype: DTypeTag::default(),
            fill_column: String::new(),
            fill_strategy: FillNullStrategy::default(),
            fill_value: String::new(),
            sort_op_column: String::new(),
            sort_op_descending: false,
            limit_n: 1000,
            datetime_column: String::new(),
            datetime_format: "%Y-%m-%d %H:%M:%S".to_string(),

            plot_type: PlotType::default(),
            plot_x: String::new(),
            plot_y_columns: Vec::new(),
            plot_multi_data: Vec::new(),
            plot_dirty: true,
            histogram_bins: 30,
            plot_x_is_datetime: false,

            plot_reset_counter: 0,

            export_format: ExportFormat::default(),

            status: "Ready".to_string(),
        }
    }
}
