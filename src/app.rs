use crate::{rrm_error::RRMError, database::FileEntryDB};
use crate::database::FileDB;
use std::process::exit;
use std::{path::PathBuf, fs};
use clap::Parser;
use directories_next::ProjectDirs;
use toml;
use serde::Deserialize;
use log::{trace, info, error};

const CONFIG_FILE: &str = "rrm_settings.toml";
const TRASH_PATH: &str = "/tmp/rrm/trash";
const DATABASE: &str = "trashDB.db";

#[derive(Parser, Clone)]
#[command(author, version, about, long_about = None)]
struct CmdArgs {
    files: Vec<String>,

    #[arg(short, long, help = "Path to Config file")]
    config_path: Option<String>,

    #[arg(short, long, help = "List all files in Trash DB")]
    list: bool,

    #[arg(long, help = "Clear all items in Trash and clear trash DB, not reversable")]
    clear_trash: bool,

    #[arg(short, long, help = "Restore last removed item")]
    undo: bool,
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

        let config: Config = if settings_path.exists() {
            let file_path = settings_path.as_path().display().to_string();
            trace!("Settings file: {}", file_path);
            if !settings_path.is_file() {
                // TODO: have some default settings for this instead
                return Err(RRMError::FileNotFound(file_path));
            }

            let file_contents = fs::read_to_string(&settings_path)?;
            toml::from_str(&file_contents).map_err(RRMError::SettingsFileParse)?
        } else {
            Config {trash_path: String::from(TRASH_PATH) }
        };

        trace!("trash path: {}", &config.trash_path);
        let mut data_dir = proj_dirs.data_dir().to_path_buf();
        trace!("Data dir: {}", data_dir.to_string_lossy());
        fs::create_dir_all(&data_dir)?;
        data_dir.push(DATABASE);
        let file_db = FileDB::new(&data_dir)?;

        let trash_path = PathBuf::from(config.trash_path);
        create_trash(&trash_path)?;
        let files = cmd_args.files.iter().map(|f| PathBuf::from(f)).collect();
        Ok(App {
            files,
            trash_path,
            file_db,
            cmd_args,
        })
    }

    /// Creates the trash bin directory if it does not exists
    /// If it exists but is not a directory Error is returned
    /// If it exists and is a directory nothing will be done

    pub fn execute(&self) -> Result<(), RRMError> {
        if self.cmd_args.clear_trash {
            // Clear everything it trash dir and DB
            let mut confirm = String::new();
            println!("Do you really want to clear the trash bin? 'y|n' (Cannot undo this)");
            std::io::stdin().read_line(&mut confirm)?;
            match confirm.to_lowercase().as_str() {
                "y\n" | "yes\n" => {
                    println!("Will clear trash bin");
                    self.permanenent_delete()?;
                    self.file_db.clear_db();
                }
                _ => {
                    println!("Will not clear trash bin");
                    exit(0);
                }
            }
        } else if self.cmd_args.list {
            // List all files and their origin in trashbin
            self.list_trash()?;

        } else {
            self.move_to_trash()?;
        }
        Ok(())
    }

    fn move_to_trash(&self) -> Result<(), RRMError> {
        for f in self.files.iter() {
            let mut new_path = PathBuf::from(&self.trash_path.clone());
            let file_name = f.as_path().file_name().unwrap().to_string_lossy().to_string();
            if f.is_symlink() {
                // TODO: Add option to follow symlink to remove file and symlink.
                // Probably a bug here.
                trace!("symling: {:?}", file_name);

                // Current behvaior, Removes the link which will breake it
                // If the file which the link is linking to also is removed and keep the same
                // relative path from the link the link will still work.
                new_path.push(&file_name);
                let res = fs::rename(f, new_path);
                match res {
                    Ok(_) => {
                        trace!("Moved symlink");
                        self.store_in_db(&file_name)?;
                    }
                    Err(e) => error!("{}", e.to_string()),
                }
            } else if f.is_dir() {
                // Will move the entire directory and everything in the directory
                // Paths will be kept.
                trace!("dir: {:?}", file_name);
                new_path.push(&file_name);
                let res = fs::rename(f, new_path);
                match res {
                    Ok(_) => {
                        trace!("Moved directory");
                        // add file to database
                        self.store_in_db(&file_name)?;
                    }
                    Err(e) => error!("{}", e.to_string()),
                }
            } else if f.is_file() {
                // Will move the file,
                trace!("file: {:?}", file_name);
                new_path.push(&file_name);

                let res = fs::rename(f, new_path);
                match res {
                    Ok(_) => {
                        trace!("Moved file");
                        self.store_in_db(&file_name)?;
                    }
                    Err(e) => error!("{}", e.to_string()),
                }
            } else {
                error!("The path is not pointing to anythin");
            }
        }
        Ok(())
    }

    fn permanenent_delete(&self) -> Result<(), RRMError> {
        for entry in fs::read_dir(&self.trash_path)? {
            let entry = entry?;
            if entry.path().is_symlink() || entry.path().is_file() {
                fs::remove_file(entry.path())?;
            } else {
                fs::remove_dir_all(entry.path())?;
            }
        }
        Ok(())
    }

    fn list_trash(&self) -> Result<(), RRMError> {
        let db_files = self.file_db.get_all()?;
        let mut dir_files: Vec<String> = Vec::with_capacity(db_files.len());

        for entry in fs::read_dir(&self.trash_path)? {
            let entry = entry?;
            dir_files.push(entry.path().file_name().unwrap().to_string_lossy().to_string());
        }

        let db_files = db_files.iter().filter(|&f| dir_files.contains(&f.name));

        println!("name ->\t\torigin");
        for e in db_files {
            println!("{} ->\t\t{}", e.name, e.origin);
        }
        Ok(())
    }

    fn store_in_db(&self, name: &String) -> Result<(), RRMError> {
        // Should be fine as we are in this directory
        let origin = std::env::current_dir().unwrap().as_path().display().to_string();
        let file_entry_db = FileEntryDB { name: name.clone(), origin };
        self.file_db.add(file_entry_db)?;
        Ok(())
    }
}

fn create_trash(trash_path: &PathBuf) -> Result<(), RRMError> {
    /* Find out if trash exists or not */
    match trash_path.try_exists() {
        Err(_) => Err(RRMError::TrashNotVerified),
        Ok(true) => {
            if trash_path.is_symlink() || trash_path.is_file() {
                Err(RRMError::TrashNotDir)
            } else {
                Ok(())
            }
        },
        Ok(false) => {
            trace!("Trash does not exists, create trashbin");
            fs::create_dir_all(trash_path)?;
            Ok(())
        },
    }
}

