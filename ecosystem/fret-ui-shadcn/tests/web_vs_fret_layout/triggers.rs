use super::*;

#[test]
fn web_vs_fret_layout_trigger_heights_match_web_fixtures() {
    let raw = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/layout_trigger_height_cases_v1.json"
    ));
    let suite: FixtureSuite<LayoutTriggerHeightCase> =
        serde_json::from_str(raw).expect("layout trigger height fixture parse");
    assert_eq!(suite.schema_version, 1);
    assert!(!suite.cases.is_empty());

    for case in suite.cases {
        eprintln!("layout trigger height case={}", case.id);
        let web = read_web_golden(&case.web_name);
        let theme = web_theme(&web);
        let web_trigger = web_find_by_tag_and_text(&theme.root, "button", &case.label)
            .unwrap_or_else(|| {
                panic!(
                    "web trigger missing: web={} label={}",
                    case.web_name, case.label
                )
            });

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            CoreSize::new(Px(theme.viewport.w), Px(theme.viewport.h)),
        );

        let snap = match case.recipe {
            LayoutTriggerHeightRecipe::PlainButton => run_fret_root(bounds, |cx| {
                vec![fret_ui_shadcn::Button::new(case.label.clone()).into_element(cx)]
            }),
            LayoutTriggerHeightRecipe::DrawerTrigger => run_fret_root(bounds, |cx| {
                use fret_ui_shadcn::{Button, ButtonVariant, Drawer, DrawerContent};

                let open: Model<bool> = cx.app.models_mut().insert(false);
                vec![Drawer::new(open).into_element(
                    cx,
                    |cx| {
                        Button::new(case.label.clone())
                            .variant(ButtonVariant::Outline)
                            .into_element(cx)
                    },
                    |cx| DrawerContent::new(vec![cx.text("Drawer content")]).into_element(cx),
                )]
            }),
            LayoutTriggerHeightRecipe::DialogTrigger => run_fret_root(bounds, |cx| {
                use fret_ui_shadcn::{Button, ButtonVariant, Dialog, DialogContent};

                let open: Model<bool> = cx.app.models_mut().insert(false);
                vec![Dialog::new(open).into_element(
                    cx,
                    |cx| {
                        Button::new(case.label.clone())
                            .variant(ButtonVariant::Outline)
                            .into_element(cx)
                    },
                    |cx| DialogContent::new(Vec::new()).into_element(cx),
                )]
            }),
        };

        let trigger = find_semantics(&snap, SemanticsRole::Button, Some(&case.label))
            .or_else(|| find_semantics(&snap, SemanticsRole::Button, None))
            .unwrap_or_else(|| {
                panic!(
                    "missing fret trigger semantics: web={} label={}",
                    case.web_name, case.label
                )
            });

        assert_close_px(
            &format!("{} trigger height", case.web_name),
            trigger.bounds.size.height,
            web_trigger.rect.h,
            1.0,
        );
    }
}

#[test]
fn web_vs_fret_layout_date_picker_trigger_geometry_matches_web_fixtures() {
    let raw = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/layout_date_picker_trigger_cases_v1.json"
    ));
    let suite: FixtureSuite<LayoutDatePickerTriggerCase> =
        serde_json::from_str(raw).expect("layout date picker fixture parse");
    assert_eq!(suite.schema_version, 1);
    assert!(!suite.cases.is_empty());

    for case in suite.cases {
        eprintln!("layout date picker trigger case={}", case.id);
        let web = read_web_golden(&case.web_name);
        let theme = web_theme(&web);

        let web_button = match case.recipe {
            LayoutDatePickerTriggerRecipe::DateRangePicker => find_first(&theme.root, &|n| {
                n.tag == "button" && contains_id(n, "date")
            })
            .expect("web button"),
            LayoutDatePickerTriggerRecipe::DatePicker
            | LayoutDatePickerTriggerRecipe::DatePickerWithPresets => {
                web_find_by_tag_and_text(&theme.root, "button", "Pick a date").expect("web button")
            }
        };

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            CoreSize::new(Px(theme.viewport.w), Px(theme.viewport.h)),
        );

        let snap = match case.recipe {
            LayoutDatePickerTriggerRecipe::DatePicker => run_fret_root(bounds, |cx| {
                use fret_ui_headless::calendar::CalendarMonth;
                use time::Month;

                let open: Model<bool> = cx.app.models_mut().insert(false);
                let month: Model<CalendarMonth> = cx
                    .app
                    .models_mut()
                    .insert(CalendarMonth::new(2026, Month::January));
                let selected: Model<Option<time::Date>> = cx.app.models_mut().insert(None);

                vec![
                    fret_ui_shadcn::DatePicker::new(open, month, selected)
                        .refine_layout(
                            LayoutRefinement::default().w_px(MetricRef::Px(Px(web_button.rect.w))),
                        )
                        .into_element(cx),
                ]
            }),
            LayoutDatePickerTriggerRecipe::DatePickerWithPresets => run_fret_root(bounds, |cx| {
                use fret_ui_headless::calendar::CalendarMonth;
                use time::Month;

                let open: Model<bool> = cx.app.models_mut().insert(false);
                let month: Model<CalendarMonth> = cx
                    .app
                    .models_mut()
                    .insert(CalendarMonth::new(2026, Month::January));
                let selected: Model<Option<time::Date>> = cx.app.models_mut().insert(None);

                vec![
                    fret_ui_shadcn::DatePickerWithPresets::new(open, month, selected)
                        .refine_layout(
                            LayoutRefinement::default().w_px(MetricRef::Px(Px(web_button.rect.w))),
                        )
                        .into_element(cx),
                ]
            }),
            LayoutDatePickerTriggerRecipe::DateRangePicker => run_fret_root(bounds, |cx| {
                use fret_ui_headless::calendar::CalendarMonth;
                use time::{Date, Month};

                let open: Model<bool> = cx.app.models_mut().insert(false);
                let month: Model<CalendarMonth> = cx
                    .app
                    .models_mut()
                    .insert(CalendarMonth::new(2022, Month::January));
                let selected: Model<fret_ui_headless::calendar::DateRangeSelection> = cx
                    .app
                    .models_mut()
                    .insert(fret_ui_headless::calendar::DateRangeSelection {
                        from: Some(
                            Date::from_calendar_date(2022, Month::January, 20).expect("from date"),
                        ),
                        to: Some(
                            Date::from_calendar_date(2022, Month::February, 9).expect("to date"),
                        ),
                    });

                vec![
                    fret_ui_shadcn::DateRangePicker::new(open, month, selected)
                        .refine_layout(
                            LayoutRefinement::default().w_px(MetricRef::Px(Px(web_button.rect.w))),
                        )
                        .into_element(cx),
                ]
            }),
        };

        let button = find_semantics(&snap, SemanticsRole::Button, Some(&case.label))
            .or_else(|| find_semantics(&snap, SemanticsRole::Button, None))
            .expect("fret date-picker trigger button");

        assert_close_px(
            &format!("{} trigger w", case.web_name),
            button.bounds.size.width,
            web_button.rect.w,
            1.0,
        );
        assert_close_px(
            &format!("{} trigger h", case.web_name),
            button.bounds.size.height,
            web_button.rect.h,
            1.0,
        );
    }
}
