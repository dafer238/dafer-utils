use std::fmt;

// ─── Main Tab Navigation ─────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum MainTab {
    #[default]
    LoadPreview,
    Modify,
    Visualize,
}

impl MainTab {
    pub fn emoji(&self) -> &'static str {
        match self {
            MainTab::LoadPreview => "📂",
            MainTab::Modify => "⛭",
            MainTab::Visualize => "📊",
        }
    }

    pub fn all() -> [MainTab; 3] {
        [MainTab::LoadPreview, MainTab::Modify, MainTab::Visualize]
    }
}

// ─── File Type ────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum FileType {
    #[default]
    Csv,
    Parquet,
}

impl fmt::Display for FileType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FileType::Csv => write!(f, "CSV"),
            FileType::Parquet => write!(f, "Parquet"),
        }
    }
}

// ─── Plot Type ────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PlotType {
    #[default]
    Scatter,
    Line,
    Bar,
    Histogram,
}

impl PlotType {
    pub fn all() -> &'static [PlotType] {
        &[PlotType::Scatter, PlotType::Line, PlotType::Bar, PlotType::Histogram]
    }

    /// Returns true if this plot type needs a Y column.
    pub fn needs_y(&self) -> bool {
        !matches!(self, PlotType::Histogram)
    }
}

impl fmt::Display for PlotType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PlotType::Scatter => write!(f, "Scatter"),
            PlotType::Line => write!(f, "Line"),
            PlotType::Bar => write!(f, "Bar"),
            PlotType::Histogram => write!(f, "Histogram"),
        }
    }
}

// ─── Export Format ────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ExportFormat {
    #[default]
    Csv,
    Parquet,
}

impl fmt::Display for ExportFormat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ExportFormat::Csv => write!(f, "CSV"),
            ExportFormat::Parquet => write!(f, "Parquet"),
        }
    }
}

// ─── Theme ────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Theme {
    #[default]
    Dark,
    Light,
}
