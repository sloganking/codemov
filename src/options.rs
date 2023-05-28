#[derive(Debug, clap::Parser)]
pub struct Args {
    /// An http link of the repository to make a video out of.
    ///
    /// An example:
    ///
    /// https://github.com/sloganking/codemov
    #[arg(long, short = 'r', help_heading = "INPUT")]
    pub repo: String,

    /// The branch of the repository to be rendered.
    #[arg(long, short = 'b', help_heading = "INPUT")]
    pub branch: String,

    /// Open the output video with the system's default image viewer.
    #[arg(long, help_heading = "OUTPUT")]
    pub open: bool,

    /// The width of the output video.
    #[arg(long, default_value_t = 1920, help_heading = "OUTPUT")]
    pub width: u32,

    /// The height of the output video.
    #[arg(long, default_value_t = 1080, help_heading = "OUTPUT")]
    pub height: u32,

    /// The frames per second the output video will have.
    #[arg(long, default_value_t = 15, help_heading = "OUTPUT")]
    pub fps: u32,

    /// Where to save the output file and what name to give it.
    #[arg(long, short = 'o', default_value_t = String::from("./output.mp4"), help_heading = "OUTPUT")]
    pub output_dir: String,
}
