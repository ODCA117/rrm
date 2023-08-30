//use core::fmt::Debug
use std::{fmt::Debug, io};
use toml;

#[derive(thiserror::Error)]
pub enum RRMError {
    #[error("Failed to read settings path")]
    ReadSettingsPath,

    #[error("Failed to read settings file")]
    ReadSettingsFile(#[from] io::Error),

    #[error("No settings file found, will go with default settings")]
    NoSettingsFileFound,

    #[error("Failed to parse settings file")]
    SettingsFileParse(toml::de::Error),
}

impl Debug for RRMError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{}", self)?;
        Ok(())
    }
}

// impl std::fmt::Display for RRMError {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         match self {
//             Cmd => writeln!(f, "failed to parse cmd args"),
//         }
//     }
// }
