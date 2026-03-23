use std::any::Any;
use std::sync::Arc;

use fret_core::{Color, Edges, FontId, FontWeight, Px, SemanticsRole, TextStyle};
use fret_icons::{IconId, ids};
use fret_runtime::ActionId;
use fret_ui::action::OnActivate;
use fret_ui::element::{AnyElement, LayoutStyle, SemanticsProps, TextProps};
use fret_ui::{ElementContext, Theme, UiHost};
use fret_ui_kit::declarative::icon as decl_icon;
use fret_ui_kit::declarative::style as decl_style;
use fret_ui_kit::typography;
use fret_ui_kit::ui;
use fret_ui_kit::{
    ChromeRefinement, ColorFallback, ColorRef, Items, Justify, LayoutRefinement, Space,
    WidgetStateProperty, WidgetStates,
};
use fret_ui_shadcn::facade::{
    Button, ButtonSize, ButtonVariant, ScrollArea, Tooltip, TooltipContent, TooltipProvider,
    TooltipTrigger,
};
use fret_ui_shadcn::raw::button::ButtonStyle;

type ActionPayloadFactory = Arc<dyn Fn() -> Box<dyn Any + Send + Sync> + 'static>;

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
        .unwrap_or_else(|| theme.color_token("foreground"));
    alpha_mul(base, alpha)
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

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
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
        let body = ui::v_stack(move |_cx| children)
            .layout(LayoutRefinement::default().w_full().min_w_0())
            .gap(Space::N0)
            .items(Items::Stretch)
            .into_element(cx);

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
        let border = theme.color_token("border");

        let bg = token_color_with_alpha(&theme, "muted", "muted.background", 0.5);

        let row = ui::h_row(move |_cx| self.children)
            .layout(self.layout)
            .gap(Space::N2)
            .justify(Justify::Between)
            .items(Items::Center)
            .into_element(cx);

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

/// Title text aligned with AI Elements `ArtifactTitle`.
pub struct ArtifactTitle {
    content: ArtifactTitleContent,
    test_id: Option<Arc<str>>,
}

enum ArtifactTitleContent {
    Text(Arc<str>),
    Children(Vec<AnyElement>),
}

impl std::fmt::Debug for ArtifactTitle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut debug = f.debug_struct("ArtifactTitle");
        match &self.content {
            ArtifactTitleContent::Text(text) => {
                debug.field("text", &text.as_ref());
            }
            ArtifactTitleContent::Children(children) => {
                debug.field("children_len", &children.len());
            }
        }
        debug.field("test_id", &self.test_id.as_deref()).finish()
    }
}

impl ArtifactTitle {
    pub fn new(text: impl Into<Arc<str>>) -> Self {
        Self {
            content: ArtifactTitleContent::Text(text.into()),
            test_id: None,
        }
    }

    pub fn new_children(children: impl IntoIterator<Item = AnyElement>) -> Self {
        Self {
            content: ArtifactTitleContent::Children(children.into_iter().collect()),
            test_id: None,
        }
    }

    pub fn test_id(mut self, id: impl Into<Arc<str>>) -> Self {
        self.test_id = Some(id.into());
        self
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let theme = Theme::global(&*cx.app).clone();
        let style = typography::as_control_text(TextStyle {
            font: FontId::default(),
            size: theme
                .metric_by_key("component.artifact.title_text_px")
                .unwrap_or_else(|| theme.metric_token("font.size")),
            weight: FontWeight::MEDIUM,
            slant: Default::default(),
            line_height: Some(theme.metric_token("font.line_height")),
            letter_spacing_em: None,
            ..Default::default()
        });
        let refinement = typography::composable_refinement_from_style(&style);
        let foreground = theme.color_token("foreground");

        let mut element = match self.content {
            ArtifactTitleContent::Text(text) => cx
                .foreground_scope(foreground, move |cx| {
                    vec![
                        fret_ui_kit::ui::raw_text(text)
                            .wrap(fret_core::TextWrap::None)
                            .overflow(fret_core::TextOverflow::Clip)
                            .into_element(cx),
                    ]
                })
                .inherit_foreground(foreground)
                .inherit_text_style(refinement),
            ArtifactTitleContent::Children(children) => cx
                .foreground_scope(foreground, move |_cx| children)
                .inherit_foreground(foreground)
                .inherit_text_style(refinement),
        };

        if let Some(test_id) = self.test_id {
            element = element.attach_semantics(
                fret_ui::element::SemanticsDecoration::default().test_id(test_id),
            );
        }

        element
    }
}

