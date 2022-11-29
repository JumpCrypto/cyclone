/// Shared arguments for the column, load, and msm examples.

#[derive(argh::FromArgs)]
/// Arguments for tests
pub struct Args {
    /// size of instance
    #[argh(positional)]
    pub size: u8,

    /// prefix of filenames
    #[argh(positional)]
    pub name: String,

    /// skip loading points
    #[argh(switch)]
    pub preloaded: bool,

    /// verbose output
    #[argh(switch, short = 'v')]
    pub verbose: bool,
}
