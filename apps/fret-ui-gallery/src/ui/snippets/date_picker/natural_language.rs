pub const SOURCE: &str = include_str!("natural_language.rs");

// region: example
use super::fixed_today;
use fret::{AppComponentCx, UiChild};
use fret_ui_headless::calendar::CalendarMonth;
use fret_ui_shadcn::{facade as shadcn, prelude::*};
use std::sync::Arc;
use time::Date;

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
            let rest = s.strip_prefix("in ")?;
            let mut parts = rest.split_whitespace();
            let n: i64 = parts.next()?.parse().ok()?;
            let unit = parts.next()?.trim_end_matches('s');

            match unit {
                "day" => base.checked_add(time::Duration::days(n)),
                "week" => base.checked_add(time::Duration::days(7 * n)),
                "month" => add_months_clamped(base, (n as i32).clamp(-1200, 1200)),
                "year" => {
                    add_months_clamped(base, (12_i32).saturating_mul((n as i32).clamp(-100, 100)))
                }
                _ => None,
            }
        }
    }
}

pub fn render(cx: &mut AppComponentCx<'_>) -> impl UiChild + use<> {
    let today = fixed_today();
    let open = cx.local_model_keyed("open", || false);
    let month = cx.local_model_keyed("month", || CalendarMonth::from_date(today));
    let selected = cx.local_model_keyed("selected", || None::<Date>);
    let value = cx.local_model_keyed("value", || String::from("In 2 days"));

    let current_value = cx
        .app
        .models()
        .read(&value, |v| v.clone())
        .unwrap_or_default();
    if let Some(parsed) = parse_natural_date_en(&current_value, today) {
        let selected_now = cx.app.models().read(&selected, |v| *v).ok().flatten();
        if selected_now != Some(parsed) {
            let _ = cx.app.models_mut().update(&selected, |v| *v = Some(parsed));
            let _ = cx
                .app
                .models_mut()
                .update(&month, |v| *v = CalendarMonth::from_date(parsed));
        }
    }

    let selected_now = cx.app.models().read(&selected, |v| *v).ok().flatten();
    let next_value = cx.slot_state(
        || None::<Date>,
        |st| {
            if selected_now != *st {
                *st = selected_now;
                Some(
                    selected_now
                        .map(format_date_month_dd_yyyy_en)
                        .unwrap_or_default(),
                )
            } else {
                None
            }
        },
    );
    if let Some(next) = next_value {
        let _ = cx.app.models_mut().update(&value, |v| *v = next);
    }

    let open_for_calendar = open.clone();
    let open_for_key = open.clone();
    let calendar_month = month.clone();
    let calendar_selected = selected.clone();
    let calendar = move |cx: &mut AppComponentCx<'_>| {
        shadcn::Calendar::new(calendar_month.clone(), calendar_selected.clone())
            .caption_layout(shadcn::CalendarCaptionLayout::Dropdown)
            .close_on_select(open_for_calendar.clone())
            .into_element(cx)
            .test_id("ui-gallery-date-picker-natural-language-calendar")
    };

    let popover = shadcn::Popover::from_open(open.clone())
        .side(shadcn::PopoverSide::Bottom)
        .align(shadcn::PopoverAlign::End)
        .side_offset(Px(8.0))
        .into_element_with(
            cx,
            move |cx| {
                let trigger = shadcn::InputGroupButton::new("")
                    .a11y_label("Select date")
                    .size(shadcn::InputGroupButtonSize::IconXs)
                    .variant(shadcn::ButtonVariant::Ghost)
                    .icon(fret_icons::IconId::new_static("lucide.calendar"))
                    .into_element(cx)
                    .test_id("ui-gallery-date-picker-natural-language-trigger");

                shadcn::PopoverTrigger::build(trigger).into_element(cx)
            },
            move |cx| {
                shadcn::PopoverContent::build(cx, |cx| [calendar(cx)])
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
        shadcn::raw::typography::muted(Arc::from(format!(
            "Your post will be published on {date_label}."
        )))
        .into_element(cx)
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
}
// endregion: example
