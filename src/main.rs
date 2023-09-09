mod rrm_error;
mod app;
mod database;

use app::App;
use std::{path::PathBuf, fs};
use log::{trace, info, error};
use rrm_error::RRMError;

fn main() -> Result<(), RRMError>{
    env_logger::init();
    let app = App::create()?;
    app.create_trash()?;
    info!("Move files {:?} to trashbin", &app.files);


    // If link/dir/file is in a path, it will be move to the root of the trashbin.
    // Path will not be copied.
    for f in app.files.iter() {
        let mut new_path = PathBuf::from(&app.trash_path.clone());
        let file_name = f.as_path().display().to_string();
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
                    app.store_in_db(&file_name);
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
                    app.store_in_db(&file_name);
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
                    app.store_in_db(&file_name);
                }
                Err(e) => error!("{}", e.to_string()),
            }
        } else {
            error!("The path is not pointing to anythin");
        }
    }
    Ok(())
}
