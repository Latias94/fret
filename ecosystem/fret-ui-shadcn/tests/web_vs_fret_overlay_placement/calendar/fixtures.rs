use super::*;
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "snake_case")]
enum CalendarRecipe {
    Calendar22OpenOverlayPlacement,
    Calendar23OpenOverlayPlacement,
    Calendar24OpenOverlayPlacement,
    Calendar25OpenOverlayPlacement,
    Calendar26OpenOverlayPlacement,
    Calendar27OpenOverlayPlacement,
    Calendar28OpenOverlayPlacement,
    Calendar29OpenOverlayPlacement,
    Calendar30OpenOverlayPlacement,
    Calendar32OpenDrawerInsets,
}

#[derive(Debug, Clone, Deserialize)]
struct CalendarCase {
    id: String,
    web_name: String,
    recipe: CalendarRecipe,
}

fn build_calendar_22_open_overlay(
    cx: &mut ElementContext<'_, App>,
    open: &Model<bool>,
) -> AnyElement {
    use fret_ui_headless::calendar::CalendarMonth;
    use fret_ui_kit::{LayoutRefinement, MetricRef};
    use fret_ui_shadcn::{Button, ButtonVariant, PopoverAlign};
    use time::Month;

    let trigger = Button::new("Select date")
        .variant(ButtonVariant::Outline)
        .refine_layout(
            LayoutRefinement::default()
                .w_px(MetricRef::Px(Px(192.0)))
                .h_px(MetricRef::Px(Px(36.0))),
        );

    let label = fret_ui_shadcn::Label::new("Date of birth").into_element(cx);
    let popover = fret_ui_shadcn::Popover::new(open.clone())
        .align(PopoverAlign::Start)
        .into_element(
            cx,
            |cx| trigger.into_element(cx),
            |cx| {
                let month: Model<CalendarMonth> = cx
                    .app
                    .models_mut()
                    .insert(CalendarMonth::new(2025, Month::June));
                let selected: Model<Option<time::Date>> = cx.app.models_mut().insert(None);
                let calendar = fret_ui_shadcn::Calendar::new(month, selected).into_element(cx);

                fret_ui_shadcn::PopoverContent::new([calendar])
                    .refine_layout(LayoutRefinement::default().w_px(MetricRef::Px(Px(249.33334))))
                    .into_element(cx)
            },
        );

    stack::vstack(
        cx,
        stack::VStackProps::default().gap(Space::N3),
        move |_cx| vec![label, popover],
    )
}

fn build_calendar_23_open_overlay(
    cx: &mut ElementContext<'_, App>,
    open: &Model<bool>,
) -> AnyElement {
    use fret_ui_headless::calendar::CalendarMonth;
    use fret_ui_kit::{LayoutRefinement, MetricRef};
    use fret_ui_shadcn::{Button, ButtonVariant, PopoverAlign};
    use time::Month;

    let trigger = Button::new("Select date")
        .variant(ButtonVariant::Outline)
        .refine_layout(
            LayoutRefinement::default()
                .w_px(MetricRef::Px(Px(224.0)))
                .h_px(MetricRef::Px(Px(36.0))),
        );

    let label = fret_ui_shadcn::Label::new("Select your stay").into_element(cx);
    let popover = fret_ui_shadcn::Popover::new(open.clone())
        .align(PopoverAlign::Start)
        .into_element(
            cx,
            |cx| trigger.into_element(cx),
            |cx| {
                let month: Model<CalendarMonth> = cx
                    .app
                    .models_mut()
                    .insert(CalendarMonth::new(2025, Month::June));
                let selected: Model<Option<time::Date>> = cx.app.models_mut().insert(None);
                let calendar = fret_ui_shadcn::Calendar::new(month, selected).into_element(cx);

                fret_ui_shadcn::PopoverContent::new([calendar])
                    .refine_layout(LayoutRefinement::default().w_px(MetricRef::Px(Px(249.33334))))
                    .into_element(cx)
            },
        );

    stack::vstack(
        cx,
        stack::VStackProps::default().gap(Space::N3),
        move |_cx| vec![label, popover],
    )
}

