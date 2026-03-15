pub const SOURCE: &str = include_str!("button.rs");

// region: example
use fret::{UiChild, UiCx};
use fret_core::Px;
use fret_ui_shadcn::{facade as shadcn, prelude::*};

pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    let value = cx.local_model(String::new);

    ui::v_flex(|cx| {
        vec![
            shadcn::Textarea::new(value)
                .a11y_label("Send message")
                .placeholder("Type your message here.")
                .refine_layout(LayoutRefinement::default().w_full())
                .into_element(cx),
            shadcn::Button::new("Send message").into_element(cx),
        ]
    })
    .gap(Space::N2)
    .layout(LayoutRefinement::default().w_full().max_w(Px(320.0)))
    .into_element(cx)
    .test_id("ui-gallery-textarea-button")
}
// endregion: example
