pub const SOURCE: &str = include_str!("rtl.rs");

// region: example
use fret::{UiChild, UiCx};
use fret_ui_shadcn::{facade as shadcn, prelude::*};

pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    let text_input = cx.local_model_keyed("text_input", String::new);
    let switch = cx.local_model_keyed("switch", || false);

    let max_w_md = LayoutRefinement::default()
        .w_full()
        .min_w_0()
        .max_w(Px(520.0));

    with_direction_provider(cx, LayoutDirection::Rtl, |cx| {
        shadcn::FieldSet::new([
            shadcn::FieldLegend::new("الملف الشخصي").into_element(cx),
            shadcn::FieldDescription::new("تحقق من محاذاة الحقول والنصوص تحت RTL.")
                .into_element(cx),
            shadcn::Field::new([
                shadcn::FieldLabel::new("البريد الإلكتروني").into_element(cx),
                shadcn::Input::new(text_input.clone())
                    .a11y_label("البريد الإلكتروني")
                    .placeholder("name@example.com")
                    .into_element(cx),
            ])
            .into_element(cx),
            shadcn::Field::new([
                shadcn::FieldLabel::new("تفعيل الإشعارات")
                    .for_control("ui-gallery-form-switch-rtl")
                    .into_element(cx),
                shadcn::Switch::new(switch.clone())
                    .control_id("ui-gallery-form-switch-rtl")
                    .a11y_label("تفعيل الإشعارات")
                    .into_element(cx),
            ])
            .orientation(shadcn::FieldOrientation::Horizontal)
            .into_element(cx),
        ])
        .refine_layout(max_w_md)
        .into_element(cx)
    })
    .test_id("ui-gallery-form-rtl")
}
// endregion: example
