mod rrm_error;
mod app;

use app::App;
use std::{path::PathBuf, fs};
use log::{trace, error};
use rrm_error::RRMError;

fn main() -> Result<(), RRMError>{
    env_logger::init();

    trace!("parse command line arguments");
    let app = App::new()?;

    // TODO, move this to the application init??
    fs::create_dir_all(PathBuf::from(&app.trash_path));

    trace!("Move files {:?} to trashbin", &app.files);

    // If link/dir/file is in a path, it will be move to the root of the trashbin.
    // Path will not be copied.
    for f in app.files.iter() {
        if f.is_symlink() {
            // TODO: Add option to follow symlink to remove file and symlink.
            trace!("symling: {:?}", f.file_name());

            // Current behvaior, Removes the link which will breake it
            // If the file which the link is linking to also is removed and keep the same
            // relative path from the link the link will still work.
            let mut new_path = PathBuf::from(app.trash_path.clone());
            let file_name = f.file_name().unwrap();
            new_path.push(file_name);

            let res = fs::rename(f, new_path);
            match res {
                Ok(_) => trace!("Moved file"),
                Err(e) => error!("{}", e.to_string()),
            }

        } else if f.is_dir() {
            // Will move the entire directory and everything in the directory
            // Paths will be kept.
            trace!("dir: {:?}", f.file_name());
            let mut new_path = PathBuf::from(app.trash_path.clone());
            let file_name = f.file_name().unwrap();
            new_path.push(file_name);

            let res = fs::rename(f, new_path);
            match res {
                Ok(_) => trace!("Moved file"),
                Err(e) => error!("{}", e.to_string()),
            }

        } else if f.is_file() {
            // Will move the file,
            trace!("file: {:?}", f.file_name());
            let mut new_path = PathBuf::from(app.trash_path.clone());
            let file_name = f.file_name().unwrap();
            new_path.push(file_name);

            let res = fs::rename(f, new_path);
            match res {
                Ok(_) => trace!("Moved file"),
                Err(e) => error!("{}", e.to_string()),
            }

        } else {
            error!("The path is not pointing to anythin");
        }
    }

    Ok(())
}
