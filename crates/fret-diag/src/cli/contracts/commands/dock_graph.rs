use clap::Args;

#[derive(Debug, Args)]
pub(crate) struct DockGraphCommandArgs {
    #[arg(value_name = "SOURCE")]
    pub source: String,

    #[arg(long = "json")]
    pub json: bool,
}
