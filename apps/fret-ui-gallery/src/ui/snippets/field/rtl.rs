pub const SOURCE: &str = include_str!("rtl.rs");

// region: example
use fret::{UiChild, UiCx};
use fret_core::Px;
use fret_ui_shadcn::{facade as shadcn, prelude::*};

pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    let rtl_name = cx.local_model_keyed("rtl_name", String::new);
    let rtl_number = cx.local_model_keyed("rtl_number", String::new);
    let max_w_md = LayoutRefinement::default().w_full().max_w(Px(520.0));
    let name_id = "ui-gallery-field-rtl-card-name";
    let number_id = "ui-gallery-field-rtl-card-number";

    with_direction_provider(cx, LayoutDirection::Rtl, |cx| {
        shadcn::FieldSet::new([
            shadcn::FieldLegend::new("طريقة الدفع").into_element(cx),
            shadcn::FieldDescription::new("جميع المعاملات آمنة ومشفرة").into_element(cx),
            shadcn::FieldGroup::new([
                shadcn::Field::new([
                    shadcn::FieldLabel::new("الاسم على البطاقة")
                        .for_control(name_id)
                        .into_element(cx),
                    shadcn::Input::new(rtl_name)
                        .control_id(name_id)
                        .a11y_label("الاسم على البطاقة")
                        .placeholder("Evil Rabbit")
                        .into_element(cx),
                ])
                .into_element(cx),
                shadcn::Field::new([
                    shadcn::FieldLabel::new("رقم البطاقة")
                        .for_control(number_id)
                        .into_element(cx),
                    shadcn::Input::new(rtl_number)
                        .control_id(number_id)
                        .a11y_label("رقم البطاقة")
                        .placeholder("1234 5678 9012 3456")
                        .into_element(cx),
                ])
                .into_element(cx),
            ])
            .into_element(cx),
        ])
        .refine_layout(max_w_md)
        .into_element(cx)
    })
    .test_id("ui-gallery-field-rtl")
}
// endregion: example
