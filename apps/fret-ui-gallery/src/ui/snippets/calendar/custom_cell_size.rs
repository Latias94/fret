pub const SOURCE: &str = include_str!("custom_cell_size.rs");

// region: example
use fret::{UiChild, UiCx};
use fret_core::Px;
use fret_ui_headless::calendar::{CalendarMonth, DateRangeSelection};
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

fn today_from_env_or_now() -> Date {
    std::env::var("FRET_UI_GALLERY_FIXED_TODAY")
        .ok()
        .and_then(|raw| parse_iso_date_ymd(&raw))
        .unwrap_or_else(|| time::OffsetDateTime::now_utc().date())
}

pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    let today = today_from_env_or_now();
    let range_start = Date::from_calendar_date(today.year(), time::Month::December, 8)
        .expect("valid custom cell size start date");
    let range_end = range_start + time::Duration::days(10);
    let month = cx.local_model_keyed("month", || CalendarMonth::from_date(range_start));
    let selected = cx.local_model_keyed("selected", || DateRangeSelection {
        from: Some(range_start),
        to: Some(range_end),
    });
    let day_button = shadcn::CalendarDayButton::new().supporting_text_by(|info| {
        if !info.in_month {
            return None;
        }

        let is_weekend = matches!(
            info.date.weekday(),
            time::Weekday::Saturday | time::Weekday::Sunday
        );
        Some(if is_weekend {
            Arc::<str>::from("$120")
        } else {
            Arc::<str>::from("$100")
        })
    });

    shadcn::CalendarRange::new(month, selected)
        .test_id_prefix("ui-gallery.calendar.custom-cell")
        .day_button(day_button)
        .cell_size(Px(44.0))
        .caption_layout(shadcn::CalendarCaptionLayout::Dropdown)
        .refine_style(ChromeRefinement::default().border_1().rounded(Radius::Lg))
        .into_element(cx)
}
// endregion: example
