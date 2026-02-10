use std::sync::Arc;

use fret_core::{Color, Px};
use fret_icons::IconId;
use fret_runtime::Model;
use fret_ui::action::OnActivate;
use fret_ui::element::{AnyElement, LayoutStyle, Length, SizeStyle, SpacerProps};
use fret_ui::{ElementContext, Theme, UiHost};
use fret_ui_kit::declarative::controllable_state::use_controllable_model;
use fret_ui_kit::declarative::icon as decl_icon;
use fret_ui_kit::declarative::model_watch::ModelWatchExt as _;
use fret_ui_kit::declarative::stack;
use fret_ui_kit::declarative::style as decl_style;
use fret_ui_kit::{
    ChromeRefinement, ColorRef, LayoutRefinement, Radius, Space, WidgetStateProperty, WidgetStates,
    ui,
};

use crate::button::{Button, ButtonSize, ButtonStyle, ButtonVariant};
use crate::test_id::attach_test_id;

fn alpha_mul(mut c: Color, mul: f32) -> Color {
    c.a = (c.a * mul).clamp(0.0, 1.0);
    c
}

fn hidden_element<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    let layout = LayoutStyle {
        size: SizeStyle {
            width: Length::Px(Px(0.0)),
            height: Length::Px(Px(0.0)),
            ..Default::default()
        },
        flex: fret_ui::element::FlexItemStyle {
            grow: 0.0,
            shrink: 0.0,
            basis: Length::Px(Px(0.0)),
            ..Default::default()
        },
        ..Default::default()
    };
    cx.spacer(SpacerProps {
        layout,
        min: Px(0.0),
    })
}

#[derive(Clone)]
struct BannerScope {
    visible: Model<bool>,
    on_close: Option<OnActivate>,
}

#[derive(Default)]
struct BannerScopeState {
    scope: Option<BannerScope>,
}

fn banner_scope_inherited<H: UiHost>(cx: &ElementContext<'_, H>) -> Option<BannerScope> {
    cx.inherited_state_where::<BannerScopeState>(|st| st.scope.is_some())
        .and_then(|st| st.scope.clone())
}

#[track_caller]
fn with_banner_scope_provider<H: UiHost, R>(
    cx: &mut ElementContext<'_, H>,
    scope: BannerScope,
    f: impl FnOnce(&mut ElementContext<'_, H>) -> R,
) -> R {
    let prev = cx.with_state(BannerScopeState::default, |st| st.scope.take());
    cx.with_state(BannerScopeState::default, |st| {
        st.scope = Some(scope);
    });
    let out = f(cx);
    cx.with_state(BannerScopeState::default, |st| {
        st.scope = prev;
    });
    out
}

/// A dismissible banner block inspired by Kibo's shadcn blocks.
///
/// Upstream inspiration (MIT):
/// - `repo-ref/kibo/packages/banner`
#[derive(Clone)]
pub struct Banner {
    children: Vec<AnyElement>,
    visible: Option<Model<bool>>,
    default_visible: bool,
    on_close: Option<OnActivate>,
    inset: bool,
    test_id: Option<Arc<str>>,
    chrome: ChromeRefinement,
    layout: LayoutRefinement,
}

impl std::fmt::Debug for Banner {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Banner")
            .field("children_len", &self.children.len())
            .field("visible", &self.visible.is_some())
            .field("default_visible", &self.default_visible)
            .field("on_close", &self.on_close.is_some())
            .field("inset", &self.inset)
            .field("test_id", &self.test_id)
            .field("chrome", &self.chrome)
            .field("layout", &self.layout)
            .finish()
    }
}

impl Banner {
    pub fn new(children: impl IntoIterator<Item = AnyElement>) -> Self {
        Self {
            children: children.into_iter().collect(),
            visible: None,
            default_visible: true,
            on_close: None,
            inset: false,
            test_id: None,
            chrome: ChromeRefinement::default(),
            layout: LayoutRefinement::default(),
        }
    }

    /// Controlled visibility model (Radix-style `visible` / `defaultVisible`).
    pub fn visible_model(mut self, visible: Model<bool>) -> Self {
        self.visible = Some(visible);
        self
    }

