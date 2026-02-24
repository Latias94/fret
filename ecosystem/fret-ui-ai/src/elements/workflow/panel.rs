use std::sync::Arc;

use fret_core::SemanticsRole;
use fret_ui::element::{AnyElement, SemanticsProps};
use fret_ui::{ElementContext, Theme, UiHost};
use fret_ui_kit::declarative::style as decl_style;
use fret_ui_kit::{ChromeRefinement, ColorFallback, ColorRef, LayoutRefinement, Radius, Space};

#[derive(Debug, Clone)]
pub struct WorkflowPanelInner {
    layout: LayoutRefinement,
}

impl Default for WorkflowPanelInner {
    fn default() -> Self {
        Self {
            layout: LayoutRefinement::default().w_full().min_w_0(),
        }
    }
}

impl WorkflowPanelInner {
    pub fn refine_layout(mut self, layout: LayoutRefinement) -> Self {
        self.layout = self.layout.merge(layout);
        self
    }
}

fret_ui_kit::ui_component_layout_only_patch_only!(WorkflowPanelInner);

/// AI Elements-aligned workflow `Panel` chrome (UI-only).
///
/// Upstream reference: `repo-ref/ai-elements/packages/elements/src/panel.tsx`.
///
/// Notes:
/// - Upstream is `@xyflow/react`-backed; in Fret this is a styling/composition wrapper only.
/// - Apps own positioning and interaction policy.
pub struct WorkflowPanel {
    children: Vec<AnyElement>,
    test_id: Option<Arc<str>>,
    inner: WorkflowPanelInner,
    layout: LayoutRefinement,
    chrome: ChromeRefinement,
}

impl std::fmt::Debug for WorkflowPanel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WorkflowPanel")
            .field("children_len", &self.children.len())
            .field("test_id", &self.test_id.as_deref())
            .field("inner", &self.inner)
            .field("layout", &self.layout)
            .field("chrome", &self.chrome)
            .finish()
    }
}

impl WorkflowPanel {
    pub fn new(children: impl IntoIterator<Item = AnyElement>) -> Self {
        Self {
            children: children.into_iter().collect(),
            test_id: None,
            inner: WorkflowPanelInner::default(),
            layout: LayoutRefinement::default()
                .m(Space::N4)
                .w_full()
                .min_w_0()
                .overflow_hidden(),
            chrome: ChromeRefinement::default(),
        }
    }

    pub fn test_id(mut self, id: impl Into<Arc<str>>) -> Self {
        self.test_id = Some(id.into());
        self
    }

    pub fn inner(mut self, inner: WorkflowPanelInner) -> Self {
        self.inner = inner;
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
            .p(Space::N1);

        let outer_layout = self.layout;
        let inner_layout = self.inner.layout;
        let children = self.children;

        let inner_props =
            decl_style::container_props(&theme, ChromeRefinement::default(), inner_layout);
        let inner = cx.container(inner_props, move |_cx| children);

        let outer_props =
            decl_style::container_props(&theme, base_chrome.merge(self.chrome), outer_layout);
        let body = cx.container(outer_props, move |_cx| [inner]);

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

fret_ui_kit::ui_component_chrome_layout!(WorkflowPanel);

#[cfg(test)]
mod ui_builder_integration_tests {
    use super::*;
    use fret_ui_kit::UiExt as _;

    #[test]
    fn workflow_panel_opts_into_ui_builder_traits() {
        fn assert_chrome_layout<
            T: fret_ui_kit::UiPatchTarget
                + fret_ui_kit::UiSupportsChrome
                + fret_ui_kit::UiSupportsLayout
                + fret_ui_kit::UiIntoElement,
        >() {
        }
        assert_chrome_layout::<WorkflowPanel>();
    }

    #[test]
    fn workflow_panel_inner_is_patch_only_and_builds() {
        let inner = WorkflowPanelInner::default().ui().w_full().build();
        let _ = WorkflowPanel::new([]).inner(inner);
    }
}
