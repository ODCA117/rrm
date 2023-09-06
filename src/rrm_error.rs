//use core::fmt::Debug
use std::{fmt::Debug, io};
use toml;

#[derive(thiserror::Error)]
pub enum RRMError {
    #[error("Failed to read settings path")]
    ReadSettingsPath,

    // Can use #[source] instead to handle same source but give different result.
    #[error("IO Error")]
    IOError(#[from] io::Error),

    #[error("Could not find file")]
    FileNotFound,

    #[error("Failed to parse settings file")]
    SettingsFileParse(toml::de::Error),

    #[error("Trash is not a directory")]
    TrashNotDir,

    #[error("Failed to verify path")]
    TrashNotVerified,

    #[error("Failed to open database")]
    DBConnection(#[from] rusqlite::Error),
}

impl Debug for RRMError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{}", self)?;
        Ok(())
    }
}

