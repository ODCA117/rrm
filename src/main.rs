mod rrm_error;
mod app;
mod database;

use app::App;
use rrm_error::RRMError;

fn main() -> Result<(), RRMError>{
    env_logger::init();
    let app = App::create()?;
    app.execute()?;

    // If link/dir/file is in a path, it will be move to the root of the trashbin.
    // Path will not be copied.
    Ok(())
}
