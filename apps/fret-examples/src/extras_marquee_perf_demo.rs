use std::sync::Arc;

use fret::advanced::KernelApp;
use fret::advanced::driver::{ViewElements, ui_app_with_hooks};
use fret::advanced::text;
use fret_core::{AppWindowId, Px};
use fret_ui::ElementContext;
use fret_ui::element::AnyElement;
use fret_ui_kit::{LayoutRefinement, Space, ui};
use fret_ui_shadcn::facade as shadcn;

struct ExtrasMarqueePerfState;

fn marquee_perf_title_text(
    cx: &mut ElementContext<'_, KernelApp>,
    text: impl Into<Arc<str>>,
) -> AnyElement {
    text::section_chrome_label(cx, text)
}

pub fn run() -> anyhow::Result<()> {
    ui_app_with_hooks("extras-marquee-perf-demo", init_window, view, |d| d)
        .with_default_diagnostics()
        .with_main_window("extras_marquee_perf_demo", (1280.0, 720.0))
        .run()
        .map_err(anyhow::Error::from)
}

fn init_window(_app: &mut KernelApp, _window: AppWindowId) -> ExtrasMarqueePerfState {
    ExtrasMarqueePerfState
}

fn view(cx: &mut ElementContext<'_, KernelApp>, _st: &mut ExtrasMarqueePerfState) -> ViewElements {
    let marquee = shadcn::extras::Marquee::new([
        "Alpha", "Beta", "Gamma", "Delta", "Epsilon", "Zeta", "Eta", "Theta",
    ])
    .speed_px_per_frame(Px(1.0))
    .track_gap(Space::N6)
    .item_gap(Space::N3)
    .into_element(cx);

    let content = ui::v_flex(|cx| {
        [
            marquee_perf_title_text(cx, "Marquee perf probe (extras)"),
            marquee,
        ]
    })
    .gap(Space::N4)
    .layout(
        LayoutRefinement::default()
            .w_full()
            .mx(Space::N8)
            .my(Space::N8),
    )
    .into_element(cx);

    vec![content].into()
}
