pub const SOURCE: &str = include_str!("test_results_demo.rs");

// region: example
use fret::{UiChild, UiCx};
use fret_core::{Edges, Px};
use fret_ui_ai as ui_ai;
use fret_ui_kit::declarative::style as decl_style;
use fret_ui_kit::{ChromeRefinement, LayoutRefinement, Space};
use fret_ui_shadcn::prelude::*;

fn progress_section(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    let theme = Theme::global(&*cx.app).clone();
    let progress = ui_ai::TestResultsProgress::from_context()
        .test_id("ui-ai-test-results-progress")
        .into_element(cx);

    let mut props = decl_style::container_props(
        &theme,
        ChromeRefinement::default().px(Space::N4).py(Space::N3),
        LayoutRefinement::default().w_full().min_w_0(),
    );
    props.border = Edges {
        top: Px(0.0),
        right: Px(0.0),
        bottom: Px(1.0),
        left: Px(0.0),
    };
    props.border_color = Some(theme.color_token("border"));

    cx.container(props, move |_cx| vec![progress])
}

pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    let summary = ui_ai::TestResultsSummaryData::new(12, 2, 1, 15).duration_ms(3245);
    let test_parts = |cx: &mut UiCx<'_>| {
        ui::children![
            cx;
            ui_ai::TestStatus::from_context(),
            ui_ai::TestName::from_context(),
            ui_ai::TestDuration::from_context(),
        ]
    };

    let auth_suite = ui_ai::TestSuite::named("Authentication", ui_ai::TestStatusKind::Passed)
        .trigger(ui_ai::TestSuiteName::from_context().test_id("ui-ai-test-suite-1-trigger"))
        .content(
            ui_ai::TestSuiteContent::new([
                ui_ai::Test::new(
                    "should login with valid credentials",
                    ui_ai::TestStatusKind::Passed,
                )
                .duration_ms(45)
                .children(test_parts(cx))
                .into_element(cx),
                ui_ai::Test::new(
                    "should reject invalid password",
                    ui_ai::TestStatusKind::Passed,
                )
                .duration_ms(32)
                .children(test_parts(cx))
                .into_element(cx),
                ui_ai::Test::new(
                    "should handle expired tokens",
                    ui_ai::TestStatusKind::Passed,
                )
                .duration_ms(28)
                .children(test_parts(cx))
                .into_element(cx),
            ])
            .test_id("ui-ai-test-suite-1-content"),
        )
        .default_open(true)
        .into_element(cx);

    let user_api_suite = ui_ai::TestSuite::named("User API", ui_ai::TestStatusKind::Failed)
        .trigger(ui_ai::TestSuiteName::from_context())
        .content(ui_ai::TestSuiteContent::new([
            ui_ai::Test::new("should create new user", ui_ai::TestStatusKind::Passed)
                .duration_ms(120)
                .children(test_parts(cx))
                .into_element(cx),
            ui_ai::Test::new(
                "should update user profile",
                ui_ai::TestStatusKind::Failed,
            )
            .duration_ms(85)
            .children(test_parts(cx))
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
                .children(test_parts(cx))
                .into_element(cx),
        ]))
    .default_open(true)
    .into_element(cx);

    let database_suite = ui_ai::TestSuite::named("Database", ui_ai::TestStatusKind::Failed)
        .trigger(ui_ai::TestSuiteName::from_context().test_id("ui-ai-test-suite-database-trigger"))
        .content(
            ui_ai::TestSuiteContent::new([
                ui_ai::Test::new("should connect to database", ui_ai::TestStatusKind::Passed)
                    .duration_ms(200)
                    .children(test_parts(cx))
                    .into_element(cx),
                ui_ai::Test::new(
                    "should handle connection timeout",
                    ui_ai::TestStatusKind::Failed,
                )
                .duration_ms(5000)
                .children(test_parts(cx))
                .details([ui_ai::TestError::new([ui_ai::TestErrorMessage::new(
                    "Connection timed out after 5000ms",
                )
                .into_element(cx)])
                .into_element(cx)])
                .into_element(cx),
            ])
            .test_id("ui-ai-test-suite-database-content"),
        )
        .into_element(cx);

    let content = ui_ai::TestResultsContent::new([auth_suite, user_api_suite, database_suite])
        .into_element(cx);

    ui_ai::TestResults::new()
        .summary(summary)
        .children([
            ui_ai::TestResultsHeader::new([
                ui_ai::TestResultsSummary::from_context().into_element(cx),
                ui_ai::TestResultsDuration::from_context().into_element(cx),
            ])
            .into_element(cx),
            progress_section(cx).into_element(cx),
            content,
        ])
        .test_id_root("ui-ai-test-results-root")
        .into_element(cx)
}
// endregion: example
