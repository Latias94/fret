pub const SOURCE: &str = include_str!("presets.rs");

// region: example
use super::fixed_today;
use fret::{AppComponentCx, UiChild};
use fret_ui_headless::calendar::CalendarMonth;
use fret_ui_shadcn::{facade as shadcn, prelude::*};
use time::Date;

pub fn render(cx: &mut AppComponentCx<'_>) -> impl UiChild + use<> {
    let today = fixed_today();
    let open = cx.local_model_keyed("open", || false);
    let month = cx.local_model_keyed("month", || CalendarMonth::from_date(today));
    let selected = cx.local_model_keyed("selected", || None::<Date>);

    shadcn::DatePickerWithPresets::new(open, month, selected)
        .today(today)
        .placeholder("Pick a date")
        .refine_layout(LayoutRefinement::default().w_px(Px(240.0)))
        .into_element(cx)
        .test_id("ui-gallery-date-picker-with-presets")
}
// endregion: example
