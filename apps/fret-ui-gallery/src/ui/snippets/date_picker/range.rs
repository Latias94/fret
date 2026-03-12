pub const SOURCE: &str = include_str!("range.rs");

// region: example
use fret_ui_headless::calendar::{CalendarMonth, DateRangeSelection};
use fret_ui_shadcn::{facade as shadcn, prelude::*};
use time::Date;

pub fn render<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    open: Model<bool>,
    month: Model<CalendarMonth>,
    selected: Model<DateRangeSelection>,
) -> AnyElement {
    let diag_calendar_roving =
        std::env::var_os("FRET_UI_GALLERY_DIAG_CALENDAR_ROVING").is_some_and(|v| !v.is_empty());

    let mut picker = shadcn::DateRangePicker::new(open, month, selected).placeholder("Pick a date");

    if diag_calendar_roving {
        let d14 = Date::from_calendar_date(2024, time::Month::February, 14).expect("valid date");
        let d15 = Date::from_calendar_date(2024, time::Month::February, 15).expect("valid date");
        picker = picker.disabled_by(move |d| d == d14 || d == d15);
    }

    picker
        .refine_layout(LayoutRefinement::default().w_px(Px(300.0)))
        .into_element(cx)
        .test_id("ui-gallery-date-picker-range")
}
// endregion: example
