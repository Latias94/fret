use clap::Args;

#[derive(Debug, Args)]
pub(crate) struct SessionArgs {
    #[arg(long = "session-auto", conflicts_with = "session")]
    pub session_auto: bool,

    #[arg(long = "session", value_name = "ID", conflicts_with = "session_auto")]
    pub session: Option<String>,
}
