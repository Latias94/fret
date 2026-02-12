use std::sync::Arc;

use fret_core::Px;
use fret_core::SemanticsRole;
use fret_ui::element::{AnyElement, ElementKind, SemanticsProps};
use fret_ui::{ElementContext, Theme, UiHost};
use fret_ui_kit::declarative::stack;
use fret_ui_kit::declarative::style as decl_style;
use fret_ui_kit::{
    ChromeRefinement, ColorFallback, ColorRef, Items, Justify, LayoutRefinement, Radius, Space,
};
use fret_ui_shadcn::{Separator, SeparatorOrientation};

const NODE_ACTION_MARKER_TEST_ID: &str = "fret-ui-ai.workflow.node-action-marker";

fn is_node_action_marker(element: &AnyElement) -> bool {
    element
        .semantics_decoration
        .as_ref()
        .and_then(|d| d.test_id.as_deref())
        == Some(NODE_ACTION_MARKER_TEST_ID)
        || match &element.kind {
            ElementKind::Semantics(props) => {
                props.test_id.as_deref() == Some(NODE_ACTION_MARKER_TEST_ID)
            }
            _ => false,
        }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct WorkflowNodeHandles {
    pub target: bool,
    pub source: bool,
}

/// AI Elements-aligned workflow `Node` chrome (UI-only).
///
/// Upstream reference: `repo-ref/ai-elements/packages/elements/src/node.tsx`.
///
/// Notes:
/// - Upstream uses `@xyflow/react` handles. In Fret we expose the **presence** of handles as a
///   styling seam only (small indicators), not as an interaction engine.
/// - Apps own graph layout, hit-testing, drag/drop, and zoom/pan policy.
#[derive(Clone)]
pub struct WorkflowNode {
    handles: WorkflowNodeHandles,
    children: Vec<AnyElement>,
    test_id: Option<Arc<str>>,
    layout: LayoutRefinement,
    chrome: ChromeRefinement,
}

impl std::fmt::Debug for WorkflowNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WorkflowNode")
            .field("handles", &self.handles)
            .field("children_len", &self.children.len())
            .field("test_id", &self.test_id.as_deref())
            .field("layout", &self.layout)
            .field("chrome", &self.chrome)
            .finish()
    }
}

impl WorkflowNode {
    pub fn new(children: impl IntoIterator<Item = AnyElement>) -> Self {
        Self {
            handles: WorkflowNodeHandles::default(),
            children: children.into_iter().collect(),
            test_id: None,
            layout: LayoutRefinement::default()
                .relative()
                .min_w_0()
                .min_h_0()
                .overflow_hidden(),
            chrome: ChromeRefinement::default(),
        }
    }

