use std::path::PathBuf;

use clap::{ArgAction, Args, Subcommand};

use super::super::shared::{ChecksArgs, DevtoolsArgs, LaunchArgs, OutputArgs, TimingArgs};

fn parse_campaign_lane(raw: &str) -> Result<crate::regression_summary::RegressionLaneV1, String> {
    crate::registry::campaigns::parse_lane(raw)
}

#[derive(Debug, Args)]
pub(crate) struct CampaignCommandArgs {
    #[command(subcommand)]
    pub command: CampaignSubcommandArgs,
}

#[derive(Debug, Subcommand)]
pub(crate) enum CampaignSubcommandArgs {
    List(CampaignListArgs),
    Show(CampaignShowArgs),
    Validate(CampaignValidateArgs),
    Share(CampaignShareArgs),
    Run(CampaignRunArgs),
}

#[derive(Debug, Args, Default)]
pub(crate) struct CampaignFilterArgs {
    #[arg(long = "lane", value_name = "LANE", value_parser = parse_campaign_lane)]
    pub lane: Option<crate::regression_summary::RegressionLaneV1>,

    #[arg(long = "tier", value_name = "TIER")]
    pub tier: Option<String>,

    #[arg(long = "tag", value_name = "TAG", action = ArgAction::Append)]
    pub tags: Vec<String>,

    #[arg(long = "platform", value_name = "PLATFORM", action = ArgAction::Append)]
    pub platforms: Vec<String>,
}

impl CampaignFilterArgs {
    pub(crate) fn append_rest(&self, rest: &mut Vec<String>) {
        if let Some(lane) = self.lane {
            rest.push("--lane".to_string());
            rest.push(crate::registry::campaigns::lane_to_str(lane).to_string());
        }
        if let Some(tier) = self.tier.as_deref() {
            rest.push("--tier".to_string());
            rest.push(tier.to_string());
        }
        for tag in &self.tags {
            rest.push("--tag".to_string());
            rest.push(tag.clone());
        }
        for platform in &self.platforms {
            rest.push("--platform".to_string());
            rest.push(platform.clone());
        }
    }
}

#[derive(Debug, Args)]
pub(crate) struct CampaignListArgs {
    #[command(flatten)]
    pub filters: CampaignFilterArgs,

    #[arg(long = "json")]
    pub json: bool,
}

#[derive(Debug, Args)]
pub(crate) struct CampaignShowArgs {
    #[arg(value_name = "CAMPAIGN_ID")]
    pub campaign_id: String,

    #[arg(long = "json")]
    pub json: bool,
}

#[derive(Debug, Args)]
pub(crate) struct CampaignValidateArgs {
    #[arg(value_name = "MANIFEST")]
    pub manifests: Vec<PathBuf>,

    #[arg(long = "json")]
    pub json: bool,
}

#[derive(Debug, Args)]
pub(crate) struct CampaignShareArgs {
    #[arg(value_name = "SOURCE")]
    pub source: String,

    #[arg(long = "include-passed")]
    pub include_passed: bool,

    #[arg(long = "top", default_value_t = 5)]
    pub top: usize,

    #[arg(long = "warmup-frames", default_value_t = 0)]
    pub warmup_frames: u64,

    #[arg(long = "json")]
    pub json: bool,
}

#[derive(Debug, Args)]
pub(crate) struct CampaignRunArgs {
    #[arg(value_name = "CAMPAIGN_ID")]
    pub campaign_ids: Vec<String>,

    #[command(flatten)]
    pub filters: CampaignFilterArgs,

    #[command(flatten)]
    pub output: OutputArgs,

    #[command(flatten)]
    pub timing: TimingArgs,

    #[command(flatten)]
    pub devtools: DevtoolsArgs,

    #[command(flatten)]
    pub checks: ChecksArgs,

    #[arg(long = "top", default_value_t = 5)]
    pub top: usize,

    #[arg(long = "glob", value_name = "GLOB", action = ArgAction::Append)]
    pub globs: Vec<String>,

    #[arg(long = "script-dir", value_name = "DIR", action = ArgAction::Append)]
    pub script_dirs: Vec<PathBuf>,

    #[arg(long = "prewarm-script", value_name = "PATH", action = ArgAction::Append)]
    pub prewarm_scripts: Vec<PathBuf>,

    #[arg(long = "prelude-script", value_name = "PATH", action = ArgAction::Append)]
    pub prelude_scripts: Vec<PathBuf>,

    #[arg(long = "prelude-each-run")]
    pub prelude_each_run: bool,

    #[arg(long = "pack")]
    pub pack: bool,

    #[arg(long = "include-screenshots")]
    pub include_screenshots: bool,

    #[arg(long = "max-test-ids", default_value_t = 200)]
    pub max_test_ids: usize,

    #[arg(long = "all-test-ids")]
    pub lint_all_test_ids_bounds: bool,

    #[arg(long = "lint-eps-px", default_value_t = 0.5)]
    pub lint_eps_px: f32,

    #[arg(long = "no-suite-lint")]
    pub no_suite_lint: bool,

    #[arg(long = "reuse-launch")]
    pub reuse_launch: bool,

    #[command(flatten)]
    pub launch: LaunchArgs,
}
