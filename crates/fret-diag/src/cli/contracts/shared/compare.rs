use clap::Args;

#[derive(Debug, Args)]
pub(crate) struct CompareArgs {
    #[arg(long = "compare-eps-px", default_value_t = 0.5)]
    pub compare_eps_px: f32,

    #[arg(long = "compare-ignore-bounds")]
    pub compare_ignore_bounds: bool,

    #[arg(long = "compare-ignore-scene-fingerprint")]
    pub compare_ignore_scene_fingerprint: bool,
}