fn build_calendar_24_open_overlay(
    cx: &mut ElementContext<'_, App>,
    open: &Model<bool>,
) -> AnyElement {
    use fret_ui_headless::calendar::CalendarMonth;
    use fret_ui_kit::{LayoutRefinement, MetricRef};
    use fret_ui_shadcn::{Button, ButtonVariant, PopoverAlign};
    use time::Month;

    let trigger = Button::new("Select date")
        .variant(ButtonVariant::Outline)
        .refine_layout(
            LayoutRefinement::default()
                .w_px(MetricRef::Px(Px(128.0)))
                .h_px(MetricRef::Px(Px(36.0))),
        );

    let label = fret_ui_shadcn::Label::new("Date").into_element(cx);
    let popover = fret_ui_shadcn::Popover::new(open.clone())
        .align(PopoverAlign::Start)
        .into_element(
            cx,
            |cx| trigger.into_element(cx),
            |cx| {
                let month: Model<CalendarMonth> = cx
                    .app
                    .models_mut()
                    .insert(CalendarMonth::new(2025, Month::June));
                let selected: Model<Option<time::Date>> = cx.app.models_mut().insert(None);
                let calendar = fret_ui_shadcn::Calendar::new(month, selected).into_element(cx);

                fret_ui_shadcn::PopoverContent::new([calendar])
                    .refine_layout(LayoutRefinement::default().w_px(MetricRef::Px(Px(249.33334))))
                    .into_element(cx)
            },
        );

    stack::vstack(
        cx,
        stack::VStackProps::default().gap(Space::N3),
        move |_cx| vec![label, popover],
    )
}

fn build_calendar_25_open_overlay(
    cx: &mut ElementContext<'_, App>,
    open: &Model<bool>,
) -> AnyElement {
    use fret_ui_headless::calendar::CalendarMonth;
    use fret_ui_kit::{LayoutRefinement, MetricRef};
    use fret_ui_shadcn::{Button, ButtonVariant, PopoverAlign};
    use time::Month;

    let trigger = Button::new("Select date")
        .variant(ButtonVariant::Outline)
        .refine_layout(
            LayoutRefinement::default()
                .w_px(MetricRef::Px(Px(1440.0)))
                .h_px(MetricRef::Px(Px(36.0))),
        );

    let label = fret_ui_shadcn::Label::new("Date").into_element(cx);
    let popover = fret_ui_shadcn::Popover::new(open.clone())
        .align(PopoverAlign::Start)
        .into_element(
            cx,
            |cx| trigger.into_element(cx),
            |cx| {
                let month: Model<CalendarMonth> = cx
                    .app
                    .models_mut()
                    .insert(CalendarMonth::new(2025, Month::June));
                let selected: Model<Option<time::Date>> = cx.app.models_mut().insert(None);
                let calendar = fret_ui_shadcn::Calendar::new(month, selected).into_element(cx);

                fret_ui_shadcn::PopoverContent::new([calendar])
                    .refine_layout(LayoutRefinement::default().w_px(MetricRef::Px(Px(249.33334))))
                    .into_element(cx)
            },
        );

    stack::vstack(
        cx,
        stack::VStackProps::default().gap(Space::N3),
        move |_cx| vec![label, popover],
    )
}