    pub fn handles(mut self, handles: WorkflowNodeHandles) -> Self {
        self.handles = handles;
        self
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

    #[track_caller]
    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let theme = Theme::global(&*cx.app).clone();

        let base_chrome = ChromeRefinement::default()
            .rounded(Radius::Md)
            .border_1()
            .bg(ColorRef::Token {
                key: "card",
                fallback: ColorFallback::ThemePanelBackground,
            })
            .border_color(ColorRef::Token {
                key: "border",
                fallback: ColorFallback::ThemePanelBorder,
            })
            .p(Space::N0);

        let props =
            decl_style::container_props(&theme, base_chrome.merge(self.chrome), self.layout);

        let handles = self.handles;
        let content_children = self.children;

        let body = cx.container(props, move |cx| {
            let mut out: Vec<AnyElement> = Vec::new();

            if handles.target {
                out.push(workflow_node_handle_indicator(
                    cx,
                    WorkflowNodeHandleSide::Target,
                ));
            }
            if handles.source {
                out.push(workflow_node_handle_indicator(
                    cx,
                    WorkflowNodeHandleSide::Source,
                ));
            }

            out.push(stack::vstack(
                cx,
                stack::VStackProps::default()
                    .gap(Space::N0)
                    .layout(LayoutRefinement::default().w_full().min_w_0()),
                move |_cx| content_children,
            ));

            out
        });

        let Some(test_id) = self.test_id else {
            return body;
        };
        cx.semantics(
            SemanticsProps {
                role: SemanticsRole::Group,
                test_id: Some(test_id),
                ..Default::default()
            },
            move |_cx| [body],
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum WorkflowNodeHandleSide {
    Target,
    Source,
}

fn workflow_node_handle_indicator<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    side: WorkflowNodeHandleSide,
) -> AnyElement {
    let theme = Theme::global(&*cx.app).clone();

    let indicator_diameter = Px(12.0);
    let indicator_offset = Px(6.0);

    let overlay_layout = match side {
        WorkflowNodeHandleSide::Target => LayoutRefinement::default()
            .absolute()
            .top(Space::N0)
            .bottom(Space::N0)
            .left_neg_px(indicator_offset),
        WorkflowNodeHandleSide::Source => LayoutRefinement::default()
            .absolute()
            .top(Space::N0)
            .bottom(Space::N0)
            .right_neg_px(indicator_offset),
    };

    let dot_props = decl_style::container_props(
        &theme,
        ChromeRefinement::default()
            .rounded(Radius::Full)
            .border_1()
            .bg(ColorRef::Token {
                key: "background",
                fallback: ColorFallback::ThemeSurfaceBackground,
            })
            .border_color(ColorRef::Token {
                key: "border",
                fallback: ColorFallback::ThemePanelBorder,
            }),
        LayoutRefinement::default()
            .w_px(indicator_diameter)
            .h_px(indicator_diameter)
            .min_w(indicator_diameter)
            .min_h(indicator_diameter),
    );

    let dot = cx.container(dot_props, |_cx| Vec::new());

    stack::vstack(
        cx,
        stack::VStackProps::default()
            .layout(overlay_layout)
            .items(Items::Center)
            .justify(Justify::Center),
        move |_cx| vec![dot],
    )
}

/// AI Elements-aligned workflow `NodeHeader` chrome (UI-only).
///
/// Upstream reference: `repo-ref/ai-elements/packages/elements/src/node.tsx` (`NodeHeader`).
#[derive(Clone)]
pub struct WorkflowNodeHeader {
    children: Vec<AnyElement>,
    test_id: Option<Arc<str>>,
    layout: LayoutRefinement,
    chrome: ChromeRefinement,
}

impl std::fmt::Debug for WorkflowNodeHeader {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WorkflowNodeHeader")
            .field("children_len", &self.children.len())
            .field("test_id", &self.test_id.as_deref())
            .field("layout", &self.layout)
            .field("chrome", &self.chrome)
            .finish()
    }
}

impl WorkflowNodeHeader {
    pub fn new(children: impl IntoIterator<Item = AnyElement>) -> Self {
        Self {
            children: children.into_iter().collect(),
            test_id: None,
            layout: LayoutRefinement::default().w_full().min_w_0(),
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

    #[track_caller]
    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let theme = Theme::global(&*cx.app).clone();
        let base_chrome = ChromeRefinement::default()
            .bg(ColorRef::Token {
                key: "secondary",
                fallback: ColorFallback::ThemeHoverBackground,
            })
            .p(Space::N3);
        let body_props =
            decl_style::container_props(&theme, base_chrome.merge(self.chrome), self.layout);

        let mut action: Option<AnyElement> = None;
        let mut left: Vec<AnyElement> = Vec::with_capacity(self.children.len());

        for child in self.children {
            let is_action = is_node_action_marker(&child);
            if is_action && action.is_none() {
                action = Some(child);
            } else {
                left.push(child);
            }
        }

        let content = if let Some(action) = action {
            let left_col = stack::vstack(
                cx,
                stack::VStackProps::default()
                    .gap(Space::N0p5)
                    .layout(LayoutRefinement::default().flex_1().min_w_0()),
                move |_cx| left,
            );
            stack::hstack(
                cx,
                stack::HStackProps::default()
                    .gap(Space::N2)
                    .layout(LayoutRefinement::default().w_full().min_w_0())
                    .justify_between()
                    .items_start(),
                move |_cx| vec![left_col, action],
            )
        } else {
            stack::vstack(
                cx,
                stack::VStackProps::default()
                    .gap(Space::N0p5)
                    .layout(LayoutRefinement::default().w_full().min_w_0()),
                move |_cx| left,
            )
        };

        let header_body = cx.container(body_props, move |_cx| vec![content]);

        let sep = Separator::new()
            .orientation(SeparatorOrientation::Horizontal)
            .into_element(cx);

        let mut out = stack::vstack(
            cx,
            stack::VStackProps::default()
                .gap(Space::N0)
                .layout(LayoutRefinement::default().w_full().min_w_0()),
            move |_cx| vec![header_body, sep],
        );

        if let Some(test_id) = self.test_id {
            out = out.test_id(test_id);
        }
        out
    }
}

/// Marker wrapper for a right-aligned action inside [`WorkflowNodeHeader`].
///
/// This mirrors the upstream `NodeAction` which is backed by `CardAction`.
#[derive(Clone)]
pub struct WorkflowNodeAction {
    children: Vec<AnyElement>,
    test_id: Option<Arc<str>>,
}

impl std::fmt::Debug for WorkflowNodeAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WorkflowNodeAction")
            .field("children_len", &self.children.len())
            .field("test_id", &self.test_id.as_deref())
            .finish()
    }
}

