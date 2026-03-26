use clap::{Args, ValueEnum};

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub(crate) enum InspectActionArg {
    On,
    Off,
    Toggle,
    Status,
}

impl InspectActionArg {
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Self::On => "on",
            Self::Off => "off",
            Self::Toggle => "toggle",
            Self::Status => "status",
        }
    }
}

#[derive(Debug, Args)]
pub(crate) struct InspectCommandArgs {
    #[arg(value_name = "ACTION")]
    pub action: InspectActionArg,

    #[arg(long = "consume-clicks", value_name = "BOOL")]
    pub consume_clicks: Option<bool>,
}
