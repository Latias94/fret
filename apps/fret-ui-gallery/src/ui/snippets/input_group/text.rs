pub const SOURCE: &str = include_str!("text.rs");

// region: example
use fret::{UiChild, UiCx};
use fret_core::Px;
use fret_ui_shadcn::{facade as shadcn, prelude::*};

pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    let value = cx.local_model(String::new);

    shadcn::InputGroup::new(value)
        .control_test_id("ui-gallery-input-group-text-control")
        .leading([shadcn::InputGroupText::new("https://")
            .into_element(cx)
            .test_id("ui-gallery-input-group-text-leading")])
        .trailing([shadcn::InputGroupText::new(".com")
            .into_element(cx)
            .test_id("ui-gallery-input-group-text-trailing")])
        .refine_layout(LayoutRefinement::default().w_full().max_w(Px(320.0)))
        .test_id("ui-gallery-input-group-text")
        .into_element(cx)
}
// endregion: example
