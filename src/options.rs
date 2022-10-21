#[derive(Debug, clap::Parser)]
pub struct Args {
    /// An http link of the repository to make a video out of.
    ///
    /// An example:
    ///
    /// https://github.com/sloganking/codemov
    #[clap(long, short = 'r', help_heading = "INPUT")]
    pub repo: String,

    /// The branch of the repository to be rendered.
    #[clap(long, short = 'b', help_heading = "INPUT")]
    pub branch: String,

    /// Open the output video with the system's default image viewer.
    #[clap(long, help_heading = "OUTPUT")]
    pub open: bool,

    /// The width of the output video.
    #[clap(long, default_value_t = 1920, help_heading = "OUTPUT")]
    pub width: u32,

    /// The height of the output video.
    #[clap(long, default_value_t = 1080, help_heading = "OUTPUT")]
    pub height: u32,

    /// The height of the output video.
    #[clap(long, default_value_t = 15, help_heading = "OUTPUT")]
    pub fps: u32,
}
