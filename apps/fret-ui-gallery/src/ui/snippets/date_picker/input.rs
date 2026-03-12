pub const SOURCE: &str = include_str!("input.rs");

// region: example
use fret_ui_headless::calendar::CalendarMonth;
use fret_ui_shadcn::{facade as shadcn, prelude::*};
use std::sync::Arc;
use time::Date;

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

pub fn render<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    open: Model<bool>,
    month: Model<CalendarMonth>,
    selected: Model<Option<Date>>,
    value: Model<String>,
) -> AnyElement {
    let current_value = cx
        .app
        .models()
        .read(&value, |v| v.clone())
        .unwrap_or_default();
    if let Some(parsed) = parse_date_month_dd_yyyy_en(&current_value) {
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
    let calendar = move |cx: &mut ElementContext<'_, H>| {
        shadcn::Calendar::new(calendar_month.clone(), calendar_selected.clone())
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
        shadcn::InputGroup::new(value)
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
}
// endregion: example
