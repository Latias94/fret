pub const SOURCE: &str = include_str!("rtl.rs");

// region: example
use fret_core::Px;
use fret_ui_shadcn::{facade as shadcn, prelude::*};

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    let value = cx.local_model(String::new);

    with_direction_provider(cx, LayoutDirection::Rtl, move |cx| {
        shadcn::InputGroup::new(value.clone())
            .a11y_label("RTL input group")
            .leading([shadcn::InputGroupText::new("lock").into_element(cx)])
            .trailing([shadcn::InputGroupText::new("sk-...").into_element(cx)])
            .refine_layout(LayoutRefinement::default().w_full().max_w(Px(320.0)))
            .into_element(cx)
    })
    .test_id("ui-gallery-input-group-rtl")
}
// endregion: example
