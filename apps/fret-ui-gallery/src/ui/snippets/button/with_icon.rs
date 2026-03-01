// region: example
use fret_ui_shadcn::{self as shadcn, prelude::*};

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    shadcn::Button::new("New Branch")
        .variant(shadcn::ButtonVariant::Outline)
        .size(shadcn::ButtonSize::Sm)
        .leading_icon(IconId::new_static("lucide.git-branch"))
        .test_id("ui-gallery-button-with-icon")
        .into_element(cx)
}
// endregion: example

