pub const SOURCE: &str = include_str!("label.rs");

// region: example
use fret::{AppComponentCx, UiChild};
use fret_core::Px;
use fret_ui_kit::primitives::control_registry::ControlId;
use fret_ui_shadcn::{facade as shadcn, prelude::*};
use time::Date;

pub fn render(cx: &mut AppComponentCx<'_>) -> impl UiChild + use<> {
    let control_id = ControlId::from("ui-gallery-date-picker-label");

    let date_picker =
        shadcn::DatePicker::new_controllable(cx, None::<Model<Option<Date>>>, None, None, false)
            .control_id(control_id.clone())
            .test_id_prefix("ui-gallery-date-picker-label")
            .refine_layout(LayoutRefinement::default().w_full())
            .into_element(cx);

    shadcn::field_group(|cx| {
        ui::children![
            cx;
            shadcn::Field::new([
                shadcn::FieldContent::new([
                    shadcn::FieldLabel::new("Date")
                        .for_control(control_id.clone())
                        .test_id("ui-gallery-date-picker-label-label")
                        .into_element(cx),
                    shadcn::FieldDescription::new("Click the label to focus the date picker trigger.")
                        .for_control(control_id.clone())
                        .into_element(cx),
                ])
                .into_element(cx),
                date_picker,
            ]),
        ]
    })
    .refine_layout(LayoutRefinement::default().w_full().max_w(Px(320.0)))
    .into_element(cx)
    .test_id("ui-gallery-date-picker-label")
}
// endregion: example
