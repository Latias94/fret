pub const SOURCE: &str = include_str!("range.rs");

// region: example
use crate::ui::snippets::date_picker::{default_month, diag_calendar_roving, fixed_today};
use fret::{UiChild, UiCx};
use fret_ui_headless::calendar::DateRangeSelection;
use fret_ui_shadcn::{facade as shadcn, prelude::*};
use time::Date;

pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    let today = fixed_today();
    let open = cx.local_model_keyed("open", || false);
    let month = cx.local_model_keyed("month", || default_month(today));
    let selected = cx.local_model_keyed("selected", DateRangeSelection::default);
    let diag_calendar_roving = diag_calendar_roving();

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
