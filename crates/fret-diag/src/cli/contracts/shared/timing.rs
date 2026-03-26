use clap::Args;

#[derive(Debug, Args)]
pub(crate) struct TimingArgs {
    #[arg(long = "timeout-ms", default_value_t = 240_000)]
    pub timeout_ms: u64,

    #[arg(long = "poll-ms", default_value_t = 50)]
    pub poll_ms: u64,

    #[arg(long = "warmup-frames", default_value_t = 0)]
    pub warmup_frames: u64,
}
