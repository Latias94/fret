pub const SOURCE: &str = include_str!("test_results_large_demo.rs");

// region: example
use fret::{UiChild, UiCx};
use fret_ui::Invalidation;
use fret_ui_ai as ui_ai;
use fret_ui_kit::ui;
use fret_ui_kit::{LayoutRefinement, Space};
use fret_ui_shadcn::prelude::*;
use std::sync::Arc;

pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    let activated = cx.local_model_keyed("activated", || false);

    let activated_now = cx
        .get_model_copied(&activated, Invalidation::Layout)
        .unwrap_or(false);

    let on_activate: ui_ai::OnTestActivate = Arc::new({
        let activated = activated.clone();
        move |host, action_cx, _name| {
            let _ = host.models_mut().update(&activated, |v| *v = true);
            host.notify(action_cx);
        }
    });

    let mut tests: Vec<AnyElement> = Vec::new();
    for i in 0..500u32 {
        let id = format!("ui-ai-test-results-large-test-{i:04}");
        let name = Arc::<str>::from(format!("test_{i:04}"));
        let mut row = ui_ai::Test::new(name.clone(), ui_ai::TestStatusKind::Passed)
            .duration_ms(3)
            .test_id(id.clone());
        if i == 350 {
            row = row.on_activate(on_activate.clone());
        }
        tests.push(row.into_element(cx));
    }

    let marker = activated_now
        .then(|| {
            cx.text("")
                .test_id("ui-ai-test-results-large-activated-marker")
        })
        .unwrap_or_else(|| cx.text(""));

    let suite = ui_ai::TestSuite::new(
        ui_ai::TestSuiteName::new("Large suite", ui_ai::TestStatusKind::Passed)
            .stats(500, 0, 0)
            .test_id("ui-ai-test-results-large-suite-trigger"),
        ui_ai::TestSuiteContent::new(tests).test_id("ui-ai-test-results-large-suite-content"),
    )
    .default_open(true)
    .into_element(cx);

    let root = ui_ai::TestResults::new()
        .summary(ui_ai::TestResultsSummaryData::new(500, 0, 0, 500).duration_ms(888))
        .children([suite, marker])
        .test_id_root("ui-ai-test-results-large-root")
        .into_element(cx);

    ui::v_flex(move |cx| {
        vec![
            cx.text("Test Results Large (AI Elements)"),
            cx.text("Scroll the page and click a deep row to set a marker."),
            root,
        ]
    })
    .layout(LayoutRefinement::default().w_full().min_w_0())
    .gap(Space::N4)
    .into_element(cx)
}
// endregion: example
