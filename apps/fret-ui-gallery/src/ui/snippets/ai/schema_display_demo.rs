pub const SOURCE: &str = include_str!("schema_display_demo.rs");

// region: example
use fret::{AppComponentCx, UiChild};
use fret_ui_ai as ui_ai;
use std::sync::Arc;

pub fn render(cx: &mut AppComponentCx<'_>) -> impl UiChild + use<> {
    let method = ui_ai::HttpMethod::Post;
    let path = Arc::<str>::from("/api/users/{userId}/posts");
    let description =
        Arc::<str>::from("Create a new post for a specific user. Requires authentication.");

    let parameters: Arc<[ui_ai::SchemaParameter]> = Arc::from(vec![
        ui_ai::SchemaParameter::new("userId", "string")
            .required(true)
            .description("The unique identifier of the user")
            .location(ui_ai::SchemaParameterLocation::Path),
        ui_ai::SchemaParameter::new("draft", "boolean")
            .description("Save as draft instead of publishing")
            .location(ui_ai::SchemaParameterLocation::Query),
    ]);

    let request_props: Arc<[ui_ai::SchemaProperty]> = Arc::from(vec![
        ui_ai::SchemaProperty::new("metadata", "object")
            .description("Additional metadata")
            .properties(Arc::from(vec![
                ui_ai::SchemaProperty::new("seoTitle", "string").description("SEO optimized title"),
                ui_ai::SchemaProperty::new("seoDescription", "string")
                    .description("Meta description"),
            ])),
        ui_ai::SchemaProperty::new("title", "string")
            .required(true)
            .description("The post title"),
        ui_ai::SchemaProperty::new("content", "string")
            .required(true)
            .description("The post content in markdown format"),
        ui_ai::SchemaProperty::new("tags", "array")
            .description("Tags for categorization")
            .items(ui_ai::SchemaProperty::new("tag", "string")),
    ]);

    let response_props: Arc<[ui_ai::SchemaProperty]> = Arc::from(vec![
        ui_ai::SchemaProperty::new("author", "object")
            .required(true)
            .properties(Arc::from(vec![
                ui_ai::SchemaProperty::new("id", "string").required(true),
                ui_ai::SchemaProperty::new("name", "string").required(true),
                ui_ai::SchemaProperty::new("avatar", "string"),
            ])),
        ui_ai::SchemaProperty::new("id", "string")
            .required(true)
            .description("Post ID"),
        ui_ai::SchemaProperty::new("title", "string").required(true),
        ui_ai::SchemaProperty::new("content", "string").required(true),
        ui_ai::SchemaProperty::new("createdAt", "string")
            .required(true)
            .description("ISO 8601 timestamp"),
    ]);

    ui_ai::SchemaDisplay::new(method, path.clone())
        .description(description.clone())
        .parameters(parameters)
        .request_body(request_props)
        .response_body(response_props)
        .test_id_root("ui-ai-schema-display-root")
        .test_id_parameters_trigger("ui-ai-schema-display-parameters-trigger")
        .test_id_request_trigger("ui-ai-schema-display-request-trigger")
        .test_id_response_trigger("ui-ai-schema-display-response-trigger")
        .into_element_with_children(cx, |cx| {
            vec![
                ui_ai::SchemaDisplayHeader::new([
                    ui_ai::SchemaDisplayMethod::from_context().into_element(cx),
                    ui_ai::SchemaDisplayPath::from_context().into_element(cx),
                ])
                .into_element(cx),
                ui_ai::SchemaDisplayDescription::from_context().into_element(cx),
                ui_ai::SchemaDisplayContent::new([
                    ui_ai::SchemaDisplayParameters::from_context().into_element(cx),
                    ui_ai::SchemaDisplayRequest::from_context()
                        .test_id_first_property_trigger(
                            "ui-ai-schema-display-request-prop0-trigger",
                        )
                        .test_id_first_property_child0_trigger(
                            "ui-ai-schema-display-request-prop0-child0-trigger",
                        )
                        .into_element(cx),
                    ui_ai::SchemaDisplayResponse::from_context()
                        .test_id_first_property_trigger(
                            "ui-ai-schema-display-response-prop0-trigger",
                        )
                        .test_id_first_property_child0_trigger(
                            "ui-ai-schema-display-response-prop0-child0-trigger",
                        )
                        .into_element(cx),
                ])
                .into_element(cx),
            ]
        })
}
// endregion: example
