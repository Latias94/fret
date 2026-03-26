use clap::Args;

use super::super::shared::ReportOutputArgs;

#[derive(Debug, Args)]
pub(crate) struct LayoutSidecarCommandArgs {
    #[arg(value_name = "SOURCE")]
    pub source: Option<String>,

    #[command(flatten)]
    pub output: ReportOutputArgs,

    #[arg(long = "print")]
    pub print: bool,
}