fn build_calendar_26_open_overlay(
    cx: &mut ElementContext<'_, App>,
    open: &Model<bool>,
) -> AnyElement {
    use fret_ui_headless::calendar::CalendarMonth;
    use fret_ui_kit::{LayoutRefinement, MetricRef};
    use fret_ui_shadcn::{Button, ButtonVariant, PopoverAlign};
    use time::{Date, Month};

    let trigger = Button::new("Jun 01, 2025")
        .variant(ButtonVariant::Outline)
        .refine_layout(
            LayoutRefinement::default()
                .w_px(MetricRef::Px(Px(224.0)))
                .h_px(MetricRef::Px(Px(36.0))),
        );

    let label = fret_ui_shadcn::Label::new("Select your stay").into_element(cx);
    let popover = fret_ui_shadcn::Popover::new(open.clone())
        .align(PopoverAlign::Start)
        .into_element(
            cx,
            |cx| trigger.into_element(cx),
            |cx| {
                let month: Model<CalendarMonth> = cx
                    .app
                    .models_mut()
                    .insert(CalendarMonth::new(2025, Month::June));
                let selected: Model<Option<Date>> = cx.app.models_mut().insert(Some(
                    Date::from_calendar_date(2025, Month::June, 1).expect("valid date"),
                ));
                let calendar = fret_ui_shadcn::Calendar::new(month, selected).into_element(cx);

                fret_ui_shadcn::PopoverContent::new([calendar])
                    .refine_layout(LayoutRefinement::default().w_px(MetricRef::Px(Px(249.33334))))
                    .into_element(cx)
            },
        );

    stack::vstack(
        cx,
        stack::VStackProps::default().gap(Space::N3),
        move |_cx| vec![label, popover],
    )
}

fn build_calendar_27_open_overlay(
    cx: &mut ElementContext<'_, App>,
    open: &Model<bool>,
) -> AnyElement {
    use fret_ui_headless::calendar::{CalendarMonth, DateRangeSelection};
    use fret_ui_shadcn::{Button, ButtonVariant, PopoverAlign};
    use time::{Date, Month};

    let trigger = Button::new("6/5/2025 - 6/20/2025").variant(ButtonVariant::Outline);
    let popover = fret_ui_shadcn::Popover::new(open.clone())
        .align(PopoverAlign::End)
        .window_margin(Px(0.0))
        .into_element(
            cx,
            |cx| trigger.into_element(cx),
            |cx| {
                let month: Model<CalendarMonth> = cx
                    .app
                    .models_mut()
                    .insert(CalendarMonth::new(2025, Month::June));
                let selected: Model<DateRangeSelection> =
                    cx.app.models_mut().insert(DateRangeSelection {
                        from: Some(
                            Date::from_calendar_date(2025, Month::June, 5).expect("valid date"),
                        ),
                        to: Some(
                            Date::from_calendar_date(2025, Month::June, 20).expect("valid date"),
                        ),
                    });
                let calendar = fret_ui_shadcn::CalendarRange::new(month, selected).into_element(cx);
                fret_ui_shadcn::PopoverContent::new([calendar]).into_element(cx)
            },
        );

    stack::hstack(
        cx,
        stack::HStackProps::default()
            .layout(LayoutRefinement::default().w_full())
            .justify_end(),
        move |_cx| vec![popover],
    )
}

fn build_calendar_28_open_overlay(
    cx: &mut ElementContext<'_, App>,
    open: &Model<bool>,
) -> AnyElement {
    use fret_ui_headless::calendar::CalendarMonth;
    use fret_ui_shadcn::{Button, ButtonVariant, PopoverAlign};
    use time::{Date, Month};

    let trigger = Button::new("Select date")
        .variant(ButtonVariant::Ghost)
        .refine_layout(LayoutRefinement::default().mr(Space::N2));
    let popover = fret_ui_shadcn::Popover::new(open.clone())
        .align(PopoverAlign::End)
        .align_offset(Px(-8.0))
        .side_offset(Px(10.0))
        .window_margin(Px(0.0))
        .into_element(
            cx,
            |cx| trigger.into_element(cx),
            |cx| {
                let month: Model<CalendarMonth> = cx
                    .app
                    .models_mut()
                    .insert(CalendarMonth::new(2025, Month::June));
                let selected: Model<Option<Date>> = cx.app.models_mut().insert(Some(
                    Date::from_calendar_date(2025, Month::June, 1).expect("valid date"),
                ));
                let calendar = fret_ui_shadcn::Calendar::new(month, selected).into_element(cx);
                fret_ui_shadcn::PopoverContent::new([calendar]).into_element(cx)
            },
        );

    stack::hstack(
        cx,
        stack::HStackProps::default()
            .layout(LayoutRefinement::default().w_full())
            .justify_end(),
        move |_cx| vec![popover],
    )
}

