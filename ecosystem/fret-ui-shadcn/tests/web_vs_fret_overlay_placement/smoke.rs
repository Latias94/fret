use super::*;

#[test]
fn web_vs_fret_overlay_placement_smoke_cases_match_web_fixtures() {
    let raw = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/overlay_placement_smoke_cases_v1.json"
    ));
    let suite: FixtureSuite<OverlayPlacementSmokeCase> =
        serde_json::from_str(raw).expect("overlay placement smoke fixture parse");
    assert_eq!(suite.schema_version, 1);
    assert!(!suite.cases.is_empty());

    for case in suite.cases {
        eprintln!("overlay placement smoke case={}", case.id);
        match case.recipe {
            OverlayPlacementSmokeRecipe::PopoverDemo => {
                assert_overlay_placement_matches(
                    &case.web_name,
                    Some("dialog"),
                    |cx, open| {
                        fret_ui_shadcn::Popover::new(open.clone()).into_element(
                            cx,
                            |cx| {
                                fret_ui_shadcn::Button::new("Open popover")
                                    .variant(fret_ui_shadcn::ButtonVariant::Outline)
                                    .into_element(cx)
                            },
                            |cx| {
                                let content = fret_ui_shadcn::PopoverContent::new(Vec::new())
                                    .refine_layout(
                                        fret_ui_kit::LayoutRefinement::default()
                                            .w_px(Px(320.0))
                                            .h_px(Px(245.33334)),
                                    )
                                    .into_element(cx);
                                if case.web_name == "popover-demo"
                                    && std::env::var("FRET_DEBUG_OVERLAY_PLACEMENT")
                                        .ok()
                                        .is_some_and(|v| v == "1")
                                {
                                    eprintln!(
                                        "popover-demo content container px size={:?}",
                                        first_container_px_size(&content)
                                    );
                                }
                                content
                            },
                        )
                    },
                    SemanticsRole::Button,
                    Some("Open popover"),
                    SemanticsRole::Dialog,
                );
            }
            OverlayPlacementSmokeRecipe::DatePickerDemo => {
                assert_overlay_placement_matches(
                    &case.web_name,
                    Some("dialog"),
                    |cx, open| {
                        use fret_ui_headless::calendar::CalendarMonth;
                        use fret_ui_kit::{LayoutRefinement, MetricRef};
                        use time::Month;

                        let month: Model<CalendarMonth> = cx
                            .app
                            .models_mut()
                            .insert(CalendarMonth::new(2026, Month::January));
                        let selected: Model<Option<time::Date>> = cx.app.models_mut().insert(None);

                        fret_ui_shadcn::DatePicker::new(open.clone(), month, selected)
                            .refine_layout(
                                LayoutRefinement::default().w_px(MetricRef::Px(Px(240.0))),
                            )
                            .into_element(cx)
                    },
                    SemanticsRole::Button,
                    Some("Pick a date"),
                    SemanticsRole::Dialog,
                );
            }
        }
    }
}
