pub const SOURCE: &str = include_str!("test_results_demo.rs");

// region: example
use fret_ui_ai as ui_ai;
use fret_ui_shadcn::prelude::*;

pub fn render<H: UiHost + 'static>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    let summary = ui_ai::TestResultsSummaryData::new(12, 2, 1, 15).duration_ms(3245);

    let auth_suite = ui_ai::TestSuite::new(
        ui_ai::TestSuiteName::new("Authentication", ui_ai::TestStatusKind::Passed)
            .stats(3, 0, 0)
            .test_id("ui-ai-test-suite-1-trigger"),
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
        ])
        .test_id("ui-ai-test-suite-1-content"),
    )
    .default_open(true)
    .into_element(cx);

    let user_api_suite = ui_ai::TestSuite::new(
        ui_ai::TestSuiteName::new("User API", ui_ai::TestStatusKind::Failed).stats(1, 1, 1),
        ui_ai::TestSuiteContent::new([
            ui_ai::Test::new("should create new user", ui_ai::TestStatusKind::Passed)
                .duration_ms(120)
                .into_element(cx),
            ui_ai::Test::new(
                "should update user profile",
                ui_ai::TestStatusKind::Failed,
            )
            .duration_ms(85)
            .details([
                ui_ai::TestError::new([
                    ui_ai::TestErrorMessage::new("Expected status 200 but received 500")
                        .into_element(cx),
                    ui_ai::TestErrorStack::new(
                        "  at Object.<anonymous> (src/user.test.ts:45:12)\n  at Promise.then.completed (node_modules/jest-circus/build/utils.js:391:28)",
                    )
                    .into_element(cx),
                ])
                .into_element(cx),
            ])
            .into_element(cx),
            ui_ai::Test::new("should delete user", ui_ai::TestStatusKind::Skipped)
                .into_element(cx),
        ]),
    )
    .default_open(true)
    .into_element(cx);

    let database_suite = ui_ai::TestSuite::new(
        ui_ai::TestSuiteName::new("Database", ui_ai::TestStatusKind::Failed).stats(1, 1, 0),
        ui_ai::TestSuiteContent::new([
            ui_ai::Test::new("should connect to database", ui_ai::TestStatusKind::Passed)
                .duration_ms(200)
                .into_element(cx),
            ui_ai::Test::new(
                "should handle connection timeout",
                ui_ai::TestStatusKind::Failed,
            )
            .duration_ms(5000)
            .details([ui_ai::TestError::new([ui_ai::TestErrorMessage::new(
                "Connection timed out after 5000ms",
            )
            .into_element(cx)])
            .into_element(cx)])
            .into_element(cx),
        ]),
    )
    .into_element(cx);

    let content = ui_ai::TestResultsContent::new([
        ui_ai::TestResultsProgress::new(summary.clone())
            .test_id("ui-ai-test-results-progress")
            .into_element(cx),
        auth_suite,
        user_api_suite,
        database_suite,
    ])
    .into_element(cx);

    ui_ai::TestResults::new()
        .summary(summary.clone())
        .children([
            ui_ai::TestResultsHeader::new([
                ui_ai::TestResultsSummary::new(summary.clone()).into_element(cx),
                ui_ai::TestResultsDuration::new(summary.clone()).into_element(cx),
            ])
            .into_element(cx),
            content,
        ])
        .test_id_root("ui-ai-test-results-root")
        .into_element(cx)
}
// endregion: example
