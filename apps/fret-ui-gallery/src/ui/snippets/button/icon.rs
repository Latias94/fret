pub const SOURCE: &str = include_str!("icon.rs");

// region: example
use fret::{AppComponentCx, UiChild};
use fret_ui_shadcn::{facade as shadcn, prelude::*};

pub fn render(cx: &mut AppComponentCx<'_>) -> impl UiChild + use<> {
    shadcn::Button::new("")
        .variant(shadcn::ButtonVariant::Outline)
        .size(shadcn::ButtonSize::Icon)
        .a11y_label("Submit")
        .icon(IconId::new_static("lucide.arrow-up-right"))
        .test_id("ui-gallery-button-icon-only")
        .into_element(cx)
}
// endregion: example
