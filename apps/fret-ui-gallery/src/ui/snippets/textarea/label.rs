pub const SOURCE: &str = include_str!("label.rs");

// region: example
use fret_core::Px;
use fret_ui_kit::primitives::control_registry::ControlId;
use fret_ui_shadcn::{facade as shadcn, prelude::*};

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    let value = cx.local_model(String::new);

    let control_id = ControlId::from("ui-gallery-textarea-label");

    shadcn::Field::new([
        shadcn::FieldLabel::new("Your message")
            .for_control(control_id.clone())
            .test_id("ui-gallery-textarea-label-label")
            .into_element(cx),
        shadcn::Textarea::new(value)
            .a11y_label("Your message")
            .placeholder("Type your message here.")
            .control_id(control_id.clone())
            .test_id("ui-gallery-textarea-label-control")
            .refine_layout(LayoutRefinement::default().w_full())
            .into_element(cx),
        shadcn::FieldDescription::new("Click the label to focus the textarea control.")
            .for_control(control_id)
            .into_element(cx),
    ])
    .refine_layout(LayoutRefinement::default().w_full().max_w(Px(320.0)))
    .into_element(cx)
    .test_id("ui-gallery-textarea-label")
}
// endregion: example
