use std::collections::HashSet;
use std::sync::Arc;

use fret_core::{
    Color, CursorIcon, Edges, FontId, FontWeight, KeyCode, Modifiers, Px, SemanticsRole, TextStyle,
};
use fret_icons::IconId;
use fret_runtime::keymap::Binding;
use fret_runtime::{
    CommandId, Effect, KeyChord, Keymap, KeymapService, Model, ModelStore, PlatformFilter,
};
use fret_ui::action::OnActivate;
use fret_ui::element::{
    AnyElement, CrossAlign, Elements, FlexProps, HoverRegionProps, MainAlign, OpacityProps,
    Overflow, PressableProps, RingStyle, SemanticsDecoration, SpacerProps,
};
use fret_ui::{CommandAvailability, ElementContext, Invalidation, Theme, UiHost};
use fret_ui_kit::command::ElementCommandGatingExt as _;
use fret_ui_kit::declarative::action_hooks::ActionHooksExt as _;
use fret_ui_kit::declarative::icon as decl_icon;
use fret_ui_kit::declarative::model_watch::ModelWatchExt as _;
use fret_ui_kit::declarative::scheduling;
use fret_ui_kit::declarative::scroll as decl_scroll;
use fret_ui_kit::declarative::style as decl_style;
use fret_ui_kit::declarative::{
    ViewportQueryHysteresis, viewport_tailwind, viewport_width_at_least,
};
use fret_ui_kit::primitives::controllable_state;
use fret_ui_kit::primitives::transition as transition_prim;
use fret_ui_kit::{ChromeRefinement, ColorRef, LayoutRefinement, MetricRef, Radius, Space, ui};

