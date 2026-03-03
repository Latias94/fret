pub const SOURCE: &str = include_str!("with_title.rs");

// region: example
use fret_ui_shadcn::{self as shadcn, prelude::*};

#[derive(Default)]
struct Models {
    with_title: Option<Model<bool>>,
}

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    let with_title = cx.with_state(Models::default, |st| st.with_title.clone());
    let with_title = with_title.unwrap_or_else(|| {
        let model = cx.app.models_mut().insert(true);
        cx.with_state(Models::default, |st| st.with_title = Some(model.clone()));
        model
    });

    shadcn::FieldGroup::new([
        shadcn::FieldLabel::new("Enable notifications")
            .for_control("ui-gallery-checkbox-with-title")
            .test_id("ui-gallery-checkbox-with-title-label")
            .wrap([shadcn::Field::new([
                shadcn::Checkbox::new(with_title.clone())
                    .control_id("ui-gallery-checkbox-with-title")
                    .a11y_label("Enable notifications")
                    .test_id("ui-gallery-checkbox-with-title")
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
        shadcn::FieldLabel::new("Enable notifications (disabled)")
            .for_control("ui-gallery-checkbox-with-title-disabled")
            .test_id("ui-gallery-checkbox-with-title-disabled-label")
            .wrap([shadcn::Field::new([
                shadcn::Checkbox::new(with_title)
                    .control_id("ui-gallery-checkbox-with-title-disabled")
                    .disabled(true)
                    .a11y_label("Enable notifications (disabled)")
                    .test_id("ui-gallery-checkbox-with-title-disabled")
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
            .disabled(true)
            .into_element(cx)])
            .into_element(cx),
    ])
    .into_element(cx)
    .test_id("ui-gallery-checkbox-with-title-section")
}
// endregion: example
