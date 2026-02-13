use std::sync::Arc;

use fret_ui::action::{ActionCx, UiActionHost};
use fret_ui::element::ScrollAxis;
use fret_ui::element::SemanticsDecoration;
use fret_ui::element::{
    AnyElement, LayoutStyle, Length, Overflow, ScrollIntrinsicMeasureMode, ScrollProps,
};
use fret_ui::scroll::ScrollHandle;
use fret_ui::{ElementContext, UiHost};
use fret_ui_kit::declarative::stack;
use fret_ui_kit::{ChromeRefinement, LayoutRefinement, Radius, Space};

use fret_ui_shadcn::{Button, ButtonSize, ButtonVariant};

pub type OnSuggestionClick = Arc<dyn Fn(&mut dyn UiActionHost, ActionCx, Arc<str>) + 'static>;

#[derive(Clone)]
/// Horizontally scrollable suggestions row aligned with AI Elements `suggestion.tsx`.
pub struct Suggestions {
    children: Vec<AnyElement>,
    layout: LayoutRefinement,
    viewport_test_id: Option<Arc<str>>,
    root_test_id: Option<Arc<str>>,
}

impl std::fmt::Debug for Suggestions {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Suggestions")
            .field("children_len", &self.children.len())
            .field("layout", &self.layout)
            .field("viewport_test_id", &self.viewport_test_id.as_deref())
            .field("root_test_id", &self.root_test_id.as_deref())
            .finish()
    }
}

impl Suggestions {
    pub fn new(children: impl IntoIterator<Item = AnyElement>) -> Self {
        Self {
            children: children.into_iter().collect(),
            layout: LayoutRefinement::default().w_full().min_w_0(),
            viewport_test_id: None,
            root_test_id: None,
        }
    }

    pub fn refine_layout(mut self, layout: LayoutRefinement) -> Self {
        self.layout = self.layout.merge(layout);
        self
    }

    pub fn viewport_test_id(mut self, id: impl Into<Arc<str>>) -> Self {
        self.viewport_test_id = Some(id.into());
        self
    }

    pub fn test_id_root(mut self, id: impl Into<Arc<str>>) -> Self {
        self.root_test_id = Some(id.into());
        self
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let row = stack::hstack(
            cx,
            stack::HStackProps::default()
                .layout(LayoutRefinement::default().flex_shrink_0())
                .gap(Space::N2)
                .items_center(),
            move |_cx| self.children,
        );

        // Prefer a plain scroll region here: shadcn/Radix `ScrollArea` roots often assume `h-full`
        // semantics, which can collapse to 0 height under auto-height stacks in layout engines
        // like Taffy. Suggestions should shrink-wrap to the pill height, matching upstream.
        let handle = cx.with_state(ScrollHandle::default, |h| h.clone());

        let mut scroll_layout = LayoutStyle::default();
        scroll_layout.size.width = Length::Fill;
        scroll_layout.size.height = Length::Auto;
        scroll_layout.size.min_width = Some(fret_core::Px(0.0));
        scroll_layout.size.min_height = Some(fret_core::Px(0.0));
        scroll_layout.overflow = Overflow::Clip;

        let mut scroll = cx.scroll(
            ScrollProps {
                layout: scroll_layout,
                axis: ScrollAxis::X,
                scroll_handle: Some(handle),
                windowed_paint: false,
                probe_unbounded: true,
                intrinsic_measure_mode: ScrollIntrinsicMeasureMode::Content,
            },
            move |_cx| vec![row],
        );

        if let Some(test_id) = self.viewport_test_id {
            scroll = scroll.attach_semantics(
                SemanticsDecoration::default()
                    .role(fret_core::SemanticsRole::Group)
                    .test_id(test_id),
            );
        }

        let el = scroll;
        el.attach_semantics(SemanticsDecoration {
            test_id: self.root_test_id,
            ..Default::default()
        })
    }
}

#[derive(Clone)]
/// A single suggestion pill aligned with AI Elements `Suggestion`.
pub struct Suggestion {
    suggestion: Arc<str>,
    label: Option<Arc<str>>,
    on_click: Option<OnSuggestionClick>,
    disabled: bool,
    test_id: Option<Arc<str>>,
    variant: ButtonVariant,
    size: ButtonSize,
    chrome: ChromeRefinement,
    layout: LayoutRefinement,
}

impl std::fmt::Debug for Suggestion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Suggestion")
            .field("suggestion", &self.suggestion.as_ref())
            .field("label", &self.label.as_deref())
            .field("has_on_click", &self.on_click.is_some())
            .field("disabled", &self.disabled)
            .field("test_id", &self.test_id.as_deref())
            .field("variant", &self.variant)
            .field("size", &self.size)
            .field("chrome", &self.chrome)
            .field("layout", &self.layout)
            .finish()
    }
}

impl Suggestion {
    pub fn new(suggestion: impl Into<Arc<str>>) -> Self {
        Self {
            suggestion: suggestion.into(),
            label: None,
            on_click: None,
            disabled: false,
            test_id: None,
            variant: ButtonVariant::Outline,
            size: ButtonSize::Sm,
            chrome: ChromeRefinement::default()
                .rounded(Radius::Full)
                .px(Space::N4),
            layout: LayoutRefinement::default(),
        }
    }

    pub fn label(mut self, label: impl Into<Arc<str>>) -> Self {
        self.label = Some(label.into());
        self
    }

    pub fn on_click(mut self, on_click: OnSuggestionClick) -> Self {
        self.on_click = Some(on_click);
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn test_id(mut self, id: impl Into<Arc<str>>) -> Self {
        self.test_id = Some(id.into());
        self
    }

    pub fn variant(mut self, variant: ButtonVariant) -> Self {
        self.variant = variant;
        self
    }

    pub fn size(mut self, size: ButtonSize) -> Self {
        self.size = size;
        self
    }

    pub fn refine_style(mut self, chrome: ChromeRefinement) -> Self {
        self.chrome = self.chrome.merge(chrome);
        self
    }

    pub fn refine_layout(mut self, layout: LayoutRefinement) -> Self {
        self.layout = self.layout.merge(layout);
        self
    }

    pub fn into_element<H: UiHost + 'static>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let label = self
            .label
            .clone()
            .unwrap_or_else(|| Arc::clone(&self.suggestion));
        let on_click = self.on_click;
        let suggestion = self.suggestion;

        let mut button = Button::new(label)
            .variant(self.variant)
            .size(self.size)
            .disabled(self.disabled)
            .refine_style(self.chrome)
            .refine_layout(self.layout);

        if let Some(test_id) = self.test_id {
            button = button.test_id(test_id);
        }

        if let Some(on_click) = on_click {
            button = button.on_activate(Arc::new(move |host, action_cx, _reason| {
                on_click(host, action_cx, Arc::clone(&suggestion));
                host.notify(action_cx);
                host.request_redraw(action_cx.window);
            }));
        }

        button.into_element(cx)
    }
}
