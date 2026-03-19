use std::sync::{Arc, Mutex};
use std::time::Duration;

use crate::optional_text_value_model::IntoOptionalTextValueModel;
use fret_core::Transform2D;
use fret_core::{
    Color, Corners, Edges, FontId, FontWeight, Point, PointerType, Px, Rect, SemanticsRole,
    TextStyle,
};
use fret_icons::ids;
use fret_runtime::{
    CommandId, Effect, InputContext, InputDispatchPhase, Model, Platform, PlatformCapabilities,
    WindowCommandGatingService, WindowCommandGatingSnapshot,
};
use fret_ui::action::OnActivate;
use fret_ui::element::{
    AnyElement, ContainerProps, ElementKind, Elements, FlexProps, LayoutQueryRegionProps,
    LayoutStyle, Length, MainAlign, PointerRegionProps, PressableA11y, PressableKeyActivation,
    PressableProps, RenderTransformProps, SemanticsDecoration, SizeStyle, StackProps,
    VisualTransformProps,
};
use fret_ui::overlay_placement::{Align, Side};
use fret_ui::{ElementContext, GlobalElementId, Invalidation, Theme, ThemeSnapshot, UiHost};
use fret_ui_kit::declarative::action_hooks::ActionHooksExt as _;
use fret_ui_kit::declarative::icon as decl_icon;
use fret_ui_kit::declarative::model_watch::ModelWatchExt as _;
use fret_ui_kit::declarative::style as decl_style;
use fret_ui_kit::declarative::transition::{
    drive_transition_with_durations_and_cubic_bezier, ticks_60hz_for_duration,
};
use fret_ui_kit::headless::safe_hover;
use fret_ui_kit::primitives::direction as direction_prim;
use fret_ui_kit::primitives::navigation_menu as radix_navigation_menu;
use fret_ui_kit::primitives::roving_focus_group;
use fret_ui_kit::primitives::{popper, popper_content};
use fret_ui_kit::theme_tokens;
use fret_ui_kit::typography;
use fret_ui_kit::{
    ChromeRefinement, ColorRef, LayoutRefinement, MetricRef, OverlayController, OverlayPresence,
    OverrideSlot, Space, WidgetState, WidgetStateProperty, WidgetStates, resolve_override_slot,
    resolve_override_slot_opt, ui,
};

use crate::overlay_motion;
use crate::rtl;

fn drive_navigation_menu_trigger_chevron_motion<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    key: Arc<str>,
    open: bool,
) -> fret_ui_kit::headless::transition::TransitionOutput {
    cx.keyed(("navigation-menu-trigger-chevron-motion", key), |cx| {
        let theme_full = Theme::global(&*cx.app);
        let duration = theme_full
            .duration_ms_by_key("duration.shadcn.motion.navigation_menu.trigger_chevron")
            .or_else(|| {
                theme_full.duration_ms_by_key("duration.motion.navigation_menu.trigger_chevron")
            })
            .or_else(|| theme_full.duration_ms_by_key("duration.shadcn.motion.300"))
            .map(|ms| Duration::from_millis(ms as u64))
            .unwrap_or(Duration::from_millis(300));
        let ticks = ticks_60hz_for_duration(duration);
        let easing = theme_full
            .easing_by_key("easing.shadcn.motion.navigation_menu.trigger_chevron")
            .or_else(|| theme_full.easing_by_key("easing.motion.navigation_menu.trigger_chevron"))
            .or_else(|| theme_full.easing_by_key("easing.shadcn.motion"))
            .or_else(|| theme_full.easing_by_key("easing.motion.standard"))
            .unwrap_or_else(|| overlay_motion::shadcn_motion_ease_bezier(cx));

        drive_transition_with_durations_and_cubic_bezier(cx, open, ticks, ticks, easing)
    })
}

fn navigation_menu_input_context<H: UiHost>(app: &H) -> InputContext {
    let caps = app
        .global::<PlatformCapabilities>()
        .cloned()
        .unwrap_or_default();
    InputContext {
        platform: Platform::current(),
        caps,
        ui_has_modal: false,
        window_arbitration: None,
        focus_is_text_input: false,
        text_boundary_mode: fret_runtime::TextBoundaryMode::UnicodeWord,
        edit_can_undo: true,
        edit_can_redo: true,
        router_can_back: false,
        router_can_forward: false,
        dispatch_phase: InputDispatchPhase::Bubble,
    }
}

fn command_is_disabled_by_gating<H: UiHost>(
    app: &H,
    gating: &WindowCommandGatingSnapshot,
    command: Option<&CommandId>,
) -> bool {
    command
        .and_then(|id| app.commands().get(id.clone()).map(|meta| (id, meta)))
        .is_some_and(|(id, meta)| !gating.is_enabled_for_command(id, meta))
}

fn open_url_on_activate(
    url: Arc<str>,
    target: Option<Arc<str>>,
    rel: Option<Arc<str>>,
) -> OnActivate {
    Arc::new(move |host, _acx, _reason| {
        host.push_effect(Effect::OpenUrl {
            url: url.to_string(),
            target: target.as_ref().map(|value| value.to_string()),
            rel: rel.as_ref().map(|value| value.to_string()),
        });
    })
}

fn nav_menu_trigger_text_style(theme: &ThemeSnapshot) -> TextStyle {
    let px = theme
        .metric_by_key("component.navigation_menu.trigger.text_px")
        .or_else(|| theme.metric_by_key(theme_tokens::metric::COMPONENT_TEXT_SM_PX))
        .or_else(|| theme.metric_by_key("metric.font.size"))
        .or_else(|| theme.metric_by_key("font.size"))
        .unwrap_or_else(|| theme.metric_token(theme_tokens::metric::COMPONENT_TEXT_SM_PX));
    let line_height = theme
        .metric_by_key("component.navigation_menu.trigger.line_height")
        // Upstream base-maia trigger uses `h-9` with `py-2.5`, which implies a smaller fixed
        // line box than Tailwind's default `text-sm` line height.
        .unwrap_or(Px(16.0));
    let mut style = typography::fixed_line_box_style(FontId::ui(), px, line_height);
    style.weight = FontWeight::MEDIUM;
    style
}

fn nav_menu_link_text_style(theme: &ThemeSnapshot) -> TextStyle {
    let px = theme
        .metric_by_key("component.navigation_menu.link.text_px")
        .or_else(|| theme.metric_by_key(theme_tokens::metric::COMPONENT_TEXT_SM_PX))
        .or_else(|| theme.metric_by_key("metric.font.size"))
        .or_else(|| theme.metric_by_key("font.size"))
        .unwrap_or_else(|| theme.metric_token(theme_tokens::metric::COMPONENT_TEXT_SM_PX));
    let line_height = theme
        .metric_by_key("component.navigation_menu.link.line_height")
        .or_else(|| theme.metric_by_key(theme_tokens::metric::COMPONENT_TEXT_SM_LINE_HEIGHT))
        .or_else(|| theme.metric_by_key("metric.font.line_height"))
        .or_else(|| theme.metric_by_key("font.line_height"))
        .unwrap_or_else(|| theme.metric_token(theme_tokens::metric::COMPONENT_TEXT_SM_LINE_HEIGHT));
    typography::fixed_line_box_style(FontId::ui(), px, line_height)
}

fn nav_menu_trigger_padding_x(theme: &ThemeSnapshot) -> Px {
    theme
        .metric_by_key("component.navigation_menu.trigger.pad_x")
        // Upstream: `px-4.5` (18px).
        .unwrap_or(Px(18.0))
}

const NAV_MENU_SAFE_CORRIDOR_BUFFER: Px = Px(5.0);

type OnOpenChange = Arc<dyn Fn(bool) + Send + Sync + 'static>;
type OnValueChange = Arc<dyn Fn(Option<Arc<str>>) + Send + Sync + 'static>;

/// Controls which query source drives the upstream Tailwind `md:*` breakpoint behavior.
///
/// Upstream shadcn/ui recipes use viewport breakpoints (`md:`) by default. In editor-grade layouts
/// (docking / resizable panels), it can be desirable to drive the same behavior from local
/// container width instead (ADR 0231).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum NavigationMenuMdBreakpointQuery {
    /// Match upstream Tailwind viewport breakpoint behavior (web parity).
    #[default]
    Viewport,
    /// Drive the breakpoint from a container query region (ADR 0231).
    Container,
}

#[derive(Default)]
struct NavigationMenuValueChangeCallbackState {
    initialized: bool,
    last_value: Option<Arc<str>>,
}

fn navigation_menu_value_change_event(
    state: &mut NavigationMenuValueChangeCallbackState,
    value: Option<Arc<str>>,
) -> Option<Option<Arc<str>>> {
    if !state.initialized {
        state.initialized = true;
        state.last_value = value;
        return None;
    }

    if state.last_value != value {
        state.last_value = value.clone();
        return Some(value);
    }

    None
}

#[derive(Default)]
struct NavigationMenuOpenChangeCallbackState {
    initialized: bool,
    last_open: bool,
    pending_complete: Option<bool>,
}

fn navigation_menu_open_change_complete_event(
    state: &mut NavigationMenuOpenChangeCallbackState,
    open: bool,
    present: bool,
    animating: bool,
) -> Option<bool> {
    if !state.initialized {
        state.initialized = true;
        state.last_open = open;
    } else if state.last_open != open {
        state.last_open = open;
        state.pending_complete = Some(open);
    }

    if state.pending_complete == Some(open) && present == open && !animating {
        state.pending_complete = None;
        return Some(open);
    }

    None
}

fn nav_menu_trigger_padding_y(theme: &ThemeSnapshot) -> Px {
    theme
        .metric_by_key("component.navigation_menu.trigger.pad_y")
        // Upstream: `py-2.5` (10px).
        .unwrap_or(Px(10.0))
}

fn nav_menu_trigger_space_px(theme: &ThemeSnapshot) -> Px {
    theme
        .metric_by_key("component.navigation_menu.trigger.space_px")
        .unwrap_or(Px(3.92))
}

fn nav_menu_trigger_radius(theme: &ThemeSnapshot) -> Px {
    theme
        .metric_by_key("component.navigation_menu.trigger.radius")
        // Upstream: `rounded-2xl` (16px).
        .unwrap_or(Px(16.0))
}

fn nav_menu_trigger_bg_hover(theme: &ThemeSnapshot) -> Color {
    // Upstream: `hover:bg-muted focus:bg-muted`.
    theme.color_token("muted")
}

fn nav_menu_trigger_bg_open(theme: &ThemeSnapshot) -> Color {
    theme
        .color_by_key("component.navigation_menu.trigger.bg_open")
        // The current new-york-v4 trigger uses an opaque accent surface for the open state.
        .unwrap_or_else(|| theme.color_token("accent"))
}

fn nav_menu_trigger_fg(theme: &ThemeSnapshot) -> Color {
    theme.color_token("foreground")
}

fn nav_menu_trigger_fg_muted(theme: &ThemeSnapshot) -> Color {
    theme.color_token("muted-foreground")
}

#[derive(Debug, Clone)]
pub struct NavigationMenuTriggerStyle {
    pub chrome: ChromeRefinement,
    pub layout: LayoutRefinement,
}

/// A shadcn/ui-aligned helper that mirrors `navigationMenuTriggerStyle()`.
///
/// Upstream returns a Tailwind/CVA class string. In Fret we return mergeable refinements.
pub fn navigation_menu_trigger_style(_theme: &ThemeSnapshot) -> NavigationMenuTriggerStyle {
    // Upstream base: `inline-flex h-9 w-max items-center justify-center ...`.
    // Interaction states (hover/open) are applied by the recipe.
    NavigationMenuTriggerStyle {
        chrome: ChromeRefinement::default(),
        layout: LayoutRefinement::default().h_px(Px(36.0)).flex_shrink_0(),
    }
}

fn nav_menu_viewport_bg(theme: &ThemeSnapshot) -> Color {
    theme.color_token("popover")
}

fn nav_menu_viewport_border(theme: &ThemeSnapshot) -> Color {
    theme.color_token("border")
}

fn nav_menu_viewport_side_offset(theme: &ThemeSnapshot) -> Px {
    theme
        .metric_by_key("component.navigation_menu.viewport.side_offset")
        // Upstream base-maia `NavigationMenuPositioner` uses `sideOffset=8`.
        .unwrap_or(Px(8.0))
}

fn nav_menu_viewport_window_margin(theme: &ThemeSnapshot) -> Px {
    theme
        .metric_by_key("component.navigation_menu.viewport.window_margin")
        .unwrap_or(Px(8.0))
}

fn nav_menu_content_switch_slide_px(theme: &ThemeSnapshot) -> Px {
    // Matches shadcn/ui's `slide-*-52` distance (13rem ≈ 208px).
    theme
        .metric_by_key("component.navigation_menu.content.switch_slide_px")
        .unwrap_or(Px(208.0))
}

fn nav_menu_indicator_diamond_size(theme: &ThemeSnapshot) -> Px {
    theme
        .metric_by_key("component.navigation_menu.indicator.diamond_size")
        .unwrap_or(Px(8.0))
}

fn nav_menu_md_breakpoint<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    query: NavigationMenuMdBreakpointQuery,
    region_id: GlobalElementId,
) -> bool {
    match query {
        NavigationMenuMdBreakpointQuery::Viewport => {
            fret_ui_kit::declarative::viewport_width_at_least(
                cx,
                Invalidation::Layout,
                fret_ui_kit::declarative::viewport_tailwind::MD,
                fret_ui_kit::declarative::ViewportQueryHysteresis::default(),
            )
        }
        NavigationMenuMdBreakpointQuery::Container => {
            // Container queries are frame-lagged. When the region width is temporarily unknown
            // (e.g. in single-pass layout test harnesses), fall back to viewport behavior so we
            // avoid branching on a missing measurement.
            let default_when_unknown = cx.environment_viewport_width(Invalidation::Layout).0
                >= fret_ui_kit::declarative::container_queries::tailwind::MD.0;
            fret_ui_kit::declarative::container_width_at_least(
                cx,
                region_id,
                Invalidation::Layout,
                default_when_unknown,
                fret_ui_kit::declarative::container_queries::tailwind::MD,
                fret_ui_kit::declarative::ContainerQueryHysteresis::default(),
            )
        }
    }
}

/// shadcn/ui `NavigationMenuViewport` (v4).
///
/// In upstream shadcn, the root optionally mounts a viewport element as a sibling of the list.
/// In Fret, the viewport is currently rendered via an overlay root, but the enable/disable outcome
/// is the same.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NavigationMenuViewport {
    enabled: bool,
}

impl Default for NavigationMenuViewport {
    fn default() -> Self {
        Self { enabled: true }
    }
}

impl NavigationMenuViewport {
    pub fn enabled(enabled: bool) -> Self {
        Self { enabled }
    }

    pub fn is_enabled(self) -> bool {
        self.enabled
    }
}

/// shadcn/ui `NavigationMenuIndicator` (v4).
///
/// Upstream renders this as an opt-in child. Fret does not render the indicator by default; enable
/// it via [`NavigationMenu::indicator`] or [`NavigationMenu::indicator_component`] when needed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NavigationMenuIndicator {
    enabled: bool,
}

impl Default for NavigationMenuIndicator {
    fn default() -> Self {
        Self { enabled: true }
    }
}

impl NavigationMenuIndicator {
    pub fn enabled(enabled: bool) -> Self {
        Self { enabled }
    }

    pub fn is_enabled(self) -> bool {
        self.enabled
    }
}

/// shadcn/ui `NavigationMenuTrigger` (v4).
///
/// In the upstream DOM implementation this is an element; in Fret this is a "spec" that provides
/// trigger children for [`NavigationMenuItem`].
#[derive(Debug, Default)]
pub struct NavigationMenuTrigger {
    children: Vec<AnyElement>,
}

impl NavigationMenuTrigger {
    pub fn new(children: impl IntoIterator<Item = AnyElement>) -> Self {
        let children = children.into_iter().collect();
        Self { children }
    }

    pub fn child(child: AnyElement) -> Self {
        Self {
            children: vec![child],
        }
    }

    pub fn children(self) -> Elements {
        Elements::from(self.children)
    }
}

/// shadcn/ui `NavigationMenuLink` (v4).
///
/// In the upstream DOM implementation this is an element that participates in Radix's
/// root-dismiss-on-select behavior. Fret does not use implicit context objects, so this wrapper
/// requires the navigation menu `model` and closes it on selection (unless the click is modified
/// with Ctrl/Meta, matching Radix semantics).
pub struct NavigationMenuLink {
    model: Model<Option<Arc<str>>>,
    children: Vec<AnyElement>,
    label: Option<Arc<str>>,
    test_id: Option<Arc<str>>,
    command: Option<CommandId>,
    on_activate: Option<OnActivate>,
    href: Option<Arc<str>>,
    target: Option<Arc<str>>,
    rel: Option<Arc<str>>,
    disabled: bool,
    dismiss_on_ctrl_or_meta: bool,
    active: bool,
    chrome: ChromeRefinement,
    layout: LayoutRefinement,
}

impl std::fmt::Debug for NavigationMenuLink {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("NavigationMenuLink")
            .field("model", &"<model>")
            .field("children_len", &self.children.len())
            .field("label", &self.label)
            .field("test_id", &self.test_id)
            .field("command", &self.command)
            .field("on_activate", &self.on_activate.is_some())
            .field("href", &self.href)
            .field("target", &self.target)
            .field("rel", &self.rel)
            .field("disabled", &self.disabled)
            .field("dismiss_on_ctrl_or_meta", &self.dismiss_on_ctrl_or_meta)
            .field("active", &self.active)
            .field("chrome", &self.chrome)
            .field("layout", &self.layout)
            .finish()
    }
}