use crate::SeparatorOrientation;
use crate::input::InputStyle as ShadcnInputStyle;
use crate::layout as shadcn_layout;
use crate::overlay_motion;
use crate::tooltip::{Tooltip, TooltipAlign, TooltipContent, TooltipProvider, TooltipSide};
use crate::{Button, ButtonSize, ButtonVariant, Input, Sheet, SheetContent, SheetSide, Skeleton};

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

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum SidebarMenuButtonVariant {
    #[default]
    Default,
    Outline,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum SidebarMenuSubButtonSize {
    Sm,
    #[default]
    Md,
}

fn sidebar_menu_button_h(theme: &Theme, size: SidebarMenuButtonSize) -> Px {
    let (key, fallback) = match size {
        SidebarMenuButtonSize::Sm => ("component.sidebar.menu_button.h_sm", Px(28.0)), // `h-7`
        SidebarMenuButtonSize::Default => ("component.sidebar.menu_button.h", Px(32.0)), // `h-8`
        SidebarMenuButtonSize::Lg => ("component.sidebar.menu_button.h_lg", Px(48.0)), // `h-12`
    };
    theme.metric_by_key(key).unwrap_or(fallback)
}

fn sidebar_menu_sub_button_h(theme: &Theme) -> Px {
    theme
        .metric_by_key("component.sidebar.menu_sub_button.h")
        .unwrap_or(Px(28.0)) // `h-7`
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

fn sidebar_width_mobile(theme: &Theme) -> Px {
    theme
        .metric_by_key("component.sidebar.width_mobile")
        .unwrap_or(Px(288.0))
}

const SIDEBAR_TOGGLE_SHORTCUT_KEY: KeyCode = KeyCode::KeyB;
const SIDEBAR_TOGGLE_COMMAND_ID: &str = "sidebar.toggle";

const SIDEBAR_COLLAPSE_OPEN_TICKS: u64 = overlay_motion::SHADCN_MOTION_TICKS_200;
const SIDEBAR_COLLAPSE_CLOSE_TICKS: u64 = overlay_motion::SHADCN_MOTION_TICKS_200;

type OnOpenChange = Arc<dyn Fn(bool) + Send + Sync + 'static>;

#[derive(Default)]
struct SidebarProviderOpenChangeCallbackState {
    initialized: bool,
    last_open: bool,
    last_open_mobile: bool,
}

fn sidebar_provider_open_change_events(
    state: &mut SidebarProviderOpenChangeCallbackState,
    open: bool,
    open_mobile: bool,
) -> (Option<bool>, Option<bool>) {
    if !state.initialized {
        state.initialized = true;
        state.last_open = open;
        state.last_open_mobile = open_mobile;
        return (None, None);
    }

    let open_changed = if state.last_open != open {
        state.last_open = open;
        Some(open)
    } else {
        None
    };
    let open_mobile_changed = if state.last_open_mobile != open_mobile {
        state.last_open_mobile = open_mobile;
        Some(open_mobile)
    } else {
        None
    };

    (open_changed, open_mobile_changed)
}

fn sidebar_open_url_on_activate(
    url: Arc<str>,
    target: Option<Arc<str>>,
    rel: Option<Arc<str>>,
) -> OnActivate {
    Arc::new(move |host, _acx, _reason| {
        host.push_effect(Effect::OpenUrl {
            url: url.to_string(),
            target: target.as_ref().map(|v| v.to_string()),
            rel: rel.as_ref().map(|v| v.to_string()),
        });
    })
}

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

fn menu_sub_button_style(theme: &Theme, size: SidebarMenuSubButtonSize) -> TextStyle {
    let (size_key, size_fallback, line_key, line_fallback) = match size {
        SidebarMenuSubButtonSize::Sm => (
            "component.sidebar.menu_sub_button.text_px_sm",
            Px(12.0),
            "component.sidebar.menu_sub_button.line_height_sm",
            Px(16.0),
        ),
        SidebarMenuSubButtonSize::Md => (
            "component.sidebar.menu_sub_button.text_px",
            Px(14.0),
            "component.sidebar.menu_sub_button.line_height",
            Px(20.0),
        ),
    };

    let text_px = theme.metric_by_key(size_key).unwrap_or(size_fallback);
    let line_height = theme.metric_by_key(line_key).unwrap_or(line_fallback);

    TextStyle {
        font: FontId::default(),
        size: text_px,
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

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum SidebarSide {
    #[default]
    Left,
    Right,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum SidebarCollapsible {
    #[default]
    Offcanvas,
    Icon,
    None,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum SidebarVariant {
    #[default]
    Sidebar,
    Floating,
    Inset,
}

#[derive(Debug, Clone, Copy, Default)]
struct SidebarSurfaceContext {
    side: SidebarSide,
    collapsible: SidebarCollapsible,
    variant: SidebarVariant,
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

    pub fn set_open_with<H: UiHost>(&self, host: &mut H, update: impl Fn(bool) -> bool) {
        let _ = host.models_mut().update(&self.open, |v| {
            *v = update(*v);
        });
    }

    pub fn set_open<H: UiHost>(&self, host: &mut H, open: bool) {
        self.set_open_with(host, |_| open);
    }

    pub fn set_open_mobile_with<H: UiHost>(&self, host: &mut H, update: impl Fn(bool) -> bool) {
        let _ = host.models_mut().update(&self.open_mobile, |v| {
            *v = update(*v);
        });
    }

    pub fn set_open_mobile<H: UiHost>(&self, host: &mut H, open_mobile: bool) {
        self.set_open_mobile_with(host, |_| open_mobile);
    }

    pub fn toggle_sidebar<H: UiHost>(&self, cx: &mut ElementContext<'_, H>) {
        if self.is_mobile {
            let _ = cx.app.models_mut().update(&self.open_mobile, |v| *v = !*v);
            return;
        }
        let _ = cx.app.models_mut().update(&self.open, |v| *v = !*v);
    }
}

#[derive(Debug, Default, Clone)]
struct SidebarProviderState {
    context: Option<SidebarContext>,
    surface_context: Option<SidebarSurfaceContext>,
}

#[derive(Debug, Default, Clone)]
struct SidebarSurfaceProviderState {
    context: Option<SidebarSurfaceContext>,
}

pub fn use_sidebar<H: UiHost>(cx: &ElementContext<'_, H>) -> Option<SidebarContext> {
    cx.inherited_state_where::<SidebarProviderState>(|st| st.context.is_some())
        .and_then(|st| st.context.clone())
}

fn use_sidebar_surface<H: UiHost>(cx: &ElementContext<'_, H>) -> Option<SidebarSurfaceContext> {
    cx.inherited_state_where::<SidebarSurfaceProviderState>(|st| st.context.is_some())
        .and_then(|st| st.context)
        .or_else(|| {
            cx.inherited_state_where::<SidebarProviderState>(|st| st.surface_context.is_some())
                .and_then(|st| st.surface_context)
        })
}

#[track_caller]
fn publish_sidebar_surface_context<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    context: SidebarSurfaceContext,
) {
    cx.with_state(SidebarProviderState::default, |st| {
        st.surface_context = Some(context);
    });
}

#[track_caller]
fn with_sidebar_provider_state<H: UiHost, R>(
    cx: &mut ElementContext<'_, H>,
    context: SidebarContext,
    f: impl FnOnce(&mut ElementContext<'_, H>) -> R,
) -> R {
    let prev = cx.with_state(SidebarProviderState::default, |st| {
        let prev = (st.context.clone(), st.surface_context);
        st.context = Some(context);
        prev
    });
    let out = f(cx);
    cx.with_state(SidebarProviderState::default, |st| {
        st.context = prev.0;
        st.surface_context = prev.1;
    });
    out
}

#[track_caller]
fn with_sidebar_surface_state<H: UiHost, R>(
    cx: &mut ElementContext<'_, H>,
    context: SidebarSurfaceContext,
    f: impl FnOnce(&mut ElementContext<'_, H>) -> R,
) -> R {
    let prev = cx.with_state(SidebarSurfaceProviderState::default, |st| {
        let prev = st.context;
        st.context = Some(context);
        prev
    });
    let out = f(cx);
    cx.with_state(SidebarSurfaceProviderState::default, |st| {
        st.context = prev;
    });
    out
}

fn sidebar_collapsed_in_scope<H: UiHost>(cx: &ElementContext<'_, H>) -> bool {
    use_sidebar(cx)
        .map(|ctx| !ctx.is_mobile && ctx.collapsed())
        .unwrap_or(false)
}

fn sidebar_sheet_side(side: SidebarSide) -> SheetSide {
    match side {
        SidebarSide::Left => SheetSide::Left,
        SidebarSide::Right => SheetSide::Right,
    }
}

fn sidebar_toggle_command_id() -> CommandId {
    CommandId::from(SIDEBAR_TOGGLE_COMMAND_ID)
}

fn sidebar_toggle_shortcut_bindings() -> [Binding; 2] {
    [
        Binding {
            platform: PlatformFilter::Macos,
            sequence: vec![KeyChord::new(
                SIDEBAR_TOGGLE_SHORTCUT_KEY,
                Modifiers {
                    meta: true,
                    ..Modifiers::default()
                },
            )],
            when: None,
            command: Some(sidebar_toggle_command_id()),
        },
        Binding {
            platform: PlatformFilter::All,
            sequence: vec![KeyChord::new(
                SIDEBAR_TOGGLE_SHORTCUT_KEY,
                Modifiers {
                    ctrl: true,
                    ..Modifiers::default()
                },
            )],
            when: None,
            command: Some(sidebar_toggle_command_id()),
        },
    ]
}

#[derive(Debug, Default, Clone)]
struct SidebarShortcutInstallGlobal {
    installed_windows: HashSet<fret_core::AppWindowId>,
}

fn ensure_sidebar_shortcut_binding<H: UiHost>(cx: &mut ElementContext<'_, H>) {
    let window = cx.window;
    let needs_install =
        cx.app
            .with_global_mut(SidebarShortcutInstallGlobal::default, |st, _app| {
                if st.installed_windows.contains(&window) {
                    false
                } else {
                    st.installed_windows.insert(window);
                    true
                }
            });

    if !needs_install {
        return;
    }

    cx.app.with_global_mut(KeymapService::default, |svc, _app| {
        let mut patch = Keymap::empty();
        for binding in sidebar_toggle_shortcut_bindings() {
            patch.push_binding(binding);
        }
        svc.keymap.extend(patch);
    });
}

fn sidebar_toggle_model(
    models: &mut ModelStore,
    open: &Model<bool>,
    open_mobile: &Model<bool>,
    is_mobile: bool,
) {
    let target = if is_mobile { open_mobile } else { open };
    let _ = models.update(target, |v| {
        *v = !*v;
    });
}

/// shadcn/ui `SidebarProvider` (V1).
///
/// Provides shared sidebar open/collapsed state and wraps descendants in `TooltipProvider`
/// with upstream-aligned default delay (`0`).
#[derive(Clone)]
pub struct SidebarProvider {
    open: Option<Model<bool>>,
    default_open: bool,
    open_mobile: Option<Model<bool>>,
    default_open_mobile: bool,
    is_mobile_override: Option<bool>,
    is_mobile_breakpoint: Px,
    on_open_change: Option<OnOpenChange>,
    on_open_mobile_change: Option<OnOpenChange>,
}

impl std::fmt::Debug for SidebarProvider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SidebarProvider")
            .field("open", &self.open)
            .field("default_open", &self.default_open)
            .field("open_mobile", &self.open_mobile)
            .field("default_open_mobile", &self.default_open_mobile)
            .field("is_mobile_override", &self.is_mobile_override)
            .field("is_mobile_breakpoint", &self.is_mobile_breakpoint)
            .field("on_open_change", &self.on_open_change.is_some())
            .field(
                "on_open_mobile_change",
                &self.on_open_mobile_change.is_some(),
            )
            .finish()
    }
}

impl SidebarProvider {
    pub fn new() -> Self {
        Self {
            open: None,
            default_open: true,
            open_mobile: None,
            default_open_mobile: false,
            is_mobile_override: None,
            is_mobile_breakpoint: viewport_tailwind::MD,
            on_open_change: None,
            on_open_mobile_change: None,
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

    /// Overrides whether the sidebar should use mobile/offcanvas behavior.
    ///
    /// When unset, `SidebarProvider` infers mobile mode from the committed per-window environment
    /// snapshot (ADR 1171) using a Tailwind-aligned viewport breakpoint.
    pub fn is_mobile(mut self, is_mobile: bool) -> Self {
        self.is_mobile_override = Some(is_mobile);
        self
    }

    /// Overrides the viewport breakpoint used to infer mobile mode when `is_mobile` is not set.
    ///
    /// This is intentionally viewport-driven (device shell), not container-query-driven.
    pub fn is_mobile_breakpoint(mut self, breakpoint: Px) -> Self {
        self.is_mobile_breakpoint = breakpoint;
        self
    }

    pub fn on_open_change(mut self, on_open_change: Option<OnOpenChange>) -> Self {
        self.on_open_change = on_open_change;
        self
    }

    pub fn on_open_mobile_change(mut self, on_open_mobile_change: Option<OnOpenChange>) -> Self {
        self.on_open_mobile_change = on_open_mobile_change;
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
        ensure_sidebar_shortcut_binding(cx);

        let open =
            controllable_state::use_controllable_model(cx, self.open, || self.default_open).model();
        let open_mobile = controllable_state::use_controllable_model(cx, self.open_mobile, || {
            self.default_open_mobile
        })
        .model();

        let open_now = cx.watch_model(&open).layout().copied().unwrap_or(true);
        let open_mobile_now = cx
            .watch_model(&open_mobile)
            .layout()
            .copied()
            .unwrap_or(false);

        let (open_changed, open_mobile_changed) = cx
            .with_state(SidebarProviderOpenChangeCallbackState::default, |state| {
                sidebar_provider_open_change_events(state, open_now, open_mobile_now)
            });
        if let (Some(open), Some(handler)) = (open_changed, self.on_open_change.as_ref()) {
            handler(open);
        }
        if let (Some(open_mobile), Some(handler)) =
            (open_mobile_changed, self.on_open_mobile_change.as_ref())
        {
            handler(open_mobile);
        }

        let state = if open_now {
            SidebarState::Expanded
        } else {
            SidebarState::Collapsed
        };

        let is_mobile = self.is_mobile_override.unwrap_or_else(|| {
            !viewport_width_at_least(
                cx,
                Invalidation::Layout,
                self.is_mobile_breakpoint,
                ViewportQueryHysteresis::default(),
            )
        });
        let context = SidebarContext {
            state,
            open: open.clone(),
            open_mobile: open_mobile.clone(),
            is_mobile,
        };

        let toggle_command = sidebar_toggle_command_id();
        let open_for_command = open.clone();
        let open_mobile_for_command = open_mobile.clone();
        let is_mobile_for_command = is_mobile;

        with_sidebar_provider_state(cx, context, |cx| {
            let root = cx.root_id();
            cx.command_on_command_for(
                root,
                Arc::new(move |host, _acx, command| {
                    if command.as_str() != SIDEBAR_TOGGLE_COMMAND_ID {
                        return false;
                    }
                    sidebar_toggle_model(
                        host.models_mut(),
                        &open_for_command,
                        &open_mobile_for_command,
                        is_mobile_for_command,
                    );
                    true
                }),
            );
            cx.command_on_command_availability_for(
                root,
                Arc::new(move |_host, acx, command| {
                    if command != toggle_command {
                        return CommandAvailability::NotHandled;
                    }
                    if !acx.focus_in_subtree {
                        return CommandAvailability::NotHandled;
                    }
                    CommandAvailability::Available
                }),
            );

            let children = TooltipProvider::new()
                .delay_duration_frames(0)
                .with_elements(cx, f)
                .into_vec();

            Elements::new(children)
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
    side: SidebarSide,
    collapsible: SidebarCollapsible,
    variant: SidebarVariant,
    chrome: ChromeRefinement,
    layout: LayoutRefinement,
}

impl Sidebar {
    pub fn new(children: impl IntoIterator<Item = AnyElement>) -> Self {
        let children = children.into_iter().collect();
        Self {
            children,
            collapsed: false,
            side: SidebarSide::Left,
            collapsible: SidebarCollapsible::Offcanvas,
            variant: SidebarVariant::Sidebar,
            chrome: ChromeRefinement::default(),
            layout: LayoutRefinement::default(),
        }
    }

    pub fn collapsed(mut self, collapsed: bool) -> Self {
        self.collapsed = collapsed;
        self
    }

    pub fn side(mut self, side: SidebarSide) -> Self {
        self.side = side;
        self
    }

    pub fn collapsible(mut self, collapsible: SidebarCollapsible) -> Self {
        self.collapsible = collapsible;
        self
    }

    pub fn variant(mut self, variant: SidebarVariant) -> Self {
        self.variant = variant;
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

    pub fn into_element_with_children<H: UiHost, I>(
        self,
        cx: &mut ElementContext<'_, H>,
        render_children: impl FnOnce(&mut ElementContext<'_, H>) -> I,
    ) -> AnyElement
    where
        I: IntoIterator<Item = AnyElement>,
    {
        let sidebar_ctx = use_sidebar(cx);
        let is_mobile = sidebar_ctx.as_ref().is_some_and(|ctx| ctx.is_mobile);

        let Self {
            children: _,
            collapsed: collapsed_override,
            side,
            collapsible,
            variant,
            chrome,
            layout,
        } = self;

        let surface_context = SidebarSurfaceContext {
            side,
            collapsible,
            variant,
        };

        publish_sidebar_surface_context(cx, surface_context);

        if is_mobile && let Some(sidebar_ctx) = sidebar_ctx {
            let sheet_side = sidebar_sheet_side(side);
            let (surface_props, sheet_size, sheet_bg, sheet_border) = {
                let theme = Theme::global(&*cx.app);
                let mut surface_props = decl_style::container_props(
                    theme,
                    ChromeRefinement::default()
                        .bg(ColorRef::Color(sidebar_bg(theme)))
                        .border_1()
                        .border_color(ColorRef::Color(sidebar_border(theme)))
                        .merge(chrome),
                    LayoutRefinement::default().w_full().h_full().merge(layout),
                );
                surface_props.layout.overflow = Overflow::Clip;

                let sheet_size = sidebar_width_mobile(theme);
                let sheet_bg = sidebar_bg(theme);
                let sheet_border = sidebar_border(theme);
                (surface_props, sheet_size, sheet_bg, sheet_border)
            };
            return Sheet::new(sidebar_ctx.open_mobile)
                .side(sheet_side)
                .size(sheet_size)
                .into_element(
                    cx,
                    |cx| {
                        cx.spacer(SpacerProps {
                            min: Px(0.0),
                            ..Default::default()
                        })
                    },
                    move |cx| {
                        let children = with_sidebar_surface_state(cx, surface_context, |cx| {
                            render_children(cx).into_iter().collect::<Vec<_>>()
                        });
                        let surface = shadcn_layout::container_flow(cx, surface_props, children);

                        SheetContent::new([surface])
                            .refine_style(
                                ChromeRefinement::default()
                                    .bg(ColorRef::Color(sheet_bg))
                                    .border_color(ColorRef::Color(sheet_border))
                                    .p(Space::N0),
                            )
                            .refine_layout(
                                LayoutRefinement::default()
                                    .w_full()
                                    .h_full()
                                    .overflow_hidden(),
                            )
                            .into_element(cx)
                    },
                );
        }

        let collapsed = sidebar_collapsed_in_scope(cx);
        let collapsed = if collapsed_override { true } else { collapsed };

        let motion = sidebar_collapse_motion(cx, collapsed);
        let expanded_progress = motion.progress;
        let props = {
            let theme = Theme::global(&*cx.app);
            let inner_w = transition_prim::lerp_px(
                sidebar_width_icon(theme),
                sidebar_width(theme),
                expanded_progress,
            );

            let variant_uses_outer_gap =
                matches!(variant, SidebarVariant::Floating | SidebarVariant::Inset);
            let outer_gap = if variant_uses_outer_gap {
                decl_style::space(theme, Space::N2)
            } else {
                Px(0.0)
            };
            let outer_border = if variant_uses_outer_gap {
                Px(1.0)
            } else {
                Px(0.0)
            };
            let total_w = Px(inner_w.0 + outer_gap.0 * 2.0 + outer_border.0 * 2.0);
            let layout = LayoutRefinement::default()
                .w_px(total_w)
                .h_full()
                .merge(layout);

            let mut chrome = ChromeRefinement::default()
                .bg(ColorRef::Color(sidebar_bg(theme)))
                .merge(chrome);

            if variant_uses_outer_gap {
                chrome = chrome.px(Space::N2).py(Space::N2);
            } else {
                chrome = chrome
                    .border_1()
                    .border_color(ColorRef::Color(sidebar_border(theme)));
            }

            if matches!(variant, SidebarVariant::Floating) {
                chrome = chrome
                    .border_1()
                    .border_color(ColorRef::Color(sidebar_border(theme)))
                    .rounded(Radius::Lg)
                    .shadow_sm();
            }

            let mut props = decl_style::container_props(theme, chrome, layout);
            props.layout.overflow = Overflow::Clip;
            props
        };

        let children = with_sidebar_surface_state(cx, surface_context, |cx| {
            render_children(cx).into_iter().collect::<Vec<_>>()
        });
        shadcn_layout::container_flow(cx, props, children)
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let sidebar_ctx = use_sidebar(cx);
        let is_mobile = sidebar_ctx.as_ref().is_some_and(|ctx| ctx.is_mobile);

        let Self {
            children,
            collapsed: collapsed_override,
            side,
            collapsible,
            variant,
            chrome,
            layout,
        } = self;

        let surface_context = SidebarSurfaceContext {
            side,
            collapsible,
            variant,
        };

        publish_sidebar_surface_context(cx, surface_context);

        if is_mobile && let Some(sidebar_ctx) = sidebar_ctx {
            let sheet_side = sidebar_sheet_side(side);
            let (surface_props, sheet_size, sheet_bg, sheet_border) = {
                let theme = Theme::global(&*cx.app);
                let mut surface_props = decl_style::container_props(
                    theme,
                    ChromeRefinement::default()
                        .bg(ColorRef::Color(sidebar_bg(theme)))
                        .border_1()
                        .border_color(ColorRef::Color(sidebar_border(theme)))
                        .merge(chrome),
                    LayoutRefinement::default().w_full().h_full().merge(layout),
                );
                surface_props.layout.overflow = Overflow::Clip;

                let sheet_size = sidebar_width_mobile(theme);
                let sheet_bg = sidebar_bg(theme);
                let sheet_border = sidebar_border(theme);
                (surface_props, sheet_size, sheet_bg, sheet_border)
            };
            return Sheet::new(sidebar_ctx.open_mobile)
                .side(sheet_side)
                .size(sheet_size)
                .into_element(
                    cx,
                    |cx| {
                        cx.spacer(SpacerProps {
                            min: Px(0.0),
                            ..Default::default()
                        })
                    },
                    move |cx| {
                        let surface = with_sidebar_surface_state(cx, surface_context, |cx| {
                            shadcn_layout::container_flow(cx, surface_props, children)
                        });

                        SheetContent::new([surface])
                            .refine_style(
                                ChromeRefinement::default()
                                    .bg(ColorRef::Color(sheet_bg))
                                    .border_color(ColorRef::Color(sheet_border))
                                    .p(Space::N0),
                            )
                            .refine_layout(
                                LayoutRefinement::default()
                                    .w_full()
                                    .h_full()
                                    .overflow_hidden(),
                            )
                            .into_element(cx)
                    },
                );
        }

        let collapsed = sidebar_collapsed_in_scope(cx);
        let collapsed = if collapsed_override { true } else { collapsed };

        let motion = sidebar_collapse_motion(cx, collapsed);
        let expanded_progress = motion.progress;
        let props = {
            let theme = Theme::global(&*cx.app);
            let inner_w = transition_prim::lerp_px(
                sidebar_width_icon(theme),
                sidebar_width(theme),
                expanded_progress,
            );

            let variant_uses_outer_gap =
                matches!(variant, SidebarVariant::Floating | SidebarVariant::Inset);
            let outer_gap = if variant_uses_outer_gap {
                decl_style::space(theme, Space::N2)
            } else {
                Px(0.0)
            };
            let outer_border = if variant_uses_outer_gap {
                Px(1.0)
            } else {
                Px(0.0)
            };
            let total_w = Px(inner_w.0 + outer_gap.0 * 2.0 + outer_border.0 * 2.0);
            let layout = LayoutRefinement::default()
                .w_px(total_w)
                .h_full()
                .merge(layout);

            let mut chrome = ChromeRefinement::default()
                .bg(ColorRef::Color(sidebar_bg(theme)))
                .merge(chrome);

            if variant_uses_outer_gap {
                chrome = chrome.px(Space::N2).py(Space::N2);
            } else {
                chrome = chrome
                    .border_1()
                    .border_color(ColorRef::Color(sidebar_border(theme)));
            }

            if matches!(variant, SidebarVariant::Floating) {
                chrome = chrome
                    .border_1()
                    .border_color(ColorRef::Color(sidebar_border(theme)))
                    .rounded(Radius::Lg)
                    .shadow_sm();
            }

            let mut props = decl_style::container_props(theme, chrome, layout);
            props.layout.overflow = Overflow::Clip;
            props
        };

        with_sidebar_surface_state(cx, surface_context, |cx| {
            shadcn_layout::container_flow(cx, props, children)
        })
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
                        sidebar_toggle_model(
                            host.models_mut(),
                            &ctx.open,
                            &ctx.open_mobile,
                            ctx.is_mobile,
                        );
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

#[derive(Clone)]
pub struct SidebarRail {
    on_click: Option<CommandId>,
    on_activate: Option<OnActivate>,
    disabled: bool,
    test_id: Option<Arc<str>>,
    chrome: ChromeRefinement,
    layout: LayoutRefinement,
}

impl std::fmt::Debug for SidebarRail {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SidebarRail")
            .field("on_click", &self.on_click)
            .field("on_activate", &self.on_activate.is_some())
            .field("disabled", &self.disabled)
            .field("test_id", &self.test_id)
            .field("chrome", &self.chrome)
            .field("layout", &self.layout)
            .finish()
    }
}

impl SidebarRail {
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
        let surface_ctx = use_sidebar_surface(cx);
        let side = surface_ctx.map(|ctx| ctx.side).unwrap_or_default();
        let collapsible = surface_ctx.map(|ctx| ctx.collapsible).unwrap_or_default();
        let variant = surface_ctx.map(|ctx| ctx.variant).unwrap_or_default();

        let rail_layout = {
            let mut layout = LayoutRefinement::default()
                .absolute()
                .top_px(Px(0.0))
                .bottom_px(Px(0.0))
                .w_px(Px(16.0));

            layout = match side {
                SidebarSide::Left => layout.right_neg_px(Px(8.0)),
                SidebarSide::Right => layout.left_neg_px(Px(8.0)),
            };

            if matches!(collapsible, SidebarCollapsible::Offcanvas) {
                layout = match side {
                    SidebarSide::Left => layout.right_neg_px(Px(4.0)),
                    SidebarSide::Right => layout.left_neg_px(Px(4.0)),
                };
            }

            if matches!(variant, SidebarVariant::Floating | SidebarVariant::Inset) {
                layout = match side {
                    SidebarSide::Left => layout.right_neg_px(Px(2.0)),
                    SidebarSide::Right => layout.left_neg_px(Px(2.0)),
                };
            }

            layout
        };

        let command = self.on_click;
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
                        sidebar_toggle_model(
                            host.models_mut(),
                            &ctx.open,
                            &ctx.open_mobile,
                            ctx.is_mobile,
                        );
                    }
                }))
            };

        let disabled = self.disabled
            || command
                .as_ref()
                .is_some_and(|cmd| !cx.command_is_enabled(cmd));

        let mut rail = Button::new("Toggle Sidebar")
            .variant(ButtonVariant::Ghost)
            .size(ButtonSize::IconSm)
            .on_hover_change(Arc::new(move |host, acx, hovered| {
                if hovered {
                    host.push_effect(Effect::CursorSetIcon {
                        window: acx.window,
                        icon: CursorIcon::ColResize,
                    });
                }
            }))
            .disabled(disabled)
            .refine_style(
                ChromeRefinement::default()
                    .p(Space::N0)
                    .rounded(Radius::Md)
                    .merge(self.chrome),
            )
            .refine_layout(rail_layout.merge(self.layout));

        if let Some(command) = command {
            rail = rail.on_click(command);
        }
        if let Some(on_activate) = toggle_on_activate {
            rail = rail.on_activate(on_activate);
        }
        if let Some(test_id) = self.test_id {
            rail = rail.test_id(test_id);
        }

        rail.into_element(cx)
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
        let sidebar_ctx = use_sidebar(cx);
        let surface_ctx = use_sidebar_surface(cx);
        let inset_variant = surface_ctx
            .map(|ctx| ctx.variant)
            .is_some_and(|variant| variant == SidebarVariant::Inset);
        let collapsed = sidebar_ctx
            .as_ref()
            .is_some_and(|ctx| !ctx.is_mobile && ctx.collapsed());

        let background = Theme::global(&*cx.app).color_required("background");
        let mut chrome = ChromeRefinement::default().bg(ColorRef::Color(background));
        let mut layout = LayoutRefinement::default().w_full().h_full().flex_1();

        if inset_variant {
            layout =
                layout
                    .m(Space::N2)
                    .mt_px(Px(0.0))
                    .ml_px(if collapsed { Px(8.0) } else { Px(0.0) });
            chrome = chrome.rounded(Radius::Lg).shadow_sm();
        }

        chrome = chrome.merge(self.chrome);
        layout = layout.merge(self.layout);
        let props = {
            let theme = Theme::global(&*cx.app);
            decl_style::container_props(theme, chrome, layout)
        };
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
        let background = Theme::global(&*cx.app).color_required("background");

        let mut input = Input::new(self.model)
            .disabled(self.disabled)
            .style(ShadcnInputStyle::default().background(ColorRef::Color(background)))
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
        let (thickness, mut layout, background) = {
            let theme = Theme::global(&*cx.app);
            let thickness = theme
                .metric_by_key("component.separator.px")
                .unwrap_or(Px(1.0));
            let margin_x = decl_style::space(theme, Space::N2);
            let layout = decl_style::layout_style(
                theme,
                LayoutRefinement::default()
                    .mx_px(margin_x)
                    .merge(self.layout),
            );
            let background = sidebar_border(theme);
            (thickness, layout, background)
        };

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
                background: Some(background),
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
        let props = {
            let theme = Theme::global(&*cx.app);
            decl_style::container_props(
                theme,
                ChromeRefinement::default().p(Space::N2),
                LayoutRefinement::default(),
            )
        };
        let children = self.children;
        shadcn_layout::container_vstack_gap(cx, props, Space::N2, children)
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
        let props = {
            let theme = Theme::global(&*cx.app);
            decl_style::container_props(
                theme,
                ChromeRefinement::default().p(Space::N2),
                LayoutRefinement::default(),
            )
        };
        let children = self.children;
        shadcn_layout::container_vstack_gap(cx, props, Space::N2, children)
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
        let gap = {
            let theme = Theme::global(&*cx.app);
            decl_style::space(theme, Space::N2)
        };

        let mut layout = LayoutRefinement::default().min_h_0().flex_1().w_full();
        if collapsed {
            layout = layout.overflow_hidden();
        }

        let children = self.children;
        decl_scroll::overflow_scrollbar(cx, layout, move |cx| {
            let col = FlexProps {
                direction: fret_core::Axis::Vertical,
                gap,
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
        let chrome = ChromeRefinement::default().p(Space::N2);
        let props = {
            let theme = Theme::global(&*cx.app);
            decl_style::container_props(
                theme,
                chrome,
                LayoutRefinement::default().w_full().min_w_0().relative(),
            )
        };
        let children = self.children;
        shadcn_layout::container_vstack(
            cx,
            props,
            fret_ui_kit::declarative::stack::VStackProps::default()
                .gap(Space::N0)
                .layout(LayoutRefinement::default().w_full()),
            children,
        )
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
        let motion = sidebar_collapse_motion(cx, collapsed);
        if !motion.present {
            return cx.spacer(fret_ui::element::SpacerProps {
                min: Px(0.0),
                ..Default::default()
            });
        }

        let (fg, size, line_height) = {
            let theme = Theme::global(&*cx.app);
            let mut fg = sidebar_fg(theme);
            fg.a = (fg.a * 0.7).clamp(0.0, 1.0);
            let size = theme
                .metric_by_key("component.sidebar.group_label_px")
                .unwrap_or(Px(12.0));
            let line_height = theme
                .metric_by_key("component.sidebar.group_label_line_height")
                .unwrap_or(Px(16.0));
            (fg, size, line_height)
        };

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

#[derive(Clone)]
pub struct SidebarGroupAction {
    label: Arc<str>,
    children: Vec<AnyElement>,
    as_child: bool,
    disabled: bool,
    collapsed: bool,
    on_click: Option<CommandId>,
    on_activate: Option<OnActivate>,
    test_id: Option<Arc<str>>,
    chrome: ChromeRefinement,
    layout: LayoutRefinement,
}

impl std::fmt::Debug for SidebarGroupAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SidebarGroupAction")
            .field("label", &self.label)
            .field("children_len", &self.children.len())
            .field("as_child", &self.as_child)
            .field("disabled", &self.disabled)
            .field("collapsed", &self.collapsed)
            .field("on_click", &self.on_click)
            .field("on_activate", &self.on_activate.is_some())
            .field("test_id", &self.test_id)
            .field("chrome", &self.chrome)
            .field("layout", &self.layout)
            .finish()
    }
}

impl SidebarGroupAction {
    pub fn new(children: impl IntoIterator<Item = AnyElement>) -> Self {
        Self {
            label: Arc::from("Sidebar Group Action"),
            children: children.into_iter().collect(),
            as_child: false,
            disabled: false,
            collapsed: false,
            on_click: None,
            on_activate: None,
            test_id: None,
            chrome: ChromeRefinement::default(),
            layout: LayoutRefinement::default(),
        }
    }

    pub fn a11y_label(mut self, label: impl Into<Arc<str>>) -> Self {
        self.label = label.into();
        self
    }

    pub fn children(mut self, children: impl IntoIterator<Item = AnyElement>) -> Self {
        self.children = children.into_iter().collect();
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn as_child(mut self, as_child: bool) -> Self {
        self.as_child = as_child;
        self
    }

    pub fn collapsed(mut self, collapsed: bool) -> Self {
        self.collapsed = collapsed;
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
        if collapsed {
            return cx.spacer(SpacerProps {
                min: Px(0.0),
                ..Default::default()
            });
        }

        let is_mobile = use_sidebar(cx).is_some_and(|ctx| ctx.is_mobile);
        let (top, right, size, hit_expand) = {
            let theme = Theme::global(&*cx.app);
            let top = theme
                .metric_by_key("component.sidebar.group_action.top")
                .unwrap_or(Px(14.0));
            let right = theme
                .metric_by_key("component.sidebar.group_action.right")
                .unwrap_or(Px(12.0));
            let size = theme
                .metric_by_key("component.sidebar.group_action.size")
                .unwrap_or(Px(20.0));

            let hit_expand = if is_mobile {
                theme
                    .metric_by_key("component.sidebar.group_action.mobile_hit_expand")
                    .unwrap_or(Px(8.0))
            } else {
                Px(0.0)
            };
            (top, right, size, hit_expand)
        };
        let hit_size = Px(size.0 + hit_expand.0 * 2.0);
        let hit_top = Px(top.0 - hit_expand.0);
        let hit_right = Px(right.0 - hit_expand.0);

        let action_layout = LayoutRefinement::default()
            .absolute()
            .top_px(hit_top)
            .right_px(hit_right)
            .w_px(hit_size)
            .h_px(hit_size)
            .merge(self.layout);

        let label = self.label;
        let as_child = self.as_child;
        let on_click = self.on_click;
        let on_activate = self.on_activate;
        let test_id = self.test_id;
        let user_chrome = self.chrome;
        let content_layout = if !as_child && hit_expand.0 > 0.0 {
            LayoutRefinement::default()
                .absolute()
                .top_px(hit_expand)
                .left_px(hit_expand)
                .w_px(size)
                .h_px(size)
        } else {
            LayoutRefinement::default().w_full().h_full()
        };
        let disabled = self.disabled
            || on_click
                .as_ref()
                .is_some_and(|cmd| !cx.command_is_enabled(cmd));
        let (ring, action_layout_style) = {
            let theme = Theme::global(&*cx.app);
            let radius = decl_style::radius(theme, Radius::Md);
            let ring = sidebar_ring(theme, radius);
            let action_layout_style = decl_style::layout_style(theme, action_layout);
            (ring, action_layout_style)
        };
        let pressable = PressableProps {
            enabled: !disabled,
            focus_ring: Some(ring),
            layout: action_layout_style,
            a11y: fret_ui::element::PressableA11y {
                role: Some(SemanticsRole::Button),
                label: Some(label),
                test_id,
                ..Default::default()
            },
            ..Default::default()
        };
        let children = self.children;
        cx.pressable(pressable, move |cx, st| {
            cx.pressable_dispatch_command_if_enabled_opt(on_click);
            if let Some(on_activate) = on_activate.clone() {
                cx.pressable_on_activate(on_activate);
            }

            let theme = Theme::global(&*cx.app);
            let fg = if disabled {
                alpha_mul(sidebar_fg(theme), 0.5)
            } else if st.hovered || st.pressed {
                sidebar_accent_fg(theme)
            } else {
                sidebar_fg(theme)
            };
            let bg = if st.hovered || st.pressed {
                sidebar_accent(theme)
            } else {
                Color::TRANSPARENT
            };
            let mut chrome = ChromeRefinement::default()
                .rounded(Radius::Md)
                .p(Space::N0)
                .text_color(ColorRef::Color(fg));
            if bg.a > 0.0 {
                chrome = chrome.bg(ColorRef::Color(bg));
            }
            let props = decl_style::container_props(
                theme,
                chrome.merge(user_chrome.clone()),
                content_layout.clone(),
            );
            let children = children.clone();
            vec![cx.container(props, move |_cx| children)]
        })
    }
}

#[derive(Debug, Clone)]
pub struct SidebarGroupContent {
    children: Vec<AnyElement>,
    chrome: ChromeRefinement,
    layout: LayoutRefinement,
}

impl SidebarGroupContent {
    pub fn new(children: impl IntoIterator<Item = AnyElement>) -> Self {
        Self {
            children: children.into_iter().collect(),
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
        let props = {
            let theme = Theme::global(&*cx.app);
            decl_style::container_props(
                theme,
                self.chrome,
                LayoutRefinement::default().w_full().merge(self.layout),
            )
        };
        let children = self.children;
        shadcn_layout::container_flow(cx, props, children)
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
                    min_width: Some(Px(0.0)),
                    ..Default::default()
                },
                ..Default::default()
            },
            ..Default::default()
        };
        let children = self.children;
        cx.flex(props, move |_cx| children)
            .attach_semantics(SemanticsDecoration::default().role(SemanticsRole::List))
    }
}

#[derive(Debug, Clone)]
pub struct SidebarMenuItem {
    children: Vec<AnyElement>,
    open: bool,
    test_id: Option<Arc<str>>,
    layout: LayoutRefinement,
}

#[derive(Debug, Clone, Copy, Default)]
struct SidebarMenuItemContext {
    hovered: bool,
    open: bool,
    focus_within: bool,
}

#[derive(Debug, Default, Clone)]
struct SidebarMenuItemProviderState {
    context: Option<SidebarMenuItemContext>,
}

fn use_sidebar_menu_item_context<H: UiHost>(
    cx: &ElementContext<'_, H>,
) -> Option<SidebarMenuItemContext> {
    cx.inherited_state_where::<SidebarMenuItemProviderState>(|st| st.context.is_some())
        .and_then(|st| st.context)
}

#[track_caller]
fn with_sidebar_menu_item_state<H: UiHost, R>(
    cx: &mut ElementContext<'_, H>,
    context: SidebarMenuItemContext,
    f: impl FnOnce(&mut ElementContext<'_, H>) -> R,
) -> R {
    let prev = cx.with_state(SidebarMenuItemProviderState::default, |st| {
        let prev = st.context;
        st.context = Some(context);
        prev
    });
    let out = f(cx);
    cx.with_state(SidebarMenuItemProviderState::default, |st| {
        st.context = prev;
    });
    out
}

fn sidebar_menu_item_hovered_in_scope<H: UiHost>(cx: &ElementContext<'_, H>) -> Option<bool> {
    use_sidebar_menu_item_context(cx).map(|ctx| ctx.hovered)
}

fn sidebar_menu_item_open_in_scope<H: UiHost>(cx: &ElementContext<'_, H>) -> Option<bool> {
    use_sidebar_menu_item_context(cx).map(|ctx| ctx.open)
}

fn sidebar_menu_item_focus_within_in_scope<H: UiHost>(cx: &ElementContext<'_, H>) -> Option<bool> {
    use_sidebar_menu_item_context(cx).map(|ctx| ctx.focus_within)
}

fn any_element_subtree_has_focus<H: UiHost>(
    cx: &ElementContext<'_, H>,
    element: &AnyElement,
) -> bool {
    cx.is_focused_element(element.id)
        || element
            .children
            .iter()
            .any(|child| any_element_subtree_has_focus(cx, child))
}

impl SidebarMenuItem {
    pub fn new(child: AnyElement) -> Self {
        Self {
            children: vec![child],
            open: false,
            test_id: None,
            layout: LayoutRefinement::default(),
        }
    }

    pub fn children(mut self, children: impl IntoIterator<Item = AnyElement>) -> Self {
        self.children = children.into_iter().collect();
        self
    }

    pub fn test_id(mut self, id: impl Into<Arc<str>>) -> Self {
        self.test_id = Some(id.into());
        self
    }

    pub fn open(mut self, open: bool) -> Self {
        self.open = open;
        self
    }

    pub fn refine_layout(mut self, layout: LayoutRefinement) -> Self {
        self.layout = self.layout.merge(layout);
        self
    }

    pub fn into_element_with_children<H: UiHost, F>(
        self,
        cx: &mut ElementContext<'_, H>,
        render_children: F,
    ) -> AnyElement
    where
        F: Fn(&mut ElementContext<'_, H>) -> Vec<AnyElement> + Clone,
    {
        let open = self.open;
        let props = {
            let theme = Theme::global(&*cx.app);
            decl_style::container_props(
                theme,
                ChromeRefinement::default(),
                LayoutRefinement::default()
                    .relative()
                    .w_full()
                    .min_w_0()
                    .merge(self.layout),
            )
        };

        let mut semantics = SemanticsDecoration::default().role(SemanticsRole::ListItem);
        if let Some(test_id) = self.test_id {
            semantics = semantics.test_id(test_id);
        }

        cx.hover_region(HoverRegionProps::default(), move |cx, hovered| {
            let focus_probe = cx.with_callsite_counters_snapshot(|cx| render_children(cx));
            let focus_within = focus_probe
                .iter()
                .any(|child| any_element_subtree_has_focus(cx, child));

            with_sidebar_menu_item_state(
                cx,
                SidebarMenuItemContext {
                    hovered,
                    open,
                    focus_within,
                },
                |cx| {
                    let render_children = render_children.clone();
                    let children = render_children(cx);
                    let node = cx
                        .container(props, move |_cx| children)
                        .attach_semantics(semantics);
                    vec![node]
                },
            )
        })
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let open = self.open;
        let props = {
            let theme = Theme::global(&*cx.app);
            decl_style::container_props(
                theme,
                ChromeRefinement::default(),
                LayoutRefinement::default()
                    .relative()
                    .w_full()
                    .min_w_0()
                    .merge(self.layout),
            )
        };

        let mut semantics = SemanticsDecoration::default().role(SemanticsRole::ListItem);
        if let Some(test_id) = self.test_id {
            semantics = semantics.test_id(test_id);
        }

        let children = self.children;
        cx.hover_region(HoverRegionProps::default(), move |cx, hovered| {
            let focus_within = children
                .iter()
                .any(|child| any_element_subtree_has_focus(cx, child));
            with_sidebar_menu_item_state(
                cx,
                SidebarMenuItemContext {
                    hovered,
                    open,
                    focus_within,
                },
                |cx| {
                    let node = cx
                        .container(props, move |_cx| children)
                        .attach_semantics(semantics);
                    vec![node]
                },
            )
        })
    }
}

#[derive(Clone)]
pub struct SidebarMenuAction {
    label: Arc<str>,
    children: Vec<AnyElement>,
    size: SidebarMenuButtonSize,
    as_child: bool,
    show_on_hover: bool,
    disabled: bool,
    collapsed: bool,
    on_click: Option<CommandId>,
    on_activate: Option<OnActivate>,
    test_id: Option<Arc<str>>,
    chrome: ChromeRefinement,
    layout: LayoutRefinement,
}

impl std::fmt::Debug for SidebarMenuAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SidebarMenuAction")
            .field("label", &self.label)
            .field("children_len", &self.children.len())
            .field("size", &self.size)
            .field("as_child", &self.as_child)
            .field("show_on_hover", &self.show_on_hover)
            .field("disabled", &self.disabled)
            .field("collapsed", &self.collapsed)
            .field("on_click", &self.on_click)
            .field("on_activate", &self.on_activate.is_some())
            .field("test_id", &self.test_id)
            .field("chrome", &self.chrome)
            .field("layout", &self.layout)
            .finish()
    }
}

impl SidebarMenuAction {
    pub fn new(children: impl IntoIterator<Item = AnyElement>) -> Self {
        Self {
            label: Arc::from("Sidebar Menu Action"),
            children: children.into_iter().collect(),
            size: SidebarMenuButtonSize::Default,
            as_child: false,
            show_on_hover: false,
            disabled: false,
            collapsed: false,
            on_click: None,
            on_activate: None,
            test_id: None,
            chrome: ChromeRefinement::default(),
            layout: LayoutRefinement::default(),
        }
    }

    pub fn a11y_label(mut self, label: impl Into<Arc<str>>) -> Self {
        self.label = label.into();
        self
    }

    pub fn children(mut self, children: impl IntoIterator<Item = AnyElement>) -> Self {
        self.children = children.into_iter().collect();
        self
    }

    pub fn size(mut self, size: SidebarMenuButtonSize) -> Self {
        self.size = size;
        self
    }

    pub fn show_on_hover(mut self, show_on_hover: bool) -> Self {
        self.show_on_hover = show_on_hover;
        self
    }

    pub fn as_child(mut self, as_child: bool) -> Self {
        self.as_child = as_child;
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
        if collapsed {
            return cx.spacer(SpacerProps {
                min: Px(0.0),
                ..Default::default()
            });
        }

        if self.show_on_hover {
            let is_mobile = use_sidebar(cx).is_some_and(|ctx| ctx.is_mobile);
            let item_hovered = sidebar_menu_item_hovered_in_scope(cx).unwrap_or(true);
            let item_open = sidebar_menu_item_open_in_scope(cx).unwrap_or(false);
            let item_focus_within = sidebar_menu_item_focus_within_in_scope(cx).unwrap_or(false);
            if !is_mobile && !item_hovered && !item_open && !item_focus_within {
                return cx.spacer(SpacerProps {
                    min: Px(0.0),
                    ..Default::default()
                });
            }
        }

        let top = sidebar_menu_affordance_top(self.size);
        let is_mobile = use_sidebar(cx).is_some_and(|ctx| ctx.is_mobile);
        let (right, size, hit_expand) = {
            let theme = Theme::global(&*cx.app);
            let right = theme
                .metric_by_key("component.sidebar.menu_action.right")
                .unwrap_or(Px(4.0));
            let size = theme
                .metric_by_key("component.sidebar.menu_action.size")
                .unwrap_or(Px(20.0));
            let hit_expand = if is_mobile {
                theme
                    .metric_by_key("component.sidebar.menu_action.mobile_hit_expand")
                    .unwrap_or(Px(8.0))
            } else {
                Px(0.0)
            };
            (right, size, hit_expand)
        };
        let hit_size = Px(size.0 + hit_expand.0 * 2.0);
        let hit_top = Px(top.0 - hit_expand.0);
        let hit_right = Px(right.0 - hit_expand.0);

        let action_layout = LayoutRefinement::default()
            .absolute()
            .top_px(hit_top)
            .right_px(hit_right)
            .w_px(hit_size)
            .h_px(hit_size)
            .merge(self.layout);

        let label = self.label;
        let as_child = self.as_child;
        let on_click = self.on_click;
        let on_activate = self.on_activate;
        let test_id = self.test_id;
        let user_chrome = self.chrome;
        let content_layout = if !as_child && hit_expand.0 > 0.0 {
            LayoutRefinement::default()
                .absolute()
                .top_px(hit_expand)
                .left_px(hit_expand)
                .w_px(size)
                .h_px(size)
        } else {
            LayoutRefinement::default().w_full().h_full()
        };
        let disabled = self.disabled
            || on_click
                .as_ref()
                .is_some_and(|cmd| !cx.command_is_enabled(cmd));
        let (ring, action_layout_style) = {
            let theme = Theme::global(&*cx.app);
            let radius = decl_style::radius(theme, Radius::Md);
            let ring = sidebar_ring(theme, radius);
            let action_layout_style = decl_style::layout_style(theme, action_layout);
            (ring, action_layout_style)
        };
        let pressable = PressableProps {
            enabled: !disabled,
            focus_ring: Some(ring),
            layout: action_layout_style,
            a11y: fret_ui::element::PressableA11y {
                role: Some(SemanticsRole::Button),
                label: Some(label),
                test_id,
                ..Default::default()
            },
            ..Default::default()
        };
        let children = self.children;
        cx.pressable(pressable, move |cx, st| {
            cx.pressable_dispatch_command_if_enabled_opt(on_click);
            if let Some(on_activate) = on_activate.clone() {
                cx.pressable_on_activate(on_activate);
            }

            let theme = Theme::global(&*cx.app);
            let fg = if disabled {
                alpha_mul(sidebar_fg(theme), 0.5)
            } else if st.hovered || st.pressed {
                sidebar_accent_fg(theme)
            } else {
                sidebar_fg(theme)
            };
            let bg = if st.hovered || st.pressed {
                sidebar_accent(theme)
            } else {
                Color::TRANSPARENT
            };
            let mut chrome = ChromeRefinement::default()
                .rounded(Radius::Md)
                .p(Space::N0)
                .text_color(ColorRef::Color(fg));
            if bg.a > 0.0 {
                chrome = chrome.bg(ColorRef::Color(bg));
            }
            let props = decl_style::container_props(
                theme,
                chrome.merge(user_chrome.clone()),
                content_layout.clone(),
            );
            let children = children.clone();
            vec![cx.container(props, move |_cx| children)]
        })
    }
}

#[derive(Debug, Clone)]
pub struct SidebarMenuBadge {
    label: Arc<str>,
    size: SidebarMenuButtonSize,
    collapsed: bool,
    test_id: Option<Arc<str>>,
    chrome: ChromeRefinement,
    layout: LayoutRefinement,
}

impl SidebarMenuBadge {
    pub fn new(label: impl Into<Arc<str>>) -> Self {
        Self {
            label: label.into(),
            size: SidebarMenuButtonSize::Default,
            collapsed: false,
            test_id: None,
            chrome: ChromeRefinement::default(),
            layout: LayoutRefinement::default(),
        }
    }

    pub fn size(mut self, size: SidebarMenuButtonSize) -> Self {
        self.size = size;
        self
    }

    pub fn collapsed(mut self, collapsed: bool) -> Self {
        self.collapsed = collapsed;
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
        let collapsed = sidebar_collapsed_in_scope(cx);
        let collapsed = if self.collapsed { true } else { collapsed };
        if collapsed {
            return cx.spacer(SpacerProps {
                min: Px(0.0),
                ..Default::default()
            });
        }

        let top = sidebar_menu_affordance_top(self.size);
        let (props, text_px, text_lh, fg) = {
            let theme = Theme::global(&*cx.app);
            let right = theme
                .metric_by_key("component.sidebar.menu_badge.right")
                .unwrap_or(Px(4.0));
            let h = theme
                .metric_by_key("component.sidebar.menu_badge.h")
                .unwrap_or(Px(20.0));
            let min_w = theme
                .metric_by_key("component.sidebar.menu_badge.min_w")
                .unwrap_or(Px(20.0));
            let text_px = theme
                .metric_by_key("component.sidebar.menu_badge.text_px")
                .unwrap_or(Px(12.0));
            let text_lh = theme
                .metric_by_key("component.sidebar.menu_badge.line_height")
                .unwrap_or(Px(16.0));
            let fg = sidebar_fg(theme);

            let props = decl_style::container_props(
                theme,
                ChromeRefinement::default()
                    .px(Space::N1)
                    .rounded(Radius::Md)
                    .merge(self.chrome),
                LayoutRefinement::default()
                    .absolute()
                    .top_px(top)
                    .right_px(right)
                    .h_px(h)
                    .min_h(h)
                    .min_w(min_w)
                    .merge(self.layout),
            );
            (props, text_px, text_lh, fg)
        };

        let text = ui::text(cx, self.label)
            .text_size_px(text_px)
            .line_height_px(text_lh)
            .font_medium()
            .text_color(ColorRef::Color(fg))
            .nowrap()
            .into_element(cx);

        let mut badge = shadcn_layout::container_hstack_centered(cx, props, Space::N0, vec![text]);
        if let Some(test_id) = self.test_id {
            badge = badge.test_id(test_id);
        }
        badge
    }
}

#[derive(Debug, Clone)]
pub struct SidebarMenuSkeleton {
    show_icon: bool,
    text_fraction: f32,
    collapsed: bool,
    test_id: Option<Arc<str>>,
    chrome: ChromeRefinement,
    layout: LayoutRefinement,
}

impl SidebarMenuSkeleton {
    pub fn new() -> Self {
        Self {
            show_icon: false,
            text_fraction: 0.7,
            collapsed: false,
            test_id: None,
            chrome: ChromeRefinement::default(),
            layout: LayoutRefinement::default(),
        }
    }

    pub fn show_icon(mut self, show_icon: bool) -> Self {
        self.show_icon = show_icon;
        self
    }

    pub fn text_fraction(mut self, text_fraction: f32) -> Self {
        self.text_fraction = text_fraction.clamp(0.5, 0.9);
        self
    }

    pub fn collapsed(mut self, collapsed: bool) -> Self {
        self.collapsed = collapsed;
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
        let collapsed = sidebar_collapsed_in_scope(cx);
        let collapsed = if self.collapsed { true } else { collapsed };
        if collapsed {
            return cx.spacer(SpacerProps {
                min: Px(0.0),
                ..Default::default()
            });
        }

        let props = {
            let theme = Theme::global(&*cx.app);
            let h = sidebar_menu_button_h(theme, SidebarMenuButtonSize::Default);
            decl_style::container_props(
                theme,
                ChromeRefinement::default()
                    .px(Space::N2)
                    .rounded(Radius::Md)
                    .merge(self.chrome),
                LayoutRefinement::default()
                    .w_full()
                    .h_px(h)
                    .merge(self.layout),
            )
        };

        let icon = if self.show_icon {
            Some(
                Skeleton::new()
                    .refine_layout(LayoutRefinement::default().w_px(Px(16.0)).h_px(Px(16.0)))
                    .into_element(cx),
            )
        } else {
            None
        };

        let text_w = Px((self.text_fraction * 100.0).clamp(50.0, 90.0));
        let text = Skeleton::new()
            .refine_layout(
                LayoutRefinement::default()
                    .h_px(Px(16.0))
                    .w_px(text_w)
                    .max_w(Px(240.0)),
            )
            .into_element(cx);

        let mut out = Vec::new();
        if let Some(icon) = icon {
            out.push(icon);
        }
        out.push(text);

        let mut node = shadcn_layout::container_hstack_centered(cx, props, Space::N2, out);
        if let Some(test_id) = self.test_id {
            node = node.test_id(test_id);
        }
        node
    }
}

#[derive(Debug, Clone)]
pub struct SidebarMenuSub {
    children: Vec<AnyElement>,
    collapsed: bool,
    chrome: ChromeRefinement,
    layout: LayoutRefinement,
}

impl SidebarMenuSub {
    pub fn new(children: impl IntoIterator<Item = AnyElement>) -> Self {
        Self {
            children: children.into_iter().collect(),
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
        if collapsed {
            return cx.spacer(SpacerProps {
                min: Px(0.0),
                ..Default::default()
            });
        }

        let mut props = {
            let theme = Theme::global(&*cx.app);
            decl_style::container_props(
                theme,
                ChromeRefinement::default()
                    .border_1()
                    .border_color(ColorRef::Color(sidebar_border(theme)))
                    .px(Space::N2p5)
                    .py(Space::N0p5)
                    .merge(self.chrome),
                LayoutRefinement::default()
                    .w_full()
                    .min_w_0()
                    .mx_px(Px(14.0))
                    .merge(self.layout),
            )
        };
        props.border.top = Px(0.0);
        props.border.right = Px(0.0);
        props.border.bottom = Px(0.0);

        let children = self.children;
        shadcn_layout::container_vstack(
            cx,
            props,
            fret_ui_kit::declarative::stack::VStackProps::default().gap(Space::N1),
            children,
        )
        .attach_semantics(SemanticsDecoration::default().role(SemanticsRole::List))
    }
}

#[derive(Debug, Clone)]
pub struct SidebarMenuSubItem {
    children: Vec<AnyElement>,
    test_id: Option<Arc<str>>,
    layout: LayoutRefinement,
}

impl SidebarMenuSubItem {
    pub fn new(child: AnyElement) -> Self {
        Self {
            children: vec![child],
            test_id: None,
            layout: LayoutRefinement::default(),
        }
    }

    pub fn children(mut self, children: impl IntoIterator<Item = AnyElement>) -> Self {
        self.children = children.into_iter().collect();
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

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let props = {
            let theme = Theme::global(&*cx.app);
            decl_style::container_props(
                theme,
                ChromeRefinement::default(),
                LayoutRefinement::default()
                    .relative()
                    .w_full()
                    .min_w_0()
                    .merge(self.layout),
            )
        };

        let mut semantics = SemanticsDecoration::default().role(SemanticsRole::ListItem);
        if let Some(test_id) = self.test_id {
            semantics = semantics.test_id(test_id);
        }

        let children = self.children;
        cx.container(props, move |_cx| children)
            .attach_semantics(semantics)
    }
}

#[derive(Clone)]
pub struct SidebarMenuSubButton {
    label: Arc<str>,
    children: Option<Vec<AnyElement>>,
    icon: Option<IconId>,
    active: bool,
    disabled: bool,
    collapsed: bool,
    size: SidebarMenuSubButtonSize,
    as_child: bool,
    href: Option<Arc<str>>,
    target: Option<Arc<str>>,
    rel: Option<Arc<str>>,
    on_navigate: Option<OnActivate>,
    on_click: Option<CommandId>,
    on_activate: Option<OnActivate>,
    test_id: Option<Arc<str>>,
}

impl std::fmt::Debug for SidebarMenuSubButton {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SidebarMenuSubButton")
            .field("label", &self.label)
            .field("children_len", &self.children.as_ref().map(Vec::len))
            .field("icon", &self.icon)
            .field("active", &self.active)
            .field("disabled", &self.disabled)
            .field("collapsed", &self.collapsed)
            .field("size", &self.size)
            .field("as_child", &self.as_child)
            .field("href", &self.href)
            .field("target", &self.target)
            .field("rel", &self.rel)
            .field("on_navigate", &self.on_navigate.is_some())
            .field("on_click", &self.on_click)
            .field("on_activate", &self.on_activate.is_some())
            .field("test_id", &self.test_id)
            .finish()
    }
}

impl SidebarMenuSubButton {
    pub fn new(label: impl Into<Arc<str>>) -> Self {
        Self {
            label: label.into(),
            children: None,
            icon: None,
            active: false,
            disabled: false,
            collapsed: false,
            size: SidebarMenuSubButtonSize::Md,
            as_child: false,
            href: None,
            target: None,
            rel: None,
            on_navigate: None,
            on_click: None,
            on_activate: None,
            test_id: None,
        }
    }

    pub fn icon(mut self, icon: IconId) -> Self {
        self.icon = Some(icon);
        self
    }

    pub fn children(mut self, children: impl IntoIterator<Item = AnyElement>) -> Self {
        self.children = Some(children.into_iter().collect());
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

    pub fn size(mut self, size: SidebarMenuSubButtonSize) -> Self {
        self.size = size;
        self
    }

    pub fn as_child(mut self, as_child: bool) -> Self {
        self.as_child = as_child;
        self
    }

    pub fn href(mut self, href: impl Into<Arc<str>>) -> Self {
        self.href = Some(href.into());
        self
    }

    pub fn target(mut self, target: impl Into<Arc<str>>) -> Self {
        self.target = Some(target.into());
        self
    }

    pub fn rel(mut self, rel: impl Into<Arc<str>>) -> Self {
        self.rel = Some(rel.into());
        self
    }

    pub fn on_navigate(mut self, on_navigate: OnActivate) -> Self {
        self.on_navigate = Some(on_navigate);
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

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let collapsed = sidebar_collapsed_in_scope(cx);
        let collapsed = if self.collapsed { true } else { collapsed };
        if collapsed {
            return cx.spacer(SpacerProps {
                min: Px(0.0),
                ..Default::default()
            });
        }

        let (ring, pressable_layout) = {
            let theme = Theme::global(&*cx.app);
            let radius = decl_style::radius(theme, Radius::Md);
            let ring = sidebar_ring(theme, radius);
            let h = sidebar_menu_sub_button_h(theme);
            let pressable_layout =
                decl_style::layout_style(theme, LayoutRefinement::default().w_full().h_px(h));
            (ring, pressable_layout)
        };
        let label = self.label.clone();
        let on_click = self.on_click.clone();
        let on_navigate = self.on_navigate.clone();
        let on_activate = self.on_activate.clone();
        let test_id = self.test_id.clone();
        let href = self.href.clone();
        let target = self.target.clone();
        let rel = self.rel.clone();
        let as_child = self.as_child;
        let a11y_role = if href.is_some() && !as_child {
            SemanticsRole::Link
        } else {
            SemanticsRole::Button
        };
        let href_for_semantics = if !as_child { href.clone() } else { None };
        let slot_children = self.children.clone();
        let disabled = self.disabled
            || on_click
                .as_ref()
                .is_some_and(|cmd| !cx.command_is_enabled(cmd));

        let pressable = PressableProps {
            enabled: !disabled,
            focus_ring: Some(ring),
            layout: pressable_layout,
            a11y: fret_ui::element::PressableA11y {
                role: Some(a11y_role),
                label: Some(label.clone()),
                test_id: test_id.clone(),
                ..Default::default()
            },
            ..Default::default()
        };

        let icon = self.icon;
        let active = self.active;
        let size = self.size;
        let navigate_handler: Option<OnActivate> = if let Some(on_navigate) = on_navigate {
            Some(on_navigate)
        } else {
            href.clone()
                .map(|url| sidebar_open_url_on_activate(url, target.clone(), rel.clone()))
        };

        let element = cx.pressable(pressable, move |cx, st| {
            cx.pressable_dispatch_command_if_enabled_opt(on_click.clone());
            if let Some(on_navigate) = navigate_handler.clone() {
                cx.pressable_on_activate(on_navigate);
            }
            if let Some(on_activate) = on_activate.clone() {
                cx.pressable_on_activate(on_activate);
            }

            let (fg, props, style, gap) = {
                let theme = Theme::global(&*cx.app);
                let bg = if active || st.hovered || st.pressed {
                    sidebar_accent(theme)
                } else {
                    Color::TRANSPARENT
                };
                let fg = if disabled {
                    alpha_mul(sidebar_fg(theme), 0.5)
                } else if active || st.hovered || st.pressed {
                    sidebar_accent_fg(theme)
                } else {
                    sidebar_fg(theme)
                };

                let chrome = if bg.a > 0.0 {
                    ChromeRefinement::default()
                        .bg(ColorRef::Color(bg))
                        .rounded(Radius::Md)
                } else {
                    ChromeRefinement::default().rounded(Radius::Md)
                };
                let h = sidebar_menu_sub_button_h(theme);
                let props = decl_style::container_props(
                    theme,
                    chrome,
                    LayoutRefinement::default().w_full().h_px(h),
                );
                let style = menu_sub_button_style(theme, size);
                let gap = decl_style::space(theme, Space::N2);
                (fg, props, style, gap)
            };

            vec![cx.container(props, move |cx| {
                let row = FlexProps {
                    direction: fret_core::Axis::Horizontal,
                    gap,
                    align: CrossAlign::Center,
                    justify: MainAlign::Start,
                    padding: Edges::all(gap),
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

                let slot_children = slot_children.clone();
                vec![cx.flex(row, move |cx| {
                    if as_child {
                        if let Some(children) = slot_children.clone() {
                            return children;
                        }
                    }
                    let mut out = Vec::new();
                    if let Some(icon) = icon {
                        out.push(decl_icon::icon(cx, icon));
                    }

                    let mut text = ui::text(cx, label.clone())
                        .w_full()
                        .min_w_0()
                        .flex_1()
                        .basis_0()
                        .text_size_px(style.size)
                        .font_weight(style.weight)
                        .text_color(ColorRef::Color(fg))
                        .truncate();
                    if let Some(line_height) = style.line_height {
                        text = text.line_height_px(line_height);
                    }

                    out.push(text.into_element(cx));
                    out
                })]
            })]
        });

        if let Some(href) = href_for_semantics {
            element.attach_semantics(SemanticsDecoration::default().value(href))
        } else {
            element
        }
    }
}

#[derive(Clone)]
pub struct SidebarMenuButton {
    label: Arc<str>,
    children: Option<Vec<AnyElement>>,
    icon: Option<IconId>,
    active: bool,
    disabled: bool,
    collapsed: bool,
    variant: SidebarMenuButtonVariant,
    size: SidebarMenuButtonSize,
    as_child: bool,
    href: Option<Arc<str>>,
    target: Option<Arc<str>>,
    rel: Option<Arc<str>>,
    on_navigate: Option<OnActivate>,
    on_click: Option<CommandId>,
    on_activate: Option<OnActivate>,
    test_id: Option<Arc<str>>,
}

impl std::fmt::Debug for SidebarMenuButton {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SidebarMenuButton")
            .field("label", &self.label)
            .field("children_len", &self.children.as_ref().map(Vec::len))
            .field("icon", &self.icon)
            .field("active", &self.active)
            .field("disabled", &self.disabled)
            .field("collapsed", &self.collapsed)
            .field("variant", &self.variant)
            .field("size", &self.size)
            .field("as_child", &self.as_child)
            .field("href", &self.href)
            .field("target", &self.target)
            .field("rel", &self.rel)
            .field("on_navigate", &self.on_navigate.is_some())
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
            children: None,
            icon: None,
            active: false,
            disabled: false,
            collapsed: false,
            variant: SidebarMenuButtonVariant::Default,
            size: SidebarMenuButtonSize::Default,
            as_child: false,
            href: None,
            target: None,
            rel: None,
            on_navigate: None,
            on_click: None,
            on_activate: None,
            test_id: None,
        }
    }

    pub fn icon(mut self, icon: IconId) -> Self {
        self.icon = Some(icon);
        self
    }

    pub fn children(mut self, children: impl IntoIterator<Item = AnyElement>) -> Self {
        self.children = Some(children.into_iter().collect());
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

    pub fn variant(mut self, variant: SidebarMenuButtonVariant) -> Self {
        self.variant = variant;
        self
    }

    pub fn size(mut self, size: SidebarMenuButtonSize) -> Self {
        self.size = size;
        self
    }

    pub fn as_child(mut self, as_child: bool) -> Self {
        self.as_child = as_child;
        self
    }

    pub fn href(mut self, href: impl Into<Arc<str>>) -> Self {
        self.href = Some(href.into());
        self
    }

    pub fn target(mut self, target: impl Into<Arc<str>>) -> Self {
        self.target = Some(target.into());
        self
    }

    pub fn rel(mut self, rel: impl Into<Arc<str>>) -> Self {
        self.rel = Some(rel.into());
        self
    }

    pub fn on_navigate(mut self, on_navigate: OnActivate) -> Self {
        self.on_navigate = Some(on_navigate);
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
        let (ring, pressable_layout) = {
            let theme = Theme::global(&*cx.app);
            let radius = decl_style::radius(theme, Radius::Md);
            let ring = sidebar_ring(theme, radius);
            let h = transition_prim::lerp_px(
                sidebar_menu_button_collapsed_h(theme),
                sidebar_menu_button_h(theme, self.size),
                expanded_progress,
            );
            let pressable_layout = decl_style::layout_style(
                theme,
                LayoutRefinement::default().w_full().h_px(MetricRef::Px(h)),
            );
            (ring, pressable_layout)
        };

        let label = self.label.clone();
        let on_click = self.on_click.clone();
        let on_navigate = self.on_navigate.clone();
        let on_activate = self.on_activate.clone();
        let test_id = self.test_id.clone();
        let href = self.href.clone();
        let target = self.target.clone();
        let rel = self.rel.clone();
        let as_child = self.as_child;
        let a11y_role = if href.is_some() && !as_child {
            SemanticsRole::Link
        } else {
            SemanticsRole::Button
        };
        let href_for_semantics = if !as_child { href.clone() } else { None };
        let slot_children = self.children.clone();
        let disabled = self.disabled
            || on_click
                .as_ref()
                .is_some_and(|cmd| !cx.command_is_enabled(cmd));
        let pressable = PressableProps {
            enabled: !disabled,
            focus_ring: Some(ring),
            layout: pressable_layout,
            a11y: fret_ui::element::PressableA11y {
                role: Some(a11y_role),
                label: Some(label.clone()),
                test_id: test_id.clone(),
                ..Default::default()
            },
            ..Default::default()
        };

        let icon = self.icon.clone();
        let active = self.active;
        let disabled = disabled;
        let variant = self.variant;
        let size = self.size;
        let expanded_progress = expanded_progress.clamp(0.0, 1.0);

        let navigate_handler: Option<OnActivate> = if let Some(on_navigate) = on_navigate {
            Some(on_navigate)
        } else {
            href.clone()
                .map(|url| sidebar_open_url_on_activate(url, target.clone(), rel.clone()))
        };

        let element = cx.pressable(pressable, move |cx, st| {
            cx.pressable_dispatch_command_if_enabled_opt(on_click.clone());
            if let Some(on_navigate) = navigate_handler.clone() {
                cx.pressable_on_activate(on_navigate);
            }
            if let Some(on_activate) = on_activate.clone() {
                cx.pressable_on_activate(on_activate);
            }
            let (fg, props, inner_gap, label_style) = {
                let theme = Theme::global(&*cx.app);
                let bg = if active || st.hovered || st.pressed {
                    sidebar_accent(theme)
                } else {
                    Color::TRANSPARENT
                };

                let fg = if disabled {
                    alpha_mul(sidebar_fg(theme), 0.5)
                } else if active || st.hovered || st.pressed {
                    sidebar_accent_fg(theme)
                } else {
                    sidebar_fg(theme)
                };

                let chrome = if matches!(variant, SidebarMenuButtonVariant::Outline) {
                    let background = theme.color_required("background");
                    let border = sidebar_border(theme);
                    let mut chrome = ChromeRefinement::default()
                        .bg(ColorRef::Color(background))
                        .border_1()
                        .border_color(ColorRef::Color(border))
                        .rounded(Radius::Md);

                    if bg.a > 0.0 {
                        chrome = chrome
                            .bg(ColorRef::Color(bg))
                            .border_color(ColorRef::Color(bg));
                    }
                    chrome
                } else if bg.a > 0.0 {
                    ChromeRefinement::default()
                        .bg(ColorRef::Color(bg))
                        .rounded(Radius::Md)
                } else {
                    ChromeRefinement::default().rounded(Radius::Md)
                };

                let h = transition_prim::lerp_px(
                    sidebar_menu_button_collapsed_h(theme),
                    sidebar_menu_button_h(theme, size),
                    expanded_progress,
                );

                let mut props = decl_style::container_props(
                    theme,
                    chrome,
                    LayoutRefinement::default().w_full().h_px(MetricRef::Px(h)),
                );
                props.layout.overflow = Overflow::Clip;

                let inner_gap = decl_style::space(theme, Space::N2); // `gap-2`
                let label_style = menu_button_style(theme);
                (fg, props, inner_gap, label_style)
            };

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
                let slot_children = slot_children.clone();
                vec![cx.flex(row, move |cx| {
                    if as_child {
                        if let Some(children) = slot_children.clone() {
                            return children;
                        }
                    }
                    let mut out = Vec::new();
                    if let Some(icon) = icon.clone() {
                        out.push(decl_icon::icon(cx, icon));
                    }
                    if label_opacity > 0.0 {
                        let text = ui::text(cx, label.clone())
                            .w_full()
                            .min_w_0()
                            .flex_1()
                            .basis_0()
                            .text_size_px(label_style.size)
                            .font_weight(label_style.weight)
                            .text_color(ColorRef::Color(fg))
                            .truncate();

                        let mut text = text;
                        if let Some(line_height) = label_style.line_height {
                            text = text.line_height_px(line_height);
                        }
                        if let Some(letter_spacing_em) = label_style.letter_spacing_em {
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
        });

        if let Some(href) = href_for_semantics {
            element.attach_semantics(SemanticsDecoration::default().value(href))
        } else {
            element
        }
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
        let (popover_bg, border, fg, label_style) = {
            let theme = Theme::global(&*cx.app);
            let popover_bg = theme
                .color_by_key("popover.background")
                .unwrap_or_else(|| theme.color_required("popover.background"));
            let border = theme
                .color_by_key("border")
                .unwrap_or_else(|| theme.color_required("border"));
            let fg = sidebar_fg(theme);
            let label_style = menu_button_style(theme);
            (popover_bg, border, fg, label_style)
        };

        let label = this.label.clone();

        let chrome = ChromeRefinement::default()
            .bg(ColorRef::Color(popover_bg))
            .border_1()
            .border_color(ColorRef::Color(border))
            .rounded(Radius::Md)
            .p(Space::N2);
        let content = TooltipContent::new({
            let mut text = ui::text(cx, label.clone())
                .text_size_px(label_style.size)
                .font_weight(label_style.weight)
                .text_color(ColorRef::Color(fg))
                .nowrap();
            if let Some(line_height) = label_style.line_height {
                text = text.line_height_px(line_height);
            }
            if let Some(letter_spacing_em) = label_style.letter_spacing_em {
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
    use std::sync::Mutex;

    use crate::shadcn_themes::{ShadcnBaseColor, ShadcnColorScheme, apply_shadcn_new_york_v4};
    use fret_app::App;
    use fret_core::{
        AppWindowId, PathCommand, PathConstraints, PathId, PathMetrics, PathService, Point, Px,
        Rect, SemanticsRole, Size as CoreSize, SvgId, SvgService, TextBlobId, TextConstraints,
        TextMetrics, TextService,
    };
    use fret_runtime::{Effect, FrameId, TickId};
    use fret_ui::element::ContainerProps;
    use fret_ui::elements;
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

    fn render_sidebar_variant_frame(
        ui: &mut UiTree<App>,
        app: &mut App,
        services: &mut FakeServices,
        window: AppWindowId,
        bounds: Rect,
        collapsed: bool,
        variant: SidebarVariant,
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
            "shadcn-sidebar-variant-width",
            |cx| {
                let child = cx.container(ContainerProps::default(), |_cx| Vec::new());
                let sidebar = Sidebar::new([child])
                    .collapsed(collapsed)
                    .variant(variant)
                    .into_element(cx);
                vec![sidebar]
            },
        );
        ui.set_root(root);
        ui.layout_all(app, services, bounds, 1.0);

        let sidebar_node = *ui.children(root).first().expect("sidebar node");
        ui.debug_node_bounds(sidebar_node).expect("sidebar bounds")
    }

    fn render_sidebar_inset_frame_with_surface(
        ui: &mut UiTree<App>,
        app: &mut App,
        services: &mut FakeServices,
        window: AppWindowId,
        bounds: Rect,
        open: bool,
        variant: SidebarVariant,
        frame: u64,
    ) -> Rect {
        app.set_frame_id(FrameId(frame));
        app.set_tick_id(TickId(frame));

        let open_model = app.models_mut().insert(open);

        let root = fret_ui::declarative::render_root(
            ui,
            app,
            services,
            window,
            bounds,
            "shadcn-sidebar-inset-variant-surface",
            |cx| {
                SidebarProvider::new()
                    .open(Some(open_model.clone()))
                    .with(cx, |cx| {
                        let inset = with_sidebar_surface_state(
                            cx,
                            SidebarSurfaceContext {
                                side: SidebarSide::Left,
                                collapsible: SidebarCollapsible::Offcanvas,
                                variant,
                            },
                            |cx| {
                                let child = cx.spacer(SpacerProps {
                                    min: Px(0.0),
                                    ..Default::default()
                                });
                                SidebarInset::new([child]).into_element(cx)
                            },
                        );
                        vec![inset]
                    })
            },
        );
        ui.set_root(root);
        ui.layout_all(app, services, bounds, 1.0);

        let inset_node = *ui.children(root).first().expect("inset node");
        ui.debug_node_bounds(inset_node).expect("inset bounds")
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
    fn sidebar_variant_floating_and_inset_expand_outer_width_on_desktop() {
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

        let sidebar_rect = render_sidebar_variant_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            false,
            SidebarVariant::Sidebar,
            24,
        );
        let floating_rect = render_sidebar_variant_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            false,
            SidebarVariant::Floating,
            24,
        );
        let inset_rect = render_sidebar_variant_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            false,
            SidebarVariant::Inset,
            24,
        );

        let theme = Theme::global(&app).clone();
        let expected_delta = decl_style::space(&theme, Space::N2).0 * 2.0 + 2.0;

        let floating_delta = floating_rect.size.width.0 - sidebar_rect.size.width.0;
        assert!(
            (floating_delta - expected_delta).abs() <= 1.0,
            "expected floating variant width delta ~{expected_delta}, got {floating_delta}"
        );

        let inset_delta = inset_rect.size.width.0 - sidebar_rect.size.width.0;
        assert!(
            (inset_delta - expected_delta).abs() <= 1.0,
            "expected inset variant width delta ~{expected_delta}, got {inset_delta}"
        );
    }

    #[test]
    fn sidebar_inset_variant_collapsed_adds_left_margin_step() {
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

        let expanded = render_sidebar_inset_frame_with_surface(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            true,
            SidebarVariant::Inset,
            1,
        );
        let collapsed = render_sidebar_inset_frame_with_surface(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            false,
            SidebarVariant::Inset,
            2,
        );

        let theme = Theme::global(&app).clone();
        let expected_shift = decl_style::space(&theme, Space::N2).0;
        let actual_shift = collapsed.origin.x.0 - expanded.origin.x.0;

        assert!(
            (actual_shift - expected_shift).abs() <= 1.0,
            "expected inset collapsed left shift ~{expected_shift}, got {actual_shift}"
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
    fn sidebar_mobile_branch_uses_sheet_dialog_semantics_and_renders_content() {
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

        let open_model = app.models_mut().insert(false);
        let open_mobile_model = app.models_mut().insert(true);

        app.set_frame_id(FrameId(1));
        app.set_tick_id(TickId(1));
        OverlayController::begin_frame(&mut app, window);
        let root = fret_ui::declarative::render_root(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "shadcn-sidebar-mobile-sheet",
            |cx| {
                SidebarProvider::new()
                    .open(Some(open_model.clone()))
                    .open_mobile(Some(open_mobile_model.clone()))
                    .is_mobile(true)
                    .with(cx, |cx| {
                        let content = SidebarMenuButton::new("Inbox")
                            .test_id("sidebar-mobile-menu-button")
                            .into_element(cx);
                        let sidebar = Sidebar::new([content]).into_element(cx);
                        vec![sidebar]
                    })
            },
        );
        ui.set_root(root);
        OverlayController::render(&mut ui, &mut app, &mut services, window, bounds);
        ui.request_semantics_snapshot();
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        let snap = ui
            .semantics_snapshot()
            .cloned()
            .expect("expected semantics snapshot");
        assert!(
            snap.nodes.iter().any(|n| n.role == SemanticsRole::Dialog),
            "expected mobile sidebar branch to expose sheet dialog semantics"
        );
        assert!(
            snap.nodes
                .iter()
                .any(|n| n.test_id.as_deref() == Some("sidebar-mobile-menu-button")),
            "expected mobile sidebar sheet content to render sidebar children"
        );
    }

    #[test]
    fn sidebar_provider_infers_mobile_from_viewport_width_when_unset() {
        let window = AppWindowId::default();
        let mut app = App::new();
        apply_shadcn_new_york_v4(&mut app, ShadcnBaseColor::Neutral, ShadcnColorScheme::Light);
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);
        let mut services = FakeServices;

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            CoreSize::new(Px(360.0), Px(640.0)),
        );

        let open_model = app.models_mut().insert(false);
        let open_mobile_model = app.models_mut().insert(true);

        app.set_frame_id(FrameId(1));
        app.set_tick_id(TickId(1));
        OverlayController::begin_frame(&mut app, window);
        let root = fret_ui::declarative::render_root(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "shadcn-sidebar-mobile-infer-from-viewport",
            |cx| {
                SidebarProvider::new()
                    .open(Some(open_model.clone()))
                    .open_mobile(Some(open_mobile_model.clone()))
                    .with(cx, |cx| {
                        let content = SidebarMenuButton::new("Inbox")
                            .test_id("sidebar-mobile-menu-button")
                            .into_element(cx);
                        let sidebar = Sidebar::new([content]).into_element(cx);
                        vec![sidebar]
                    })
            },
        );
        ui.set_root(root);
        OverlayController::render(&mut ui, &mut app, &mut services, window, bounds);
        ui.request_semantics_snapshot();
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        let snap = ui
            .semantics_snapshot()
            .cloned()
            .expect("expected semantics snapshot");
        assert!(
            snap.nodes.iter().any(|n| n.role == SemanticsRole::Dialog),
            "expected inferred mobile sidebar branch to expose sheet dialog semantics"
        );
        assert!(
            snap.nodes
                .iter()
                .any(|n| n.test_id.as_deref() == Some("sidebar-mobile-menu-button")),
            "expected inferred mobile sidebar sheet content to render sidebar children"
        );
    }

    #[test]
    fn sidebar_mobile_provider_open_false_does_not_hide_collapsed_sensitive_children() {
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

        let open_model = app.models_mut().insert(false);

        let root = fret_ui::declarative::render_root(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "shadcn-sidebar-mobile-not-collapsed",
            |cx| {
                SidebarProvider::new()
                    .open(Some(open_model.clone()))
                    .is_mobile(true)
                    .with(cx, |cx| {
                        let skeleton = SidebarMenuSkeleton::new()
                            .show_icon(true)
                            .test_id("sidebar-mobile-skeleton")
                            .into_element(cx);
                        vec![skeleton]
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
        assert!(
            snap.nodes
                .iter()
                .any(|n| n.test_id.as_deref() == Some("sidebar-mobile-skeleton")),
            "expected mobile provider open=false path to keep collapsed-sensitive children visible"
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

    #[test]
    fn sidebar_menu_and_item_expose_list_semantics() {
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

        let root = fret_ui::declarative::render_root(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "shadcn-sidebar-menu-semantics",
            |cx| {
                let button = SidebarMenuButton::new("Inbox")
                    .test_id("sidebar-menu-button")
                    .into_element(cx);
                let item = SidebarMenuItem::new(button.clone())
                    .children([button])
                    .test_id("sidebar-menu-item")
                    .into_element(cx);
                let menu = SidebarMenu::new([item]).into_element(cx);
                vec![menu]
            },
        );
        ui.set_root(root);
        ui.request_semantics_snapshot();
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        let snap = ui
            .semantics_snapshot()
            .cloned()
            .expect("expected semantics snapshot");

        assert!(
            snap.nodes.iter().any(|n| n.role == SemanticsRole::List),
            "expected sidebar menu to expose list semantics"
        );
        let item = snap
            .nodes
            .iter()
            .find(|n| n.test_id.as_deref() == Some("sidebar-menu-item"))
            .expect("expected sidebar menu item semantics node");
        assert_eq!(
            item.role,
            SemanticsRole::ListItem,
            "expected sidebar menu item to expose list item semantics"
        );
    }

    #[test]
    fn sidebar_header_and_footer_apply_gap_two_spacing_contract() {
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

        let root = fret_ui::declarative::render_root(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "shadcn-sidebar-header-footer-gap",
            |cx| {
                let header_a = SidebarMenuButton::new("Header A")
                    .test_id("sidebar-header-a")
                    .into_element(cx);
                let header_b = SidebarMenuButton::new("Header B")
                    .test_id("sidebar-header-b")
                    .into_element(cx);
                let header = SidebarHeader::new([header_a, header_b]).into_element(cx);

                let footer_a = SidebarMenuButton::new("Footer A")
                    .test_id("sidebar-footer-a")
                    .into_element(cx);
                let footer_b = SidebarMenuButton::new("Footer B")
                    .test_id("sidebar-footer-b")
                    .into_element(cx);
                let footer = SidebarFooter::new([footer_a, footer_b]).into_element(cx);

                vec![header, footer]
            },
        );
        ui.set_root(root);
        ui.request_semantics_snapshot();
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        let snap = ui
            .semantics_snapshot()
            .cloned()
            .expect("expected semantics snapshot");

        let header_a = snap
            .nodes
            .iter()
            .find(|n| n.test_id.as_deref() == Some("sidebar-header-a"))
            .expect("expected sidebar header first child semantics node");
        let header_b = snap
            .nodes
            .iter()
            .find(|n| n.test_id.as_deref() == Some("sidebar-header-b"))
            .expect("expected sidebar header second child semantics node");
        let footer_a = snap
            .nodes
            .iter()
            .find(|n| n.test_id.as_deref() == Some("sidebar-footer-a"))
            .expect("expected sidebar footer first child semantics node");
        let footer_b = snap
            .nodes
            .iter()
            .find(|n| n.test_id.as_deref() == Some("sidebar-footer-b"))
            .expect("expected sidebar footer second child semantics node");

        let expected_gap = 8.0_f32;
        let header_gap = header_b.bounds.origin.y.0
            - (header_a.bounds.origin.y.0 + header_a.bounds.size.height.0);
        let footer_gap = footer_b.bounds.origin.y.0
            - (footer_a.bounds.origin.y.0 + footer_a.bounds.size.height.0);

        assert!(
            (header_gap - expected_gap).abs() <= 1.0,
            "expected sidebar header child gap ~{expected_gap}, got {header_gap}"
        );
        assert!(
            (footer_gap - expected_gap).abs() <= 1.0,
            "expected sidebar footer child gap ~{expected_gap}, got {footer_gap}"
        );
    }

    #[test]
    fn sidebar_group_wrapper_is_relative_and_stretches_width() {
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

        let root = fret_ui::declarative::render_root(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "shadcn-sidebar-group-wrapper",
            |cx| {
                let group_action = SidebarGroupAction::new(Vec::<AnyElement>::new())
                    .test_id("sidebar-group-action")
                    .into_element(cx);
                let group_content =
                    SidebarGroupContent::new(Vec::<AnyElement>::new()).into_element(cx);
                let group = SidebarGroup::new([group_action, group_content]).into_element(cx);
                let sidebar = Sidebar::new([group]).into_element(cx);
                vec![sidebar]
            },
        );
        ui.set_root(root);
        ui.request_semantics_snapshot();
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        let snap = ui
            .semantics_snapshot()
            .cloned()
            .expect("expected semantics snapshot");
        let group_action = snap
            .nodes
            .iter()
            .find(|n| n.test_id.as_deref() == Some("sidebar-group-action"))
            .expect("expected sidebar group action semantics node");

        let action_left = group_action.bounds.origin.x.0;

        assert!(
            action_left > 10.0,
            "expected sidebar group action to be inset from sidebar left edge via relative wrapper, got x={action_left}"
        );
    }

    #[test]
    fn sidebar_group_action_mobile_hit_area_expands_vs_desktop() {
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

        let render_case = |is_mobile: bool,
                           ui: &mut UiTree<App>,
                           app: &mut App,
                           services: &mut FakeServices,
                           test_id: &'static str|
         -> Rect {
            let root = fret_ui::declarative::render_root(
                ui,
                app,
                services,
                window,
                bounds,
                "shadcn-sidebar-group-action-mobile-hit-area",
                |cx| {
                    SidebarProvider::new().is_mobile(is_mobile).with(cx, |cx| {
                        let action = SidebarGroupAction::new(Vec::<AnyElement>::new())
                            .test_id(test_id)
                            .into_element(cx);
                        let group = SidebarGroup::new([action]).into_element(cx);
                        vec![group]
                    })
                },
            );
            ui.set_root(root);
            ui.request_semantics_snapshot();
            ui.layout_all(app, services, bounds, 1.0);

            let snap = ui
                .semantics_snapshot()
                .cloned()
                .expect("expected semantics snapshot");
            snap.nodes
                .iter()
                .find(|n| n.test_id.as_deref() == Some(test_id))
                .map(|n| n.bounds)
                .expect("expected sidebar group action semantics node")
        };

        let desktop = render_case(
            false,
            &mut ui,
            &mut app,
            &mut services,
            "sidebar-group-action-desktop",
        );
        let mobile = render_case(
            true,
            &mut ui,
            &mut app,
            &mut services,
            "sidebar-group-action-mobile",
        );

        assert!(
            mobile.size.width.0 > desktop.size.width.0 + 8.0,
            "expected mobile group action hit area to expand width; desktop={} mobile={}",
            desktop.size.width.0,
            mobile.size.width.0
        );
        assert!(
            mobile.size.height.0 > desktop.size.height.0 + 8.0,
            "expected mobile group action hit area to expand height; desktop={} mobile={}",
            desktop.size.height.0,
            mobile.size.height.0
        );
    }

    #[test]
    fn sidebar_content_uses_collapse_only_for_overflow_not_padding() {
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

        let open_model = app.models_mut().insert(true);

        let render_with_open = |open: bool,
                                ui: &mut UiTree<App>,
                                app: &mut App,
                                services: &mut FakeServices|
         -> Rect {
            let _ = app.models_mut().update(&open_model, |value| {
                *value = open;
            });

            let root = fret_ui::declarative::render_root(
                ui,
                app,
                services,
                window,
                bounds,
                "shadcn-sidebar-content-padding",
                |cx| {
                    SidebarProvider::new()
                        .open(Some(open_model.clone()))
                        .with(cx, |cx| {
                            let marker = SidebarMenuButton::new("Inbox")
                                .test_id("sidebar-content-marker")
                                .into_element(cx);
                            let content = SidebarContent::new([marker]).into_element(cx);
                            let sidebar = Sidebar::new([content]).into_element(cx);
                            vec![sidebar]
                        })
                },
            );
            ui.set_root(root);
            ui.request_semantics_snapshot();
            ui.layout_all(app, services, bounds, 1.0);

            let snap = ui
                .semantics_snapshot()
                .cloned()
                .expect("expected semantics snapshot");
            snap.nodes
                .iter()
                .find(|n| n.test_id.as_deref() == Some("sidebar-content-marker"))
                .map(|n| n.bounds)
                .expect("expected sidebar content marker semantics node")
        };

        let expanded = render_with_open(true, &mut ui, &mut app, &mut services);
        let collapsed = render_with_open(false, &mut ui, &mut app, &mut services);

        let vertical_delta = (collapsed.origin.y.0 - expanded.origin.y.0).abs();
        assert!(
            vertical_delta <= 1.0,
            "expected collapse to keep content top padding stable, got delta={vertical_delta}"
        );
    }

    #[test]
    fn sidebar_menu_action_top_offset_tracks_menu_button_size() {
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

        let cases = [
            (SidebarMenuButtonSize::Sm, 4.0_f32),
            (SidebarMenuButtonSize::Default, 6.0_f32),
            (SidebarMenuButtonSize::Lg, 10.0_f32),
        ];

        for (size, expected_top) in cases {
            let root = fret_ui::declarative::render_root(
                &mut ui,
                &mut app,
                &mut services,
                window,
                bounds,
                "shadcn-sidebar-menu-action-offset",
                |cx| {
                    let button = SidebarMenuButton::new("Projects")
                        .size(size)
                        .test_id("sidebar-menu-button")
                        .into_element(cx);
                    let action = SidebarMenuAction::new(Vec::<AnyElement>::new())
                        .size(size)
                        .test_id("sidebar-menu-action")
                        .into_element(cx);
                    let item = SidebarMenuItem::new(button.clone())
                        .children([button, action])
                        .into_element(cx);
                    let menu = SidebarMenu::new([item]).into_element(cx);
                    vec![menu]
                },
            );
            ui.set_root(root);
            ui.request_semantics_snapshot();
            ui.layout_all(&mut app, &mut services, bounds, 1.0);

            let snap = ui
                .semantics_snapshot()
                .cloned()
                .expect("expected semantics snapshot");
            let button = snap
                .nodes
                .iter()
                .find(|n| n.test_id.as_deref() == Some("sidebar-menu-button"))
                .expect("expected sidebar menu button semantics node");
            let action = snap
                .nodes
                .iter()
                .find(|n| n.test_id.as_deref() == Some("sidebar-menu-action"))
                .expect("expected sidebar menu action semantics node");

            let actual_top = action.bounds.origin.y.0 - button.bounds.origin.y.0;
            assert!(
                (actual_top - expected_top).abs() <= 1.0,
                "expected menu action top offset ~{expected_top}px for {:?}, got {actual_top}px",
                size
            );
        }
    }

    #[test]
    fn sidebar_menu_action_mobile_hit_area_expands_vs_desktop() {
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

        let render_case = |is_mobile: bool,
                           ui: &mut UiTree<App>,
                           app: &mut App,
                           services: &mut FakeServices,
                           test_id: &'static str|
         -> Rect {
            let root = fret_ui::declarative::render_root(
                ui,
                app,
                services,
                window,
                bounds,
                "shadcn-sidebar-menu-action-mobile-hit-area",
                |cx| {
                    SidebarProvider::new().is_mobile(is_mobile).with(cx, |cx| {
                        let button = SidebarMenuButton::new("Projects")
                            .test_id("sidebar-menu-button")
                            .into_element(cx);
                        let action = SidebarMenuAction::new(Vec::<AnyElement>::new())
                            .test_id(test_id)
                            .into_element(cx);
                        let item = SidebarMenuItem::new(button.clone())
                            .children([button, action])
                            .into_element(cx);
                        vec![SidebarMenu::new([item]).into_element(cx)]
                    })
                },
            );
            ui.set_root(root);
            ui.request_semantics_snapshot();
            ui.layout_all(app, services, bounds, 1.0);

            let snap = ui
                .semantics_snapshot()
                .cloned()
                .expect("expected semantics snapshot");
            snap.nodes
                .iter()
                .find(|n| n.test_id.as_deref() == Some(test_id))
                .map(|n| n.bounds)
                .expect("expected sidebar menu action semantics node")
        };

        let desktop = render_case(
            false,
            &mut ui,
            &mut app,
            &mut services,
            "sidebar-menu-action-desktop",
        );
        let mobile = render_case(
            true,
            &mut ui,
            &mut app,
            &mut services,
            "sidebar-menu-action-mobile",
        );

        assert!(
            mobile.size.width.0 > desktop.size.width.0 + 8.0,
            "expected mobile menu action hit area to expand width; desktop={} mobile={}",
            desktop.size.width.0,
            mobile.size.width.0
        );
        assert!(
            mobile.size.height.0 > desktop.size.height.0 + 8.0,
            "expected mobile menu action hit area to expand height; desktop={} mobile={}",
            desktop.size.height.0,
            mobile.size.height.0
        );
    }

    #[test]
    fn sidebar_collapsed_hides_group_and_menu_affordances() {
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

        let open_model = app.models_mut().insert(false);
        let root = fret_ui::declarative::render_root(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "shadcn-sidebar-collapsed-affordance-visibility",
            |cx| {
                SidebarProvider::new()
                    .open(Some(open_model.clone()))
                    .with(cx, |cx| {
                        let button = SidebarMenuButton::new("Inbox")
                            .test_id("sidebar-menu-button")
                            .into_element(cx);
                        let action = SidebarMenuAction::new(Vec::<AnyElement>::new())
                            .test_id("sidebar-menu-action")
                            .into_element(cx);
                        let badge = SidebarMenuBadge::new("12")
                            .test_id("sidebar-menu-badge")
                            .into_element(cx);
                        let item = SidebarMenuItem::new(button.clone())
                            .children([button, action, badge])
                            .into_element(cx);

                        let menu = SidebarMenu::new([item]).into_element(cx);
                        let group_action = SidebarGroupAction::new(Vec::<AnyElement>::new())
                            .test_id("sidebar-group-action")
                            .into_element(cx);
                        let group_content = SidebarGroupContent::new([menu]).into_element(cx);
                        let group =
                            SidebarGroup::new([group_action, group_content]).into_element(cx);
                        vec![group]
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

        assert!(
            snap.nodes
                .iter()
                .any(|n| n.test_id.as_deref() == Some("sidebar-menu-button")),
            "expected sidebar menu button to remain visible in collapsed state"
        );
        assert!(
            !snap
                .nodes
                .iter()
                .any(|n| n.test_id.as_deref() == Some("sidebar-group-action")),
            "expected sidebar group action to be hidden when collapsed"
        );
        assert!(
            !snap
                .nodes
                .iter()
                .any(|n| n.test_id.as_deref() == Some("sidebar-menu-action")),
            "expected sidebar menu action to be hidden when collapsed"
        );
        assert!(
            !snap
                .nodes
                .iter()
                .any(|n| n.test_id.as_deref() == Some("sidebar-menu-badge")),
            "expected sidebar menu badge to be hidden when collapsed"
        );
    }

    #[test]
    fn sidebar_rail_toggles_provider_open_model_on_activate() {
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
            "shadcn-sidebar-rail-toggle",
            |cx| {
                SidebarProvider::new()
                    .open(Some(open_model.clone()))
                    .with(cx, |cx| {
                        let rail = SidebarRail::new().test_id("sidebar-rail").into_element(cx);
                        vec![rail]
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
        let rail = snap
            .nodes
            .iter()
            .find(|n| n.test_id.as_deref() == Some("sidebar-rail"))
            .expect("expected sidebar rail semantics node");

        let center = Point::new(
            Px(rail.bounds.origin.x.0 + rail.bounds.size.width.0 * 0.5),
            Px(rail.bounds.origin.y.0 + rail.bounds.size.height.0 * 0.5),
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
            "expected sidebar rail to toggle open model to false"
        );
    }

    #[test]
    fn sidebar_rail_hover_sets_col_resize_cursor_icon() {
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

        let root = fret_ui::declarative::render_root(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "shadcn-sidebar-rail-hover-cursor",
            |cx| {
                SidebarProvider::new().with(cx, |cx| {
                    vec![SidebarRail::new().test_id("sidebar-rail").into_element(cx)]
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
        let rail = snap
            .nodes
            .iter()
            .find(|n| n.test_id.as_deref() == Some("sidebar-rail"))
            .expect("expected sidebar rail semantics node");

        let center = Point::new(
            Px(rail.bounds.origin.x.0 + rail.bounds.size.width.0 * 0.5),
            Px(rail.bounds.origin.y.0 + rail.bounds.size.height.0 * 0.5),
        );

        let _ = app.flush_effects();
        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::Pointer(fret_core::PointerEvent::Move {
                pointer_id: fret_core::PointerId(0),
                position: center,
                buttons: fret_core::MouseButtons::default(),
                modifiers: fret_core::Modifiers::default(),
                pointer_type: fret_core::PointerType::Mouse,
            }),
        );

        let effects = app.flush_effects();
        assert!(
            effects.iter().any(|effect| {
                matches!(
                    effect,
                    Effect::CursorSetIcon { window: w, icon }
                        if *w == window && *icon == CursorIcon::ColResize
                )
            }),
            "expected sidebar rail hover to request col-resize cursor icon"
        );
    }

    #[test]
    fn sidebar_rail_tracks_side_and_offcanvas_position_matrix() {
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

        let render_case = |ui: &mut UiTree<App>,
                           app: &mut App,
                           services: &mut FakeServices,
                           side: SidebarSide,
                           collapsible: SidebarCollapsible,
                           test_id: &str| {
            let root = fret_ui::declarative::render_root(
                ui,
                app,
                services,
                window,
                bounds,
                "shadcn-sidebar-rail-side-matrix",
                |cx| {
                    let child = cx.spacer(SpacerProps {
                        min: Px(0.0),
                        ..Default::default()
                    });
                    let sidebar = Sidebar::new([child])
                        .side(side)
                        .collapsible(collapsible)
                        .into_element(cx);
                    let rail = SidebarRail::new().test_id(test_id).into_element(cx);
                    vec![sidebar, rail]
                },
            );
            ui.set_root(root);
            ui.request_semantics_snapshot();
            ui.layout_all(app, services, bounds, 1.0);

            let snap = ui
                .semantics_snapshot()
                .cloned()
                .expect("expected semantics snapshot");
            snap.nodes
                .iter()
                .find(|n| n.test_id.as_deref() == Some(test_id))
                .map(|n| n.bounds)
                .expect("expected sidebar rail semantics node")
        };

        let left_offcanvas = render_case(
            &mut ui,
            &mut app,
            &mut services,
            SidebarSide::Left,
            SidebarCollapsible::Offcanvas,
            "sidebar-rail-left-offcanvas",
        );
        let right_offcanvas = render_case(
            &mut ui,
            &mut app,
            &mut services,
            SidebarSide::Right,
            SidebarCollapsible::Offcanvas,
            "sidebar-rail-right-offcanvas",
        );

        assert!(
            left_offcanvas.origin.x.0 > right_offcanvas.origin.x.0,
            "expected left-side rail to be anchored on the right edge, got left={} right={}",
            left_offcanvas.origin.x.0,
            right_offcanvas.origin.x.0
        );

        assert!(
            (left_offcanvas.size.width.0 - 16.0).abs() <= 1.0
                && (right_offcanvas.size.width.0 - 16.0).abs() <= 1.0,
            "expected sidebar rail width ~16px, got left={} right={}",
            left_offcanvas.size.width.0,
            right_offcanvas.size.width.0
        );
    }

    #[test]
    fn sidebar_menu_action_show_on_hover_hides_until_item_hovered_on_desktop() {
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

        let root = fret_ui::declarative::render_root(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "shadcn-sidebar-menu-action-show-on-hover",
            |cx| {
                let item = SidebarMenuItem::new(cx.spacer(SpacerProps {
                    min: Px(0.0),
                    ..Default::default()
                }))
                .test_id("sidebar-menu-item")
                .into_element_with_children(cx, |cx| {
                    let button = SidebarMenuButton::new("Projects")
                        .test_id("sidebar-menu-button")
                        .into_element(cx);
                    let action = SidebarMenuAction::new(Vec::<AnyElement>::new())
                        .show_on_hover(true)
                        .test_id("sidebar-menu-action")
                        .into_element(cx);
                    vec![button, action]
                });
                let menu = SidebarMenu::new([item]).into_element(cx);
                vec![menu]
            },
        );
        ui.set_root(root);
        ui.request_semantics_snapshot();
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        let first = ui
            .semantics_snapshot()
            .cloned()
            .expect("expected semantics snapshot");

        assert!(
            !first
                .nodes
                .iter()
                .any(|n| n.test_id.as_deref() == Some("sidebar-menu-action")),
            "expected show_on_hover action to be hidden before hovering menu item"
        );

        let item = first
            .nodes
            .iter()
            .find(|n| n.test_id.as_deref() == Some("sidebar-menu-item"))
            .expect("expected sidebar menu item semantics node");
        let item_center = Point::new(
            Px(item.bounds.origin.x.0 + item.bounds.size.width.0 * 0.5),
            Px(item.bounds.origin.y.0 + item.bounds.size.height.0 * 0.5),
        );

        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::Pointer(fret_core::PointerEvent::Move {
                pointer_id: fret_core::PointerId(0),
                position: item_center,
                buttons: fret_core::MouseButtons::default(),
                modifiers: fret_core::Modifiers::default(),
                pointer_type: fret_core::PointerType::Mouse,
            }),
        );

        let root = fret_ui::declarative::render_root(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "shadcn-sidebar-menu-action-show-on-hover",
            |cx| {
                let item = SidebarMenuItem::new(cx.spacer(SpacerProps {
                    min: Px(0.0),
                    ..Default::default()
                }))
                .test_id("sidebar-menu-item")
                .into_element_with_children(cx, |cx| {
                    let button = SidebarMenuButton::new("Projects")
                        .test_id("sidebar-menu-button")
                        .into_element(cx);
                    let action = SidebarMenuAction::new(Vec::<AnyElement>::new())
                        .show_on_hover(true)
                        .test_id("sidebar-menu-action")
                        .into_element(cx);
                    vec![button, action]
                });
                let menu = SidebarMenu::new([item]).into_element(cx);
                vec![menu]
            },
        );
        ui.set_root(root);

        ui.request_semantics_snapshot();
        ui.layout_all(&mut app, &mut services, bounds, 1.0);
        let second = ui
            .semantics_snapshot()
            .cloned()
            .expect("expected semantics snapshot after hover");

        assert!(
            second
                .nodes
                .iter()
                .any(|n| n.test_id.as_deref() == Some("sidebar-menu-action")),
            "expected show_on_hover action to appear after hovering menu item"
        );
    }

    #[test]
    fn sidebar_menu_action_show_on_hover_visible_when_item_open_without_hover() {
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

        let root = fret_ui::declarative::render_root(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "shadcn-sidebar-menu-action-show-on-hover-open",
            |cx| {
                let button = SidebarMenuButton::new("Projects")
                    .test_id("sidebar-menu-button")
                    .into_element(cx);
                let action = SidebarMenuAction::new(Vec::<AnyElement>::new())
                    .show_on_hover(true)
                    .test_id("sidebar-menu-action")
                    .into_element(cx);
                let item = SidebarMenuItem::new(button.clone())
                    .open(true)
                    .children([button, action])
                    .test_id("sidebar-menu-item")
                    .into_element(cx);
                let menu = SidebarMenu::new([item]).into_element(cx);
                vec![menu]
            },
        );
        ui.set_root(root);
        ui.request_semantics_snapshot();
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        let snap = ui
            .semantics_snapshot()
            .cloned()
            .expect("expected semantics snapshot");

        assert!(
            snap.nodes
                .iter()
                .any(|n| n.test_id.as_deref() == Some("sidebar-menu-action")),
            "expected show_on_hover action to remain visible when menu item is open"
        );
    }

    #[test]
    fn sidebar_menu_action_show_on_hover_visible_when_menu_item_focus_within() {
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

        let button_element_id: Arc<std::sync::Mutex<Option<fret_ui::GlobalElementId>>> =
            Arc::new(std::sync::Mutex::new(None));
        let button_element_id_for_render = button_element_id.clone();
        let root = fret_ui::declarative::render_root(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "shadcn-sidebar-menu-action-show-on-hover-focus-within",
            |cx| {
                let item = SidebarMenuItem::new(cx.spacer(SpacerProps {
                    min: Px(0.0),
                    ..Default::default()
                }))
                .test_id("sidebar-menu-item")
                .into_element_with_children(cx, move |cx| {
                    let button = SidebarMenuButton::new("Projects")
                        .test_id("sidebar-menu-button")
                        .into_element(cx);
                    if use_sidebar_menu_item_context(cx).is_some() {
                        let mut guard = button_element_id_for_render
                            .lock()
                            .unwrap_or_else(|e| e.into_inner());
                        *guard = Some(button.id);
                    }

                    let action = SidebarMenuAction::new(Vec::<AnyElement>::new())
                        .show_on_hover(true)
                        .test_id("sidebar-menu-action")
                        .into_element(cx);
                    vec![button, action]
                });
                let menu = SidebarMenu::new([item]).into_element(cx);
                vec![menu]
            },
        );
        ui.set_root(root);
        ui.request_semantics_snapshot();
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        let first = ui
            .semantics_snapshot()
            .cloned()
            .expect("expected semantics snapshot");
        assert!(
            !first
                .nodes
                .iter()
                .any(|n| n.test_id.as_deref() == Some("sidebar-menu-action")),
            "expected show_on_hover action to be hidden before focus"
        );

        let button_element_id = button_element_id
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .clone()
            .expect("sidebar menu button element id");
        let button_node = elements::node_for_element(&mut app, window, button_element_id)
            .expect("sidebar menu button node id");
        ui.set_focus(Some(button_node));
        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::KeyDown {
                key: fret_core::KeyCode::Tab,
                modifiers: fret_core::Modifiers::default(),
                repeat: false,
            },
        );

        let root = fret_ui::declarative::render_root(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "shadcn-sidebar-menu-action-show-on-hover-focus-within",
            |cx| {
                let item = SidebarMenuItem::new(cx.spacer(SpacerProps {
                    min: Px(0.0),
                    ..Default::default()
                }))
                .test_id("sidebar-menu-item")
                .into_element_with_children(cx, |cx| {
                    let button = SidebarMenuButton::new("Projects")
                        .test_id("sidebar-menu-button")
                        .into_element(cx);
                    let action = SidebarMenuAction::new(Vec::<AnyElement>::new())
                        .show_on_hover(true)
                        .test_id("sidebar-menu-action")
                        .into_element(cx);
                    vec![button, action]
                });
                let menu = SidebarMenu::new([item]).into_element(cx);
                vec![menu]
            },
        );
        ui.set_root(root);
        ui.request_semantics_snapshot();
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        let second = ui
            .semantics_snapshot()
            .cloned()
            .expect("expected semantics snapshot after focus");
        assert!(
            second
                .nodes
                .iter()
                .any(|n| n.test_id.as_deref() == Some("sidebar-menu-action")),
            "expected show_on_hover action to be visible when menu item has focus-within"
        );
    }

    #[test]
    fn sidebar_menu_action_as_child_keeps_button_semantics_and_dispatches_activate() {
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

        let triggered = app.models_mut().insert(false);
        let triggered_for_handler = triggered.clone();

        let root = fret_ui::declarative::render_root(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "shadcn-sidebar-menu-action-as-child",
            |cx| {
                let button = SidebarMenuButton::new("Projects")
                    .test_id("sidebar-menu-button")
                    .into_element(cx);
                let on_activate: OnActivate = Arc::new(move |host, _acx, _reason| {
                    let _ = host
                        .models_mut()
                        .update(&triggered_for_handler, |v| *v = true);
                });
                let action = SidebarMenuAction::new([ui::text(cx, "...").into_element(cx)])
                    .as_child(true)
                    .on_activate(on_activate)
                    .test_id("sidebar-menu-action")
                    .into_element(cx);
                let item = SidebarMenuItem::new(button.clone())
                    .children([button, action])
                    .into_element(cx);
                let menu = SidebarMenu::new([item]).into_element(cx);
                vec![menu]
            },
        );
        ui.set_root(root);
        ui.request_semantics_snapshot();
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        let snap = ui
            .semantics_snapshot()
            .cloned()
            .expect("expected semantics snapshot");
        let action = snap
            .nodes
            .iter()
            .find(|n| n.test_id.as_deref() == Some("sidebar-menu-action"))
            .expect("expected sidebar menu action semantics node");
        assert_eq!(
            action.role,
            SemanticsRole::Button,
            "expected sidebar menu action as_child path to keep button semantics"
        );

        let center = Point::new(
            Px(action.bounds.origin.x.0 + action.bounds.size.width.0 * 0.5),
            Px(action.bounds.origin.y.0 + action.bounds.size.height.0 * 0.5),
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

        let triggered = app
            .models()
            .get_copied(&triggered)
            .expect("triggered model");
        assert!(
            triggered,
            "expected sidebar menu action as_child path to dispatch activate handler"
        );
    }

    #[test]
    fn sidebar_menu_sub_button_href_path_uses_link_semantics_and_dispatches_navigation() {
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

        let navigated = app.models_mut().insert(false);
        let navigated_for_handler = navigated.clone();
        let root = fret_ui::declarative::render_root(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "shadcn-sidebar-menu-sub-button-href",
            |cx| {
                let on_navigate: OnActivate = Arc::new(move |host, _acx, _reason| {
                    let _ = host
                        .models_mut()
                        .update(&navigated_for_handler, |v| *v = true);
                });
                let button = SidebarMenuSubButton::new("Docs")
                    .href("/docs")
                    .on_navigate(on_navigate)
                    .test_id("sidebar-menu-sub-button")
                    .into_element(cx);
                vec![button]
            },
        );
        ui.set_root(root);
        ui.request_semantics_snapshot();
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        let snap = ui
            .semantics_snapshot()
            .cloned()
            .expect("expected semantics snapshot");
        let button = snap
            .nodes
            .iter()
            .find(|n| n.test_id.as_deref() == Some("sidebar-menu-sub-button"))
            .expect("expected sidebar menu sub button semantics node");
        assert_eq!(
            button.role,
            SemanticsRole::Link,
            "expected sidebar menu sub button href path to use link semantics"
        );
        assert_eq!(
            button.value.as_deref(),
            Some("/docs"),
            "expected sidebar menu sub button href path to expose href semantics value"
        );

        let center = Point::new(
            Px(button.bounds.origin.x.0 + button.bounds.size.width.0 * 0.5),
            Px(button.bounds.origin.y.0 + button.bounds.size.height.0 * 0.5),
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

        let navigated = app
            .models()
            .get_copied(&navigated)
            .expect("navigated model");
        assert!(
            navigated,
            "expected sidebar menu sub button href path to invoke navigation callback"
        );
    }

    #[test]
    fn sidebar_menu_button_href_path_uses_link_semantics_and_dispatches_navigation() {
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

        let navigated = app.models_mut().insert(false);
        let navigated_for_handler = navigated.clone();
        let root = fret_ui::declarative::render_root(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "shadcn-sidebar-menu-button-href",
            |cx| {
                let on_navigate: OnActivate = Arc::new(move |host, _acx, _reason| {
                    let _ = host
                        .models_mut()
                        .update(&navigated_for_handler, |v| *v = true);
                });
                let button = SidebarMenuButton::new("Docs")
                    .href("/docs")
                    .on_navigate(on_navigate)
                    .test_id("sidebar-menu-button")
                    .into_element(cx);
                vec![button]
            },
        );
        ui.set_root(root);
        ui.request_semantics_snapshot();
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        let snap = ui
            .semantics_snapshot()
            .cloned()
            .expect("expected semantics snapshot");
        let button = snap
            .nodes
            .iter()
            .find(|n| n.test_id.as_deref() == Some("sidebar-menu-button"))
            .expect("expected sidebar menu button semantics node");
        assert_eq!(
            button.role,
            SemanticsRole::Link,
            "expected sidebar menu button href path to use link semantics"
        );
        assert_eq!(
            button.value.as_deref(),
            Some("/docs"),
            "expected sidebar menu button href path to expose href semantics value"
        );

        let center = Point::new(
            Px(button.bounds.origin.x.0 + button.bounds.size.width.0 * 0.5),
            Px(button.bounds.origin.y.0 + button.bounds.size.height.0 * 0.5),
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

        let navigated = app
            .models()
            .get_copied(&navigated)
            .expect("navigated model");
        assert!(
            navigated,
            "expected sidebar menu button href path to invoke navigation callback"
        );
    }

    #[test]
    fn sidebar_menu_sub_button_href_without_on_navigate_emits_open_url_effect() {
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

        let root = fret_ui::declarative::render_root(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "shadcn-sidebar-menu-sub-button-href-open-url",
            |cx| {
                let button = SidebarMenuSubButton::new("Docs")
                    .href("https://example.com/docs")
                    .target("_blank")
                    .rel("noopener noreferrer")
                    .test_id("sidebar-menu-sub-button")
                    .into_element(cx);
                vec![button]
            },
        );
        ui.set_root(root);
        ui.request_semantics_snapshot();
        ui.layout_all(&mut app, &mut services, bounds, 1.0);
        let _ = app.flush_effects();

        let snap = ui
            .semantics_snapshot()
            .cloned()
            .expect("expected semantics snapshot");
        let button = snap
            .nodes
            .iter()
            .find(|n| n.test_id.as_deref() == Some("sidebar-menu-sub-button"))
            .expect("expected sidebar menu sub button semantics node");
        assert_eq!(
            button.role,
            SemanticsRole::Link,
            "expected sidebar menu sub button href fallback path to keep link semantics"
        );
        assert_eq!(
            button.value.as_deref(),
            Some("https://example.com/docs"),
            "expected sidebar menu sub button href fallback path to expose href semantics value"
        );
        let center = Point::new(
            Px(button.bounds.origin.x.0 + button.bounds.size.width.0 * 0.5),
            Px(button.bounds.origin.y.0 + button.bounds.size.height.0 * 0.5),
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

        let effects = app.flush_effects();
        assert!(
            effects.iter().any(|effect| {
                matches!(
                    effect,
                    Effect::OpenUrl {
                        url,
                        target,
                        rel,
                    } if url == "https://example.com/docs"
                        && target.as_deref() == Some("_blank")
                        && rel.as_deref() == Some("noopener noreferrer")
                )
            }),
            "expected sidebar menu sub button href fallback to emit Effect::OpenUrl with target/rel"
        );
    }

    #[test]
    fn sidebar_menu_button_href_without_on_navigate_emits_open_url_effect() {
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

        let root = fret_ui::declarative::render_root(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "shadcn-sidebar-menu-button-href-open-url",
            |cx| {
                let button = SidebarMenuButton::new("Docs")
                    .href("https://example.com/docs")
                    .target("_blank")
                    .rel("noopener noreferrer")
                    .test_id("sidebar-menu-button")
                    .into_element(cx);
                vec![button]
            },
        );
        ui.set_root(root);
        ui.request_semantics_snapshot();
        ui.layout_all(&mut app, &mut services, bounds, 1.0);
        let _ = app.flush_effects();

        let snap = ui
            .semantics_snapshot()
            .cloned()
            .expect("expected semantics snapshot");
        let button = snap
            .nodes
            .iter()
            .find(|n| n.test_id.as_deref() == Some("sidebar-menu-button"))
            .expect("expected sidebar menu button semantics node");
        assert_eq!(
            button.role,
            SemanticsRole::Link,
            "expected sidebar menu button href fallback path to keep link semantics"
        );
        assert_eq!(
            button.value.as_deref(),
            Some("https://example.com/docs"),
            "expected sidebar menu button href fallback path to expose href semantics value"
        );
        let center = Point::new(
            Px(button.bounds.origin.x.0 + button.bounds.size.width.0 * 0.5),
            Px(button.bounds.origin.y.0 + button.bounds.size.height.0 * 0.5),
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

        let effects = app.flush_effects();
        assert!(
            effects.iter().any(|effect| {
                matches!(
                    effect,
                    Effect::OpenUrl {
                        url,
                        target,
                        rel,
                    } if url == "https://example.com/docs"
                        && target.as_deref() == Some("_blank")
                        && rel.as_deref() == Some("noopener noreferrer")
                )
            }),
            "expected sidebar menu button href fallback to emit Effect::OpenUrl with target/rel"
        );
    }

    #[test]
    fn sidebar_menu_button_outline_variant_adds_chrome_while_default_remains_plain() {
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

        let render_case = |ui: &mut UiTree<App>,
                           app: &mut App,
                           services: &mut FakeServices,
                           variant: SidebarMenuButtonVariant,
                           test_id: &'static str|
         -> Rect {
            let root = fret_ui::declarative::render_root(
                ui,
                app,
                services,
                window,
                bounds,
                "shadcn-sidebar-menu-button-variant",
                |cx| {
                    let button = SidebarMenuButton::new("Projects")
                        .variant(variant)
                        .test_id(test_id)
                        .into_element(cx);
                    vec![button]
                },
            );
            ui.set_root(root);
            ui.request_semantics_snapshot();
            ui.layout_all(app, services, bounds, 1.0);

            let snap = ui
                .semantics_snapshot()
                .cloned()
                .expect("expected semantics snapshot");
            snap.nodes
                .iter()
                .find(|n| n.test_id.as_deref() == Some(test_id))
                .map(|n| n.bounds)
                .expect("expected sidebar menu button semantics node")
        };

        let default_bounds = render_case(
            &mut ui,
            &mut app,
            &mut services,
            SidebarMenuButtonVariant::Default,
            "sidebar-menu-button-default",
        );
        let outline_bounds = render_case(
            &mut ui,
            &mut app,
            &mut services,
            SidebarMenuButtonVariant::Outline,
            "sidebar-menu-button-outline",
        );

        assert!(
            outline_bounds.size.height.0 >= default_bounds.size.height.0 - 1.0,
            "expected outline sidebar menu button to preserve baseline geometry"
        );
    }

    #[test]
    fn sidebar_menu_skeleton_hides_in_collapsed_state() {
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

        let open_model = app.models_mut().insert(false);

        let root = fret_ui::declarative::render_root(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "shadcn-sidebar-menu-skeleton-collapsed",
            |cx| {
                SidebarProvider::new()
                    .open(Some(open_model.clone()))
                    .with(cx, |cx| {
                        let skeleton = SidebarMenuSkeleton::new()
                            .show_icon(true)
                            .test_id("sidebar-menu-skeleton")
                            .into_element(cx);
                        vec![skeleton]
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
        assert!(
            !snap
                .nodes
                .iter()
                .any(|n| n.test_id.as_deref() == Some("sidebar-menu-skeleton")),
            "expected sidebar menu skeleton to be hidden when collapsed"
        );
    }

    #[test]
    fn sidebar_menu_sub_surfaces_expose_list_semantics() {
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

        let root = fret_ui::declarative::render_root(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "shadcn-sidebar-menu-sub-semantics",
            |cx| {
                let sub_button = SidebarMenuSubButton::new("Child")
                    .test_id("sidebar-menu-sub-button")
                    .into_element(cx);
                let sub_item = SidebarMenuSubItem::new(sub_button.clone())
                    .children([sub_button])
                    .test_id("sidebar-menu-sub-item")
                    .into_element(cx);
                let sub_menu = SidebarMenuSub::new([sub_item]).into_element(cx);
                vec![sub_menu]
            },
        );
        ui.set_root(root);
        ui.request_semantics_snapshot();
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        let snap = ui
            .semantics_snapshot()
            .cloned()
            .expect("expected semantics snapshot");

        assert!(
            snap.nodes.iter().any(|n| n.role == SemanticsRole::List),
            "expected sidebar menu sub to expose list semantics"
        );
        let item = snap
            .nodes
            .iter()
            .find(|n| n.test_id.as_deref() == Some("sidebar-menu-sub-item"))
            .expect("expected sidebar menu sub item semantics node");
        assert_eq!(
            item.role,
            SemanticsRole::ListItem,
            "expected sidebar menu sub item to expose list item semantics"
        );
    }

    #[test]
    fn sidebar_menu_sub_button_sizes_match_shadcn_row_height() {
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

        for size in [SidebarMenuSubButtonSize::Sm, SidebarMenuSubButtonSize::Md] {
            let root = fret_ui::declarative::render_root(
                &mut ui,
                &mut app,
                &mut services,
                window,
                bounds,
                "shadcn-sidebar-menu-sub-button-size",
                |cx| {
                    let button = SidebarMenuSubButton::new("Child")
                        .size(size)
                        .test_id("sidebar-menu-sub-button")
                        .into_element(cx);
                    vec![button]
                },
            );
            ui.set_root(root);
            ui.request_semantics_snapshot();
            ui.layout_all(&mut app, &mut services, bounds, 1.0);

            let snap = ui
                .semantics_snapshot()
                .cloned()
                .expect("expected semantics snapshot");
            let button = snap
                .nodes
                .iter()
                .find(|n| n.test_id.as_deref() == Some("sidebar-menu-sub-button"))
                .expect("expected sidebar menu sub button semantics node");
            assert!(
                (button.bounds.size.height.0 - 28.0).abs() <= 1.0,
                "expected sidebar menu sub button height ~28px for {:?}, got {}",
                size,
                button.bounds.size.height.0
            );
        }
    }

    #[test]
    fn sidebar_menu_sub_button_as_child_renders_custom_children() {
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

        let root = fret_ui::declarative::render_root(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "shadcn-sidebar-menu-sub-button-as-child",
            |cx| {
                let child = ui::text(cx, "Custom Child")
                    .into_element(cx)
                    .test_id("sidebar-menu-sub-button-child");
                let button = SidebarMenuSubButton::new("Child")
                    .as_child(true)
                    .children([child])
                    .test_id("sidebar-menu-sub-button")
                    .into_element(cx);
                vec![button]
            },
        );
        ui.set_root(root);
        ui.request_semantics_snapshot();
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        let snap = ui
            .semantics_snapshot()
            .cloned()
            .expect("expected semantics snapshot");

        assert!(
            snap.nodes
                .iter()
                .any(|n| n.test_id.as_deref() == Some("sidebar-menu-sub-button-child")),
            "expected as_child sidebar menu sub button to render custom child"
        );
        let button = snap
            .nodes
            .iter()
            .find(|n| n.test_id.as_deref() == Some("sidebar-menu-sub-button"))
            .expect("expected sidebar menu sub button semantics node");
        assert_eq!(
            button.role,
            SemanticsRole::Button,
            "expected as_child sidebar menu sub button to retain button semantics"
        );
    }

    #[test]
    fn sidebar_menu_button_as_child_renders_custom_children() {
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

        let root = fret_ui::declarative::render_root(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "shadcn-sidebar-menu-button-as-child",
            |cx| {
                let child = ui::text(cx, "Custom Child")
                    .into_element(cx)
                    .test_id("sidebar-menu-button-child");
                let button = SidebarMenuButton::new("Projects")
                    .as_child(true)
                    .children([child])
                    .test_id("sidebar-menu-button")
                    .into_element(cx);
                vec![button]
            },
        );
        ui.set_root(root);
        ui.request_semantics_snapshot();
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        let snap = ui
            .semantics_snapshot()
            .cloned()
            .expect("expected semantics snapshot");

        assert!(
            snap.nodes
                .iter()
                .any(|n| n.test_id.as_deref() == Some("sidebar-menu-button-child")),
            "expected as_child sidebar menu button to render custom child"
        );
        let button = snap
            .nodes
            .iter()
            .find(|n| n.test_id.as_deref() == Some("sidebar-menu-button"))
            .expect("expected sidebar menu button semantics node");
        assert_eq!(
            button.role,
            SemanticsRole::Button,
            "expected as_child sidebar menu button to retain button semantics"
        );
    }

    #[test]
    fn sidebar_menu_sub_button_as_child_with_href_keeps_button_semantics() {
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

        let root = fret_ui::declarative::render_root(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "shadcn-sidebar-menu-sub-button-as-child-href",
            |cx| {
                let child = ui::text(cx, "Custom Child")
                    .into_element(cx)
                    .test_id("sidebar-menu-sub-button-child");
                let button = SidebarMenuSubButton::new("Child")
                    .as_child(true)
                    .href("https://example.com/docs")
                    .children([child])
                    .test_id("sidebar-menu-sub-button")
                    .into_element(cx);
                vec![button]
            },
        );
        ui.set_root(root);
        ui.request_semantics_snapshot();
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        let snap = ui
            .semantics_snapshot()
            .cloned()
            .expect("expected semantics snapshot");
        let button = snap
            .nodes
            .iter()
            .find(|n| n.test_id.as_deref() == Some("sidebar-menu-sub-button"))
            .expect("expected sidebar menu sub button semantics node");
        assert_eq!(
            button.role,
            SemanticsRole::Button,
            "expected as_child sidebar menu sub button href path to keep button semantics"
        );
        assert_eq!(
            button.value.as_deref(),
            None,
            "expected as_child sidebar menu sub button href path to avoid default href semantics value"
        );
    }

    #[test]
    fn sidebar_menu_button_as_child_with_href_keeps_button_semantics() {
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

        let root = fret_ui::declarative::render_root(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "shadcn-sidebar-menu-button-as-child-href",
            |cx| {
                let child = ui::text(cx, "Custom Child")
                    .into_element(cx)
                    .test_id("sidebar-menu-button-child");
                let button = SidebarMenuButton::new("Projects")
                    .as_child(true)
                    .href("https://example.com/docs")
                    .children([child])
                    .test_id("sidebar-menu-button")
                    .into_element(cx);
                vec![button]
            },
        );
        ui.set_root(root);
        ui.request_semantics_snapshot();
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        let snap = ui
            .semantics_snapshot()
            .cloned()
            .expect("expected semantics snapshot");
        let button = snap
            .nodes
            .iter()
            .find(|n| n.test_id.as_deref() == Some("sidebar-menu-button"))
            .expect("expected sidebar menu button semantics node");
        assert_eq!(
            button.role,
            SemanticsRole::Button,
            "expected as_child sidebar menu button href path to keep button semantics"
        );
        assert_eq!(
            button.value.as_deref(),
            None,
            "expected as_child sidebar menu button href path to avoid default href semantics value"
        );
    }

    #[test]
    fn sidebar_provider_on_open_change_builder_sets_handler() {
        let provider = SidebarProvider::new().on_open_change(Some(Arc::new(|_open| {})));

        assert!(provider.on_open_change.is_some());
    }

    #[test]
    fn sidebar_provider_on_open_mobile_change_builder_sets_handler() {
        let provider =
            SidebarProvider::new().on_open_mobile_change(Some(Arc::new(|_open_mobile| {})));

        assert!(provider.on_open_mobile_change.is_some());
    }

    #[test]
    fn sidebar_provider_open_change_events_emit_only_on_state_change() {
        let mut state = SidebarProviderOpenChangeCallbackState::default();

        let (open_changed, open_mobile_changed) =
            sidebar_provider_open_change_events(&mut state, false, false);
        assert_eq!(open_changed, None);
        assert_eq!(open_mobile_changed, None);

        let (open_changed, open_mobile_changed) =
            sidebar_provider_open_change_events(&mut state, true, false);
        assert_eq!(open_changed, Some(true));
        assert_eq!(open_mobile_changed, None);

        let (open_changed, open_mobile_changed) =
            sidebar_provider_open_change_events(&mut state, true, false);
        assert_eq!(open_changed, None);
        assert_eq!(open_mobile_changed, None);

        let (open_changed, open_mobile_changed) =
            sidebar_provider_open_change_events(&mut state, true, true);
        assert_eq!(open_changed, None);
        assert_eq!(open_mobile_changed, Some(true));

        let (open_changed, open_mobile_changed) =
            sidebar_provider_open_change_events(&mut state, false, false);
        assert_eq!(open_changed, Some(false));
        assert_eq!(open_mobile_changed, Some(false));
    }

    #[test]
    fn sidebar_provider_open_change_callbacks_follow_model_changes() {
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

        let open_model = app.models_mut().insert(false);
        let open_mobile_model = app.models_mut().insert(false);

        let open_events: Arc<Mutex<Vec<bool>>> = Arc::new(Mutex::new(Vec::new()));
        let open_mobile_events: Arc<Mutex<Vec<bool>>> = Arc::new(Mutex::new(Vec::new()));

        let render_frame = |ui: &mut UiTree<App>, app: &mut App, services: &mut FakeServices| {
            let root = fret_ui::declarative::render_root(
                ui,
                app,
                services,
                window,
                bounds,
                "shadcn-sidebar-provider-open-change-callbacks",
                |cx| {
                    let open_events = Arc::clone(&open_events);
                    let open_mobile_events = Arc::clone(&open_mobile_events);
                    SidebarProvider::new()
                        .open(Some(open_model.clone()))
                        .open_mobile(Some(open_mobile_model.clone()))
                        .on_open_change(Some(Arc::new(move |open| {
                            open_events.lock().expect("open events lock").push(open);
                        })))
                        .on_open_mobile_change(Some(Arc::new(move |open_mobile| {
                            open_mobile_events
                                .lock()
                                .expect("open_mobile events lock")
                                .push(open_mobile);
                        })))
                        .with(cx, |_cx| Vec::<AnyElement>::new())
                },
            );
            ui.set_root(root);
            ui.layout_all(app, services, bounds, 1.0);
        };

        render_frame(&mut ui, &mut app, &mut services);
        assert!(
            open_events.lock().expect("open events lock").is_empty(),
            "expected initial render to not emit open callback"
        );
        assert!(
            open_mobile_events
                .lock()
                .expect("open_mobile events lock")
                .is_empty(),
            "expected initial render to not emit open_mobile callback"
        );

        let _ = app.models_mut().update(&open_model, |value| {
            *value = true;
        });
        render_frame(&mut ui, &mut app, &mut services);
        assert_eq!(
            open_events.lock().expect("open events lock").as_slice(),
            [true],
            "expected open callback to emit when open model changes"
        );
        assert!(
            open_mobile_events
                .lock()
                .expect("open_mobile events lock")
                .is_empty(),
            "expected open_mobile callback to stay silent when open_mobile unchanged"
        );

        render_frame(&mut ui, &mut app, &mut services);
        assert_eq!(
            open_events.lock().expect("open events lock").as_slice(),
            [true],
            "expected unchanged open state to avoid duplicate callback"
        );

        let _ = app.models_mut().update(&open_mobile_model, |value| {
            *value = true;
        });
        render_frame(&mut ui, &mut app, &mut services);
        assert_eq!(
            open_mobile_events
                .lock()
                .expect("open_mobile events lock")
                .as_slice(),
            [true],
            "expected open_mobile callback to emit when open_mobile model changes"
        );

        let _ = app.models_mut().update(&open_model, |value| {
            *value = false;
        });
        let _ = app.models_mut().update(&open_mobile_model, |value| {
            *value = false;
        });
        render_frame(&mut ui, &mut app, &mut services);

        assert_eq!(
            open_events.lock().expect("open events lock").as_slice(),
            [true, false],
            "expected open callback to track both transitions"
        );
        assert_eq!(
            open_mobile_events
                .lock()
                .expect("open_mobile events lock")
                .as_slice(),
            [true, false],
            "expected open_mobile callback to track both transitions"
        );
    }

    #[test]
    fn sidebar_context_set_open_and_set_open_mobile_update_models() {
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

        let open_model = app.models_mut().insert(false);
        let open_mobile_model = app.models_mut().insert(false);
        let open_for_assert = open_model.clone();
        let open_mobile_for_assert = open_mobile_model.clone();

        let root = fret_ui::declarative::render_root(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "shadcn-sidebar-context-set-open",
            |cx| {
                SidebarProvider::new()
                    .open(Some(open_model.clone()))
                    .open_mobile(Some(open_mobile_model.clone()))
                    .with(cx, |cx| {
                        if let Some(ctx) = use_sidebar(cx) {
                            ctx.set_open(cx.app, true);
                            ctx.set_open_mobile(cx.app, true);
                        }
                        Vec::<AnyElement>::new()
                    })
            },
        );
        ui.set_root(root);
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        let open_now = app
            .models()
            .get_copied(&open_for_assert)
            .expect("open model value");
        let open_mobile_now = app
            .models()
            .get_copied(&open_mobile_for_assert)
            .expect("open mobile model value");

        assert!(open_now, "expected ctx.set_open(true) to update open model");
        assert!(
            open_mobile_now,
            "expected ctx.set_open_mobile(true) to update open_mobile model"
        );
    }

    #[test]
    fn sidebar_context_function_style_setters_update_from_previous_value() {
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

        let open_model = app.models_mut().insert(false);
        let open_mobile_model = app.models_mut().insert(false);
        let open_for_assert = open_model.clone();
        let open_mobile_for_assert = open_mobile_model.clone();

        let root = fret_ui::declarative::render_root(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "shadcn-sidebar-context-function-style-setters",
            |cx| {
                SidebarProvider::new()
                    .open(Some(open_model.clone()))
                    .open_mobile(Some(open_mobile_model.clone()))
                    .with(cx, |cx| {
                        if let Some(ctx) = use_sidebar(cx) {
                            ctx.set_open_with(cx.app, |value| !value);
                            ctx.set_open_mobile_with(cx.app, |value| !value);
                        }
                        Vec::<AnyElement>::new()
                    })
            },
        );
        ui.set_root(root);
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        let open_now = app
            .models()
            .get_copied(&open_for_assert)
            .expect("open model value");
        let open_mobile_now = app
            .models()
            .get_copied(&open_mobile_for_assert)
            .expect("open mobile model value");

        assert!(
            open_now,
            "expected ctx.set_open_with(|prev| !prev) to update open model"
        );
        assert!(
            open_mobile_now,
            "expected ctx.set_open_mobile_with(|prev| !prev) to update open_mobile model"
        );
    }

    #[test]
    fn sidebar_provider_handles_sidebar_toggle_command_in_focus_subtree() {
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
        let open_for_assert = open_model.clone();

        let root = fret_ui::declarative::render_root(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "shadcn-sidebar-toggle-command",
            |cx| {
                SidebarProvider::new()
                    .open(Some(open_model.clone()))
                    .with(cx, |cx| {
                        let mut trigger_props = PressableProps::default();
                        trigger_props.focusable = true;
                        trigger_props.a11y.test_id =
                            Some(Arc::from("sidebar-toggle-command-focus"));
                        let focus_target =
                            cx.pressable(trigger_props, |_cx, _st| Vec::<AnyElement>::new());
                        vec![focus_target]
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
        let focus_node = snap
            .nodes
            .iter()
            .find(|n| n.test_id.as_deref() == Some("sidebar-toggle-command-focus"))
            .expect("expected focus target semantics node");
        ui.set_focus(Some(focus_node.id));

        let handled = ui.dispatch_command(&mut app, &mut services, &sidebar_toggle_command_id());
        assert!(
            handled,
            "expected sidebar.toggle command to be handled in provider subtree"
        );

        let open_now = app
            .models()
            .get_copied(&open_for_assert)
            .expect("open model value");
        assert!(
            !open_now,
            "expected sidebar.toggle command to flip open model to false"
        );
    }

    #[test]
    fn sidebar_provider_registers_ctrl_or_meta_b_shortcut_binding() {
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

        let root = fret_ui::declarative::render_root(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "shadcn-sidebar-shortcut-binding",
            |cx| SidebarProvider::new().with(cx, |_cx| Vec::<AnyElement>::new()),
        );
        ui.set_root(root);
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        let keymap_service = app
            .global::<KeymapService>()
            .expect("expected keymap service after sidebar provider install");

        let ctrl_binding = keymap_service.keymap.resolve(
            &fret_runtime::InputContext::default(),
            KeyChord::new(
                SIDEBAR_TOGGLE_SHORTCUT_KEY,
                Modifiers {
                    ctrl: true,
                    ..Modifiers::default()
                },
            ),
        );
        let mac_binding = keymap_service.keymap.resolve(
            &fret_runtime::InputContext {
                platform: fret_runtime::Platform::Macos,
                ..Default::default()
            },
            KeyChord::new(
                SIDEBAR_TOGGLE_SHORTCUT_KEY,
                Modifiers {
                    meta: true,
                    ..Modifiers::default()
                },
            ),
        );
        let command_registered = app.commands().get(sidebar_toggle_command_id()).is_some();

        assert_eq!(
            ctrl_binding,
            Some(sidebar_toggle_command_id()),
            "expected ctrl+b to resolve sidebar.toggle"
        );
        assert_eq!(
            mac_binding,
            Some(sidebar_toggle_command_id()),
            "expected cmd+b to resolve sidebar.toggle on mac platform"
        );
        assert!(
            !command_registered,
            "expected sidebar provider to avoid mutating global command registry"
        );
    }
}
