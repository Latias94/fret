pub const SOURCE: &str = include_str!("with_text.rs");

// region: example
use fret::{UiChild, UiCx};
use fret_core::Px;
use fret_ui_kit::primitives::control_registry::ControlId;
use fret_ui_shadcn::{facade as shadcn, prelude::*};

pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    let value = cx.local_model(String::new);

    let id = ControlId::from("ui-gallery-textarea-message-2");
    ui::v_flex(|cx| {
        vec![
            shadcn::Label::new("Your Message")
                .for_control(id.clone())
                .into_element(cx),
            shadcn::Textarea::new(value)
                .a11y_label("Your Message")
                .placeholder("Type your message here.")
                .control_id(id)
                .refine_layout(LayoutRefinement::default().w_full())
                .into_element(cx),
            ui::text("Your message will be copied to the support team.")
                .text_sm()
                .text_color(ColorRef::Token {
                    key: "muted-foreground",
                    fallback: fret_ui_kit::ColorFallback::ThemeTextMuted,
                })
                .w_full()
                .min_w_0()
                .into_element(cx),
        ]
    })
    .gap(Space::N3)
    .items_start()
    .layout(LayoutRefinement::default().w_full().max_w(Px(320.0)))
    .into_element(cx)
    .test_id("ui-gallery-textarea-with-text")
}
// endregion: example
