// region: example
use fret_ui_shadcn::{self as shadcn, prelude::*};

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    shadcn::extras::Tags::new(["Alpha", "Beta", "Gamma", "A much longer tag label", "Zeta"])
        .into_element(cx)
        .test_id("ui-gallery-shadcn-extras-tags")
}
// endregion: example

