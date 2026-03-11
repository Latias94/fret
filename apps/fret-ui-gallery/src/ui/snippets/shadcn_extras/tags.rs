pub const SOURCE: &str = include_str!("tags.rs");

// region: example
use fret_ui_shadcn::prelude::*;

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    fret_ui_shadcn::extras::Tags::new(["Alpha", "Beta", "Gamma", "A much longer tag label", "Zeta"])
        .into_element(cx)
        .test_id("ui-gallery-shadcn-extras-tags")
}
// endregion: example
