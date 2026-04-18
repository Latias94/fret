pub const SOURCE: &str = include_str!("agent_demo.rs");

// region: example
use fret::{AppComponentCx, UiChild};
use fret_core::Px;
use fret_ui_ai as ui_ai;
use fret_ui_kit::ui;
use fret_ui_kit::{LayoutRefinement, MetricRef, Space};
use serde_json::json;
use std::sync::Arc;

pub fn render(cx: &mut AppComponentCx<'_>) -> impl UiChild + use<> {
    let max_w = LayoutRefinement::default()
        .w_full()
        .max_w(MetricRef::Px(Px(760.0)))
        .min_w_0();

    let tool_web_search = ui_ai::AgentToolDefinition {
        description: Some(Arc::from("Search the web for information")),
        input_schema: json!({
            "type": "object",
            "properties": {
                "query": { "type": "string", "description": "The search query" }
            },
            "required": ["query"]
        }),
        json_schema: None,
    };

    let tool_read_url = ui_ai::AgentToolDefinition {
        description: Some(Arc::from("Read and parse content from a URL")),
        input_schema: json!({
            "type": "object",
            "properties": {
                "url": { "type": "string", "format": "uri", "description": "The URL to read" }
            },
            "required": ["url"]
        }),
        json_schema: None,
    };

    let tools = ui_ai::AgentTools::empty()
        .children([
            ui_ai::AgentTool::new("web_search", tool_web_search)
                .trigger_test_id("ui-ai-agent-demo-tool-search-trigger"),
            ui_ai::AgentTool::new("read_url", tool_read_url)
                .trigger_test_id("ui-ai-agent-demo-tool-read-file-trigger"),
        ])
        .into_element(cx);

    let output_schema = Arc::from(
        "z.object({\n  sentiment: z.enum(['positive', 'negative', 'neutral']),\n  score: z.number(),\n  summary: z.string(),\n})",
    );

    let content = ui_ai::AgentContent::empty()
        .children([
            ui_ai::AgentInstructions::new(
                "Analyze the sentiment of the provided text and return a structured analysis with sentiment classification, confidence score, and summary.",
            )
            .into_element(cx),
            tools,
            ui_ai::AgentOutput::new(output_schema).into_element(cx),
        ])
        .into_element(cx);

    let agent = ui_ai::Agent::empty()
        .children([
            ui_ai::AgentHeader::new("Sentiment Analyzer")
                .model("anthropic/claude-sonnet-4-5")
                .test_id("ui-ai-agent-demo-header")
                .into_element(cx),
            content,
        ])
        .test_id("ui-ai-agent-demo-root")
        .refine_layout(max_w)
        .into_element(cx);

    ui::v_flex(move |cx| {
        vec![
            cx.text("Agent"),
            cx.text(
                "Composable agent chrome with model, instructions, expandable tool schemas, and structured output.",
            ),
            agent,
        ]
    })
    .layout(LayoutRefinement::default().w_full().min_w_0())
    .gap(Space::N4)
    .into_element(cx)
}
// endregion: example
