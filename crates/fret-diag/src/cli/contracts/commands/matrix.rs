use clap::{ArgAction, Args, ValueEnum};

use super::super::shared::{CompareArgs, OutputArgs, TimingArgs};

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub(crate) enum MatrixTargetArg {
    UiGallery,
}

impl MatrixTargetArg {
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Self::UiGallery => "ui-gallery",
        }
    }
}

#[derive(Debug, Args)]
pub(crate) struct MatrixCommandArgs {
    #[arg(value_name = "TARGET")]
    pub target: MatrixTargetArg,

    #[command(flatten)]
    pub output: OutputArgs,

    #[command(flatten)]
    pub timing: TimingArgs,

    #[command(flatten)]
    pub compare: CompareArgs,

    #[arg(long = "check-view-cache-reuse-min")]
    pub check_view_cache_reuse_min: Option<u64>,

    #[arg(long = "check-view-cache-reuse-stable-min")]
    pub check_view_cache_reuse_stable_min: Option<u64>,

    #[arg(long = "check-overlay-synthesis-min")]
    pub check_overlay_synthesis_min: Option<u64>,

    #[arg(long = "check-viewport-input-min")]
    pub check_viewport_input_min: Option<u64>,

    #[arg(long = "env", value_name = "KEY=VALUE", action = ArgAction::Append)]
    pub env: Vec<String>,

    #[arg(long = "launch-high-priority", requires = "launch")]
    pub launch_high_priority: bool,

    #[arg(
        long = "launch",
        value_name = "CMD",
        num_args = 1..,
        allow_hyphen_values = true,
        required = true
    )]
    pub launch: Vec<String>,
}

impl MatrixCommandArgs {
    pub(crate) fn normalized_launch_argv(&self) -> Vec<String> {
        let mut values = self.launch.clone();
        if values.first().is_some_and(|value| value == "--") {
            values.remove(0);
        }
        values
    }
}