fn build_calendar_29_open_overlay(
    cx: &mut ElementContext<'_, App>,
    open: &Model<bool>,
) -> AnyElement {
    use fret_ui_headless::calendar::CalendarMonth;
    use fret_ui_shadcn::{Button, ButtonVariant, PopoverAlign};
    use time::{Date, Month};

    let trigger = Button::new("Select date").variant(ButtonVariant::Ghost);
    let popover = fret_ui_shadcn::Popover::new(open.clone())
        .align(PopoverAlign::End)
        .window_margin(Px(0.0))
        .into_element(
            cx,
            |cx| trigger.into_element(cx),
            |cx| {
                let month: Model<CalendarMonth> = cx
                    .app
                    .models_mut()
                    .insert(CalendarMonth::new(2025, Month::June));
                let selected: Model<Option<Date>> = cx.app.models_mut().insert(Some(
                    Date::from_calendar_date(2025, Month::June, 3).expect("valid date"),
                ));
                let calendar = fret_ui_shadcn::Calendar::new(month, selected).into_element(cx);
                fret_ui_shadcn::PopoverContent::new([calendar]).into_element(cx)
            },
        );

    stack::hstack(
        cx,
        stack::HStackProps::default()
            .layout(LayoutRefinement::default().w_full())
            .justify_end(),
        move |_cx| vec![popover],
    )
}

fn build_calendar_30_open_overlay(
    cx: &mut ElementContext<'_, App>,
    open: &Model<bool>,
) -> AnyElement {
    use fret_ui_headless::calendar::{CalendarMonth, DateRangeSelection};
    use fret_ui_kit::{LayoutRefinement, MetricRef};
    use fret_ui_shadcn::{Button, ButtonVariant, PopoverAlign};
    use time::{Date, Month};

    let trigger = Button::new("Jun 4 - 10, 2025")
        .variant(ButtonVariant::Outline)
        .refine_layout(
            LayoutRefinement::default()
                .w_px(MetricRef::Px(Px(224.0)))
                .h_px(MetricRef::Px(Px(36.0))),
        );

    let label = fret_ui_shadcn::Label::new("Select your stay").into_element(cx);
    let popover = fret_ui_shadcn::Popover::new(open.clone())
        .align(PopoverAlign::Start)
        .into_element(
            cx,
            |cx| trigger.into_element(cx),
            |cx| {
                let month: Model<CalendarMonth> = cx
                    .app
                    .models_mut()
                    .insert(CalendarMonth::new(2025, Month::June));
                let selected: Model<DateRangeSelection> =
                    cx.app.models_mut().insert(DateRangeSelection {
                        from: Some(
                            Date::from_calendar_date(2025, Month::June, 4).expect("valid date"),
                        ),
                        to: Some(
                            Date::from_calendar_date(2025, Month::June, 10).expect("valid date"),
                        ),
                    });
                let calendar = fret_ui_shadcn::CalendarRange::new(month, selected).into_element(cx);

                fret_ui_shadcn::PopoverContent::new([calendar])
                    .refine_layout(LayoutRefinement::default().w_px(MetricRef::Px(Px(249.33334))))
                    .into_element(cx)
            },
        );

    stack::vstack(
        cx,
        stack::VStackProps::default().gap(Space::N3),
        move |_cx| vec![label, popover],
    )
}

fn assert_calendar_32_open_drawer_insets_match() {
    use fret_ui_headless::calendar::CalendarMonth;
    use fret_ui_shadcn::{Button, ButtonVariant, Drawer, DrawerContent, DrawerHeader};
    use time::Month;

    assert_viewport_anchored_overlay_placement_matches(
        "calendar-32",
        "dialog",
        SemanticsRole::Dialog,
        |cx, open| {
            Drawer::new(open.clone()).into_element(
                cx,
                |cx| {
                    Button::new("Select date")
                        .variant(ButtonVariant::Outline)
                        .into_element(cx)
                },
                |cx| {
                    let month: Model<CalendarMonth> = cx
                        .app
                        .models_mut()
                        .insert(CalendarMonth::new(2025, Month::June));
                    let selected: Model<Option<time::Date>> = cx.app.models_mut().insert(None);
                    let calendar = fret_ui_shadcn::Calendar::new(month, selected).into_element(cx);

                    DrawerContent::new(vec![DrawerHeader::new(vec![]).into_element(cx), calendar])
                        .into_element(cx)
                },
            )
        },
    );
}

