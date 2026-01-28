use std::sync::Arc;

use fret_core::Transform2D;
use fret_core::{Color, Corners, Edges, FontId, FontWeight, Point, Px, SemanticsRole, TextStyle};
use fret_icons::ids;
use fret_runtime::{
    CommandId, InputContext, InputDispatchPhase, Model, Platform, PlatformCapabilities,
    WindowCommandGatingService, WindowCommandGatingSnapshot,
};
use fret_ui::element::{
    AnyElement, ContainerProps, Elements, FlexProps, LayoutStyle, Length, MainAlign,
    PointerRegionProps, PressableA11y, PressableProps, RenderTransformProps, SizeStyle, StackProps,
    VisualTransformProps,
};
use fret_ui::overlay_placement::{Align, Side};
use fret_ui::{ElementContext, Theme, UiHost};
use fret_ui_kit::declarative::icon as decl_icon;
use fret_ui_kit::declarative::model_watch::ModelWatchExt as _;
use fret_ui_kit::declarative::style as decl_style;
use fret_ui_kit::primitives::direction as direction_prim;
use fret_ui_kit::primitives::navigation_menu as radix_navigation_menu;
use fret_ui_kit::primitives::{popper, popper_content};
use fret_ui_kit::theme_tokens;
use fret_ui_kit::{
    ChromeRefinement, ColorRef, LayoutRefinement, MetricRef, OverlayController, OverlayPresence,
    OverrideSlot, Radius, Space, WidgetState, WidgetStateProperty, WidgetStates,
    resolve_override_slot, resolve_override_slot_opt, ui,
};

use crate::overlay_motion;

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

fn nav_menu_trigger_text_style(theme: &Theme) -> TextStyle {
    let px = theme
        .metric_by_key("component.navigation_menu.trigger.text_px")
        .or_else(|| theme.metric_by_key(theme_tokens::metric::COMPONENT_TEXT_SM_PX))
        .or_else(|| theme.metric_by_key("metric.font.size"))
        .or_else(|| theme.metric_by_key("font.size"))
        .unwrap_or_else(|| theme.metric_required(theme_tokens::metric::COMPONENT_TEXT_SM_PX));
    let line_height = theme
        .metric_by_key("component.navigation_menu.trigger.line_height")
        .or_else(|| theme.metric_by_key(theme_tokens::metric::COMPONENT_TEXT_SM_LINE_HEIGHT))
        .or_else(|| theme.metric_by_key("metric.font.line_height"))
        .or_else(|| theme.metric_by_key("font.line_height"))
        .unwrap_or_else(|| {
            theme.metric_required(theme_tokens::metric::COMPONENT_TEXT_SM_LINE_HEIGHT)
        });
    TextStyle {
        font: FontId::default(),
        size: px,
        weight: FontWeight::MEDIUM,
        slant: Default::default(),
        line_height: Some(line_height),
        letter_spacing_em: None,
    }
}

fn nav_menu_trigger_padding_x(theme: &Theme) -> Px {
    theme
        .metric_by_key("component.navigation_menu.trigger.pad_x")
        .unwrap_or_else(|| MetricRef::space(Space::N4).resolve(theme))
}

fn nav_menu_trigger_padding_y(theme: &Theme) -> Px {
    theme
        .metric_by_key("component.navigation_menu.trigger.pad_y")
        .unwrap_or_else(|| MetricRef::space(Space::N2).resolve(theme))
}

fn nav_menu_trigger_radius(theme: &Theme) -> Px {
    theme
        .metric_by_key("component.navigation_menu.trigger.radius")
        .unwrap_or_else(|| MetricRef::radius(Radius::Md).resolve(theme))
}

fn nav_menu_trigger_bg_hover(theme: &Theme) -> Color {
    theme.color_required("accent")
}

fn nav_menu_trigger_fg(theme: &Theme) -> Color {
    theme.color_required("foreground")
}

fn nav_menu_trigger_fg_muted(theme: &Theme) -> Color {
    theme.color_required("muted-foreground")
}

