use crate::rrm_error::RRMError;
use std::{path::PathBuf, fs};
use clap::Parser;
use directories_next::ProjectDirs;
use toml;
use serde::Deserialize;
use log::trace;

const CONFIG_FILE: &str = "rrm_settings.toml";

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct CmdArgs {
    files: Vec<String>,

    #[arg(short, long)]
    config_path: Option<String>,
}

pub struct App {
    pub files: Vec<PathBuf>,
    pub trash_path: PathBuf,
}

#[derive(Deserialize)]
struct Config {
    trash_path: String,
}

impl App {
    pub fn new() -> Result<App, RRMError> {
        let cmd_args = CmdArgs::parse();
        let settings_path = match cmd_args.config_path {
            Some(p) => PathBuf::from(p),
            None => {
                /* try to read the default config path, may seem a bit to OS independent, make it more
                * difficult to test */
                // Linux:   /home/Alice/.config/rrm
                // Windows: C:\Users\Alice\AppData\Roaming\rrm\rrm
                // macOS:   /Users/Alice/Library/Application Support/ODCA.rrm
                let proj_dirs = ProjectDirs::from("", "ODCA",  "rrm").ok_or(RRMError::ReadSettingsPath)?;
                let mut settings_path = proj_dirs.config_dir().to_path_buf();
                settings_path.push(CONFIG_FILE);
                settings_path
            }
        };
        trace!("Settings file: {}", settings_path.to_string_lossy());
        if !settings_path.is_file() {
            // TODO: have some default settings for this instead
            return Err(RRMError::NoSettingsFileFound);
        }

        let file_contents = fs::read_to_string(&settings_path)?;
        let settings: Config = toml::from_str(&file_contents).map_err(RRMError::SettingsFileParse)?;
        Ok(App {
            files: Vec::new(),
            trash_path: PathBuf::from(settings.trash_path),  // Make this configurable
        })
    }
}

