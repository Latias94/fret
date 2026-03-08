pub const SOURCE: &str = include_str!("schema_display_demo.rs");

// region: example
use fret_ui_ai as ui_ai;
use fret_ui_kit::ui;
use fret_ui_kit::{LayoutRefinement, Space};
use fret_ui_shadcn::prelude::*;
use std::sync::Arc;

pub fn render<H: UiHost + 'static>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    let request_props: Arc<[ui_ai::SchemaProperty]> = Arc::from(vec![
        ui_ai::SchemaProperty::new("request", "object")
            .required(true)
            .description("Request payload")
            .properties(Arc::from(vec![
                ui_ai::SchemaProperty::new("prompt", "string")
                    .required(true)
                    .description("User prompt"),
                ui_ai::SchemaProperty::new("temperature", "number")
                    .description("Sampling temperature"),
            ])),
    ]);

    let response_props: Arc<[ui_ai::SchemaProperty]> = Arc::from(vec![
        ui_ai::SchemaProperty::new("response", "object")
            .required(true)
            .description("Response payload")
            .properties(Arc::from(vec![
                ui_ai::SchemaProperty::new("text", "string").description("Assistant output"),
                ui_ai::SchemaProperty::new("usage", "object").properties(Arc::from(vec![
                    ui_ai::SchemaProperty::new("prompt_tokens", "number"),
                    ui_ai::SchemaProperty::new("completion_tokens", "number"),
                ])),
            ])),
    ]);

    let request_section = ui_ai::SchemaDisplayRequest::new(request_props)
        .default_open(true)
        .test_id_first_property_trigger("ui-ai-schema-display-request-prop0-trigger")
        .test_id_first_property_child0_trigger("ui-ai-schema-display-request-prop0-child0-trigger")
        .into_element(cx);

    let response_section = ui_ai::SchemaDisplayResponse::new(response_props)
        .default_open(true)
        .test_id_first_property_trigger("ui-ai-schema-display-response-prop0-trigger")
        .test_id_first_property_child0_trigger("ui-ai-schema-display-response-prop0-child0-trigger")
        .into_element(cx);

    let schema = ui_ai::SchemaDisplay::new(ui_ai::HttpMethod::Post, "/v1/chat")
        .description("SchemaDisplay is a chrome surface for request/response shapes.")
        .children([
            ui_ai::SchemaDisplayHeader::new([ui::h_row(move |cx| {
                vec![
                    ui_ai::SchemaDisplayMethod::new(ui_ai::HttpMethod::Post).into_element(cx),
                    ui_ai::SchemaDisplayPath::new(Arc::<str>::from("/v1/chat")).into_element(cx),
                ]
            })
            .gap(Space::N3)
            .items_center()
            .layout(LayoutRefinement::default().min_w_0())
            .into_element(cx)])
            .into_element(cx),
            ui_ai::SchemaDisplayContent::new([request_section, response_section]).into_element(cx),
        ])
        .test_id_root("ui-ai-schema-display-root")
        .into_element(cx);

    ui::v_flex(move |cx| {
        vec![
            cx.text("SchemaDisplay (AI Elements)"),
            cx.text("Expandable schema sections with stable per-property selectors."),
            schema,
        ]
    })
    .layout(LayoutRefinement::default().w_full().min_w_0())
    .gap(Space::N4)
    .into_element(cx)
}
// endregion: example
