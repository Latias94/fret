use super::super::super::super::*;

pub(in crate::ui) fn preview_calendar(
    cx: &mut ElementContext<'_, App>,
    month: Model<fret_ui_headless::calendar::CalendarMonth>,
    selected: Model<Option<Date>>,
) -> Vec<AnyElement> {
    use fret_ui_headless::calendar::DateRangeSelection;

    let theme = Theme::global(&*cx.app).snapshot();
    let today = time::OffsetDateTime::now_utc().date();

    #[derive(Default, Clone)]
    struct CalendarModels {
        caption_month: Option<Model<fret_ui_headless::calendar::CalendarMonth>>,
        caption_selected: Option<Model<Option<Date>>>,
        range_month: Option<Model<fret_ui_headless::calendar::CalendarMonth>>,
        range_selected: Option<Model<DateRangeSelection>>,
        presets_month: Option<Model<fret_ui_headless::calendar::CalendarMonth>>,
        presets_selected: Option<Model<Option<Date>>>,
        time_month: Option<Model<fret_ui_headless::calendar::CalendarMonth>>,
        time_selected: Option<Model<Option<Date>>>,
        time_from: Option<Model<String>>,
        time_to: Option<Model<String>>,
        booked_month: Option<Model<fret_ui_headless::calendar::CalendarMonth>>,
        booked_selected: Option<Model<Option<Date>>>,
        custom_cell_month: Option<Model<fret_ui_headless::calendar::CalendarMonth>>,
        custom_cell_selected: Option<Model<Option<Date>>>,
        week_number_month: Option<Model<fret_ui_headless::calendar::CalendarMonth>>,
        week_number_selected: Option<Model<Option<Date>>>,
        rtl_month: Option<Model<fret_ui_headless::calendar::CalendarMonth>>,
        rtl_selected: Option<Model<Option<Date>>>,
    }

    let initial_month = cx
        .get_model_copied(&month, Invalidation::Layout)
        .unwrap_or_else(|| fret_ui_headless::calendar::CalendarMonth::from_date(today));

    let state = cx.with_state(CalendarModels::default, |st| st.clone());

    let caption_month = match state.caption_month {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert(initial_month);
            cx.with_state(CalendarModels::default, |st| {
                st.caption_month = Some(model.clone())
            });
            model
        }
    };
    let caption_selected = match state.caption_selected {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert(None::<Date>);
            cx.with_state(CalendarModels::default, |st| {
                st.caption_selected = Some(model.clone())
            });
            model
        }
    };

    let range_month = match state.range_month {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert(initial_month);
            cx.with_state(CalendarModels::default, |st| {
                st.range_month = Some(model.clone())
            });
            model
        }
    };
    let range_selected = match state.range_selected {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert(DateRangeSelection::default());
            cx.with_state(CalendarModels::default, |st| {
                st.range_selected = Some(model.clone())
            });
            model
        }
    };

    let preset_date = time::Date::from_calendar_date(today.year(), time::Month::February, 12)
        .expect("valid preset date");
    let presets_initial_month = fret_ui_headless::calendar::CalendarMonth::from_date(preset_date);
    let presets_month = match state.presets_month {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert(presets_initial_month);
            cx.with_state(CalendarModels::default, |st| {
                st.presets_month = Some(model.clone())
            });
            model
        }
    };
    let presets_selected = match state.presets_selected {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert(Some(preset_date));
            cx.with_state(CalendarModels::default, |st| {
                st.presets_selected = Some(model.clone())
            });
            model
        }
    };

    let time_date = time::Date::from_calendar_date(today.year(), today.month(), 12)
        .expect("valid time picker date");
    let time_initial_month = fret_ui_headless::calendar::CalendarMonth::from_date(time_date);
    let time_month = match state.time_month {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert(time_initial_month);
            cx.with_state(CalendarModels::default, |st| {
                st.time_month = Some(model.clone())
            });
            model
        }
    };
    let time_selected = match state.time_selected {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert(Some(time_date));
            cx.with_state(CalendarModels::default, |st| {
                st.time_selected = Some(model.clone())
            });
            model
        }
    };
    let time_from = match state.time_from {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert(String::from("10:30:00"));
            cx.with_state(CalendarModels::default, |st| {
                st.time_from = Some(model.clone())
            });
            model
        }
    };
    let time_to = match state.time_to {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert(String::from("12:30:00"));
            cx.with_state(CalendarModels::default, |st| {
                st.time_to = Some(model.clone())
            });
            model
        }
    };

    let booked_month = match state.booked_month {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert(initial_month);
            cx.with_state(CalendarModels::default, |st| {
                st.booked_month = Some(model.clone())
            });
            model
        }
    };
    let booked_selected = match state.booked_selected {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert(None::<Date>);
            cx.with_state(CalendarModels::default, |st| {
                st.booked_selected = Some(model.clone())
            });
            model
        }
    };

    let custom_cell_month = match state.custom_cell_month {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert(initial_month);
            cx.with_state(CalendarModels::default, |st| {
                st.custom_cell_month = Some(model.clone())
            });
            model
        }
    };
    let custom_cell_selected = match state.custom_cell_selected {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert(None::<Date>);
            cx.with_state(CalendarModels::default, |st| {
                st.custom_cell_selected = Some(model.clone())
            });
            model
        }
    };

    let week_number_month = match state.week_number_month {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert(initial_month);
            cx.with_state(CalendarModels::default, |st| {
                st.week_number_month = Some(model.clone())
            });
            model
        }
    };
    let week_number_selected = match state.week_number_selected {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert(None::<Date>);
            cx.with_state(CalendarModels::default, |st| {
                st.week_number_selected = Some(model.clone())
            });
            model
        }
    };

    let rtl_month = match state.rtl_month {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert(initial_month);
            cx.with_state(CalendarModels::default, |st| {
                st.rtl_month = Some(model.clone())
            });
            model
        }
    };
    let rtl_selected = match state.rtl_selected {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert(Some(today));
            cx.with_state(CalendarModels::default, |st| {
                st.rtl_selected = Some(model.clone())
            });
            model
        }
    };

    let section = |cx: &mut ElementContext<'_, App>, title: &'static str, body: AnyElement| {
        stack::vstack(
            cx,
            stack::VStackProps::default().gap(Space::N2).items_start(),
            move |cx| vec![shadcn::typography::h4(cx, title), body],
        )
    };

    let basic = {
        let body = stack::hstack(
            cx,
            stack::HStackProps::default().gap(Space::N6).items_start(),
            |cx| {
                let selected_str = cx
                    .get_model_copied(&selected, Invalidation::Layout)
                    .flatten()
                    .map(|d| d.to_string())
                    .unwrap_or_else(|| "<none>".to_string());

                vec![
                    shadcn::Calendar::new(month.clone(), selected.clone())
                        .refine_style(ChromeRefinement::default().border_1().rounded(Radius::Lg))
                        .into_element(cx),
                    stack::vstack(
                        cx,
                        stack::VStackProps::default().gap(Space::N1).items_start(),
                        |cx| {
                            vec![cx.text_props(TextProps {
                                layout: Default::default(),
                                text: Arc::from(format!("selected={}", selected_str)),
                                style: None,
                                color: Some(theme.color_required("muted-foreground")),
                                wrap: TextWrap::None,
                                overflow: TextOverflow::Clip,
                            })]
                        },
                    ),
                ]
            },
        );
        section(cx, "Basic", body)
    };

    let range = {
        let body = stack::hstack(
            cx,
            stack::HStackProps::default().gap(Space::N6).items_start(),
            |cx| {
                let range = cx
                    .get_model_copied(&range_selected, Invalidation::Layout)
                    .unwrap_or_default();
                let from = range
                    .from
                    .map(|d| d.to_string())
                    .unwrap_or_else(|| "<none>".to_string());
                let to = range
                    .to
                    .map(|d| d.to_string())
                    .unwrap_or_else(|| "<none>".to_string());

                vec![
                    shadcn::CalendarRange::new(range_month.clone(), range_selected.clone())
                        .number_of_months(2)
                        .refine_style(ChromeRefinement::default().border_1().rounded(Radius::Lg))
                        .into_element(cx),
                    stack::vstack(
                        cx,
                        stack::VStackProps::default().gap(Space::N1).items_start(),
                        |cx| {
                            vec![
                                cx.text_props(TextProps {
                                    layout: Default::default(),
                                    text: Arc::from(format!("from={}", from)),
                                    style: None,
                                    color: Some(theme.color_required("muted-foreground")),
                                    wrap: TextWrap::None,
                                    overflow: TextOverflow::Clip,
                                }),
                                cx.text_props(TextProps {
                                    layout: Default::default(),
                                    text: Arc::from(format!("to={}", to)),
                                    style: None,
                                    color: Some(theme.color_required("muted-foreground")),
                                    wrap: TextWrap::None,
                                    overflow: TextOverflow::Clip,
                                }),
                            ]
                        },
                    ),
                ]
            },
        );
        section(cx, "Range Calendar", body)
    };

    let month_year_selector = {
        let body = shadcn::Calendar::new(caption_month.clone(), caption_selected.clone())
            .caption_layout(shadcn::CalendarCaptionLayout::Dropdown)
            .refine_style(ChromeRefinement::default().border_1().rounded(Radius::Lg))
            .into_element(cx);
        section(cx, "Month and Year Selector", body)
    };

    let presets = {
        let preset_button =
            |cx: &mut ElementContext<'_, App>, label: &'static str, days: i64| -> AnyElement {
                let month = presets_month.clone();
                let selected = presets_selected.clone();
                shadcn::Button::new(label)
                    .variant(shadcn::ButtonVariant::Outline)
                    .size(shadcn::ButtonSize::Sm)
                    .refine_layout(LayoutRefinement::default().flex_1().w_full())
                    .on_activate(Arc::new(move |host, _acx, _reason| {
                        let new_date = today + time::Duration::days(days);
                        let _ = host.models_mut().update(&selected, |v| *v = Some(new_date));
                        let _ = host.models_mut().update(&month, |m| {
                            *m = fret_ui_headless::calendar::CalendarMonth::from_date(new_date);
                        });
                    }))
                    .into_element(cx)
            };

        let calendar = shadcn::Calendar::new(presets_month.clone(), presets_selected.clone())
            .cell_size(Px(38.0))
            .refine_style(ChromeRefinement::default().p(Space::N0))
            .into_element(cx);

        let footer = stack::vstack(
            cx,
            stack::VStackProps::default()
                .gap(Space::N2)
                .layout(LayoutRefinement::default().w_full())
                .items_start(),
            |cx| {
                vec![
                    stack::hstack(
                        cx,
                        stack::HStackProps::default()
                            .gap(Space::N2)
                            .layout(LayoutRefinement::default().w_full()),
                        |cx| {
                            vec![
                                preset_button(cx, "Today", 0),
                                preset_button(cx, "Tomorrow", 1),
                                preset_button(cx, "In 3 days", 3),
                            ]
                        },
                    ),
                    stack::hstack(
                        cx,
                        stack::HStackProps::default()
                            .gap(Space::N2)
                            .layout(LayoutRefinement::default().w_full()),
                        |cx| {
                            vec![
                                preset_button(cx, "In a week", 7),
                                preset_button(cx, "In 2 weeks", 14),
                            ]
                        },
                    ),
                ]
            },
        );

        let card = shadcn::Card::new(vec![
            shadcn::CardContent::new(vec![calendar]).into_element(cx),
            shadcn::CardFooter::new(vec![footer]).into_element(cx),
        ])
        .size(shadcn::CardSize::Sm)
        .refine_layout(
            LayoutRefinement::default()
                .max_w(MetricRef::Px(Px(300.0)))
                .min_w_0(),
        )
        .into_element(cx);

        section(cx, "Presets", card)
    };

    let date_and_time_picker = {
        let clock_fg = ColorRef::Color(theme.color_required("muted-foreground"));
        let clock_icon = |cx: &mut ElementContext<'_, App>| {
            shadcn::icon::icon_with(
                cx,
                fret_icons::IconId::new_static("lucide.clock-2"),
                None,
                Some(clock_fg.clone()),
            )
        };

        let calendar = shadcn::Calendar::new(time_month.clone(), time_selected.clone())
            .refine_style(ChromeRefinement::default().p(Space::N0))
            .into_element(cx);

        let footer = shadcn::FieldGroup::new([
            shadcn::Field::new([
                shadcn::FieldLabel::new("Start Time").into_element(cx),
                shadcn::InputGroup::new(time_from.clone())
                    .a11y_label("Start Time")
                    .trailing([clock_icon(cx)])
                    .into_element(cx),
            ])
            .into_element(cx),
            shadcn::Field::new([
                shadcn::FieldLabel::new("End Time").into_element(cx),
                shadcn::InputGroup::new(time_to.clone())
                    .a11y_label("End Time")
                    .trailing([clock_icon(cx)])
                    .into_element(cx),
            ])
            .into_element(cx),
        ])
        .into_element(cx);

        let card = shadcn::Card::new(vec![
            shadcn::CardContent::new(vec![calendar]).into_element(cx),
            shadcn::CardFooter::new(vec![footer]).into_element(cx),
        ])
        .size(shadcn::CardSize::Sm)
        .refine_layout(LayoutRefinement::default().min_w_0())
        .into_element(cx);

        section(cx, "Date and Time Picker", card)
    };

    let booked_dates = {
        let body = stack::hstack(
            cx,
            stack::HStackProps::default().gap(Space::N6).items_start(),
            |cx| {
                vec![
                    shadcn::Calendar::new(booked_month.clone(), booked_selected.clone())
                        .disabled_by(|d| {
                            matches!(d.weekday(), time::Weekday::Saturday | time::Weekday::Sunday)
                        })
                        .refine_style(ChromeRefinement::default().border_1().rounded(Radius::Lg))
                        .into_element(cx),
                    cx.text_props(TextProps {
                        layout: Default::default(),
                        text: Arc::from("Disabled: weekends"),
                        style: None,
                        color: Some(theme.color_required("muted-foreground")),
                        wrap: TextWrap::None,
                        overflow: TextOverflow::Clip,
                    }),
                ]
            },
        );
        section(cx, "Booked dates", body)
    };

    let custom_cell_size = {
        let body = shadcn::Calendar::new(custom_cell_month.clone(), custom_cell_selected.clone())
            .cell_size(Px(44.0))
            .refine_style(ChromeRefinement::default().border_1().rounded(Radius::Lg))
            .into_element(cx);
        section(cx, "Custom Cell Size", body)
    };

    let week_numbers = {
        let body = shadcn::Calendar::new(week_number_month.clone(), week_number_selected.clone())
            .show_week_number(true)
            .refine_style(ChromeRefinement::default().border_1().rounded(Radius::Lg))
            .into_element(cx);
        section(cx, "Week Numbers", body)
    };

    let rtl = {
        let body = fret_ui_kit::primitives::direction::with_direction_provider(
            cx,
            fret_ui_kit::primitives::direction::LayoutDirection::Rtl,
            |cx| {
                shadcn::Calendar::new(rtl_month.clone(), rtl_selected.clone())
                    .cell_size(Px(36.0))
                    .caption_layout(shadcn::CalendarCaptionLayout::Dropdown)
                    .refine_style(ChromeRefinement::default().border_1().rounded(Radius::Lg))
                    .into_element(cx)
            },
        );
        section(cx, "RTL", body)
    };

    vec![stack::vstack(
        cx,
        stack::VStackProps::default()
            .gap(Space::N6)
            .items_start()
            .layout(LayoutRefinement::default().w_full()),
        |_cx| {
            vec![
                basic,
                range,
                month_year_selector,
                presets,
                date_and_time_picker,
                booked_dates,
                custom_cell_size,
                week_numbers,
                rtl,
            ]
        },
    )]
}
