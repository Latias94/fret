//! AI Elements-aligned `Checkpoint` surfaces.

use std::sync::Arc;

use fret_core::{Px, SemanticsRole};
use fret_ui::action::OnActivate;
use fret_ui::element::{AnyElement, SemanticsDecoration};
use fret_ui::{ElementContext, Theme, UiHost};
use fret_ui_kit::declarative::icon as decl_icon;
use fret_ui_kit::declarative::stack;
use fret_ui_kit::declarative::style as decl_style;
use fret_ui_kit::{ChromeRefinement, ColorFallback, ColorRef, LayoutRefinement, Space};
use fret_ui_kit::{WidgetStateProperty, WidgetStates};
use fret_ui_shadcn::button::ButtonStyle;
use fret_ui_shadcn::{
    Button, ButtonSize, ButtonVariant, Separator, Tooltip, TooltipAlign, TooltipSide,
};

/// Checkpoint row aligned with AI Elements `Checkpoint`.
#[derive(Debug)]
pub struct Checkpoint {
    children: Vec<AnyElement>,
    test_id: Option<Arc<str>>,
    layout: LayoutRefinement,
    chrome: ChromeRefinement,
}

impl Checkpoint {
    pub fn new(children: impl IntoIterator<Item = AnyElement>) -> Self {
        Self {
            children: children.into_iter().collect(),
            test_id: None,
            layout: LayoutRefinement::default()
                .w_full()
                .min_w_0()
                .overflow_hidden(),
            chrome: ChromeRefinement::default(),
        }
    }

    pub fn test_id(mut self, test_id: impl Into<Arc<str>>) -> Self {
        self.test_id = Some(test_id.into());
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
        let separator = Separator::new()
            .refine_layout(LayoutRefinement::default().flex_1().min_w_0())
            .into_element(cx);

        let children = self.children;
        let row = stack::hstack(
            cx,
            stack::HStackProps::default()
                .layout(LayoutRefinement::default().w_full().min_w_0())
                .gap(Space::N0p5)
                .items_center(),
            move |_cx| {
                let mut out = children;
                out.push(separator);
                out
            },
        );
        let mut row = cx.container(
            decl_style::container_props(&theme, self.chrome, self.layout),
            move |_cx| vec![row],
        );

        let Some(test_id) = self.test_id else {
            return row;
        };
        row.attach_semantics(
            SemanticsDecoration::default()
                .role(SemanticsRole::Group)
                .test_id(test_id),
        )
    }
}

/// Default icon aligned with AI Elements `CheckpointIcon` (Bookmark).
#[derive(Debug, Clone)]
pub struct CheckpointIcon {
    icon: fret_icons::IconId,
    size: Px,
    color: Option<ColorRef>,
    layout: LayoutRefinement,
}

impl Default for CheckpointIcon {
    fn default() -> Self {
        Self {
            icon: fret_icons::IconId::new_static("lucide.bookmark"),
            size: Px(16.0),
            color: None,
            layout: LayoutRefinement::default().flex_shrink_0(),
        }
    }
}

impl CheckpointIcon {
    pub fn icon_id(mut self, icon: fret_icons::IconId) -> Self {
        self.icon = icon;
        self
    }

    pub fn size(mut self, size: Px) -> Self {
        self.size = size;
        self
    }

    pub fn color(mut self, color: ColorRef) -> Self {
        self.color = Some(color);
        self
    }

    pub fn refine_layout(mut self, layout: LayoutRefinement) -> Self {
        self.layout = self.layout.merge(layout);
        self
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let theme = Theme::global(&*cx.app).clone();
        let fg = theme.color_token("muted-foreground");
        let color = self.color.unwrap_or(ColorRef::Color(fg));
        let icon = decl_icon::icon_with(cx, self.icon, Some(self.size), Some(color));
        let layout = decl_style::layout_style(&theme, self.layout);
        cx.container(
            fret_ui::element::ContainerProps {
                layout,
                ..Default::default()
            },
            move |_cx| vec![icon],
        )
    }
}

