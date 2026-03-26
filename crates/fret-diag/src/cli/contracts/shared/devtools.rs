use clap::Args;

#[derive(Debug, Args)]
pub(crate) struct DevtoolsArgs {
    #[arg(long = "devtools-ws-url", value_name = "URL")]
    pub devtools_ws_url: Option<String>,

    #[arg(long = "devtools-token", value_name = "TOKEN")]
    pub devtools_token: Option<String>,

    #[arg(long = "devtools-session-id", value_name = "ID")]
    pub devtools_session_id: Option<String>,
}
