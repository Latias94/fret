pub const SOURCE: &str = include_str!("kbd.rs");

// region: example
use fret::{AppComponentCx, UiChild};
use fret_core::Px;
use fret_icons::IconId;
use fret_ui_shadcn::{facade as shadcn, prelude::*};

pub fn render(cx: &mut AppComponentCx<'_>) -> impl UiChild + use<> {
    let value = cx.local_model(String::new);

    shadcn::InputGroup::new(value)
        .a11y_label("Search")
        .placeholder("Search...")
        .leading([icon::icon(cx, IconId::new_static("lucide.search"))])
        .trailing([
            shadcn::Kbd::new("⌘").into_element(cx),
            shadcn::Kbd::new("K").into_element(cx),
        ])
        .trailing_has_kbd(true)
        .refine_layout(LayoutRefinement::default().w_full().max_w(Px(320.0)))
        .test_id("ui-gallery-input-group-kbd")
        .into_element(cx)
}
// endregion: example
