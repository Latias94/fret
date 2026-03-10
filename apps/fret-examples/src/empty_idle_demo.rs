use fret::advanced::prelude::*;
use fret_ui::ElementContext;

struct EmptyIdleState;

pub fn run() -> anyhow::Result<()> {
    ui_app("empty-idle-demo", init_window, view)
        .setup(fret_bootstrap::install_default_i18n_backend)
        .with_main_window("empty_idle_demo", (520.0, 240.0))
        .run()
        .map_err(anyhow::Error::from)
}

fn init_window(_app: &mut KernelApp, _window: AppWindowId) -> EmptyIdleState {
    EmptyIdleState
}

fn view(_cx: &mut ElementContext<'_, KernelApp>, _st: &mut EmptyIdleState) -> ViewElements {
    // Intentionally empty: this demo is used as a baseline for process/resource footprint.
    ViewElements::default()
}
