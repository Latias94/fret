use super::*;

#[test]
fn web_vs_fret_calendar_22_open_overlay_placement_matches() {
    assert_overlay_placement_matches(
        "calendar-22",
        Some("dialog"),
        |cx, open| {
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
                        let calendar =
                            fret_ui_shadcn::Calendar::new(month, selected).into_element(cx);

                        fret_ui_shadcn::PopoverContent::new([calendar])
                            .refine_layout(
                                LayoutRefinement::default().w_px(MetricRef::Px(Px(249.33334))),
                            )
                            .into_element(cx)
                    },
                );

            stack::vstack(
                cx,
                stack::VStackProps::default().gap(Space::N3),
                move |_cx| vec![label, popover],
            )
        },
        SemanticsRole::Button,
        Some("Select date"),
        SemanticsRole::Dialog,
    );
}
#[test]
fn web_vs_fret_calendar_22_open_overlay_placement_matches_mobile_tiny_viewport() {
    assert_overlay_placement_matches(
        "calendar-22.vp375x240",
        Some("dialog"),
        |cx, open| {
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
                        let calendar =
                            fret_ui_shadcn::Calendar::new(month, selected).into_element(cx);

                        fret_ui_shadcn::PopoverContent::new([calendar])
                            .refine_layout(
                                LayoutRefinement::default().w_px(MetricRef::Px(Px(249.33334))),
                            )
                            .into_element(cx)
                    },
                );

            stack::vstack(
                cx,
                stack::VStackProps::default().gap(Space::N3),
                move |_cx| vec![label, popover],
            )
        },
        SemanticsRole::Button,
        Some("Select date"),
        SemanticsRole::Dialog,
    );
}
#[test]
fn web_vs_fret_calendar_23_open_overlay_placement_matches() {
    assert_overlay_placement_matches(
        "calendar-23",
        Some("dialog"),
        |cx, open| {
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
                        let calendar =
                            fret_ui_shadcn::Calendar::new(month, selected).into_element(cx);

                        fret_ui_shadcn::PopoverContent::new([calendar])
                            .refine_layout(
                                LayoutRefinement::default().w_px(MetricRef::Px(Px(249.33334))),
                            )
                            .into_element(cx)
                    },
                );

            stack::vstack(
                cx,
                stack::VStackProps::default().gap(Space::N3),
                move |_cx| vec![label, popover],
            )
        },
        SemanticsRole::Button,
        Some("Select date"),
        SemanticsRole::Dialog,
    );
}
#[test]
fn web_vs_fret_calendar_23_open_overlay_placement_matches_mobile_tiny_viewport() {
    assert_overlay_placement_matches(
        "calendar-23.vp375x240",
        Some("dialog"),
        |cx, open| {
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
                        let calendar =
                            fret_ui_shadcn::Calendar::new(month, selected).into_element(cx);

                        fret_ui_shadcn::PopoverContent::new([calendar])
                            .refine_layout(
                                LayoutRefinement::default().w_px(MetricRef::Px(Px(249.33334))),
                            )
                            .into_element(cx)
                    },
                );

            stack::vstack(
                cx,
                stack::VStackProps::default().gap(Space::N3),
                move |_cx| vec![label, popover],
            )
        },
        SemanticsRole::Button,
        Some("Select date"),
        SemanticsRole::Dialog,
    );
}
#[test]
fn web_vs_fret_calendar_24_open_overlay_placement_matches() {
    assert_overlay_placement_matches(
        "calendar-24",
        Some("dialog"),
        |cx, open| {
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
                        let calendar =
                            fret_ui_shadcn::Calendar::new(month, selected).into_element(cx);

                        fret_ui_shadcn::PopoverContent::new([calendar])
                            .refine_layout(
                                LayoutRefinement::default().w_px(MetricRef::Px(Px(249.33334))),
                            )
                            .into_element(cx)
                    },
                );

            stack::vstack(
                cx,
                stack::VStackProps::default().gap(Space::N3),
                move |_cx| vec![label, popover],
            )
        },
        SemanticsRole::Button,
        Some("Select date"),
        SemanticsRole::Dialog,
    );
}
#[test]
fn web_vs_fret_calendar_25_open_overlay_placement_matches() {
    assert_overlay_placement_matches(
        "calendar-25",
        Some("dialog"),
        |cx, open| {
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
                        let calendar =
                            fret_ui_shadcn::Calendar::new(month, selected).into_element(cx);

                        fret_ui_shadcn::PopoverContent::new([calendar])
                            .refine_layout(
                                LayoutRefinement::default().w_px(MetricRef::Px(Px(249.33334))),
                            )
                            .into_element(cx)
                    },
                );

            stack::vstack(
                cx,
                stack::VStackProps::default().gap(Space::N3),
                move |_cx| vec![label, popover],
            )
        },
        SemanticsRole::Button,
        Some("Select date"),
        SemanticsRole::Dialog,
    );
}
#[test]
fn web_vs_fret_calendar_26_open_overlay_placement_matches() {
    assert_overlay_placement_matches(
        "calendar-26",
        Some("dialog"),
        |cx, open| {
            use fret_ui_headless::calendar::CalendarMonth;
            use fret_ui_kit::{LayoutRefinement, MetricRef};
            use fret_ui_shadcn::{Button, ButtonVariant, PopoverAlign};
            use time::{Date, Month};

            let trigger = Button::new("Jun 01, 2025")
                .variant(ButtonVariant::Outline)
                .refine_layout(
                    LayoutRefinement::default()
                        .w_px(MetricRef::Px(Px(154.66667)))
                        .h_px(MetricRef::Px(Px(36.0))),
                );

            let label = fret_ui_shadcn::Label::new("Check-in").into_element(cx);
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
                        let calendar =
                            fret_ui_shadcn::Calendar::new(month, selected).into_element(cx);

                        fret_ui_shadcn::PopoverContent::new([calendar])
                            .refine_layout(
                                LayoutRefinement::default().w_px(MetricRef::Px(Px(249.33334))),
                            )
                            .into_element(cx)
                    },
                );

            stack::vstack(
                cx,
                stack::VStackProps::default().gap(Space::N3),
                move |_cx| vec![label, popover],
            )
        },
        SemanticsRole::Button,
        Some("Jun 01, 2025"),
        SemanticsRole::Dialog,
    );
}
#[test]
fn web_vs_fret_calendar_27_open_overlay_placement_matches() {
    assert_overlay_placement_matches(
        "calendar-27",
        Some("dialog"),
        |cx, open| {
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
                                    Date::from_calendar_date(2025, Month::June, 5)
                                        .expect("valid date"),
                                ),
                                to: Some(
                                    Date::from_calendar_date(2025, Month::June, 20)
                                        .expect("valid date"),
                                ),
                            });
                        let calendar =
                            fret_ui_shadcn::CalendarRange::new(month, selected).into_element(cx);
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
        },
        SemanticsRole::Button,
        None,
        SemanticsRole::Dialog,
    );
}
#[test]
fn web_vs_fret_calendar_28_open_overlay_placement_matches() {
    assert_overlay_placement_matches(
        "calendar-28",
        Some("dialog"),
        |cx, open| {
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
                        let calendar =
                            fret_ui_shadcn::Calendar::new(month, selected).into_element(cx);
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
        },
        SemanticsRole::Button,
        None,
        SemanticsRole::Dialog,
    );
}
#[test]
fn web_vs_fret_calendar_29_open_overlay_placement_matches() {
    assert_overlay_placement_matches(
        "calendar-29",
        Some("dialog"),
        |cx, open| {
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
                        let calendar =
                            fret_ui_shadcn::Calendar::new(month, selected).into_element(cx);
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
        },
        SemanticsRole::Button,
        None,
        SemanticsRole::Dialog,
    );
}
#[test]
fn web_vs_fret_calendar_30_open_overlay_placement_matches() {
    assert_overlay_placement_matches(
        "calendar-30",
        Some("dialog"),
        |cx, open| {
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
                                    Date::from_calendar_date(2025, Month::June, 4)
                                        .expect("valid date"),
                                ),
                                to: Some(
                                    Date::from_calendar_date(2025, Month::June, 10)
                                        .expect("valid date"),
                                ),
                            });
                        let calendar =
                            fret_ui_shadcn::CalendarRange::new(month, selected).into_element(cx);

                        fret_ui_shadcn::PopoverContent::new([calendar])
                            .refine_layout(
                                LayoutRefinement::default().w_px(MetricRef::Px(Px(249.33334))),
                            )
                            .into_element(cx)
                    },
                );

            stack::vstack(
                cx,
                stack::VStackProps::default().gap(Space::N3),
                move |_cx| vec![label, popover],
            )
        },
        SemanticsRole::Button,
        Some("Jun 4 - 10, 2025"),
        SemanticsRole::Dialog,
    );
}
#[test]
fn web_vs_fret_calendar_32_open_drawer_insets_match() {
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
