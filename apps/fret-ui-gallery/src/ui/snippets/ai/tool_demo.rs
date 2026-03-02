pub const SOURCE: &str = include_str!("tool_demo.rs");

// region: example
use fret_ui_ai as ui_ai;
use fret_ui_kit::declarative::stack;
use fret_ui_kit::{LayoutRefinement, Space};
use fret_ui_shadcn::prelude::*;

pub fn render<H: UiHost + 'static>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    let tool = ui_ai::Tool::new(
        ui_ai::ToolHeader::new("tool-grep", ui_ai::ToolStatus::OutputAvailable)
            .title("grep")
            .test_id("ui-ai-tool-demo-trigger"),
        ui_ai::ToolContent::new([
            cx.text("Tool content"),
            cx.text("").test_id("ui-ai-tool-demo-content-marker"),
        ]),
    )
    .into_element(cx);

    stack::vstack(
        cx,
        stack::VStackProps::default()
            .layout(LayoutRefinement::default().w_full().min_w_0())
            .gap(Space::N4),
        move |cx| {
            vec![
                cx.text("Tool (AI Elements)"),
                cx.text("Toggle the disclosure to show/hide content."),
                tool,
            ]
        },
    )
}
// endregion: example
