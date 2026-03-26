use clap::Args;

#[derive(Debug, Args)]
pub(crate) struct WarmupFramesArgs {
    #[arg(long = "warmup-frames", default_value_t = 0)]
    pub warmup_frames: u64,
}
