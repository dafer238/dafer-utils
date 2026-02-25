//! Persistence module: save/load application state using bincode.
//!
//! Only serializable metadata is persisted (data source config + operations).
//! DataFrames are NEVER serialized — they are rebuilt from the lazy pipeline.

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::Path;

use crate::datasource::DataSource;
use crate::operations::Operation;

/// Serializable application state for persistence.
/// Contains everything needed to reconstruct the full pipeline.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersistentState {
    pub source: Option<DataSource>,
    pub operations: Vec<Operation>,
}

impl PersistentState {
    /// Save state to a binary file using bincode.
    pub fn save(&self, path: &Path) -> Result<()> {
        let encoded = bincode::serialize(self)?;
        std::fs::write(path, encoded)?;
        Ok(())
    }

    /// Load state from a binary file.
    pub fn load(path: &Path) -> Result<Self> {
        let data = std::fs::read(path)?;
        let state: Self = bincode::deserialize(&data)?;
        Ok(state)
    }
}

impl Default for PersistentState {
    fn default() -> Self {
        Self {
            source: None,
            operations: Vec::new(),
        }
    }
}
