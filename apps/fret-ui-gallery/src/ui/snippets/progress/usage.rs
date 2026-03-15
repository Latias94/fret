pub const SOURCE: &str = include_str!("usage.rs");

// region: example
use fret::{UiChild, UiCx};
use fret_ui_shadcn::facade as shadcn;

pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    shadcn::Progress::new(cx.local_model(|| 33.0))
        .a11y_label("Progress")
        .into_element(cx)
        .test_id("ui-gallery-progress-usage")
}
// endregion: example
