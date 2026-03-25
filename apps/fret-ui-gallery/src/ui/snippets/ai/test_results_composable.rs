pub const SOURCE: &str = include_str!("test_results_composable.rs");

// region: example
use fret::{UiChild, UiCx};
use fret_core::{Edges, Px};
use fret_ui::Theme;
use fret_ui::element::AnyElement;
use fret_ui_ai as ui_ai;
use fret_ui_kit::declarative::style as decl_style;
use fret_ui_kit::{ChromeRefinement, LayoutRefinement, Space};

fn progress_section(cx: &mut UiCx<'_>) -> AnyElement {
    let theme = Theme::global(&*cx.app).clone();
    let progress = ui_ai::TestResultsProgress::from_context()
        .children([
            cx.text("12 of 15 checks are healthy"),
            cx.text("2 failures still need follow-up"),
        ])
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

    let auth_suite = ui_ai::TestSuite::named("Authentication", ui_ai::TestStatusKind::Failed)
        .trigger(
            ui_ai::TestSuiteName::from_context().children([
                cx.text("Authentication"),
                ui_ai::TestSuiteStats::new(2, 1, 0)
                    .children([cx.text("2 pass / 1 fail")])
                    .into_element(cx),
            ]),
        )
        .content(ui_ai::TestSuiteContent::new([
            ui_ai::Test::new(
                "should reject stale refresh tokens",
                ui_ai::TestStatusKind::Failed,
            )
            .duration_ms(85)
            .details([ui_ai::TestError::new([
                ui_ai::TestErrorMessage::new("Expected 401 but received 200").into_element(cx),
                ui_ai::TestErrorStack::new(
                    "  at auth.refresh.test.ts:48:19\n  at jest-circus/build/utils.js:391:28",
                )
                .into_element(cx),
            ])
            .into_element(cx)])
            .into_element_with_children(cx, |cx| {
                vec![
                    ui_ai::TestStatus::from_context()
                        .children([cx.text("FAIL")])
                        .into_element(cx),
                    ui_ai::TestName::from_context()
                        .children([cx.text("should reject stale refresh tokens")])
                        .into_element(cx),
                    ui_ai::TestDuration::from_context()
                        .children([cx.text("85ms cold cache")])
                        .into_element(cx),
                ]
            }),
            ui_ai::Test::new(
                "should rotate keys after password reset",
                ui_ai::TestStatusKind::Passed,
            )
            .duration_ms(41)
            .into_element_with_children(cx, |cx| {
                vec![
                    ui_ai::TestStatus::from_context()
                        .children([cx.text("PASS")])
                        .into_element(cx),
                    ui_ai::TestName::from_context()
                        .children([cx.text("should rotate keys after password reset")])
                        .into_element(cx),
                    ui_ai::TestDuration::from_context()
                        .children([cx.text("41ms warm path")])
                        .into_element(cx),
                ]
            }),
        ]))
        .default_open(true)
        .into_element(cx);

    ui_ai::TestResults::new()
        .summary(summary)
        .into_element_with_children(cx, |cx| {
            vec![
                ui_ai::TestResultsHeader::new([
                    ui_ai::TestResultsSummary::from_context()
                        .children([
                            cx.text("12 passing"),
                            cx.text("2 failing"),
                            cx.text("1 skipped"),
                        ])
                        .into_element(cx),
                    ui_ai::TestResultsDuration::from_context()
                        .children([cx.text("3.25s wall time")])
                        .into_element(cx),
                ])
                .into_element(cx),
                progress_section(cx),
                ui_ai::TestResultsContent::new([auth_suite]).into_element(cx),
            ]
        })
}
// endregion: example
