pub const SOURCE: &str = include_str!("schema_display_basic.rs");

// region: example
use fret::{AppComponentCx, UiChild};
use fret_ui_ai as ui_ai;

pub fn render(cx: &mut AppComponentCx<'_>) -> impl UiChild + use<> {
    ui_ai::SchemaDisplay::new(ui_ai::HttpMethod::Get, "/api/users")
        .description("List all users")
        .into_element(cx)
}
// endregion: example
