pub const SOURCE: &str = include_str!("icon.rs");

// region: example
use fret::{UiChild, UiCx};
use fret_ui_shadcn::{facade as shadcn, prelude::*};

pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    shadcn::Button::new("")
        .variant(shadcn::ButtonVariant::Outline)
        .size(shadcn::ButtonSize::Icon)
        .a11y_label("Submit")
        .icon(IconId::new_static("lucide.arrow-up-right"))
        .test_id("ui-gallery-button-icon-only")
        .into_element(cx)
}
// endregion: example
