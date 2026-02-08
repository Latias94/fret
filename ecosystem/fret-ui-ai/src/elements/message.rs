use std::sync::Arc;

use fret_core::SemanticsRole;
use fret_ui::element::{AnyElement, SemanticsProps};
use fret_ui::{ElementContext, Theme, UiHost};
use fret_ui_kit::declarative::stack;
use fret_ui_kit::declarative::style as decl_style;
use fret_ui_kit::{ChromeRefinement, ColorRef, Justify, LayoutRefinement, Radius, Space};

use crate::model::MessageRole;

#[derive(Clone)]
/// A role-aware message wrapper aligned with AI Elements `Message` (`message.tsx`).
///
/// This component is layout-only: it is responsible for alignment (user → right) and spacing
/// between message sections (content, actions, toolbars).
pub struct Message {
    from: MessageRole,
    children: Vec<AnyElement>,
    test_id: Option<Arc<str>>,
    gap: Space,
    layout: LayoutRefinement,
}

impl std::fmt::Debug for Message {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Message")
            .field("from", &self.from)
            .field("children_len", &self.children.len())
            .field("test_id", &self.test_id.as_deref())
            .field("gap", &self.gap)
            .field("layout", &self.layout)
            .finish()
    }
}

impl Message {
    pub fn new(from: MessageRole, children: impl IntoIterator<Item = AnyElement>) -> Self {
        Self {
            from,
            children: children.into_iter().collect(),
            test_id: None,
            gap: Space::N2,
            layout: LayoutRefinement::default(),
        }
    }

    pub fn test_id(mut self, id: impl Into<Arc<str>>) -> Self {
        self.test_id = Some(id.into());
        self
    }

    pub fn gap(mut self, gap: Space) -> Self {
        self.gap = gap;
        self
    }

    pub fn refine_layout(mut self, layout: LayoutRefinement) -> Self {
        self.layout = self.layout.merge(layout);
        self
    }

    pub fn into_element<H: UiHost + 'static>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let justify = if self.from == MessageRole::User {
            Justify::End
        } else {
            Justify::Start
        };

        let gap = self.gap;
        let children = self.children;
        let layout = self.layout.merge(LayoutRefinement::default().w_full());
        let inner_layout = if self.from == MessageRole::User {
            LayoutRefinement::default().min_w_0()
        } else {
            LayoutRefinement::default().w_full().min_w_0()
        };

        let inner = stack::vstack(
            cx,
            stack::VStackProps::default().layout(inner_layout).gap(gap),
            move |_cx| children,
        );

        let row = stack::hstack(
            cx,
            stack::HStackProps::default()
                .layout(layout)
                .gap(Space::N0)
                .justify(justify),
            move |_cx| vec![inner],
        );

        let Some(test_id) = self.test_id else {
            return row;
        };
        cx.semantics(
            SemanticsProps {
                role: SemanticsRole::Group,
                test_id: Some(test_id),
                ..Default::default()
            },
            move |_cx| vec![row],
        )
    }
}

#[derive(Clone)]
/// Message bubble surface aligned with AI Elements `MessageContent`.
///
/// Upstream styles user messages as a rounded bubble (`bg-secondary px-4 py-3`) and renders
/// assistant messages as plain flow content (no bubble by default).
pub struct MessageContent {
    from: MessageRole,
    children: Vec<AnyElement>,
    test_id: Option<Arc<str>>,
    layout: LayoutRefinement,
    chrome: ChromeRefinement,
}

impl std::fmt::Debug for MessageContent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MessageContent")
            .field("from", &self.from)
            .field("children_len", &self.children.len())
            .field("test_id", &self.test_id.as_deref())
            .field("layout", &self.layout)
            .field("chrome", &self.chrome)
            .finish()
    }
}

impl MessageContent {
    pub fn new(from: MessageRole, children: impl IntoIterator<Item = AnyElement>) -> Self {
        Self {
            from,
            children: children.into_iter().collect(),
            test_id: None,
            layout: LayoutRefinement::default(),
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

    pub fn into_element<H: UiHost + 'static>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let theme = Theme::global(&*cx.app).clone();

        let base_chrome = if self.from == MessageRole::User {
            let bg = theme
                .color_by_key("fret.ai.message.user.bg")
                .unwrap_or_else(|| theme.color_required("secondary"));
            ChromeRefinement::default()
                .bg(ColorRef::Color(bg))
                .px(Space::N4)
                .py(Space::N3)
                .rounded(Radius::Lg)
        } else {
            ChromeRefinement::default()
        };

        let chrome = base_chrome.merge(self.chrome);
        let base_layout = if self.from == MessageRole::User {
            // User messages should size to their bubble content.
            LayoutRefinement::default().min_w_0()
        } else {
            // Assistant/system/tool messages should participate in the full-width flow so text
            // measurement receives stable width constraints (avoids 0-width wrap explosions).
            LayoutRefinement::default().w_full().min_w_0()
        };
        let layout = base_layout.merge(self.layout);
        let children = self.children;

        let props = decl_style::container_props(&theme, chrome, layout);
        let content = cx.container(props, move |cx| {
            vec![stack::vstack(
                cx,
                stack::VStackProps::default()
                    .layout(LayoutRefinement::default().min_w_0())
                    .gap(Space::N2),
                move |_cx| children,
            )]
        });

        let Some(test_id) = self.test_id else {
            return content;
        };
        cx.semantics(
            SemanticsProps {
                role: SemanticsRole::Group,
                test_id: Some(test_id),
                ..Default::default()
            },
            move |_cx| vec![content],
        )
    }
}
