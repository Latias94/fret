pub const SOURCE: &str = include_str!("rtl.rs");

// region: example
use fret::{UiChild, UiCx};
use fret_core::Px;
use fret_ui_shadcn::{facade as shadcn, prelude::*};

pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    let basic = cx.local_model_keyed("rtl_basic", || false);
    let description = cx.local_model_keyed("rtl_description", || true);
    let disabled = cx.local_model_keyed("rtl_disabled", || false);
    let with_title = cx.local_model_keyed("rtl_with_title", || false);

    with_direction_provider(cx, LayoutDirection::Rtl, |cx| {
        shadcn::field_group(|cx| {
            ui::children![
                cx;
                shadcn::Field::new([
                    shadcn::Checkbox::new(basic)
                        .control_id("ui-gallery-checkbox-rtl-basic")
                        .a11y_label("قبول الشروط والأحكام")
                        .test_id("ui-gallery-checkbox-rtl-basic")
                        .into_element(cx),
                    shadcn::Label::new("قبول الشروط والأحكام")
                        .for_control("ui-gallery-checkbox-rtl-basic")
                        .test_id("ui-gallery-checkbox-rtl-basic-label")
                        .into_element(cx),
                ])
                .orientation(shadcn::FieldOrientation::Horizontal)
                .into_element(cx),
                shadcn::Field::new([
                    shadcn::Checkbox::new(description)
                        .control_id("ui-gallery-checkbox-rtl-description")
                        .a11y_label("قبول الشروط والأحكام")
                        .test_id("ui-gallery-checkbox-rtl-description")
                        .into_element(cx),
                    shadcn::FieldContent::new([
                        shadcn::FieldLabel::new("قبول الشروط والأحكام")
                            .for_control("ui-gallery-checkbox-rtl-description")
                            .into_element(cx),
                        shadcn::FieldDescription::new("بالنقر على هذا المربع، فإنك توافق على الشروط.")
                            .into_element(cx),
                    ])
                    .into_element(cx),
                ])
                .orientation(shadcn::FieldOrientation::Horizontal)
                .into_element(cx),
                shadcn::Field::new([
                    shadcn::Checkbox::new(disabled)
                        .control_id("ui-gallery-checkbox-rtl-disabled")
                        .disabled(true)
                        .a11y_label("تفعيل الإشعارات")
                        .test_id("ui-gallery-checkbox-rtl-disabled")
                        .into_element(cx),
                    shadcn::FieldLabel::new("تفعيل الإشعارات")
                        .for_control("ui-gallery-checkbox-rtl-disabled")
                        .into_element(cx),
                ])
                .disabled(true)
                .orientation(shadcn::FieldOrientation::Horizontal)
                .into_element(cx),
                shadcn::FieldLabel::new("تفعيل الإشعارات")
                    .for_control("ui-gallery-checkbox-rtl-with-title")
                    .test_id("ui-gallery-checkbox-rtl-with-title-label")
                    .wrap([shadcn::Field::new([
                        shadcn::Checkbox::new(with_title)
                            .control_id("ui-gallery-checkbox-rtl-with-title")
                            .a11y_label("تفعيل الإشعارات")
                            .test_id("ui-gallery-checkbox-rtl-with-title")
                            .into_element(cx),
                        shadcn::FieldContent::new([
                            shadcn::FieldTitle::new("تفعيل الإشعارات").into_element(cx),
                            shadcn::FieldDescription::new(
                                "يمكنك تفعيل أو إلغاء تفعيل الإشعارات في أي وقت.",
                            )
                            .into_element(cx),
                        ])
                        .into_element(cx),
                    ])
                    .orientation(shadcn::FieldOrientation::Horizontal)
                    .into_element(cx)])
                    .into_element(cx),
            ]
        })
        .refine_layout(LayoutRefinement::default().w_full().max_w(Px(384.0)))
        .into_element(cx)
    })
    .test_id("ui-gallery-checkbox-rtl")
}
// endregion: example