impl WorkflowNodeAction {
    pub fn new(children: impl IntoIterator<Item = AnyElement>) -> Self {
        Self {
            children: children.into_iter().collect(),
            test_id: None,
        }
    }

    pub fn test_id(mut self, id: impl Into<Arc<str>>) -> Self {
        self.test_id = Some(id.into());
        self
    }

    #[track_caller]
    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let children = self.children;
        let mut el = cx.semantics(
            SemanticsProps {
                role: fret_core::SemanticsRole::Group,
                test_id: Some(Arc::<str>::from(NODE_ACTION_MARKER_TEST_ID)),
                ..Default::default()
            },
            move |_cx| children,
        );

        if let Some(test_id) = self.test_id {
            el = el.test_id(test_id);
        }

        el
    }
}

pub type WorkflowNodeTitle = fret_ui_shadcn::CardTitle;
pub type WorkflowNodeDescription = fret_ui_shadcn::CardDescription;

/// AI Elements-aligned workflow `NodeContent` chrome (UI-only).
///
/// Upstream reference: `repo-ref/ai-elements/packages/elements/src/node.tsx` (`NodeContent`).
#[derive(Clone)]
pub struct WorkflowNodeContent {
    children: Vec<AnyElement>,
    test_id: Option<Arc<str>>,
    layout: LayoutRefinement,
    chrome: ChromeRefinement,
}

impl std::fmt::Debug for WorkflowNodeContent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WorkflowNodeContent")
            .field("children_len", &self.children.len())
            .field("test_id", &self.test_id.as_deref())
            .field("layout", &self.layout)
            .field("chrome", &self.chrome)
            .finish()
    }
}

impl WorkflowNodeContent {
    pub fn new(children: impl IntoIterator<Item = AnyElement>) -> Self {
        Self {
            children: children.into_iter().collect(),
            test_id: None,
            layout: LayoutRefinement::default().w_full().min_w_0(),
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

    #[track_caller]
    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let theme = Theme::global(&*cx.app).clone();
        let base_chrome = ChromeRefinement::default().p(Space::N3);
        let props =
            decl_style::container_props(&theme, base_chrome.merge(self.chrome), self.layout);
        let children = self.children;
        let mut el = cx.container(props, move |_cx| children);
        if let Some(test_id) = self.test_id {
            el = el.test_id(test_id);
        }
        el
    }
}

/// AI Elements-aligned workflow `NodeFooter` chrome (UI-only).
///
/// Upstream reference: `repo-ref/ai-elements/packages/elements/src/node.tsx` (`NodeFooter`).
#[derive(Clone)]
pub struct WorkflowNodeFooter {
    children: Vec<AnyElement>,
    test_id: Option<Arc<str>>,
    layout: LayoutRefinement,
    chrome: ChromeRefinement,
}

impl std::fmt::Debug for WorkflowNodeFooter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WorkflowNodeFooter")
            .field("children_len", &self.children.len())
            .field("test_id", &self.test_id.as_deref())
            .field("layout", &self.layout)
            .field("chrome", &self.chrome)
            .finish()
    }
}

impl WorkflowNodeFooter {
    pub fn new(children: impl IntoIterator<Item = AnyElement>) -> Self {
        Self {
            children: children.into_iter().collect(),
            test_id: None,
            layout: LayoutRefinement::default().w_full().min_w_0(),
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

    #[track_caller]
    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let theme = Theme::global(&*cx.app).clone();
        let base_chrome = ChromeRefinement::default()
            .bg(ColorRef::Token {
                key: "secondary",
                fallback: ColorFallback::ThemeHoverBackground,
            })
            .p(Space::N3);
        let body_props =
            decl_style::container_props(&theme, base_chrome.merge(self.chrome), self.layout);

        let sep = Separator::new()
            .orientation(SeparatorOrientation::Horizontal)
            .into_element(cx);

        let children = self.children;
        let mut footer_body = cx.container(body_props, move |_cx| children);
        if let Some(test_id) = self.test_id.clone() {
            footer_body = footer_body.test_id(test_id);
        }

        stack::vstack(
            cx,
            stack::VStackProps::default()
                .gap(Space::N0)
                .layout(LayoutRefinement::default().w_full().min_w_0()),
            move |_cx| vec![sep, footer_body],
        )
    }
}
