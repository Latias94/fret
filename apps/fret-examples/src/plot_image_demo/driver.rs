use anyhow::Context as _;
use fret::app::prelude::*;

use super::PlotImageDemoView;

pub fn build_app() -> fret::app::App {
    crate::build_default_view_demo_app()
}

pub fn build_runner_config() -> fret_launch::WinitRunnerConfig {
    crate::build_default_view_demo_runner_config("fret-demo plot_image_demo", (960.0, 640.0))
}

pub fn build_fn_driver() -> impl fret_launch::WinitAppDriver {
    crate::build_default_view_demo_fn_driver::<PlotImageDemoView>("plot-image-demo")
}

#[cfg(not(target_arch = "wasm32"))]
pub fn run() -> anyhow::Result<()> {
    FretApp::new("plot-image-demo")
        .window("plot_image_demo", (960.0, 640.0))
        .view::<PlotImageDemoView>()?
        .run()
        .context("run plot_image_demo app")
}

#[cfg(target_arch = "wasm32")]
pub fn run() -> anyhow::Result<()> {
    Ok(())
}
