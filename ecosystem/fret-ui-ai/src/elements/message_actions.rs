use std::sync::Arc;

use fret_core::SemanticsRole;
use fret_ui::action::OnActivate;
use fret_ui::element::{AnyElement, SemanticsProps};
use fret_ui::{ElementContext, UiHost};
use fret_ui_kit::declarative::stack;
use fret_ui_kit::{Justify, LayoutRefinement, Space};
use fret_ui_shadcn::{
    Button, ButtonSize, ButtonVariant, Tooltip, TooltipContent, TooltipProvider, TooltipTrigger,
};

#[derive(Clone)]
/// A row container for per-message action buttons (AI Elements `MessageActions`-style).
pub struct MessageActions {
    children: Vec<AnyElement>,
    justify: Justify,
    test_id: Option<Arc<str>>,
    layout: LayoutRefinement,
}

impl std::fmt::Debug for MessageActions {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MessageActions")
            .field("children_len", &self.children.len())
            .field("justify", &self.justify)
            .field("test_id", &self.test_id.as_deref())
            .field("layout", &self.layout)
            .finish()
    }
}

impl MessageActions {
    pub fn new(children: impl IntoIterator<Item = AnyElement>) -> Self {
        Self {
            children: children.into_iter().collect(),
            justify: Justify::Start,
            test_id: None,
            layout: LayoutRefinement::default(),
        }
    }

    pub fn justify(mut self, justify: Justify) -> Self {
        self.justify = justify;
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

    pub fn into_element<H: UiHost + 'static>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let layout = self.layout.merge(LayoutRefinement::default().w_full());
        let justify = self.justify;
        let children = self.children;

        let row = stack::hstack(
            cx,
            stack::HStackProps::default()
                .layout(layout)
                .gap(Space::N1)
                .justify(justify)
                .items_center(),
            |_cx| children,
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
/// A single action button with an optional tooltip (AI Elements `MessageAction`-style).
pub struct MessageAction {
    tooltip: Option<Arc<str>>,
    tooltip_panel_test_id: Option<Arc<str>>,
    label: Arc<str>,
    children: Vec<AnyElement>,
    on_activate: Option<OnActivate>,
    disabled: bool,
    test_id: Option<Arc<str>>,
    variant: ButtonVariant,
    size: ButtonSize,
}

impl std::fmt::Debug for MessageAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MessageAction")
            .field("tooltip", &self.tooltip.as_deref())
            .field(
                "tooltip_panel_test_id",
                &self.tooltip_panel_test_id.as_deref(),
            )
            .field("label", &self.label.as_ref())
            .field("children_len", &self.children.len())
            .field("has_on_activate", &self.on_activate.is_some())
            .field("disabled", &self.disabled)
            .field("test_id", &self.test_id.as_deref())
            .field("variant", &self.variant)
            .field("size", &self.size)
            .finish()
    }
}

impl MessageAction {
    pub fn new(label: impl Into<Arc<str>>) -> Self {
        Self {
            tooltip: None,
            tooltip_panel_test_id: None,
            label: label.into(),
            children: Vec::new(),
            on_activate: None,
            disabled: false,
            test_id: None,
            variant: ButtonVariant::Ghost,
            size: ButtonSize::IconSm,
        }
    }

    pub fn tooltip(mut self, tooltip: impl Into<Arc<str>>) -> Self {
        self.tooltip = Some(tooltip.into());
        self
    }

    /// Optional diagnostic-only selector for the tooltip placed panel bounds.
    ///
    /// Default: derived from `test_id` as `${test_id}-tooltip-panel`.
    pub fn tooltip_panel_test_id(mut self, id: impl Into<Arc<str>>) -> Self {
        self.tooltip_panel_test_id = Some(id.into());
        self
    }

    pub fn children(mut self, children: impl IntoIterator<Item = AnyElement>) -> Self {
        self.children = children.into_iter().collect();
        self
    }

    pub fn on_activate(mut self, on_activate: OnActivate) -> Self {
        self.on_activate = Some(on_activate);
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

    pub fn into_element<H: UiHost + 'static>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let tooltip_text = self.tooltip.clone();

        let mut btn = Button::new(self.label.clone())
            .variant(self.variant)
            .size(self.size)
            .disabled(self.disabled)
            .children(self.children);
        if let Some(on_activate) = self.on_activate {
            btn = btn.on_activate(on_activate);
        }
        if let Some(test_id) = self.test_id.clone() {
            btn = btn.test_id(test_id);
        }
        let btn = btn.into_element(cx);

        let Some(tooltip_text) = tooltip_text else {
            return btn;
        };

        let panel_test_id = self.tooltip_panel_test_id.or_else(|| {
            self.test_id
                .as_ref()
                .map(|id| Arc::<str>::from(format!("{id}-tooltip-panel")))
        });

        let provider = TooltipProvider::new();
        let mut out = provider.with(cx, move |cx| {
            let trigger = TooltipTrigger::new(btn).into_element(cx);
            let content = TooltipContent::new(vec![TooltipContent::text(cx, tooltip_text.clone())])
                .into_element(cx);

            let mut tip = Tooltip::new(trigger, content);
            if let Some(panel_test_id) = panel_test_id.clone() {
                tip = tip.panel_test_id(panel_test_id);
            }

            vec![tip.into_element(cx)]
        });

        out.pop().unwrap_or_else(|| cx.text(""))
    }
}
