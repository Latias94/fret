use std::sync::Arc;

use fret_core::{Color, Edges, FontId, FontWeight, Px, SemanticsRole, TextStyle};
use fret_icons::{IconId, ids};
use fret_ui::action::OnActivate;
use fret_ui::element::{AnyElement, LayoutStyle, SemanticsProps, TextProps};
use fret_ui::{ElementContext, Theme, UiHost};
use fret_ui_kit::declarative::icon as decl_icon;
use fret_ui_kit::declarative::stack;
use fret_ui_kit::declarative::style as decl_style;
use fret_ui_kit::{
    ChromeRefinement, ColorFallback, ColorRef, Items, Justify, LayoutRefinement, Space,
    WidgetStateProperty, WidgetStates,
};
use fret_ui_shadcn::button::ButtonStyle;
use fret_ui_shadcn::{
    Button, ButtonSize, ButtonVariant, ScrollArea, Tooltip, TooltipContent, TooltipProvider,
    TooltipTrigger,
};

fn alpha_mul(color: Color, mul: f32) -> Color {
    let mul = mul.clamp(0.0, 1.0);
    Color {
        a: (color.a * mul).clamp(0.0, 1.0),
        ..color
    }
}

fn token_color_with_alpha(
    theme: &Theme,
    key: &'static str,
    fallback_key: &'static str,
    alpha: f32,
) -> Color {
    let base = theme
        .color_by_key(key)
        .or_else(|| theme.color_by_key(fallback_key))
        .unwrap_or_else(|| theme.color_required("foreground"));
    alpha_mul(base, alpha)
}

fn resolve_muted_fg(theme: &Theme) -> Color {
    theme
        .color_by_key("muted-foreground")
        .or_else(|| theme.color_by_key("muted_foreground"))
        .unwrap_or_else(|| theme.color_required("foreground"))
}

fn action_button_style() -> ButtonStyle {
    let muted = Some(ColorRef::Token {
        key: "muted-foreground",
        fallback: ColorFallback::ThemeTextMuted,
    });
    let fg = Some(ColorRef::Token {
        key: "foreground",
        fallback: ColorFallback::ThemeTextPrimary,
    });

    ButtonStyle::default().foreground(
        WidgetStateProperty::new(muted)
            .when(WidgetStates::HOVERED, fg.clone())
            .when(WidgetStates::ACTIVE, fg),
    )
}

#[derive(Clone)]
/// Structured container for displaying generated content with header actions (AI Elements `artifact.tsx`).
pub struct Artifact {
    children: Vec<AnyElement>,
    test_id_root: Option<Arc<str>>,
    layout: LayoutRefinement,
    chrome: ChromeRefinement,
}

impl std::fmt::Debug for Artifact {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Artifact")
            .field("children_len", &self.children.len())
            .field("test_id_root", &self.test_id_root.as_deref())
            .field("layout", &self.layout)
            .field("chrome", &self.chrome)
            .finish()
    }
}

impl Artifact {
    pub fn new(children: impl IntoIterator<Item = AnyElement>) -> Self {
        Self {
            children: children.into_iter().collect(),
            test_id_root: None,
            layout: LayoutRefinement::default().min_w_0().overflow_hidden(),
            chrome: ChromeRefinement::default(),
        }
    }

    pub fn test_id_root(mut self, id: impl Into<Arc<str>>) -> Self {
        self.test_id_root = Some(id.into());
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

        let base_chrome = ChromeRefinement::default()
            .rounded(fret_ui_kit::Radius::Lg)
            .border_1()
            .shadow_sm()
            .bg(ColorRef::Token {
                key: "background",
                fallback: ColorFallback::ThemeSurfaceBackground,
            })
            .border_color(ColorRef::Token {
                key: "border",
                fallback: ColorFallback::ThemePanelBorder,
            });

        let children = self.children;
        let body = stack::vstack(
            cx,
            stack::VStackProps::default()
                .layout(LayoutRefinement::default().w_full().min_w_0())
                .gap(Space::N0)
                .items(Items::Stretch),
            move |_cx| children,
        );

        let root = cx.container(
            decl_style::container_props(&theme, base_chrome.merge(self.chrome), self.layout),
            move |_cx| vec![body],
        );

        let Some(test_id) = self.test_id_root else {
            return root;
        };
        root.attach_semantics(
            fret_ui::element::SemanticsDecoration::default()
                .role(SemanticsRole::Group)
                .test_id(test_id),
        )
    }
}