    pub fn default_visible(mut self, visible: bool) -> Self {
        self.default_visible = visible;
        self
    }

    pub fn on_close(mut self, on_close: OnActivate) -> Self {
        self.on_close = Some(on_close);
        self
    }

    pub fn inset(mut self, inset: bool) -> Self {
        self.inset = inset;
        self
    }

    pub fn test_id(mut self, id: impl Into<Arc<str>>) -> Self {
        self.test_id = Some(id.into());
        self
    }

    pub fn refine_style(mut self, style: ChromeRefinement) -> Self {
        self.chrome = self.chrome.merge(style);
        self
    }

    pub fn refine_layout(mut self, layout: LayoutRefinement) -> Self {
        self.layout = self.layout.merge(layout);
        self
    }

    #[track_caller]
    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        cx.scope(|cx| {
            let theme = Theme::global(&*cx.app).clone();

            let visible = use_controllable_model(cx, self.visible, || self.default_visible);
            let is_visible = cx
                .watch_model(&visible.model())
                .layout()
                .copied()
                .unwrap_or(true);
            if !is_visible {
                return hidden_element(cx);
            }

            let bg = theme.color_required("primary");
            let fg = theme.color_required("primary-foreground");

            let mut chrome = ChromeRefinement::default()
                .px(Space::N4)
                .py(Space::N2)
                .bg(ColorRef::Color(bg))
                .text_color(ColorRef::Color(fg));
            if self.inset {
                chrome = chrome.rounded(Radius::Lg);
            }
            chrome = chrome.merge(self.chrome);

            let layout = LayoutRefinement::default().w_full().merge(self.layout);
            let props = decl_style::container_props(&theme, chrome, layout);
            let children = self.children;

            let scope = BannerScope {
                visible: visible.model(),
                on_close: self.on_close.clone(),
            };

            let test_id = self
                .test_id
                .unwrap_or_else(|| Arc::<str>::from("shadcn-extras.banner"));

            let el = with_banner_scope_provider(cx, scope, |cx| {
                cx.container(props, move |cx| {
                    vec![stack::hstack(
                        cx,
                        stack::HStackProps::default()
                            .justify_between()
                            .items_center()
                            .gap_x(Space::N2)
                            .layout(LayoutRefinement::default().w_full().min_w_0()),
                        |_cx| children,
                    )]
                })
            });

            attach_test_id(el, test_id)
        })
    }
}

#[derive(Debug, Clone)]
pub struct BannerIcon {
    child: AnyElement,
    test_id: Option<Arc<str>>,
}

impl BannerIcon {
    pub fn new(child: AnyElement) -> Self {
        Self {
            child,
            test_id: None,
        }
    }

    pub fn test_id(mut self, id: impl Into<Arc<str>>) -> Self {
        self.test_id = Some(id.into());
        self
    }

    #[track_caller]
    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let theme = Theme::global(&*cx.app).clone();
        let base = theme.color_required("background");
        let border = alpha_mul(base, 0.20);
        let bg = alpha_mul(base, 0.10);

        let chrome = ChromeRefinement::default()
            .p(Space::N1)
            .rounded(Radius::Full)
            .border_1()
            .border_color(ColorRef::Color(border))
            .bg(ColorRef::Color(bg))
            .shadow_sm();

        let props = decl_style::container_props(&theme, chrome, LayoutRefinement::default());
        let child = self.child;
        let el = cx.container(props, move |_cx| vec![child]);
        attach_test_id(
            el,
            self.test_id
                .unwrap_or_else(|| Arc::<str>::from("shadcn-extras.banner-icon")),
        )
    }
}

#[derive(Debug, Clone)]
pub struct BannerTitle {
    text: Arc<str>,
    test_id: Option<Arc<str>>,
}

impl BannerTitle {
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

    #[track_caller]
    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let el = ui::text(cx, self.text)
            .text_sm()
            .flex_1()
            .min_w_0()
            .into_element(cx);
        attach_test_id(
            el,
            self.test_id
                .unwrap_or_else(|| Arc::<str>::from("shadcn-extras.banner-title")),
        )
    }
}