fn nav_menu_viewport_bg(theme: &Theme) -> Color {
    theme.color_required("popover")
}

fn nav_menu_viewport_border(theme: &Theme) -> Color {
    theme.color_required("border")
}

fn nav_menu_viewport_side_offset(theme: &Theme) -> Px {
    theme
        .metric_by_key("component.navigation_menu.viewport.side_offset")
        .unwrap_or(Px(6.0))
}

fn nav_menu_viewport_window_margin(theme: &Theme) -> Px {
    theme
        .metric_by_key("component.navigation_menu.viewport.window_margin")
        .unwrap_or(Px(8.0))
}

fn nav_menu_content_switch_slide_px(theme: &Theme) -> Px {
    // Matches shadcn/ui's `slide-*-52` distance (13rem ≈ 208px).
    theme
        .metric_by_key("component.navigation_menu.content.switch_slide_px")
        .unwrap_or(Px(208.0))
}

fn nav_menu_indicator_diamond_size(theme: &Theme) -> Px {
    theme
        .metric_by_key("component.navigation_menu.indicator.diamond_size")
        .unwrap_or(Px(8.0))
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
#[derive(Debug, Clone, Default)]
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
#[derive(Debug, Clone)]
pub struct NavigationMenuLink {
    model: Model<Option<Arc<str>>>,
    children: Vec<AnyElement>,
    label: Option<Arc<str>>,
    test_id: Option<Arc<str>>,
    command: Option<CommandId>,
    disabled: bool,
    dismiss_on_ctrl_or_meta: bool,
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
            disabled: false,
            dismiss_on_ctrl_or_meta: false,
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

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// When `false` (default), activation with Ctrl/Meta pressed does not dismiss the menu.
    pub fn dismiss_on_ctrl_or_meta(mut self, dismiss_on_ctrl_or_meta: bool) -> Self {
        self.dismiss_on_ctrl_or_meta = dismiss_on_ctrl_or_meta;
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
        let label = self.label.clone();
        let test_id = self.test_id.clone();
        let children = std::rc::Rc::new(self.children);
        let dismiss_on_ctrl_or_meta = self.dismiss_on_ctrl_or_meta;

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

        cx.pressable_with_id_props(move |cx, _st, link_id| {
            let modifier_state: Arc<std::sync::Mutex<ModifierState>> = cx.with_state_for(
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

            let mut pressable = PressableProps::default();
            pressable.enabled = !disabled;
            pressable.focusable = !disabled;
            pressable.a11y = PressableA11y {
                role: Some(SemanticsRole::Button),
                label: label.clone(),
                test_id: test_id.clone(),
                ..Default::default()
            };

            (pressable, children.as_ref().clone())
        })
    }
}

/// shadcn/ui `NavigationMenuContent` (v4).
///
/// In the upstream DOM implementation this is an element; in Fret this is a "spec" that provides
/// viewport content for [`NavigationMenuItem`].
#[derive(Debug, Clone, Default)]
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

#[derive(Debug, Clone)]
pub struct NavigationMenuItem {
    value: Arc<str>,
    label: Arc<str>,
    content: Vec<AnyElement>,
    trigger: Option<Vec<AnyElement>>,
    disabled: bool,
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
            disabled: false,
        }
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
#[derive(Debug, Clone, Default)]
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

#[derive(Clone)]
pub struct NavigationMenu {
    model: Option<Model<Option<Arc<str>>>>,
    default_value: Option<Arc<str>>,
    items: Vec<NavigationMenuItem>,
    disabled: bool,
    viewport: bool,
    indicator: bool,
    chrome: ChromeRefinement,
    layout: LayoutRefinement,
    style: NavigationMenuStyle,
    config: radix_navigation_menu::NavigationMenuConfig,
}

impl std::fmt::Debug for NavigationMenu {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("NavigationMenu")
            .field("model", &"<model>")
            .field("items_len", &self.items.len())
            .field("disabled", &self.disabled)
            .field("viewport", &self.viewport)
            .field("chrome", &self.chrome)
            .field("layout", &self.layout)
            .field("config", &self.config)
            .finish()
    }
}

impl NavigationMenu {
    pub fn new(model: Model<Option<Arc<str>>>) -> Self {
        Self {
            model: Some(model),
            default_value: None,
            items: Vec::new(),
            disabled: false,
            viewport: true,
            indicator: false,
            chrome: ChromeRefinement::default(),
            layout: LayoutRefinement::default(),
            style: NavigationMenuStyle::default(),
            config: radix_navigation_menu::NavigationMenuConfig::default(),
        }
    }

    pub fn uncontrolled<T: Into<Arc<str>>>(default_value: Option<T>) -> Self {
        Self {
            model: None,
            default_value: default_value.map(Into::into),
            items: Vec::new(),
            disabled: false,
            viewport: true,
            indicator: false,
            chrome: ChromeRefinement::default(),
            layout: LayoutRefinement::default(),
            style: NavigationMenuStyle::default(),
            config: radix_navigation_menu::NavigationMenuConfig::default(),
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

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let controlled_model = self.model;
        let default_value = self.default_value;
        let items = self.items;
        let menu_disabled = self.disabled;
        let viewport_enabled = self.viewport;
        let indicator_enabled = self.indicator;
        let chrome = self.chrome;
        let layout = self.layout;
        let style = self.style;
        let cfg = self.config;

        let value_model =
            radix_navigation_menu::navigation_menu_use_value_model(cx, controlled_model, || {
                default_value.clone()
            })
            .model();

        let theme = Theme::global(&*cx.app).clone();

        let trigger_pad_x = nav_menu_trigger_padding_x(&theme);
        let trigger_pad_y = nav_menu_trigger_padding_y(&theme);
        let trigger_radius = nav_menu_trigger_radius(&theme);
        let trigger_bg_hover = nav_menu_trigger_bg_hover(&theme);
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
            .when(WidgetStates::OPEN, Some(ColorRef::Color(trigger_bg_hover)));
        let default_trigger_fg = WidgetStateProperty::new(ColorRef::Color(trigger_fg))
            .when(WidgetStates::DISABLED, ColorRef::Color(trigger_fg_muted));

        let viewport_bg = nav_menu_viewport_bg(&theme);
        let viewport_border = nav_menu_viewport_border(&theme);
        let viewport_radius = theme
            .metric_by_key("component.navigation_menu.viewport.radius")
            .unwrap_or_else(|| MetricRef::radius(Radius::Md).resolve(&theme));
        let root_gap = MetricRef::space(Space::N3).resolve(&theme);
        let content_switch_slide_px = nav_menu_content_switch_slide_px(&theme);
        let viewport_shadow = decl_style::shadow(&theme, viewport_radius);
        let content_pad_y = MetricRef::space(Space::N2).resolve(&theme);
        let content_pad_left = MetricRef::space(Space::N2).resolve(&theme);
        let content_pad_right = MetricRef::space(Space::N2p5).resolve(&theme);
        let content_padding = Edges {
            top: content_pad_y,
            right: content_pad_right,
            bottom: content_pad_y,
            left: content_pad_left,
        };

        let root_props = decl_style::container_props(&theme, chrome, layout);

        cx.container(root_props, move |cx| {
            let root_id = cx.root_id();
            let nav_ctx = radix_navigation_menu::NavigationMenuRoot::new(value_model.clone())
                .config(cfg)
                .disabled(menu_disabled)
                .context(cx, root_id);
            let root_state = nav_ctx.root_state.clone();

            #[derive(Default)]
            struct OpenModelState {
                model: Option<Model<bool>>,
            }

            #[derive(Default)]
            struct SelectionSyncState {
                last_selected: Option<Arc<str>>,
            }

            let open_model =
                cx.with_state_for(root_id, OpenModelState::default, |st| st.model.clone());
            let open_model = if let Some(model) = open_model {
                model
            } else {
                let model = cx.app.models_mut().insert(false);
                cx.with_state_for(root_id, OpenModelState::default, |st| {
                    st.model = Some(model.clone());
                });
                model
            };

            let selected: Option<Arc<str>> =
                cx.watch_model(&value_model).layout().cloned().flatten();
            let selected_changed = cx.with_state_for(root_id, SelectionSyncState::default, |st| {
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
            }

            let open: bool = cx
                .watch_model(&open_model)
                .layout()
                .copied()
                .unwrap_or(false);
            let open_for_motion = open && selected.is_some();
            let motion = OverlayController::transition_with_durations_and_easing(
                cx,
                open_for_motion,
                overlay_motion::SHADCN_MOTION_TICKS_100,
                overlay_motion::SHADCN_MOTION_TICKS_100,
                overlay_motion::shadcn_ease,
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

            let list_props = FlexProps {
                layout: LayoutStyle::default(),
                direction: fret_core::Axis::Horizontal,
                gap: Px(4.0), // Tailwind `space-x-1`
                padding: Edges::all(Px(0.0)),
                justify: MainAlign::Start,
                align: fret_ui::element::CrossAlign::Center,
                wrap: false,
                ..Default::default()
            };

            let items_for_children = items.clone();
            let value_for_viewport = value_model.clone();
            let trigger_text_style_for_list = trigger_text_style.clone();
            let nav_ctx_for_list = nav_ctx.clone();
            let theme_for_list = theme.clone();
            let default_trigger_bg_for_list = default_trigger_bg.clone();
            let default_trigger_fg_for_list = default_trigger_fg.clone();
            let style_for_list = style.clone();

            let list = cx.flex(list_props, move |cx| {
                items_for_children
                    .iter()
                    .map(|item| {
                        let item = item.clone();
                        let item_value = item.value.clone();
                        let label = item.label.clone();
                        let disabled = menu_disabled || item.disabled;
                        let trigger_text_style_for_item = trigger_text_style_for_list.clone();
                        let nav_ctx_for_item = nav_ctx_for_list.clone();
                        let theme_for_item = theme_for_list.clone();
                        let default_trigger_bg = default_trigger_bg_for_list.clone();
                        let default_trigger_fg = default_trigger_fg_for_list.clone();
                        let style_override = style_for_list.clone();

                        cx.keyed(item_value.clone(), |cx| {
                            let trigger_text_style = trigger_text_style_for_item.clone();

                            let mut pressable = PressableProps::default();
                            pressable.enabled = !disabled;
                            pressable.focusable = !disabled;
                            pressable.a11y = PressableA11y {
                                role: Some(SemanticsRole::Button),
                                label: Some(label.clone()),
                                ..Default::default()
                            };

                            let pointer_props = PointerRegionProps {
                                layout: LayoutStyle::default(),
                                enabled: true,
                            };

                            let trigger_children = item.trigger.clone();
                            let item_label = item.label.clone();
                            if item.content.is_empty() {
                                // shadcn/ui demo uses a `NavigationMenuLink` for items with no
                                // content (e.g. "Docs"), styled via `navigationMenuTriggerStyle()`.
                                // These should behave like a link (no chevron, no open/close).
                                let trigger_text_style = trigger_text_style.clone();
                                let trigger_children = trigger_children.clone();

                                return cx.pressable(pressable, move |cx, st| {
                                    let hovered = st.hovered && !st.pressed;
                                    let pressed = st.pressed;
                                    let fg = if disabled {
                                        trigger_fg_muted
                                    } else {
                                        trigger_fg
                                    };
                                    let bg = (hovered || pressed).then_some(trigger_bg_hover);

                                    let mut layout = LayoutStyle::default();
                                    layout.size.width = Length::Auto;

                                    let wrapper = ContainerProps {
                                        layout,
                                        padding: Edges {
                                            top: trigger_pad_y,
                                            right: trigger_pad_x,
                                            bottom: trigger_pad_y,
                                            left: trigger_pad_x,
                                        },
                                        background: bg,
                                        shadow: None,
                                        border: Edges::all(Px(0.0)),
                                        border_color: None,
                                        corner_radii: Corners::all(trigger_radius),
                                        ..Default::default()
                                    };

                                    let content_children =
                                        trigger_children.clone().unwrap_or_else(|| {
                                            let style = trigger_text_style.clone();
                                            let mut label = ui::label(cx, item_label.clone())
                                                .text_size_px(style.size)
                                                .font_weight(style.weight)
                                                .text_color(ColorRef::Color(fg))
                                                .nowrap();
                                            if let Some(line_height) = style.line_height {
                                                label = label.line_height_px(line_height);
                                            }
                                            if let Some(letter_spacing_em) = style.letter_spacing_em
                                            {
                                                label = label
                                                    .letter_spacing_em(letter_spacing_em);
                                            }
                                            vec![label.into_element(cx)]
                                        });

                                    let row = cx.flex(
                                        FlexProps {
                                            layout: LayoutStyle::default(),
                                            direction: fret_core::Axis::Horizontal,
                                            gap: Px(0.0),
                                            padding: Edges::all(Px(0.0)),
                                            justify: MainAlign::Center,
                                            align: fret_ui::element::CrossAlign::Center,
                                            wrap: false,
                                            ..Default::default()
                                        },
                                        move |_cx| content_children,
                                    );

                                    vec![cx.container(wrapper, move |_cx| vec![row])]
                                });
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
                                        layout.size.width = Length::Auto;

                                        let wrapper = ContainerProps {
                                            layout,
                                            padding: Edges {
                                                top: trigger_pad_y,
                                                right: trigger_pad_x,
                                                bottom: trigger_pad_y,
                                                left: trigger_pad_x,
                                            },
                                            background: bg,
                                            shadow: None,
                                            border: Edges::all(Px(0.0)),
                                            border_color: None,
                                            corner_radii: Corners::all(trigger_radius),
                                            ..Default::default()
                                        };

                                        let content_children =
                                            trigger_children.clone().unwrap_or_else(|| {
                                                let style = trigger_text_style.clone();
                                                let mut label = ui::label(cx, item_label.clone())
                                                    .text_size_px(style.size)
                                                    .font_weight(style.weight)
                                                    .text_color(fg_ref.clone())
                                                    .nowrap();
                                                if let Some(line_height) = style.line_height {
                                                    label = label.line_height_px(line_height);
                                                }
                                                if let Some(letter_spacing_em) =
                                                    style.letter_spacing_em
                                                {
                                                    label = label.letter_spacing_em(letter_spacing_em);
                                                }
                                                vec![label.into_element(cx)]
                                            });

                                        let fg_ref_for_chevron = fg_ref.clone();
                                        let chevron_rotation = if is_open { 180.0 } else { 0.0 };
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
                                                    layout.inset.top = Some(Px(1.0)); // `top-[1px]`
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
                                        // We model that as a deterministic, non-semantic spacer.
                                        row_children.push(cx.container(
                                            ContainerProps {
                                                layout: {
                                                    let mut layout = LayoutStyle::default();
                                                    layout.size = SizeStyle {
                                                        width: Length::Px(Px(4.0)),
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
                                                layout: LayoutStyle::default(),
                                                direction: fret_core::Axis::Horizontal,
                                                gap: Px(0.0),
                                                padding: Edges::all(Px(0.0)),
                                                justify: MainAlign::Center,
                                                align: fret_ui::element::CrossAlign::Center,
                                                wrap: false,
                                                ..Default::default()
                                            },
                                            move |_cx| row_children,
                                        );

                                        vec![cx.container(wrapper, move |_cx| vec![row])]
                                    },
                                )
                        })
                    })
                    .collect::<Vec<_>>()
            });

            let viewport = active_idx
                .and_then(|idx| items.get(idx))
                .map(|active| active.content.clone())
                .unwrap_or_default();

            let has_content = !viewport.is_empty();
            let is_open = selected_local.is_some() && has_content && open_for_motion;
            let overlay_presence = OverlayPresence {
                present: motion.present && has_content,
                interactive: is_open,
            };

            let content_switch = radix_navigation_menu::navigation_menu_content_switch(transition)
                .map(|sw| {
                    let from_children = items
                        .get(sw.from_idx)
                        .map(|it| it.content.clone())
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
                let viewport_children = viewport.clone();
                let content_switch = content_switch.clone();
                let content_switch_slide_px = content_switch_slide_px;

                let mut panel_props = if viewport_enabled {
                    ContainerProps {
                        layout: LayoutStyle {
                            overflow: fret_ui::element::Overflow::Visible,
                            ..Default::default()
                        },
                        padding: Edges::all(Px(0.0)),
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
                        padding: Edges::all(Px(0.0)),
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
                    panel_props.layout.size.width = Length::Px(content_size.width);
                    panel_props.layout.size.height = Length::Px(content_size.height);
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
                };

                let opacity = opacity;
                let scale = scale;
                let selected_value_for_content_id = selected_local.clone();
                let selected_for_overlay = selected_local.clone();
                radix_navigation_menu::navigation_menu_request_viewport_overlay(
                    cx,
                    root_id,
                    value_model.clone(),
                    open_model.clone(),
                    overlay_presence,
                    selected_for_overlay.as_deref(),
                    args,
                    move |cx, layout| {
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
                        let viewport_children = viewport_children;
                        let content_switch = content_switch;
                        let content_switch_slide_px = content_switch_slide_px;
                        let content_padding = content_padding;
                        let indicator_diamond_shadow = indicator_diamond_shadow;
                        let indicator_diamond_corners = indicator_diamond_corners;
                        let viewport_enabled = viewport_enabled;
                        let selected_value_for_registry: Arc<str> = Arc::from(selected_value_key);

                        let content =
                            radix_navigation_menu::navigation_menu_viewport_content_pressable_with_id_props(
                                cx,
                                selected_value_key,
                                move |cx, _st, content_id| {
                                let root_state_for_hover = root_state_for_hover.clone();
                                let value_for_hover = value_for_hover.clone();
                                cx.pressable_on_hover_change(Arc::new(
                                    move |host, action_cx, hovered| {
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
                                let children = vec![cx.container(panel_props, move |cx| {
                                    let mut clip_layout = LayoutStyle::default();
                                    clip_layout.overflow = fret_ui::element::Overflow::Clip;

                                    let clip_props = ContainerProps {
                                        layout: clip_layout,
                                        corner_radii: Corners::all(viewport_radius),
                                        ..Default::default()
                                    };

                                    vec![cx.container(clip_props, move |cx| {
                                    let Some((t, forward, from_children)) = content_switch.clone()
                                    else {
                                        let children = viewport_children.clone();
                                        let body = cx.keyed("viewport-body", |cx| {
                                            let layout = LayoutStyle::default();
                                            cx.container(
                                                ContainerProps {
                                                    layout,
                                                    padding: content_padding,
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

                                    let to_children = viewport_children.clone();
                                    let t = t.clamp(0.0, 1.0);
                                    let slide = content_switch_slide_px.0;

                                    let (from_dx, to_dx) = if forward {
                                        (-slide * t, slide * (1.0 - t))
                                    } else {
                                        (slide * t, -slide * (1.0 - t))
                                    };

                                    let value_for_registry_for_layers = value_for_registry.clone();

                                    // In shadcn/ui (Radix), `NavigationMenuContent` keeps its
                                    // intrinsic size even during switch animations. In Fret, we
                                    // must preserve that intrinsic sizing so overlay placement can
                                    // converge to the same bounds and so viewport sizing can
                                    // observe the real content size (not the previous panel size).
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

                                            let from_children = vec![cx.keyed("from-body", |cx| {
                                                cx.container(
                                                    ContainerProps {
                                                        layout: LayoutStyle::default(),
                                                        padding: content_padding,
                                                        ..Default::default()
                                                    },
                                                    {
                                                        let from_children = from_children.clone();
                                                        move |_cx| from_children
                                                    },
                                                )
                                            })];
                                            let to_body = cx.keyed("to-body", |cx| {
                                                cx.container(
                                                    ContainerProps {
                                                        layout: LayoutStyle::default(),
                                                        padding: content_padding,
                                                        ..Default::default()
                                                    },
                                                    {
                                                        let to_children = to_children.clone();
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
                                    if viewport_enabled_for_registry {
                                        radix_navigation_menu::navigation_menu_register_viewport_content_id(
                                            cx,
                                            root_id_for_registry,
                                            value_for_registry.clone(),
                                            stack.id,
                                        );
                                    } else {
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
                                        focus_ring_bounds: None,
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
                        // are intrinsically sized in Radix/shadcn. Use an autosizing wrapper so we
                        // don't have to provide a fixed `placed.size` up-front (which would
                        // otherwise lock the content to an estimated/previous size during switch
                        // interactions).
                        let panel = popper_content::popper_wrapper_at_autosize(
                            cx,
                            layout.placed.origin,
                            move |_cx| vec![content],
                        );

                        if viewport_enabled {
                            radix_navigation_menu::navigation_menu_register_viewport_panel_id(
                                cx,
                                root_id,
                                panel.id,
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
                                    diamond_layout.inset.left = Some(Px(diamond_left));
                                    diamond_layout.inset.top = Some(Px(diamond_top));
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
                                                    padding: Edges::all(Px(0.0)),
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

            vec![cx.flex(
                FlexProps {
                    layout: LayoutStyle::default(),
                    direction: fret_core::Axis::Vertical,
                    gap: root_gap,
                    padding: Edges::all(Px(0.0)),
                    justify: MainAlign::Start,
                    align: fret_ui::element::CrossAlign::Stretch,
                    wrap: false,
                    ..Default::default()
                },
                move |_cx| {
                    let mut out = Vec::new();
                    out.push(list);
                    out
                },
            )]
        })
    }
}

pub fn navigation_menu<H: UiHost, I>(
    cx: &mut ElementContext<'_, H>,
    model: Model<Option<Arc<str>>>,
    f: impl FnOnce(&mut ElementContext<'_, H>) -> I,
) -> AnyElement
where
    I: IntoIterator<Item = NavigationMenuItem>,
{
    NavigationMenu::new(model).items(f(cx)).into_element(cx)
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

pub fn navigation_menu_uncontrolled<H: UiHost, T: Into<Arc<str>>, I>(
    cx: &mut ElementContext<'_, H>,
    default_value: Option<T>,
    f: impl FnOnce(&mut ElementContext<'_, H>) -> I,
) -> AnyElement
where
    I: IntoIterator<Item = NavigationMenuItem>,
{
    NavigationMenu::uncontrolled(default_value)
        .items(f(cx))
        .into_element(cx)
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

    fn bump_frame(app: &mut App) {
        app.set_tick_id(TickId(app.tick_id().0.saturating_add(1)));
        app.set_frame_id(FrameId(app.frame_id().0.saturating_add(1)));
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
        let go_btn = snap
            .nodes
            .iter()
            .find(|n| n.role == SemanticsRole::Button && n.label.as_deref() == Some("Go"))
            .expect("Go button semantics");
        let pos = Point::new(
            Px(go_btn.bounds.origin.x.0 + go_btn.bounds.size.width.0 * 0.5),
            Px(go_btn.bounds.origin.y.0 + go_btn.bounds.size.height.0 * 0.5),
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
                            vec![cx.container(
                                ContainerProps {
                                    padding: Edges {
                                        top: Px(100.0),
                                        right: Px(0.0),
                                        bottom: Px(0.0),
                                        left: Px(500.0),
                                    },
                                    ..Default::default()
                                },
                                move |cx| {
                                    let items = vec![
                                        NavigationMenuItem::new(
                                            "alpha",
                                            "Alpha",
                                            vec![cx.text("A")],
                                        ),
                                        NavigationMenuItem::new("beta", "Beta", vec![cx.text("B")]),
                                    ];
                                    vec![
                                        NavigationMenu::new(model_for_render.clone())
                                            .items(items)
                                            .into_element(cx),
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
                        .on_click(cmd.clone())
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
                        .on_click(cmd.clone())
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
                        .on_click(cmd.clone())
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
}