#[derive(Clone)]
/// Header row (title/description + action group) aligned with AI Elements `ArtifactHeader`.
pub struct ArtifactHeader {
    children: Vec<AnyElement>,
    test_id: Option<Arc<str>>,
    layout: LayoutRefinement,
    chrome: ChromeRefinement,
}

impl std::fmt::Debug for ArtifactHeader {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ArtifactHeader")
            .field("children_len", &self.children.len())
            .field("test_id", &self.test_id.as_deref())
            .field("layout", &self.layout)
            .field("chrome", &self.chrome)
            .finish()
    }
}

impl ArtifactHeader {
    pub fn new(children: impl IntoIterator<Item = AnyElement>) -> Self {
        Self {
            children: children.into_iter().collect(),
            test_id: None,
            layout: LayoutRefinement::default().w_full().min_w_0(),
            chrome: ChromeRefinement::default().px(Space::N4).py(Space::N3),
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
        let border = theme.color_required("border");

        let bg = token_color_with_alpha(&theme, "muted", "muted.background", 0.5);

        let row = stack::hstack(
            cx,
            stack::HStackProps::default()
                .layout(self.layout)
                .gap(Space::N2)
                .justify(Justify::Between)
                .items_center(),
            move |_cx| self.children,
        );

        let mut props =
            decl_style::container_props(&theme, self.chrome, LayoutRefinement::default());
        props.background = Some(bg);
        props.border = Edges {
            top: Px(0.0),
            right: Px(0.0),
            bottom: Px(1.0),
            left: Px(0.0),
        };
        props.border_color = Some(border);

        let header = cx.container(props, move |_cx| vec![row]);

        let Some(test_id) = self.test_id else {
            return header;
        };
        cx.semantics(
            SemanticsProps {
                role: SemanticsRole::Group,
                test_id: Some(test_id),
                ..Default::default()
            },
            move |_cx| vec![header],
        )
    }
}

#[derive(Clone)]
/// Title text aligned with AI Elements `ArtifactTitle`.
pub struct ArtifactTitle {
    text: Arc<str>,
    test_id: Option<Arc<str>>,
}

impl std::fmt::Debug for ArtifactTitle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ArtifactTitle")
            .field("text", &self.text.as_ref())
            .field("test_id", &self.test_id.as_deref())
            .finish()
    }
}

impl ArtifactTitle {
    pub fn new(text: impl Into<Arc<str>>) -> Self {
        Self {
            text: text.into(),
            test_id: None,
        }
    }

    pub fn test_id(mut self, id: impl Into<Arc<str>>) -> Self {
        self.test_id = Some(id.into());
        self
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let theme = Theme::global(&*cx.app).clone();

        let mut text = cx.text_props(TextProps {
            layout: LayoutStyle::default(),
            text: self.text,
            style: Some(TextStyle {
                font: FontId::default(),
                size: theme
                    .metric_by_key("component.artifact.title_text_px")
                    .unwrap_or_else(|| theme.metric_required("font.size")),
                weight: FontWeight::MEDIUM,
                slant: Default::default(),
                line_height: Some(theme.metric_required("font.line_height")),
                letter_spacing_em: None,
            }),
            color: Some(theme.color_required("foreground")),
            wrap: fret_core::TextWrap::None,
            overflow: fret_core::TextOverflow::Clip,
        });

        if let Some(test_id) = self.test_id {
            text = text.attach_semantics(
                fret_ui::element::SemanticsDecoration::default().test_id(test_id),
            );
        }

        text
    }
}

