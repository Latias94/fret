use clap::Args;

use super::super::shared::ReportOutputArgs;

fn parse_bundle_v2_mode(raw: &str) -> Result<String, String> {
    match raw {
        "all" | "changed" | "last" | "off" => Ok(raw.to_string()),
        _ => Err("invalid value for --mode (expected all|changed|last|off)".to_string()),
    }
}

#[derive(Debug, Args)]
pub(crate) struct BundleV2CommandArgs {
    #[arg(value_name = "SOURCE")]
    pub source: Option<String>,

    #[command(flatten)]
    pub output: ReportOutputArgs,

    #[arg(
        long = "mode",
        value_name = "MODE",
        default_value = "last",
        value_parser = parse_bundle_v2_mode
    )]
    pub mode: String,

    #[arg(long = "pretty")]
    pub pretty: bool,

    #[arg(long = "force")]
    pub force: bool,
}
