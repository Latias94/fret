use clap::Args;

#[derive(Debug, Args)]
pub(crate) struct DevtoolsArgs {
    #[arg(
        long = "devtools-ws-url",
        value_name = "URL",
        requires = "devtools_token"
    )]
    pub devtools_ws_url: Option<String>,

    #[arg(
        long = "devtools-token",
        value_name = "TOKEN",
        requires = "devtools_ws_url"
    )]
    pub devtools_token: Option<String>,

    #[arg(
        long = "devtools-session-id",
        value_name = "ID",
        requires = "devtools_ws_url"
    )]
    pub devtools_session_id: Option<String>,
}
