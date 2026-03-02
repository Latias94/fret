pub const SOURCE: &str = include_str!("rtl.rs");

// region: example
use fret_app::App;
use fret_ui_kit::primitives::direction::{LayoutDirection, with_direction_provider};
use fret_ui_shadcn::{self as shadcn, prelude::*};

pub fn render(
    cx: &mut ElementContext<'_, App>,
    text_input: Model<String>,
    switch: Model<bool>,
    max_w_md: LayoutRefinement,
) -> AnyElement {
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
