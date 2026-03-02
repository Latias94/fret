use super::super::*;
use crate::ui::doc_layout::{self, DocSection};
use crate::ui::snippets::date_picker as snippets;

use std::sync::Arc;

use fret_ui_headless::calendar::{CalendarMonth, DateRangeSelection};

pub(super) fn preview_date_picker(
    cx: &mut ElementContext<'_, App>,
    open: Model<bool>,
    month: Model<CalendarMonth>,
    selected: Model<Option<Date>>,
) -> Vec<AnyElement> {
    #[derive(Default)]
    struct DatePickerModels {
        basic_open: Option<Model<bool>>,
        basic_month: Option<Model<CalendarMonth>>,
        basic_selected: Option<Model<Option<Date>>>,
        range_open: Option<Model<bool>>,
        range_month: Option<Model<CalendarMonth>>,
        range_selected: Option<Model<DateRangeSelection>>,
        dob_open: Option<Model<bool>>,
        dob_month: Option<Model<CalendarMonth>>,
        dob_selected: Option<Model<Option<Date>>>,
        rtl_open: Option<Model<bool>>,
        rtl_month: Option<Model<CalendarMonth>>,
        rtl_selected: Option<Model<Option<Date>>>,
        presets_open: Option<Model<bool>>,
        presets_month: Option<Model<CalendarMonth>>,
        presets_selected: Option<Model<Option<Date>>>,
        input_open: Option<Model<bool>>,
        input_month: Option<Model<CalendarMonth>>,
        input_selected: Option<Model<Option<Date>>>,
        input_value: Option<Model<String>>,
        input_last_selected: Option<Date>,
        natural_open: Option<Model<bool>>,
        natural_month: Option<Model<CalendarMonth>>,
        natural_selected: Option<Model<Option<Date>>>,
        natural_value: Option<Model<String>>,
        natural_last_selected: Option<Date>,
        time_open: Option<Model<bool>>,
        time_month: Option<Model<CalendarMonth>>,
        time_selected: Option<Model<Option<Date>>>,
        time_value: Option<Model<String>>,
    }

    fn parse_iso_date_ymd(raw: &str) -> Option<Date> {
        let raw = raw.trim();
        let (year, rest) = raw.split_once('-')?;
        let (month, day) = rest.split_once('-')?;

        let year: i32 = year.parse().ok()?;
        let month: u8 = month.parse().ok()?;
        let day: u8 = day.parse().ok()?;

        let month = time::Month::try_from(month).ok()?;
        Date::from_calendar_date(year, month, day).ok()
    }

    fn format_date_month_dd_yyyy_en(date: Date) -> String {
        use time::Month;

        let month = match date.month() {
            Month::January => "January",
            Month::February => "February",
            Month::March => "March",
            Month::April => "April",
            Month::May => "May",
            Month::June => "June",
            Month::July => "July",
            Month::August => "August",
            Month::September => "September",
            Month::October => "October",
            Month::November => "November",
            Month::December => "December",
        };

        format!("{month} {:02}, {}", date.day(), date.year())
    }

    fn parse_date_month_dd_yyyy_en(raw: &str) -> Option<Date> {
        use time::Month;

        let raw = raw.trim();
        let (month, rest) = raw.split_once(' ')?;
        let (day, year) = rest.split_once(", ")?;

        let month = match month {
            "January" => Month::January,
            "February" => Month::February,
            "March" => Month::March,
            "April" => Month::April,
            "May" => Month::May,
            "June" => Month::June,
            "July" => Month::July,
            "August" => Month::August,
            "September" => Month::September,
            "October" => Month::October,
            "November" => Month::November,
            "December" => Month::December,
            _ => return None,
        };

        let day: u8 = day.parse().ok()?;
        let year: i32 = year.parse().ok()?;
        Date::from_calendar_date(year, month, day).ok()
    }

    fn add_months_clamped(date: Date, delta_months: i32) -> Option<Date> {
        let year = date.year() as i64;
        let month0 = (date.month() as i32) - 1;
        let total_months = year.checked_mul(12)? + month0 as i64 + delta_months as i64;
        let target_year = total_months.div_euclid(12) as i32;
        let target_month0 = total_months.rem_euclid(12) as u8;
        let target_month = time::Month::try_from(target_month0 + 1).ok()?;

        let mut day = date.day();
        loop {
            if let Ok(d) = Date::from_calendar_date(target_year, target_month, day) {
                return Some(d);
            }
            day = day.checked_sub(1)?;
        }
    }

    fn parse_natural_date_en(raw: &str, base: Date) -> Option<Date> {
        let raw = raw.trim();
        if raw.is_empty() {
            return None;
        }

        if let Some(d) = parse_iso_date_ymd(raw) {
            return Some(d);
        }
        if let Some(d) = parse_date_month_dd_yyyy_en(raw) {
            return Some(d);
        }

        let s = raw.to_ascii_lowercase();
        match s.as_str() {
            "today" => Some(base),
            "tomorrow" => base.checked_add(time::Duration::days(1)),
            "yesterday" => base.checked_add(time::Duration::days(-1)),
            "next week" => base.checked_add(time::Duration::days(7)),
            "next month" => add_months_clamped(base, 1),
            "next year" => add_months_clamped(base, 12),
            _ => {
                let Some(rest) = s.strip_prefix("in ") else {
                    return None;
                };
                let mut parts = rest.split_whitespace();
                let n: i64 = parts.next()?.parse().ok()?;
                let unit = parts.next()?.trim_end_matches('s');

                match unit {
                    "day" => base.checked_add(time::Duration::days(n)),
                    "week" => base.checked_add(time::Duration::days(7 * n)),
                    "month" => add_months_clamped(base, (n as i32).clamp(-1200, 1200)),
                    "year" => add_months_clamped(
                        base,
                        (12_i32).saturating_mul((n as i32).clamp(-100, 100)),
                    ),
                    _ => None,
                }
            }
        }
    }

    let diag_calendar_roving =
        std::env::var_os("FRET_UI_GALLERY_DIAG_CALENDAR_ROVING").is_some_and(|v| !v.is_empty());

    let today = std::env::var("FRET_UI_GALLERY_FIXED_TODAY")
        .ok()
        .and_then(|raw| parse_iso_date_ymd(&raw))
        .unwrap_or_else(|| time::OffsetDateTime::now_utc().date());

    let (
        basic_open,
        basic_month,
        basic_selected,
        range_open,
        range_month,
        range_selected,
        dob_open,
        dob_month,
        dob_selected,
        rtl_open,
        rtl_month,
        rtl_selected,
        presets_open,
        presets_month,
        presets_selected,
        input_open,
        input_month,
        input_selected,
        input_value,
        natural_open,
        natural_month,
        natural_selected,
        natural_value,
        time_open,
        time_month,
        time_selected,
        time_value,
    ) = cx.with_state(DatePickerModels::default, |st| {
        (
            st.basic_open.clone(),
            st.basic_month.clone(),
            st.basic_selected.clone(),
            st.range_open.clone(),
            st.range_month.clone(),
            st.range_selected.clone(),
            st.dob_open.clone(),
            st.dob_month.clone(),
            st.dob_selected.clone(),
            st.rtl_open.clone(),
            st.rtl_month.clone(),
            st.rtl_selected.clone(),
            st.presets_open.clone(),
            st.presets_month.clone(),
            st.presets_selected.clone(),
            st.input_open.clone(),
            st.input_month.clone(),
            st.input_selected.clone(),
            st.input_value.clone(),
            st.natural_open.clone(),
            st.natural_month.clone(),
            st.natural_selected.clone(),
            st.natural_value.clone(),
            st.time_open.clone(),
            st.time_month.clone(),
            st.time_selected.clone(),
            st.time_value.clone(),
        )
    });

    let (
        basic_open,
        basic_month,
        basic_selected,
        range_open,
        range_month,
        range_selected,
        dob_open,
        dob_month,
        dob_selected,
        rtl_open,
        rtl_month,
        rtl_selected,
        presets_open,
        presets_month,
        presets_selected,
        input_open,
        input_month,
        input_selected,
        input_value,
        natural_open,
        natural_month,
        natural_selected,
        natural_value,
        time_open,
        time_month,
        time_selected,
        time_value,
    ) = match (
        basic_open,
        basic_month,
        basic_selected,
        range_open,
        range_month,
        range_selected,
        dob_open,
        dob_month,
        dob_selected,
        rtl_open,
        rtl_month,
        rtl_selected,
        presets_open,
        presets_month,
        presets_selected,
        input_open,
        input_month,
        input_selected,
        input_value,
        natural_open,
        natural_month,
        natural_selected,
        natural_value,
        time_open,
        time_month,
        time_selected,
        time_value,
    ) {
        (
            Some(basic_open),
            Some(basic_month),
            Some(basic_selected),
            Some(range_open),
            Some(range_month),
            Some(range_selected),
            Some(dob_open),
            Some(dob_month),
            Some(dob_selected),
            Some(rtl_open),
            Some(rtl_month),
            Some(rtl_selected),
            Some(presets_open),
            Some(presets_month),
            Some(presets_selected),
            Some(input_open),
            Some(input_month),
            Some(input_selected),
            Some(input_value),
            Some(natural_open),
            Some(natural_month),
            Some(natural_selected),
            Some(natural_value),
            Some(time_open),
            Some(time_month),
            Some(time_selected),
            Some(time_value),
        ) => (
            basic_open,
            basic_month,
            basic_selected,
            range_open,
            range_month,
            range_selected,
            dob_open,
            dob_month,
            dob_selected,
            rtl_open,
            rtl_month,
            rtl_selected,
            presets_open,
            presets_month,
            presets_selected,
            input_open,
            input_month,
            input_selected,
            input_value,
            natural_open,
            natural_month,
            natural_selected,
            natural_value,
            time_open,
            time_month,
            time_selected,
            time_value,
        ),
        _ => {
            let basic_open = cx.app.models_mut().insert(false);
            let range_open = cx.app.models_mut().insert(false);
            let diag_month = CalendarMonth::from_date(
                Date::from_calendar_date(2024, time::Month::February, 1).expect("valid date"),
            );
            let basic_month = cx.app.models_mut().insert(if diag_calendar_roving {
                diag_month
            } else {
                CalendarMonth::from_date(today)
            });
            let basic_selected = cx.app.models_mut().insert(None::<Date>);
            let range_month = cx.app.models_mut().insert(if diag_calendar_roving {
                diag_month
            } else {
                CalendarMonth::from_date(today)
            });
            let range_selected = cx.app.models_mut().insert(if diag_calendar_roving {
                DateRangeSelection::default()
            } else {
                DateRangeSelection::default()
            });
            let dob_open = cx.app.models_mut().insert(false);
            let dob_month = cx.app.models_mut().insert(if diag_calendar_roving {
                diag_month
            } else {
                CalendarMonth::from_date(today)
            });
            let dob_selected = cx.app.models_mut().insert(None::<Date>);
            let rtl_open = cx.app.models_mut().insert(false);
            let rtl_month = cx.app.models_mut().insert(if diag_calendar_roving {
                diag_month
            } else {
                CalendarMonth::from_date(today)
            });
            let rtl_selected = cx.app.models_mut().insert(Some(today));

            let presets_open = cx.app.models_mut().insert(false);
            let presets_month = cx.app.models_mut().insert(CalendarMonth::from_date(today));
            let presets_selected = cx.app.models_mut().insert(None::<Date>);

            let input_open = cx.app.models_mut().insert(false);
            let input_seed = Date::from_calendar_date(2025, time::Month::June, 1).expect("date");
            let input_month = cx
                .app
                .models_mut()
                .insert(CalendarMonth::from_date(input_seed));
            let input_selected = cx.app.models_mut().insert(Some(input_seed));
            let input_value = cx
                .app
                .models_mut()
                .insert(format_date_month_dd_yyyy_en(input_seed));

            let input_last_selected = Some(input_seed);

            let natural_open = cx.app.models_mut().insert(false);
            let natural_seed_value = String::from("In 2 days");
            let natural_seed_date = parse_natural_date_en(&natural_seed_value, today);
            let natural_selected = cx.app.models_mut().insert(natural_seed_date);
            let natural_month = cx
                .app
                .models_mut()
                .insert(CalendarMonth::from_date(natural_seed_date.unwrap_or(today)));
            let natural_value = cx.app.models_mut().insert(natural_seed_value);
            let natural_last_selected = natural_seed_date;

            let time_open = cx.app.models_mut().insert(false);
            let time_month = cx.app.models_mut().insert(CalendarMonth::from_date(today));
            let time_selected = cx.app.models_mut().insert(None::<Date>);
            let time_value = cx.app.models_mut().insert(String::from("10:30:00"));

            cx.with_state(DatePickerModels::default, |st| {
                st.basic_open = Some(basic_open.clone());
                st.basic_month = Some(basic_month.clone());
                st.basic_selected = Some(basic_selected.clone());
                st.range_open = Some(range_open.clone());
                st.range_month = Some(range_month.clone());
                st.range_selected = Some(range_selected.clone());
                st.dob_open = Some(dob_open.clone());
                st.dob_month = Some(dob_month.clone());
                st.dob_selected = Some(dob_selected.clone());
                st.rtl_open = Some(rtl_open.clone());
                st.rtl_month = Some(rtl_month.clone());
                st.rtl_selected = Some(rtl_selected.clone());
                st.presets_open = Some(presets_open.clone());
                st.presets_month = Some(presets_month.clone());
                st.presets_selected = Some(presets_selected.clone());
                st.input_open = Some(input_open.clone());
                st.input_month = Some(input_month.clone());
                st.input_selected = Some(input_selected.clone());
                st.input_value = Some(input_value.clone());
                st.input_last_selected = input_last_selected;
                st.natural_open = Some(natural_open.clone());
                st.natural_month = Some(natural_month.clone());
                st.natural_selected = Some(natural_selected.clone());
                st.natural_value = Some(natural_value.clone());
                st.natural_last_selected = natural_last_selected;
                st.time_open = Some(time_open.clone());
                st.time_month = Some(time_month.clone());
                st.time_selected = Some(time_selected.clone());
                st.time_value = Some(time_value.clone());
            });

            (
                basic_open,
                basic_month,
                basic_selected,
                range_open,
                range_month,
                range_selected,
                dob_open,
                dob_month,
                dob_selected,
                rtl_open,
                rtl_month,
                rtl_selected,
                presets_open,
                presets_month,
                presets_selected,
                input_open,
                input_month,
                input_selected,
                input_value,
                natural_open,
                natural_month,
                natural_selected,
                natural_value,
                time_open,
                time_month,
                time_selected,
                time_value,
            )
        }
    };

    let demo = snippets::demo::render(cx, open.clone(), month.clone(), selected.clone());
    let basic = snippets::basic::render(
        cx,
        basic_open.clone(),
        basic_month.clone(),
        basic_selected.clone(),
    );

    let mut range_picker = shadcn::DateRangePicker::new(
        range_open.clone(),
        range_month.clone(),
        range_selected.clone(),
    )
    .placeholder("Pick a date");

    if diag_calendar_roving {
        let d14 = Date::from_calendar_date(2024, time::Month::February, 14).expect("valid date");
        let d15 = Date::from_calendar_date(2024, time::Month::February, 15).expect("valid date");
        range_picker = range_picker.disabled_by(move |d| d == d14 || d == d15);
    }

    let range_picker = range_picker
        .refine_layout(LayoutRefinement::default().w_px(Px(300.0)))
        .into_element(cx)
        .test_id("ui-gallery-date-picker-range");
    let range = range_picker;

    let dob = {
        let open = dob_open.clone();
        let month = dob_month.clone();
        let selected = dob_selected.clone();

        let selected_now = cx.app.models().read(&selected, |v| *v).ok().flatten();
        if let Some(selected_now) = selected_now {
            let _ = cx
                .app
                .models_mut()
                .update(&month, |m| *m = CalendarMonth::from_date(selected_now));
        }

        let button_text = selected_now
            .map(|d| d.to_string())
            .unwrap_or_else(|| String::from("Select date"));

        shadcn::Field::new([
            shadcn::FieldLabel::new("Date of birth").into_element(cx),
            shadcn::Popover::new(open.clone())
                .side(shadcn::PopoverSide::Bottom)
                .align(shadcn::PopoverAlign::Start)
                .into_element(
                    cx,
                    |cx| {
                        shadcn::Button::new(button_text)
                            .variant(shadcn::ButtonVariant::Outline)
                            .toggle_model(open.clone())
                            .content_justify_start()
                            .text_weight(FontWeight::NORMAL)
                            .refine_layout(LayoutRefinement::default().w_px(Px(176.0)))
                            .into_element(cx)
                            .test_id("ui-gallery-date-picker-dob-trigger")
                    },
                    |cx| {
                        let calendar = shadcn::Calendar::new(month.clone(), selected.clone())
                            .caption_layout(shadcn::CalendarCaptionLayout::Dropdown)
                            .test_id_prefix("ui-gallery-date-picker-dob-calendar")
                            .close_on_select(open.clone())
                            .into_element(cx)
                            .test_id("ui-gallery-date-picker-dob-calendar");

                        shadcn::PopoverContent::new([calendar])
                            .refine_style(ChromeRefinement::default().p(Space::N0))
                            .refine_layout(
                                LayoutRefinement::default()
                                    .w(fret_ui_kit::LengthRefinement::Auto)
                                    .overflow_hidden(),
                            )
                            .into_element(cx)
                            .test_id("ui-gallery-date-picker-dob-content")
                    },
                ),
        ])
        .into_element(cx)
        .test_id("ui-gallery-date-picker-dob")
    };

    #[derive(Default)]
    struct DropdownsModels {
        open: Option<Model<bool>>,
        month: Option<Model<CalendarMonth>>,
        selected: Option<Model<Option<Date>>>,
    }

    let dropdowns = {
        let (open, month, selected) = cx.with_state(DropdownsModels::default, |st| {
            (st.open.clone(), st.month.clone(), st.selected.clone())
        });

        let (open, month, selected) = match (open, month, selected) {
            (Some(open), Some(month), Some(selected)) => (open, month, selected),
            _ => {
                let open = cx.app.models_mut().insert(false);
                let month = cx.app.models_mut().insert(CalendarMonth::from_date(today));
                let selected = cx.app.models_mut().insert(None::<Date>);

                cx.with_state(DropdownsModels::default, |st| {
                    st.open = Some(open.clone());
                    st.month = Some(month.clone());
                    st.selected = Some(selected.clone());
                });

                (open, month, selected)
            }
        };

        let is_desktop = fret_ui_kit::declarative::viewport_queries::viewport_width_at_least(
            cx,
            fret_ui::Invalidation::Layout,
            fret_ui_kit::declarative::viewport_queries::tailwind::MD,
            fret_ui_kit::declarative::viewport_queries::ViewportQueryHysteresis::default(),
        );

        let content_month = month.clone();
        let content_selected = selected.clone();
        let content = move |cx: &mut ElementContext<'_, App>| {
            shadcn::Calendar::new(content_month.clone(), content_selected.clone())
                .caption_layout(shadcn::CalendarCaptionLayout::Dropdown)
                .test_id_prefix("ui-gallery-date-picker-dropdowns-calendar")
                .into_element(cx)
        };

        let trigger_open = open.clone();
        let trigger = move |cx: &mut ElementContext<'_, App>| {
            shadcn::Button::new("Pick a date")
                .variant(shadcn::ButtonVariant::Outline)
                .toggle_model(trigger_open.clone())
                .refine_layout(LayoutRefinement::default().w_px(Px(240.0)))
                .into_element(cx)
                .test_id("ui-gallery-date-picker-dropdowns-trigger")
        };

        let overlay = if is_desktop {
            shadcn::Popover::new(open.clone())
                .side(shadcn::PopoverSide::Bottom)
                .align(shadcn::PopoverAlign::Start)
                .into_element(
                    cx,
                    move |cx| trigger(cx),
                    move |cx| {
                        shadcn::PopoverContent::new([content(cx)])
                            .refine_style(ChromeRefinement::default().p(Space::N0))
                            .refine_layout(
                                LayoutRefinement::default()
                                    .w(fret_ui_kit::LengthRefinement::Auto)
                                    .min_w_0()
                                    .min_h_0(),
                            )
                            .into_element(cx)
                            .test_id("ui-gallery-date-picker-dropdowns-popover-content")
                    },
                )
        } else {
            let done_open = open.clone();
            shadcn::Drawer::new(open.clone()).into_element(
                cx,
                move |cx| trigger(cx),
                move |cx| {
                    shadcn::DrawerContent::new([
                        content(cx),
                        shadcn::DrawerFooter::new([shadcn::Button::new("Done")
                            .variant(shadcn::ButtonVariant::Outline)
                            .toggle_model(done_open.clone())
                            .into_element(cx)
                            .test_id("ui-gallery-date-picker-dropdowns-done")])
                        .into_element(cx),
                    ])
                    .into_element(cx)
                    .test_id("ui-gallery-date-picker-dropdowns-drawer-content")
                },
            )
        };

        shadcn::Field::new([
            shadcn::FieldLabel::new("With dropdowns").into_element(cx),
            overlay,
        ])
        .into_element(cx)
        .test_id("ui-gallery-date-picker-dropdowns")
    };

    let presets = shadcn::DatePickerWithPresets::new(
        presets_open.clone(),
        presets_month.clone(),
        presets_selected.clone(),
    )
    .today(today)
    .placeholder("Pick a date")
    .refine_layout(LayoutRefinement::default().w_px(Px(240.0)))
    .into_element(cx)
    .test_id("ui-gallery-date-picker-with-presets");

    let natural_language = {
        let current_value = cx
            .app
            .models()
            .read(&natural_value, |v| v.clone())
            .unwrap_or_default();

        if let Some(parsed) = parse_natural_date_en(&current_value, today) {
            let selected_now = cx
                .app
                .models()
                .read(&natural_selected, |v| *v)
                .ok()
                .flatten();
            if selected_now != Some(parsed) {
                let _ = cx
                    .app
                    .models_mut()
                    .update(&natural_selected, |v| *v = Some(parsed));
                let _ = cx
                    .app
                    .models_mut()
                    .update(&natural_month, |v| *v = CalendarMonth::from_date(parsed));
            }
        }

        let selected_now = cx
            .app
            .models()
            .read(&natural_selected, |v| *v)
            .ok()
            .flatten();
        let next_value = cx.with_state(DatePickerModels::default, |st| {
            if selected_now != st.natural_last_selected {
                st.natural_last_selected = selected_now;
                Some(
                    selected_now
                        .map(format_date_month_dd_yyyy_en)
                        .unwrap_or_default(),
                )
            } else {
                None
            }
        });
        if let Some(next) = next_value {
            let _ = cx.app.models_mut().update(&natural_value, |v| *v = next);
        }

        let open_for_calendar = natural_open.clone();
        let open_for_key = natural_open.clone();
        let month = natural_month.clone();
        let selected = natural_selected.clone();
        let value = natural_value.clone();

        let calendar = move |cx: &mut ElementContext<'_, App>| {
            shadcn::Calendar::new(month.clone(), selected.clone())
                .caption_layout(shadcn::CalendarCaptionLayout::Dropdown)
                .close_on_select(open_for_calendar.clone())
                .into_element(cx)
                .test_id("ui-gallery-date-picker-natural-language-calendar")
        };

        let popover = shadcn::Popover::new(natural_open.clone())
            .side(shadcn::PopoverSide::Bottom)
            .align(shadcn::PopoverAlign::End)
            .side_offset(Px(8.0))
            .into_element(
                cx,
                move |cx| {
                    let trigger = shadcn::InputGroupButton::new("")
                        .a11y_label("Select date")
                        .size(shadcn::InputGroupButtonSize::IconXs)
                        .variant(shadcn::ButtonVariant::Ghost)
                        .icon(fret_icons::IconId::new_static("lucide.calendar"))
                        .into_element(cx)
                        .test_id("ui-gallery-date-picker-natural-language-trigger");

                    shadcn::PopoverTrigger::new(trigger).into_element(cx)
                },
                move |cx| {
                    shadcn::PopoverContent::new([calendar(cx)])
                        .refine_style(ChromeRefinement::default().p(Space::N0))
                        .refine_layout(
                            LayoutRefinement::default()
                                .w(fret_ui_kit::LengthRefinement::Auto)
                                .overflow_hidden(),
                        )
                        .into_element(cx)
                        .test_id("ui-gallery-date-picker-natural-language-content")
                },
            );

        let helper_text = {
            let date_label = selected_now
                .map(format_date_month_dd_yyyy_en)
                .unwrap_or_else(|| String::from("—"));
            shadcn::typography::muted(
                cx,
                Arc::from(format!("Your post will be published on {date_label}.")),
            )
        };

        shadcn::Field::new([
            shadcn::FieldLabel::new("Schedule Date").into_element(cx),
            shadcn::InputGroup::new(value)
                .a11y_label("Schedule Date")
                .control_test_id("ui-gallery-date-picker-natural-language-control")
                .control_on_key_down(Arc::new(move |host, _acx, kcx| {
                    if kcx.key == fret_core::KeyCode::ArrowDown {
                        let _ = host.models_mut().update(&open_for_key, |v| *v = true);
                        return true;
                    }
                    false
                }))
                .trailing([popover])
                .trailing_has_button(true)
                .refine_layout(LayoutRefinement::default().w_px(Px(280.0)))
                .into_element(cx),
            helper_text,
        ])
        .refine_layout(LayoutRefinement::default().w_full().max_w(Px(320.0)))
        .into_element(cx)
        .test_id("ui-gallery-date-picker-natural-language")
    };

    let input = {
        let current_value = cx
            .app
            .models()
            .read(&input_value, |v| v.clone())
            .unwrap_or_default();

        if let Some(parsed) = parse_date_month_dd_yyyy_en(&current_value) {
            let selected_now = cx.app.models().read(&input_selected, |v| *v).ok().flatten();
            if selected_now != Some(parsed) {
                let _ = cx
                    .app
                    .models_mut()
                    .update(&input_selected, |v| *v = Some(parsed));
                let _ = cx
                    .app
                    .models_mut()
                    .update(&input_month, |v| *v = CalendarMonth::from_date(parsed));
            }
        }

        let selected_now = cx.app.models().read(&input_selected, |v| *v).ok().flatten();
        let next_value = cx.with_state(DatePickerModels::default, |st| {
            if selected_now != st.input_last_selected {
                st.input_last_selected = selected_now;
                Some(
                    selected_now
                        .map(format_date_month_dd_yyyy_en)
                        .unwrap_or_default(),
                )
            } else {
                None
            }
        });
        if let Some(next) = next_value {
            let _ = cx.app.models_mut().update(&input_value, |v| *v = next);
        }

        let open = input_open.clone();
        let open_for_calendar = open.clone();
        let open_for_key = open.clone();
        let month = input_month.clone();
        let selected = input_selected.clone();
        let calendar = move |cx: &mut ElementContext<'_, App>| {
            shadcn::Calendar::new(month.clone(), selected.clone())
                .close_on_select(open_for_calendar.clone())
                .into_element(cx)
                .test_id("ui-gallery-date-picker-input-calendar")
        };

        let popover = shadcn::Popover::new(open.clone())
            .side(shadcn::PopoverSide::Bottom)
            .align(shadcn::PopoverAlign::End)
            .align_offset(Px(-8.0))
            .side_offset(Px(10.0))
            .into_element(
                cx,
                move |cx| {
                    let trigger = shadcn::InputGroupButton::new("")
                        .a11y_label("Select date")
                        .size(shadcn::InputGroupButtonSize::IconSm)
                        .icon(fret_icons::IconId::new_static("lucide.calendar"))
                        .into_element(cx)
                        .test_id("ui-gallery-date-picker-input-trigger");

                    shadcn::PopoverTrigger::new(trigger).into_element(cx)
                },
                move |cx| {
                    shadcn::PopoverContent::new([calendar(cx)])
                        .refine_style(ChromeRefinement::default().p(Space::N0))
                        .refine_layout(
                            LayoutRefinement::default().w(fret_ui_kit::LengthRefinement::Auto),
                        )
                        .into_element(cx)
                        .test_id("ui-gallery-date-picker-input-content")
                },
            );

        shadcn::Field::new([
            shadcn::FieldLabel::new("Subscription Date").into_element(cx),
            shadcn::InputGroup::new(input_value.clone())
                .a11y_label("Subscription Date")
                .control_test_id("ui-gallery-date-picker-input-control")
                .control_on_key_down(Arc::new(move |host, _acx, kcx| {
                    if kcx.key == fret_core::KeyCode::ArrowDown {
                        let _ = host.models_mut().update(&open_for_key, |v| *v = true);
                        return true;
                    }
                    false
                }))
                .trailing([popover])
                .trailing_has_button(true)
                .refine_layout(LayoutRefinement::default().w_px(Px(240.0)))
                .into_element(cx),
        ])
        .into_element(cx)
        .test_id("ui-gallery-date-picker-input")
    };

    let time_picker = {
        let date = shadcn::DatePicker::new(time_open.clone(), time_month.clone(), time_selected)
            .placeholder("Select date")
            .refine_layout(LayoutRefinement::default().w_px(Px(180.0)))
            .into_element(cx)
            .test_id("ui-gallery-date-picker-time-date");

        let time = shadcn::InputGroup::new(time_value.clone())
            .a11y_label("Time")
            .control_test_id("ui-gallery-date-picker-time-control")
            .refine_layout(LayoutRefinement::default().w_px(Px(180.0)))
            .into_element(cx)
            .test_id("ui-gallery-date-picker-time-input");

        shadcn::FieldGroup::new([
            shadcn::Field::new([shadcn::FieldLabel::new("Date").into_element(cx), date])
                .into_element(cx),
            shadcn::Field::new([shadcn::FieldLabel::new("Time").into_element(cx), time])
                .into_element(cx),
        ])
        .into_element(cx)
        .test_id("ui-gallery-date-picker-time")
    };

    let rtl = doc_layout::rtl(cx, |cx| {
        shadcn::DatePicker::new(rtl_open.clone(), rtl_month.clone(), rtl_selected.clone())
            .placeholder("Pick a date")
            .refine_layout(LayoutRefinement::default().w_px(Px(240.0)))
            .into_element(cx)
    })
    .test_id("ui-gallery-date-picker-rtl");

    let notes_stack = doc_layout::notes(
        cx,
        [
            "This page mirrors shadcn Date Picker docs (new-york-v4) and keeps the diag suite stable.",
            "Calendar dropdown caption improves large-jump navigation compared with arrow-only controls.",
            "For diag runs, some dates are intentionally disabled (via env flag) to validate skip behavior.",
            "Natural language picker uses a small built-in parser (subset of chrono-node behavior).",
        ],
    );

    let body = doc_layout::render_doc_page(
        cx,
        Some(
            "Preview mirrors shadcn Date Picker docs (Basic + Range + Date of birth + Input + Time + Natural language + RTL). Extra: presets.",
        ),
        vec![
            DocSection::new("Demo", demo)
                .description("A compact date picker trigger (docs: Date Picker demo).")
                .code_rust_from_file_region(include_str!("../snippets/date_picker/demo.rs"), "example")
                .max_w(Px(980.0))
                .no_shell(),
            DocSection::new("Basic", basic)
                .description("A basic date picker component (docs: Date Picker Basic).")
                .code_rust_from_file_region(
                    include_str!("../snippets/date_picker/basic.rs"),
                    "example",
                )
                .max_w(Px(980.0))
                .no_shell(),
            DocSection::new("Range Picker", range)
                .description("A date picker component for selecting a range of dates.")
                .code(
                    "rust",
                    r#"shadcn::DateRangePicker::new(open, month, range_selected)
    .placeholder("Pick a date")
    .into_element(cx);"#,
                )
                .max_w(Px(980.0))
                .no_shell(),
            DocSection::new("Date of Birth", dob)
                .description(
                    "A date picker component with a dropdown caption layout for month/year selection.",
                )
                .code(
                    "rust",
                    r#"let open = cx.app.models_mut().insert(false);
let month = cx.app.models_mut().insert(CalendarMonth::from_date(today));
let selected = cx.app.models_mut().insert(None::<Date>);

shadcn::Field::new([
    shadcn::FieldLabel::new("Date of birth").into_element(cx),
    shadcn::Popover::new(open.clone()).into_element(
        cx,
        |cx| {
            shadcn::Button::new("Select date")
                .variant(shadcn::ButtonVariant::Outline)
                .toggle_model(open.clone())
                .into_element(cx)
        },
        |cx| {
            let calendar = shadcn::Calendar::new(month, selected)
                .caption_layout(shadcn::CalendarCaptionLayout::Dropdown)
                .close_on_select(open)
                .into_element(cx);

            shadcn::PopoverContent::new([calendar])
                .refine_style(ChromeRefinement::default().p(Space::N0))
                .refine_layout(LayoutRefinement::default().w(LengthRefinement::Auto))
                .into_element(cx)
        },
    ),
])
.into_element(cx);"#,
                )
                .max_w(Px(780.0))
                .no_shell(),
            DocSection::new("Input", input)
                .description(
                    "InputGroup + calendar button + popover calendar (docs: Date Picker Input).",
                )
                .code(
                    "rust",
                    r#"let open = cx.app.models_mut().insert(false);
let date = cx.app.models_mut().insert(None::<Date>);
let month = cx.app.models_mut().insert(CalendarMonth::from_date(today));
let value = cx.app.models_mut().insert(String::new());

let calendar = move |cx: &mut ElementContext<'_, App>| {
    shadcn::Calendar::new(month.clone(), date.clone())
        .close_on_select(open.clone())
        .into_element(cx)
};

let popover = shadcn::Popover::new(open.clone()).into_element(
    cx,
    move |cx| {
        let trigger = shadcn::InputGroupButton::new("")
            .a11y_label("Select date")
            .size(shadcn::InputGroupButtonSize::IconSm)
            .icon(fret_icons::IconId::new_static("lucide.calendar"))
            .into_element(cx);
        shadcn::PopoverTrigger::new(trigger).into_element(cx)
    },
    move |cx| {
        shadcn::PopoverContent::new([calendar(cx)])
            .refine_style(ChromeRefinement::default().p(Space::N0))
            .refine_layout(LayoutRefinement::default().w(LengthRefinement::Auto))
            .into_element(cx)
    },
);

shadcn::InputGroup::new(value)
    .a11y_label("Subscription Date")
    .trailing([popover])
    .trailing_has_button(true)
    .into_element(cx);"#,
                )
                .max_w(Px(780.0))
                .no_shell(),
            DocSection::new("Time Picker", time_picker)
                .description("Date + time fields side-by-side (docs: Date Picker Time).")
                .code(
                    "rust",
                    r#"let date_open = cx.app.models_mut().insert(false);
let date_month = cx.app.models_mut().insert(CalendarMonth::from_date(today));
let date = cx.app.models_mut().insert(None::<Date>);
let time = cx.app.models_mut().insert(String::from("10:30:00"));

let date_picker = shadcn::DatePicker::new(date_open, date_month, date)
    .placeholder("Select date")
    .into_element(cx);

let time_input = shadcn::InputGroup::new(time)
    .a11y_label("Time")
    .into_element(cx);

shadcn::FieldGroup::new([
    shadcn::Field::new([shadcn::FieldLabel::new("Date").into_element(cx), date_picker])
        .into_element(cx),
    shadcn::Field::new([shadcn::FieldLabel::new("Time").into_element(cx), time_input])
        .into_element(cx),
])
.into_element(cx);"#,
                )
                .max_w(Px(780.0))
                .no_shell(),
            DocSection::new("Natural Language Picker", natural_language)
                .description("This example parses natural language into a date (subset).")
                .code(
                    "rust",
                    r#"shadcn::InputGroup::new(value)
    .control_on_key_down(Arc::new(|host, _, kcx| {
        if kcx.key == KeyCode::ArrowDown {
            let _ = host.models_mut().update(&open, |v| *v = true);
            return true;
        }
        false
    }))
    .trailing([popover])
    .trailing_has_button(true)
    .into_element(cx);"#,
                )
                .max_w(Px(780.0))
                .no_shell(),
            DocSection::new("RTL", rtl)
                .description("All shadcn components should work under an RTL direction provider.")
                .code(
                    "rust",
                    r#"with_direction_provider(LayoutDirection::Rtl, |cx| {
    shadcn::DatePicker::new(open, month, selected).into_element(cx)
});"#,
                )
                .max_w(Px(780.0))
                .no_shell(),
            DocSection::new("Extras: With Presets", presets)
                .description("shadcn `date-picker-with-presets` (Select + Calendar in a popover).")
                .code(
                    "rust",
                    r#"shadcn::DatePickerWithPresets::new(open, month, selected)
    .today(today) // optional: make presets deterministic
    .into_element(cx);"#,
                )
                .max_w(Px(780.0))
                .no_shell(),
            DocSection::new("Extras: With Dropdowns", dropdowns)
                .description(
                    "Gallery-only: desktop uses a Popover; mobile uses a Drawer. Calendar caption uses dropdown month/year selection.",
                )
                .code(
                    "rust",
                    r#"let is_desktop = viewport_queries::viewport_width_at_least(
    cx,
    Invalidation::Layout,
    viewport_queries::tailwind::MD,
    viewport_queries::ViewportQueryHysteresis::default(),
);

let calendar = shadcn::Calendar::new(month, selected)
    .caption_layout(shadcn::CalendarCaptionLayout::Dropdown)
    .into_element(cx);

if is_desktop {
    shadcn::Popover::new(open).into_element(cx, trigger, |_| calendar)
} else {
    shadcn::Drawer::new(open).into_element(cx, trigger, |_| calendar)
};"#,
                )
                .max_w(Px(780.0))
                .no_shell(),
            DocSection::new("Notes", notes_stack)
                .description("Guidelines and parity notes for date picker recipes.")
                .max_w(Px(780.0)),
        ],
    );

    vec![body]
}