#[derive(Clone)]
/// Description text aligned with AI Elements `ArtifactDescription`.
pub struct ArtifactDescription {
    text: Arc<str>,
    test_id: Option<Arc<str>>,
}

impl std::fmt::Debug for ArtifactDescription {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ArtifactDescription")
            .field("text", &self.text.as_ref())
            .field("test_id", &self.test_id.as_deref())
            .finish()
    }
}

impl ArtifactDescription {
    pub fn new(text: impl Into<Arc<str>>) -> Self {
        Self {
            text: text.into(),
            test_id: None,
        }
    }

    pub fn test_id(mut self, id: impl Into<Arc<str>>) -> Self {
        self.test_id = Some(id.into());
        self
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let theme = Theme::global(&*cx.app).clone();
        let muted_fg = resolve_muted_fg(&theme);

        let mut text = cx.text_props(TextProps {
            layout: LayoutStyle::default(),
            text: self.text,
            style: Some(TextStyle {
                font: FontId::default(),
                size: theme
                    .metric_by_key("component.artifact.description_text_px")
                    .unwrap_or_else(|| theme.metric_required("font.size")),
                weight: FontWeight::NORMAL,
                slant: Default::default(),
                line_height: Some(theme.metric_required("font.line_height")),
                letter_spacing_em: None,
            }),
            color: Some(muted_fg),
            wrap: fret_core::TextWrap::None,
            overflow: fret_core::TextOverflow::Clip,
        });

        if let Some(test_id) = self.test_id {
            text = text.attach_semantics(
                fret_ui::element::SemanticsDecoration::default().test_id(test_id),
            );
        }

        text
    }
}

#[derive(Clone)]
/// Action group row aligned with AI Elements `ArtifactActions`.
pub struct ArtifactActions {
    children: Vec<AnyElement>,
    test_id: Option<Arc<str>>,
    layout: LayoutRefinement,
}

impl std::fmt::Debug for ArtifactActions {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ArtifactActions")
            .field("children_len", &self.children.len())
            .field("test_id", &self.test_id.as_deref())
            .field("layout", &self.layout)
            .finish()
    }
}

