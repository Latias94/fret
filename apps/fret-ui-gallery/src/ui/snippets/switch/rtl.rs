pub const SOURCE: &str = include_str!("rtl.rs");

// region: example
use fret::{UiChild, UiCx};
use fret_core::Px;
use fret_ui_kit::primitives::control_registry::ControlId;
use fret_ui_shadcn::{facade as shadcn, prelude::*};

pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    let rtl = cx.local_model(|| false);
    let control_id = ControlId::from("ui-gallery-switch-rtl");

    with_direction_provider(cx, LayoutDirection::Rtl, |cx| {
        shadcn::Field::new([
            shadcn::FieldContent::new([
                shadcn::FieldLabel::new("المشاركة عبر الأجهزة")
                    .for_control(control_id.clone())
                    .test_id("ui-gallery-switch-rtl-label")
                    .into_element(cx),
                shadcn::FieldDescription::new(
                    "يتم مشاركة التركيز عبر الأجهزة، ويتم إيقاف تشغيله عند مغادرة التطبيق.",
                )
                .into_element(cx),
            ])
            .into_element(cx),
            shadcn::Switch::new(rtl)
                .control_id(control_id)
                .a11y_label("المشاركة عبر الأجهزة")
                .test_id("ui-gallery-switch-rtl-toggle")
                .into_element(cx),
        ])
        .orientation(shadcn::FieldOrientation::Horizontal)
        .refine_layout(LayoutRefinement::default().w_full().max_w(Px(384.0)))
        .into_element(cx)
    })
    .test_id("ui-gallery-switch-rtl")
}

// endregion: example
