use fret::app::App;
use fret_bootstrap::ui_app_driver;
use fret_runtime::PlatformCapabilities;

use super::{SimpleTodoView, install_demo_icons, install_demo_theme};

pub fn build_app() -> App {
    let mut app = App::new();
    app.set_global(PlatformCapabilities::default());
    install_demo_icons(&mut app);
    install_demo_theme(&mut app);
    app
}

pub fn build_runner_config() -> fret_launch::WinitRunnerConfig {
    fret_launch::WinitRunnerConfig {
        main_window_title: "fret-demo simple-todo".to_string(),
        main_window_size: fret_launch::WindowLogicalSize::new(560.0, 520.0),
        ..Default::default()
    }
}

pub fn build_fn_driver() -> impl fret_launch::WinitAppDriver {
    ui_app_driver::UiAppDriver::new(
        "simple-todo-demo",
        fret::advanced::view::view_init_window::<SimpleTodoView>,
        fret::advanced::view::view_view::<SimpleTodoView>,
    )
    .on_preferences(
        ui_app_driver::default_on_preferences::<
            fret::advanced::view::ViewWindowState<SimpleTodoView>,
        >,
    )
    .into_fn_driver()
}

#[cfg(not(target_arch = "wasm32"))]
pub fn run() -> anyhow::Result<()> {
    let _ = tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("fret=info".parse().unwrap())
                .add_directive("fret_render=info".parse().unwrap())
                .add_directive("fret_launch=info".parse().unwrap()),
        )
        .try_init();

    let app = build_app();
    let config = build_runner_config();
    let driver = build_fn_driver();
    crate::run_native_with_compat_driver(config, app, driver)
}

#[cfg(target_arch = "wasm32")]
pub fn run() -> anyhow::Result<()> {
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use fret_icons::IconRegistry;

    #[test]
    fn build_app_installs_semantic_checkbox_icons() {
        let app = build_app();
        let icons = app
            .global::<IconRegistry>()
            .expect("expected icon registry in simple todo demo app");

        assert!(icons.resolve(&fret_icons::ids::ui::CHECK).is_ok());
        assert!(icons.resolve(&fret_icons::ids::ui::MINUS).is_ok());
    }
}
