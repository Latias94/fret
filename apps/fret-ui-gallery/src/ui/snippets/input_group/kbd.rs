pub const SOURCE: &str = include_str!("kbd.rs");

// region: example
use fret::{UiChild, UiCx};
use fret_core::Px;
use fret_ui_shadcn::{facade as shadcn, prelude::*};

pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    let value = cx.local_model(String::new);

    shadcn::InputGroup::new(value)
        .a11y_label("Kbd example")
        .leading([shadcn::InputGroupText::new("Ctrl").into_element(cx)])
        .trailing([shadcn::InputGroupText::new("K").into_element(cx)])
        .leading_has_kbd(true)
        .trailing_has_kbd(true)
        .refine_layout(LayoutRefinement::default().w_full().max_w(Px(320.0)))
        .test_id("ui-gallery-input-group-kbd")
        .into_element(cx)
}
// endregion: example
