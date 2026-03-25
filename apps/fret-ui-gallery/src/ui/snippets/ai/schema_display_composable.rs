pub const SOURCE: &str = include_str!("schema_display_composable.rs");

// region: example
use fret::{UiChild, UiCx};
use fret_ui_ai as ui_ai;
use std::sync::Arc;

pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    let method = ui_ai::HttpMethod::Put;
    let path = Arc::<str>::from("/api/posts/{postId}");
    let description =
        Arc::<str>::from("Compose a custom request pane while keeping the root chrome.");
    let request_body: Arc<[ui_ai::SchemaProperty]> = Arc::from(vec![
        ui_ai::SchemaProperty::new("draft", "boolean")
            .description("Curated app-level publishing toggle"),
        ui_ai::SchemaProperty::new("title", "string").required(true),
    ]);

    let request_section = ui_ai::SchemaDisplayRequest::new(request_body)
        .children([
            ui_ai::SchemaDisplayProperty::new(
                ui_ai::SchemaProperty::new("draft", "boolean")
                    .description("Curated app-level publishing toggle"),
            )
            .into_element(cx),
            ui_ai::SchemaDisplayExample::new(
                "{\n  \"title\": \"Hello, Fret\",\n  \"draft\": true\n}",
            )
            .into_element(cx),
        ])
        .into_element(cx);

    ui_ai::SchemaDisplay::new(method, path.clone())
        .description(description.clone())
        .children([
            ui_ai::SchemaDisplayHeader::new([
                ui_ai::SchemaDisplayMethod::new(method).into_element(cx),
                ui_ai::SchemaDisplayPath::new(path).into_element(cx),
            ])
            .into_element(cx),
            ui_ai::SchemaDisplayDescription::new(description).into_element(cx),
            ui_ai::SchemaDisplayContent::new([request_section]).into_element(cx),
        ])
        .into_element(cx)
}
// endregion: example