/// Description text aligned with AI Elements `ArtifactDescription`.
pub struct ArtifactDescription {
    content: ArtifactDescriptionContent,
    test_id: Option<Arc<str>>,
}

enum ArtifactDescriptionContent {
    Text(Arc<str>),
    Children(Vec<AnyElement>),
}

impl std::fmt::Debug for ArtifactDescription {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut debug = f.debug_struct("ArtifactDescription");
        match &self.content {
            ArtifactDescriptionContent::Text(text) => {
                debug.field("text", &text.as_ref());
            }
            ArtifactDescriptionContent::Children(children) => {
                debug.field("children_len", &children.len());
            }
        }
        debug.field("test_id", &self.test_id.as_deref()).finish()
    }
}

impl ArtifactDescription {
    pub fn new(text: impl Into<Arc<str>>) -> Self {
        Self {
            content: ArtifactDescriptionContent::Text(text.into()),
            test_id: None,
        }
    }

    pub fn new_children(children: impl IntoIterator<Item = AnyElement>) -> Self {
        Self {
            content: ArtifactDescriptionContent::Children(children.into_iter().collect()),
            test_id: None,
        }
    }

    pub fn test_id(mut self, id: impl Into<Arc<str>>) -> Self {
        self.test_id = Some(id.into());
        self
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let theme = Theme::global(&*cx.app).clone();
        let refinement = typography::description_text_refinement_with_fallbacks(
            &theme,
            "component.artifact.description_text",
            Some("font.size"),
            Some("font.line_height"),
        );
        let foreground = typography::muted_foreground_color(&theme);

        let mut text = match self.content {
            ArtifactDescriptionContent::Text(text) => {
                typography::scope_description_text_with_fallbacks(
                    cx.text_props(TextProps {
                        layout: LayoutStyle::default(),
                        text,
                        style: None,
                        color: None,
                        wrap: fret_core::TextWrap::None,
                        overflow: fret_core::TextOverflow::Clip,
                        align: fret_core::TextAlign::Start,
                        ink_overflow: Default::default(),
                    }),
                    &theme,
                    "component.artifact.description_text",
                    Some("font.size"),
                    Some("font.line_height"),
                )
            }
            ArtifactDescriptionContent::Children(children) => cx
                .foreground_scope(foreground, move |_cx| children)
                .inherit_foreground(foreground)
                .inherit_text_style(refinement),
        };

        if let Some(test_id) = self.test_id {
            text = text.attach_semantics(
                fret_ui::element::SemanticsDecoration::default().test_id(test_id),
            );
        }

        text
    }
}

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

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let children = self.children;

        let row = ui::h_row(move |_cx| children)
            .layout(self.layout)
            .gap(Space::N1)
            .items(Items::Center)
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

/// A single action button with an optional tooltip (AI Elements `ArtifactAction`).
pub struct ArtifactAction {
    tooltip: Option<Arc<str>>,
    label: Option<Arc<str>>,
    icon: Option<IconId>,
    children: Vec<AnyElement>,
    action: Option<ActionId>,
    action_payload: Option<ActionPayloadFactory>,
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
            .field("action", &self.action)
            .field("action_payload", &self.action_payload.is_some())
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
            action: None,
            action_payload: None,
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

    /// Bind a stable action ID to this artifact action (action-first authoring).
    pub fn action(mut self, action: impl Into<fret_runtime::ActionId>) -> Self {
        self.action = Some(action.into());
        self
    }

