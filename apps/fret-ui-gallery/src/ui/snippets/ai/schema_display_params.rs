pub const SOURCE: &str = include_str!("schema_display_params.rs");

// region: example
use fret::{UiChild, UiCx};
use fret_ui_ai as ui_ai;
use std::sync::Arc;

pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    let parameters: Arc<[ui_ai::SchemaParameter]> = Arc::from(vec![
        ui_ai::SchemaParameter::new("userId", "string")
            .required(true)
            .location(ui_ai::SchemaParameterLocation::Path),
        ui_ai::SchemaParameter::new("include", "string")
            .location(ui_ai::SchemaParameterLocation::Query),
    ]);

    ui_ai::SchemaDisplay::new(ui_ai::HttpMethod::Get, "/api/users/{userId}")
        .parameters(parameters)
        .into_element(cx)
}
// endregion: example
