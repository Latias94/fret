pub const SOURCE: &str = include_str!("rtl.rs");

// region: example
use fret_ui_headless::calendar::CalendarMonth;
use fret_ui_shadcn::{facade as shadcn, prelude::*};
use time::Date;

pub fn render<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    open: Model<bool>,
    month: Model<CalendarMonth>,
    selected: Model<Option<Date>>,
) -> AnyElement {
    with_direction_provider(cx, LayoutDirection::Rtl, |cx| {
        shadcn::DatePicker::new(open, month, selected)
            .placeholder("Pick a date")
            .refine_layout(LayoutRefinement::default().w_px(Px(240.0)))
            .into_element(cx)
    })
    .test_id("ui-gallery-date-picker-rtl")
}
// endregion: example
