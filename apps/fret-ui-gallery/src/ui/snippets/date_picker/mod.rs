use fret_ui_headless::calendar::CalendarMonth;
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

pub(crate) fn fixed_today() -> Date {
    std::env::var("FRET_UI_GALLERY_FIXED_TODAY")
        .ok()
        .and_then(|raw| parse_iso_date_ymd(&raw))
        .unwrap_or_else(|| time::OffsetDateTime::now_utc().date())
}

pub(crate) fn diag_calendar_roving() -> bool {
    std::env::var_os("FRET_UI_GALLERY_DIAG_CALENDAR_ROVING").is_some_and(|value| !value.is_empty())
}

pub(crate) fn default_month(today: Date) -> CalendarMonth {
    if diag_calendar_roving() {
        CalendarMonth::from_date(
            Date::from_calendar_date(2024, time::Month::February, 1).expect("valid date"),
        )
    } else {
        CalendarMonth::from_date(today)
    }
}

pub mod basic;
pub mod demo;
pub mod dob;
pub mod dropdowns;
pub mod input;
pub mod label;
pub mod natural_language;
pub mod notes;
pub mod presets;
pub mod range;
pub mod rtl;
pub mod time_picker;
pub mod usage;
