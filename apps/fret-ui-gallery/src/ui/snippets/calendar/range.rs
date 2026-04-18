pub const SOURCE: &str = include_str!("range.rs");

// region: example
use fret::{AppComponentCx, UiChild};
use fret_ui_headless::calendar::{CalendarMonth, DateRangeSelection};
use fret_ui_shadcn::{facade as shadcn, prelude::*};
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

pub fn render(cx: &mut AppComponentCx<'_>) -> impl UiChild + use<> {
    let today = today_from_env_or_now();
    let from = time::Date::from_calendar_date(today.year(), time::Month::January, 12)
        .expect("valid range start date");
    let to = from + time::Duration::days(30);
    let month = cx.local_model_keyed("month", || CalendarMonth::from_date(from));
    let selected = cx.local_model_keyed("selected", || DateRangeSelection {
        from: Some(from),
        to: Some(to),
    });

    shadcn::CalendarRange::new(month, selected)
        .number_of_months(2)
        .test_id_prefix("ui-gallery.calendar.range")
        .refine_style(ChromeRefinement::default().border_1().rounded(Radius::Lg))
        .into_element(cx)
}
// endregion: example
