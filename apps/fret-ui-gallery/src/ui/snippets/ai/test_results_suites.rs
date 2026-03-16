pub const SOURCE: &str = include_str!("test_results_suites.rs");

// region: example
use fret::{UiChild, UiCx};
use fret_ui_ai as ui_ai;
use fret_ui_shadcn::prelude::*;

pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    let summary = ui_ai::TestResultsSummaryData::new(3, 0, 0, 3).duration_ms(150);
    let test_parts = |cx: &mut UiCx<'_>| {
        ui::children![
            cx;
            ui_ai::TestStatus::from_context(),
            ui_ai::TestName::from_context(),
            ui_ai::TestDuration::from_context(),
        ]
    };

    let auth_suite = ui_ai::TestSuite::named("Auth", ui_ai::TestStatusKind::Passed)
        .trigger(ui_ai::TestSuiteName::from_context())
        .content(ui_ai::TestSuiteContent::new([
            ui_ai::Test::new("should login", ui_ai::TestStatusKind::Passed)
                .duration_ms(45)
                .children(test_parts(cx))
                .into_element(cx),
            ui_ai::Test::new("should logout", ui_ai::TestStatusKind::Passed)
                .duration_ms(32)
                .children(test_parts(cx))
                .into_element(cx),
            ui_ai::Test::new("should refresh token", ui_ai::TestStatusKind::Passed)
                .duration_ms(73)
                .children(test_parts(cx))
                .into_element(cx),
        ]))
        .into_element(cx);

    ui_ai::TestResults::new()
        .summary(summary)
        .children([
            ui_ai::TestResultsHeader::new([
                ui_ai::TestResultsSummary::from_context().into_element(cx)
            ])
            .into_element(cx),
            ui_ai::TestResultsContent::new([auth_suite]).into_element(cx),
        ])
        .into_element(cx)
}
// endregion: example
