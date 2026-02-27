use super::super::super::super::super::*;

use super::models::CalendarHandles;
use fret_ui::ThemeSnapshot;
use fret_ui_headless::calendar_solar_hijri::SolarHijriMonth;

pub(super) fn demo(cx: &mut ElementContext<'_, App>, models: &CalendarHandles) -> AnyElement {
    let month = models.caption_month.clone();
    let selected = models.caption_selected.clone();

    shadcn::Calendar::new(month, selected)
        .test_id_prefix("ui-gallery.calendar.demo")
        .caption_layout(shadcn::CalendarCaptionLayout::Dropdown)
        .refine_style(
            ChromeRefinement::default()
                .border_1()
                .rounded(Radius::Md)
                .shadow_sm(),
        )
        .into_element(cx)
}

pub(super) fn basic(cx: &mut ElementContext<'_, App>, models: &CalendarHandles) -> AnyElement {
    let month = models.caption_month.clone();
    let selected = models.caption_selected.clone();

    shadcn::Calendar::new(month, selected)
        .test_id_prefix("ui-gallery.calendar.basic")
        .refine_style(ChromeRefinement::default().border_1().rounded(Radius::Lg))
        .into_element(cx)
}

pub(super) fn locale(cx: &mut ElementContext<'_, App>, models: &CalendarHandles) -> AnyElement {
    let month = models.locale_month.clone();
    let selected = models.locale_selected.clone();

    shadcn::Calendar::new(month, selected)
        .locale(shadcn::calendar::CalendarLocale::Es)
        .week_start(time::Weekday::Monday)
        .test_id_prefix("ui-gallery.calendar.locale")
        .refine_style(ChromeRefinement::default().border_1().rounded(Radius::Lg))
        .into_element(cx)
}

pub(super) fn range(cx: &mut ElementContext<'_, App>, models: &CalendarHandles) -> AnyElement {
    let range_month = models.range_month.clone();
    let range_selected = models.range_selected.clone();

    shadcn::CalendarRange::new(range_month, range_selected)
        .number_of_months(2)
        .test_id_prefix("ui-gallery.calendar.range")
        .refine_style(ChromeRefinement::default().border_1().rounded(Radius::Lg))
        .into_element(cx)
}

pub(super) fn responsive_mixed_semantics(
    cx: &mut ElementContext<'_, App>,
    models: &CalendarHandles,
) -> AnyElement {
    #[derive(Default, Clone)]
    struct ResponsiveModels {
        popover_open: Option<Model<bool>>,
    }

    let state = cx.with_state(ResponsiveModels::default, |st| st.clone());
    let popover_open = match state.popover_open {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert(false);
            cx.with_state(ResponsiveModels::default, |st| {
                st.popover_open = Some(model.clone())
            });
            model
        }
    };

    let range_month = models.range_month.clone();
    let range_selected = models.range_selected.clone();

    let panel_calendar = shadcn::CalendarRange::new(range_month.clone(), range_selected.clone())
        .number_of_months(2)
        .test_id_prefix("ui-gallery.calendar.responsive.panel")
        .refine_style(ChromeRefinement::default().border_1().rounded(Radius::Lg))
        .into_element(cx);

    let panel = stack::vstack(
        cx,
        stack::VStackProps::default()
            .gap(Space::N2)
            .items_start()
            .layout(LayoutRefinement::default().w_px(Px(420.0)).min_w_0()),
        move |cx| {
            vec![
                shadcn::Badge::new("Panel: container queries").into_element(cx),
                panel_calendar,
            ]
        },
    )
    .test_id("ui-gallery-calendar-responsive-panel");

    let popover = shadcn::Popover::new(popover_open.clone())
        .side(shadcn::PopoverSide::Bottom)
        .align(shadcn::PopoverAlign::Start)
        .into_element(
            cx,
            move |cx| {
                shadcn::Button::new("Open calendar popover")
                    .variant(shadcn::ButtonVariant::Outline)
                    .toggle_model(popover_open.clone())
                    .test_id("ui-gallery-calendar-responsive-popover-trigger")
                    .into_element(cx)
            },
            move |cx| {
                let calendar =
                    shadcn::CalendarRange::new(range_month.clone(), range_selected.clone())
                        .number_of_months(2)
                        .test_id_prefix("ui-gallery.calendar.responsive.popover")
                        .into_element(cx);

                shadcn::PopoverContent::new([calendar])
                    .refine_style(ChromeRefinement::default().p(Space::N0))
                    .refine_layout(
                        LayoutRefinement::default()
                            .w(fret_ui_kit::LengthRefinement::Auto)
                            .min_w_0()
                            .overflow_hidden(),
                    )
                    .into_element(cx)
                    .test_id("ui-gallery-calendar-responsive-popover-content")
            },
        );

    stack::hstack(
        cx,
        stack::HStackProps::default()
            .gap(Space::N6)
            .items_start()
            .layout(LayoutRefinement::default().w_full().min_w_0()),
        move |_cx| vec![panel, popover],
    )
}

