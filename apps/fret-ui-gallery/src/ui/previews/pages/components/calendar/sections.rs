use super::super::super::super::super::*;

use super::models::CalendarHandles;
use fret_ui::ThemeSnapshot;

fn section(cx: &mut ElementContext<'_, App>, title: &'static str, body: AnyElement) -> AnyElement {
    stack::vstack(
        cx,
        stack::VStackProps::default().gap(Space::N2).items_start(),
        move |cx| vec![shadcn::typography::h4(cx, title), body],
    )
}

pub(super) fn basic(
    cx: &mut ElementContext<'_, App>,
    theme: &ThemeSnapshot,
    models: &CalendarHandles,
) -> AnyElement {
    let month = models.month.clone();
    let selected = models.selected.clone();

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
}

pub(super) fn range(
    cx: &mut ElementContext<'_, App>,
    theme: &ThemeSnapshot,
    models: &CalendarHandles,
) -> AnyElement {
    let range_month = models.range_month.clone();
    let range_selected = models.range_selected.clone();

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
}

pub(super) fn month_year_selector(
    cx: &mut ElementContext<'_, App>,
    models: &CalendarHandles,
) -> AnyElement {
    let caption_month = models.caption_month.clone();
    let caption_selected = models.caption_selected.clone();

    let body = shadcn::Calendar::new(caption_month, caption_selected)
        .caption_layout(shadcn::CalendarCaptionLayout::Dropdown)
        .refine_style(ChromeRefinement::default().border_1().rounded(Radius::Lg))
        .into_element(cx);
    section(cx, "Month and Year Selector", body)
}

pub(super) fn presets(
    cx: &mut ElementContext<'_, App>,
    models: &CalendarHandles,
    today: Date,
) -> AnyElement {
    let presets_month = models.presets_month.clone();
    let presets_selected = models.presets_selected.clone();

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
}

pub(super) fn date_and_time_picker(
    cx: &mut ElementContext<'_, App>,
    theme: &ThemeSnapshot,
    models: &CalendarHandles,
) -> AnyElement {
    let time_month = models.time_month.clone();
    let time_selected = models.time_selected.clone();
    let time_from = models.time_from.clone();
    let time_to = models.time_to.clone();

    let clock_fg = ColorRef::Color(theme.color_required("muted-foreground"));
    let clock_icon = |cx: &mut ElementContext<'_, App>| {
        shadcn::icon::icon_with(
            cx,
            fret_icons::IconId::new_static("lucide.clock-2"),
            None,
            Some(clock_fg.clone()),
        )
    };

    let calendar = shadcn::Calendar::new(time_month, time_selected)
        .refine_style(ChromeRefinement::default().p(Space::N0))
        .into_element(cx);

    let footer = shadcn::FieldGroup::new([
        shadcn::Field::new([
            shadcn::FieldLabel::new("Start Time").into_element(cx),
            shadcn::InputGroup::new(time_from)
                .a11y_label("Start Time")
                .trailing([clock_icon(cx)])
                .into_element(cx),
        ])
        .into_element(cx),
        shadcn::Field::new([
            shadcn::FieldLabel::new("End Time").into_element(cx),
            shadcn::InputGroup::new(time_to)
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
}

pub(super) fn booked_dates(
    cx: &mut ElementContext<'_, App>,
    theme: &ThemeSnapshot,
    models: &CalendarHandles,
) -> AnyElement {
    let booked_month = models.booked_month.clone();
    let booked_selected = models.booked_selected.clone();

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
}

pub(super) fn custom_cell_size(
    cx: &mut ElementContext<'_, App>,
    models: &CalendarHandles,
) -> AnyElement {
    let custom_cell_month = models.custom_cell_month.clone();
    let custom_cell_selected = models.custom_cell_selected.clone();

    let body = shadcn::Calendar::new(custom_cell_month, custom_cell_selected)
        .cell_size(Px(44.0))
        .refine_style(ChromeRefinement::default().border_1().rounded(Radius::Lg))
        .into_element(cx);
    section(cx, "Custom Cell Size", body)
}

pub(super) fn week_numbers(
    cx: &mut ElementContext<'_, App>,
    models: &CalendarHandles,
) -> AnyElement {
    let week_number_month = models.week_number_month.clone();
    let week_number_selected = models.week_number_selected.clone();

    let body = shadcn::Calendar::new(week_number_month, week_number_selected)
        .show_week_number(true)
        .refine_style(ChromeRefinement::default().border_1().rounded(Radius::Lg))
        .into_element(cx);
    section(cx, "Week Numbers", body)
}

pub(super) fn rtl(cx: &mut ElementContext<'_, App>, models: &CalendarHandles) -> AnyElement {
    let rtl_month = models.rtl_month.clone();
    let rtl_selected = models.rtl_selected.clone();

    let body = fret_ui_kit::primitives::direction::with_direction_provider(
        cx,
        fret_ui_kit::primitives::direction::LayoutDirection::Rtl,
        |cx| {
            shadcn::Calendar::new(rtl_month, rtl_selected)
                .cell_size(Px(36.0))
                .caption_layout(shadcn::CalendarCaptionLayout::Dropdown)
                .refine_style(ChromeRefinement::default().border_1().rounded(Radius::Lg))
                .into_element(cx)
        },
    );
    section(cx, "RTL", body)
}
