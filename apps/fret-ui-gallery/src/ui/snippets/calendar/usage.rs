pub const SOURCE: &str = include_str!("usage.rs");

// region: example
use fret::{AppComponentCx, UiChild};
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
    let selected = cx.local_model_keyed("selected", || Some(today));

    shadcn::Calendar::new_controllable(cx, Some(selected), Some(today))
        .test_id_prefix("ui-gallery.calendar.usage")
        .refine_style(ChromeRefinement::default().border_1().rounded(Radius::Lg))
        .into_element(cx)
        .test_id("ui-gallery-calendar-usage")
}
// endregion: example