impl NavigationMenuLink {
    pub fn new(
        model: Model<Option<Arc<str>>>,
        children: impl IntoIterator<Item = AnyElement>,
    ) -> Self {
        let children = children.into_iter().collect();
        Self {
            model,
            children,
            label: None,
            test_id: None,
            command: None,
            on_activate: None,
            href: None,
            target: None,
            rel: None,
            disabled: false,
            dismiss_on_ctrl_or_meta: false,
            active: false,
            chrome: ChromeRefinement::default(),
            layout: LayoutRefinement::default(),
        }
    }

    pub fn child(model: Model<Option<Arc<str>>>, child: AnyElement) -> Self {
        Self::new(model, vec![child])
    }

    pub fn label(mut self, label: impl Into<Arc<str>>) -> Self {
        self.label = Some(label.into());
        self
    }

    pub fn test_id(mut self, test_id: impl Into<Arc<str>>) -> Self {
        self.test_id = Some(test_id.into());
        self
    }

    pub fn on_click(mut self, command: impl Into<CommandId>) -> Self {
        self.command = Some(command.into());
        self
    }

    /// Bind a stable action ID to this navigation menu link (action-first authoring).
    ///
    /// v1 compatibility: `ActionId` is `CommandId`-compatible (ADR 0307), so this dispatches
    /// through the existing command pipeline.
    pub fn action(mut self, action: impl Into<fret_runtime::ActionId>) -> Self {
        self.command = Some(action.into());
        self
    }

    pub fn on_activate(mut self, on_activate: OnActivate) -> Self {
        self.on_activate = Some(on_activate);
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

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// When `false` (default), activation with Ctrl/Meta pressed does not dismiss the menu.
    pub fn dismiss_on_ctrl_or_meta(mut self, dismiss_on_ctrl_or_meta: bool) -> Self {
        self.dismiss_on_ctrl_or_meta = dismiss_on_ctrl_or_meta;
        self
    }

    /// Marks the link as active (Radix `data-active=true` styling outcome).
    pub fn active(mut self, active: bool) -> Self {
        self.active = active;
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
        #[derive(Default)]
        struct ModifierState {
            suppress_dismiss_for_next_activate: bool,
        }

        let model = self.model.clone();
        let disabled_explicit = self.disabled;
        let command = self.command;
        let on_activate = self.on_activate.clone();
        let href_for_action = self.href.clone();
        let href_for_semantics = self.href.clone();
        let target = self.target.clone();
        let rel = self.rel.clone();
        let should_fallback_open_url = command.is_none() && on_activate.is_none();
        let fallback_open_url = if should_fallback_open_url {
            href_for_action
                .clone()
                .map(|href| open_url_on_activate(href, target.clone(), rel.clone()))
        } else {
            None
        };
        let label = self.label.clone();
        let test_id = self.test_id.clone();
        let children = self.children;
        let dismiss_on_ctrl_or_meta = self.dismiss_on_ctrl_or_meta;
        let active = self.active;
        let chrome = self.chrome;
        let layout = self.layout;

        let fallback_input_ctx = navigation_menu_input_context(&*cx.app);
        let gating = cx
            .app
            .global::<WindowCommandGatingService>()
            .and_then(|svc| svc.snapshot(cx.window))
            .cloned()
            .unwrap_or_else(|| {
                fret_runtime::snapshot_for_window_with_input_ctx_fallback(
                    &*cx.app,
                    cx.window,
                    fallback_input_ctx,
                )
            });
        let disabled =
            disabled_explicit || command_is_disabled_by_gating(&*cx.app, &gating, command.as_ref());

        fn apply_link_inherited_style(
            mut element: AnyElement,
            fg: Color,
            text_style: &TextStyle,
            icon_fg: Color,
            default_icon_color: Color,
        ) -> AnyElement {
            match &mut element.kind {
                ElementKind::Text(props) => {
                    if props.style.is_none() {
                        props.style = Some(text_style.clone());
                    }
                    if props.color.is_none() {
                        props.color = Some(fg);
                    }
                }
                ElementKind::StyledText(props) => {
                    if props.style.is_none() {
                        props.style = Some(text_style.clone());
                    }
                    if props.color.is_none() {
                        props.color = Some(fg);
                    }
                }
                ElementKind::SelectableText(props) => {
                    if props.style.is_none() {
                        props.style = Some(text_style.clone());
                    }
                    if props.color.is_none() {
                        props.color = Some(fg);
                    }
                }
                ElementKind::SvgIcon(props) => {
                    // Align shadcn: `[&_svg:not([class*='text-'])]:text-muted-foreground`.
                    //
                    // Heuristic:
                    // - `declarative::icon::icon(...)` defaults to `muted-foreground` and inherits
                    //   `currentColor` unless an explicit color was provided.
                    // - Older callsites may build an `SvgIcon` with the default white color.
                    //
                    // In `NavigationMenuLink`, default icons should remain muted even when the
                    // link foreground changes on hover/focus/active.
                    let is_default_white = props.color
                        == Color {
                            r: 1.0,
                            g: 1.0,
                            b: 1.0,
                            a: 1.0,
                        };
                    let is_default_muted_fg = props.color == default_icon_color;
                    if props.inherit_color && (is_default_white || is_default_muted_fg) {
                        props.inherit_color = false;
                        props.color = icon_fg;
                    }
                }
                _ => {}
            }

            element.children = element
                .children
                .into_iter()
                .map(|child| {
                    apply_link_inherited_style(child, fg, text_style, icon_fg, default_icon_color)
                })
                .collect();
            element
        }

        let mut element = cx.pressable_with_id_props(move |cx, st, link_id| {
            let modifier_state: Arc<std::sync::Mutex<ModifierState>> = cx.state_for(
                link_id,
                || Arc::new(std::sync::Mutex::new(ModifierState::default())),
                |s| s.clone(),
            );

            let modifier_state_for_pointer = modifier_state.clone();
            cx.pressable_add_on_pointer_down(Arc::new(move |_host, _cx, down| {
                use fret_ui::action::PressablePointerDownResult as R;

                let suppress =
                    (down.modifiers.ctrl || down.modifiers.meta) && !dismiss_on_ctrl_or_meta;
                let mut st = modifier_state_for_pointer
                    .lock()
                    .unwrap_or_else(|e| e.into_inner());
                st.suppress_dismiss_for_next_activate = suppress;
                R::Continue
            }));

            let modifier_state_for_activate = modifier_state.clone();
            let model_for_activate = model.clone();
            cx.pressable_add_on_activate(Arc::new(move |host, action_cx, _reason| {
                if disabled {
                    return;
                }

                if let Some(command) = command.as_ref() {
                    host.dispatch_command(Some(action_cx.window), command.clone());
                }

                if let Some(on_activate) = on_activate.clone() {
                    on_activate(host, action_cx, _reason);
                } else if let Some(on_activate) = fallback_open_url.clone() {
                    on_activate(host, action_cx, _reason);
                }

                let mut st = modifier_state_for_activate
                    .lock()
                    .unwrap_or_else(|e| e.into_inner());
                let suppress = st.suppress_dismiss_for_next_activate;
                st.suppress_dismiss_for_next_activate = false;
                if suppress {
                    return;
                }

                let _ = host.models_mut().update(&model_for_activate, |v| *v = None);
            }));

            let theme = Theme::global(&*cx.app).snapshot();
            let radius = nav_menu_link_radius(&theme);
            let ring = decl_style::focus_ring(&theme, radius);

            let hovered = st.hovered && !st.pressed;
            let focused = st.focused;
            let pressed = st.pressed;

            let muted = theme.color_token("muted");
            let default_fg = theme.color_token("foreground");

            let mut bg_active = muted;
            bg_active.a *= 0.5;

            let use_hover_chrome = hovered || focused || pressed;
            let bg = if use_hover_chrome {
                Some(muted)
            } else if active {
                Some(bg_active)
            } else {
                None
            };
            let fg = default_fg;

            let mut pressable = PressableProps::default();
            pressable.enabled = !disabled;
            pressable.focusable = !disabled;
            pressable.focus_ring = Some(ring);
            pressable.key_activation = PressableKeyActivation::EnterOnly;
            pressable.layout = decl_style::layout_style(
                &theme,
                LayoutRefinement::default().w_full().min_w_0().merge(layout),
            );
            pressable.a11y = PressableA11y {
                role: Some(SemanticsRole::Link),
                label: label.clone(),
                test_id: test_id.clone(),
                ..Default::default()
            };

            let icon_fg = theme.color_token("muted-foreground");
            let default_icon_color = theme
                .color_by_key("muted-foreground")
                .unwrap_or_else(|| theme.color_token("muted-foreground"));
            let text_style = nav_menu_link_text_style(&theme);

            let content = if children.len() <= 1 {
                children
            } else {
                vec![cx.flex(
                    FlexProps {
                        layout: LayoutStyle::default(),
                        // Upstream base-maia `NavigationMenuLink` is `flex items-center gap-1.5`.
                        direction: fret_core::Axis::Horizontal,
                        gap: MetricRef::space(Space::N1p5).resolve(&theme).into(),
                        padding: Edges::all(Px(0.0)).into(),
                        justify: MainAlign::Start,
                        align: fret_ui::element::CrossAlign::Center,
                        wrap: false,
                    },
                    move |_cx| children,
                )]
            };

            let styled: Vec<AnyElement> = content
                .into_iter()
                .map(|child| {
                    apply_link_inherited_style(child, fg, &text_style, icon_fg, default_icon_color)
                })
                .collect();

            let mut base_props = decl_style::container_props(
                &theme,
                ChromeRefinement::default()
                    // Upstream base-maia: `rounded-xl p-3`.
                    .radius(radius)
                    .p(Space::N3)
                    .merge(chrome.clone()),
                LayoutRefinement::default().w_full().min_w_0(),
            );
            base_props.background = bg;

            let content = cx.container(base_props, move |_cx| styled);
            let content = if disabled {
                cx.opacity(0.5, move |_cx| vec![content])
            } else {
                content
            };

            (pressable, vec![content])
        });

        if let Some(href) = href_for_semantics {
            element = element.attach_semantics(SemanticsDecoration::default().value(href));
        }

        element
    }
}

fn nav_menu_link_radius(theme: &ThemeSnapshot) -> Px {
    theme
        .metric_by_key("component.navigation_menu.link.radius")
        // Upstream base-maia: `rounded-xl` (12px).
        .unwrap_or(Px(12.0))
}

/// shadcn/ui `NavigationMenuContent` (v4).
///
/// In the upstream DOM implementation this is an element; in Fret this is a "spec" that provides
/// viewport content for [`NavigationMenuItem`].
#[derive(Debug, Default)]
pub struct NavigationMenuContent {
    children: Vec<AnyElement>,
}

impl NavigationMenuContent {
    pub fn new(children: impl IntoIterator<Item = AnyElement>) -> Self {
        let children = children.into_iter().collect();
        Self { children }
    }

    pub fn child(child: AnyElement) -> Self {
        Self {
            children: vec![child],
        }
    }

    pub fn children(self) -> Elements {
        Elements::from(self.children)
    }
}

pub struct NavigationMenuItem {
    value: Arc<str>,
    label: Arc<str>,
    content: Vec<AnyElement>,
    trigger: Option<Vec<AnyElement>>,
    trigger_test_id: Option<Arc<str>>,
    command: Option<CommandId>,
    on_activate: Option<OnActivate>,
    href: Option<Arc<str>>,
    target: Option<Arc<str>>,
    rel: Option<Arc<str>>,
    disabled: bool,
}

impl std::fmt::Debug for NavigationMenuItem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("NavigationMenuItem")
            .field("value", &self.value)
            .field("label", &self.label)
            .field("content_len", &self.content.len())
            .field("trigger_len", &self.trigger.as_ref().map(Vec::len))
            .field("trigger_test_id", &self.trigger_test_id)
            .field("command", &self.command)
            .field("on_activate", &self.on_activate.is_some())
            .field("href", &self.href)
            .field("target", &self.target)
            .field("rel", &self.rel)
            .field("disabled", &self.disabled)
            .finish()
    }
}

impl NavigationMenuItem {
    pub fn new(
        value: impl Into<Arc<str>>,
        label: impl Into<Arc<str>>,
        content: impl IntoIterator<Item = AnyElement>,
    ) -> Self {
        let content = content.into_iter().collect();
        Self {
            value: value.into(),
            label: label.into(),
            content,
            trigger: None,
            trigger_test_id: None,
            command: None,
            on_activate: None,
            href: None,
            target: None,
            rel: None,
            disabled: false,
        }
    }

    pub fn trigger_test_id(mut self, test_id: impl Into<Arc<str>>) -> Self {
        self.trigger_test_id = Some(test_id.into());
        self
    }

    pub fn on_click(mut self, command: impl Into<CommandId>) -> Self {
        self.command = Some(command.into());
        self
    }

    /// Bind a stable action ID to this navigation menu item trigger (action-first authoring).
    ///
    /// v1 compatibility: `ActionId` is `CommandId`-compatible (ADR 0307), so this dispatches
    /// through the existing command pipeline.
    pub fn action(mut self, action: impl Into<fret_runtime::ActionId>) -> Self {
        self.command = Some(action.into());
        self
    }

    pub fn on_activate(mut self, on_activate: OnActivate) -> Self {
        self.on_activate = Some(on_activate);
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

    pub fn trigger_children(mut self, children: impl IntoIterator<Item = AnyElement>) -> Self {
        self.trigger = Some(children.into_iter().collect());
        self
    }

    pub fn trigger(mut self, trigger: NavigationMenuTrigger) -> Self {
        self.trigger = Some(trigger.children().into_vec());
        self
    }

    pub fn trigger_child(mut self, child: AnyElement) -> Self {
        self.trigger = Some(vec![child]);
        self
    }

    pub fn content(mut self, content: NavigationMenuContent) -> Self {
        self.content = content.children().into_vec();
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
}

/// shadcn/ui `NavigationMenuList` (v4).
///
/// In the upstream DOM implementation this is a structural wrapper. In Fret it is a named
/// container for `NavigationMenuItem` specs so recipes read closer to shadcn docs.
#[derive(Debug, Default)]
pub struct NavigationMenuList {
    items: Vec<NavigationMenuItem>,
}

impl NavigationMenuList {
    pub fn new(items: impl IntoIterator<Item = NavigationMenuItem>) -> Self {
        Self {
            items: items.into_iter().collect(),
        }
    }

    pub fn items(mut self, items: impl IntoIterator<Item = NavigationMenuItem>) -> Self {
        self.items = items.into_iter().collect();
        self
    }

    pub fn into_items(self) -> Vec<NavigationMenuItem> {
        self.items
    }
}

#[derive(Debug, Clone, Default)]
pub struct NavigationMenuStyle {
    pub trigger_background: OverrideSlot<ColorRef>,
    pub trigger_foreground: OverrideSlot<ColorRef>,
}

impl NavigationMenuStyle {
    pub fn trigger_background(
        mut self,
        trigger_background: WidgetStateProperty<Option<ColorRef>>,
    ) -> Self {
        self.trigger_background = Some(trigger_background);
        self
    }

    pub fn trigger_foreground(
        mut self,
        trigger_foreground: WidgetStateProperty<Option<ColorRef>>,
    ) -> Self {
        self.trigger_foreground = Some(trigger_foreground);
        self
    }

    pub fn merged(mut self, other: Self) -> Self {
        if other.trigger_background.is_some() {
            self.trigger_background = other.trigger_background;
        }
        if other.trigger_foreground.is_some() {
            self.trigger_foreground = other.trigger_foreground;
        }
        self
    }
}

pub struct NavigationMenu {
    model: Option<Model<Option<Arc<str>>>>,
    default_value: Option<Arc<str>>,
    items: Vec<NavigationMenuItem>,
    disabled: bool,
    viewport: bool,
    indicator: bool,
    viewport_test_id: Option<Arc<str>>,
    md_breakpoint_query: NavigationMenuMdBreakpointQuery,
    /// Optional override for which query region drives responsive variants (ADR 0231).
    ///
    /// When unset, the component uses its own internal region wrapper. When set, the caller must
    /// ensure the id refers to a `LayoutQueryRegion`, otherwise changes will not participate in
    /// invalidation.
    query_region: Option<GlobalElementId>,
    chrome: ChromeRefinement,
    layout: LayoutRefinement,
    style: NavigationMenuStyle,
    config: radix_navigation_menu::NavigationMenuConfig,
    on_value_change: Option<OnValueChange>,
    on_open_change_complete: Option<OnOpenChange>,
}

impl std::fmt::Debug for NavigationMenu {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("NavigationMenu")
            .field("model", &"<model>")
            .field("items_len", &self.items.len())
            .field("disabled", &self.disabled)
            .field("viewport", &self.viewport)
            .field("md_breakpoint_query", &self.md_breakpoint_query)
            .field("chrome", &self.chrome)
            .field("layout", &self.layout)
            .field("config", &self.config)
            .field("on_value_change", &self.on_value_change.is_some())
            .field(
                "on_open_change_complete",
                &self.on_open_change_complete.is_some(),
            )
            .finish()
    }
}

impl NavigationMenu {
    pub fn new(model: impl IntoOptionalTextValueModel) -> Self {
        Self {
            model: Some(model.into_optional_text_value_model()),
            default_value: None,
            items: Vec::new(),
            disabled: false,
            viewport: true,
            indicator: true,
            viewport_test_id: None,
            md_breakpoint_query: NavigationMenuMdBreakpointQuery::Viewport,
            query_region: None,
            chrome: ChromeRefinement::default(),
            layout: LayoutRefinement::default(),
            style: NavigationMenuStyle::default(),
            config: radix_navigation_menu::NavigationMenuConfig::default(),
            on_value_change: None,
            on_open_change_complete: None,
        }
    }

