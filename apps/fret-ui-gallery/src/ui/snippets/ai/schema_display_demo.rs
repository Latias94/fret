pub const SOURCE: &str = include_str!("schema_display_demo.rs");

// region: example
use fret::{UiChild, UiCx};
use fret_ui_ai as ui_ai;
use std::sync::Arc;

pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
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

    let parameters_section = ui_ai::SchemaDisplayParameters::new(parameters.clone())
        .default_open(true)
        .test_id_trigger("ui-ai-schema-display-parameters-trigger")
        .into_element(cx);

    let request_section = ui_ai::SchemaDisplayRequest::new(request_props)
        .default_open(true)
        .test_id_trigger("ui-ai-schema-display-request-trigger")
        .test_id_first_property_trigger("ui-ai-schema-display-request-prop0-trigger")
        .test_id_first_property_child0_trigger("ui-ai-schema-display-request-prop0-child0-trigger")
        .into_element(cx);

    let response_section = ui_ai::SchemaDisplayResponse::new(response_props)
        .default_open(true)
        .test_id_trigger("ui-ai-schema-display-response-trigger")
        .test_id_first_property_trigger("ui-ai-schema-display-response-prop0-trigger")
        .test_id_first_property_child0_trigger("ui-ai-schema-display-response-prop0-child0-trigger")
        .into_element(cx);

    ui_ai::SchemaDisplay::new(method, path.clone())
        .description(description.clone())
        .parameters(parameters)
        .request_body(Arc::from([
            ui_ai::SchemaProperty::new("metadata", "object")
                .description("Additional metadata")
                .properties(Arc::from(vec![
                    ui_ai::SchemaProperty::new("seoTitle", "string")
                        .description("SEO optimized title"),
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
        ]))
        .response_body(Arc::from([
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
        ]))
        .children([
            ui_ai::SchemaDisplayHeader::new([
                ui_ai::SchemaDisplayMethod::new(method).into_element(cx),
                ui_ai::SchemaDisplayPath::new(path).into_element(cx),
            ])
            .into_element(cx),
            ui_ai::SchemaDisplayDescription::new(description).into_element(cx),
            ui_ai::SchemaDisplayContent::new([
                parameters_section,
                request_section,
                response_section,
            ])
            .into_element(cx),
        ])
        .test_id_root("ui-ai-schema-display-root")
        .into_element(cx)
}
// endregion: example
