mod rrm_error;
mod setting;

use std::path::Path;
use std::{path::PathBuf, fs};
use crate::setting::CmdArgs;
use log::{trace, error};
use rrm_error::RRMError;

struct App {
    files: Vec<PathBuf>,
    args: CmdArgs,
    trash: String,
}


fn main() -> Result<(), RRMError>{
    env_logger::init();

    trace!("parse command line arguments");
    let args = CmdArgs::parse_cmd_args()?;

    let mut app = App {files: Vec::new(), args, trash: String::from("/tmp/rrm/trashbin/")};
    fs::create_dir_all(PathBuf::from(&app.trash));

    for f in app.args.files.iter() {
        app.files.push(PathBuf::from(f));
    }
    trace!("Move files {:?} to trashbin", &app.args.files);

    // If link/dir/file is in a path, it will be move to the root of the trashbin.
    // Path will not be copied.
    for f in app.files.iter() {
        if f.is_symlink() {
            // TODO: Add option to follow symlink to remove file and symlink.
            trace!("symling: {:?}", f.file_name());

            // Current behvaior, Removes the link which will breake it
            // If the file which the link is linking to also is removed and keep the same
            // relative path from the link the link will still work.
            let mut new_path = PathBuf::from(app.trash.clone());
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
            let mut new_path = PathBuf::from(app.trash.clone());
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
            let mut new_path = PathBuf::from(app.trash.clone());
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
