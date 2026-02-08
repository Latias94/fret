use std::sync::Arc;

use fret_core::{Color, Edges, FontId, FontWeight, Px, SemanticsRole, TextStyle};
use fret_icons::IconId;
use fret_runtime::{CommandId, Model};
use fret_ui::action::OnActivate;
use fret_ui::element::{
    AnyElement, CrossAlign, Elements, FlexProps, MainAlign, OpacityProps, Overflow, PressableProps,
    RingStyle, SpacerProps,
};
use fret_ui::{ElementContext, Theme, UiHost};
use fret_ui_kit::command::ElementCommandGatingExt as _;
use fret_ui_kit::declarative::action_hooks::ActionHooksExt as _;
use fret_ui_kit::declarative::icon as decl_icon;
use fret_ui_kit::declarative::model_watch::ModelWatchExt as _;
use fret_ui_kit::declarative::scheduling;
use fret_ui_kit::declarative::scroll as decl_scroll;
use fret_ui_kit::declarative::style as decl_style;
use fret_ui_kit::primitives::controllable_state;
use fret_ui_kit::primitives::transition as transition_prim;
use fret_ui_kit::{ui, ChromeRefinement, ColorRef, LayoutRefinement, MetricRef, Radius, Space};

use crate::input::InputStyle as ShadcnInputStyle;
use crate::layout as shadcn_layout;
use crate::overlay_motion;
use crate::tooltip::{Tooltip, TooltipAlign, TooltipContent, TooltipProvider, TooltipSide};
use crate::SeparatorOrientation;
use crate::{Button, ButtonSize, ButtonVariant, Input};

fn alpha_mul(mut c: Color, mul: f32) -> Color {
    c.a *= mul;
    c
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum SidebarMenuButtonSize {
    Sm,
    #[default]
    Default,
    Lg,
}

fn sidebar_menu_button_h(theme: &Theme, size: SidebarMenuButtonSize) -> Px {
    let (key, fallback) = match size {
        SidebarMenuButtonSize::Sm => ("component.sidebar.menu_button.h_sm", Px(28.0)), // `h-7`
        SidebarMenuButtonSize::Default => ("component.sidebar.menu_button.h", Px(32.0)), // `h-8`
        SidebarMenuButtonSize::Lg => ("component.sidebar.menu_button.h_lg", Px(48.0)), // `h-12`
    };
    theme.metric_by_key(key).unwrap_or(fallback)
}

fn sidebar_menu_affordance_top(size: SidebarMenuButtonSize) -> Px {
    match size {
        SidebarMenuButtonSize::Sm => Px(4.0),      // `top-1`
        SidebarMenuButtonSize::Default => Px(6.0), // `top-1.5`
        SidebarMenuButtonSize::Lg => Px(10.0),     // `top-2.5`
    }
}

fn sidebar_menu_button_collapsed_h(theme: &Theme) -> Px {
    theme
        .metric_by_key("component.sidebar.menu_button.h_collapsed")
        .unwrap_or(Px(32.0)) // `size-8!`
}

fn sidebar_width(theme: &Theme) -> Px {
    theme
        .metric_by_key("component.sidebar.width")
        .unwrap_or(Px(256.0))
}

fn sidebar_width_icon(theme: &Theme) -> Px {
    theme
        .metric_by_key("component.sidebar.width_icon")
        .unwrap_or(Px(48.0))
}

const SIDEBAR_COLLAPSE_OPEN_TICKS: u64 = overlay_motion::SHADCN_MOTION_TICKS_200;
const SIDEBAR_COLLAPSE_CLOSE_TICKS: u64 = overlay_motion::SHADCN_MOTION_TICKS_200;

#[track_caller]
fn sidebar_collapse_motion<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    collapsed: bool,
) -> transition_prim::TransitionOutput {
    let motion = transition_prim::drive_transition_with_durations_and_easing_with_mount_behavior(
        cx,
        !collapsed,
        SIDEBAR_COLLAPSE_OPEN_TICKS,
        SIDEBAR_COLLAPSE_CLOSE_TICKS,
        overlay_motion::shadcn_ease,
        false,
    );

    scheduling::set_continuous_frames(cx, motion.animating);
    motion
}

fn sidebar_bg(theme: &Theme) -> Color {
    theme
        .color_by_key("sidebar.background")
        .or_else(|| theme.color_by_key("sidebar"))
        .unwrap_or_else(|| theme.color_required("sidebar"))
}

fn sidebar_fg(theme: &Theme) -> Color {
    theme
        .color_by_key("sidebar.foreground")
        .or_else(|| theme.color_by_key("sidebar-foreground"))
        .unwrap_or_else(|| theme.color_required("sidebar-foreground"))
}

fn sidebar_border(theme: &Theme) -> Color {
    theme
        .color_by_key("sidebar.border")
        .or_else(|| theme.color_by_key("sidebar-border"))
        .or_else(|| theme.color_by_key("border"))
        .unwrap_or_else(|| theme.color_required("sidebar-border"))
}

fn sidebar_accent(theme: &Theme) -> Color {
    theme
        .color_by_key("sidebar.accent")
        .or_else(|| theme.color_by_key("sidebar-accent"))
        .or_else(|| theme.color_by_key("accent"))
        .unwrap_or_else(|| theme.color_required("sidebar-accent"))
}

fn sidebar_accent_fg(theme: &Theme) -> Color {
    theme
        .color_by_key("sidebar.accent.foreground")
        .or_else(|| theme.color_by_key("sidebar-accent-foreground"))
        .or_else(|| theme.color_by_key("accent-foreground"))
        .unwrap_or_else(|| theme.color_required("sidebar-accent-foreground"))
}

fn sidebar_ring(theme: &Theme, radius: Px) -> RingStyle {
    decl_style::focus_ring(theme, radius)
}

