use crate::enums::{BrowseMode, FileType, LoadMode, Theme};
use std::path::PathBuf;

#[derive(Default)]
pub struct AppState {
    pub theme: Theme,
    pub browse_mode: BrowseMode,
    pub file_type: FileType,
    pub load_mode: LoadMode,
    pub selected_path: Option<PathBuf>,
    /// Status message for the UI status line (e.g., "Loading...", "Computing...", etc.)
    pub status_message: Option<String>,
}
