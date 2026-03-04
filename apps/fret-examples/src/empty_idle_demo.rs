use fret_app::App;
use fret_core::AppWindowId;
use fret_ui::ElementContext;

struct EmptyIdleState;

pub fn run() -> anyhow::Result<()> {
    fret_bootstrap::ui_app("empty-idle-demo", init_window, view)
        .init_app(fret_bootstrap::install_default_i18n_backend)
        .with_main_window("empty_idle_demo", (520.0, 240.0))
        .run()
        .map_err(anyhow::Error::from)
}

fn init_window(_app: &mut App, _window: AppWindowId) -> EmptyIdleState {
    EmptyIdleState
}

fn view(_cx: &mut ElementContext<'_, App>, _st: &mut EmptyIdleState) -> fret_ui::element::Elements {
    // Intentionally empty: this demo is used as a baseline for process/resource footprint.
    fret_ui::element::Elements::default()
}
