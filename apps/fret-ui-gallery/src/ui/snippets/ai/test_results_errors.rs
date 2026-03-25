pub const SOURCE: &str = include_str!("test_results_errors.rs");

// region: example
use fret::{UiChild, UiCx};
use fret_ui_ai as ui_ai;
use fret_ui_shadcn::prelude::*;

pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    let summary = ui_ai::TestResultsSummaryData::new(1, 1, 0, 2).duration_ms(130);
    let test_parts = |cx: &mut UiCx<'_>| {
        ui::children![
            cx;
            ui_ai::TestStatus::from_context().into_element(cx),
            ui_ai::TestName::from_context().into_element(cx),
            ui_ai::TestDuration::from_context().into_element(cx),
        ]
    };

    let suite = ui_ai::TestSuite::named("API", ui_ai::TestStatusKind::Failed)
        .trigger(ui_ai::TestSuiteName::from_context())
        .content(ui_ai::TestSuiteContent::new([
            ui_ai::Test::new("should fetch data", ui_ai::TestStatusKind::Passed)
                .duration_ms(45)
                .into_element_with_children(cx, test_parts),
            ui_ai::Test::new("should update", ui_ai::TestStatusKind::Failed)
                .duration_ms(85)
                .details([
                    ui_ai::TestError::new([
                        ui_ai::TestErrorMessage::new("Expected 200, got 500").into_element(cx),
                        ui_ai::TestErrorStack::new(
                            "    at Object.<anonymous> (/app/src/api.test.ts:45:12)\n    at Module._compile (node:internal/modules/cjs/loader:1369:14)\n    at Module._extensions..js (node:internal/modules/cjs/loader:1427:10)",
                        )
                        .into_element(cx),
                    ])
                    .into_element(cx),
                ])
                .into_element_with_children(cx, test_parts),
        ]))
    .default_open(true)
    .into_element(cx);

    ui_ai::TestResults::new()
        .summary(summary)
        .into_element_with_children(cx, |cx| {
            vec![
                ui_ai::TestResultsHeader::new([
                    ui_ai::TestResultsSummary::from_context().into_element(cx)
                ])
                .into_element(cx),
                ui_ai::TestResultsContent::new([suite]).into_element(cx),
            ]
        })
}
// endregion: example
