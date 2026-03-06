pub const SOURCE: &str = include_str!("test_results_suites.rs");

// region: example
use fret_ui_ai as ui_ai;
use fret_ui_shadcn::prelude::*;

pub fn render<H: UiHost + 'static>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    let summary = ui_ai::TestResultsSummaryData::new(5, 1, 1, 7).duration_ms(1480);

    let auth_suite = ui_ai::TestSuite::new(
        ui_ai::TestSuiteName::new("Authentication", ui_ai::TestStatusKind::Passed).stats(3, 0, 0),
        ui_ai::TestSuiteContent::new([
            ui_ai::Test::new(
                "should login with valid credentials",
                ui_ai::TestStatusKind::Passed,
            )
            .duration_ms(45)
            .into_element(cx),
            ui_ai::Test::new(
                "should reject invalid password",
                ui_ai::TestStatusKind::Passed,
            )
            .duration_ms(32)
            .into_element(cx),
            ui_ai::Test::new(
                "should handle expired tokens",
                ui_ai::TestStatusKind::Passed,
            )
            .duration_ms(28)
            .into_element(cx),
        ]),
    )
    .default_open(true)
    .into_element(cx);

    let user_api_suite = ui_ai::TestSuite::new(
        ui_ai::TestSuiteName::new("User API", ui_ai::TestStatusKind::Failed).stats(2, 1, 1),
        ui_ai::TestSuiteContent::new([
            ui_ai::Test::new("should create new user", ui_ai::TestStatusKind::Passed)
                .duration_ms(120)
                .into_element(cx),
            ui_ai::Test::new("should update user profile", ui_ai::TestStatusKind::Failed)
                .duration_ms(85)
                .into_element(cx),
            ui_ai::Test::new("should delete user", ui_ai::TestStatusKind::Skipped).into_element(cx),
            ui_ai::Test::new("should list users", ui_ai::TestStatusKind::Passed)
                .duration_ms(51)
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
            ui_ai::TestResultsContent::new([auth_suite, user_api_suite]).into_element(cx),
        ])
        .into_element(cx)
}
// endregion: example
