use std::path::PathBuf;

use clap::{Args, Subcommand};

#[derive(Debug, Args)]
pub(crate) struct ListCommandArgs {
    #[command(subcommand)]
    pub command: ListSubcommandArgs,
}

#[derive(Debug, Subcommand)]
pub(crate) enum ListSubcommandArgs {
    Scripts(ListScriptsArgs),
    Suites(ListSuitesArgs),
    Sessions(ListSessionsArgs),
}

#[derive(Debug, Args, Default)]
pub(crate) struct ListFilterArgs {
    #[arg(long = "contains", value_name = "NEEDLE")]
    pub contains: Option<String>,

    #[arg(long = "case-sensitive")]
    pub case_sensitive: bool,

    #[arg(long = "all")]
    pub all: bool,
}

impl ListFilterArgs {
    pub(crate) fn append_rest(&self, rest: &mut Vec<String>) {
        if let Some(needle) = self.contains.as_deref() {
            rest.push("--contains".to_string());
            rest.push(needle.to_string());
        }
        if self.case_sensitive {
            rest.push("--case-sensitive".to_string());
        }
        if self.all {
            rest.push("--all".to_string());
        }
    }
}

#[derive(Debug, Args)]
pub(crate) struct ListScriptsArgs {
    #[command(flatten)]
    pub filters: ListFilterArgs,

    #[arg(long = "top", value_name = "N")]
    pub top: Option<usize>,

    #[arg(long = "json")]
    pub json: bool,
}

#[derive(Debug, Args)]
pub(crate) struct ListSuitesArgs {
    #[command(flatten)]
    pub filters: ListFilterArgs,

    #[arg(long = "top", value_name = "N")]
    pub top: Option<usize>,

    #[arg(long = "json")]
    pub json: bool,
}

#[derive(Debug, Args)]
pub(crate) struct ListSessionsArgs {
    #[command(flatten)]
    pub filters: ListFilterArgs,

    #[arg(long = "dir", value_name = "DIR")]
    pub dir: Option<PathBuf>,

    #[arg(long = "top", value_name = "N")]
    pub top: Option<usize>,

    #[arg(long = "json")]
    pub json: bool,
}
