// region: example
use fret_ui_headless::calendar::CalendarMonth;
use fret_ui_shadcn::{self as shadcn, prelude::*};
use time::Date;

pub fn render<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    date_open: Model<bool>,
    date_month: Model<CalendarMonth>,
    date: Model<Option<Date>>,
    time_value: Model<String>,
) -> AnyElement {
    let date = shadcn::DatePicker::new(date_open, date_month, date)
        .placeholder("Select date")
        .refine_layout(LayoutRefinement::default().w_px(Px(180.0)))
        .into_element(cx)
        .test_id("ui-gallery-date-picker-time-date");

    let time = shadcn::InputGroup::new(time_value)
        .a11y_label("Time")
        .control_test_id("ui-gallery-date-picker-time-control")
        .refine_layout(LayoutRefinement::default().w_px(Px(180.0)))
        .into_element(cx)
        .test_id("ui-gallery-date-picker-time-input");

    shadcn::FieldGroup::new([
        shadcn::Field::new([shadcn::FieldLabel::new("Date").into_element(cx), date])
            .into_element(cx),
        shadcn::Field::new([shadcn::FieldLabel::new("Time").into_element(cx), time])
            .into_element(cx),
    ])
    .into_element(cx)
    .test_id("ui-gallery-date-picker-time")
}
// endregion: example

