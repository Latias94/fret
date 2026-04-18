pub const SOURCE: &str = include_str!("usage.rs");

// region: example
use fret::{AppComponentCx, UiChild};
use fret_ui_shadcn::facade as shadcn;

pub fn render(cx: &mut AppComponentCx<'_>) -> impl UiChild + use<> {
    shadcn::Switch::new(cx.local_model(|| false))
        .a11y_label("Airplane mode")
        .into_element(cx)
        .test_id("ui-gallery-switch-usage")
}
// endregion: example