    pub fn uncontrolled<T: Into<Arc<str>>>(default_value: Option<T>) -> Self {
        Self {
            model: None,
            default_value: default_value.map(Into::into),
            items: Vec::new(),
            disabled: false,
            viewport: true,
            indicator: true,
            viewport_test_id: None,
            md_breakpoint_query: NavigationMenuMdBreakpointQuery::Viewport,
            query_region: None,
            chrome: ChromeRefinement::default(),
            layout: LayoutRefinement::default(),
            style: NavigationMenuStyle::default(),
            config: radix_navigation_menu::NavigationMenuConfig::default(),
            on_value_change: None,
            on_open_change_complete: None,
        }
    }

    pub fn items(mut self, items: impl IntoIterator<Item = NavigationMenuItem>) -> Self {
        self.items = items.into_iter().collect();
        self
    }

    pub fn list(mut self, list: NavigationMenuList) -> Self {
        self.items = list.into_items();
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Overrides which query region drives responsive variants (ADR 0231).
    ///
    /// This is primarily intended for editor-grade layouts where the navigation menu lives inside
    /// a resizable panel/dock split and should adapt to the local container width.
    ///
    /// Note: this is only consulted when [`NavigationMenu::md_breakpoint_query`] is set to
    /// [`NavigationMenuMdBreakpointQuery::Container`].
    pub fn container_query_region(mut self, region: GlobalElementId) -> Self {
        self.query_region = Some(region);
        self
    }

    /// Controls which query source drives the upstream Tailwind `md:*` breakpoint behavior.
    ///
    /// Default: [`NavigationMenuMdBreakpointQuery::Viewport`] (web parity).
    pub fn md_breakpoint_query(mut self, query: NavigationMenuMdBreakpointQuery) -> Self {
        self.md_breakpoint_query = query;
        self
    }

    /// When `true` (default), content is presented through a shared "viewport" panel with
    /// size interpolation, matching shadcn/ui composition.
    ///
    /// When `false`, content uses its own measured size without viewport interpolation (closer to
    /// Radix's "no Viewport component mounted" behavior).
    pub fn viewport(mut self, viewport: bool) -> Self {
        self.viewport = viewport;
        self
    }

    pub fn viewport_component(mut self, viewport: NavigationMenuViewport) -> Self {
        self.viewport = viewport.is_enabled();
        self
    }

    /// When `false`, the indicator is not rendered.
    pub fn indicator(mut self, indicator: bool) -> Self {
        self.indicator = indicator;
        self
    }

    pub fn indicator_component(mut self, indicator: NavigationMenuIndicator) -> Self {
        self.indicator = indicator.is_enabled();
        self
    }

    /// Attaches a stable `test_id` to the viewport panel surface when it is mounted.
    ///
    /// This is intended for deterministic diagnostics (`fretboard diag`) and should not be relied
    /// on by production UX.
    pub fn viewport_test_id(mut self, test_id: impl Into<Arc<str>>) -> Self {
        self.viewport_test_id = Some(test_id.into());
        self
    }

    pub fn refine_chrome(mut self, chrome: ChromeRefinement) -> Self {
        self.chrome = self.chrome.merge(chrome);
        self
    }

    pub fn refine_layout(mut self, layout: LayoutRefinement) -> Self {
        self.layout = self.layout.merge(layout);
        self
    }

    pub fn style(mut self, style: NavigationMenuStyle) -> Self {
        self.style = self.style.merged(style);
        self
    }

    pub fn config(mut self, config: radix_navigation_menu::NavigationMenuConfig) -> Self {
        self.config = config;
        self
    }

    /// Sets the hover-open delay window (Base UI `delay`).
    pub fn delay_duration(mut self, duration: std::time::Duration) -> Self {
        self.config.delay_duration = duration;
        self
    }

    /// Sets the hover-open delay in milliseconds (Base UI `delay`).
    pub fn delay_ms(mut self, millis: u64) -> Self {
        self.config.delay_duration = std::time::Duration::from_millis(millis);
        self
    }

    /// Sets the delayed-close window (Base UI `closeDelay`).
    pub fn close_delay_duration(mut self, duration: std::time::Duration) -> Self {
        self.config.close_delay_duration = duration;
        self
    }

    /// Sets the delayed-close window in milliseconds (Base UI `closeDelay`).
    pub fn close_delay_ms(mut self, millis: u64) -> Self {
        self.config.close_delay_duration = std::time::Duration::from_millis(millis);
        self
    }

    /// Sets the skip-delay window used after close (Base UI `skipDelayDuration`).
    pub fn skip_delay_duration(mut self, duration: std::time::Duration) -> Self {
        self.config.skip_delay_duration = duration;
        self
    }

    /// Sets the skip-delay window in milliseconds (Base UI `skipDelayDuration`).
    pub fn skip_delay_ms(mut self, millis: u64) -> Self {
        self.config.skip_delay_duration = std::time::Duration::from_millis(millis);
        self
    }

    /// Called when selected value changes (Base UI `onValueChange`).
    pub fn on_value_change(mut self, on_value_change: Option<OnValueChange>) -> Self {
        self.on_value_change = on_value_change;
        self
    }

    /// Called when open/close transition settles (Base UI `onOpenChangeComplete`).
    pub fn on_open_change_complete(
        mut self,
        on_open_change_complete: Option<OnOpenChange>,
    ) -> Self {
        self.on_open_change_complete = on_open_change_complete;
        self
    }

    #[track_caller]
    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let controlled_model = self.model;
        let default_value = self.default_value;
        let mut items = self.items;
        let menu_disabled = self.disabled;
        let viewport_enabled = self.viewport;
        let indicator_enabled = self.indicator;
        let viewport_test_id = self.viewport_test_id;
        let md_breakpoint_query = self.md_breakpoint_query;
        let query_region_override = self.query_region;
        let chrome = self.chrome;
        let layout = self.layout;
        let style = self.style;
        let cfg = self.config;
        let on_value_change = self.on_value_change;
        let on_open_change_complete = self.on_open_change_complete;

        let value_model =
            radix_navigation_menu::navigation_menu_use_value_model(cx, controlled_model, || {
                default_value.clone()
            })
            .model();

        let theme = Theme::global(&*cx.app).snapshot();

        let trigger_pad_x = nav_menu_trigger_padding_x(&theme);
        let trigger_pad_y = nav_menu_trigger_padding_y(&theme);
        let trigger_radius = nav_menu_trigger_radius(&theme);
        let trigger_bg_hover = nav_menu_trigger_bg_hover(&theme);
        let trigger_bg_open = nav_menu_trigger_bg_open(&theme);
        let trigger_fg = nav_menu_trigger_fg(&theme);
        let trigger_fg_muted = nav_menu_trigger_fg_muted(&theme);
        let trigger_text_style = nav_menu_trigger_text_style(&theme);
        let default_trigger_bg = WidgetStateProperty::new(None)
            .when(
                WidgetStates::HOVERED,
                Some(ColorRef::Color(trigger_bg_hover)),
            )
            .when(
                WidgetStates::ACTIVE,
                Some(ColorRef::Color(trigger_bg_hover)),
            )
            .when(WidgetStates::OPEN, Some(ColorRef::Color(trigger_bg_open)))
            .when(
                WidgetStates::OPEN | WidgetStates::HOVERED,
                Some(ColorRef::Color(trigger_bg_hover)),
            )
            .when(
                WidgetStates::OPEN | WidgetStates::ACTIVE,
                Some(ColorRef::Color(trigger_bg_hover)),
            );
        let default_trigger_fg = WidgetStateProperty::new(ColorRef::Color(trigger_fg))
            .when(WidgetStates::DISABLED, ColorRef::Color(trigger_fg_muted));

        let viewport_bg = nav_menu_viewport_bg(&theme);
        let viewport_border = nav_menu_viewport_border(&theme);
        let viewport_radius = theme
            .metric_by_key("component.navigation_menu.viewport.radius")
            // The new-york-v4 recipe uses `rounded-md` (8px) for the content/viewport surface.
            .unwrap_or(Px(8.0));
        let content_switch_slide_px = nav_menu_content_switch_slide_px(&theme);
        let viewport_shadow = decl_style::shadow(&theme, viewport_radius);
        let dir = crate::direction::use_direction(cx, None);
        // Upstream base-maia `NavigationMenuContent` uses `p-2.5 pr-3`.
        let content_pad_y = MetricRef::space(Space::N2p5).resolve(&theme);
        let content_pad_left = MetricRef::space(Space::N2p5).resolve(&theme);
        let content_pad_right = MetricRef::space(Space::N3).resolve(&theme);
        let content_padding = rtl::padding_edges_with_inline_start_end(
            dir,
            content_pad_y,
            content_pad_y,
            content_pad_left,
            content_pad_right,
        );

        if std::env::var("FRET_DEBUG_NAV_MENU_TRIGGER")
            .ok()
            .is_some_and(|v| v == "1")
        {
            static PRINT_ONCE: std::sync::Once = std::sync::Once::new();
            PRINT_ONCE.call_once(|| {
                let trigger_space_px = nav_menu_trigger_space_px(&theme);
                eprintln!(
                    "nav-menu trigger metrics: pad_x={} pad_y={} space_px={} text_px={} line_height={}",
                    trigger_pad_x.0,
                    trigger_pad_y.0,
                    trigger_space_px.0,
                    trigger_text_style.size.0,
                    trigger_text_style.line_height.map(|v| v.0).unwrap_or(-1.0),
                );
            });
        }

        let root_props = decl_style::container_props(&theme, chrome, layout);

        let region_props = LayoutQueryRegionProps {
            layout: decl_style::layout_style(
                &theme,
                LayoutRefinement::default().w_full().min_w_0(),
            ),
            name: None,
        };

        fret_ui_kit::declarative::container_query_region_with_id(
            cx,
            "shadcn.navigation_menu",
            region_props,
            move |cx, region_id| {
                let region_id_for_queries = query_region_override.unwrap_or(region_id);
                vec![cx.container(root_props, move |cx| {
                    let root_id = cx.root_id();
                    let nav_ctx = radix_navigation_menu::NavigationMenuRoot::new(value_model.clone())
                        .config(cfg)
                        .disabled(menu_disabled)
                        .context(cx, root_id);
                    let root_state = nav_ctx.root_state.clone();

                    #[derive(Default)]
                    struct SelectionSyncState {
                        last_selected: Option<Arc<str>>,
                    }

                    #[derive(Default)]
                    struct SafeCorridorState {
                        last_pointer: Option<Point>,
                        trigger_anchor: Option<Rect>,
                        viewport_panel: Option<Rect>,
                        pointer_in_corridor: bool,
                    }

                    let open_model = cx.model_for(root_id, || false);
                    let trigger_tab_stop_model = cx.model_for(root_id, || None::<Arc<str>>);

                    let selected: Option<Arc<str>> =
                        cx.watch_model(&value_model).layout().cloned().flatten();
                    if let Some(handler) = on_value_change.as_ref() {
                        let changed = cx.slot_state(
                            NavigationMenuValueChangeCallbackState::default,
                            |state| navigation_menu_value_change_event(state, selected.clone()),
                        );
                        if let Some(value) = changed {
                            handler(value);
                        }
                    }
                    let safe_corridor: Arc<Mutex<SafeCorridorState>> = cx.state_for(
                        root_id,
                        || Arc::new(Mutex::new(SafeCorridorState::default())),
                        |st| st.clone(),
                    );
                    let selected_changed = cx.state_for(root_id, SelectionSyncState::default, |st| {
                        let changed = selected != st.last_selected;
                        if changed {
                            st.last_selected = selected.clone();
                        }
                        changed
                    });

                    if selected_changed {
                        let selected = selected.clone();
                        let _ = cx
                            .app
                            .models_mut()
                            .update(&open_model, |v| *v = selected.is_some());
                        if selected.is_none() {
                            let mut safe = safe_corridor.lock().unwrap_or_else(|e| e.into_inner());
                            safe.pointer_in_corridor = false;
                            safe.viewport_panel = None;
                            safe.trigger_anchor = None;
                        }
                    }

                    if selected.is_some() {
                        let should_keep_open = {
                            let safe = safe_corridor.lock().unwrap_or_else(|e| e.into_inner());
                            safe.pointer_in_corridor
                        };
                        if should_keep_open {
                            let _ = cx.app.models_mut().update(&open_model, |v| *v = true);
                        }
                    }

                    let open: bool = cx
                        .watch_model(&open_model)
                        .layout()
                        .copied()
                        .unwrap_or(false);
                    let open_for_motion = open && selected.is_some();
                    let motion = OverlayController::transition_with_durations_and_cubic_bezier_duration(
                        cx,
                        open_for_motion,
                        overlay_motion::shadcn_motion_duration_200(cx),
                        overlay_motion::shadcn_motion_duration_200(cx),
                        overlay_motion::shadcn_motion_ease_bezier(cx),
                    );
                    let opacity = motion.progress;
                    let scale = if viewport_enabled {
                        // shadcn new-york:
                        // - Viewport: `zoom-in-90` on open, `zoom-out-95` on close.
                        if open_for_motion {
                            0.9 + 0.1 * opacity
                        } else {
                            0.95 + 0.05 * opacity
                        }
                    } else {
                        // When `viewport=false`, content behaves like a popover-ish surface with
                        // `zoom-in-95` / `zoom-out-95`.
                        0.95 + 0.05 * opacity
                    };

                    let mut selected_local = radix_navigation_menu::navigation_menu_viewport_selected_value(
                        cx,
                        root_id,
                        selected.clone(),
                        motion.present,
                    );

                    if !open_for_motion && selected.is_some() && !motion.present {
                        let mut host = fret_ui::action::UiActionHostAdapter { app: &mut *cx.app };
                        let action_cx = fret_ui::action::ActionCx {
                            window: cx.window,
                            target: root_id,
                        };
                        let mut st = root_state.lock().unwrap_or_else(|e| e.into_inner());
                        st.on_item_dismiss(&mut host, action_cx, &value_model, cfg);
                        selected_local = None;
                    }

                    let active_idx = selected_local.as_deref().and_then(|v| {
                        items
                            .iter()
                            .position(|it| it.value.as_ref() == v)
                            .filter(|_| !menu_disabled)
                    });

                    let values: Vec<Arc<str>> = items.iter().map(|it| it.value.clone()).collect();
                    let transition = radix_navigation_menu::navigation_menu_content_transition(
                        cx,
                        root_id,
                        open_for_motion,
                        selected.clone(),
                        &values,
                    );

                    let md_breakpoint =
                        nav_menu_md_breakpoint(cx, md_breakpoint_query, region_id_for_queries);
                    let list_props = FlexProps {
                        layout: LayoutStyle::default(),
                        direction: fret_core::Axis::Horizontal,
                        gap: Px(4.0).into(), // Tailwind `space-x-1`
                        padding: Edges::all(Px(0.0)).into(),
                        justify: MainAlign::Center,
                        align: fret_ui::element::CrossAlign::Center,
                        wrap: true,
                        ..Default::default()
                    };

                    let items_for_children = &mut items;
                    let value_for_viewport = value_model.clone();
                    let value_for_viewport_for_list = value_for_viewport.clone();
                    let trigger_text_style_for_list = trigger_text_style.clone();
                    let nav_ctx_for_list = nav_ctx.clone();
                    let theme_for_list = theme.clone();
                    let default_trigger_bg_for_list = default_trigger_bg.clone();
                    let default_trigger_fg_for_list = default_trigger_fg.clone();
                    let style_for_list = style.clone();

                    let roving_disabled: Arc<[bool]> = items_for_children
                        .iter()
                        .map(|it| menu_disabled || it.disabled)
                        .collect::<Vec<_>>()
                        .into();

                    // Radix `NavigationMenu` trigger row uses roving focus: only one trigger is in the Tab
                    // order (`tabIndex=0`), while other triggers are focusable via arrow keys / click
                    // (`tabIndex=-1`). Model the Tab-stop outcome by setting `PressableProps.focusable`
                    // only on the active trigger.
                    let trigger_tab_stop: Option<Arc<str>> = cx
                        .watch_model(&trigger_tab_stop_model)
                        .layout()
                        .cloned()
                        .flatten();
                    let roving_keys: Arc<[Arc<str>]> = items_for_children
                        .iter()
                        .map(|it| it.value.clone())
                        .collect::<Vec<_>>()
                        .into();
                    let desired_tab_stop = roving_focus_group::active_index_from_str_keys(
                        &roving_keys,
                        selected.as_deref(),
                        &roving_disabled,
                    )
                    .and_then(|idx| roving_keys.get(idx).cloned());
                    let valid_current = trigger_tab_stop.as_ref().and_then(|current| {
                        let idx = roving_keys
                            .iter()
                            .position(|k| k.as_ref() == current.as_ref())?;
                        if roving_disabled.get(idx).copied().unwrap_or(true) {
                            return None;
                        }
                        Some(current.clone())
                    });
                    let tab_stop_value = if selected.is_some() {
                        desired_tab_stop
                    } else {
                        valid_current.or(desired_tab_stop)
                    };
                    if tab_stop_value.as_deref() != trigger_tab_stop.as_deref() {
                        let next = tab_stop_value.clone();
                        let _ = cx
                            .app
                            .models_mut()
                            .update(&trigger_tab_stop_model, |v| *v = next);
                    }

                    let roving_props = roving_focus_group::RovingFlexProps {
                        flex: list_props,
                        roving: roving_focus_group::RovingFocusProps {
                            enabled: true,
                            wrap: true,
                            disabled: roving_disabled.clone(),
                        },
                    };
                    let dir_for_roving = direction_prim::use_direction_in_scope(cx, None);
                    let tab_stop_value_for_list = tab_stop_value.clone();
                    let trigger_tab_stop_model_for_list = trigger_tab_stop_model.clone();

                    let list = roving_focus_group::roving_focus_group_apg_with_direction(
                        cx,
                        roving_props,
                        roving_focus_group::TypeaheadPolicy::None,
                        dir_for_roving,
                        move |cx| {
                            items_for_children
                                .iter_mut()
                                .map(|item| {
                                let item_value = item.value.clone();
                                let label = item.label.clone();
                                let disabled = menu_disabled || item.disabled;
                                let tab_stop = tab_stop_value_for_list
                                    .as_ref()
                                    .is_some_and(|v| v.as_ref() == item_value.as_ref());
                                let trigger_test_id = item.trigger_test_id.clone();
                                let trigger_chrome_test_id = trigger_test_id
                                    .clone()
                                    .map(|id| Arc::<str>::from(format!("{id}.chrome")));
                                let trigger_tab_stop_model = trigger_tab_stop_model_for_list.clone();
                                let trigger_text_style_for_item = trigger_text_style_for_list.clone();
                                let nav_ctx_for_item = nav_ctx_for_list.clone();
                                let theme_for_item = theme_for_list.clone();
                                let default_trigger_bg = default_trigger_bg_for_list.clone();
                                let default_trigger_fg = default_trigger_fg_for_list.clone();
                                let style_override = style_for_list.clone();
                                let value_for_viewport = value_for_viewport_for_list.clone();

                                let trigger_children = item.trigger.take();
                                let content_is_empty = item.content.is_empty();
                                let item_label = item.label.clone();
                                let command = item.command.clone();
                                let on_activate = item.on_activate.clone();
                                let href_for_action = item.href.clone();
                                let href_for_semantics = item.href.clone();
                                let target = item.target.clone();
                                let rel = item.rel.clone();
                                let should_fallback_open_url =
                                    command.is_none() && on_activate.is_none();
                                let fallback_open_url = if should_fallback_open_url {
                                    href_for_action
                                        .clone()
                                        .map(|href| open_url_on_activate(href, target.clone(), rel.clone()))
                                } else {
                                    None
                                };
                                cx.keyed(item_value.clone(), move |cx| {
                                    let trigger_text_style = trigger_text_style_for_item.clone();

                                    let mut pressable = PressableProps::default();
                                    pressable.enabled = !disabled;
                                    pressable.focusable = !disabled && tab_stop;
                                    pressable.layout = decl_style::layout_style(
                                        &theme_for_item,
                                        navigation_menu_trigger_style(&theme_for_item).layout,
                                    );
                                    pressable.a11y = PressableA11y {
                                        role: Some(SemanticsRole::Button),
                                        label: Some(label.clone()),
                                        test_id: trigger_test_id.clone(),
                                        ..Default::default()
                                    };

                                    let pointer_props = PointerRegionProps {
                                        layout: LayoutStyle::default(),
                                        enabled: true,
                                        ..Default::default()
                                    };

                                    if content_is_empty {
                                        // shadcn/ui demo uses a `NavigationMenuLink` for items with no
                                        // content (e.g. "Docs"), styled via `navigationMenuTriggerStyle()`.
                                        // These should behave like a link (no chevron, no open/close).
                                        let trigger_text_style = trigger_text_style.clone();
                                        let trigger_children = trigger_children;
                                        let command = command.clone();
                                        let model_for_activate = value_for_viewport.clone();
                                        let item_on_activate = on_activate.clone();
                                        let item_fallback_open_url = fallback_open_url.clone();
                                        let combined_on_activate: OnActivate = Arc::new(
                                            move |host, action_cx, reason| {
                                                if let Some(on_activate) = item_on_activate.clone() {
                                                    on_activate(host, action_cx, reason);
                                                } else if let Some(on_activate) =
                                                    item_fallback_open_url.clone()
                                                {
                                                    on_activate(host, action_cx, reason);
                                                }
                                                let _ = host
                                                    .models_mut()
                                                    .update(&model_for_activate, |v| *v = None);
                                            },
                                        );

                                        pressable.a11y.role = Some(SemanticsRole::Link);
                                        pressable.key_activation = PressableKeyActivation::EnterOnly;
                                        pressable.focus_ring =
                                            Some(decl_style::focus_ring(&theme_for_item, trigger_radius));

                                        let mut element = cx.pressable(pressable, move |cx, st| {
                                            cx.pressable_dispatch_command_if_enabled_opt(command.clone());
                                            cx.pressable_on_activate(combined_on_activate.clone());

                                            if st.focused {
                                                let _ = cx.app.models_mut().update(
                                                    &trigger_tab_stop_model,
                                                    |v| *v = Some(item_value.clone()),
                                                );
                                            }

                                            let hovered = st.hovered && !st.pressed;
                                            let pressed = st.pressed;
                                            let fg = if disabled {
                                                trigger_fg_muted
                                            } else {
                                                trigger_fg
                                            };
                                            let bg = (hovered || pressed).then_some(trigger_bg_hover);

                                            let mut layout = LayoutStyle::default();
                                            layout.size.height = Length::Fill;

                                            let wrapper = ContainerProps {
                                                layout,
                                                padding: Edges {
                                                    top: trigger_pad_y,
                                                    right: trigger_pad_x,
                                                    bottom: trigger_pad_y,
                                                    left: trigger_pad_x,
                                                }.into(),
                                                background: bg,
                                                shadow: None,
                                                border: Edges::all(Px(0.0)),
                                                border_color: None,
                                                corner_radii: Corners::all(trigger_radius),
                                                ..Default::default()
                                            };

                                            let content_children = trigger_children.unwrap_or_else(|| {
                                                    let style = trigger_text_style.clone();
                                                    let mut label = ui::label(item_label.clone())
                                                        .text_size_px(style.size)
                                                        .font_weight(style.weight)
                                                        .text_color(ColorRef::Color(fg))
                                                        .nowrap();
                                                    if let Some(line_height) = style.line_height {
                                                        label = label
                                                            .line_height_px(line_height)
                                                            .line_height_policy(
                                                                fret_core::TextLineHeightPolicy::FixedFromStyle,
                                                            );
                                                    }
                                                    if let Some(letter_spacing_em) = style.letter_spacing_em
                                                    {
                                                        label = label.letter_spacing_em(letter_spacing_em);
                                                    }
                                                    vec![label.into_element(cx)]
                                                });

                                            let row = cx.flex(
                                                FlexProps {
                                                    layout: {
                                                        let mut layout = LayoutStyle::default();
                                                        layout.size.height = Length::Fill;
                                                        layout
                                                    },
                                                    direction: fret_core::Axis::Horizontal,
                                                    gap: Px(0.0).into(),
                                                    padding: Edges::all(Px(0.0)).into(),
                                                    justify: MainAlign::Center,
                                                    align: fret_ui::element::CrossAlign::Center,
                                                    wrap: false,
                                                    ..Default::default()
                                                },
                                                move |_cx| content_children,
                                            );

                                            let child = cx.container(wrapper, move |_cx| vec![row]);
                                            let mut chrome = child;
                                            if let Some(test_id) = trigger_chrome_test_id.clone() {
                                                chrome = chrome.test_id(test_id);
                                            }
                                            vec![chrome]
                                        });

                                        if let Some(href) = href_for_semantics.clone() {
                                            element = element.attach_semantics(
                                                SemanticsDecoration::default().value(href),
                                            );
                                        }

                                        return element;
                                    }
                                    radix_navigation_menu::NavigationMenuTrigger::new(item_value.clone())
                                        .label(label.clone())
                                        .disabled(disabled)
                                        .into_element(
                                            cx,
                                            &nav_ctx_for_item,
                                            pressable,
                                            pointer_props,
                                            move |cx, st, is_open| {
                                                if st.focused {
                                                    let _ = cx.app.models_mut().update(
                                                        &trigger_tab_stop_model,
                                                        |v| *v = Some(item_value.clone()),
                                                    );
                                                }

                                                let mut states =
                                                    WidgetStates::from_pressable(cx, st, !disabled);
                                                states.set(WidgetState::Open, is_open);

                                                let fg_ref = resolve_override_slot(
                                                    style_override.trigger_foreground.as_ref(),
                                                    &default_trigger_fg,
                                                    states,
                                                );
                                                let bg_ref = resolve_override_slot_opt(
                                                    style_override.trigger_background.as_ref(),
                                                    &default_trigger_bg,
                                                    states,
                                                );

                                                let bg = bg_ref
                                                    .as_ref()
                                                    .map(|color| color.resolve(&theme_for_item));

                                                let mut layout = LayoutStyle::default();
                                                layout.size.height = Length::Fill;

                                                let wrapper = ContainerProps {
                                                    layout,
                                                    padding: Edges {
                                                        top: trigger_pad_y,
                                                        right: trigger_pad_x,
                                                        bottom: trigger_pad_y,
                                                        left: trigger_pad_x,
                                                    }.into(),
                                                    background: bg,
                                                    shadow: None,
                                                    border: Edges::all(Px(0.0)),
                                                    border_color: None,
                                                    corner_radii: Corners::all(trigger_radius),
                                                    ..Default::default()
                                                };

                                                let content_children =
                                                    trigger_children.unwrap_or_else(|| {
                                                        let style = trigger_text_style.clone();
                                                        let mut label = ui::label( item_label.clone())
                                                            .text_size_px(style.size)
                                                            .font_weight(style.weight)
                                                            .text_color(fg_ref.clone())
                                                            .nowrap();
                                                        if let Some(line_height) = style.line_height {
                                                            label = label
                                                                .line_height_px(line_height)
                                                                .line_height_policy(
                                                                    fret_core::TextLineHeightPolicy::FixedFromStyle,
                                                                );
                                                        }
                                                        if let Some(letter_spacing_em) =
                                                            style.letter_spacing_em
                                                        {
                                                            label = label.letter_spacing_em(letter_spacing_em);
                                                        }
                                                        vec![label.into_element(cx)]
                                                    });

                                                let fg_ref_for_chevron = fg_ref.clone();
                                                let chevron_motion = drive_navigation_menu_trigger_chevron_motion(
                                                    cx,
                                                    item_value.clone(),
                                                    is_open,
                                                );
                                                let chevron_rotation = 180.0 * chevron_motion.progress;
                                                let chevron_size = Px(12.0); // Tailwind `size-3`
                                                let chevron_center =
                                                    Point::new(Px(chevron_size.0 * 0.5), Px(chevron_size.0 * 0.5));
                                                let chevron_transform = Transform2D::rotation_about_degrees(
                                                    chevron_rotation,
                                                    chevron_center,
                                                );

                                                let chevron = cx.visual_transform_props(
                                                    VisualTransformProps {
                                                        layout: {
                                                            let mut layout = LayoutStyle::default();
                                                            layout.size = SizeStyle {
                                                                width: Length::Px(chevron_size),
                                                                height: Length::Px(chevron_size),
                                                                ..Default::default()
                                                            };
                                                            layout.flex.shrink = 0.0;
                                                            layout.position = fret_ui::element::PositionStyle::Relative;
                                                            layout.inset.top = Some(Px(1.0)).into(); // `top-[1px]`
                                                            layout.margin.left =
                                                                fret_ui::element::MarginEdge::Px(Px(4.0)); // `ml-1`
                                                            layout
                                                        },
                                                        transform: chevron_transform,
                                                    },
                                                    move |cx| {
                                                        vec![decl_icon::icon_with(
                                                            cx,
                                                            ids::ui::CHEVRON_DOWN,
                                                            Some(chevron_size),
                                                            Some(fg_ref_for_chevron.clone()),
                                                        )]
                                                    },
                                                );

                                                let mut row_children = content_children;
                                                // Upstream adds a literal `" "` text node between the label
                                                // and the chevron icon, in addition to `ml-1` on the icon.
                                                // We model the outcome as a spacer metric so the trigger
                                                // width matches the extracted web goldens closely.
                                                let space_px = nav_menu_trigger_space_px(&theme_for_item);
                                                row_children.push(cx.container(
                                                    ContainerProps {
                                                        layout: {
                                                            let mut layout = LayoutStyle::default();
                                                            layout.size = SizeStyle {
                                                                width: Length::Px(space_px),
                                                                height: Length::Px(Px(0.0)),
                                                                ..Default::default()
                                                            };
                                                            layout.flex.shrink = 0.0;
                                                            layout
                                                        },
                                                        ..Default::default()
                                                    },
                                                    |_cx| Vec::new(),
                                                ));
                                                row_children.push(chevron);
                                                let row = cx.flex(
                                                    FlexProps {
                                                        layout: {
                                                            let mut layout = LayoutStyle::default();
                                                            layout.size.height = Length::Fill;
                                                            layout
                                                        },
                                                        direction: fret_core::Axis::Horizontal,
                                                        gap: Px(0.0).into(),
                                                        padding: Edges::all(Px(0.0)).into(),
                                                        justify: MainAlign::Center,
                                                        align: fret_ui::element::CrossAlign::Center,
                                                        wrap: false,
                                                        ..Default::default()
                                                    },
                                                    move |_cx| row_children,
                                                );

                                                let child = cx.container(wrapper, move |_cx| vec![row]);
                                                let mut chrome = child;
                                                if let Some(test_id) = trigger_chrome_test_id.clone() {
                                                    chrome = chrome.test_id(test_id);
                                                }
                                                vec![chrome]
                                            },
                                        )
                                })
                                })
                                .collect::<Vec<_>>()
                        },
                    );

                    let viewport_children = active_idx
                        .and_then(|idx| items.get_mut(idx))
                        .map(|active| std::mem::take(&mut active.content))
                        .unwrap_or_default();

                    let has_content = !viewport_children.is_empty();
                    let is_open = selected_local.is_some() && has_content && open_for_motion;
                    let overlay_presence = OverlayPresence {
                        // Keep the viewport overlay request alive for the full close transition, even when
                        // the current selection has already been cleared (e.g. Escape dismissal). The
                        // overlay controller needs an explicit "present=false" update to unmount cleanly.
                        present: motion.present,
                        interactive: is_open,
                    };
                    let open_change_complete = cx.slot_state(
                        NavigationMenuOpenChangeCallbackState::default,
                        |state| {
                            navigation_menu_open_change_complete_event(
                                state,
                                is_open,
                                overlay_presence.present,
                                motion.animating,
                            )
                        },
                    );
                    if let (Some(open), Some(handler)) =
                        (open_change_complete, on_open_change_complete.as_ref())
                    {
                        handler(open);
                    }

                    if !overlay_presence.present {
                        let mut safe = safe_corridor.lock().unwrap_or_else(|e| e.into_inner());
                        safe.viewport_panel = None;
                        safe.pointer_in_corridor = false;
                    }

                    let content_switch = radix_navigation_menu::navigation_menu_content_switch(transition)
                        .map(|sw| {
                            let from_children = items
                                .get_mut(sw.from_idx)
                                .map(|it| std::mem::take(&mut it.content))
                                .unwrap_or_default();
                            (sw.progress, sw.forward, from_children)
                        });

                    if overlay_presence.present {
                        let side_offset = nav_menu_viewport_side_offset(&theme);
                        let window_margin = nav_menu_viewport_window_margin(&theme);
                        let indicator_size = if indicator_enabled { side_offset } else { Px(0.0) };
                        let indicator_diamond_size = nav_menu_indicator_diamond_size(&theme);
                        let indicator_diamond_corners = {
                            let mut corners = Corners::all(Px(0.0));
                            corners.top_left = Px(2.0);
                            corners
                        };
                        let mut indicator_diamond_shadow = decl_style::shadow_md(&theme, Px(0.0));
                        indicator_diamond_shadow.corner_radii = indicator_diamond_corners;

                        let estimated = fret_core::Size::new(Px(320.0), Px(240.0));
                        let measured = selected_local
                            .as_deref()
                            .and_then(|value| {
                                radix_navigation_menu::navigation_menu_viewport_content_id(
                                    cx, root_id, value,
                                )
                            })
                            .and_then(|id| cx.last_bounds_for_element(id).map(|r| r.size));

                        if let Some(selected_value) = selected_local.as_deref()
                            && let Some(trigger_id) =
                                radix_navigation_menu::navigation_menu_trigger_id(cx, root_id, selected_value)
                            && let Some(trigger_anchor) = cx
                                .last_visual_bounds_for_element(trigger_id)
                                .or_else(|| cx.last_bounds_for_element(trigger_id))
                        {
                            let mut safe = safe_corridor.lock().unwrap_or_else(|e| e.into_inner());
                            safe.trigger_anchor = Some(trigger_anchor);
                        }
                        if viewport_enabled {
                            if let (Some(selected_value), Some(size)) = (selected_local.clone(), measured) {
                                radix_navigation_menu::navigation_menu_register_viewport_size(
                                    cx,
                                    root_id,
                                    selected_value,
                                    size,
                                );
                            }
                        }

                        let fallback = measured.unwrap_or(estimated);
                        let content_size = if viewport_enabled {
                            radix_navigation_menu::navigation_menu_viewport_size_for_transition(
                                cx,
                                root_id,
                                selected_local.clone(),
                                &values,
                                transition,
                                fallback,
                            )
                            .size
                        } else {
                            fallback
                        };

                        let root_state_for_viewport = root_state.clone();
                        let value_for_hover = value_for_viewport.clone();
                        let viewport_children_for_panel = viewport_children;
                        let content_switch_for_panel = content_switch;
                        let content_switch_slide_px = content_switch_slide_px;

                        let mut panel_props = if viewport_enabled {
                            ContainerProps {
                                layout: LayoutStyle {
                                    overflow: fret_ui::element::Overflow::Visible,
                                    ..Default::default()
                                },
                                padding: Edges::all(Px(0.0)).into(),
                                background: Some(viewport_bg),
                                shadow: Some(viewport_shadow),
                                border: Edges::all(Px(1.0)),
                                border_color: Some(viewport_border),
                                corner_radii: Corners::all(viewport_radius),
                                ..Default::default()
                            }
                        } else {
                            ContainerProps {
                                layout: LayoutStyle {
                                    overflow: fret_ui::element::Overflow::Visible,
                                    ..Default::default()
                                },
                                padding: Edges::all(Px(0.0)).into(),
                                background: Some(viewport_bg),
                                shadow: Some(viewport_shadow),
                                border: Edges::all(Px(1.0)),
                                border_color: Some(viewport_border),
                                corner_radii: Corners::all(viewport_radius),
                                ..Default::default()
                            }
                        };
                        if viewport_enabled {
                            // Match shadcn/ui + Radix: the viewport panel is sized by the measured content
                            // dimensions (CSS vars). Tailwind preflight uses `box-sizing: border-box`, so
                            // the viewport border is included in those dimensions.
                            //
                            // We apply this even when the first measurement hasn't been observed yet so
                            // popper placement and panel layout remain consistent across frames.
                            panel_props.layout.size.height = Length::Px(content_size.height);
                            panel_props.layout.size.width = if md_breakpoint {
                                // Desktop: `md:w-[var(--radix-navigation-menu-viewport-width)]`.
                                Length::Px(content_size.width)
                            } else {
                                // Mobile: `w-full` so the panel tracks the anchor width.
                                Length::Fill
                            };
                        }

                        let placement = popper::PopperContentPlacement::new(
                            direction_prim::use_direction_in_scope(cx, None),
                            Side::Bottom,
                            Align::Start,
                            side_offset,
                        );

                        let args = radix_navigation_menu::NavigationMenuViewportOverlayRequestArgs {
                            window_margin,
                            placement,
                            placement_anchor_override: viewport_enabled.then_some(root_id),
                            content_size,
                            indicator_size,
                            width_tracks_anchor: !md_breakpoint,
                        };

                        let opacity = opacity;
                        let scale = scale;
                        let selected_value_for_content_id = selected_local.clone();
                        let selected_for_overlay = selected_local.clone();
                        let safe_corridor_for_overlay_pointer_move = safe_corridor.clone();
                        let safe_corridor_for_overlay_layout = safe_corridor.clone();
                        let safe_corridor_for_content_hover = safe_corridor.clone();
                        let on_pointer_move: fret_ui::action::OnDismissiblePointerMove = Arc::new(
                            move |_host: &mut dyn fret_ui::action::UiActionHost,
                                  _acx: fret_ui::action::ActionCx,
                                  mv: fret_ui::action::PointerMoveCx| {
                                if mv.pointer_type == PointerType::Touch {
                                    return false;
                                }
                                let mut safe = safe_corridor_for_overlay_pointer_move
                                    .lock()
                                    .unwrap_or_else(|e| e.into_inner());
                                safe.last_pointer = Some(mv.position);
                                safe.pointer_in_corridor = safe
                                    .trigger_anchor
                                    .zip(safe.viewport_panel)
                                    .is_some_and(|(trigger_anchor, viewport_panel)| {
                                        safe_hover::safe_hover_contains(
                                            mv.position,
                                            trigger_anchor,
                                            viewport_panel,
                                            NAV_MENU_SAFE_CORRIDOR_BUFFER,
                                        )
                                    });
                                false
                            },
                        );
                        let on_pointer_move = Some(on_pointer_move);
                        radix_navigation_menu::navigation_menu_request_viewport_overlay(
                            cx,
                            root_id,
                            cfg,
                            value_model.clone(),
                            open_model.clone(),
                            overlay_presence,
                            selected_for_overlay.as_deref(),
                            args,
                            on_pointer_move,
                            move |cx, layout| {
                                {
                                    let mut safe = safe_corridor_for_overlay_layout
                                        .lock()
                                        .unwrap_or_else(|e| e.into_inner());
                                    safe.viewport_panel = Some(layout.placed);
                                }
                                let Some(selected_value_key) =
                                    selected_value_for_content_id.as_deref()
                                else {
                                    return radix_navigation_menu::NavigationMenuViewportOverlayRenderOutput {
                                        opacity,
                                        transform: overlay_motion::shadcn_zoom_transform_with_scale(
                                            layout.transform_origin,
                                            scale,
                                        ),
                                        children: Vec::new(),
                                    };
                                };

                                let root_state_for_hover = root_state_for_viewport.clone();
                                let value_for_hover = value_for_hover.clone();
                                let panel_props = panel_props;
                                let viewport_children = viewport_children_for_panel;
                                let content_switch = content_switch_for_panel;
                                let content_switch_slide_px = content_switch_slide_px;
                                let content_padding = content_padding;
                                let indicator_diamond_shadow = indicator_diamond_shadow;
                                let indicator_diamond_corners = indicator_diamond_corners;
                                let viewport_enabled = viewport_enabled;
                                let selected_value_for_registry: Arc<str> = Arc::from(selected_value_key);

                                let content =
                                    radix_navigation_menu::navigation_menu_viewport_content_pressable_with_id_props(
                                        cx,
                                        root_id,
                                        selected_value_key,
                                        move |cx, _st, content_id| {
                                        let root_state_for_hover = root_state_for_hover.clone();
                                        let value_for_hover = value_for_hover.clone();
                                        cx.pressable_on_hover_change(Arc::new(
                                            move |host, action_cx, hovered| {
                                                if hovered {
                                                    let mut safe = safe_corridor_for_content_hover
                                                        .lock()
                                                        .unwrap_or_else(|e| e.into_inner());
                                                    safe.pointer_in_corridor = false;
                                                }
                                                let mut root = root_state_for_hover
                                                    .lock()
                                                    .unwrap_or_else(|e| e.into_inner());
                                                if hovered {
                                                    root.on_content_enter(host);
                                                } else {
                                                    root.on_content_leave(
                                                        host,
                                                        action_cx,
                                                        &value_for_hover,
                                                        cfg,
                                                    );
                                                }
                                            },
                                        ));

                                        let root_id_for_registry = root_id;
                                        let value_for_registry = selected_value_for_registry.clone();
                                        let content_id_for_registry = content_id;
                                        let viewport_enabled_for_registry = viewport_enabled;
                                        let md_breakpoint_for_registry = md_breakpoint;
                                        let children = vec![cx.container(panel_props, move |cx| {
                                            let mut clip_layout = LayoutStyle::default();
                                            clip_layout.overflow = fret_ui::element::Overflow::Clip;
                                            if viewport_enabled_for_registry {
                                                clip_layout.size = SizeStyle {
                                                    width: Length::Fill,
                                                    height: Length::Fill,
                                                    ..Default::default()
                                                };
                                            }

                                            let clip_props = ContainerProps {
                                                layout: clip_layout,
                                                corner_radii: Corners::all(viewport_radius),
                                                ..Default::default()
                                            };

                                            vec![cx.container(clip_props, move |cx| {
                                            let Some((t, forward, from_children)) = content_switch
                                            else {
                                                let children = viewport_children;
                                                let viewport_enabled_for_body = viewport_enabled_for_registry;
                                                let md_breakpoint_for_body = md_breakpoint_for_registry;
                                                let body = cx.keyed("viewport-body", move |cx| {
                                                    let mut body_layout = LayoutStyle::default();
                                                    if viewport_enabled_for_body && !md_breakpoint_for_body {
                                                        body_layout.size.width = Length::Fill;
                                                    }
                                                    cx.container(
                                                        ContainerProps {
                                                            layout: body_layout,
                                                            padding: content_padding.into(),
                                                            ..Default::default()
                                                        },
                                                        move |_cx| children,
                                                    )
                                                });
                                                if viewport_enabled_for_registry {
                                                    radix_navigation_menu::navigation_menu_register_viewport_content_id(
                                                        cx,
                                                        root_id_for_registry,
                                                        value_for_registry.clone(),
                                                        body.id,
                                                    );
                                                } else {
                                                    radix_navigation_menu::navigation_menu_register_viewport_content_id(
                                                        cx,
                                                        root_id_for_registry,
                                                        value_for_registry.clone(),
                                                        content_id_for_registry,
                                                    );
                                                }
                                                return vec![body];
                                            };

                                            let to_children = viewport_children;
                                            let t = t.clamp(0.0, 1.0);
                                            let slide = content_switch_slide_px.0;

                                            let (from_dx, to_dx) = if forward {
                                                (-slide * t, slide * (1.0 - t))
                                            } else {
                                                (slide * t, -slide * (1.0 - t))
                                            };

                                            let value_for_registry_for_layers = value_for_registry.clone();

                                            let mut layout_for_layers = LayoutStyle::default();
                                            layout_for_layers.overflow = fret_ui::element::Overflow::Clip;

                                            let stack = cx.stack_props(
                                                StackProps {
                                                    layout: layout_for_layers,
                                                },
                                                move |cx| {
                                                    let layer_layout = LayoutStyle::default();

                                                    let from_opacity = 1.0 - t;
                                                    let to_opacity = t;

                                                    let viewport_enabled_for_body = viewport_enabled_for_registry;
                                                    let md_breakpoint_for_body = md_breakpoint_for_registry;

                                                    let from_children = vec![cx.keyed("from-body", move |cx| {
                                                        let mut body_layout = LayoutStyle::default();
                                                        if viewport_enabled_for_body && !md_breakpoint_for_body {
                                                            body_layout.size.width = Length::Fill;
                                                        }
                                                        cx.container(
                                                            ContainerProps {
                                                                layout: body_layout,
                                                                padding: content_padding.into(),
                                                                ..Default::default()
                                                            },
                                                            {
                                                                let from_children = from_children;
                                                                move |_cx| from_children
                                                            },
                                                        )
                                                    })];
                                                    let viewport_enabled_for_body = viewport_enabled_for_registry;
                                                    let md_breakpoint_for_body = md_breakpoint_for_registry;
                                                    let to_body = cx.keyed("to-body", move |cx| {
                                                        let mut body_layout = LayoutStyle::default();
                                                        if viewport_enabled_for_body && !md_breakpoint_for_body {
                                                            body_layout.size.width = Length::Fill;
                                                        }
                                                        cx.container(
                                                            ContainerProps {
                                                                layout: body_layout,
                                                                padding: content_padding.into(),
                                                                ..Default::default()
                                                            },
                                                            {
                                                                let to_children = to_children;
                                                                move |_cx| to_children
                                                            },
                                                        )
                                                    });
                                                    if viewport_enabled_for_registry {
                                                        radix_navigation_menu::navigation_menu_register_viewport_content_id(
                                                            cx,
                                                            root_id_for_registry,
                                                            value_for_registry_for_layers.clone(),
                                                            to_body.id,
                                                        );
                                                    }
                                                    let to_children = vec![to_body];

                                                    let from = overlay_motion::wrap_opacity_and_render_transform_with_layouts(
                                                        cx,
                                                        layer_layout,
                                                        from_opacity,
                                                        RenderTransformProps {
                                                            layout: layer_layout,
                                                            transform: Transform2D::translation(Point::new(
                                                                Px(from_dx),
                                                                Px(0.0),
                                                            )),
                                                        },
                                                        from_children,
                                                    );

                                                    let to = overlay_motion::wrap_opacity_and_render_transform_with_layouts(
                                                        cx,
                                                        layer_layout,
                                                        to_opacity,
                                                        RenderTransformProps {
                                                            layout: layer_layout,
                                                            transform: Transform2D::translation(Point::new(
                                                                Px(to_dx),
                                                                Px(0.0),
                                                            )),
                                                        },
                                                        to_children,
                                                    );

                                                    vec![from, to]
                                                },
                                            );
                                            if !viewport_enabled_for_registry {
                                                radix_navigation_menu::navigation_menu_register_viewport_content_id(
                                                    cx,
                                                    root_id_for_registry,
                                                    value_for_registry.clone(),
                                                    content_id_for_registry,
                                                );
                                            }
                                            vec![stack]
                                        })]
                                        })];

                                        (
                                            PressableProps {
                                                layout: LayoutStyle::default(),
                                                enabled: true,
                                                focusable: false,
                                                focus_ring: None,
                                                focus_ring_always_paint: false,
                                                focus_ring_bounds: None,
                                                key_activation: Default::default(),
                                                a11y: PressableA11y::default(),
                                            },
                                            children,
                                        )
                                    },
                                );

                                let transform = overlay_motion::shadcn_zoom_transform_with_scale(
                                    layout.transform_origin,
                                    scale,
                                );

                                // `NavigationMenuContent` (desktop) and `NavigationMenuViewport` (mobile)
                                // are intrinsically sized in Radix/shadcn.
                                //
                                // We keep an autosizing wrapper for desktop-style transitions, but use a
                                // fixed-size wrapper on mobile when the panel is `w-full` (fill-based),
                                // otherwise `Length::Fill` has no definite width to resolve against.
                                let mut panel = if viewport_enabled && !md_breakpoint {
                                    popper_content::popper_wrapper_at(
                                        cx,
                                        layout.placed,
                                        Edges::all(Px(0.0)),
                                        move |_cx| vec![content],
                                    )
                                } else {
                                    popper_content::popper_wrapper_at_autosize(
                                        cx,
                                        layout.placed.origin,
                                        move |_cx| vec![content],
                                    )
                                };

                                if viewport_enabled {
                                    radix_navigation_menu::navigation_menu_register_viewport_panel_id(
                                        cx,
                                        root_id,
                                        panel.id,
                                    );
                                }

                                if let Some(test_id) = viewport_test_id.as_ref() {
                                    panel = panel.attach_semantics(
                                        SemanticsDecoration::default().test_id(test_id.clone()),
                                    );
                                }

                                let mut children = Vec::new();
                                if indicator_enabled && indicator_size.0 > 0.0 {
                                    let indicator = popper_content::popper_wrapper_panel_at(
                                        cx,
                                        layout.indicator_rect,
                                        Edges::all(Px(0.0)),
                                        fret_ui::element::Overflow::Clip,
                                        move |cx| {
                                            let track_w = layout.indicator_rect.size.width.0.max(0.0);
                                            let track_h = layout.indicator_rect.size.height.0.max(0.0);

                                            let diamond_size = indicator_diamond_size.0.max(0.0);
                                            let diamond_left = ((track_w - diamond_size) * 0.5).max(0.0);
                                            // Tailwind `top-[60%]` uses percentage units. In CSS, relative
                                            // positioning percentage offsets resolve against the containing
                                            // block's size (the indicator track), not the element's own size.
                                            let diamond_top = (track_h - diamond_size + track_h * 0.60).max(0.0);

                                            let mut diamond_layout = LayoutStyle::default();
                                            diamond_layout.position = fret_ui::element::PositionStyle::Absolute;
                                            diamond_layout.inset.left = Some(Px(diamond_left)).into();
                                            diamond_layout.inset.top = Some(Px(diamond_top)).into();
                                            diamond_layout.size = SizeStyle {
                                                width: Length::Px(Px(diamond_size)),
                                                height: Length::Px(Px(diamond_size)),
                                                ..Default::default()
                                            };

                                            let center = Point::new(Px(diamond_size * 0.5), Px(diamond_size * 0.5));
                                            let rotate = Transform2D::rotation_about_degrees(45.0, center);

                                            let diamond = cx.visual_transform_props(
                                                VisualTransformProps {
                                                    layout: diamond_layout,
                                                    transform: rotate,
                                                },
                                                move |cx| {
                                                    let mut layout = LayoutStyle::default();
                                                    layout.size = SizeStyle {
                                                        width: Length::Fill,
                                                        height: Length::Fill,
                                                        ..Default::default()
                                                    };

                                                    vec![cx.container(
                                                        ContainerProps {
                                                            layout,
                                                            padding: Edges::all(Px(0.0)).into(),
                                                            background: Some(viewport_border),
                                                            shadow: Some(indicator_diamond_shadow),
                                                            border: Edges::all(Px(0.0)),
                                                            border_color: None,
                                                            corner_radii: indicator_diamond_corners,
                                                            ..Default::default()
                                                        },
                                                        |_cx| Vec::new(),
                                                    )]
                                                },
                                            );

                                            radix_navigation_menu::navigation_menu_register_indicator_diamond_id(
                                                cx,
                                                root_id,
                                                diamond.id,
                                            );

                                            vec![diamond]
                                        },
                                    );
                                    radix_navigation_menu::navigation_menu_register_indicator_track_id(
                                        cx,
                                        root_id,
                                        indicator.id,
                                    );
                                    children.push(indicator);
                                }
                                children.push(panel);

                                radix_navigation_menu::NavigationMenuViewportOverlayRenderOutput {
                                    opacity,
                                    transform,
                                    children,
                                }
                            },
                        );
                    }

                    vec![list]
                })]
            },
        )
    }
}

/// Builder-preserving controlled helper for the common navigation-menu root path.
pub fn navigation_menu<H: UiHost, I>(
    cx: &mut ElementContext<'_, H>,
    model: impl IntoOptionalTextValueModel,
    f: impl FnOnce(&mut ElementContext<'_, H>) -> I,
) -> NavigationMenu
where
    I: IntoIterator<Item = NavigationMenuItem>,
{
    NavigationMenu::new(model).items(f(cx))
}

pub fn navigation_menu_list<H: UiHost, I>(
    cx: &mut ElementContext<'_, H>,
    f: impl FnOnce(&mut ElementContext<'_, H>) -> I,
) -> NavigationMenuList
where
    I: IntoIterator<Item = NavigationMenuItem>,
{
    NavigationMenuList::new(f(cx))
}

/// Builder-preserving uncontrolled helper for the common `defaultValue` root path.
pub fn navigation_menu_uncontrolled<H: UiHost, T: Into<Arc<str>>, I>(
    cx: &mut ElementContext<'_, H>,
    default_value: Option<T>,
    f: impl FnOnce(&mut ElementContext<'_, H>) -> I,
) -> NavigationMenu
where
    I: IntoIterator<Item = NavigationMenuItem>,
{
    NavigationMenu::uncontrolled(default_value).items(f(cx))
}

/// Alias for `NavigationMenu` that reads closer to Radix naming (`Root`).
pub type NavigationMenuRoot = NavigationMenu;

#[cfg(test)]
mod tests {
    use super::*;

    use fret_app::App;
    use fret_core::{
        AppWindowId, KeyCode, Modifiers, MouseButton, MouseButtons, Point, PointerEvent,
        PointerType, Px, Rect, Size,
    };
    use fret_core::{PathCommand, PathConstraints, PathId, PathMetrics, PathService, PathStyle};
    use fret_core::{SvgId, SvgService};
    use fret_core::{TextBlobId, TextConstraints, TextMetrics, TextService};
    use fret_runtime::{
        CommandMeta, CommandScope, FrameId, TickId, WindowCommandActionAvailabilityService,
        WindowCommandEnabledService, WindowCommandGatingService, WindowCommandGatingSnapshot,
    };
    use fret_ui::tree::UiTree;
    use fret_ui_kit::OverlayController;
    use fret_ui_kit::primitives::direction as direction_prim;
    use fret_ui_kit::primitives::direction::LayoutDirection;
    use std::collections::HashMap;

    #[derive(Default)]
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
                    size: Size::new(Px(10.0), Px(10.0)),
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
            _style: PathStyle,
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

    impl fret_core::MaterialService for FakeServices {
        fn register_material(
            &mut self,
            _desc: fret_core::MaterialDescriptor,
        ) -> Result<fret_core::MaterialId, fret_core::MaterialRegistrationError> {
            Ok(fret_core::MaterialId::default())
        }

        fn unregister_material(&mut self, _id: fret_core::MaterialId) -> bool {
            true
        }
    }

    fn bump_frame(app: &mut App) {
        app.set_tick_id(TickId(app.tick_id().0.saturating_add(1)));
        app.set_frame_id(FrameId(app.frame_id().0.saturating_add(1)));
    }

    #[test]
    fn trigger_chevron_motion_advances_and_settles_like_a_300ms_transition() {
        let window = AppWindowId::default();
        let mut app = App::new();

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(400.0), Px(240.0)),
        );

        app.set_tick_id(TickId(1));
        app.set_frame_id(FrameId(1));
        let closed =
            fret_ui::elements::with_element_cx(&mut app, window, bounds, "nav-menu", |cx| {
                drive_navigation_menu_trigger_chevron_motion(cx, Arc::<str>::from("alpha"), false)
            });
        assert!(!closed.present);
        assert_eq!(closed.progress, 0.0);
        assert!(!closed.animating);

        let expected_ticks = ticks_60hz_for_duration(Duration::from_millis(300));
        let mut frames = 0u64;
        let mut last_progress = 0.0f32;
        loop {
            frames += 1;
            app.set_tick_id(TickId(1 + frames));
            app.set_frame_id(FrameId(1 + frames));

            let out =
                fret_ui::elements::with_element_cx(&mut app, window, bounds, "nav-menu", |cx| {
                    drive_navigation_menu_trigger_chevron_motion(
                        cx,
                        Arc::<str>::from("alpha"),
                        true,
                    )
                });

            assert!(
                out.present,
                "expected chevron transition to be present while open=true"
            );
            assert!(
                out.progress + 1e-6 >= last_progress,
                "expected chevron progress to be monotonic (last={last_progress} now={})",
                out.progress
            );
            last_progress = out.progress;

            if !out.animating {
                assert!(
                    (out.progress - 1.0).abs() <= 1e-3,
                    "expected chevron to settle at progress=1 (got {})",
                    out.progress
                );
                break;
            }

            assert!(
                frames <= expected_ticks + 12,
                "expected chevron transition to settle near 300ms (ticks={expected_ticks}, frames={frames})"
            );
        }

        // Closing should animate back toward 0 and settle deterministically.
        let mut frames = 0u64;
        loop {
            frames += 1;
            app.set_tick_id(TickId(10_000 + frames));
            app.set_frame_id(FrameId(10_000 + frames));

            let out =
                fret_ui::elements::with_element_cx(&mut app, window, bounds, "nav-menu", |cx| {
                    drive_navigation_menu_trigger_chevron_motion(
                        cx,
                        Arc::<str>::from("alpha"),
                        false,
                    )
                });

            if !out.animating {
                assert!(!out.present);
                assert!(
                    out.progress.abs() <= 1e-3,
                    "expected chevron to settle at progress=0 (got {})",
                    out.progress
                );
                break;
            }

            assert!(
                frames <= expected_ticks + 12,
                "expected chevron close transition to settle near 300ms (ticks={expected_ticks}, frames={frames})"
            );
        }
    }

    fn assert_align_start_for_desired_width(
        dir: LayoutDirection,
        anchor: Rect,
        panel: Rect,
        desired_width: Px,
    ) {
        let eps = 0.75;
        match dir {
            LayoutDirection::Ltr => {
                assert!(
                    (panel.origin.x.0 - anchor.origin.x.0).abs() <= eps,
                    "expected LTR start alignment (panel.left == anchor.left); anchor={anchor:?} panel={panel:?}",
                );
            }
            LayoutDirection::Rtl => {
                let anchor_right = anchor.origin.x.0 + anchor.size.width.0;
                let expected_left = anchor_right - desired_width.0;
                assert!(
                    (panel.origin.x.0 - expected_left).abs() <= eps,
                    "expected RTL start alignment (panel.left == anchor.right - desired_width); desired_width={desired_width:?} anchor={anchor:?} panel={panel:?}",
                );
            }
        }
    }

    #[test]
    fn navigation_menu_delay_aliases_update_config() {
        let mut app = App::new();
        let model = app.models_mut().insert(None::<Arc<str>>);
        let menu = NavigationMenu::new(model)
            .delay_ms(120)
            .close_delay_ms(80)
            .skip_delay_ms(360);

        assert_eq!(
            menu.config.delay_duration,
            std::time::Duration::from_millis(120)
        );
        assert_eq!(
            menu.config.close_delay_duration,
            std::time::Duration::from_millis(80)
        );
        assert_eq!(
            menu.config.skip_delay_duration,
            std::time::Duration::from_millis(360)
        );
    }

    #[test]
    fn navigation_menu_trigger_style_applies_h_9_and_flex_shrink_0() {
        let app = App::new();
        let theme = Theme::global(&app).snapshot();
        let style = navigation_menu_trigger_style(&theme);
        let layout = decl_style::layout_style(&theme, style.layout);

        assert_eq!(layout.size.height, Length::Px(Px(36.0)));
        assert_eq!(layout.flex.shrink, 0.0);
    }

    #[test]
    fn navigation_menu_duration_aliases_update_config() {
        let mut app = App::new();
        let model = app.models_mut().insert(None::<Arc<str>>);
        let menu = NavigationMenu::new(model)
            .delay_duration(std::time::Duration::from_millis(10))
            .close_delay_duration(std::time::Duration::from_millis(20))
            .skip_delay_duration(std::time::Duration::from_millis(30));

        assert_eq!(
            menu.config.delay_duration,
            std::time::Duration::from_millis(10)
        );
        assert_eq!(
            menu.config.close_delay_duration,
            std::time::Duration::from_millis(20)
        );
        assert_eq!(
            menu.config.skip_delay_duration,
            std::time::Duration::from_millis(30)
        );
    }

    #[test]
    fn navigation_menu_on_open_change_complete_builder_sets_handler() {
        let mut app = App::new();
        let model = app.models_mut().insert(None::<Arc<str>>);
        let menu = NavigationMenu::new(model).on_open_change_complete(Some(Arc::new(|_open| {})));

        assert!(menu.on_open_change_complete.is_some());
    }

    #[test]
    fn navigation_menu_on_value_change_builder_sets_handler() {
        let mut app = App::new();
        let model = app.models_mut().insert(None::<Arc<str>>);
        let menu = NavigationMenu::new(model).on_value_change(Some(Arc::new(|_value| {})));

        assert!(menu.on_value_change.is_some());
    }

    #[test]
    fn navigation_menu_value_change_event_emits_only_on_state_change() {
        let mut state = NavigationMenuValueChangeCallbackState::default();

        let changed = navigation_menu_value_change_event(&mut state, None);
        assert_eq!(changed, None);

        let changed = navigation_menu_value_change_event(&mut state, Some(Arc::from("components")));
        assert_eq!(
            changed
                .as_ref()
                .and_then(|v| v.as_ref().map(|s| s.as_ref())),
            Some("components")
        );

        let changed = navigation_menu_value_change_event(&mut state, Some(Arc::from("components")));
        assert_eq!(changed, None);

        let changed = navigation_menu_value_change_event(&mut state, Some(Arc::from("docs")));
        assert_eq!(
            changed
                .as_ref()
                .and_then(|v| v.as_ref().map(|s| s.as_ref())),
            Some("docs")
        );

        let changed = navigation_menu_value_change_event(&mut state, None);
        assert_eq!(changed, Some(None));
    }

    #[test]
    fn navigation_menu_open_change_complete_event_emits_after_settle() {
        let mut state = NavigationMenuOpenChangeCallbackState::default();

        let completed = navigation_menu_open_change_complete_event(&mut state, false, false, false);
        assert_eq!(completed, None);

        let completed = navigation_menu_open_change_complete_event(&mut state, true, true, true);
        assert_eq!(completed, None);

        let completed = navigation_menu_open_change_complete_event(&mut state, true, true, false);
        assert_eq!(completed, Some(true));

        let completed = navigation_menu_open_change_complete_event(&mut state, true, true, false);
        assert_eq!(completed, None);

        let completed = navigation_menu_open_change_complete_event(&mut state, false, true, true);
        assert_eq!(completed, None);

        let completed = navigation_menu_open_change_complete_event(&mut state, false, false, false);
        assert_eq!(completed, Some(false));

        let completed = navigation_menu_open_change_complete_event(&mut state, false, false, false);
        assert_eq!(completed, None);
    }

    #[test]
    fn navigation_menu_open_change_complete_event_completes_without_animation() {
        let mut state = NavigationMenuOpenChangeCallbackState::default();

        let _ = navigation_menu_open_change_complete_event(&mut state, false, false, false);
        let completed = navigation_menu_open_change_complete_event(&mut state, true, true, false);
        assert_eq!(completed, Some(true));

        let completed = navigation_menu_open_change_complete_event(&mut state, false, false, false);
        assert_eq!(completed, Some(false));
    }

    #[derive(Clone, Copy, Debug, Default)]
    struct NavigationMenuRenderedWidths {
        container_width: Option<Px>,
        viewport_panel_width: Option<Px>,
    }

    fn render_menu_and_get_rendered_widths(
        ui: &mut UiTree<App>,
        app: &mut App,
        services: &mut FakeServices,
        window: AppWindowId,
        bounds: Rect,
        model: Model<Option<Arc<str>>>,
        md_breakpoint_query: NavigationMenuMdBreakpointQuery,
        settle_frames: usize,
    ) -> NavigationMenuRenderedWidths {
        let mut widths = NavigationMenuRenderedWidths::default();
        let settle_frames = settle_frames.max(1);

        for _ in 0..settle_frames {
            let model_for_frame = model.clone();
            let md_breakpoint_query_for_frame = md_breakpoint_query;

            bump_frame(app);
            OverlayController::begin_frame(app, window);
            let root = fret_ui::declarative::render_root(
                ui,
                app,
                services,
                window,
                bounds,
                "navigation-menu-breakpoint",
                move |cx| {
                    let menu_width = Px(400.0);
                    let content_width = Px(720.0);

                    let wrapper = ContainerProps {
                        layout: {
                            let mut layout = LayoutStyle::default();
                            layout.size.width = Length::Px(menu_width);
                            layout.size.height = Length::Fill;
                            layout
                        },
                        ..Default::default()
                    };

                    let wrapper = cx.container(wrapper, move |cx| {
                        let items = vec![NavigationMenuItem::new(
                            "alpha",
                            "Alpha",
                            vec![cx.container(
                                ContainerProps {
                                    layout: {
                                        let mut layout = LayoutStyle::default();
                                        layout.size.width = Length::Px(content_width);
                                        layout.size.height = Length::Px(Px(40.0));
                                        layout
                                    },
                                    ..Default::default()
                                },
                                |_cx| Vec::<AnyElement>::new(),
                            )],
                        )];

                        vec![
                            NavigationMenu::new(model_for_frame.clone())
                                .items(items)
                                .viewport_test_id("nav.viewport")
                                .md_breakpoint_query(md_breakpoint_query_for_frame)
                                .refine_layout(LayoutRefinement::default().w_full())
                                .into_element(cx),
                        ]
                    });

                    vec![wrapper.attach_semantics(
                        SemanticsDecoration::default().test_id(Arc::<str>::from("nav.container")),
                    )]
                },
            );
            ui.set_root(root);
            OverlayController::render(ui, app, services, window, bounds);
            ui.request_semantics_snapshot();
            ui.layout_all(app, services, bounds, 1.0);

            if let Some(snap) = ui.semantics_snapshot() {
                widths.container_width = snap
                    .nodes
                    .iter()
                    .find(|n| n.test_id.as_deref() == Some("nav.container"))
                    .map(|n| n.bounds.size.width);
                widths.viewport_panel_width = snap
                    .nodes
                    .iter()
                    .find(|n| n.test_id.as_deref() == Some("nav.viewport"))
                    .map(|n| n.bounds.size.width);
            }
        }

        widths
    }

    #[test]
    fn navigation_menu_md_breakpoint_query_can_follow_viewport_or_container_width() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let model = app.models_mut().insert(Some(Arc::<str>::from("alpha")));

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(1000.0), Px(320.0)),
        );
        let mut services = FakeServices::default();

