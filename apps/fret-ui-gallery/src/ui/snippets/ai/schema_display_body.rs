pub const SOURCE: &str = include_str!("schema_display_body.rs");

// region: example
use fret::{AppComponentCx, UiChild};
use fret_ui_ai as ui_ai;
use std::sync::Arc;

pub fn render(cx: &mut AppComponentCx<'_>) -> impl UiChild + use<> {
    let request_body: Arc<[ui_ai::SchemaProperty]> = Arc::from(vec![
        ui_ai::SchemaProperty::new("title", "string").required(true),
        ui_ai::SchemaProperty::new("content", "string").required(true),
    ]);
    let response_body: Arc<[ui_ai::SchemaProperty]> = Arc::from(vec![
        ui_ai::SchemaProperty::new("id", "string").required(true),
        ui_ai::SchemaProperty::new("createdAt", "string").required(true),
    ]);

    ui_ai::SchemaDisplay::new(ui_ai::HttpMethod::Post, "/api/posts")
        .request_body(request_body)
        .response_body(response_body)
        .into_element(cx)
}
// endregion: example
