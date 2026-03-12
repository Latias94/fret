pub const SOURCE: &str = include_str!("field.rs");

// region: example
use fret_core::Px;
use fret_ui_kit::primitives::control_registry::ControlId;
use fret_ui_shadcn::{facade as shadcn, prelude::*};

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    let value = cx.local_model(String::new);

    let id = ControlId::from("ui-gallery-textarea-message");

    ui::v_flex(|cx| {
        vec![
            shadcn::Label::new("Your message")
                .for_control(id.clone())
                .into_element(cx),
            shadcn::Textarea::new(value)
                .a11y_label("Your message")
                .placeholder("Type your message here.")
                .control_id(id)
                .refine_layout(LayoutRefinement::default().w_full())
                .into_element(cx),
        ]
    })
    .gap(Space::N3)
    .items_start()
    .layout(LayoutRefinement::default().w_full().max_w(Px(320.0)))
    .into_element(cx)
    .test_id("ui-gallery-textarea-field")
}
// endregion: example
