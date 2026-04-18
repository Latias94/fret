pub const SOURCE: &str = include_str!("time_picker.rs");

// region: example
use super::{default_month, fixed_today};
use fret::{AppComponentCx, UiChild};
use fret_ui_shadcn::{facade as shadcn, prelude::*};
use time::Date;

pub fn render(cx: &mut AppComponentCx<'_>) -> impl UiChild + use<> {
    let today = fixed_today();
    let date_open = cx.local_model_keyed("date_open", || false);
    let date_month = cx.local_model_keyed("date_month", || default_month(today));
    let date = cx.local_model_keyed("date", || None::<Date>);
    let time_value = cx.local_model_keyed("time_value", || String::from("10:30:00"));

    let date = shadcn::DatePicker::new(date_open, date_month, date)
        .close_on_select(true)
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

    shadcn::field_group(|cx| {
        ui::children![
            cx;
            shadcn::Field::new([shadcn::FieldLabel::new("Date").into_element(cx), date]),
            shadcn::Field::new([shadcn::FieldLabel::new("Time").into_element(cx), time]),
        ]
    })
    .into_element(cx)
    .test_id("ui-gallery-date-picker-time")
}
// endregion: example
