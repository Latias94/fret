use fret::app::prelude::*;

struct EmptyIdleView;

pub fn run() -> anyhow::Result<()> {
    FretApp::new("empty-idle-demo")
        .window("empty_idle_demo", (520.0, 240.0))
        .setup(fret_bootstrap::install_default_i18n_backend)
        .view::<EmptyIdleView>()?
        .run()
        .map_err(anyhow::Error::from)
}

impl View for EmptyIdleView {
    fn init(_app: &mut App, _window: WindowId) -> Self {
        Self
    }

    fn render(&mut self, _cx: &mut AppUi<'_, '_>) -> Ui {
        // Intentionally empty: this demo is used as a baseline for process/resource footprint.
        Vec::new().into()
    }
}
