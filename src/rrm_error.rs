//use core::fmt::Debug
use std::fmt::Debug;

#[derive(thiserror::Error)]
pub enum RRMError {
    #[error("Failed parsing command line arguments")]
    Cmd,
    #[error("Failed reading file name")]
    ReadFileName,
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