#[derive(Clone)]
pub struct BannerAction {
    label: Arc<str>,
    on_activate: Option<OnActivate>,
    disabled: bool,
    test_id: Option<Arc<str>>,
}

impl std::fmt::Debug for BannerAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BannerAction")
            .field("label", &self.label)
            .field("on_activate", &self.on_activate.is_some())
            .field("disabled", &self.disabled)
            .field("test_id", &self.test_id)
            .finish()
    }
}

impl BannerAction {
    pub fn new(label: impl Into<Arc<str>>) -> Self {
        Self {
            label: label.into(),
            on_activate: None,
            disabled: false,
            test_id: None,
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

    #[track_caller]
    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let theme = Theme::global(&*cx.app).clone();
        let fg = theme.color_required("primary-foreground");
        let mut hover_bg = theme.color_required("background");
        hover_bg = alpha_mul(hover_bg, 0.10);

        let style = ButtonStyle::default()
            .background(
                WidgetStateProperty::new(Some(ColorRef::Color(Color::TRANSPARENT)))
                    .when(WidgetStates::HOVERED, Some(ColorRef::Color(hover_bg))),
            )
            .foreground(WidgetStateProperty::new(Some(ColorRef::Color(fg))));

        let mut button = Button::new(self.label)
            .variant(ButtonVariant::Outline)
            .size(ButtonSize::Sm)
            .disabled(self.disabled)
            .style(style);
        if let Some(id) = self.test_id {
            button = button.test_id(id);
        }
        if let Some(on_activate) = self.on_activate {
            button = button.on_activate(on_activate);
        }
        button.into_element(cx)
    }
}

#[derive(Clone)]
pub struct BannerClose {
    on_activate: Option<OnActivate>,
    disabled: bool,
    test_id: Option<Arc<str>>,
}

impl std::fmt::Debug for BannerClose {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BannerClose")
            .field("on_activate", &self.on_activate.is_some())
            .field("disabled", &self.disabled)
            .field("test_id", &self.test_id)
            .finish()
    }
}

impl BannerClose {
    pub fn new() -> Self {
        Self {
            on_activate: None,
            disabled: false,
            test_id: None,
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

    #[track_caller]
    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let theme = Theme::global(&*cx.app).clone();
        let fg = theme.color_required("primary-foreground");
        let mut hover_bg = theme.color_required("background");
        hover_bg = alpha_mul(hover_bg, 0.10);

        let scope = banner_scope_inherited(cx);
        let visible = scope.as_ref().map(|s| s.visible.clone());
        let on_close = scope.and_then(|s| s.on_close.clone());
        let on_activate_user = self.on_activate.clone();

        let style = ButtonStyle::default()
            .background(
                WidgetStateProperty::new(Some(ColorRef::Color(Color::TRANSPARENT)))
                    .when(WidgetStates::HOVERED, Some(ColorRef::Color(hover_bg))),
            )
            .foreground(WidgetStateProperty::new(Some(ColorRef::Color(fg))));

        let icon = decl_icon::icon_with(
            cx,
            IconId::new_static("lucide.x"),
            Some(Px(18.0)),
            Some(ColorRef::Color(fg)),
        );

        let on_activate: OnActivate = Arc::new(move |host, action_cx, reason| {
            if let Some(visible) = visible.as_ref() {
                let _ = host.models_mut().update(visible, |v| *v = false);
                host.notify(action_cx);
            }

            if let Some(on_close) = on_close.as_ref() {
                on_close(host, action_cx, reason);
            }
            if let Some(user) = on_activate_user.as_ref() {
                user(host, action_cx, reason);
            }
        });

        let mut button = Button::new("Close")
            .variant(ButtonVariant::Ghost)
            .size(ButtonSize::Icon)
            .disabled(self.disabled)
            .style(style)
            .children([icon])
            .on_activate(on_activate);

        if let Some(id) = self.test_id {
            button = button.test_id(id);
        }

        button.into_element(cx)
    }
}

impl Default for BannerClose {
    fn default() -> Self {
        Self::new()
    }
}