/// Trigger button aligned with AI Elements `CheckpointTrigger`.
pub struct CheckpointTrigger {
    children: Vec<AnyElement>,
    a11y_label: Arc<str>,
    tooltip: Option<Arc<str>>,
    on_activate: Option<OnActivate>,
    variant: ButtonVariant,
    size: ButtonSize,
    muted_foreground: bool,
    test_id: Option<Arc<str>>,
    tooltip_panel_test_id: Option<Arc<str>>,
}

impl std::fmt::Debug for CheckpointTrigger {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CheckpointTrigger")
            .field("children_len", &self.children.len())
            .field("a11y_label", &self.a11y_label)
            .field("has_tooltip", &self.tooltip.is_some())
            .field("has_on_activate", &self.on_activate.is_some())
            .field("variant", &self.variant)
            .field("size", &self.size)
            .field("test_id", &self.test_id.as_deref())
            .field(
                "tooltip_panel_test_id",
                &self.tooltip_panel_test_id.as_deref(),
            )
            .finish()
    }
}

impl CheckpointTrigger {
    pub fn new(children: impl IntoIterator<Item = AnyElement>) -> Self {
        Self {
            children: children.into_iter().collect(),
            a11y_label: Arc::<str>::from("Restore checkpoint"),
            tooltip: None,
            on_activate: None,
            variant: ButtonVariant::Ghost,
            size: ButtonSize::Sm,
            muted_foreground: true,
            test_id: None,
            tooltip_panel_test_id: None,
        }
    }

    pub fn a11y_label(mut self, label: impl Into<Arc<str>>) -> Self {
        self.a11y_label = label.into();
        self
    }

    pub fn tooltip(mut self, tooltip: impl Into<Arc<str>>) -> Self {
        self.tooltip = Some(tooltip.into());
        self
    }

    pub fn on_activate(mut self, on_activate: OnActivate) -> Self {
        self.on_activate = Some(on_activate);
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

    /// When true (default), uses `text-muted-foreground` for the idle state while preserving
    /// shadcn's hover/active foreground overrides (aligns AI Elements `Checkpoint` root text color).
    pub fn muted_foreground(mut self, enabled: bool) -> Self {
        self.muted_foreground = enabled;
        self
    }

    pub fn test_id(mut self, test_id: impl Into<Arc<str>>) -> Self {
        self.test_id = Some(test_id.into());
        self
    }

    pub fn tooltip_panel_test_id(mut self, test_id: impl Into<Arc<str>>) -> Self {
        self.tooltip_panel_test_id = Some(test_id.into());
        self
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let button = {
            let a11y_label = self.a11y_label;
            let children = self.children;
            let variant = self.variant;
            let size = self.size;
            let muted_foreground = self.muted_foreground;
            let mut b = Button::new(a11y_label)
                .children(children)
                .variant(variant)
                .size(size);
            if muted_foreground && variant == ButtonVariant::Ghost {
                let muted = ColorRef::Token {
                    key: "muted-foreground",
                    fallback: ColorFallback::ThemeTextMuted,
                };
                b = b.style(
                    ButtonStyle::default().foreground(
                        WidgetStateProperty::new(Some(muted))
                            .when(WidgetStates::HOVERED, None)
                            .when(WidgetStates::ACTIVE, None),
                    ),
                );
            }
            if let Some(on_activate) = self.on_activate {
                b = b.on_activate(on_activate);
            }
            if let Some(test_id) = self.test_id {
                b = b.test_id(test_id);
            }
            b.into_element(cx)
        };

        let Some(tooltip) = self.tooltip else {
            return button;
        };

        let content = cx.text(tooltip);
        let mut tooltip = Tooltip::new(button, content)
            .align(TooltipAlign::Start)
            .side(TooltipSide::Bottom);
        if let Some(panel_test_id) = self.tooltip_panel_test_id {
            tooltip = tooltip.panel_test_id(panel_test_id);
        }
        tooltip.into_element(cx)
    }
}
