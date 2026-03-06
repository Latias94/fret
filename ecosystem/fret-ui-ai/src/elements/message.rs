use std::sync::Arc;

use fret_core::SemanticsRole;
use fret_ui::element::{AnyElement, SemanticsProps};
use fret_ui::{ElementContext, Theme, UiHost};
use fret_ui_kit::declarative::style as decl_style;
use fret_ui_kit::ui;
use fret_ui_kit::{ChromeRefinement, ColorRef, Justify, LayoutRefinement, Radius, Space};

use crate::model::MessageRole;

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

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let justify = if self.from == MessageRole::User {
            Justify::End
        } else {
            Justify::Start
        };

        let gap = self.gap;
        let children = self.children;
        let layout = {
            let base = LayoutRefinement::default().w_full().max_w_percent(95.0);
            let merged = self.layout.merge(base);
            if self.from == MessageRole::User {
                merged.ml_auto()
            } else {
                merged
            }
        };
        let inner_layout = if self.from == MessageRole::User {
            LayoutRefinement::default().min_w_0()
        } else {
            LayoutRefinement::default().w_full().min_w_0()
        };

        let inner = ui::v_stack(move |_cx| children)
            .layout(inner_layout)
            .gap(gap)
            .into_element(cx);

        let row = ui::h_row(move |_cx| vec![inner])
            .layout(layout)
            .gap(Space::N0)
            .justify(justify)
            .into_element(cx);

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

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let theme = Theme::global(&*cx.app).clone();

        let bubble_fg = if self.from == MessageRole::User {
            // Upstream (ai-elements) uses `text-foreground` for user bubbles.
            Some(
                theme
                    .color_by_key("fret.ai.message.user.fg")
                    .unwrap_or_else(|| theme.color_token("foreground")),
            )
        } else {
            None
        };
        let base_chrome = if self.from == MessageRole::User {
            let bg = theme
                .color_by_key("fret.ai.message.user.bg")
                .unwrap_or_else(|| theme.color_token("secondary"));
            ChromeRefinement::default()
                .bg(ColorRef::Color(bg))
                .px(Space::N4)
                .py(Space::N3)
                .rounded(Radius::Lg)
        } else {
            ChromeRefinement::default()
        };

        let chrome = base_chrome.merge(self.chrome);
        let base_layout = {
            let mut layout = LayoutRefinement::default()
                .min_w_0()
                .max_w_full()
                .overflow_hidden();
            // Assistant/system/tool messages should participate in the full-width flow so text
            // measurement receives stable width constraints (avoids 0-width wrap explosions).
            if self.from != MessageRole::User {
                layout = layout.w_full();
            }
            layout
        };
        let layout = base_layout.merge(self.layout);
        let children = self.children;

        let props = decl_style::container_props(&theme, chrome, layout);
        let content = cx.container(props, move |cx| {
            let stack = ui::v_stack(move |_cx| children)
                .layout(LayoutRefinement::default().min_w_0())
                .gap(Space::N2)
                .into_element(cx);

            match bubble_fg {
                Some(fg) => vec![stack.inherit_foreground(fg)],
                None => vec![stack],
            }
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

fret_ui_kit::ui_component_layout_only!(Message);
fret_ui_kit::ui_component_chrome_layout!(MessageContent);

#[cfg(test)]
mod tests {
    use super::*;

    use fret_app::App;
    use fret_core::{AppWindowId, Point, Px, Rect, Size};

    fn contains_foreground_scope(el: &AnyElement) -> bool {
        matches!(el.kind, fret_ui::element::ElementKind::ForegroundScope(_))
            || el.children.iter().any(contains_foreground_scope)
    }

    fn find_first_inherited_foreground_node(el: &AnyElement) -> Option<&AnyElement> {
        if el.inherited_foreground.is_some() {
            return Some(el);
        }
        el.children
            .iter()
            .find_map(find_first_inherited_foreground_node)
    }

    #[test]
    fn message_content_user_bubble_attaches_foreground_without_wrapper() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(320.0), Px(160.0)),
        );

        fret_ui::elements::with_element_cx(&mut app, window, bounds, "message-fg", |cx| {
            let theme = Theme::global(&*cx.app).clone();
            let expected_fg = theme
                .color_by_key("fret.ai.message.user.fg")
                .unwrap_or_else(|| theme.color_token("foreground"));

            let el = MessageContent::new(MessageRole::User, [cx.text("Hello")]).into_element(cx);
            let inherited = find_first_inherited_foreground_node(&el)
                .expect("expected message bubble subtree to carry inherited foreground");

            assert_eq!(inherited.inherited_foreground, Some(expected_fg));
            assert!(
                !contains_foreground_scope(&el),
                "expected message bubble content to attach inherited foreground without inserting a ForegroundScope"
            );
        });
    }

    #[test]
    fn message_content_assistant_defaults_to_fill_width_for_stable_wrap() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(320.0), Px(160.0)),
        );

        let el =
            fret_ui::elements::with_element_cx(&mut app, window, bounds, "message-width", |cx| {
                MessageContent::new(MessageRole::Assistant, [cx.text("Wrapped content")])
                    .into_element(cx)
            });

        let fret_ui::element::ElementKind::Container(props) = &el.kind else {
            panic!("expected MessageContent to render a container root");
        };

        assert_eq!(props.layout.size.width, fret_ui::element::Length::Fill);
        assert_eq!(
            props.layout.size.min_width,
            Some(fret_ui::element::Length::Px(Px(0.0)))
        );
    }
}

#[cfg(test)]
mod ui_builder_integration_tests {
    use super::*;

    #[test]
    fn message_components_opt_into_ui_builder_traits() {
        fn assert_layout_only<T: fret_ui_kit::UiPatchTarget + fret_ui_kit::UiSupportsLayout>() {}
        fn assert_chrome_layout<
            T: fret_ui_kit::UiPatchTarget
                + fret_ui_kit::UiSupportsChrome
                + fret_ui_kit::UiSupportsLayout,
        >() {
        }
        fn assert_into_element<T: fret_ui_kit::UiIntoElement>() {}

        assert_layout_only::<Message>();
        assert_into_element::<Message>();

        assert_chrome_layout::<MessageContent>();
        assert_into_element::<MessageContent>();
    }
}
