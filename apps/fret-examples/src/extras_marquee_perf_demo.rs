use fret_app::App;
use fret_bootstrap::ui_app_with_hooks;
use fret_core::{AppWindowId, Px};
use fret_ui::ElementContext;
use fret_ui_kit::declarative::stack;
use fret_ui_kit::{LayoutRefinement, Space, ui};

struct ExtrasMarqueePerfState;

pub fn run() -> anyhow::Result<()> {
    ui_app_with_hooks("extras-marquee-perf-demo", init_window, view, |d| d)
        .with_default_diagnostics()
        .with_main_window("extras_marquee_perf_demo", (1280.0, 720.0))
        .run()
        .map_err(anyhow::Error::from)
}

fn init_window(_app: &mut App, _window: AppWindowId) -> ExtrasMarqueePerfState {
    ExtrasMarqueePerfState
}

fn view(
    cx: &mut ElementContext<'_, App>,
    _st: &mut ExtrasMarqueePerfState,
) -> fret_bootstrap::ui_app_driver::ViewElements {
    let marquee = fret_ui_shadcn::extras::Marquee::new([
        "Alpha", "Beta", "Gamma", "Delta", "Epsilon", "Zeta", "Eta", "Theta",
    ])
    .speed_px_per_frame(Px(1.0))
    .track_gap(Space::N6)
    .item_gap(Space::N3)
    .into_element(cx);

    let content = stack::vstack(
        cx,
        stack::VStackProps::default().gap_y(Space::N4).layout(
            LayoutRefinement::default()
                .w_full()
                .mx(Space::N8)
                .my(Space::N8),
        ),
        |cx| {
            vec![
                ui::text(cx, "Marquee perf probe (extras)")
                    .font_semibold()
                    .into_element(cx),
                marquee,
            ]
        },
    );

    vec![content].into()
}
