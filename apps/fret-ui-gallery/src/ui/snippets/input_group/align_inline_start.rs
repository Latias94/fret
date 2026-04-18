pub const SOURCE: &str = include_str!("align_inline_start.rs");

// region: example
use fret::{AppComponentCx, UiChild};
use fret_core::Px;
use fret_ui_shadcn::{facade as shadcn, prelude::*};

pub fn render(cx: &mut AppComponentCx<'_>) -> impl UiChild + use<> {
    let value = cx.local_model(String::new);

    shadcn::InputGroup::new(value)
        .a11y_label("Inline start addon")
        .leading([shadcn::InputGroupText::new("@").into_element(cx)])
        .refine_layout(LayoutRefinement::default().w_full().max_w(Px(320.0)))
        .test_id("ui-gallery-input-group-align-inline-start")
        .into_element(cx)
}
// endregion: example
