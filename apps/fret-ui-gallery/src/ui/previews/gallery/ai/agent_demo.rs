use super::super::super::super::*;

pub(in crate::ui) fn preview_ai_agent_demo(
    cx: &mut ElementContext<'_, App>,
    _theme: &Theme,
) -> Vec<AnyElement> {
    use fret_core::Px;
    use fret_ui_kit::declarative::stack;
    use fret_ui_kit::{LayoutRefinement, MetricRef, Space};
    use serde_json::json;

    let max_w = LayoutRefinement::default()
        .w_full()
        .max_w(MetricRef::Px(Px(760.0)))
        .min_w_0();

    let tool_search = ui_ai::AgentToolDefinition {
        description: Some(Arc::from("Search the codebase")),
        input_schema: json!({
            "type": "object",
            "properties": {
                "query": { "type": "string" },
                "limit": { "type": "number" }
            },
            "required": ["query"]
        }),
        json_schema: None,
    };

    let tool_read_file = ui_ai::AgentToolDefinition {
        description: Some(Arc::from("Read a file from disk")),
        input_schema: json!({
            "type": "object",
            "properties": {
                "path": { "type": "string" }
            },
            "required": ["path"]
        }),
        json_schema: None,
    };

    let tools = {
        let item_1 = ui_ai::AgentTool::new("tool-search", tool_search)
            .trigger_test_id("ui-ai-agent-demo-tool-search-trigger")
            .into_item(cx);
        let item_2 = ui_ai::AgentTool::new("tool-read-file", tool_read_file)
            .trigger_test_id("ui-ai-agent-demo-tool-read-file-trigger")
            .into_item(cx);

        let accordion =
            shadcn::Accordion::multiple_uncontrolled(["tool-search"]).items([item_1, item_2]);

        ui_ai::AgentTools::new(accordion).into_element(cx)
    };

    let content = ui_ai::AgentContent::new([
        ui_ai::AgentInstructions::new(
            "You are an AI agent integrated into an editor. Follow the user's instructions and use tools responsibly.",
        )
        .into_element(cx),
        tools,
        ui_ai::AgentOutput::new("export type Output = { summary: string; files: string[] };")
            .into_element(cx),
    ])
    .into_element(cx);

    let agent = ui_ai::Agent::new([
        ui_ai::AgentHeader::new("Fret Agent")
            .model("gpt-4.1")
            .test_id("ui-ai-agent-demo-header")
            .into_element(cx),
        content,
    ])
    .test_id("ui-ai-agent-demo-root")
    .refine_layout(max_w)
    .into_element(cx);

    vec![stack::vstack(
        cx,
        stack::VStackProps::default()
            .layout(LayoutRefinement::default().w_full().min_w_0())
            .gap(Space::N4),
        move |cx| {
            vec![
                cx.text("Agent (AI Elements)"),
                cx.text("Schema + tools disclosure chrome. Apps own tool execution."),
                agent,
            ]
        },
    )]
}