    /// Attach a payload for parameterized artifact actions (ADR 0312).
    pub fn action_payload<T>(mut self, payload: T) -> Self
    where
        T: Any + Send + Sync + Clone + 'static,
    {
        let payload = Arc::new(payload);
        self.action_payload = Some(Arc::new(move || Box::new(payload.as_ref().clone())));
        self
    }

    /// Like [`ArtifactAction::action_payload`], but computes the payload lazily.
    pub fn action_payload_factory(mut self, payload: ActionPayloadFactory) -> Self {
        self.action_payload = Some(payload);
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

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
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
        if let Some(action) = self.action {
            btn = btn.action(action);
        }
        if let Some(payload) = self.action_payload {
            btn = btn.action_payload_factory(payload);
        }
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
            let content =
                TooltipContent::build(cx, |_cx| [TooltipContent::text(tooltip_text.clone())])
                    .into_element(cx);
            vec![Tooltip::new(cx, trigger, content).into_element(cx)]
        });

        out.pop().unwrap_or_else(|| cx.text(""))
    }
}

/// Close button aligned with AI Elements `ArtifactClose`.
pub struct ArtifactClose {
    children: Vec<AnyElement>,
    action: Option<ActionId>,
    action_payload: Option<ActionPayloadFactory>,
    on_activate: Option<OnActivate>,
    disabled: bool,
    test_id: Option<Arc<str>>,
    variant: ButtonVariant,
    size: ButtonSize,
}

impl std::fmt::Debug for ArtifactClose {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ArtifactClose")
            .field("children_len", &self.children.len())
            .field("action", &self.action)
            .field("action_payload", &self.action_payload.is_some())
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
            children: Vec::new(),
            action: None,
            action_payload: None,
            on_activate: None,
            disabled: false,
            test_id: None,
            variant: ButtonVariant::Ghost,
            size: ButtonSize::IconSm,
        }
    }

    pub fn children(mut self, children: impl IntoIterator<Item = AnyElement>) -> Self {
        self.children = children.into_iter().collect();
        self
    }

    /// Bind a stable action ID to this artifact close affordance (action-first authoring).
    pub fn action(mut self, action: impl Into<fret_runtime::ActionId>) -> Self {
        self.action = Some(action.into());
        self
    }

    /// Attach a payload for parameterized artifact-close actions (ADR 0312).
    pub fn action_payload<T>(mut self, payload: T) -> Self
    where
        T: Any + Send + Sync + Clone + 'static,
    {
        let payload = Arc::new(payload);
        self.action_payload = Some(Arc::new(move || Box::new(payload.as_ref().clone())));
        self
    }

    /// Like [`ArtifactClose::action_payload`], but computes the payload lazily.
    pub fn action_payload_factory(mut self, payload: ActionPayloadFactory) -> Self {
        self.action_payload = Some(payload);
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

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let mut btn = Button::new("")
            .a11y_label("Close")
            .variant(self.variant)
            .size(self.size)
            .style(action_button_style())
            .disabled(self.disabled);
        if self.children.is_empty() {
            btn = btn.icon(ids::ui::CLOSE);
        } else {
            btn = btn.children(self.children);
        }
        if let Some(action) = self.action {
            btn = btn.action(action);
        }
        if let Some(payload) = self.action_payload {
            btn = btn.action_payload_factory(payload);
        }
        if let Some(on_activate) = self.on_activate {
            btn = btn.on_activate(on_activate);
        }
        if let Some(test_id) = self.test_id {
            btn = btn.test_id(test_id);
        }
        btn.into_element(cx)
    }
}

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

#[cfg(test)]
mod tests {
    use super::*;

    use fret_app::App;
    use fret_core::{AppWindowId, Point, Rect, Size};
    use fret_ui::element::ElementKind;
    use fret_ui::{Theme, ThemeConfig};