        // Some widths depend on committed element bounds (observed across frames). Render a few
        // frames to let overlays open and measurements settle.
        let settle_frames = 5;

        let viewport_widths = render_menu_and_get_rendered_widths(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            model.clone(),
            NavigationMenuMdBreakpointQuery::Viewport,
            settle_frames,
        );
        let container_w = viewport_widths
            .container_width
            .expect("expected container semantics node");
        let panel_w_viewport = viewport_widths
            .viewport_panel_width
            .expect("expected viewport panel semantics node (viewport query)");

        let container_widths = render_menu_and_get_rendered_widths(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            model.clone(),
            NavigationMenuMdBreakpointQuery::Container,
            settle_frames,
        );
        let panel_w_container = container_widths
            .viewport_panel_width
            .expect("expected viewport panel semantics node (container query)");

        assert!(
            panel_w_viewport.0 > container_w.0 + 200.0,
            "expected viewport-md sizing to follow content width; got {panel_w_viewport:?}",
        );
        assert!(
            (panel_w_container.0 - container_w.0).abs() <= 2.0,
            "expected container-md sizing to keep mobile w-full behavior (anchor width {container_w:?}); got {panel_w_container:?}",
        );
    }

    #[test]
    fn hovering_trigger_opens_after_delay_like_radix() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let model = app.models_mut().insert(None::<Arc<str>>);

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(400.0), Px(240.0)),
        );
        let mut services = FakeServices::default();

        bump_frame(&mut app);
        let root = fret_ui::declarative::render_root(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "navigation-menu",
            |cx| {
                let items = vec![
                    NavigationMenuItem::new("alpha", "Alpha", vec![cx.text("A")]),
                    NavigationMenuItem::new("beta", "Beta", vec![cx.text("B")]),
                ];
                vec![
                    NavigationMenu::new(model.clone())
                        .items(items)
                        .into_element(cx),
                ]
            },
        );
        ui.set_root(root);
        ui.request_semantics_snapshot();
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        let snap = ui.semantics_snapshot().expect("semantics snapshot");
        let alpha_btn = snap
            .nodes
            .iter()
            .find(|n| n.role == SemanticsRole::Button && n.label.as_deref() == Some("Alpha"))
            .expect("alpha button semantics");
        let pos = Point::new(
            Px(alpha_btn.bounds.origin.x.0 + alpha_btn.bounds.size.width.0 * 0.5),
            Px(alpha_btn.bounds.origin.y.0 + alpha_btn.bounds.size.height.0 * 0.5),
        );

        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::Pointer(PointerEvent::Move {
                pointer_id: fret_core::PointerId(0),
                position: pos,
                buttons: MouseButtons::default(),
                modifiers: Modifiers::default(),
                pointer_type: PointerType::Mouse,
            }),
        );

        let effects = app.flush_effects();
        let token = effects
            .iter()
            .find_map(|e| match e {
                fret_runtime::Effect::SetTimer { token, after, .. }
                    if *after
                        == radix_navigation_menu::NavigationMenuConfig::default()
                            .delay_duration =>
                {
                    Some(*token)
                }
                _ => None,
            })
            .expect("expected delayed-open timer");

        ui.dispatch_event(&mut app, &mut services, &fret_core::Event::Timer { token });

        let selected = app.models().get_cloned(&model).flatten();
        assert_eq!(selected.as_deref(), Some("alpha"));
    }

    #[test]
    fn viewport_disabled_still_opens_after_delay_like_radix() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let model = app.models_mut().insert(None::<Arc<str>>);

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(400.0), Px(240.0)),
        );
        let mut services = FakeServices::default();

        bump_frame(&mut app);
        let root = fret_ui::declarative::render_root(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "navigation-menu-no-viewport",
            |cx| {
                let items = vec![
                    NavigationMenuItem::new("alpha", "Alpha", vec![cx.text("A")]),
                    NavigationMenuItem::new("beta", "Beta", vec![cx.text("B")]),
                ];
                vec![
                    NavigationMenu::new(model.clone())
                        .viewport(false)
                        .items(items)
                        .into_element(cx),
                ]
            },
        );
        ui.set_root(root);
        ui.request_semantics_snapshot();
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        let snap = ui.semantics_snapshot().expect("semantics snapshot");
        let alpha_btn = snap
            .nodes
            .iter()
            .find(|n| n.role == SemanticsRole::Button && n.label.as_deref() == Some("Alpha"))
            .expect("alpha button semantics");
        let pos = Point::new(
            Px(alpha_btn.bounds.origin.x.0 + alpha_btn.bounds.size.width.0 * 0.5),
            Px(alpha_btn.bounds.origin.y.0 + alpha_btn.bounds.size.height.0 * 0.5),
        );

        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::Pointer(PointerEvent::Move {
                pointer_id: fret_core::PointerId(0),
                position: pos,
                buttons: MouseButtons::default(),
                modifiers: Modifiers::default(),
                pointer_type: PointerType::Mouse,
            }),
        );

        let effects = app.flush_effects();
        let token = effects
            .iter()
            .find_map(|e| match e {
                fret_runtime::Effect::SetTimer { token, after, .. }
                    if *after
                        == radix_navigation_menu::NavigationMenuConfig::default()
                            .delay_duration =>
                {
                    Some(*token)
                }
                _ => None,
            })
            .expect("expected delayed-open timer");

        ui.dispatch_event(&mut app, &mut services, &fret_core::Event::Timer { token });

        let selected = app.models().get_cloned(&model).flatten();
        assert_eq!(selected.as_deref(), Some("alpha"));
    }

    #[test]
    fn escape_close_sets_trigger_gate_and_does_not_reopen_on_pointer_move() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let model = app.models_mut().insert(None::<Arc<str>>);

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(400.0), Px(240.0)),
        );
        let mut services = FakeServices::default();

        bump_frame(&mut app);
        let root = fret_ui::declarative::render_root(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "navigation-menu-escape",
            |cx| {
                let items = vec![
                    NavigationMenuItem::new("alpha", "Alpha", vec![cx.text("A")]),
                    NavigationMenuItem::new("beta", "Beta", vec![cx.text("B")]),
                ];
                vec![
                    NavigationMenu::new(model.clone())
                        .items(items)
                        .into_element(cx),
                ]
            },
        );
        ui.set_root(root);
        ui.request_semantics_snapshot();
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        let snap = ui.semantics_snapshot().expect("semantics snapshot");
        let alpha_btn = snap
            .nodes
            .iter()
            .find(|n| n.role == SemanticsRole::Button && n.label.as_deref() == Some("Alpha"))
            .expect("alpha button semantics");
        let pos = Point::new(
            Px(alpha_btn.bounds.origin.x.0 + alpha_btn.bounds.size.width.0 * 0.5),
            Px(alpha_btn.bounds.origin.y.0 + alpha_btn.bounds.size.height.0 * 0.5),
        );

        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::Pointer(PointerEvent::Down {
                pointer_id: fret_core::PointerId(0),
                position: pos,
                button: MouseButton::Left,
                modifiers: Modifiers::default(),
                pointer_type: PointerType::Mouse,
                click_count: 1,
            }),
        );
        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::Pointer(PointerEvent::Up {
                pointer_id: fret_core::PointerId(0),
                position: pos,
                button: MouseButton::Left,
                modifiers: Modifiers::default(),
                is_click: true,
                click_count: 1,
                pointer_type: PointerType::Mouse,
            }),
        );

        let selected = app.models().get_cloned(&model).flatten();
        assert_eq!(selected.as_deref(), Some("alpha"));

        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::KeyDown {
                key: KeyCode::Escape,
                modifiers: Modifiers::default(),
                repeat: false,
            },
        );
        let selected = app.models().get_cloned(&model).flatten();
        assert_eq!(selected, None);

        app.flush_effects();

        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::Pointer(PointerEvent::Move {
                pointer_id: fret_core::PointerId(0),
                position: pos,
                buttons: MouseButtons::default(),
                modifiers: Modifiers::default(),
                pointer_type: PointerType::Mouse,
            }),
        );

        let effects = app.flush_effects();
        let has_open_timer = effects.iter().any(|e| matches!(e,
            fret_runtime::Effect::SetTimer { after, .. }
                if *after == radix_navigation_menu::NavigationMenuConfig::default().delay_duration
        ));
        assert!(
            !has_open_timer,
            "expected no delayed-open timer after escape gating"
        );
    }

    #[test]
    fn horizontal_arrow_keys_move_focus_between_triggers_like_radix() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let model = app.models_mut().insert(None::<Arc<str>>);

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(520.0), Px(320.0)),
        );
        let mut services = FakeServices::default();

        bump_frame(&mut app);
        let root = fret_ui::declarative::render_root(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "navigation-menu-roving",
            |cx| {
                let items = vec![
                    NavigationMenuItem::new("alpha", "Alpha", std::iter::empty()),
                    NavigationMenuItem::new("beta", "Beta", std::iter::empty()),
                ];
                vec![
                    NavigationMenu::new(model.clone())
                        .items(items)
                        .into_element(cx),
                ]
            },
        );
        ui.set_root(root);
        ui.request_semantics_snapshot();
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        let (alpha_id, beta_id) = {
            let snap = ui.semantics_snapshot().expect("semantics snapshot");
            let alpha_id = snap
                .nodes
                .iter()
                .find(|n| n.role == SemanticsRole::Link && n.label.as_deref() == Some("Alpha"))
                .or_else(|| {
                    snap.nodes.iter().find(|n| {
                        n.role == SemanticsRole::Button && n.label.as_deref() == Some("Alpha")
                    })
                })
                .expect("Alpha trigger semantics")
                .id;
            let beta_id = snap
                .nodes
                .iter()
                .find(|n| n.role == SemanticsRole::Link && n.label.as_deref() == Some("Beta"))
                .or_else(|| {
                    snap.nodes.iter().find(|n| {
                        n.role == SemanticsRole::Button && n.label.as_deref() == Some("Beta")
                    })
                })
                .expect("Beta trigger semantics")
                .id;
            (alpha_id, beta_id)
        };

        ui.set_focus(Some(alpha_id));
        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::KeyDown {
                key: KeyCode::ArrowRight,
                modifiers: Modifiers::default(),
                repeat: false,
            },
        );

        ui.request_semantics_snapshot();
        ui.layout_all(&mut app, &mut services, bounds, 1.0);
        let snap = ui.semantics_snapshot().expect("semantics snapshot");
        assert_eq!(snap.focus, Some(beta_id));
    }

    #[test]
    fn entry_key_focuses_first_link_like_radix() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let model = app.models_mut().insert(None::<Arc<str>>);

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(520.0), Px(320.0)),
        );
        let mut services = FakeServices::default();

        let render_frame = |ui: &mut UiTree<App>,
                            app: &mut App,
                            services: &mut FakeServices,
                            frame: u64| {
            app.set_tick_id(TickId(frame));
            app.set_frame_id(FrameId(frame));
            OverlayController::begin_frame(app, window);
            let model_for_render = model.clone();
            let root = fret_ui::declarative::render_root(
                ui,
                app,
                services,
                window,
                bounds,
                "navigation-menu-entry-key",
                move |cx| {
                    let content = vec![
                        NavigationMenuLink::new(model_for_render.clone(), vec![cx.text("Go")])
                            .label("Go")
                            .into_element(cx),
                        NavigationMenuLink::new(model_for_render.clone(), vec![cx.text("Later")])
                            .label("Later")
                            .into_element(cx),
                    ];
                    let items = vec![
                        NavigationMenuItem::new("alpha", "Alpha", content),
                        NavigationMenuItem::new("docs", "Docs", std::iter::empty()),
                    ];
                    vec![
                        NavigationMenu::new(model_for_render.clone())
                            .items(items)
                            .into_element(cx),
                    ]
                },
            );
            ui.set_root(root);
            OverlayController::render(ui, app, services, window, bounds);
            ui.request_semantics_snapshot();
            ui.layout_all(app, services, bounds, 1.0);
        };

        render_frame(&mut ui, &mut app, &mut services, 1);

        let snap = ui.semantics_snapshot().expect("semantics snapshot");
        let alpha_btn = snap
            .nodes
            .iter()
            .find(|n| n.role == SemanticsRole::Button && n.label.as_deref() == Some("Alpha"))
            .expect("alpha button semantics");
        let pos = Point::new(
            Px(alpha_btn.bounds.origin.x.0 + alpha_btn.bounds.size.width.0 * 0.5),
            Px(alpha_btn.bounds.origin.y.0 + alpha_btn.bounds.size.height.0 * 0.5),
        );

        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::Pointer(PointerEvent::Move {
                pointer_id: fret_core::PointerId(0),
                position: pos,
                buttons: MouseButtons::default(),
                modifiers: Modifiers::default(),
                pointer_type: PointerType::Mouse,
            }),
        );

        let open_token = app
            .flush_effects()
            .iter()
            .find_map(|e| match e {
                fret_runtime::Effect::SetTimer { token, after, .. }
                    if *after
                        == radix_navigation_menu::NavigationMenuConfig::default()
                            .delay_duration =>
                {
                    Some(*token)
                }
                _ => None,
            })
            .expect("expected delayed-open timer");

        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::Timer { token: open_token },
        );

        render_frame(&mut ui, &mut app, &mut services, 2);

        let snap = ui.semantics_snapshot().expect("semantics snapshot");
        let alpha_btn = snap
            .nodes
            .iter()
            .find(|n| n.role == SemanticsRole::Button && n.label.as_deref() == Some("Alpha"))
            .expect("alpha button semantics");
        ui.set_focus(Some(alpha_btn.id));

        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::KeyDown {
                key: KeyCode::ArrowDown,
                modifiers: Modifiers::default(),
                repeat: false,
            },
        );

        let cmd = app
            .flush_effects()
            .iter()
            .find_map(|e| match e {
                fret_runtime::Effect::Command { command, .. }
                    if command.as_str() == "focus.next" =>
                {
                    Some(command.clone())
                }
                _ => None,
            })
            .expect("expected focus.next command effect");
        let _ = ui.dispatch_command(&mut app, &mut services, &cmd);

        render_frame(&mut ui, &mut app, &mut services, 3);
        let snap = ui.semantics_snapshot().expect("semantics snapshot");

        let go_link = snap
            .nodes
            .iter()
            .find(|n| n.role == SemanticsRole::Link && n.label.as_deref() == Some("Go"))
            .expect("Go link semantics");
        assert_eq!(snap.focus, Some(go_link.id));
    }

    #[test]
    fn navigation_menu_link_does_not_dismiss_on_ctrl_click() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let model = app.models_mut().insert(Some(Arc::from("alpha")));

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(400.0), Px(240.0)),
        );
        let mut services = FakeServices::default();

        bump_frame(&mut app);
        let root = fret_ui::declarative::render_root(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "navigation-menu-link",
            |cx| {
                vec![
                    NavigationMenuLink::new(model.clone(), vec![cx.text("Go")])
                        .label("Go")
                        .into_element(cx),
                ]
            },
        );
        ui.set_root(root);
        ui.request_semantics_snapshot();
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        let snap = ui.semantics_snapshot().expect("semantics snapshot");
        let go_link = snap
            .nodes
            .iter()
            .find(|n| n.role == SemanticsRole::Link && n.label.as_deref() == Some("Go"))
            .expect("Go link semantics");
        let pos = Point::new(
            Px(go_link.bounds.origin.x.0 + go_link.bounds.size.width.0 * 0.5),
            Px(go_link.bounds.origin.y.0 + go_link.bounds.size.height.0 * 0.5),
        );

        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::Pointer(PointerEvent::Down {
                pointer_id: fret_core::PointerId(0),
                position: pos,
                button: MouseButton::Left,
                modifiers: Modifiers {
                    ctrl: true,
                    ..Default::default()
                },
                pointer_type: PointerType::Mouse,
                click_count: 1,
            }),
        );
        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::Pointer(PointerEvent::Up {
                pointer_id: fret_core::PointerId(0),
                position: pos,
                button: MouseButton::Left,
                modifiers: Modifiers {
                    ctrl: true,
                    ..Default::default()
                },
                is_click: true,
                click_count: 1,
                pointer_type: PointerType::Mouse,
            }),
        );

        let selected = app.models().get_cloned(&model).flatten();
        assert_eq!(selected.as_deref(), Some("alpha"));

        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::Pointer(PointerEvent::Down {
                pointer_id: fret_core::PointerId(0),
                position: pos,
                button: MouseButton::Left,
                modifiers: Modifiers::default(),
                pointer_type: PointerType::Mouse,
                click_count: 1,
            }),
        );
        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::Pointer(PointerEvent::Up {
                pointer_id: fret_core::PointerId(0),
                position: pos,
                button: MouseButton::Left,
                modifiers: Modifiers::default(),
                is_click: true,
                click_count: 1,
                pointer_type: PointerType::Mouse,
            }),
        );

        let selected = app.models().get_cloned(&model).flatten();
        assert_eq!(selected, None);
    }

    #[test]
    fn navigation_menu_link_default_icons_do_not_inherit_current_color() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(400.0), Px(240.0)),
        );

        let model = app.models_mut().insert(None::<Arc<str>>);

        let element = fret_ui::elements::with_element_cx(&mut app, window, bounds, "test", |cx| {
            NavigationMenuLink::new(
                model.clone(),
                [decl_icon::icon(cx, ids::ui::CHEVRON_DOWN), cx.text("Go")],
            )
            .into_element(cx)
        });

        fn find_svg_icon(el: &AnyElement) -> Option<&fret_ui::element::SvgIconProps> {
            match &el.kind {
                fret_ui::element::ElementKind::SvgIcon(props) => Some(props),
                _ => el.children.iter().find_map(find_svg_icon),
            }
        }

        let icon = find_svg_icon(&element).expect("expected an SvgIcon under NavigationMenuLink");
        assert!(
            !icon.inherit_color,
            "expected default link icon to opt out of inheriting currentColor"
        );
    }

    #[test]
    fn navigation_menu_indicator_can_be_disabled() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let model = app.models_mut().insert::<Option<Arc<str>>>(None);

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(400.0), Px(240.0)),
        );
        let mut services = FakeServices::default();

        bump_frame(&mut app);
        let root = fret_ui::declarative::render_root(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "navigation-menu-indicator-off",
            |cx| {
                let items = vec![
                    NavigationMenuItem::new("alpha", "Alpha", vec![cx.text("A")]),
                    NavigationMenuItem::new("beta", "Beta", vec![cx.text("B")]),
                ];
                vec![
                    NavigationMenu::new(model.clone())
                        .indicator(false)
                        .items(items)
                        .into_element(cx),
                ]
            },
        );
        ui.set_root(root);
        ui.request_semantics_snapshot();
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        let snap = ui.semantics_snapshot().expect("semantics snapshot");
        let alpha_btn = snap
            .nodes
            .iter()
            .find(|n| n.role == SemanticsRole::Button && n.label.as_deref() == Some("Alpha"))
            .expect("alpha button semantics");
        let pos = Point::new(
            Px(alpha_btn.bounds.origin.x.0 + alpha_btn.bounds.size.width.0 * 0.5),
            Px(alpha_btn.bounds.origin.y.0 + alpha_btn.bounds.size.height.0 * 0.5),
        );

        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::Pointer(PointerEvent::Down {
                pointer_id: fret_core::PointerId(0),
                position: pos,
                button: MouseButton::Left,
                modifiers: Modifiers::default(),
                pointer_type: PointerType::Mouse,
                click_count: 1,
            }),
        );
        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::Pointer(PointerEvent::Up {
                pointer_id: fret_core::PointerId(0),
                position: pos,
                button: MouseButton::Left,
                modifiers: Modifiers::default(),
                is_click: true,
                click_count: 1,
                pointer_type: PointerType::Mouse,
            }),
        );

        let selected = app.models().get_cloned(&model).flatten();
        assert_eq!(selected.as_deref(), Some("alpha"));
    }

    #[test]
    fn navigation_menu_keeps_open_while_pointer_moves_through_safe_corridor() {
        fn center(rect: Rect) -> Point {
            Point::new(
                Px(rect.origin.x.0 + rect.size.width.0 * 0.5),
                Px(rect.origin.y.0 + rect.size.height.0 * 0.5),
            )
        }

        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let model = app.models_mut().insert(None::<Arc<str>>);
        let nav_cfg = radix_navigation_menu::NavigationMenuConfig::default();

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(800.0), Px(600.0)),
        );
        let mut services = FakeServices::default();

        let render_frame =
            |ui: &mut UiTree<App>, app: &mut App, services: &mut FakeServices, frame: u64| {
                app.set_tick_id(TickId(frame));
                app.set_frame_id(FrameId(frame));
                OverlayController::begin_frame(app, window);
                let model_for_render = model.clone();
                let root = fret_ui::declarative::render_root(
                    ui,
                    app,
                    services,
                    window,
                    bounds,
                    "navigation-menu-safe-corridor",
                    move |cx| {
                        let nav_cfg_for_render = nav_cfg;
                        let items = vec![
                            NavigationMenuItem::new("alpha", "Alpha", vec![cx.text("A")]),
                            NavigationMenuItem::new("beta", "Beta", vec![cx.text("B")]),
                        ];
                        vec![
                            NavigationMenu::new(model_for_render.clone())
                                .config(nav_cfg_for_render)
                                .items(items)
                                .into_element(cx),
                        ]
                    },
                );
                ui.set_root(root);
                OverlayController::render(ui, app, services, window, bounds);
                ui.request_semantics_snapshot();
                ui.layout_all(app, services, bounds, 1.0);
            };

        render_frame(&mut ui, &mut app, &mut services, 1);

        let snap = ui.semantics_snapshot().expect("semantics snapshot");
        let alpha_btn = snap
            .nodes
            .iter()
            .find(|n| n.role == SemanticsRole::Button && n.label.as_deref() == Some("Alpha"))
            .expect("alpha button semantics");
        let alpha_bounds = alpha_btn.bounds;
        let trigger_center = center(alpha_bounds);

        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::Pointer(PointerEvent::Move {
                pointer_id: fret_core::PointerId(0),
                position: trigger_center,
                buttons: MouseButtons::default(),
                modifiers: Modifiers::default(),
                pointer_type: PointerType::Mouse,
            }),
        );

        let open_token = app
            .flush_effects()
            .iter()
            .find_map(|e| match e {
                fret_runtime::Effect::SetTimer { token, after, .. }
                    if *after == nav_cfg.delay_duration =>
                {
                    Some(*token)
                }
                _ => None,
            })
            .expect("expected delayed-open timer");

        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::Timer { token: open_token },
        );

        render_frame(&mut ui, &mut app, &mut services, 2);
        render_frame(&mut ui, &mut app, &mut services, 3);
        let selected = app.models().get_cloned(&model).flatten();
        assert_eq!(selected.as_deref(), Some("alpha"));

        let overlay_stack = OverlayController::stack_snapshot_for_window(&ui, &mut app, window);
        let root_id = overlay_stack
            .topmost_popover
            .expect("expected navigation menu viewport overlay root id");
        let viewport_panel_id = fret_ui::elements::with_element_cx(
            &mut app,
            window,
            bounds,
            "navigation-menu-safe-corridor",
            |cx| radix_navigation_menu::navigation_menu_viewport_panel_id(cx, root_id),
        )
        .expect("viewport panel id");
        let viewport_panel =
            fret_ui::elements::bounds_for_element(&mut app, window, viewport_panel_id)
                .expect("viewport panel bounds");

        let transit_point = Point::new(
            Px(alpha_bounds.origin.x.0 + alpha_bounds.size.width.0 * 0.5),
            Px(alpha_bounds.origin.y.0 + alpha_bounds.size.height.0 + 2.0),
        );
        assert!(
            !alpha_bounds.contains(transit_point),
            "transit point should be outside trigger bounds"
        );
        assert!(
            !viewport_panel.contains(transit_point),
            "transit point should be outside viewport panel bounds"
        );

        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::Pointer(PointerEvent::Move {
                pointer_id: fret_core::PointerId(0),
                position: transit_point,
                buttons: MouseButtons::default(),
                modifiers: Modifiers::default(),
                pointer_type: PointerType::Mouse,
            }),
        );

        render_frame(&mut ui, &mut app, &mut services, 4);
        let selected = app.models().get_cloned(&model).flatten();
        assert_eq!(selected.as_deref(), Some("alpha"));

        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::Pointer(PointerEvent::Move {
                pointer_id: fret_core::PointerId(0),
                position: Point::new(Px(760.0), Px(560.0)),
                buttons: MouseButtons::default(),
                modifiers: Modifiers::default(),
                pointer_type: PointerType::Mouse,
            }),
        );

        render_frame(&mut ui, &mut app, &mut services, 5);
        render_frame(&mut ui, &mut app, &mut services, 6);

        let close_token = app
            .flush_effects()
            .iter()
            .find_map(|e| match e {
                fret_runtime::Effect::SetTimer { token, after, .. }
                    if *after == nav_cfg.close_delay_duration =>
                {
                    Some(*token)
                }
                _ => None,
            })
            .expect("expected delayed-close timer");

        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::Timer { token: close_token },
        );
        render_frame(&mut ui, &mut app, &mut services, 7);

        let selected = app.models().get_cloned(&model).flatten();
        assert_eq!(selected, None);
    }

    #[test]
    fn navigation_menu_viewport_align_start_respects_direction_provider() {
        fn run(dir: LayoutDirection) -> (Rect, Rect) {
            let window = AppWindowId::default();
            let mut app = App::new();
            let mut ui: UiTree<App> = UiTree::new();
            ui.set_window(window);

            let model = app.models_mut().insert(Some(Arc::from("alpha")));

            let bounds = Rect::new(
                Point::new(Px(0.0), Px(0.0)),
                Size::new(Px(1200.0), Px(700.0)),
            );
            let mut services = FakeServices::default();

            let render_frame = |ui: &mut UiTree<App>,
                                app: &mut App,
                                services: &mut FakeServices| {
                bump_frame(app);
                OverlayController::begin_frame(app, window);
                let model_for_render = model.clone();
                let root = fret_ui::declarative::render_root(
                    ui,
                    app,
                    services,
                    window,
                    bounds,
                    "navigation-menu-dir",
                    move |cx| {
                        direction_prim::with_direction_provider(cx, dir, |cx| {
                            let theme = Theme::global(&*cx.app).snapshot();
                            let region_props = LayoutQueryRegionProps {
                                layout: decl_style::layout_style(
                                    &theme,
                                    LayoutRefinement::default().w_full().min_w_0(),
                                ),
                                name: None,
                            };

                            vec![cx.layout_query_region_with_id(
                                region_props,
                                move |cx, region_id| {
                                    vec![
                                        cx.container(
                                            ContainerProps {
                                                layout: LayoutStyle {
                                                    size: SizeStyle {
                                                        width: Length::Fill,
                                                        ..Default::default()
                                                    },
                                                    ..Default::default()
                                                },
                                                padding: Edges {
                                                    top: Px(100.0),
                                                    right: Px(0.0),
                                                    bottom: Px(0.0),
                                                    left: Px(500.0),
                                                }
                                                .into(),
                                                ..Default::default()
                                            },
                                            move |cx| {
                                                let items = vec![
                                                    NavigationMenuItem::new(
                                                        "alpha",
                                                        "Alpha",
                                                        vec![cx.text("A")],
                                                    ),
                                                    NavigationMenuItem::new(
                                                        "beta",
                                                        "Beta",
                                                        vec![cx.text("B")],
                                                    ),
                                                ];
                                                vec![
                                                    NavigationMenu::new(model_for_render.clone())
                                                        .container_query_region(region_id)
                                                        .items(items)
                                                        .into_element(cx),
                                                ]
                                            },
                                        ),
                                    ]
                                },
                            )]
                        })
                    },
                );
                ui.set_root(root);
                OverlayController::render(ui, app, services, window, bounds);
                ui.request_semantics_snapshot();
                ui.layout_all(app, services, bounds, 1.0);
            };

            // Three frames:
            // - frame 1 establishes trigger/root bounds,
            // - frame 2 mounts the overlay and records viewport content bounds,
            // - frame 3 uses last-frame content bounds to drive viewport sizing.
            render_frame(&mut ui, &mut app, &mut services);
            render_frame(&mut ui, &mut app, &mut services);
            render_frame(&mut ui, &mut app, &mut services);

            let overlay_stack = OverlayController::stack_snapshot_for_window(&ui, &mut app, window);
            assert!(
                overlay_stack.topmost_popover.is_some(),
                "expected a popover overlay for the navigation menu viewport; snapshot={overlay_stack:?}"
            );
            let popover_layer = overlay_stack
                .stack
                .iter()
                .rev()
                .find(|e| e.kind == fret_ui_kit::OverlayStackEntryKind::Popover && e.visible)
                .expect("expected a visible popover layer entry");
            assert!(
                popover_layer.hit_testable,
                "expected navigation menu popover to be hit-testable; entry={popover_layer:?} snapshot={overlay_stack:?}"
            );

            let root_id = overlay_stack
                .topmost_popover
                .expect("expected a popover overlay id");
            let anchor = fret_ui::elements::bounds_for_element(&mut app, window, root_id)
                .expect("expected navigation menu root bounds");
            let viewport_panel_id = fret_ui::elements::with_element_cx(
                &mut app,
                window,
                bounds,
                "navigation-menu-dir",
                |cx| radix_navigation_menu::navigation_menu_viewport_panel_id(cx, root_id),
            )
            .expect("expected viewport panel id for navigation menu root");
            let panel = fret_ui::elements::bounds_for_element(&mut app, window, viewport_panel_id)
                .expect("expected viewport panel bounds");
            (anchor, panel)
        }

        let desired_width = Px(320.0);

        let (ltr_anchor, ltr_panel) = run(LayoutDirection::Ltr);
        assert_align_start_for_desired_width(
            LayoutDirection::Ltr,
            ltr_anchor,
            ltr_panel,
            desired_width,
        );

        let (rtl_anchor, rtl_panel) = run(LayoutDirection::Rtl);
        assert_align_start_for_desired_width(
            LayoutDirection::Rtl,
            rtl_anchor,
            rtl_panel,
            desired_width,
        );
    }

    #[test]
    fn navigation_menu_link_is_disabled_by_window_command_enabled_service() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let model = app.models_mut().insert(None::<Arc<str>>);
        let cmd = CommandId::from("test.disabled-command");
        app.commands_mut().register(
            cmd.clone(),
            CommandMeta::new("Disabled Command").with_scope(CommandScope::Widget),
        );

        app.set_global(WindowCommandEnabledService::default());
        app.with_global_mut(WindowCommandEnabledService::default, |svc, _app| {
            svc.set_enabled(window, cmd.clone(), false);
        });

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(400.0), Px(240.0)),
        );
        let mut services = FakeServices::default();

        bump_frame(&mut app);
        let root = fret_ui::declarative::render_root(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "navigation-menu-link",
            |cx| {
                vec![
                    NavigationMenuLink::new(model.clone(), vec![cx.text("Link")])
                        .label("Link")
                        .action(cmd.clone())
                        .test_id("disabled-link")
                        .into_element(cx),
                ]
            },
        );
        ui.set_root(root);
        ui.request_semantics_snapshot();
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        let snap = ui.semantics_snapshot().expect("semantics snapshot");
        let node = snap
            .nodes
            .iter()
            .find(|n| n.test_id.as_deref() == Some("disabled-link"))
            .expect("expected a semantics node for the link test_id");
        assert!(node.flags.disabled);
    }

    #[test]
    fn navigation_menu_link_is_disabled_when_widget_action_is_unavailable() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let model = app.models_mut().insert(None::<Arc<str>>);
        let cmd = CommandId::from("test.widget-action");
        app.commands_mut().register(
            cmd.clone(),
            CommandMeta::new("Widget Action").with_scope(CommandScope::Widget),
        );

        app.set_global(WindowCommandActionAvailabilityService::default());
        app.with_global_mut(
            WindowCommandActionAvailabilityService::default,
            |svc, _app| {
                let mut snapshot: HashMap<CommandId, bool> = HashMap::new();
                snapshot.insert(cmd.clone(), false);
                svc.set_snapshot(window, snapshot);
            },
        );

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(400.0), Px(240.0)),
        );
        let mut services = FakeServices::default();

        bump_frame(&mut app);
        let root = fret_ui::declarative::render_root(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "navigation-menu-link-widget-action",
            |cx| {
                vec![
                    NavigationMenuLink::new(model.clone(), vec![cx.text("Link")])
                        .label("Link")
                        .action(cmd.clone())
                        .test_id("disabled-link")
                        .into_element(cx),
                ]
            },
        );
        ui.set_root(root);
        ui.request_semantics_snapshot();
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        let snap = ui.semantics_snapshot().expect("semantics snapshot");
        let node = snap
            .nodes
            .iter()
            .find(|n| n.test_id.as_deref() == Some("disabled-link"))
            .expect("expected a semantics node for the link test_id");
        assert!(node.flags.disabled);
    }

    #[test]
    fn navigation_menu_link_prefers_window_command_gating_snapshot_when_present() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let model = app.models_mut().insert(None::<Arc<str>>);
        let cmd = CommandId::from("test.widget-action");
        app.commands_mut().register(
            cmd.clone(),
            CommandMeta::new("Widget Action").with_scope(CommandScope::Widget),
        );

        app.set_global(WindowCommandActionAvailabilityService::default());
        app.with_global_mut(
            WindowCommandActionAvailabilityService::default,
            |svc, _app| {
                let mut snapshot: HashMap<CommandId, bool> = HashMap::new();
                snapshot.insert(cmd.clone(), true);
                svc.set_snapshot(window, snapshot);
            },
        );

        app.set_global(WindowCommandGatingService::default());
        app.with_global_mut(WindowCommandGatingService::default, |svc, app| {
            let input_ctx = super::navigation_menu_input_context(app);
            let enabled_overrides: HashMap<CommandId, bool> = HashMap::new();
            let mut availability: HashMap<CommandId, bool> = HashMap::new();
            availability.insert(cmd.clone(), false);
            svc.set_snapshot(
                window,
                WindowCommandGatingSnapshot::new(input_ctx, enabled_overrides)
                    .with_action_availability(Some(Arc::new(availability))),
            );
        });

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(400.0), Px(240.0)),
        );
        let mut services = FakeServices::default();

        bump_frame(&mut app);
        let root = fret_ui::declarative::render_root(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "navigation-menu-link-gating",
            |cx| {
                vec![
                    NavigationMenuLink::new(model.clone(), vec![cx.text("Link")])
                        .label("Link")
                        .action(cmd.clone())
                        .test_id("disabled-link")
                        .into_element(cx),
                ]
            },
        );
        ui.set_root(root);
        ui.request_semantics_snapshot();
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        let snap = ui.semantics_snapshot().expect("semantics snapshot");
        let node = snap
            .nodes
            .iter()
            .find(|n| n.test_id.as_deref() == Some("disabled-link"))
            .expect("expected a semantics node for the link test_id");
        assert!(node.flags.disabled);
    }

    #[test]
    fn navigation_menu_link_href_without_on_activate_emits_open_url_effect() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let model = app.models_mut().insert(None::<Arc<str>>);
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(400.0), Px(240.0)),
        );
        let mut services = FakeServices::default();

        bump_frame(&mut app);
        let root = fret_ui::declarative::render_root(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "navigation-menu-link-href-open-url",
            |cx| {
                vec![
                    NavigationMenuLink::new(model.clone(), vec![cx.text("Docs")])
                        .label("Docs")
                        .href("https://example.com/docs")
                        .target("_blank")
                        .rel("noopener noreferrer")
                        .test_id("navigation-menu-link-href")
                        .into_element(cx),
                ]
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
        let node = snap
            .nodes
            .iter()
            .find(|n| n.test_id.as_deref() == Some("navigation-menu-link-href"))
            .expect("expected navigation menu link semantics node");
        assert_eq!(node.role, SemanticsRole::Link);
        assert_eq!(node.value.as_deref(), Some("https://example.com/docs"));

        let center = Point::new(
            Px(node.bounds.origin.x.0 + node.bounds.size.width.0 * 0.5),
            Px(node.bounds.origin.y.0 + node.bounds.size.height.0 * 0.5),
        );
        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::Pointer(PointerEvent::Down {
                pointer_id: fret_core::PointerId(0),
                position: center,
                button: MouseButton::Left,
                modifiers: Modifiers::default(),
                click_count: 1,
                pointer_type: PointerType::Mouse,
            }),
        );
        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::Pointer(PointerEvent::Up {
                pointer_id: fret_core::PointerId(0),
                position: center,
                button: MouseButton::Left,
                modifiers: Modifiers::default(),
                click_count: 1,
                pointer_type: PointerType::Mouse,
                is_click: true,
            }),
        );

        let effects = app.flush_effects();
        assert!(
            effects.iter().any(|effect| {
                matches!(
                    effect,
                    fret_runtime::Effect::OpenUrl { url, target, rel }
                        if url == "https://example.com/docs"
                            && target.as_deref() == Some("_blank")
                            && rel.as_deref() == Some("noopener noreferrer")
                )
            }),
            "expected navigation menu link href fallback to emit Effect::OpenUrl with target/rel"
        );
    }

    #[test]
    fn navigation_menu_item_href_without_on_activate_emits_open_url_effect() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let model = app.models_mut().insert(None::<Arc<str>>);
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(400.0), Px(240.0)),
        );
        let mut services = FakeServices::default();

        bump_frame(&mut app);
        let root = fret_ui::declarative::render_root(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "navigation-menu-item-href-open-url",
            |cx| {
                vec![
                    NavigationMenu::new(model.clone())
                        .list(NavigationMenuList::new([NavigationMenuItem::new(
                            "docs",
                            "Documentation",
                            std::iter::empty(),
                        )
                        .href("https://example.com/docs")
                        .target("_blank")
                        .rel("noopener noreferrer")
                        .trigger_test_id("navigation-menu-item-href")]))
                        .into_element(cx),
                ]
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
        let node = snap
            .nodes
            .iter()
            .find(|n| n.test_id.as_deref() == Some("navigation-menu-item-href"))
            .expect("expected navigation menu item semantics node");
        assert_eq!(node.role, SemanticsRole::Link);
        assert_eq!(node.value.as_deref(), Some("https://example.com/docs"));

        let center = Point::new(
            Px(node.bounds.origin.x.0 + node.bounds.size.width.0 * 0.5),
            Px(node.bounds.origin.y.0 + node.bounds.size.height.0 * 0.5),
        );
        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::Pointer(PointerEvent::Down {
                pointer_id: fret_core::PointerId(0),
                position: center,
                button: MouseButton::Left,
                modifiers: Modifiers::default(),
                click_count: 1,
                pointer_type: PointerType::Mouse,
            }),
        );
        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::Pointer(PointerEvent::Up {
                pointer_id: fret_core::PointerId(0),
                position: center,
                button: MouseButton::Left,
                modifiers: Modifiers::default(),
                click_count: 1,
                pointer_type: PointerType::Mouse,
                is_click: true,
            }),
        );

        let effects = app.flush_effects();
        assert!(
            effects.iter().any(|effect| {
                matches!(
                    effect,
                    fret_runtime::Effect::OpenUrl { url, target, rel }
                        if url == "https://example.com/docs"
                            && target.as_deref() == Some("_blank")
                            && rel.as_deref() == Some("noopener noreferrer")
                )
            }),
            "expected contentless navigation menu item href fallback to emit Effect::OpenUrl with target/rel"
        );
    }
}