impl ArtifactActions {
    pub fn new(children: impl IntoIterator<Item = AnyElement>) -> Self {
        Self {
            children: children.into_iter().collect(),
            test_id: None,
            layout: LayoutRefinement::default(),
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

    pub fn into_element<H: UiHost + 'static>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let children = self.children;

        let row = stack::hstack(
            cx,
            stack::HStackProps::default()
                .layout(self.layout)
                .gap(Space::N1)
                .items(Items::Center),
            move |_cx| children,
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
/// A single action button with an optional tooltip (AI Elements `ArtifactAction`).
pub struct ArtifactAction {
    tooltip: Option<Arc<str>>,
    label: Option<Arc<str>>,
    icon: Option<IconId>,
    children: Vec<AnyElement>,
    on_activate: Option<OnActivate>,
    disabled: bool,
    test_id: Option<Arc<str>>,
    variant: ButtonVariant,
    size: ButtonSize,
}

impl std::fmt::Debug for ArtifactAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ArtifactAction")
            .field("tooltip", &self.tooltip.as_deref())
            .field("label", &self.label.as_deref())
            .field("icon", &self.icon)
            .field("children_len", &self.children.len())
            .field("has_on_activate", &self.on_activate.is_some())
            .field("disabled", &self.disabled)
            .field("test_id", &self.test_id.as_deref())
            .field("variant", &self.variant)
            .field("size", &self.size)
            .finish()
    }
}

impl ArtifactAction {
    pub fn new() -> Self {
        Self {
            tooltip: None,
            label: None,
            icon: None,
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

    pub fn label(mut self, label: impl Into<Arc<str>>) -> Self {
        self.label = Some(label.into());
        self
    }

    pub fn icon(mut self, icon: IconId) -> Self {
        self.icon = Some(icon);
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
        let label = self
            .label
            .clone()
            .or_else(|| tooltip_text.clone())
            .unwrap_or_else(|| Arc::<str>::from("Action"));

        let children = if let Some(icon) = self.icon {
            vec![decl_icon::icon(cx, icon)]
        } else {
            self.children
        };

        let mut btn = Button::new(label)
            .variant(self.variant)
            .size(self.size)
            .style(action_button_style())
            .disabled(self.disabled)
            .children(children);
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

        let provider = TooltipProvider::new();
        let mut out = provider.with(cx, move |cx| {
            let trigger = TooltipTrigger::new(btn).into_element(cx);
            let content = TooltipContent::new(vec![TooltipContent::text(cx, tooltip_text.clone())])
                .into_element(cx);
            vec![Tooltip::new(trigger, content).into_element(cx)]
        });

        out.pop().unwrap_or_else(|| cx.text(""))
    }
}

#[derive(Clone)]
/// Close button aligned with AI Elements `ArtifactClose`.
pub struct ArtifactClose {
    on_activate: Option<OnActivate>,
    disabled: bool,
    test_id: Option<Arc<str>>,
    variant: ButtonVariant,
    size: ButtonSize,
}

impl std::fmt::Debug for ArtifactClose {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ArtifactClose")
            .field("has_on_activate", &self.on_activate.is_some())
            .field("disabled", &self.disabled)
            .field("test_id", &self.test_id.as_deref())
            .field("variant", &self.variant)
            .field("size", &self.size)
            .finish()
    }
}

impl ArtifactClose {
    pub fn new() -> Self {
        Self {
            on_activate: None,
            disabled: false,
            test_id: None,
            variant: ButtonVariant::Ghost,
            size: ButtonSize::IconSm,
        }
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
        let mut btn = Button::new("Close")
            .variant(self.variant)
            .size(self.size)
            .style(action_button_style())
            .disabled(self.disabled)
            .children([decl_icon::icon(cx, ids::ui::CLOSE)]);
        if let Some(on_activate) = self.on_activate {
            btn = btn.on_activate(on_activate);
        }
        if let Some(test_id) = self.test_id {
            btn = btn.test_id(test_id);
        }
        btn.into_element(cx)
    }
}

#[derive(Clone)]
/// Scrollable content region aligned with AI Elements `ArtifactContent`.
pub struct ArtifactContent {
    children: Vec<AnyElement>,
    axis: fret_ui::element::ScrollAxis,
    viewport_test_id: Option<Arc<str>>,
    layout: LayoutRefinement,
    chrome: ChromeRefinement,
}

impl std::fmt::Debug for ArtifactContent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ArtifactContent")
            .field("children_len", &self.children.len())
            .field("axis", &self.axis)
            .field("viewport_test_id", &self.viewport_test_id.as_deref())
            .field("layout", &self.layout)
            .field("chrome", &self.chrome)
            .finish()
    }
}

impl ArtifactContent {
    pub fn new(children: impl IntoIterator<Item = AnyElement>) -> Self {
        Self {
            children: children.into_iter().collect(),
            axis: fret_ui::element::ScrollAxis::Both,
            viewport_test_id: None,
            layout: LayoutRefinement::default().min_w_0().min_h_0().flex_1(),
            chrome: ChromeRefinement::default().p(Space::N4),
        }
    }

    pub fn axis(mut self, axis: fret_ui::element::ScrollAxis) -> Self {
        self.axis = axis;
        self
    }

    pub fn viewport_test_id(mut self, id: impl Into<Arc<str>>) -> Self {
        self.viewport_test_id = Some(id.into());
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

        let content = cx.container(
            decl_style::container_props(&theme, self.chrome, LayoutRefinement::default()),
            move |_cx| self.children,
        );

        let mut scroll = ScrollArea::new([content])
            .axis(self.axis)
            .refine_layout(self.layout);
        if let Some(test_id) = self.viewport_test_id {
            scroll = scroll.viewport_test_id(test_id);
        }
        scroll.into_element(cx)
    }
}
