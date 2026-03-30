pub const SOURCE: &str = include_str!("customization.rs");

// region: example
use fret::{UiChild, UiCx};
use fret_ui_shadcn::facade as shadcn;

fn project_spinner() -> shadcn::Spinner {
    shadcn::Spinner::new().icon(fret_icons::ids::ui::SETTINGS)
}

pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    project_spinner()
        .into_element(cx)
        .test_id("ui-gallery-spinner-customization")
}
// endregion: example
