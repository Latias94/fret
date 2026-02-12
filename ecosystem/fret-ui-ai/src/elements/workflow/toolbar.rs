use std::sync::Arc;

use fret_core::SemanticsRole;
use fret_ui::element::{AnyElement, SemanticsProps};
use fret_ui::{ElementContext, Theme, UiHost};
use fret_ui_kit::declarative::stack;
use fret_ui_kit::declarative::style as decl_style;
use fret_ui_kit::{
    ChromeRefinement, ColorFallback, ColorRef, Items, LayoutRefinement, Radius, Space,
};

/// AI Elements-aligned workflow `Toolbar` chrome (UI-only).
///
/// Upstream reference: `repo-ref/ai-elements/packages/elements/src/toolbar.tsx`.
///
/// Notes:
/// - Upstream is `@xyflow/react`-backed (`NodeToolbar`) and handles placement automatically.
/// - In Fret this is a styling/composition wrapper only; apps own positioning.
#[derive(Clone)]
pub struct WorkflowToolbar {
    children: Vec<AnyElement>,
    test_id: Option<Arc<str>>,
    layout: LayoutRefinement,
    chrome: ChromeRefinement,
}

impl std::fmt::Debug for WorkflowToolbar {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WorkflowToolbar")
            .field("children_len", &self.children.len())
            .field("test_id", &self.test_id.as_deref())
            .field("layout", &self.layout)
            .field("chrome", &self.chrome)
            .finish()
    }
}

impl WorkflowToolbar {
    pub fn new(children: impl IntoIterator<Item = AnyElement>) -> Self {
        Self {
            children: children.into_iter().collect(),
            test_id: None,
            layout: LayoutRefinement::default().min_w_0(),
            chrome: ChromeRefinement::default(),
        }
    }

    pub fn test_id(mut self, id: impl Into<Arc<str>>) -> Self {
        self.test_id = Some(id.into());
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

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let theme = Theme::global(&*cx.app).clone();
        let base_chrome = ChromeRefinement::default()
            .rounded(Radius::Sm)
            .border_1()
            .bg(ColorRef::Token {
                key: "background",
                fallback: ColorFallback::ThemePanelBackground,
            })
            .border_color(ColorRef::Token {
                key: "border",
                fallback: ColorFallback::ThemePanelBorder,
            })
            .p(Space::N1p5);

        let children = self.children;
        let row = stack::hstack(
            cx,
            stack::HStackProps::default()
                .gap(Space::N1)
                .items(Items::Center)
                .layout(LayoutRefinement::default().min_w_0()),
            move |_cx| children,
        );

        let props =
            decl_style::container_props(&theme, base_chrome.merge(self.chrome), self.layout);
        let body = cx.container(props, move |_cx| [row]);

        let Some(test_id) = self.test_id else {
            return body;
        };
        cx.semantics(
            SemanticsProps {
                role: SemanticsRole::Toolbar,
                test_id: Some(test_id),
                ..Default::default()
            },
            move |_cx| [body],
        )
    }
}