pub(super) fn month_year_selector(
    cx: &mut ElementContext<'_, App>,
    models: &CalendarHandles,
) -> AnyElement {
    let caption_month = models.caption_month.clone();
    let caption_selected = models.caption_selected.clone();

    let body = shadcn::Calendar::new(caption_month, caption_selected)
        .test_id_prefix("ui-gallery.calendar.caption")
        .caption_layout(shadcn::CalendarCaptionLayout::Dropdown)
        .refine_style(ChromeRefinement::default().border_1().rounded(Radius::Lg))
        .into_element(cx);
    body
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
                .refine_layout(LayoutRefinement::default().flex_1())
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
        .test_id_prefix("ui-gallery.calendar.presets")
        .fixed_weeks(true)
        .cell_size(Px(38.0))
        .refine_style(ChromeRefinement::default().p(Space::N0))
        .into_element(cx);

    let footer_buttons = vec![
        preset_button(cx, "Today", 0),
        preset_button(cx, "Tomorrow", 1),
        preset_button(cx, "In 3 days", 3),
        preset_button(cx, "In a week", 7),
        preset_button(cx, "In 2 weeks", 14),
    ];

    let card = shadcn::Card::new(vec![
        shadcn::CardContent::new(vec![calendar]).into_element(cx),
        shadcn::CardFooter::new(footer_buttons)
            .border_top(true)
            .wrap(true)
            .gap(Space::N2)
            .into_element(cx),
    ])
    .size(shadcn::CardSize::Sm)
    .refine_layout(
        LayoutRefinement::default()
            .max_w(MetricRef::Px(Px(300.0)))
            .min_w_0(),
    )
    .into_element(cx);

    card
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

    let clock_fg = ColorRef::Color(theme.color_token("muted-foreground"));
    let clock_icon = |cx: &mut ElementContext<'_, App>| {
        shadcn::icon::icon_with(
            cx,
            fret_icons::IconId::new_static("lucide.clock-2"),
            None,
            Some(clock_fg.clone()),
        )
    };

    let calendar = shadcn::Calendar::new(time_month, time_selected)
        .test_id_prefix("ui-gallery.calendar.time")
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

    card
}

pub(super) fn booked_dates(
    cx: &mut ElementContext<'_, App>,
    models: &CalendarHandles,
    today: Date,
) -> AnyElement {
    let booked_month = models.booked_month.clone();
    let booked_selected = models.booked_selected.clone();

    let booked_dates = {
        let year = today.year();
        let start = time::Date::from_calendar_date(year, time::Month::January, 12)
            .expect("valid booked date");
        Arc::<[Date]>::from(
            (0..15)
                .map(|i| start + time::Duration::days(i))
                .collect::<Vec<_>>(),
        )
    };

    shadcn::Calendar::new(booked_month, booked_selected)
        .test_id_prefix("ui-gallery.calendar.booked")
        .disabled(fret_ui_headless::calendar::DayMatcher::dates(booked_dates))
        .refine_style(ChromeRefinement::default().border_1().rounded(Radius::Lg))
        .into_element(cx)
}

pub(super) fn custom_cell_size(
    cx: &mut ElementContext<'_, App>,
    models: &CalendarHandles,
) -> AnyElement {
    let custom_cell_month = models.custom_cell_month.clone();
    let custom_cell_selected = models.custom_cell_selected.clone();

    let body = shadcn::Calendar::new(custom_cell_month, custom_cell_selected)
        .test_id_prefix("ui-gallery.calendar.custom-cell")
        .cell_size(Px(44.0))
        .caption_layout(shadcn::CalendarCaptionLayout::Dropdown)
        .refine_style(ChromeRefinement::default().border_1().rounded(Radius::Lg))
        .into_element(cx);
    body
}

pub(super) fn week_numbers(
    cx: &mut ElementContext<'_, App>,
    models: &CalendarHandles,
) -> AnyElement {
    let week_number_month = models.week_number_month.clone();
    let week_number_selected = models.week_number_selected.clone();

    let body = shadcn::Calendar::new(week_number_month, week_number_selected)
        .test_id_prefix("ui-gallery.calendar.week-numbers")
        .show_week_number(true)
        .refine_style(ChromeRefinement::default().border_1().rounded(Radius::Lg))
        .into_element(cx);
    body
}

pub(super) fn rtl(cx: &mut ElementContext<'_, App>, models: &CalendarHandles) -> AnyElement {
    let rtl_month = models.rtl_month.clone();
    let rtl_selected = models.rtl_selected.clone();

    let body = fret_ui_kit::primitives::direction::with_direction_provider(
        cx,
        fret_ui_kit::primitives::direction::LayoutDirection::Rtl,
        |cx| {
            shadcn::Calendar::new(rtl_month, rtl_selected)
                .test_id_prefix("ui-gallery.calendar.rtl")
                .cell_size(Px(36.0))
                .caption_layout(shadcn::CalendarCaptionLayout::Dropdown)
                .refine_style(ChromeRefinement::default().border_1().rounded(Radius::Lg))
                .into_element(cx)
        },
    );
    body
}

pub(super) fn hijri(cx: &mut ElementContext<'_, App>, models: &CalendarHandles) -> AnyElement {
    let month: Model<SolarHijriMonth> = models.hijri_month.clone();
    let selected = models.hijri_selected.clone();

    shadcn::CalendarHijri::new(month, selected)
        .cell_size(Px(38.0))
        .refine_style(ChromeRefinement::default().border_1().rounded(Radius::Lg))
        .into_element(cx)
}
