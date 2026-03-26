use clap::Args;

#[derive(Debug, Args)]
pub(crate) struct ChecksArgs {
    #[arg(long = "check-stale-paint", value_name = "TEST_ID")]
    pub check_stale_paint: Option<String>,

    #[arg(long = "check-stale-paint-eps", default_value_t = 0.5)]
    pub check_stale_paint_eps: f32,

    #[arg(long = "check-stale-scene", value_name = "TEST_ID")]
    pub check_stale_scene: Option<String>,

    #[arg(long = "check-stale-scene-eps", default_value_t = 0.5)]
    pub check_stale_scene_eps: f32,

    #[arg(long = "check-idle-no-paint-min", value_name = "N")]
    pub check_idle_no_paint_min: Option<u64>,

    #[arg(long = "check-pixels-changed", value_name = "TEST_ID")]
    pub check_pixels_changed: Option<String>,

    #[arg(long = "check-pixels-unchanged", value_name = "TEST_ID")]
    pub check_pixels_unchanged: Option<String>,
}
