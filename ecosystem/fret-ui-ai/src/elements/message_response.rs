use std::sync::Arc;

use fret_ui::element::{AnyElement, ContainerProps};
use fret_ui::{ElementContext, UiHost};
use fret_ui_kit::declarative::style as decl_style;
use fret_ui_kit::{LayoutRefinement, Space};

/// Assistant response renderer (Markdown-first).
///
/// This is the Fret ecosystem equivalent of ai-elements' `MessageResponse` (Streamdown). It uses
/// `fret-markdown` to render markdown content and delegates code fences to `fret-code-view`.
#[derive(Clone)]
pub struct MessageResponse {
    source: Arc<str>,
    layout: LayoutRefinement,
    padding: Space,
    on_link_activate: Option<fret_markdown::OnLinkActivate>,
}

impl std::fmt::Debug for MessageResponse {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MessageResponse")
            .field("source_len", &self.source.len())
            .field("layout", &self.layout)
            .field("padding", &self.padding)
            .field("has_on_link_activate", &self.on_link_activate.is_some())
            .finish()
    }
}

impl MessageResponse {
    pub fn new(source: impl Into<Arc<str>>) -> Self {
        Self {
            source: source.into(),
            layout: LayoutRefinement::default(),
            padding: Space::N0,
            on_link_activate: None,
        }
    }

    pub fn refine_layout(mut self, layout: LayoutRefinement) -> Self {
        self.layout = self.layout.merge(layout);
        self
    }

    pub fn padding(mut self, padding: Space) -> Self {
        self.padding = padding;
        self
    }

    pub fn on_link_activate(mut self, on_link_activate: fret_markdown::OnLinkActivate) -> Self {
        self.on_link_activate = Some(on_link_activate);
        self
    }

    pub fn into_element<H: UiHost + 'static>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let theme = fret_ui::Theme::global(&*cx.app).clone();

        let mut components = fret_markdown::MarkdownComponents::<H>::default();
        components.on_link_activate = self.on_link_activate;

        let content = fret_markdown::Markdown::new(self.source).into_element_with(cx, &components);

        let root_layout = decl_style::layout_style(&theme, self.layout);
        let padding_px = decl_style::space(&theme, self.padding);

        cx.container(
            ContainerProps {
                layout: root_layout,
                padding: fret_core::Edges::all(padding_px),
                ..Default::default()
            },
            move |_cx| vec![content],
        )
    }
}
