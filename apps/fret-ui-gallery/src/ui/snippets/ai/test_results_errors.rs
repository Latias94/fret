pub const SOURCE: &str = include_str!("test_results_errors.rs");

// region: example
use fret_ui_ai as ui_ai;
use fret_ui_shadcn::prelude::*;

pub fn render<H: UiHost + 'static>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    let summary = ui_ai::TestResultsSummaryData::new(2, 1, 0, 3).duration_ms(5420);

    let suite = ui_ai::TestSuite::new(
        ui_ai::TestSuiteName::new("Database", ui_ai::TestStatusKind::Failed).stats(2, 1, 0),
        ui_ai::TestSuiteContent::new([
            ui_ai::Test::new("should connect to database", ui_ai::TestStatusKind::Passed)
                .duration_ms(200)
                .into_element(cx),
            ui_ai::Test::new("should fetch active sessions", ui_ai::TestStatusKind::Passed)
                .duration_ms(220)
                .into_element(cx),
            ui_ai::Test::new(
                "should handle connection timeout",
                ui_ai::TestStatusKind::Failed,
            )
            .duration_ms(5000)
            .details([
                ui_ai::TestError::new([
                    ui_ai::TestErrorMessage::new("Connection timed out after 5000ms")
                        .into_element(cx),
                    ui_ai::TestErrorStack::new(
                        "  at connect (src/db/client.ts:88:14)\n  at processTicksAndRejections (node:internal/process/task_queues:95:5)",
                    )
                    .into_element(cx),
                ])
                .into_element(cx),
            ])
            .into_element(cx),
        ]),
    )
    .default_open(true)
    .into_element(cx);

    ui_ai::TestResults::new()
        .summary(summary.clone())
        .children([
            ui_ai::TestResultsHeader::new([
                ui_ai::TestResultsSummary::new(summary.clone()).into_element(cx),
                ui_ai::TestResultsDuration::new(summary.clone()).into_element(cx),
            ])
            .into_element(cx),
            ui_ai::TestResultsContent::new([suite]).into_element(cx),
        ])
        .into_element(cx)
}
// endregion: example
