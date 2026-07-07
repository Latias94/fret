use anyhow::Context as _;
use fret::app::prelude::*;

use super::TagsDemoView;

pub fn build_app() -> fret::app::App {
    crate::build_default_view_demo_app()
}

pub fn build_runner_config() -> fret_launch::WinitRunnerConfig {
    crate::build_default_view_demo_runner_config("fret-demo tags_demo", (960.0, 640.0))
}

pub fn build_fn_driver() -> impl fret_launch::WinitAppDriver {
    crate::build_default_view_demo_fn_driver::<TagsDemoView>("tags-demo")
}

#[cfg(not(target_arch = "wasm32"))]
pub fn run() -> anyhow::Result<()> {
    FretApp::new("tags-demo")
        .window("tags_demo", (960.0, 640.0))
        .view::<TagsDemoView>()?
        .run()
        .context("run tags_demo app")
}

#[cfg(target_arch = "wasm32")]
pub fn run() -> anyhow::Result<()> {
    Ok(())
}
