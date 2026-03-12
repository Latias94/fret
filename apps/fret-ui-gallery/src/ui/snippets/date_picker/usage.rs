pub const SOURCE: &str = include_str!("usage.rs");

// region: example
use fret_ui_headless::calendar::CalendarMonth;
use fret_ui_shadcn::{facade as shadcn, prelude::*};
use time::Date;

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    let today = time::OffsetDateTime::now_utc().date();
    let open = cx.local_model_keyed("open", || false);
    let month = cx.local_model_keyed("month", || CalendarMonth::from_date(today));
    let selected = cx.local_model_keyed("selected", || None::<Date>);

    shadcn::DatePicker::new(open, month, selected)
        .placeholder("Pick a date")
        .test_id_prefix("ui-gallery-date-picker-usage")
        .refine_layout(LayoutRefinement::default().w_px(Px(280.0)))
        .into_element(cx)
}
// endregion: example
