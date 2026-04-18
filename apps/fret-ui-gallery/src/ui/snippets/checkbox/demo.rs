pub const SOURCE: &str = include_str!("demo.rs");

// region: example
use fret::{AppComponentCx, UiChild};
use fret_core::Px;
use fret_ui_shadcn::{facade as shadcn, prelude::*};

pub fn render(cx: &mut AppComponentCx<'_>) -> impl UiChild + use<> {
    let basic = cx.local_model_keyed("demo_basic", || false);
    let description = cx.local_model_keyed("demo_description", || true);
    let disabled = cx.local_model_keyed("demo_disabled", || false);
    let with_title = cx.local_model_keyed("demo_with_title", || false);

    shadcn::field_group(|cx| {
        ui::children![
            cx;
            shadcn::Field::new([
                shadcn::Checkbox::new(basic)
                    .control_id("ui-gallery-checkbox-demo-toggle")
                    .a11y_label("Accept terms and conditions")
                    .test_id("ui-gallery-checkbox-demo-toggle")
                    .into_element(cx),
                shadcn::Label::new("Accept terms and conditions")
                    .for_control("ui-gallery-checkbox-demo-toggle")
                    .test_id("ui-gallery-checkbox-demo-label")
                    .into_element(cx),
            ])
            .orientation(shadcn::FieldOrientation::Horizontal)
            .into_element(cx),
            shadcn::Field::new([
                shadcn::Checkbox::new(description)
                    .control_id("ui-gallery-checkbox-demo-description")
                    .a11y_label("Accept terms and conditions")
                    .test_id("ui-gallery-checkbox-demo-description")
                    .into_element(cx),
                shadcn::FieldContent::new([
                    shadcn::FieldLabel::new("Accept terms and conditions")
                        .for_control("ui-gallery-checkbox-demo-description")
                        .into_element(cx),
                    shadcn::FieldDescription::new(
                        "By clicking this checkbox, you agree to the terms.",
                    )
                    .into_element(cx),
                ])
                .into_element(cx),
            ])
            .orientation(shadcn::FieldOrientation::Horizontal)
            .into_element(cx),
            shadcn::Field::new([
                shadcn::Checkbox::new(disabled)
                    .control_id("ui-gallery-checkbox-demo-disabled")
                    .disabled(true)
                    .a11y_label("Enable notifications")
                    .test_id("ui-gallery-checkbox-demo-disabled")
                    .into_element(cx),
                shadcn::FieldLabel::new("Enable notifications")
                    .for_control("ui-gallery-checkbox-demo-disabled")
                    .into_element(cx),
            ])
            .disabled(true)
            .orientation(shadcn::FieldOrientation::Horizontal)
            .into_element(cx),
            shadcn::FieldLabel::new("Enable notifications")
                .for_control("ui-gallery-checkbox-demo-with-title")
                .wrap([shadcn::Field::new([
                    shadcn::Checkbox::new(with_title)
                        .control_id("ui-gallery-checkbox-demo-with-title")
                        .a11y_label("Enable notifications")
                        .test_id("ui-gallery-checkbox-demo-with-title")
                        .into_element(cx),
                    shadcn::FieldContent::new([
                        shadcn::FieldTitle::new("Enable notifications").into_element(cx),
                        shadcn::FieldDescription::new(
                            "You can enable or disable notifications at any time.",
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
    .test_id("ui-gallery-checkbox-demo")
}
// endregion: example
