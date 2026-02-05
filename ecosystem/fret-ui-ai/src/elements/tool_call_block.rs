use std::sync::Arc;

use fret_core::SemanticsRole;
use fret_ui::element::{AnyElement, SemanticsProps};
use fret_ui::{ElementContext, UiHost};
use fret_ui_kit::{ChromeRefinement, LayoutRefinement};

use crate::elements::{Tool, ToolContent, ToolHeader, ToolInput, ToolOutput, ToolStatus};
use crate::model::ToolCall;

#[derive(Clone)]
/// A collapsible tool call block (input/output + lifecycle chrome).
///
/// This is intended to be rendered as a `MessagePart::ToolCall` inside `MessageParts`.
pub struct ToolCallBlock {
    call: ToolCall,
    default_open: bool,
    test_id_root: Option<Arc<str>>,
    test_id_trigger: Option<Arc<str>>,
    layout: LayoutRefinement,
    chrome: ChromeRefinement,
}

impl std::fmt::Debug for ToolCallBlock {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ToolCallBlock")
            .field("id", &self.call.id.as_ref())
            .field("name", &self.call.name.as_ref())
            .field("state", &self.call.state)
            .field("default_open", &self.default_open)
            .field("test_id_root", &self.test_id_root.as_deref())
            .field("test_id_trigger", &self.test_id_trigger.as_deref())
            .field("layout", &self.layout)
            .field("chrome", &self.chrome)
            .finish()
    }
}

impl ToolCallBlock {
    pub fn new(call: ToolCall) -> Self {
        Self {
            call,
            default_open: false,
            test_id_root: None,
            test_id_trigger: None,
            layout: LayoutRefinement::default(),
            chrome: ChromeRefinement::default(),
        }
    }

    pub fn default_open(mut self, open: bool) -> Self {
        self.default_open = open;
        self
    }

    pub fn test_id_root(mut self, id: impl Into<Arc<str>>) -> Self {
        self.test_id_root = Some(id.into());
        self
    }

    pub fn test_id_trigger(mut self, id: impl Into<Arc<str>>) -> Self {
        self.test_id_trigger = Some(id.into());
        self
    }

    pub fn refine_layout(mut self, layout: LayoutRefinement) -> Self {
        self.layout = self.layout.merge(layout);
        self
    }

    pub fn refine_style(mut self, chrome: ChromeRefinement) -> Self {
        self.chrome = self.chrome.merge(chrome);
        self
    }

    pub fn into_element<H: UiHost + 'static>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let call = self.call;
        let status = ToolStatus::from_tool_call_state(call.state);
        let mut header = ToolHeader::new(call.name.clone(), status);
        if let Some(test_id) = self.test_id_trigger.clone() {
            header = header.test_id(test_id);
        }

        let mut content_children: Vec<AnyElement> = Vec::new();
        if let Some(input) = call.input.clone() {
            content_children.push(ToolInput::new(input).into_element(cx));
        }
        if let Some(output) =
            ToolOutput::new(call.output.clone(), call.error.clone()).into_element(cx)
        {
            content_children.push(output);
        }
        if content_children.is_empty() {
            content_children.push(cx.text("No details."));
        }

        let content = ToolContent::new(content_children).refine_style(ChromeRefinement::default());
        let tool = Tool::new(header, content)
            .default_open(self.default_open)
            .refine_layout(self.layout.w_full())
            .refine_style(self.chrome);
        let tool = tool.into_element(cx);

        let Some(test_id) = self.test_id_root else {
            return tool;
        };
        cx.semantics(
            SemanticsProps {
                role: SemanticsRole::Group,
                test_id: Some(test_id),
                ..Default::default()
            },
            move |_cx| vec![tool],
        )
    }
}