fn menu_button_style(theme: &Theme) -> TextStyle {
    let size = theme
        .metric_by_key("component.sidebar.menu_button_px")
        .or_else(|| theme.metric_by_key("font.size"))
        .unwrap_or_else(|| theme.metric_required("font.size"));
    let line_height = theme
        .metric_by_key("component.sidebar.menu_button_line_height")
        .or_else(|| theme.metric_by_key("font.line_height"))
        .unwrap_or_else(|| theme.metric_required("font.line_height"));
    TextStyle {
        font: FontId::default(),
        size,
        weight: FontWeight::MEDIUM,
        line_height: Some(line_height),
        ..Default::default()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SidebarState {
    Expanded,
    Collapsed,
}

impl SidebarState {
    fn collapsed(self) -> bool {
        matches!(self, Self::Collapsed)
    }
}

#[derive(Debug, Clone)]
pub struct SidebarContext {
    pub state: SidebarState,
    pub open: Model<bool>,
    pub open_mobile: Model<bool>,
    pub is_mobile: bool,
}

impl SidebarContext {
    pub fn collapsed(&self) -> bool {
        self.state.collapsed()
    }

    pub fn toggle_sidebar<H: UiHost>(&self, cx: &mut ElementContext<'_, H>) {
        if self.is_mobile {
            cx.pressable_toggle_bool(&self.open_mobile);
        } else {
            cx.pressable_toggle_bool(&self.open);
        }
    }
}

#[derive(Debug, Default, Clone)]
struct SidebarProviderState {
    context: Option<SidebarContext>,
}

pub fn use_sidebar<H: UiHost>(cx: &ElementContext<'_, H>) -> Option<SidebarContext> {
    cx.inherited_state_where::<SidebarProviderState>(|st| st.context.is_some())
        .and_then(|st| st.context.clone())
}

#[track_caller]
fn with_sidebar_provider_state<H: UiHost, R>(
    cx: &mut ElementContext<'_, H>,
    context: SidebarContext,
    f: impl FnOnce(&mut ElementContext<'_, H>) -> R,
) -> R {
    let prev = cx.with_state(SidebarProviderState::default, |st| {
        let prev = st.context.clone();
        st.context = Some(context);
        prev
    });
    let out = f(cx);
    cx.with_state(SidebarProviderState::default, |st| {
        st.context = prev;
    });
    out
}

fn sidebar_collapsed_in_scope<H: UiHost>(cx: &ElementContext<'_, H>) -> bool {
    use_sidebar(cx).map(|ctx| ctx.collapsed()).unwrap_or(false)
}

/// shadcn/ui `SidebarProvider` (V1).
///
/// Provides shared sidebar open/collapsed state and wraps descendants in `TooltipProvider`
/// with upstream-aligned default delay (`0`).
#[derive(Debug, Clone)]
pub struct SidebarProvider {
    open: Option<Model<bool>>,
    default_open: bool,
    open_mobile: Option<Model<bool>>,
    default_open_mobile: bool,
    is_mobile: bool,
}

impl SidebarProvider {
    pub fn new() -> Self {
        Self {
            open: None,
            default_open: true,
            open_mobile: None,
            default_open_mobile: false,
            is_mobile: false,
        }
    }

    pub fn open(mut self, open: Option<Model<bool>>) -> Self {
        self.open = open;
        self
    }

    pub fn default_open(mut self, default_open: bool) -> Self {
        self.default_open = default_open;
        self
    }

    pub fn open_mobile(mut self, open_mobile: Option<Model<bool>>) -> Self {
        self.open_mobile = open_mobile;
        self
    }

    pub fn default_open_mobile(mut self, default_open_mobile: bool) -> Self {
        self.default_open_mobile = default_open_mobile;
        self
    }

    pub fn is_mobile(mut self, is_mobile: bool) -> Self {
        self.is_mobile = is_mobile;
        self
    }

    pub fn with<H: UiHost, I>(
        self,
        cx: &mut ElementContext<'_, H>,
        f: impl FnOnce(&mut ElementContext<'_, H>) -> I,
    ) -> Vec<AnyElement>
    where
        I: IntoIterator<Item = AnyElement>,
    {
        self.with_elements(cx, f).into_vec()
    }

    pub fn with_elements<H: UiHost, I>(
        self,
        cx: &mut ElementContext<'_, H>,
        f: impl FnOnce(&mut ElementContext<'_, H>) -> I,
    ) -> Elements
    where
        I: IntoIterator<Item = AnyElement>,
    {
        let open =
            controllable_state::use_controllable_model(cx, self.open, || self.default_open).model();
        let open_mobile = controllable_state::use_controllable_model(cx, self.open_mobile, || {
            self.default_open_mobile
        })
        .model();

        let open_now = cx.watch_model(&open).layout().copied().unwrap_or(true);
        let state = if open_now {
            SidebarState::Expanded
        } else {
            SidebarState::Collapsed
        };

        let context = SidebarContext {
            state,
            open,
            open_mobile,
            is_mobile: self.is_mobile,
        };

        with_sidebar_provider_state(cx, context, |cx| {
            TooltipProvider::new()
                .delay_duration_frames(0)
                .with_elements(cx, f)
        })
    }
}

/// shadcn/ui `Sidebar` (V1).
///
/// This is implemented as a declarative composition surface (not a retained widget), so it can
/// fully participate in Tailwind-like layout/style refinements.
#[derive(Debug, Clone)]
pub struct Sidebar {
    children: Vec<AnyElement>,
    collapsed: bool,
    chrome: ChromeRefinement,
    layout: LayoutRefinement,
}

impl Sidebar {
    pub fn new(children: impl IntoIterator<Item = AnyElement>) -> Self {
        let children = children.into_iter().collect();
        Self {
            children,
            collapsed: false,
            chrome: ChromeRefinement::default(),
            layout: LayoutRefinement::default(),
        }
    }

    pub fn collapsed(mut self, collapsed: bool) -> Self {
        self.collapsed = collapsed;
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

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let collapsed = sidebar_collapsed_in_scope(cx);
        let collapsed = if self.collapsed { true } else { collapsed };
        let theme = Theme::global(&*cx.app).clone();

        let motion = sidebar_collapse_motion(cx, collapsed);
        let expanded_progress = motion.progress;
        let w = transition_prim::lerp_px(
            sidebar_width_icon(&theme),
            sidebar_width(&theme),
            expanded_progress,
        );
        let layout = LayoutRefinement::default()
            .w_px(w)
            .h_full()
            .merge(self.layout);

        let chrome = ChromeRefinement::default()
            .bg(ColorRef::Color(sidebar_bg(&theme)))
            .border_1()
            .border_color(ColorRef::Color(sidebar_border(&theme)))
            .merge(self.chrome);

        let mut props = decl_style::container_props(&theme, chrome, layout);
        props.layout.overflow = Overflow::Clip;

        let children = self.children;
        shadcn_layout::container_flow(cx, props, children)
    }
}

#[derive(Clone)]
pub struct SidebarTrigger {
    on_click: Option<CommandId>,
    on_activate: Option<OnActivate>,
    disabled: bool,
    test_id: Option<Arc<str>>,
    chrome: ChromeRefinement,
    layout: LayoutRefinement,
}

impl std::fmt::Debug for SidebarTrigger {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SidebarTrigger")
            .field("on_click", &self.on_click)
            .field("on_activate", &self.on_activate.is_some())
            .field("disabled", &self.disabled)
            .field("test_id", &self.test_id)
            .field("chrome", &self.chrome)
            .field("layout", &self.layout)
            .finish()
    }
}

impl SidebarTrigger {
    pub fn new() -> Self {
        Self {
            on_click: None,
            on_activate: None,
            disabled: false,
            test_id: None,
            chrome: ChromeRefinement::default(),
            layout: LayoutRefinement::default(),
        }
    }

    pub fn on_click(mut self, command: impl Into<CommandId>) -> Self {
        self.on_click = Some(command.into());
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

    pub fn refine_style(mut self, style: ChromeRefinement) -> Self {
        self.chrome = self.chrome.merge(style);
        self
    }

    pub fn refine_layout(mut self, layout: LayoutRefinement) -> Self {
        self.layout = self.layout.merge(layout);
        self
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let sidebar_ctx = use_sidebar(cx);
        let user_on_activate = self.on_activate.clone();
        let toggle_on_activate: Option<OnActivate> =
            if user_on_activate.is_none() && sidebar_ctx.is_none() {
                None
            } else {
                Some(Arc::new(move |host, action_cx, reason| {
                    if let Some(on_activate) = user_on_activate.as_ref() {
                        on_activate(host, action_cx, reason);
                    }

                    if let Some(ctx) = sidebar_ctx.as_ref() {
                        let model = if ctx.is_mobile {
                            ctx.open_mobile.clone()
                        } else {
                            ctx.open.clone()
                        };
                        let _ = host.models_mut().update(&model, |v| {
                            *v = !*v;
                        });
                    }
                }))
            };

        let mut trigger = Button::new("Toggle Sidebar")
            .variant(ButtonVariant::Ghost)
            .size(ButtonSize::Icon)
            .children([decl_icon::icon(cx, IconId::new_static("lucide.panel-left"))])
            .disabled(self.disabled)
            .refine_style(self.chrome)
            .refine_layout(
                LayoutRefinement::default()
                    .w_px(Px(28.0))
                    .h_px(Px(28.0))
                    .merge(self.layout),
            );

        if let Some(command) = self.on_click {
            trigger = trigger.on_click(command);
        }
        if let Some(on_activate) = toggle_on_activate {
            trigger = trigger.on_activate(on_activate);
        }
        if let Some(test_id) = self.test_id {
            trigger = trigger.test_id(test_id);
        }

        trigger.into_element(cx)
    }
}

#[derive(Debug, Clone)]
pub struct SidebarInset {
    children: Vec<AnyElement>,
    chrome: ChromeRefinement,
    layout: LayoutRefinement,
}

impl SidebarInset {
    pub fn new(children: impl IntoIterator<Item = AnyElement>) -> Self {
        let children = children.into_iter().collect();
        Self {
            children,
            chrome: ChromeRefinement::default(),
            layout: LayoutRefinement::default(),
        }
    }

    pub fn refine_style(mut self, style: ChromeRefinement) -> Self {
        self.chrome = self.chrome.merge(style);
        self
    }

    pub fn refine_layout(mut self, layout: LayoutRefinement) -> Self {
        self.layout = self.layout.merge(layout);
        self
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let theme = Theme::global(&*cx.app).clone();
        let chrome = ChromeRefinement::default()
            .bg(ColorRef::Color(theme.color_required("background")))
            .merge(self.chrome);
        let layout = LayoutRefinement::default()
            .w_full()
            .h_full()
            .flex_1()
            .merge(self.layout);
        let props = decl_style::container_props(&theme, chrome, layout);
        let children = self.children;
        shadcn_layout::container_flow(cx, props, children)
    }
}

#[derive(Debug, Clone)]
pub struct SidebarInput {
    model: Model<String>,
    a11y_label: Option<Arc<str>>,
    placeholder: Option<Arc<str>>,
    disabled: bool,
    submit_command: Option<CommandId>,
    cancel_command: Option<CommandId>,
    chrome: ChromeRefinement,
    layout: LayoutRefinement,
}

impl SidebarInput {
    pub fn new(model: Model<String>) -> Self {
        Self {
            model,
            a11y_label: None,
            placeholder: None,
            disabled: false,
            submit_command: None,
            cancel_command: None,
            chrome: ChromeRefinement::default(),
            layout: LayoutRefinement::default(),
        }
    }

    pub fn a11y_label(mut self, label: impl Into<Arc<str>>) -> Self {
        self.a11y_label = Some(label.into());
        self
    }

    pub fn placeholder(mut self, placeholder: impl Into<Arc<str>>) -> Self {
        self.placeholder = Some(placeholder.into());
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn submit_command(mut self, command: impl Into<CommandId>) -> Self {
        self.submit_command = Some(command.into());
        self
    }

    pub fn cancel_command(mut self, command: impl Into<CommandId>) -> Self {
        self.cancel_command = Some(command.into());
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

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let theme = Theme::global(&*cx.app).clone();

        let mut input = Input::new(self.model)
            .disabled(self.disabled)
            .style(
                ShadcnInputStyle::default()
                    .background(ColorRef::Color(theme.color_required("background"))),
            )
            .refine_style(self.chrome)
            .refine_layout(
                LayoutRefinement::default()
                    .h_px(Px(32.0))
                    .w_full()
                    .merge(self.layout),
            );

        if let Some(label) = self.a11y_label {
            input = input.a11y_label(label);
        }
        if let Some(placeholder) = self.placeholder {
            input = input.placeholder(placeholder);
        }
        if let Some(command) = self.submit_command {
            input = input.submit_command(command);
        }
        if let Some(command) = self.cancel_command {
            input = input.cancel_command(command);
        }

        input.into_element(cx)
    }
}

#[derive(Debug, Clone)]
pub struct SidebarSeparator {
    orientation: SeparatorOrientation,
    layout: LayoutRefinement,
}

impl SidebarSeparator {
    pub fn new() -> Self {
        Self {
            orientation: SeparatorOrientation::Horizontal,
            layout: LayoutRefinement::default(),
        }
    }

    pub fn orientation(mut self, orientation: SeparatorOrientation) -> Self {
        self.orientation = orientation;
        self
    }

    pub fn refine_layout(mut self, layout: LayoutRefinement) -> Self {
        self.layout = self.layout.merge(layout);
        self
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let theme = Theme::global(&*cx.app).clone();
        let thickness = theme
            .metric_by_key("component.separator.px")
            .unwrap_or(Px(1.0));
        let margin_x = decl_style::space(&theme, Space::N2);
        let mut layout = decl_style::layout_style(
            &theme,
            LayoutRefinement::default()
                .mx_px(margin_x)
                .merge(self.layout),
        );

        match self.orientation {
            SeparatorOrientation::Horizontal => {
                layout.size = fret_ui::element::SizeStyle {
                    width: fret_ui::element::Length::Fill,
                    height: fret_ui::element::Length::Px(thickness),
                    min_height: Some(thickness),
                    max_height: Some(thickness),
                    ..layout.size
                };
            }
            SeparatorOrientation::Vertical => {
                layout.size = fret_ui::element::SizeStyle {
                    width: fret_ui::element::Length::Px(thickness),
                    height: fret_ui::element::Length::Fill,
                    min_width: Some(thickness),
                    max_width: Some(thickness),
                    ..layout.size
                };
            }
        }

        cx.container(
            fret_ui::element::ContainerProps {
                layout,
                background: Some(sidebar_border(&theme)),
                ..Default::default()
            },
            |_cx| Vec::new(),
        )
    }
}

#[derive(Debug, Clone)]
pub struct SidebarHeader {
    children: Vec<AnyElement>,
}

impl SidebarHeader {
    pub fn new(children: impl IntoIterator<Item = AnyElement>) -> Self {
        let children = children.into_iter().collect();
        Self { children }
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let theme = Theme::global(&*cx.app).clone();
        let props = decl_style::container_props(
            &theme,
            ChromeRefinement::default().p(Space::N2),
            LayoutRefinement::default(),
        );
        let children = self.children;
        shadcn_layout::container_flow(cx, props, children)
    }
}

#[derive(Debug, Clone)]
pub struct SidebarFooter {
    children: Vec<AnyElement>,
}

impl SidebarFooter {
    pub fn new(children: impl IntoIterator<Item = AnyElement>) -> Self {
        let children = children.into_iter().collect();
        Self { children }
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let theme = Theme::global(&*cx.app).clone();
        let props = decl_style::container_props(
            &theme,
            ChromeRefinement::default().p(Space::N2),
            LayoutRefinement::default(),
        );
        let children = self.children;
        shadcn_layout::container_flow(cx, props, children)
    }
}

#[derive(Debug, Clone)]
pub struct SidebarContent {
    children: Vec<AnyElement>,
    collapsed: bool,
}

impl SidebarContent {
    pub fn new(children: impl IntoIterator<Item = AnyElement>) -> Self {
        let children = children.into_iter().collect();
        Self {
            children,
            collapsed: false,
        }
    }

    pub fn collapsed(mut self, collapsed: bool) -> Self {
        self.collapsed = collapsed;
        self
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let collapsed = sidebar_collapsed_in_scope(cx);
        let collapsed = if self.collapsed { true } else { collapsed };
        let theme = Theme::global(&*cx.app).clone();

        let mut layout = LayoutRefinement::default().h_full();
        if collapsed {
            layout = layout.overflow_hidden();
        }

        let children = self.children;
        decl_scroll::overflow_scrollbar(cx, layout, move |cx| {
            let gap = decl_style::space(&theme, Space::N2);
            let col = FlexProps {
                direction: fret_core::Axis::Vertical,
                gap,
                padding: Edges::all(gap),
                layout: fret_ui::element::LayoutStyle {
                    size: fret_ui::element::SizeStyle {
                        width: fret_ui::element::Length::Fill,
                        height: fret_ui::element::Length::Auto,
                        ..Default::default()
                    },
                    ..Default::default()
                },
                ..Default::default()
            };

            vec![cx.flex(col, move |_cx| children)]
        })
    }
}

#[derive(Debug, Clone)]
pub struct SidebarGroup {
    children: Vec<AnyElement>,
}

impl SidebarGroup {
    pub fn new(children: impl IntoIterator<Item = AnyElement>) -> Self {
        let children = children.into_iter().collect();
        Self { children }
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let theme = Theme::global(&*cx.app).clone();
        let chrome = ChromeRefinement::default().p(Space::N2);
        let props = decl_style::container_props(&theme, chrome, LayoutRefinement::default());
        let children = self.children;
        shadcn_layout::container_flow(cx, props, children)
    }
}

#[derive(Debug, Clone)]
pub struct SidebarGroupLabel {
    text: Arc<str>,
    collapsed: bool,
}

impl SidebarGroupLabel {
    pub fn new(text: impl Into<Arc<str>>) -> Self {
        Self {
            text: text.into(),
            collapsed: false,
        }
    }

    pub fn collapsed(mut self, collapsed: bool) -> Self {
        self.collapsed = collapsed;
        self
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let collapsed = sidebar_collapsed_in_scope(cx);
        let collapsed = if self.collapsed { true } else { collapsed };
        let theme = Theme::global(&*cx.app).clone();
        let motion = sidebar_collapse_motion(cx, collapsed);
        if !motion.present {
            return cx.spacer(fret_ui::element::SpacerProps {
                min: Px(0.0),
                ..Default::default()
            });
        }

        let fg = sidebar_fg(&theme);
        let mut fg = fg;
        fg.a = (fg.a * 0.7).clamp(0.0, 1.0);

        let size = theme
            .metric_by_key("component.sidebar.group_label_px")
            .unwrap_or(Px(12.0));
        let line_height = theme
            .metric_by_key("component.sidebar.group_label_line_height")
            .unwrap_or(Px(16.0));

        let text = ui::text(cx, self.text)
            .text_size_px(size)
            .line_height_px(line_height)
            .font_medium()
            .text_color(ColorRef::Color(fg))
            .nowrap()
            .into_element(cx);

        cx.opacity_props(
            OpacityProps {
                layout: fret_ui::element::LayoutStyle::default(),
                opacity: motion.progress,
            },
            move |_cx| vec![text],
        )
    }
}

#[derive(Debug, Clone)]
pub struct SidebarMenu {
    children: Vec<AnyElement>,
}

impl SidebarMenu {
    pub fn new(children: impl IntoIterator<Item = AnyElement>) -> Self {
        let children = children.into_iter().collect();
        Self { children }
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let props = FlexProps {
            direction: fret_core::Axis::Vertical,
            gap: Px(4.0),
            layout: fret_ui::element::LayoutStyle {
                size: fret_ui::element::SizeStyle {
                    width: fret_ui::element::Length::Fill,
                    height: fret_ui::element::Length::Auto,
                    ..Default::default()
                },
                ..Default::default()
            },
            ..Default::default()
        };
        let children = self.children;
        cx.flex(props, move |_cx| children)
    }
}

#[derive(Debug, Clone)]
pub struct SidebarMenuItem {
    child: AnyElement,
}

impl SidebarMenuItem {
    pub fn new(child: AnyElement) -> Self {
        Self { child }
    }

    pub fn into_element<H: UiHost>(self, _cx: &mut ElementContext<'_, H>) -> AnyElement {
        self.child
    }
}

#[derive(Clone)]
pub struct SidebarMenuButton {
    label: Arc<str>,
    icon: Option<IconId>,
    active: bool,
    disabled: bool,
    collapsed: bool,
    size: SidebarMenuButtonSize,
    on_click: Option<CommandId>,
    on_activate: Option<OnActivate>,
    test_id: Option<Arc<str>>,
}

impl std::fmt::Debug for SidebarMenuButton {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SidebarMenuButton")
            .field("label", &self.label)
            .field("icon", &self.icon)
            .field("active", &self.active)
            .field("disabled", &self.disabled)
            .field("collapsed", &self.collapsed)
            .field("size", &self.size)
            .field("on_click", &self.on_click)
            .field("on_activate", &self.on_activate.is_some())
            .field("test_id", &self.test_id)
            .finish()
    }
}

impl SidebarMenuButton {
    pub fn new(label: impl Into<Arc<str>>) -> Self {
        Self {
            label: label.into(),
            icon: None,
            active: false,
            disabled: false,
            collapsed: false,
            size: SidebarMenuButtonSize::Default,
            on_click: None,
            on_activate: None,
            test_id: None,
        }
    }

    pub fn icon(mut self, icon: IconId) -> Self {
        self.icon = Some(icon);
        self
    }

    pub fn active(mut self, active: bool) -> Self {
        self.active = active;
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn collapsed(mut self, collapsed: bool) -> Self {
        self.collapsed = collapsed;
        self
    }

    pub fn size(mut self, size: SidebarMenuButtonSize) -> Self {
        self.size = size;
        self
    }

    pub fn on_click(mut self, command: impl Into<CommandId>) -> Self {
        self.on_click = Some(command.into());
        self
    }

    pub fn on_activate(mut self, on_activate: OnActivate) -> Self {
        self.on_activate = Some(on_activate);
        self
    }

    pub fn test_id(mut self, id: impl Into<Arc<str>>) -> Self {
        self.test_id = Some(id.into());
        self
    }

    fn build_button<H: UiHost>(
        &self,
        cx: &mut ElementContext<'_, H>,
        expanded_progress: f32,
    ) -> AnyElement {
        let theme = Theme::global(&*cx.app).clone();

        let radius = decl_style::radius(&theme, Radius::Md);
        let ring = sidebar_ring(&theme, radius);

        let label = self.label.clone();
        let h = transition_prim::lerp_px(
            sidebar_menu_button_collapsed_h(&theme),
            sidebar_menu_button_h(&theme, self.size),
            expanded_progress,
        );

        let on_click = self.on_click.clone();
        let on_activate = self.on_activate.clone();
        let test_id = self.test_id.clone();
        let disabled = self.disabled
            || on_click
                .as_ref()
                .is_some_and(|cmd| !cx.command_is_enabled(cmd));
        let pressable = PressableProps {
            enabled: !disabled,
            focus_ring: Some(ring),
            layout: decl_style::layout_style(
                &theme,
                LayoutRefinement::default().w_full().h_px(MetricRef::Px(h)),
            ),
            a11y: fret_ui::element::PressableA11y {
                role: Some(SemanticsRole::Button),
                label: Some(label.clone()),
                test_id: test_id.clone(),
                ..Default::default()
            },
            ..Default::default()
        };

        let icon = self.icon.clone();
        let active = self.active;
        let disabled = disabled;
        let size = self.size;
        let expanded_progress = expanded_progress.clamp(0.0, 1.0);

        cx.pressable(pressable, move |cx, st| {
            cx.pressable_dispatch_command_if_enabled_opt(on_click.clone());
            if let Some(on_activate) = on_activate.clone() {
                cx.pressable_on_activate(on_activate);
            }
            let theme = Theme::global(&*cx.app).clone();

            let bg = if active || st.hovered || st.pressed {
                sidebar_accent(&theme)
            } else {
                Color::TRANSPARENT
            };

            let fg = if disabled {
                alpha_mul(sidebar_fg(&theme), 0.5)
            } else if active || st.hovered || st.pressed {
                sidebar_accent_fg(&theme)
            } else {
                sidebar_fg(&theme)
            };

            let chrome = if bg.a > 0.0 {
                ChromeRefinement::default()
                    .bg(ColorRef::Color(bg))
                    .rounded(Radius::Md)
            } else {
                ChromeRefinement::default().rounded(Radius::Md)
            };

            let h = transition_prim::lerp_px(
                sidebar_menu_button_collapsed_h(&theme),
                sidebar_menu_button_h(&theme, size),
                expanded_progress,
            );

            let mut props = decl_style::container_props(
                &theme,
                chrome,
                LayoutRefinement::default().w_full().h_px(MetricRef::Px(h)),
            );
            props.layout.overflow = Overflow::Clip;

            let inner_gap = decl_style::space(&theme, Space::N2); // `gap-2`

            vec![cx.container(props, move |cx| {
                let row = FlexProps {
                    direction: fret_core::Axis::Horizontal,
                    gap: inner_gap,
                    align: CrossAlign::Center,
                    justify: MainAlign::Start,
                    padding: Edges::all(inner_gap), // `p-2`
                    layout: fret_ui::element::LayoutStyle {
                        size: fret_ui::element::SizeStyle {
                            width: fret_ui::element::Length::Fill,
                            height: fret_ui::element::Length::Fill,
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                    ..Default::default()
                };

                let label = label.clone();
                let icon = icon.clone();
                let label_opacity = expanded_progress;
                vec![cx.flex(row, move |cx| {
                    let mut out = Vec::new();
                    if let Some(icon) = icon.clone() {
                        out.push(decl_icon::icon(cx, icon));
                    }
                    if label_opacity > 0.0 {
                        let style = menu_button_style(&theme);
                        let text = ui::text(cx, label.clone())
                            .w_full()
                            .min_w_0()
                            .flex_1()
                            .basis_0()
                            .text_size_px(style.size)
                            .font_weight(style.weight)
                            .text_color(ColorRef::Color(fg))
                            .truncate();

                        let mut text = text;
                        if let Some(line_height) = style.line_height {
                            text = text.line_height_px(line_height);
                        }
                        if let Some(letter_spacing_em) = style.letter_spacing_em {
                            text = text.letter_spacing_em(letter_spacing_em);
                        }

                        let text = text.into_element(cx);
                        out.push(cx.opacity_props(
                            OpacityProps {
                                layout: fret_ui::element::LayoutStyle::default(),
                                opacity: label_opacity,
                            },
                            move |_cx| vec![text],
                        ));
                    } else {
                        out.push(cx.spacer(SpacerProps {
                            min: Px(0.0),
                            ..Default::default()
                        }));
                    }
                    out
                })]
            })]
        })
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let collapsed = sidebar_collapsed_in_scope(cx);
        let collapsed = if self.collapsed { true } else { collapsed };
        let mut this = self;
        this.collapsed = collapsed;

        let motion = sidebar_collapse_motion(cx, collapsed);
        let expanded_progress = motion.progress;
        let button = this.build_button(cx, expanded_progress);

        if !collapsed || expanded_progress > 0.01 {
            return button;
        }

        // In collapsed (icon) mode, show the label via a tooltip.
        let theme = Theme::global(&*cx.app).clone();

        let label = this.label.clone();

        let chrome = ChromeRefinement::default()
            .bg(ColorRef::Color(
                theme
                    .color_by_key("popover.background")
                    .unwrap_or_else(|| theme.color_required("popover.background")),
            ))
            .border_1()
            .border_color(ColorRef::Color(
                theme
                    .color_by_key("border")
                    .unwrap_or_else(|| theme.color_required("border")),
            ))
            .rounded(Radius::Md)
            .p(Space::N2);
        let content = TooltipContent::new({
            let style = menu_button_style(&theme);
            let mut text = ui::text(cx, label.clone())
                .text_size_px(style.size)
                .font_weight(style.weight)
                .text_color(ColorRef::Color(sidebar_fg(&theme)))
                .nowrap();
            if let Some(line_height) = style.line_height {
                text = text.line_height_px(line_height);
            }
            if let Some(letter_spacing_em) = style.letter_spacing_em {
                text = text.letter_spacing_em(letter_spacing_em);
            }
            vec![text.into_element(cx)]
        })
        .refine_style(chrome)
        .refine_layout(LayoutRefinement::default().overflow_hidden())
        .into_element(cx);

        Tooltip::new(button, content)
            .side(TooltipSide::Right)
            .align(TooltipAlign::Center)
            .side_offset(Px(8.0))
            .into_element(cx)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::shadcn_themes::{apply_shadcn_new_york_v4, ShadcnBaseColor, ShadcnColorScheme};
    use fret_app::App;
    use fret_core::{
        AppWindowId, PathCommand, PathConstraints, PathId, PathMetrics, PathService, Point, Px,
        Rect, SemanticsRole, Size as CoreSize, SvgId, SvgService, TextBlobId, TextConstraints,
        TextMetrics, TextService,
    };
    use fret_runtime::{FrameId, TickId};
    use fret_ui::element::ContainerProps;
    use fret_ui::tree::UiTree;
    use fret_ui_kit::OverlayController;

    struct FakeServices;

    impl TextService for FakeServices {
        fn prepare(
            &mut self,
            _input: &fret_core::TextInput,
            _constraints: TextConstraints,
        ) -> (TextBlobId, TextMetrics) {
            (
                TextBlobId::default(),
                TextMetrics {
                    size: CoreSize::new(Px(10.0), Px(10.0)),
                    baseline: Px(8.0),
                },
            )
        }

        fn release(&mut self, _blob: TextBlobId) {}
    }

    impl PathService for FakeServices {
        fn prepare(
            &mut self,
            _commands: &[PathCommand],
            _style: fret_core::PathStyle,
            _constraints: PathConstraints,
        ) -> (PathId, PathMetrics) {
            (PathId::default(), PathMetrics::default())
        }

        fn release(&mut self, _path: PathId) {}
    }

    impl SvgService for FakeServices {
        fn register_svg(&mut self, _bytes: &[u8]) -> SvgId {
            SvgId::default()
        }

        fn unregister_svg(&mut self, _svg: SvgId) -> bool {
            true
        }
    }

    fn render_sidebar_frame(
        ui: &mut UiTree<App>,
        app: &mut App,
        services: &mut FakeServices,
        window: AppWindowId,
        bounds: Rect,
        collapsed: bool,
        frame: u64,
    ) -> Rect {
        app.set_frame_id(FrameId(frame));
        app.set_tick_id(TickId(frame));

        let root = fret_ui::declarative::render_root(
            ui,
            app,
            services,
            window,
            bounds,
            "shadcn-sidebar-motion",
            |cx| {
                let child = cx.container(ContainerProps::default(), |_cx| Vec::new());
                let sidebar = Sidebar::new([child]).collapsed(collapsed).into_element(cx);
                vec![sidebar]
            },
        );
        ui.set_root(root);
        ui.layout_all(app, services, bounds, 1.0);

        let sidebar_node = *ui.children(root).first().expect("sidebar node");
        ui.debug_node_bounds(sidebar_node).expect("sidebar bounds")
    }

    #[test]
    fn sidebar_collapse_animates_width_between_expanded_and_icon() {
        let window = AppWindowId::default();
        let mut app = App::new();
        apply_shadcn_new_york_v4(&mut app, ShadcnBaseColor::Neutral, ShadcnColorScheme::Light);
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);
        let mut services = FakeServices;

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            CoreSize::new(Px(1200.0), Px(720.0)),
        );

        let theme = Theme::global(&app).clone();
        let expanded_w = sidebar_width(&theme).0;
        let icon_w = sidebar_width_icon(&theme).0;

        let mut expanded_rect = Rect::default();
        for frame in 1..=24 {
            expanded_rect = render_sidebar_frame(
                &mut ui,
                &mut app,
                &mut services,
                window,
                bounds,
                false,
                frame,
            );
        }
        let expanded_actual = expanded_rect.size.width.0;
        assert!(
            (expanded_actual - expanded_w).abs() <= 1.0,
            "expected expanded width ~{expanded_w}, got {expanded_actual}"
        );

        let mut min_transitioning = f32::INFINITY;
        let mut max_transitioning = f32::NEG_INFINITY;
        for frame in 25..=31 {
            let w = render_sidebar_frame(
                &mut ui,
                &mut app,
                &mut services,
                window,
                bounds,
                true,
                frame,
            )
            .size
            .width
            .0;
            min_transitioning = min_transitioning.min(w);
            max_transitioning = max_transitioning.max(w);
        }

        assert!(
            min_transitioning < expanded_w - 0.5,
            "expected collapse motion to reduce width below expanded ({expanded_w}), min={min_transitioning}"
        );
        assert!(
            max_transitioning > icon_w + 0.5,
            "expected early collapse motion to remain above icon width ({icon_w}), max={max_transitioning}"
        );
    }

    #[test]
    fn sidebar_collapse_settles_to_icon_width() {
        let window = AppWindowId::default();
        let mut app = App::new();
        apply_shadcn_new_york_v4(&mut app, ShadcnBaseColor::Neutral, ShadcnColorScheme::Light);
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);
        let mut services = FakeServices;

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            CoreSize::new(Px(1200.0), Px(720.0)),
        );

        let theme = Theme::global(&app).clone();
        let icon_w = sidebar_width_icon(&theme).0;

        for frame in 1..=24 {
            let _ = render_sidebar_frame(
                &mut ui,
                &mut app,
                &mut services,
                window,
                bounds,
                false,
                frame,
            );
        }

        let mut collapsed_rect = Rect::default();
        for frame in 25..=56 {
            collapsed_rect = render_sidebar_frame(
                &mut ui,
                &mut app,
                &mut services,
                window,
                bounds,
                true,
                frame,
            );
        }

        let collapsed_actual = collapsed_rect.size.width.0;
        assert!(
            (collapsed_actual - icon_w).abs() <= 1.0,
            "expected collapsed width ~{icon_w}, got {collapsed_actual}"
        );
    }

    #[test]
    fn sidebar_provider_collapsed_drives_sidebar_width_without_manual_prop() {
        let window = AppWindowId::default();
        let mut app = App::new();
        apply_shadcn_new_york_v4(&mut app, ShadcnBaseColor::Neutral, ShadcnColorScheme::Light);
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);
        let mut services = FakeServices;

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            CoreSize::new(Px(1200.0), Px(720.0)),
        );

        let open_model = app.models_mut().insert(false);

        app.set_frame_id(FrameId(1));
        app.set_tick_id(TickId(1));
        let root = fret_ui::declarative::render_root(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "shadcn-sidebar-provider-collapse",
            |cx| {
                SidebarProvider::new()
                    .open(Some(open_model.clone()))
                    .with(cx, |cx| {
                        let child = cx.container(ContainerProps::default(), |_cx| Vec::new());
                        let sidebar = Sidebar::new([child]).into_element(cx);
                        vec![sidebar]
                    })
            },
        );
        ui.set_root(root);
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        let theme = Theme::global(&app).clone();
        let icon_w = sidebar_width_icon(&theme).0;

        let sidebar_node = *ui.children(root).first().expect("sidebar node");
        let sidebar_bounds = ui.debug_node_bounds(sidebar_node).expect("sidebar bounds");
        assert!(
            (sidebar_bounds.size.width.0 - icon_w).abs() <= 1.0,
            "expected provider-collapsed width ~{icon_w}, got {}",
            sidebar_bounds.size.width.0
        );
    }

    #[test]
    fn sidebar_menu_button_collapsed_uses_tooltip_semantics_not_hover_card() {
        let window = AppWindowId::default();
        let mut app = App::new();
        apply_shadcn_new_york_v4(&mut app, ShadcnBaseColor::Neutral, ShadcnColorScheme::Light);
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);
        let mut services = FakeServices;

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            CoreSize::new(Px(1024.0), Px(640.0)),
        );

        let render_frame =
            |ui: &mut UiTree<App>, app: &mut App, services: &mut FakeServices, frame: u64| {
                app.set_frame_id(FrameId(frame));
                app.set_tick_id(TickId(frame));
                OverlayController::begin_frame(app, window);
                let root = fret_ui::declarative::render_root(
                    ui,
                    app,
                    services,
                    window,
                    bounds,
                    "shadcn-sidebar-collapsed-tooltip",
                    |cx| {
                        TooltipProvider::new()
                            .delay_duration_frames(0)
                            .with_elements(cx, |cx| {
                                let button = SidebarMenuButton::new("Settings")
                                    .collapsed(true)
                                    .icon(IconId::new_static("lucide.settings-2"))
                                    .test_id("sidebar-settings-button")
                                    .into_element(cx);
                                vec![button]
                            })
                    },
                );
                ui.set_root(root);
                OverlayController::render(ui, app, services, window, bounds);
                ui.layout_all(app, services, bounds, 1.0);
            };

        render_frame(&mut ui, &mut app, &mut services, 1);
        ui.request_semantics_snapshot();
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        let first = ui
            .semantics_snapshot()
            .cloned()
            .expect("expected first semantics snapshot");
        let trigger = first
            .nodes
            .iter()
            .find(|n| n.test_id.as_deref() == Some("sidebar-settings-button"))
            .expect("expected sidebar menu button semantics node");
        let trigger_center = Point::new(
            Px(trigger.bounds.origin.x.0 + trigger.bounds.size.width.0 * 0.5),
            Px(trigger.bounds.origin.y.0 + trigger.bounds.size.height.0 * 0.5),
        );

        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::Pointer(fret_core::PointerEvent::Move {
                pointer_id: fret_core::PointerId(0),
                position: trigger_center,
                buttons: fret_core::MouseButtons::default(),
                modifiers: fret_core::Modifiers::default(),
                pointer_type: fret_core::PointerType::Mouse,
            }),
        );

        render_frame(&mut ui, &mut app, &mut services, 2);

        let settle_frames = crate::overlay_motion::SHADCN_MOTION_TICKS_100 + 2;
        for step in 0..settle_frames {
            let tick = 3 + step;
            render_frame(&mut ui, &mut app, &mut services, tick);
        }

        ui.request_semantics_snapshot();
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        let snap = ui
            .semantics_snapshot()
            .cloned()
            .expect("expected semantics snapshot");
        let trigger = snap
            .nodes
            .iter()
            .find(|n| n.test_id.as_deref() == Some("sidebar-settings-button"))
            .expect("expected focused sidebar menu button semantics node");
        let tooltip = snap
            .nodes
            .iter()
            .find(|n| n.role == SemanticsRole::Tooltip)
            .expect("expected collapsed sidebar menu button tooltip semantics node");

        assert!(
            trigger.described_by.iter().any(|id| *id == tooltip.id),
            "expected sidebar menu button to be described by tooltip content when collapsed"
        );
    }

    #[test]
    fn sidebar_input_and_separator_match_shadcn_base_metrics() {
        let window = AppWindowId::default();
        let mut app = App::new();
        apply_shadcn_new_york_v4(&mut app, ShadcnBaseColor::Neutral, ShadcnColorScheme::Light);
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);
        let mut services = FakeServices;

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            CoreSize::new(Px(1024.0), Px(640.0)),
        );

        let model = app.models_mut().insert(String::new());

        let root = fret_ui::declarative::render_root(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "shadcn-sidebar-input-separator",
            |cx| {
                let input = SidebarInput::new(model.clone())
                    .a11y_label("Sidebar Search")
                    .placeholder("Search")
                    .into_element(cx);
                let separator = SidebarSeparator::new().into_element(cx);
                vec![cx.container(ContainerProps::default(), move |_cx| {
                    vec![input.clone(), separator.clone()]
                })]
            },
        );
        ui.set_root(root);
        ui.request_semantics_snapshot();
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        let snap = ui
            .semantics_snapshot()
            .cloned()
            .expect("expected semantics snapshot");

        let input_node = snap
            .nodes
            .iter()
            .find(|n| {
                n.role == SemanticsRole::TextField && n.label.as_deref() == Some("Sidebar Search")
            })
            .expect("expected sidebar input semantics node");
        assert!(
            (input_node.bounds.size.height.0 - 32.0).abs() <= 1.0,
            "expected sidebar input height ~32px, got {}",
            input_node.bounds.size.height.0
        );

        let sep_node = *ui.children(root).first().expect("wrapper node");
        let sep_node = *ui.children(sep_node).get(1).expect("separator node");
        let sep_bounds = ui.debug_node_bounds(sep_node).expect("separator bounds");
        assert!(
            (sep_bounds.size.height.0 - 1.0).abs() <= 1.0,
            "expected sidebar separator thickness ~1px, got {}",
            sep_bounds.size.height.0
        );
    }

    #[test]
    fn sidebar_trigger_toggles_provider_open_model_on_activate() {
        let window = AppWindowId::default();
        let mut app = App::new();
        apply_shadcn_new_york_v4(&mut app, ShadcnBaseColor::Neutral, ShadcnColorScheme::Light);
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);
        let mut services = FakeServices;

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            CoreSize::new(Px(1024.0), Px(640.0)),
        );

        let open_model = app.models_mut().insert(true);

        let root = fret_ui::declarative::render_root(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "shadcn-sidebar-trigger-toggle",
            |cx| {
                SidebarProvider::new()
                    .open(Some(open_model.clone()))
                    .with(cx, |cx| {
                        let trigger = SidebarTrigger::new()
                            .test_id("sidebar-trigger")
                            .into_element(cx);
                        vec![trigger]
                    })
            },
        );
        ui.set_root(root);
        ui.request_semantics_snapshot();
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        let snap = ui
            .semantics_snapshot()
            .cloned()
            .expect("expected semantics snapshot");
        let trigger = snap
            .nodes
            .iter()
            .find(|n| n.test_id.as_deref() == Some("sidebar-trigger"))
            .expect("expected sidebar trigger semantics node");

        let center = Point::new(
            Px(trigger.bounds.origin.x.0 + trigger.bounds.size.width.0 * 0.5),
            Px(trigger.bounds.origin.y.0 + trigger.bounds.size.height.0 * 0.5),
        );

        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::Pointer(fret_core::PointerEvent::Down {
                pointer_id: fret_core::PointerId(0),
                position: center,
                button: fret_core::MouseButton::Left,
                modifiers: fret_core::Modifiers::default(),
                click_count: 1,
                pointer_type: fret_core::PointerType::Mouse,
            }),
        );
        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::Pointer(fret_core::PointerEvent::Up {
                pointer_id: fret_core::PointerId(0),
                position: center,
                button: fret_core::MouseButton::Left,
                modifiers: fret_core::Modifiers::default(),
                pointer_type: fret_core::PointerType::Mouse,
                click_count: 1,
                is_click: true,
            }),
        );

        let open_now = app
            .models()
            .get_copied(&open_model)
            .expect("sidebar open model");
        assert!(
            !open_now,
            "expected sidebar trigger to toggle open model to false"
        );
    }
}
