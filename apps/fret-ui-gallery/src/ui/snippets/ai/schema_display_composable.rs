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

    ui_ai::SchemaDisplay::new(method, path.clone())
        .description(description.clone())
        .request_body(request_body.clone())
        .into_element_with_children(cx, |cx| {
            vec![
                ui_ai::SchemaDisplayHeader::new([
                    ui_ai::SchemaDisplayMethod::from_context().into_element(cx),
                    ui_ai::SchemaDisplayPath::from_context().into_element(cx),
                ])
                .into_element(cx),
                ui_ai::SchemaDisplayDescription::from_context().into_element(cx),
                ui_ai::SchemaDisplayContent::new([ui_ai::SchemaDisplayRequest::from_context()
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
                    .into_element(cx)])
                .into_element(cx),
            ]
        })
}
// endregion: example