    fn bounds() -> Rect {
        Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(720.0), Px(480.0)),
        )
    }

    fn has_test_id(element: &AnyElement, test_id: &str) -> bool {
        if element
            .semantics_decoration
            .as_ref()
            .and_then(|d| d.test_id.as_deref())
            == Some(test_id)
        {
            return true;
        }

        element
            .children
            .iter()
            .any(|child| has_test_id(child, test_id))
    }

    fn has_scoped_text_style(
        element: &AnyElement,
        refinement: &fret_core::TextStyleRefinement,
        foreground: fret_core::Color,
    ) -> bool {
        if element.inherited_text_style.as_ref() == Some(refinement)
            && element.inherited_foreground == Some(foreground)
        {
            return true;
        }

        element
            .children
            .iter()
            .any(|child| has_scoped_text_style(child, refinement, foreground))
    }

    #[test]
    fn artifact_keeps_group_role_when_stamping_test_id() {
        let window = AppWindowId::default();
        let mut app = App::new();

        let element =
            fret_ui::elements::with_element_cx(&mut app, window, bounds(), "test", |cx| {
                Artifact::new([cx.text("content")])
                    .test_id_root("ui-ai-artifact-root")
                    .into_element(cx)
            });

        assert_eq!(
            element.semantics_decoration.as_ref().and_then(|d| d.role),
            Some(SemanticsRole::Group)
        );
        assert_eq!(
            element
                .semantics_decoration
                .as_ref()
                .and_then(|d| d.test_id.as_deref()),
            Some("ui-ai-artifact-root")
        );
    }

    #[test]
    fn artifact_close_renders_custom_children_when_provided() {
        let window = AppWindowId::default();
        let mut app = App::new();

        let element =
            fret_ui::elements::with_element_cx(&mut app, window, bounds(), "test", |cx| {
                ArtifactClose::new()
                    .children([cx.text("Dismiss").test_id("custom-close-label")])
                    .into_element(cx)
            });

        assert!(has_test_id(&element, "custom-close-label"));
    }

    #[test]
    fn artifact_title_children_scope_inherited_title_typography() {
        let window = AppWindowId::default();
        let mut app = App::new();
        Theme::with_global_mut(&mut app, |theme| {
            theme.apply_config(&ThemeConfig {
                name: "Artifact Title Test".to_string(),
                metrics: std::collections::HashMap::from([
                    ("font.size".to_string(), 14.0),
                    ("font.line_height".to_string(), 20.0),
                    ("component.artifact.title_text_px".to_string(), 12.0),
                ]),
                ..ThemeConfig::default()
            });
        });

        let element =
            fret_ui::elements::with_element_cx(&mut app, window, bounds(), "test", |cx| {
                ArtifactTitle::new_children([cx.text("Artifact heading")])
                    .test_id("artifact-title")
                    .into_element(cx)
            });

        let text = element
            .children
            .iter()
            .find_map(|child| match &child.kind {
                ElementKind::Text(props) if props.text.as_ref() == "Artifact heading" => {
                    Some(child)
                }
                _ => None,
            })
            .expect("expected nested artifact title text");
        let ElementKind::Text(props) = &text.kind else {
            panic!("expected nested artifact title leaf to be text");
        };
        assert!(props.style.is_none());
        assert!(props.color.is_none());
        assert_eq!(
            element
                .semantics_decoration
                .as_ref()
                .and_then(|d| d.test_id.as_deref()),
            Some("artifact-title")
        );

        let theme = Theme::global(&app).snapshot();
        let style = typography::as_control_text(TextStyle {
            font: FontId::default(),
            size: theme
                .metric_by_key("component.artifact.title_text_px")
                .unwrap_or_else(|| theme.metric_token("font.size")),
            weight: FontWeight::MEDIUM,
            slant: Default::default(),
            line_height: Some(theme.metric_token("font.line_height")),
            letter_spacing_em: None,
            ..Default::default()
        });
        let expected_refinement = typography::composable_refinement_from_style(&style);
        let expected_foreground = theme.color_token("foreground");
        assert!(has_scoped_text_style(
            &element,
            &expected_refinement,
            expected_foreground
        ));
    }

    #[test]
    fn artifact_description_scopes_inherited_description_typography() {
        let window = AppWindowId::default();
        let mut app = App::new();
        Theme::with_global_mut(&mut app, |theme| {
            theme.apply_config(&ThemeConfig {
                name: "Artifact Test".to_string(),
                metrics: std::collections::HashMap::from([
                    ("font.size".to_string(), 14.0),
                    ("font.line_height".to_string(), 20.0),
                    ("component.artifact.description_text_px".to_string(), 12.0),
                ]),
                colors: std::collections::HashMap::from([(
                    "muted-foreground".to_string(),
                    "#778899".to_string(),
                )]),
                ..ThemeConfig::default()
            });
        });

        let element =
            fret_ui::elements::with_element_cx(&mut app, window, bounds(), "test", |cx| {
                ArtifactDescription::new("Artifact summary")
                    .test_id("artifact-description")
                    .into_element(cx)
            });

        let ElementKind::Text(props) = &element.kind else {
            panic!("expected ArtifactDescription to build a Text element");
        };
        assert!(props.style.is_none());
        assert!(props.color.is_none());
        assert_eq!(props.wrap, fret_core::TextWrap::None);
        assert_eq!(
            element
                .semantics_decoration
                .as_ref()
                .and_then(|d| d.test_id.as_deref()),
            Some("artifact-description")
        );

        let theme = Theme::global(&app).snapshot();
        assert_eq!(
            element.inherited_foreground,
            Some(typography::muted_foreground_color(&theme))
        );
        assert_eq!(
            element.inherited_text_style,
            Some(typography::description_text_refinement_with_fallbacks(
                &theme,
                "component.artifact.description_text",
                Some("font.size"),
                Some("font.line_height"),
            ))
        );
    }

    #[test]
    fn artifact_description_children_scope_inherited_description_typography() {
        let window = AppWindowId::default();
        let mut app = App::new();
        Theme::with_global_mut(&mut app, |theme| {
            theme.apply_config(&ThemeConfig {
                name: "Artifact Description Test".to_string(),
                metrics: std::collections::HashMap::from([
                    ("font.size".to_string(), 14.0),
                    ("font.line_height".to_string(), 20.0),
                    ("component.artifact.description_text_px".to_string(), 12.0),
                ]),
                colors: std::collections::HashMap::from([(
                    "muted-foreground".to_string(),
                    "#778899".to_string(),
                )]),
                ..ThemeConfig::default()
            });
        });

        let element =
            fret_ui::elements::with_element_cx(&mut app, window, bounds(), "test", |cx| {
                ArtifactDescription::new_children([cx.text("Artifact summary")])
                    .test_id("artifact-description")
                    .into_element(cx)
            });

        let text = element
            .children
            .iter()
            .find_map(|child| match &child.kind {
                ElementKind::Text(props) if props.text.as_ref() == "Artifact summary" => {
                    Some(child)
                }
                _ => None,
            })
            .expect("expected nested artifact description text");
        let ElementKind::Text(props) = &text.kind else {
            panic!("expected nested artifact description leaf to be text");
        };
        assert!(props.style.is_none());
        assert!(props.color.is_none());
        assert_eq!(
            element
                .semantics_decoration
                .as_ref()
                .and_then(|d| d.test_id.as_deref()),
            Some("artifact-description")
        );

        let theme = Theme::global(&app).snapshot();
        let expected_refinement = typography::description_text_refinement_with_fallbacks(
            &theme,
            "component.artifact.description_text",
            Some("font.size"),
            Some("font.line_height"),
        );
        let expected_foreground = typography::muted_foreground_color(&theme);
        assert!(has_scoped_text_style(
            &element,
            &expected_refinement,
            expected_foreground
        ));
    }
}
