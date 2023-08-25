use clap::Parser;
use crate::rrm_error::RRMError;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
pub struct CmdArgs {
    pub files: Vec<String>
}

impl CmdArgs {
    pub fn parse_cmd_args() -> Result<CmdArgs, RRMError> {
        let cmd_args = CmdArgs::parse();
        Ok(cmd_args)
    }
}


