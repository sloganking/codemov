use std::path::PathBuf;

#[derive(Debug, clap::Parser)]
pub struct Args {
    /// The directory to read UTF-8 encoded text files from.
    #[clap(long, short = 'r', default_value = "input", help_heading = "IO")]
    pub repo: String,

    /// Open the output video with the system's default image viewer.
    #[clap(long, help_heading = "OUTPUT")]
    pub open: bool,
}
