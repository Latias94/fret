use super::super::super::super::*;

pub(in crate::ui) fn preview_ai_test_results_demo(
    cx: &mut ElementContext<'_, App>,
    _theme: &Theme,
) -> Vec<AnyElement> {
    use fret_ui_kit::declarative::stack;
    use fret_ui_kit::{LayoutRefinement, Space};

    let suite = ui_ai::TestSuite::new(
        ui_ai::TestSuiteName::new("Core invariants", ui_ai::TestStatusKind::Passed)
            .stats(12, 0, 1)
            .test_id("ui-ai-test-suite-1-trigger"),
        ui_ai::TestSuiteContent::new([ui_ai::Test::new(
            "selection_extends_to_word_boundary",
            ui_ai::TestStatusKind::Passed,
        )
        .duration_ms(12)
        .into_element(cx)])
        .test_id("ui-ai-test-suite-1-content"),
    )
    .into_element(cx);

    let root = ui_ai::TestResults::new()
        .summary(ui_ai::TestResultsSummaryData::new(12, 0, 1, 13).duration_ms(1234))
        .children([suite])
        .test_id_root("ui-ai-test-results-root")
        .into_element(cx);

    vec![stack::vstack(
        cx,
        stack::VStackProps::default()
            .layout(LayoutRefinement::default().w_full().min_w_0())
            .gap(Space::N4),
        move |cx| {
            vec![
                cx.text("Test Results (AI Elements)"),
                cx.text("Expand a suite to reveal its tests."),
                root,
            ]
        },
    )]
}
