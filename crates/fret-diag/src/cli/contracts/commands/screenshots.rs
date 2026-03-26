use clap::Args;

#[derive(Debug, Args)]
pub(crate) struct ScreenshotsCommandArgs {
    #[arg(value_name = "SOURCE")]
    pub source: String,

    #[arg(long = "json")]
    pub json: bool,
}
