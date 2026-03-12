pub const SOURCE: &str = include_str!("spinner.rs");

// region: example
use fret_core::Px;
use fret_ui_shadcn::{facade as shadcn, prelude::*};

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    let value = cx.local_model(String::new);

    shadcn::InputGroup::new(value)
        .a11y_label("Spinner example")
        .leading([shadcn::Spinner::new().into_element(cx)])
        .refine_layout(LayoutRefinement::default().w_full().max_w(Px(320.0)))
        .test_id("ui-gallery-input-group-spinner")
        .into_element(cx)
}
// endregion: example