#[test]
fn web_vs_fret_calendar_cases_match_web_fixtures() {
    let raw = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/overlay_placement_calendar_cases_v1.json"
    ));
    let suite: FixtureSuite<CalendarCase> =
        serde_json::from_str(raw).expect("calendar fixture parse");
    assert_eq!(suite.schema_version, 1);
    assert!(!suite.cases.is_empty());

    for case in suite.cases {
        eprintln!("calendar case={}", case.id);
        match case.recipe {
            CalendarRecipe::Calendar22OpenOverlayPlacement => {
                assert_overlay_placement_matches(
                    &case.web_name,
                    Some("dialog"),
                    |cx, open| build_calendar_22_open_overlay(cx, open),
                    SemanticsRole::Button,
                    Some("Select date"),
                    SemanticsRole::Dialog,
                );
            }
            CalendarRecipe::Calendar23OpenOverlayPlacement => {
                assert_overlay_placement_matches(
                    &case.web_name,
                    Some("dialog"),
                    |cx, open| build_calendar_23_open_overlay(cx, open),
                    SemanticsRole::Button,
                    Some("Select date"),
                    SemanticsRole::Dialog,
                );
            }
            CalendarRecipe::Calendar24OpenOverlayPlacement => {
                assert_overlay_placement_matches(
                    &case.web_name,
                    Some("dialog"),
                    |cx, open| build_calendar_24_open_overlay(cx, open),
                    SemanticsRole::Button,
                    Some("Select date"),
                    SemanticsRole::Dialog,
                );
            }
            CalendarRecipe::Calendar25OpenOverlayPlacement => {
                assert_overlay_placement_matches(
                    &case.web_name,
                    Some("dialog"),
                    |cx, open| build_calendar_25_open_overlay(cx, open),
                    SemanticsRole::Button,
                    Some("Select date"),
                    SemanticsRole::Dialog,
                );
            }
            CalendarRecipe::Calendar26OpenOverlayPlacement => {
                assert_overlay_placement_matches(
                    &case.web_name,
                    Some("dialog"),
                    |cx, open| build_calendar_26_open_overlay(cx, open),
                    SemanticsRole::Button,
                    Some("Jun 01, 2025"),
                    SemanticsRole::Dialog,
                );
            }
            CalendarRecipe::Calendar27OpenOverlayPlacement => {
                assert_overlay_placement_matches(
                    &case.web_name,
                    Some("dialog"),
                    |cx, open| build_calendar_27_open_overlay(cx, open),
                    SemanticsRole::Button,
                    None,
                    SemanticsRole::Dialog,
                );
            }
            CalendarRecipe::Calendar28OpenOverlayPlacement => {
                assert_overlay_placement_matches(
                    &case.web_name,
                    Some("dialog"),
                    |cx, open| build_calendar_28_open_overlay(cx, open),
                    SemanticsRole::Button,
                    None,
                    SemanticsRole::Dialog,
                );
            }
            CalendarRecipe::Calendar29OpenOverlayPlacement => {
                assert_overlay_placement_matches(
                    &case.web_name,
                    Some("dialog"),
                    |cx, open| build_calendar_29_open_overlay(cx, open),
                    SemanticsRole::Button,
                    None,
                    SemanticsRole::Dialog,
                );
            }
            CalendarRecipe::Calendar30OpenOverlayPlacement => {
                assert_overlay_placement_matches(
                    &case.web_name,
                    Some("dialog"),
                    |cx, open| build_calendar_30_open_overlay(cx, open),
                    SemanticsRole::Button,
                    Some("Jun 4 - 10, 2025"),
                    SemanticsRole::Dialog,
                );
            }
            CalendarRecipe::Calendar32OpenDrawerInsets => {
                assert_eq!(case.web_name, "calendar-32");
                assert_calendar_32_open_drawer_insets_match();
            }
        }
    }
}
