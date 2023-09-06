use crate::{rrm_error::RRMError, database::FileEntryDB};
use crate::database::FileDB;
use std::{path::PathBuf, fs};
use clap::Parser;
use directories_next::ProjectDirs;
use toml;
use serde::Deserialize;
use log::trace;

const CONFIG_FILE: &str = "rrm_settings.toml";
const DATABASE: &str = "trashDB.db";

#[derive(Parser, Clone)]
#[command(author, version, about, long_about = None)]
struct CmdArgs {
    files: Vec<String>,

    #[arg(short, long)]
    config_path: Option<String>,

    #[arg(short)]
    add: Option<String>,

    #[arg(short)]
    remove: Option<String>,

    #[arg(short)]
    get: Option<String>,
}

pub struct App {
    pub files: Vec<PathBuf>,
    pub trash_path: PathBuf,
    file_db: FileDB,
    cmd_args: CmdArgs,
}

#[derive(Deserialize)]
struct Config {
    trash_path: String,
}

impl App {
    pub fn create() -> Result<App, RRMError> {
        let cmd_args = CmdArgs::parse();

        /* try to read the default project dirs, may seem a bit to OS independent, make it more
        * difficult to test */
        // Linux:   /home/Alice/.config/rrm
        // Windows: C:\Users\Alice\AppData\Roaming\rrm\rrm
        // macOS:   /Users/Alice/Library/Application Support/ODCA.rrm
        let proj_dirs = ProjectDirs::from("", "ODCA",  "rrm").ok_or(RRMError::ReadSettingsPath)?;
        let settings_path = match &cmd_args.config_path {
            Some(p) => PathBuf::from(p),
            None => {
                let mut settings_path = proj_dirs.config_dir().to_path_buf();
                settings_path.push(CONFIG_FILE);
                settings_path
            }
        };
        let file_path = settings_path.as_path().display().to_string();
        trace!("Settings file: {}", file_path);
        if !settings_path.is_file() {
            // TODO: have some default settings for this instead
            return Err(RRMError::FileNotFound(file_path));
        }

        let file_contents = fs::read_to_string(&settings_path)?;
        trace!("Config file content: {}", &file_contents);
        let config: Config = toml::from_str(&file_contents).map_err(RRMError::SettingsFileParse)?;
        trace!("trash path: {}", &config.trash_path);

        let mut data_dir = proj_dirs.data_dir().to_path_buf();
        trace!("Data dir: {}", data_dir.to_string_lossy());
        fs::create_dir_all(&data_dir)?;
        data_dir.push(DATABASE);
        let file_db = FileDB::new(&data_dir)?;

        Ok(App {
            files: Vec::new(),
            trash_path: PathBuf::from(config.trash_path),  // Make this configurable
            file_db,
            cmd_args,
        })
    }

    /// Creates the trash bin directory if it does not exists
    /// If it exists but is not a directory Error is returned
    /// If it exists and is a directory nothing will be done
    pub fn create_trash(&self) -> Result<(), RRMError> {

        /* Find out if trash exists or not */
        match self.trash_path.try_exists() {
            Err(_) => Err(RRMError::TrashNotVerified),
            Ok(true) => {
                if self.trash_path.is_symlink() || self.trash_path.is_file() {
                    Err(RRMError::TrashNotDir)
                } else {
                    Ok(())
                }
            },
            Ok(false) => {
                trace!("Trash does not exists, create trashbin");
                fs::create_dir_all(&self.trash_path)?;
                Ok(())
            },
        }
    }

    pub fn test_database(&mut self) -> Result<(), RRMError>{
        if let Some(name) = &self.cmd_args.add {
            let file: FileEntryDB = FileEntryDB {name: name.clone(), origin: "test_origin".to_string()};
            self.file_db.add(file)?;
        }

        if let Some(name) = &self.cmd_args.get {
            let file = self.file_db.get(name)?;
        }

        if let Some(name) = &self.cmd_args.remove {
            self.file_db.remove(name)?;
        }

        Ok(())
    }
}

