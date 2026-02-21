use std::cell::Cell;
use std::sync::Arc;
use std::time::Duration;

use fret_core::{Color, Corners, DrawOrder, Edges, FontId, FontWeight, Px, TextStyle};
use fret_runtime::Model;
use fret_ui::element::{
    AnyElement, ContainerProps, CrossAlign, FlexProps, LayoutStyle, Length, MainAlign,
    PressableProps, RovingFlexProps, RovingFocusProps, SpinnerProps, StackProps, SvgIconProps,
};
use fret_ui::elements::GlobalElementId;
use fret_ui::{ElementContext, Theme, UiHost};
use fret_ui_headless::motion::tolerance::Tolerance;
use fret_ui_kit::declarative::action_hooks::ActionHooksExt;
use fret_ui_kit::declarative::chrome::control_chrome_pressable_with_id_props;
use fret_ui_kit::declarative::model_watch::ModelWatchExt as _;
use fret_ui_kit::declarative::motion_springs::shared_indicator_spring_description;
use fret_ui_kit::declarative::motion_value::{
    MotionToSpecF32, MotionValueF32Update, SpringSpecF32, drive_motion_value_f32,
};
use fret_ui_kit::declarative::style as decl_style;
use fret_ui_kit::declarative::transition;
use fret_ui_kit::{
    ChromeRefinement, ColorRef, LayoutRefinement, MetricRef, OverrideSlot, Radius, Space,
    WidgetState, WidgetStateProperty, WidgetStates, resolve_override_slot,
    resolve_override_slot_opt, ui,
};

#[derive(Debug, Default, Clone)]
struct TabsListLayoutRuntime {
    triggers: Vec<GlobalElementId>,
}

#[derive(Debug, Default, Clone)]
struct TabsContentPresenceRuntime {
    active_value: Option<Arc<str>>,
    exiting_values: Vec<Arc<str>>,
}

fn alpha_mul(mut c: Color, mul: f32) -> Color {
    c.a *= mul;
    c
}

fn apply_trigger_inherited_style(
    mut element: AnyElement,
    fg: Color,
    text_style: &TextStyle,
) -> AnyElement {
    match &mut element.kind {
        fret_ui::element::ElementKind::Text(props) => {
            if props.style.is_none() {
                props.style = Some(text_style.clone());
            }
            if props.color.is_none() {
                props.color = Some(fg);
            }
        }
        fret_ui::element::ElementKind::SvgIcon(SvgIconProps { color, .. }) => {
            let is_default = *color
                == Color {
                    r: 1.0,
                    g: 1.0,
                    b: 1.0,
                    a: 1.0,
                };
            if is_default {
                *color = fg;
            }
        }
        fret_ui::element::ElementKind::Spinner(SpinnerProps { color, .. }) => {
            color.get_or_insert(fg);
        }
        _ => {}
    }

    element.children = element
        .children
        .into_iter()
        .map(|child| apply_trigger_inherited_style(child, fg, text_style))
        .collect();
    element
}

fn tabs_gap(theme: &Theme) -> Px {
    theme
        .metric_by_key("component.tabs.gap")
        .unwrap_or_else(|| MetricRef::space(Space::N2).resolve(theme))
}

fn tabs_list_height(theme: &Theme) -> Px {
    theme
        .metric_by_key("component.tabs.list_height")
        .unwrap_or(Px(36.0))
}

fn tabs_list_padding(theme: &Theme) -> Px {
    theme
        .metric_by_key("component.tabs.list_padding")
        .unwrap_or(Px(3.0))
}

fn tabs_list_bg(theme: &Theme) -> Color {
    theme.color_token("muted")
}

fn tabs_list_fg_muted(theme: &Theme) -> Color {
    theme.color_token("muted-foreground")
}

fn tabs_trigger_text_style(theme: &Theme) -> TextStyle {
    let px = theme
        .metric_by_key("component.tabs.trigger.text_px")
        .or_else(|| theme.metric_by_key("font.size"))
        .unwrap_or_else(|| theme.metric_token("font.size"));
    let line_height = theme
        .metric_by_key("component.tabs.trigger.line_height")
        .or_else(|| theme.metric_by_key("font.line_height"))
        .unwrap_or_else(|| theme.metric_token("font.line_height"));
    TextStyle {
        font: FontId::default(),
        size: px,
        weight: FontWeight::MEDIUM,
        line_height: Some(line_height),
        ..Default::default()
    }
}

fn tabs_trigger_radius(theme: &Theme) -> Px {
    theme
        .metric_by_key("component.tabs.trigger.radius")
        .unwrap_or_else(|| MetricRef::radius(Radius::Md).resolve(theme))
}

fn tabs_trigger_bg_active(theme: &Theme) -> Color {
    theme.color_token("background")
}

fn tabs_trigger_border_active(theme: &Theme) -> Color {
    theme
        .color_by_key("input")
        .or_else(|| theme.color_by_key("border"))
        .expect("missing theme token: input/border")
}

fn tabs_trigger_border_width(theme: &Theme) -> Px {
    theme
        .metric_by_key("component.tabs.trigger.border_width")
        .unwrap_or(Px(1.0))
}

use fret_ui_kit::primitives::tabs as radix_tabs;
pub use fret_ui_kit::primitives::tabs::{TabsActivationMode, TabsOrientation};

type OnValueChange = Arc<dyn Fn(Option<Arc<str>>) + Send + Sync + 'static>;
type OnValueChangeWithSource =
    Arc<dyn Fn(Option<Arc<str>>, TabsValueChangeSource) + Send + Sync + 'static>;
type OnValueChangeWithDetails =
    Arc<dyn Fn(Option<Arc<str>>, TabsValueChangeDetails) + Send + Sync + 'static>;
type OnValueChangeWithEventDetails =
    Arc<dyn Fn(Option<Arc<str>>, &mut TabsValueChangeEventDetails) + Send + Sync + 'static>;

/// Base UI-compatible activation direction metadata for tab changes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TabsActivationDirection {
    Left,
    Right,
    Up,
    Down,
    None,
}

impl TabsActivationDirection {
    fn from_indices(
        orientation: TabsOrientation,
        prev: Option<usize>,
        next: Option<usize>,
    ) -> Self {
        let (Some(prev), Some(next)) = (prev, next) else {
            return Self::None;
        };
        if prev == next {
            return Self::None;
        }

        match orientation {
            TabsOrientation::Horizontal => {
                if next < prev {
                    Self::Left
                } else {
                    Self::Right
                }
            }
            TabsOrientation::Vertical => {
                if next < prev {
                    Self::Up
                } else {
                    Self::Down
                }
            }
        }
    }
}

/// Change metadata for Base UI parity.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TabsValueChangeDetails {
    pub source: TabsValueChangeSource,
    pub activation_direction: TabsActivationDirection,
}

/// Base UI-style cancellable event details for tab value changes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TabsValueChangeEventDetails {
    pub source: TabsValueChangeSource,
    pub activation_direction: TabsActivationDirection,
    is_canceled: bool,
}

impl TabsValueChangeEventDetails {
    pub fn prevent_default(&mut self) {
        self.is_canceled = true;
    }

    pub fn is_canceled(&self) -> bool {
        self.is_canceled
    }
}

/// Source metadata for tab value changes.
///
/// This mirrors the high-level intent exposed by Base UI event details without introducing a
/// cancellation contract yet.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TabsValueChangeSource {
    /// Selection changed via roving-focus active-item movement (`activationMode=Automatic`).
    RovingActiveChange,
    /// Selection changed from pointer-down handling on a trigger.
    PointerDown,
    /// Selection changed from trigger activation (keyboard/pointer click activation phase).
    Activate,
}

fn tabs_shared_indicator<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    container_id: GlobalElementId,
    orientation: TabsOrientation,
    tab_count: usize,
    selected_idx: Option<usize>,
    indicator_test_id: Option<Arc<str>>,
    disabled: bool,
    style_override: &TabsStyle,
) -> AnyElement {
    cx.named("tabs_shared_indicator", move |cx| {
        let container_bounds = cx
            .last_bounds_for_element(container_id)
            .unwrap_or(cx.bounds);
        let tab_bounds = selected_idx
            .and_then(|idx| {
                cx.with_state_for(container_id, TabsListLayoutRuntime::default, |rt| {
                    rt.triggers.get(idx).copied()
                })
            })
            .and_then(|tab_id| cx.last_bounds_for_element(tab_id));

        let (
            target_x,
            target_y,
            target_width,
            target_height,
            bg,
            border_color,
            border_w,
            shadow,
            radius,
            spring,
        ) = {
            let theme = Theme::global(&*cx.app);

            let mut states = WidgetStates::empty();
            if disabled {
                states |= WidgetStates::DISABLED;
            }
            if selected_idx.is_some() {
                states |= WidgetStates::SELECTED;
            }

            // shadcn new-york-v4 `TabsTrigger` defaults:
            // - light: `text-foreground`
            // - dark: `text-muted-foreground`
            let fg_inactive = if theme.name.contains("/dark") {
                ColorRef::Color(tabs_list_fg_muted(theme))
            } else {
                ColorRef::Color(theme.color_token("foreground"))
            };
            let fg_active = ColorRef::Color(theme.color_token("foreground"));
            let fg_disabled = ColorRef::Color(alpha_mul(theme.color_token("foreground"), 0.5));

            let bg_active = ColorRef::Color(tabs_trigger_bg_active(theme));
            let border_active = ColorRef::Color(tabs_trigger_border_active(theme));
            let border_w = tabs_trigger_border_width(theme);
            let radius = tabs_trigger_radius(theme);

            let default_trigger_fg = WidgetStateProperty::new(fg_inactive)
                .when(WidgetStates::SELECTED, fg_active)
                .when(WidgetStates::DISABLED, fg_disabled);
            let default_trigger_bg =
                WidgetStateProperty::new(None).when(WidgetStates::SELECTED, Some(bg_active));
            let default_trigger_border =
                WidgetStateProperty::new(None).when(WidgetStates::SELECTED, Some(border_active));

            let _ = default_trigger_fg; // keep token-resolution aligned with trigger defaults

            let bg = resolve_override_slot_opt(
                style_override.trigger_background.as_ref(),
                &default_trigger_bg,
                states,
            )
            .map(|bg| bg.resolve(theme))
            .unwrap_or(Color::TRANSPARENT);
            let border_color = resolve_override_slot_opt(
                style_override.trigger_border_color.as_ref(),
                &default_trigger_border,
                states,
            )
            .map(|border| border.resolve(theme))
            .unwrap_or(Color::TRANSPARENT);

            let (target_x, target_y, target_width, target_height) = if tab_count > 0 {
                if let Some(tab_bounds) = tab_bounds {
                    (
                        tab_bounds.origin.x.0 - container_bounds.origin.x.0,
                        tab_bounds.origin.y.0 - container_bounds.origin.y.0,
                        tab_bounds.size.width.0,
                        tab_bounds.size.height.0,
                    )
                } else if let Some(idx) = selected_idx {
                    match orientation {
                        TabsOrientation::Horizontal => {
                            let tab_w = container_bounds.size.width.0 / (tab_count as f32);
                            (
                                tab_w * (idx as f32),
                                0.0,
                                tab_w,
                                container_bounds.size.height.0,
                            )
                        }
                        TabsOrientation::Vertical => {
                            let tab_h = container_bounds.size.height.0 / (tab_count as f32);
                            (
                                0.0,
                                tab_h * (idx as f32),
                                container_bounds.size.width.0,
                                tab_h,
                            )
                        }
                    }
                } else {
                    (0.0, 0.0, 0.0, 0.0)
                }
            } else {
                (0.0, 0.0, 0.0, 0.0)
            };

            let shadow =
                (!disabled && selected_idx.is_some()).then(|| decl_style::shadow_sm(theme, radius));
            let spring = shared_indicator_spring_description(&*cx.app);

            (
                target_x,
                target_y,
                target_width,
                target_height,
                bg,
                border_color,
                border_w,
                shadow,
                radius,
                spring,
            )
        };

        let spec = MotionToSpecF32::Spring(SpringSpecF32 {
            spring,
            tolerance: Tolerance::default(),
            snap_to_target: true,
        });

        let x = drive_motion_value_f32(
            cx,
            target_x,
            MotionValueF32Update::To {
                target: target_x,
                spec,
                kick: None,
            },
        );
        let y = drive_motion_value_f32(
            cx,
            target_y,
            MotionValueF32Update::To {
                target: target_y,
                spec,
                kick: None,
            },
        );
        let width = drive_motion_value_f32(
            cx,
            target_width,
            MotionValueF32Update::To {
                target: target_width,
                spec,
                kick: None,
            },
        );
        let height = drive_motion_value_f32(
            cx,
            target_height,
            MotionValueF32Update::To {
                target: target_height,
                spec,
                kick: None,
            },
        );

        let mut props = fret_ui::element::CanvasProps::default();
        props.layout.position = fret_ui::element::PositionStyle::Absolute;
        props.layout.inset.top = Some(Px(0.0));
        props.layout.inset.right = Some(Px(0.0));
        props.layout.inset.bottom = Some(Px(0.0));
        props.layout.inset.left = Some(Px(0.0));

        let mut indicator = cx.canvas(props, move |p| {
            if height.value <= 0.0 || width.value <= 0.0 || bg.a <= 0.0 {
                return;
            }

            let bounds = p.bounds();
            // The shared indicator targets trigger bounds tracked relative to the *list container*
            // element (`container_bounds`). Depending on how absolute-positioned children are
            // resolved in the current layout backend, the canvas bounds may be anchored to the
            // container's padding box or content box. Convert from container-local offsets to
            // canvas-local offsets so the indicator stays aligned under list padding.
            let dx = container_bounds.origin.x.0 - bounds.origin.x.0;
            let dy = container_bounds.origin.y.0 - bounds.origin.y.0;

            let x_px = (x.value + dx).clamp(0.0, bounds.size.width.0);
            let y_px = (y.value + dy).clamp(0.0, bounds.size.height.0);
            let max_width = (bounds.size.width.0 - x_px).max(0.0);
            let max_height = (bounds.size.height.0 - y_px).max(0.0);
            let width_px = width.value.clamp(0.0, max_width);
            let height_px = height.value.clamp(0.0, max_height);

            let outer = fret_core::Rect::new(
                fret_core::Point::new(Px(bounds.origin.x.0 + x_px), Px(bounds.origin.y.0 + y_px)),
                fret_core::Size::new(Px(width_px), Px(height_px)),
            );

            if let Some(shadow) = shadow {
                fret_ui::paint::paint_shadow(p.scene(), DrawOrder(0), outer, shadow);
            }

            let corners = Corners::all(radius);
            if border_w.0 > 0.0 && border_color.a > 0.0 {
                fret_ui::paint::paint_state_layer(
                    p.scene(),
                    DrawOrder(1),
                    outer,
                    border_color,
                    1.0,
                    corners,
                );

                let inset = border_w.0.max(0.0);
                let inner = fret_core::Rect::new(
                    fret_core::Point::new(
                        Px(outer.origin.x.0 + inset),
                        Px(outer.origin.y.0 + inset),
                    ),
                    fret_core::Size::new(
                        Px((outer.size.width.0 - inset * 2.0).max(0.0)),
                        Px((outer.size.height.0 - inset * 2.0).max(0.0)),
                    ),
                );
                let inner_radius = Px((radius.0 - inset).max(0.0));
                fret_ui::paint::paint_state_layer(
                    p.scene(),
                    DrawOrder(2),
                    inner,
                    bg,
                    1.0,
                    Corners::all(inner_radius),
                );
            } else {
                fret_ui::paint::paint_state_layer(p.scene(), DrawOrder(1), outer, bg, 1.0, corners);
            }
        });

        if let Some(test_id) = indicator_test_id.as_ref() {
            indicator = indicator.test_id(test_id.clone());
        }

        indicator
    })
}

fn set_tabs_value_and_emit_change(
    host: &mut dyn fret_ui::action::UiActionHost,
    model: &Model<Option<Arc<str>>>,
    values: &[Arc<str>],
    disabled: &[bool],
    next: Option<Arc<str>>,
    on_value_change: Option<&OnValueChange>,
    on_value_change_with_source: Option<&OnValueChangeWithSource>,
    on_value_change_with_details: Option<&OnValueChangeWithDetails>,
    on_value_change_with_event_details: Option<&OnValueChangeWithEventDetails>,
    source: TabsValueChangeSource,
    orientation: TabsOrientation,
) {
    let prev_value = host.models_mut().read(model, |v| v.clone()).ok().flatten();
    if prev_value == next {
        return;
    }
    let prev_idx = fret_ui_kit::primitives::tabs::active_index_from_values(
        values,
        prev_value.as_deref(),
        disabled,
    );
    let next_idx =
        fret_ui_kit::primitives::tabs::active_index_from_values(values, next.as_deref(), disabled);
    let activation_direction =
        TabsActivationDirection::from_indices(orientation, prev_idx, next_idx);

    if let Some(on_value_change_with_event_details) = on_value_change_with_event_details {
        let mut event_details = TabsValueChangeEventDetails {
            source,
            activation_direction,
            is_canceled: false,
        };
        on_value_change_with_event_details(next.clone(), &mut event_details);
        if event_details.is_canceled() {
            return;
        }
    }

    let mut changed = false;
    let next_for_update = next.clone();
    let _ = host.models_mut().update(model, |value| {
        if *value != next_for_update {
            *value = next_for_update.clone();
            changed = true;
        }
    });

    if changed && let Some(on_value_change) = on_value_change {
        on_value_change(next.clone());
    }

    if changed && let Some(on_value_change_with_source) = on_value_change_with_source {
        on_value_change_with_source(next.clone(), source);
    }

    if changed && let Some(on_value_change_with_details) = on_value_change_with_details {
        on_value_change_with_details(
            next,
            TabsValueChangeDetails {
                source,
                activation_direction,
            },
        );
    }
}

#[derive(Debug, Clone, Default)]
pub struct TabsStyle {
    pub trigger_background: OverrideSlot<ColorRef>,
    pub trigger_foreground: OverrideSlot<ColorRef>,
    pub trigger_border_color: OverrideSlot<ColorRef>,
}

impl TabsStyle {
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

    pub fn trigger_border_color(
        mut self,
        trigger_border_color: WidgetStateProperty<Option<ColorRef>>,
    ) -> Self {
        self.trigger_border_color = Some(trigger_border_color);
        self
    }

    pub fn merged(mut self, other: Self) -> Self {
        if other.trigger_background.is_some() {
            self.trigger_background = other.trigger_background;
        }
        if other.trigger_foreground.is_some() {
            self.trigger_foreground = other.trigger_foreground;
        }
        if other.trigger_border_color.is_some() {
            self.trigger_border_color = other.trigger_border_color;
        }
        self
    }
}

/// shadcn/ui `TabsTrigger` (v4).
///
/// This is a "spec" type consumed by [`TabsList`] and [`TabsRoot`]. It mirrors the Radix/shadcn
/// authoring shape while letting Fret drive the underlying semantics and interaction wiring.
#[derive(Debug, Clone)]
pub struct TabsTrigger {
    value: Arc<str>,
    label: Arc<str>,
    children: Option<Vec<AnyElement>>,
    disabled: bool,
}

impl TabsTrigger {
    pub fn new(value: impl Into<Arc<str>>, label: impl Into<Arc<str>>) -> Self {
        Self {
            value: value.into(),
            label: label.into(),
            children: None,
            disabled: false,
        }
    }

    /// Overrides the default trigger contents (the label text) to match shadcn usage patterns where
    /// triggers can include icons/badges.
    pub fn children(mut self, children: impl IntoIterator<Item = AnyElement>) -> Self {
        self.children = Some(children.into_iter().collect());
        self
    }

    pub fn child(mut self, child: AnyElement) -> Self {
        self.children = Some(vec![child]);
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
}

/// shadcn/ui `TabsList` (v4).
#[derive(Debug, Clone, Default)]
pub struct TabsList {
    triggers: Vec<TabsTrigger>,
}

impl TabsList {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn trigger(mut self, trigger: TabsTrigger) -> Self {
        self.triggers.push(trigger);
        self
    }

    pub fn triggers(mut self, triggers: impl IntoIterator<Item = TabsTrigger>) -> Self {
        self.triggers.extend(triggers);
        self
    }
}

/// shadcn/ui `TabsContent` (v4).
///
/// Notes:
/// - Fret currently provides "force mount all panels" via [`TabsRoot::force_mount_content`], which
///   approximates Radix `TabsContent forceMount` semantics.
#[derive(Debug, Clone)]
pub struct TabsContent {
    value: Arc<str>,
    children: Vec<AnyElement>,
}

impl TabsContent {
    pub fn new(value: impl Into<Arc<str>>, children: impl IntoIterator<Item = AnyElement>) -> Self {
        Self {
            value: value.into(),
            children: children.into_iter().collect(),
        }
    }
}

/// A composable, Radix/shadcn-shaped tabs surface (`TabsRoot` / `TabsList` / `TabsTrigger` /
/// `TabsContent`).
///
/// This is the recommended authoring surface when translating upstream shadcn/ui examples.
#[derive(Clone)]
pub struct TabsRoot {
    model: Option<Model<Option<Arc<str>>>>,
    default_value: Option<Arc<str>>,
    list: TabsList,
    contents: Vec<TabsContent>,
    disabled: bool,
    orientation: TabsOrientation,
    activation_mode: TabsActivationMode,
    loop_navigation: bool,
    style: TabsStyle,
    chrome: ChromeRefinement,
    layout: LayoutRefinement,
    force_mount_content: bool,
    list_full_width: bool,
    content_fill_remaining: bool,
    shared_indicator_motion: bool,
    content_presence_motion: bool,
    test_id: Option<Arc<str>>,
    on_value_change: Option<OnValueChange>,
    on_value_change_with_source: Option<OnValueChangeWithSource>,
    on_value_change_with_details: Option<OnValueChangeWithDetails>,
    on_value_change_with_event_details: Option<OnValueChangeWithEventDetails>,
}

impl std::fmt::Debug for TabsRoot {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TabsRoot")
            .field("model", &"<model>")
            .field("list_triggers_len", &self.list.triggers.len())
            .field("contents_len", &self.contents.len())
            .field("disabled", &self.disabled)
            .field("orientation", &self.orientation)
            .field("activation_mode", &self.activation_mode)
            .field("loop_navigation", &self.loop_navigation)
            .field("style", &self.style)
            .field("chrome", &self.chrome)
            .field("layout", &self.layout)
            .field("force_mount_content", &self.force_mount_content)
            .field("shared_indicator_motion", &self.shared_indicator_motion)
            .field("content_presence_motion", &self.content_presence_motion)
            .field("on_value_change", &self.on_value_change.is_some())
            .field(
                "on_value_change_with_source",
                &self.on_value_change_with_source.is_some(),
            )
            .field(
                "on_value_change_with_details",
                &self.on_value_change_with_details.is_some(),
            )
            .field(
                "on_value_change_with_event_details",
                &self.on_value_change_with_event_details.is_some(),
            )
            .finish()
    }
}

impl TabsRoot {
    pub fn new(model: Model<Option<Arc<str>>>) -> Self {
        Self {
            model: Some(model),
            default_value: None,
            list: TabsList::default(),
            contents: Vec::new(),
            disabled: false,
            orientation: TabsOrientation::default(),
            activation_mode: TabsActivationMode::default(),
            loop_navigation: true,
            style: TabsStyle::default(),
            chrome: ChromeRefinement::default(),
            layout: LayoutRefinement::default(),
            force_mount_content: false,
            list_full_width: false,
            content_fill_remaining: false,
            shared_indicator_motion: false,
            content_presence_motion: false,
            test_id: None,
            on_value_change: None,
            on_value_change_with_source: None,
            on_value_change_with_details: None,
            on_value_change_with_event_details: None,
        }
    }

    /// Creates an uncontrolled tabs root with an optional initial value (Radix `defaultValue`).
    pub fn uncontrolled<T: Into<Arc<str>>>(default_value: Option<T>) -> Self {
        Self {
            model: None,
            default_value: default_value.map(Into::into),
            list: TabsList::default(),
            contents: Vec::new(),
            disabled: false,
            orientation: TabsOrientation::default(),
            activation_mode: TabsActivationMode::default(),
            loop_navigation: true,
            style: TabsStyle::default(),
            chrome: ChromeRefinement::default(),
            layout: LayoutRefinement::default(),
            force_mount_content: false,
            list_full_width: false,
            content_fill_remaining: false,
            shared_indicator_motion: false,
            content_presence_motion: false,
            test_id: None,
            on_value_change: None,
            on_value_change_with_source: None,
            on_value_change_with_details: None,
            on_value_change_with_event_details: None,
        }
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Sets the uncontrolled initial selection value (Radix `defaultValue`).
    ///
    /// Note: If a controlled `model` is provided, this value is ignored.
    pub fn default_value<T: Into<Arc<str>>>(mut self, default_value: Option<T>) -> Self {
        self.default_value = default_value.map(Into::into);
        self
    }

    pub fn orientation(mut self, orientation: TabsOrientation) -> Self {
        self.orientation = orientation;
        self
    }

    pub fn activation_mode(mut self, activation_mode: TabsActivationMode) -> Self {
        self.activation_mode = activation_mode;
        self
    }

    /// When `true` (default), arrow key navigation loops at the ends (Radix `loop` behavior).
    pub fn loop_navigation(mut self, loop_navigation: bool) -> Self {
        self.loop_navigation = loop_navigation;
        self
    }

    pub fn style(mut self, style: TabsStyle) -> Self {
        self.style = self.style.merged(style);
        self
    }

    pub fn list(mut self, list: TabsList) -> Self {
        self.list = list;
        self
    }

    pub fn content(mut self, content: TabsContent) -> Self {
        self.contents.push(content);
        self
    }

    pub fn contents(mut self, contents: impl IntoIterator<Item = TabsContent>) -> Self {
        self.contents.extend(contents);
        self
    }

    /// When `true`, all tab panel subtrees remain mounted even when inactive.
    ///
    /// This approximates Radix `TabsContent forceMount` by keeping each panel subtree in the
    /// declarative element tree while gating layout/paint/semantics and interactivity via
    /// `InteractivityGate`.
    pub fn force_mount_content(mut self, force_mount_content: bool) -> Self {
        self.force_mount_content = force_mount_content;
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

    /// When `true`, the tab list stretches to the full available width (new-york-v4 default is
    /// `w-fit`).
    pub fn list_full_width(mut self, full_width: bool) -> Self {
        self.list_full_width = full_width;
        self
    }

    /// When `true`, `TabsContent` tries to fill the remaining main-axis space within the root
    /// flex container (Tailwind-like `flex-1`).
    ///
    /// Notes:
    /// - This should only be used when the parent layout provides a definite main-axis size.
    /// - In auto-sized compositions, forcing `flex: 1` can trigger very deep layout recursion.
    pub fn content_fill_remaining(mut self, fill: bool) -> Self {
        self.content_fill_remaining = fill;
        self
    }

    pub fn shared_indicator_motion(mut self, enabled: bool) -> Self {
        self.shared_indicator_motion = enabled;
        self
    }

    /// When `true`, the active tab panel crossfades when the active tab changes.
    ///
    /// Notes:
    /// - This is a "fluid tabs" style enhancement and is intentionally opt-in.
    /// - For now, this is only applied when `force_mount_content=false`.
    pub fn content_presence_motion(mut self, enabled: bool) -> Self {
        self.content_presence_motion = enabled;
        self
    }

    pub fn test_id(mut self, test_id: impl Into<Arc<str>>) -> Self {
        self.test_id = Some(test_id.into());
        self
    }

    pub fn test_id_opt(mut self, test_id: Option<Arc<str>>) -> Self {
        self.test_id = test_id;
        self
    }

    /// Called when the selected tab value changes (Base UI `onValueChange`).
    pub fn on_value_change(mut self, on_value_change: Option<OnValueChange>) -> Self {
        self.on_value_change = on_value_change;
        self
    }

    /// Called when the selected tab value changes, with source metadata.
    pub fn on_value_change_with_source(
        mut self,
        on_value_change_with_source: Option<OnValueChangeWithSource>,
    ) -> Self {
        self.on_value_change_with_source = on_value_change_with_source;
        self
    }

    /// Called when the selected tab value changes, with Base UI-compatible change metadata.
    pub fn on_value_change_with_details(
        mut self,
        on_value_change_with_details: Option<OnValueChangeWithDetails>,
    ) -> Self {
        self.on_value_change_with_details = on_value_change_with_details;
        self
    }

    /// Called when the selected tab value changes, with cancellable event details.
    pub fn on_value_change_with_event_details(
        mut self,
        on_value_change_with_event_details: Option<OnValueChangeWithEventDetails>,
    ) -> Self {
        self.on_value_change_with_event_details = on_value_change_with_event_details;
        self
    }

    #[track_caller]
    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let list = self.list.clone();
        let contents = self.contents.clone();

        let mut content_by_value: std::collections::HashMap<Arc<str>, Vec<AnyElement>> =
            std::collections::HashMap::new();
        for content in contents {
            content_by_value.insert(content.value.clone(), content.children);
        }

        let items: Vec<TabsItem> = list
            .triggers
            .iter()
            .cloned()
            .map(|trigger| {
                TabsItem::new(
                    trigger.value.clone(),
                    trigger.label.clone(),
                    content_by_value
                        .remove(trigger.value.as_ref())
                        .unwrap_or_default(),
                )
                .trigger_children(trigger.children.clone().unwrap_or_else(|| Vec::new()))
                .disabled(trigger.disabled)
            })
            .collect();

        let tabs = if let Some(model) = self.model.clone() {
            Tabs::new(model)
        } else {
            Tabs::uncontrolled(self.default_value.clone())
        };

        tabs.disabled(self.disabled)
            .orientation(self.orientation)
            .activation_mode(self.activation_mode)
            .loop_navigation(self.loop_navigation)
            .on_value_change(self.on_value_change)
            .on_value_change_with_source(self.on_value_change_with_source)
            .on_value_change_with_details(self.on_value_change_with_details)
            .on_value_change_with_event_details(self.on_value_change_with_event_details)
            .style(self.style)
            .refine_style(self.chrome)
            .refine_layout(self.layout)
            .force_mount_content(self.force_mount_content)
            .list_full_width(self.list_full_width)
            .content_fill_remaining(self.content_fill_remaining)
            .shared_indicator_motion(self.shared_indicator_motion)
            .content_presence_motion(self.content_presence_motion)
            .test_id_opt(self.test_id)
            .items(items)
            .into_element(cx)
    }
}

#[derive(Debug, Clone)]
pub struct TabsItem {
    value: Arc<str>,
    label: Arc<str>,
    content: Vec<AnyElement>,
    trigger: Option<Vec<AnyElement>>,
    trigger_test_id: Option<Arc<str>>,
    disabled: bool,
}

impl TabsItem {
    pub fn new(
        value: impl Into<Arc<str>>,
        label: impl Into<Arc<str>>,
        content: impl IntoIterator<Item = AnyElement>,
    ) -> Self {
        Self {
            value: value.into(),
            label: label.into(),
            content: content.into_iter().collect(),
            trigger: None,
            trigger_test_id: None,
            disabled: false,
        }
    }

    pub fn trigger_children(mut self, children: impl IntoIterator<Item = AnyElement>) -> Self {
        let children: Vec<AnyElement> = children.into_iter().collect();
        if children.is_empty() {
            self.trigger = None;
        } else {
            self.trigger = Some(children);
        }
        self
    }

    pub fn trigger_child(mut self, child: AnyElement) -> Self {
        self.trigger = Some(vec![child]);
        self
    }

    pub fn trigger_test_id(mut self, id: impl Into<Arc<str>>) -> Self {
        self.trigger_test_id = Some(id.into());
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
}

#[derive(Clone)]
pub struct Tabs {
    model: Option<Model<Option<Arc<str>>>>,
    default_value: Option<Arc<str>>,
    items: Vec<TabsItem>,
    disabled: bool,
    orientation: TabsOrientation,
    activation_mode: TabsActivationMode,
    loop_navigation: bool,
    style: TabsStyle,
    chrome: ChromeRefinement,
    layout: LayoutRefinement,
    force_mount_content: bool,
    list_full_width: bool,
    content_fill_remaining: bool,
    shared_indicator_motion: bool,
    content_presence_motion: bool,
    test_id: Option<Arc<str>>,
    on_value_change: Option<OnValueChange>,
    on_value_change_with_source: Option<OnValueChangeWithSource>,
    on_value_change_with_details: Option<OnValueChangeWithDetails>,
    on_value_change_with_event_details: Option<OnValueChangeWithEventDetails>,
}

impl std::fmt::Debug for Tabs {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Tabs")
            .field("model", &"<model>")
            .field("items_len", &self.items.len())
            .field("disabled", &self.disabled)
            .field("orientation", &self.orientation)
            .field("activation_mode", &self.activation_mode)
            .field("loop_navigation", &self.loop_navigation)
            .field("style", &self.style)
            .field("chrome", &self.chrome)
            .field("layout", &self.layout)
            .field("force_mount_content", &self.force_mount_content)
            .field("shared_indicator_motion", &self.shared_indicator_motion)
            .field("content_presence_motion", &self.content_presence_motion)
            .field("on_value_change", &self.on_value_change.is_some())
            .field(
                "on_value_change_with_source",
                &self.on_value_change_with_source.is_some(),
            )
            .field(
                "on_value_change_with_details",
                &self.on_value_change_with_details.is_some(),
            )
            .field(
                "on_value_change_with_event_details",
                &self.on_value_change_with_event_details.is_some(),
            )
            .finish()
    }
}

impl Tabs {
    pub fn new(model: Model<Option<Arc<str>>>) -> Self {
        Self {
            model: Some(model),
            default_value: None,
            items: Vec::new(),
            disabled: false,
            orientation: TabsOrientation::default(),
            activation_mode: TabsActivationMode::default(),
            loop_navigation: true,
            style: TabsStyle::default(),
            chrome: ChromeRefinement::default(),
            layout: LayoutRefinement::default(),
            force_mount_content: false,
            list_full_width: false,
            content_fill_remaining: false,
            shared_indicator_motion: false,
            content_presence_motion: false,
            test_id: None,
            on_value_change: None,
            on_value_change_with_source: None,
            on_value_change_with_details: None,
            on_value_change_with_event_details: None,
        }
    }

    /// Creates an uncontrolled tabs root with an optional initial value (Radix `defaultValue`).
    pub fn uncontrolled<T: Into<Arc<str>>>(default_value: Option<T>) -> Self {
        Self {
            model: None,
            default_value: default_value.map(Into::into),
            items: Vec::new(),
            disabled: false,
            orientation: TabsOrientation::default(),
            activation_mode: TabsActivationMode::default(),
            loop_navigation: true,
            style: TabsStyle::default(),
            chrome: ChromeRefinement::default(),
            layout: LayoutRefinement::default(),
            force_mount_content: false,
            list_full_width: false,
            content_fill_remaining: false,
            shared_indicator_motion: false,
            content_presence_motion: false,
            test_id: None,
            on_value_change: None,
            on_value_change_with_source: None,
            on_value_change_with_details: None,
            on_value_change_with_event_details: None,
        }
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Sets the uncontrolled initial selection value (Radix `defaultValue`).
    ///
    /// Note: If a controlled `model` is provided, this value is ignored.
    pub fn default_value<T: Into<Arc<str>>>(mut self, default_value: Option<T>) -> Self {
        self.default_value = default_value.map(Into::into);
        self
    }

    pub fn orientation(mut self, orientation: TabsOrientation) -> Self {
        self.orientation = orientation;
        self
    }

    pub fn activation_mode(mut self, activation_mode: TabsActivationMode) -> Self {
        self.activation_mode = activation_mode;
        self
    }

    /// When `true` (default), arrow key navigation loops at the ends (Radix `loop` behavior).
    pub fn loop_navigation(mut self, loop_navigation: bool) -> Self {
        self.loop_navigation = loop_navigation;
        self
    }

    pub fn style(mut self, style: TabsStyle) -> Self {
        self.style = self.style.merged(style);
        self
    }

    pub fn item(mut self, item: TabsItem) -> Self {
        self.items.push(item);
        self
    }

    pub fn items(mut self, items: impl IntoIterator<Item = TabsItem>) -> Self {
        self.items.extend(items);
        self
    }

    /// When `true`, all tab panel subtrees remain mounted even when inactive.
    ///
    /// This approximates Radix `TabsContent forceMount` by keeping each panel subtree in the
    /// declarative element tree while gating layout/paint/semantics and interactivity via
    /// `InteractivityGate`.
    pub fn force_mount_content(mut self, force_mount_content: bool) -> Self {
        self.force_mount_content = force_mount_content;
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

    /// When `true`, the tab list stretches to the full available width (new-york-v4 default is
    /// `w-fit`).
    pub fn list_full_width(mut self, full_width: bool) -> Self {
        self.list_full_width = full_width;
        self
    }

    /// When `true`, `TabsContent` tries to fill the remaining main-axis space within the root
    /// flex container (Tailwind-like `flex-1`).
    ///
    /// Notes:
    /// - This should only be used when the parent layout provides a definite main-axis size.
    /// - In auto-sized compositions, forcing `flex: 1` can trigger very deep layout recursion.
    pub fn content_fill_remaining(mut self, fill: bool) -> Self {
        self.content_fill_remaining = fill;
        self
    }

    pub fn shared_indicator_motion(mut self, enabled: bool) -> Self {
        self.shared_indicator_motion = enabled;
        self
    }

    /// When `true`, the active tab panel crossfades when the active tab changes.
    ///
    /// This is intended for "fluid tabs" style motion, and is intentionally opt-in.
    ///
    /// Note: This is only applied when `force_mount_content=false`.
    pub fn content_presence_motion(mut self, enabled: bool) -> Self {
        self.content_presence_motion = enabled;
        self
    }

    pub fn test_id(mut self, test_id: impl Into<Arc<str>>) -> Self {
        self.test_id = Some(test_id.into());
        self
    }

    pub fn test_id_opt(mut self, test_id: Option<Arc<str>>) -> Self {
        self.test_id = test_id;
        self
    }

    /// Called when the selected tab value changes (Base UI `onValueChange`).
    pub fn on_value_change(mut self, on_value_change: Option<OnValueChange>) -> Self {
        self.on_value_change = on_value_change;
        self
    }

    /// Called when the selected tab value changes, with source metadata.
    pub fn on_value_change_with_source(
        mut self,
        on_value_change_with_source: Option<OnValueChangeWithSource>,
    ) -> Self {
        self.on_value_change_with_source = on_value_change_with_source;
        self
    }

    /// Called when the selected tab value changes, with Base UI-compatible change metadata.
    pub fn on_value_change_with_details(
        mut self,
        on_value_change_with_details: Option<OnValueChangeWithDetails>,
    ) -> Self {
        self.on_value_change_with_details = on_value_change_with_details;
        self
    }

    /// Called when the selected tab value changes, with cancellable event details.
    pub fn on_value_change_with_event_details(
        mut self,
        on_value_change_with_event_details: Option<OnValueChangeWithEventDetails>,
    ) -> Self {
        self.on_value_change_with_event_details = on_value_change_with_event_details;
        self
    }

    #[track_caller]
    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let controlled_model = self.model;
        let default_value = self.default_value;
        let items = self.items;
        let tabs_disabled = self.disabled;
        let orientation = self.orientation;
        let activation_mode = self.activation_mode;
        let loop_navigation = self.loop_navigation;
        let style_override = self.style;
        let chrome = self.chrome;
        let layout = self.layout;
        let force_mount_content = self.force_mount_content;
        let list_full_width = self.list_full_width;
        let content_fill_remaining = self.content_fill_remaining;
        let shared_indicator_motion = self.shared_indicator_motion;
        let content_presence_motion = self.content_presence_motion;
        let root_test_id = self.test_id;
        let root_test_id_for_children = root_test_id.clone();
        let on_value_change = self.on_value_change;
        let on_value_change_with_source = self.on_value_change_with_source;
        let on_value_change_with_details = self.on_value_change_with_details;
        let on_value_change_with_event_details = self.on_value_change_with_event_details;

        let model =
            radix_tabs::tabs_use_value_model(cx, controlled_model, || default_value.clone())
                .model();

        let theme = Theme::global(&*cx.app).clone();
        let gap = tabs_gap(&theme);
        let text_style = tabs_trigger_text_style(&theme);

        let selected: Option<Arc<str>> = cx.watch_model(&model).layout().cloned().flatten();

        let values: Vec<Arc<str>> = items.iter().map(|i| i.value.clone()).collect();
        let disabled_flags: Vec<bool> = items.iter().map(|i| tabs_disabled || i.disabled).collect();
        let active_idx = fret_ui_kit::primitives::tabs::active_index_from_values(
            &values,
            selected.as_deref(),
            &disabled_flags,
        );

        let values_arc: Arc<[Arc<str>]> = Arc::from(values.into_boxed_slice());
        let disabled_flags_arc: Arc<[bool]> = Arc::from(disabled_flags.clone().into_boxed_slice());
        let roving = RovingFocusProps {
            enabled: !tabs_disabled,
            wrap: loop_navigation,
            disabled: disabled_flags_arc.clone(),
            ..Default::default()
        };
        let tab_set_size = u32::try_from(items.len())
            .ok()
            .and_then(|size| (size > 0).then_some(size));

        let list_height = tabs_list_height(&theme);
        let list_padding = tabs_list_padding(&theme);
        let mut list_props = decl_style::container_props(
            &theme,
            ChromeRefinement::default()
                .rounded(Radius::Lg)
                .bg(ColorRef::Color(tabs_list_bg(&theme)))
                .text_color(ColorRef::Color(tabs_list_fg_muted(&theme))),
            LayoutRefinement::default().h_px(list_height),
        );
        list_props.padding = Edges::all(list_padding);
        if list_full_width {
            list_props.layout.size.width = Length::Fill;
            list_props.layout.flex.align_self = Some(CrossAlign::Stretch);
        } else {
            // new-york-v4: `TabsList` uses `w-fit` (do not stretch to full width).
            // If the parent container happens to be a flex with `align-items: stretch`, opt out.
            list_props.layout.flex.align_self = Some(CrossAlign::Start);
        }
        let tab_panel_layout = {
            // `TabsContent` should fill the available width by default. Without this, max-content
            // descendants (long lines / unwrapped text) can force the tab panel wider than its
            // parent, causing horizontal overflow in app shells like the UI gallery.
            let mut refinement = LayoutRefinement::default().w_full().min_w_0();
            if content_fill_remaining {
                refinement = refinement.flex_1();
            }
            decl_style::layout_style(&theme, refinement)
        };

        let active_label = active_idx
            .and_then(|active| items.get(active))
            .map(|item| item.label.clone())
            .unwrap_or_else(|| Arc::from(""));
        let active_children = active_idx
            .and_then(|active| items.get(active))
            .and_then(|item| (!force_mount_content).then_some(item.content.clone()))
            .unwrap_or_default();
        let active_value = active_idx
            .and_then(|active| items.get(active))
            .map(|item| item.value.clone());

        let root_props = decl_style::container_props(&theme, chrome, layout);

        let mut root = cx.container(root_props, move |cx| {
            let selected_tab_element: Cell<Option<u64>> = Cell::new(None);
            let selected_tab_element = &selected_tab_element;
            let tab_trigger_elements: Vec<Cell<Option<u64>>> =
                (0..items.len()).map(|_| Cell::new(None)).collect();
            let tab_trigger_elements = &tab_trigger_elements;
            let items_for_list = items.clone();
            let mut children: Vec<AnyElement> = Vec::new();
            let content_stage_test_id = root_test_id_for_children
                .as_ref()
                .map(|id| Arc::<str>::from(format!("{id}-content-stage")));

            let tab_list_semantics = radix_tabs::tab_list_semantics_props(list_props.layout);
            children.push(cx.semantics(tab_list_semantics, move |cx| {
                vec![cx.container(list_props, move |cx| {
                        let list_container_id = cx.root_id();
                        let indicator_test_id = root_test_id_for_children
                            .as_ref()
                            .map(|id| Arc::<str>::from(format!("{id}-shared-indicator")));

                        let mut list_children: Vec<AnyElement> = Vec::new();
                        if shared_indicator_motion {
                            list_children.push(tabs_shared_indicator(
                                cx,
                                list_container_id,
                                orientation,
                                items_for_list.len(),
                                active_idx,
                                indicator_test_id,
                                tabs_disabled,
                                &style_override,
                            ));
                        }

                        list_children.push(cx.roving_flex(
                            RovingFlexProps {
                                flex: FlexProps {
                                    layout: {
                                        let mut layout = LayoutStyle::default();
                                        // shadcn new-york-v4: `TabsList` is an `inline-flex` container with
                                        // `items-center`. Our list is a container overlaying multiple
                                        // children (shared-indicator + trigger row), so we keep the list as
                                        // a non-flex container and instead stretch the trigger row to the
                                        // list content box and center triggers within it.
                                        //
                                        // Without this, the trigger row shrink-wraps to `trigger_h` and is
                                        // top-aligned in the list content box, producing a 1px off-center
                                        // active highlight (bottom gap only).
                                        layout.size.height = Length::Fill;
                                        if list_full_width {
                                            layout.size.width = Length::Fill;
                                        }
                                        layout
                                    },
                                    direction: match orientation {
                                        TabsOrientation::Horizontal => fret_core::Axis::Horizontal,
                                        TabsOrientation::Vertical => fret_core::Axis::Vertical,
                                    },
                                    gap: Px(0.0),
                                    padding: Edges::all(Px(0.0)),
                                    // NOTE: Taffy currently shrink-wraps auto-sized flex containers
                                    // around the sum of flex bases, not the min-content widths.
                                    // With Tailwind-like `flex-1` (`basis=0`), that yields a
                                    // zero-width container and a negative "centered" offset.
                                    // `justify-start` matches shadcn for `flex-1` triggers while
                                    // avoiding negative positions in shrink-wrapped lists.
                                    justify: MainAlign::Start,
                                    align: CrossAlign::Center,
                                    wrap: false,
                                    ..Default::default()
                                },
                                roving,
                            },
                            |cx| {
                                cx.roving_nav_apg();
                                if activation_mode == TabsActivationMode::Automatic {
                                    let model_for_roving = model.clone();
                                    let values_for_roving = values_arc.clone();
                                    let disabled_for_roving = disabled_flags_arc.clone();
                                    let on_value_change_for_roving = on_value_change.clone();
                                    let on_value_change_with_source_for_roving =
                                        on_value_change_with_source.clone();
                                    let on_value_change_with_details_for_roving =
                                        on_value_change_with_details.clone();
                                    let on_value_change_with_event_details_for_roving =
                                        on_value_change_with_event_details.clone();
                                    let orientation_for_roving = orientation;
                                    cx.roving_on_active_change(Arc::new(move |host, _acx, idx| {
                                        let Some(value) = values_for_roving.get(idx).cloned() else {
                                            return;
                                        };
                                        set_tabs_value_and_emit_change(
                                            host,
                                            &model_for_roving,
                                            values_for_roving.as_ref(),
                                            disabled_for_roving.as_ref(),
                                            Some(value),
                                            on_value_change_for_roving.as_ref(),
                                            on_value_change_with_source_for_roving.as_ref(),
                                            on_value_change_with_details_for_roving.as_ref(),
                                            on_value_change_with_event_details_for_roving.as_ref(),
                                            TabsValueChangeSource::RovingActiveChange,
                                            orientation_for_roving,
                                        );
                                    }));
                                }

                                // shadcn new-york-v4 `TabsTrigger` defaults:
                                // - light: `text-foreground`
                                // - dark: `text-muted-foreground`
                                let fg_inactive = if theme.name.contains("/dark") {
                                    ColorRef::Color(tabs_list_fg_muted(&theme))
                                } else {
                                    ColorRef::Color(theme.color_token("foreground"))
                                };
                                let fg_active = ColorRef::Color(theme.color_token("foreground"));
                                let fg_disabled =
                                    ColorRef::Color(alpha_mul(theme.color_token("foreground"), 0.5));
                                let radius = tabs_trigger_radius(&theme);
                                let ring = decl_style::focus_ring(&theme, radius);
                                let bg_active =
                                    ColorRef::Color(tabs_trigger_bg_active(&theme));
                                let border_active =
                                    ColorRef::Color(tabs_trigger_border_active(&theme));
                                let border_w = tabs_trigger_border_width(&theme);

                                let default_trigger_fg = WidgetStateProperty::new(fg_inactive)
                                    .when(WidgetStates::SELECTED, fg_active)
                                    .when(WidgetStates::DISABLED, fg_disabled);
                                let default_trigger_bg = WidgetStateProperty::new(None)
                                    .when(WidgetStates::SELECTED, Some(bg_active));
                                let default_trigger_border = WidgetStateProperty::new(None)
                                    .when(WidgetStates::SELECTED, Some(border_active));

                                let pad_x = MetricRef::space(Space::N2).resolve(&theme);
                                let pad_y = MetricRef::space(Space::N1).resolve(&theme);
                                // new-york-v4: trigger uses `h-[calc(100%-1px)]` relative to the list
                                // content box (after list padding).
                                //
                                // In Fret, centering a `-1px` height delta produces a half-pixel
                                // offset which can snap inconsistently (esp. at non-integer scale
                                // factors) and read as a 1px vertical misalignment. Prefer an even
                                // delta so the centered position lands on whole pixels.
                                let trigger_h = Px(
                                    (list_height.0 - list_padding.0 * 2.0 - 2.0).max(0.0),
                                );
                                let trigger_refinement = if list_full_width {
                                    LayoutRefinement::default().flex_1().h_px(trigger_h)
                                } else {
                                    LayoutRefinement::default().h_px(trigger_h)
                                };
                                let trigger_layout =
                                    decl_style::layout_style(&theme, trigger_refinement);

                                let list_item_count = items_for_list.len();
                                let mut out: Vec<AnyElement> =
                                    Vec::with_capacity(disabled_flags.len());
                                for (idx, item) in items_for_list.iter().cloned().enumerate() {
                                    let on_value_change = on_value_change.clone();
                                    let on_value_change_with_source =
                                        on_value_change_with_source.clone();
                                    let on_value_change_with_details =
                                        on_value_change_with_details.clone();
                                    let on_value_change_with_event_details =
                                        on_value_change_with_event_details.clone();
                                    let values_for_change = values_arc.clone();
                                    let disabled_for_change = disabled_flags_arc.clone();
                                    let orientation_for_change = orientation;
                                    let item_disabled =
                                        disabled_flags.get(idx).copied().unwrap_or(true);
                                    let tab_stop = active_idx.is_some_and(|a| a == idx);
                                    let active = tab_stop;

                                    let style_override = style_override.clone();
                                    let default_trigger_fg = default_trigger_fg.clone();
                                    let default_trigger_bg = default_trigger_bg.clone();
                                    let default_trigger_border = default_trigger_border.clone();
                                    let theme = theme.clone();

                                    let shadow = (!shared_indicator_motion && active && !item_disabled)
                                        .then(|| decl_style::shadow_sm(&theme, radius));

                                    let value = item.value.clone();
                                    let label = item.label.clone();
                                    let trigger_children = item.trigger.clone();
                                    let trigger_test_id = item.trigger_test_id.clone();
                                    let model = model.clone();
                                    let text_style = text_style.clone();

                                    out.push(cx.keyed(value.clone(), move |cx| {
                                        control_chrome_pressable_with_id_props(cx, move |cx, st, id| {
                                        let value_for_pointer = value.clone();
                                        let model_for_pointer = model.clone();
                                        let values_for_pointer = values_for_change.clone();
                                        let disabled_for_pointer = disabled_for_change.clone();
                                        let on_value_change_for_pointer = on_value_change.clone();
                                        let on_value_change_with_source_for_pointer =
                                            on_value_change_with_source.clone();
                                        let on_value_change_with_details_for_pointer =
                                            on_value_change_with_details.clone();
                                        let on_value_change_with_event_details_for_pointer =
                                            on_value_change_with_event_details.clone();

                                        cx.pressable_add_on_pointer_down(Arc::new(
                                            move |host, _cx, down| {
                                                use fret_ui::action::PressablePointerDownResult as R;

                                                match radix_tabs::tabs_trigger_pointer_down_action(
                                                    down.pointer_type,
                                                    down.button,
                                                    down.modifiers,
                                                    item_disabled,
                                                ) {
                                                    radix_tabs::TabsTriggerPointerDownAction::Select => {
                                                        set_tabs_value_and_emit_change(
                                                            host,
                                                            &model_for_pointer,
                                                            values_for_pointer.as_ref(),
                                                            disabled_for_pointer.as_ref(),
                                                            Some(value_for_pointer.clone()),
                                                            on_value_change_for_pointer.as_ref(),
                                                            on_value_change_with_source_for_pointer
                                                                .as_ref(),
                                                            on_value_change_with_details_for_pointer
                                                                .as_ref(),
                                                            on_value_change_with_event_details_for_pointer
                                                                .as_ref(),
                                                            TabsValueChangeSource::PointerDown,
                                                            orientation_for_change,
                                                        );
                                                        R::Continue
                                                    }
                                                    radix_tabs::TabsTriggerPointerDownAction::PreventFocus => {
                                                        host.prevent_default(
                                                            fret_runtime::DefaultAction::FocusOnPointerDown,
                                                        );
                                                        R::SkipDefault
                                                    }
                                                    radix_tabs::TabsTriggerPointerDownAction::Ignore => R::Continue,
                                                }
                                            },
                                        ));
                                        let value_for_activate = value.clone();
                                        let model_for_activate = model.clone();
                                        let values_for_activate = values_for_change.clone();
                                        let disabled_for_activate = disabled_for_change.clone();
                                        let on_value_change_for_activate = on_value_change.clone();
                                        let on_value_change_with_source_for_activate =
                                            on_value_change_with_source.clone();
                                        let on_value_change_with_details_for_activate =
                                            on_value_change_with_details.clone();
                                        let on_value_change_with_event_details_for_activate =
                                            on_value_change_with_event_details.clone();
                                        cx.pressable_add_on_activate(Arc::new(
                                            move |host, _acx, _reason| {
                                                set_tabs_value_and_emit_change(
                                                    host,
                                                    &model_for_activate,
                                                    values_for_activate.as_ref(),
                                                    disabled_for_activate.as_ref(),
                                                    Some(value_for_activate.clone()),
                                                    on_value_change_for_activate.as_ref(),
                                                    on_value_change_with_source_for_activate
                                                        .as_ref(),
                                                    on_value_change_with_details_for_activate
                                                        .as_ref(),
                                                    on_value_change_with_event_details_for_activate
                                                        .as_ref(),
                                                    TabsValueChangeSource::Activate,
                                                    orientation_for_change,
                                                );
                                            },
                                        ));
                                        if active {
                                            selected_tab_element.set(Some(id.0));
                                        }
                                        if force_mount_content
                                            && let Some(cell) = tab_trigger_elements.get(idx)
                                        {
                                            cell.set(Some(id.0));
                                        }

                                        cx.with_state_for(
                                            list_container_id,
                                            TabsListLayoutRuntime::default,
                                            |rt| {
                                                if rt.triggers.len() != list_item_count {
                                                    rt.triggers.resize(list_item_count, id);
                                                }
                                                if let Some(slot) = rt.triggers.get_mut(idx) {
                                                    *slot = id;
                                                }
                                            },
                                        );

                                        let mut states =
                                            WidgetStates::from_pressable(cx, st, !item_disabled);
                                        states.set(WidgetState::Selected, active);

                                        let fg_ref = resolve_override_slot(
                                            style_override.trigger_foreground.as_ref(),
                                            &default_trigger_fg,
                                            states,
                                        );
                                        let fg = fg_ref.resolve(&theme);
                                        let bg = if shared_indicator_motion {
                                            None
                                        } else {
                                            resolve_override_slot_opt(
                                                style_override.trigger_background.as_ref(),
                                                &default_trigger_bg,
                                                states,
                                            )
                                            .map(|bg| bg.resolve(&theme))
                                        };
                                        let border = resolve_override_slot_opt(
                                            style_override.trigger_border_color.as_ref(),
                                            &default_trigger_border,
                                            states,
                                        )
                                        .map(|border| border.resolve(&theme))
                                        .unwrap_or(Color::TRANSPARENT);
                                        let border = if shared_indicator_motion {
                                            Color::TRANSPARENT
                                        } else {
                                            border
                                        };

                                        let mut a11y =
                                            fret_ui_kit::primitives::tabs::tab_a11y_with_collection(
                                                Some(label.clone()),
                                                active,
                                                u32::try_from(idx + 1).ok(),
                                                tab_set_size,
                                            );
                                        if let Some(test_id) = trigger_test_id.as_ref() {
                                            a11y.test_id = Some(test_id.clone());
                                        }

                                        let props = PressableProps {
                                            layout: trigger_layout,
                                            enabled: !item_disabled,
                                            focusable: tab_stop || st.focused,
                                            focus_ring: Some(ring),
                                            a11y,
                                            ..Default::default()
                                        };

                                        let chrome = ContainerProps {
                                            padding: Edges {
                                                top: pad_y,
                                                right: pad_x,
                                                bottom: pad_y,
                                                left: pad_x,
                                            },
                                            background: bg,
                                            shadow,
                                            border: Edges::all(border_w),
                                            border_color: Some(border),
                                            corner_radii: Corners::all(radius),
                                            ..Default::default()
                                        };

                                        let content = move |cx: &mut ElementContext<'_, H>| {
                                            let base = trigger_children.clone().unwrap_or_else(|| {
                                                let style = text_style.clone();
                                                let mut text = ui::label(cx, label.clone())
                                                    .text_size_px(style.size)
                                                    .font_weight(style.weight)
                                                    .text_color(fg_ref.clone())
                                                    .nowrap();
                                                if let Some(line_height) = style.line_height {
                                                    // Match web/GPUI baseline behavior in fixed-height controls by
                                                    // treating the allocated bounds height as the effective line box.
                                                    //
                                                    // This opts into a "half-leading" baseline placement model for
                                                    // the first line, but does not force the element height to equal
                                                    // the configured line height.
                                                    text = text
                                                        .line_height_px(line_height)
                                                        .line_height_policy(
                                                            fret_core::TextLineHeightPolicy::FixedFromStyle,
                                                        )
                                                        .h_full()
                                                        .line_box_in_bounds();
                                                }
                                                if let Some(letter_spacing_em) =
                                                    style.letter_spacing_em
                                                {
                                                    text = text.letter_spacing_em(letter_spacing_em);
                                                }
                                                vec![text.into_element(cx)]
                                            });

                                            let styled: Vec<AnyElement> = base
                                                .into_iter()
                                                .map(|child| {
                                                    apply_trigger_inherited_style(child, fg, &text_style)
                                                })
                                                .collect();

                                            vec![cx.flex(
                                                FlexProps {
                                                    layout: {
                                                        let mut layout = LayoutStyle::default();
                                                     layout.size.width = Length::Fill;
                                                     layout.size.height = Length::Fill;
                                                     layout
                                                 },
                                                    direction: fret_core::Axis::Horizontal,
                                                    gap: Px(6.0),
                                                    padding: Edges::all(Px(0.0)),
                                                    justify: MainAlign::Center,
                                                    align: CrossAlign::Center,
                                                    wrap: false,
                                                },
                                                move |_cx| styled,
                                            )]
                                        };

                                        (props, chrome, content)
                                        })
                                     }));
                                 }
                                 out
                             },
                        ));
                        list_children
                    })]
            }));

            if !force_mount_content {
                let active_label_opt = (!active_label.is_empty()).then_some(active_label.clone());

                if content_presence_motion {
                    let theme = Theme::global(&*cx.app);
                    let enter_duration =
                        Duration::from_millis(theme.duration_ms_token("duration.motion.presence.enter") as u64);
                    let exit_duration =
                        Duration::from_millis(theme.duration_ms_token("duration.motion.presence.exit") as u64);
                    let easing = theme.easing_token("easing.motion.standard");

                    let (switched, exiting_values) =
                        cx.with_state(TabsContentPresenceRuntime::default, |st| {
                            let switched = st.active_value.is_some() && st.active_value != active_value;
                            if switched {
                                if let Some(prev) = st.active_value.take() {
                                    if !st.exiting_values.iter().any(|v| v == &prev) {
                                        st.exiting_values.push(prev);
                                    }
                                }
                            }

                            st.active_value = active_value.clone();
                            if let Some(v) = active_value.as_ref() {
                                st.exiting_values.retain(|x| x != v);
                            }

                            // Keep this bounded to avoid retaining unbounded content trees if callers
                            // spam tab switching mid-animation.
                            const MAX_EXITING: usize = 2;
                            if st.exiting_values.len() > MAX_EXITING {
                                st.exiting_values
                                    .drain(0..(st.exiting_values.len() - MAX_EXITING));
                            }

                            (switched, st.exiting_values.clone())
                        });

                    let mut prune: Vec<Arc<str>> = Vec::new();
                    let mut stacked_panels: Vec<AnyElement> = Vec::new();

                    for value in exiting_values.iter().cloned() {
                        let Some(item) = items.iter().find(|it| it.value == value) else {
                            prune.push(value);
                            continue;
                        };

                        let content = item.content.clone();
                        let (present, panel) = cx.keyed(("tabs_panel", value.clone()), |cx| {
                            let out = transition::drive_transition_with_durations_and_cubic_bezier_duration_with_mount_behavior(
                                cx,
                                false,
                                enter_duration,
                                exit_duration,
                                easing,
                                false,
                            );

                            let panel = cx.opacity(out.progress, |cx| {
                                vec![cx.interactivity_gate(out.present, false, move |cx| {
                                    vec![cx.container(
                                        ContainerProps {
                                            layout: tab_panel_layout,
                                            ..Default::default()
                                        },
                                        move |_cx| content,
                                    )]
                                })]
                            });

                            (out.present, panel)
                        });
                        if present {
                            stacked_panels.push(panel);
                        } else {
                            prune.push(value);
                        }
                    }

                    if let Some(active_value) = active_value.as_ref() {
                        if let Some(item) = items.iter().find(|it| it.value == *active_value) {
                            let labelled_by_element = selected_tab_element.get();
                            let content = item.content.clone();
                            let active_value = active_value.clone();

                            let panel = cx.keyed(("tabs_panel", active_value), |cx| {
                                let out = transition::drive_transition_with_durations_and_cubic_bezier_duration_with_mount_behavior(
                                    cx,
                                    true,
                                    enter_duration,
                                    exit_duration,
                                    easing,
                                    switched,
                                );
                                cx.opacity(out.progress, |cx| {
                                    vec![cx.semantics(
                                        radix_tabs::tab_panel_semantics_props(
                                            tab_panel_layout,
                                            active_label_opt.clone(),
                                            labelled_by_element,
                                        ),
                                        move |_cx| content,
                                    )]
                                })
                            });
                            stacked_panels.push(panel);
                        }
                    }

                    if !prune.is_empty() {
                        cx.with_state(TabsContentPresenceRuntime::default, |st| {
                            st.exiting_values.retain(|v| !prune.iter().any(|p| p == v));
                        });
                    }

                    if !stacked_panels.is_empty() {
                        let mut stage = cx.stack_props(
                            StackProps {
                                layout: tab_panel_layout,
                            },
                            move |_cx| stacked_panels,
                        );
                        if let Some(test_id) = content_stage_test_id.as_ref() {
                            stage = stage.test_id(test_id.clone());
                        }
                        children.push(stage);
                    }
                } else if let Some(panel) = radix_tabs::tab_panel_with_gate(
                    cx,
                    true,
                    false,
                    tab_panel_layout,
                    active_label_opt,
                    selected_tab_element.get(),
                    move |_cx| active_children,
                ) {
                    children.push(panel);
                }
            }

            if force_mount_content {
                for (idx, item) in items.iter().cloned().enumerate() {
                    let active = active_idx.is_some_and(|a| a == idx);
                    let labelled_by_element = tab_trigger_elements
                        .get(idx)
                        .and_then(|cell| cell.get());
                    let label = item.label.clone();
                    let content = item.content.clone();

                    let panel = radix_tabs::tab_panel_with_gate(
                        cx,
                        active,
                        true,
                        tab_panel_layout,
                        (!label.is_empty()).then_some(label),
                        labelled_by_element,
                        move |_cx| content,
                    )
                    .expect("force-mounted tabs content should always render a subtree");
                    children.push(panel);
                }
            }

            vec![cx.flex(
                FlexProps {
                    direction: match orientation {
                        TabsOrientation::Horizontal => fret_core::Axis::Vertical,
                        TabsOrientation::Vertical => fret_core::Axis::Horizontal,
                    },
                    gap,
                    padding: Edges::all(Px(0.0)),
                    justify: MainAlign::Start,
                    align: CrossAlign::Stretch,
                    wrap: false,
                    ..Default::default()
                },
                move |_cx| children,
            )]
        });

        if let Some(test_id) = root_test_id.as_ref() {
            root = root.test_id(test_id.clone());
        }

        root
    }
}

pub fn tabs<H: UiHost, I>(
    cx: &mut ElementContext<'_, H>,
    model: Model<Option<Arc<str>>>,
    f: impl FnOnce(&mut ElementContext<'_, H>) -> I,
) -> AnyElement
where
    I: IntoIterator<Item = TabsItem>,
{
    Tabs::new(model).items(f(cx)).into_element(cx)
}

pub fn tabs_uncontrolled<H: UiHost, T: Into<Arc<str>>, I>(
    cx: &mut ElementContext<'_, H>,
    default_value: Option<T>,
    f: impl FnOnce(&mut ElementContext<'_, H>) -> I,
) -> AnyElement
where
    I: IntoIterator<Item = TabsItem>,
{
    Tabs::uncontrolled(default_value)
        .items(f(cx))
        .into_element(cx)
}

#[cfg(test)]
mod tests {
    use super::*;
    use fret_app::App;
    use fret_core::{
        AppWindowId, Modifiers, MouseButton, Point, PointerType, Px, Rect, SemanticsRole, Size,
        SvgId, SvgService,
    };
    use fret_core::{PathCommand, PathConstraints, PathId, PathMetrics, PathService, PathStyle};
    use fret_core::{TextBlobId, TextConstraints, TextMetrics, TextService};
    use fret_runtime::{FrameId, TickId};
    use fret_ui::element::ColumnProps;
    use fret_ui::elements::{ElementRuntime, GlobalElementId, node_for_element};
    use fret_ui::tree::UiTree;

    #[test]
    fn tabs_selected_trigger_is_vertically_centered_in_tab_list() {
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

        let root = fret_ui::declarative::render_root(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "tabs-vert-center",
            |cx| {
                let items = vec![
                    TabsItem::new("alpha", "Alpha", vec![]),
                    TabsItem::new("beta", "Beta", vec![]),
                    TabsItem::new("gamma", "Gamma", vec![]),
                ];
                vec![Tabs::new(model.clone()).items(items).into_element(cx)]
            },
        );

        ui.set_root(root);
        ui.request_semantics_snapshot();
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        let snap = ui.semantics_snapshot().expect("semantics snapshot");
        let tab_list = snap
            .nodes
            .iter()
            .find(|n| n.role == SemanticsRole::TabList)
            .expect("tablist node");
        let selected_tab = snap
            .nodes
            .iter()
            .find(|n| n.role == SemanticsRole::Tab && n.flags.selected)
            .expect("selected tab");

        let tab_list_top = tab_list.bounds.origin.y.0;
        let tab_list_bottom = tab_list.bounds.origin.y.0 + tab_list.bounds.size.height.0;
        let tab_top = selected_tab.bounds.origin.y.0;
        let tab_bottom = selected_tab.bounds.origin.y.0 + selected_tab.bounds.size.height.0;

        let top_margin = tab_top - tab_list_top;
        let bottom_margin = tab_list_bottom - tab_bottom;

        let diff = (top_margin - bottom_margin).abs();
        assert!(
            diff <= 0.51,
            "selected tab should be vertically centered in tablist: top_margin={top_margin:.3}, bottom_margin={bottom_margin:.3}, diff={diff:.3}"
        );
    }

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

    fn render(
        ui: &mut UiTree<App>,
        app: &mut App,
        services: &mut dyn fret_core::UiServices,
        window: AppWindowId,
        bounds: Rect,
        model: Model<Option<Arc<str>>>,
        activation_mode: TabsActivationMode,
    ) -> fret_core::NodeId {
        let root =
            fret_ui::declarative::render_root(ui, app, services, window, bounds, "tabs", |cx| {
                let items = vec![
                    TabsItem::new("alpha", "Alpha", vec![]),
                    TabsItem::new("beta", "Beta", vec![]),
                    TabsItem::new("gamma", "Gamma", vec![]),
                ];
                vec![
                    Tabs::new(model)
                        .activation_mode(activation_mode)
                        .items(items)
                        .into_element(cx),
                ]
            });
        ui.set_root(root);
        ui.request_semantics_snapshot();
        ui.layout_all(app, services, bounds, 1.0);
        root
    }

    fn render_uncontrolled(
        ui: &mut UiTree<App>,
        app: &mut App,
        services: &mut dyn fret_core::UiServices,
        window: AppWindowId,
        bounds: Rect,
        default_value: Option<Arc<str>>,
    ) -> fret_core::NodeId {
        let root = fret_ui::declarative::render_root(
            ui,
            app,
            services,
            window,
            bounds,
            "tabs-uncontrolled",
            |cx| {
                let items = vec![
                    TabsItem::new("alpha", "Alpha", vec![]),
                    TabsItem::new("beta", "Beta", vec![]),
                    TabsItem::new("gamma", "Gamma", vec![]),
                ];
                vec![
                    Tabs::uncontrolled(default_value.clone())
                        .activation_mode(TabsActivationMode::Manual)
                        .items(items)
                        .into_element(cx),
                ]
            },
        );
        ui.set_root(root);
        ui.request_semantics_snapshot();
        ui.layout_all(app, services, bounds, 1.0);
        root
    }

    fn render_composable(
        ui: &mut UiTree<App>,
        app: &mut App,
        services: &mut dyn fret_core::UiServices,
        window: AppWindowId,
        bounds: Rect,
        model: Model<Option<Arc<str>>>,
        activation_mode: TabsActivationMode,
    ) -> fret_core::NodeId {
        let root = fret_ui::declarative::render_root(
            ui,
            app,
            services,
            window,
            bounds,
            "tabs-composable",
            |cx| {
                let list = TabsList::new()
                    .trigger(TabsTrigger::new("alpha", "Alpha"))
                    .trigger(TabsTrigger::new("beta", "Beta"))
                    .trigger(TabsTrigger::new("gamma", "Gamma"));
                let contents = vec![
                    TabsContent::new("alpha", vec![]),
                    TabsContent::new("beta", vec![]),
                    TabsContent::new("gamma", vec![]),
                ];
                vec![
                    TabsRoot::new(model)
                        .activation_mode(activation_mode)
                        .list(list)
                        .contents(contents)
                        .into_element(cx),
                ]
            },
        );
        ui.set_root(root);
        ui.request_semantics_snapshot();
        ui.layout_all(app, services, bounds, 1.0);
        root
    }

    fn bump_frame(app: &mut App) {
        app.set_tick_id(TickId(app.tick_id().0.saturating_add(1)));
        app.set_frame_id(FrameId(app.frame_id().0.saturating_add(1)));
    }

    #[test]
    fn tabs_layout_regression_does_not_stack_overflow_in_auto_sized_column() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let model = app.models_mut().insert(Some(Arc::from("alpha")));
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            fret_core::Size::new(Px(560.0), Px(520.0)),
        );
        let mut services = FakeServices::default();

        let root = fret_ui::declarative::render_root(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "tabs-layout-regression",
            |cx| {
                let mut page = ContainerProps::default();
                page.layout.size.width = Length::Fill;
                page.layout.size.height = Length::Fill;
                page.padding = Edges::all(Px(16.0));

                vec![cx.container(page, |cx| {
                    let items = vec![
                        TabsItem::new("alpha", "Alpha", vec![cx.text("Panel")]),
                        TabsItem::new("beta", "Beta", vec![cx.text("Panel")]),
                        TabsItem::new("gamma", "Gamma", vec![cx.text("Panel")]),
                    ];

                    let mut col = ColumnProps::default();
                    col.layout.size.width = Length::Fill;
                    col.layout.size.height = Length::Auto;
                    col.gap = Px(16.0);

                    vec![cx.column(col, |cx| {
                        vec![
                            cx.text("Header"),
                            Tabs::new(model.clone())
                                .refine_layout(LayoutRefinement::default().w_full())
                                .items(items)
                                .into_element(cx),
                        ]
                    })]
                })]
            },
        );

        ui.set_root(root);
        ui.request_semantics_snapshot();
        ui.layout_all(&mut app, &mut services, bounds, 1.0);
    }

    #[test]
    fn tabs_trigger_mouse_down_selects_immediately_like_radix() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let model = app.models_mut().insert(Some(Arc::from("alpha")));

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            fret_core::Size::new(Px(400.0), Px(240.0)),
        );
        let mut services = FakeServices::default();

        let _root = render(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            model.clone(),
            TabsActivationMode::Manual,
        );

        let snap = ui.semantics_snapshot().expect("semantics snapshot");
        let beta_tab = snap
            .nodes
            .iter()
            .find(|n| n.role == SemanticsRole::Tab && n.label.as_deref() == Some("Beta"))
            .expect("beta tab");

        let click = Point::new(
            Px(beta_tab.bounds.origin.x.0 + beta_tab.bounds.size.width.0 / 2.0),
            Px(beta_tab.bounds.origin.y.0 + beta_tab.bounds.size.height.0 / 2.0),
        );

        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::Pointer(fret_core::PointerEvent::Down {
                pointer_id: fret_core::PointerId(0),
                position: click,
                button: MouseButton::Left,
                modifiers: Modifiers::default(),
                pointer_type: PointerType::Mouse,
                click_count: 1,
            }),
        );

        let selected = app.models().get_cloned(&model).flatten();
        assert_eq!(selected.as_deref(), Some("beta"));
    }

    #[test]
    fn tabs_root_composable_selects_on_left_click() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let model = app.models_mut().insert(Some(Arc::from("alpha")));

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            fret_core::Size::new(Px(400.0), Px(240.0)),
        );
        let mut services = FakeServices::default();

        let _root = render_composable(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            model.clone(),
            TabsActivationMode::Manual,
        );

        let snap = ui.semantics_snapshot().expect("semantics snapshot");
        let beta_tab = snap
            .nodes
            .iter()
            .find(|n| n.role == SemanticsRole::Tab && n.label.as_deref() == Some("Beta"))
            .expect("beta tab");

        let click = Point::new(
            Px(beta_tab.bounds.origin.x.0 + beta_tab.bounds.size.width.0 / 2.0),
            Px(beta_tab.bounds.origin.y.0 + beta_tab.bounds.size.height.0 / 2.0),
        );

        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::Pointer(fret_core::PointerEvent::Down {
                pointer_id: fret_core::PointerId(0),
                position: click,
                button: MouseButton::Left,
                modifiers: Modifiers::default(),
                pointer_type: PointerType::Mouse,
                click_count: 1,
            }),
        );

        let selected = app.models().get_cloned(&model).flatten();
        assert_eq!(selected.as_deref(), Some("beta"));
    }

    #[test]
    fn tabs_trigger_ctrl_click_does_not_select_or_focus() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let model = app.models_mut().insert(Some(Arc::from("alpha")));

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            fret_core::Size::new(Px(400.0), Px(240.0)),
        );
        let mut services = FakeServices::default();

        let _root = render(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            model.clone(),
            TabsActivationMode::Manual,
        );

        let snap = ui.semantics_snapshot().expect("semantics snapshot");
        let alpha_tab = snap
            .nodes
            .iter()
            .find(|n| n.role == SemanticsRole::Tab && n.label.as_deref() == Some("Alpha"))
            .expect("alpha tab");

        let click = Point::new(
            Px(alpha_tab.bounds.origin.x.0 + alpha_tab.bounds.size.width.0 / 2.0),
            Px(alpha_tab.bounds.origin.y.0 + alpha_tab.bounds.size.height.0 / 2.0),
        );

        let mut modifiers = Modifiers::default();
        modifiers.ctrl = true;

        assert_eq!(ui.focus(), None);
        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::Pointer(fret_core::PointerEvent::Down {
                pointer_id: fret_core::PointerId(0),
                position: click,
                button: MouseButton::Left,
                modifiers,
                pointer_type: PointerType::Mouse,
                click_count: 1,
            }),
        );

        let selected = app.models().get_cloned(&model).flatten();
        assert_eq!(selected.as_deref(), Some("alpha"));
        assert_eq!(ui.focus(), None);
    }

    #[test]
    fn tabs_uncontrolled_applies_default_value_once_and_allows_selection_changes() {
        fn tab_is_selected(ui: &UiTree<App>, label: &str) -> bool {
            ui.semantics_snapshot()
                .expect("semantics snapshot")
                .nodes
                .iter()
                .find(|n| n.role == SemanticsRole::Tab && n.label.as_deref() == Some(label))
                .map(|n| n.flags.selected)
                .unwrap_or(false)
        }

        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            fret_core::Size::new(Px(400.0), Px(240.0)),
        );
        let mut services = FakeServices::default();

        let _root = render_uncontrolled(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            Some(Arc::from("alpha")),
        );
        assert!(tab_is_selected(&ui, "Alpha"));

        let snap = ui.semantics_snapshot().expect("semantics snapshot");
        let beta_tab = snap
            .nodes
            .iter()
            .find(|n| n.role == SemanticsRole::Tab && n.label.as_deref() == Some("Beta"))
            .expect("beta tab");

        let click = Point::new(
            Px(beta_tab.bounds.origin.x.0 + beta_tab.bounds.size.width.0 / 2.0),
            Px(beta_tab.bounds.origin.y.0 + beta_tab.bounds.size.height.0 / 2.0),
        );

        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::Pointer(fret_core::PointerEvent::Down {
                pointer_id: fret_core::PointerId(0),
                position: click,
                button: MouseButton::Left,
                modifiers: Modifiers::default(),
                pointer_type: PointerType::Mouse,
                click_count: 1,
            }),
        );

        bump_frame(&mut app);
        let _root = render_uncontrolled(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            Some(Arc::from("alpha")),
        );
        assert!(tab_is_selected(&ui, "Beta"));

        // The internal model should not be reset by repeatedly passing the same default value.
        bump_frame(&mut app);
        let _root = render_uncontrolled(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            Some(Arc::from("alpha")),
        );
        assert!(tab_is_selected(&ui, "Beta"));
    }

    #[test]
    fn tabs_on_value_change_builder_sets_handler() {
        let mut app = App::new();
        let model = app.models_mut().insert(None::<Arc<str>>);

        let tabs = Tabs::new(model).on_value_change(Some(Arc::new(|_value| {})));
        assert!(tabs.on_value_change.is_some());
    }

    #[test]
    fn tabs_root_on_value_change_builder_sets_handler() {
        let mut app = App::new();
        let model = app.models_mut().insert(None::<Arc<str>>);

        let tabs = TabsRoot::new(model).on_value_change(Some(Arc::new(|_value| {})));
        assert!(tabs.on_value_change.is_some());
    }

    #[test]
    fn tabs_on_value_change_with_source_builder_sets_handler() {
        let mut app = App::new();
        let model = app.models_mut().insert(None::<Arc<str>>);

        let tabs =
            Tabs::new(model).on_value_change_with_source(Some(Arc::new(|_value, _source| {})));
        assert!(tabs.on_value_change_with_source.is_some());
    }

    #[test]
    fn tabs_root_on_value_change_with_source_builder_sets_handler() {
        let mut app = App::new();
        let model = app.models_mut().insert(None::<Arc<str>>);

        let tabs =
            TabsRoot::new(model).on_value_change_with_source(Some(Arc::new(|_value, _source| {})));
        assert!(tabs.on_value_change_with_source.is_some());
    }

    #[test]
    fn tabs_on_value_change_fires_once_when_selection_changes() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let model = app.models_mut().insert(Some(Arc::from("alpha")));
        let changed_values: Arc<std::sync::Mutex<Vec<Option<Arc<str>>>>> =
            Arc::new(std::sync::Mutex::new(Vec::new()));
        let changed_values_for_handler = changed_values.clone();

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            fret_core::Size::new(Px(400.0), Px(240.0)),
        );
        let mut services = FakeServices::default();

        let root = fret_ui::declarative::render_root(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "tabs-on-value-change",
            |cx| {
                let items = vec![
                    TabsItem::new("alpha", "Alpha", vec![]),
                    TabsItem::new("beta", "Beta", vec![]),
                    TabsItem::new("gamma", "Gamma", vec![]),
                ];
                vec![
                    Tabs::new(model.clone())
                        .activation_mode(TabsActivationMode::Manual)
                        .on_value_change(Some(Arc::new(move |value| {
                            changed_values_for_handler
                                .lock()
                                .unwrap_or_else(|e| e.into_inner())
                                .push(value);
                        })))
                        .items(items)
                        .into_element(cx),
                ]
            },
        );

        ui.set_root(root);
        ui.request_semantics_snapshot();
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        let snap = ui.semantics_snapshot().expect("semantics snapshot");
        let beta_tab = snap
            .nodes
            .iter()
            .find(|n| n.role == SemanticsRole::Tab && n.label.as_deref() == Some("Beta"))
            .expect("beta tab");

        let click = Point::new(
            Px(beta_tab.bounds.origin.x.0 + beta_tab.bounds.size.width.0 / 2.0),
            Px(beta_tab.bounds.origin.y.0 + beta_tab.bounds.size.height.0 / 2.0),
        );

        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::Pointer(fret_core::PointerEvent::Down {
                pointer_id: fret_core::PointerId(0),
                position: click,
                button: MouseButton::Left,
                modifiers: Modifiers::default(),
                pointer_type: PointerType::Mouse,
                click_count: 1,
            }),
        );

        let selected = app.models().get_cloned(&model).flatten();
        assert_eq!(selected.as_deref(), Some("beta"));

        let values = changed_values.lock().unwrap_or_else(|e| e.into_inner());
        assert_eq!(values.len(), 1);
        assert_eq!(values[0].as_deref(), Some("beta"));
    }

    #[test]
    fn tabs_on_value_change_with_source_reports_pointer_down() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let model = app.models_mut().insert(Some(Arc::from("alpha")));
        let changed: Arc<std::sync::Mutex<Vec<(Option<Arc<str>>, TabsValueChangeSource)>>> =
            Arc::new(std::sync::Mutex::new(Vec::new()));
        let changed_for_handler = changed.clone();

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            fret_core::Size::new(Px(400.0), Px(240.0)),
        );
        let mut services = FakeServices::default();

        let root = fret_ui::declarative::render_root(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "tabs-on-value-change-with-source-pointer",
            |cx| {
                let items = vec![
                    TabsItem::new("alpha", "Alpha", vec![]),
                    TabsItem::new("beta", "Beta", vec![]),
                    TabsItem::new("gamma", "Gamma", vec![]),
                ];
                vec![
                    Tabs::new(model.clone())
                        .activation_mode(TabsActivationMode::Manual)
                        .on_value_change_with_source(Some(Arc::new(move |value, source| {
                            changed_for_handler
                                .lock()
                                .unwrap_or_else(|e| e.into_inner())
                                .push((value, source));
                        })))
                        .items(items)
                        .into_element(cx),
                ]
            },
        );

        ui.set_root(root);
        ui.request_semantics_snapshot();
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        let snap = ui.semantics_snapshot().expect("semantics snapshot");
        let beta_tab = snap
            .nodes
            .iter()
            .find(|n| n.role == SemanticsRole::Tab && n.label.as_deref() == Some("Beta"))
            .expect("beta tab");

        let click = Point::new(
            Px(beta_tab.bounds.origin.x.0 + beta_tab.bounds.size.width.0 / 2.0),
            Px(beta_tab.bounds.origin.y.0 + beta_tab.bounds.size.height.0 / 2.0),
        );

        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::Pointer(fret_core::PointerEvent::Down {
                pointer_id: fret_core::PointerId(0),
                position: click,
                button: MouseButton::Left,
                modifiers: Modifiers::default(),
                pointer_type: PointerType::Mouse,
                click_count: 1,
            }),
        );

        let selected = app.models().get_cloned(&model).flatten();
        assert_eq!(selected.as_deref(), Some("beta"));

        let changed = changed.lock().unwrap_or_else(|e| e.into_inner());
        assert_eq!(changed.len(), 1);
        assert_eq!(changed[0].0.as_deref(), Some("beta"));
        assert_eq!(changed[0].1, TabsValueChangeSource::PointerDown);
    }

    #[test]
    fn tabs_on_value_change_with_source_reports_roving_active_change() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let model = app.models_mut().insert(Some(Arc::from("alpha")));
        let changed: Arc<std::sync::Mutex<Vec<(Option<Arc<str>>, TabsValueChangeSource)>>> =
            Arc::new(std::sync::Mutex::new(Vec::new()));
        let changed_for_handler = changed.clone();

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            fret_core::Size::new(Px(400.0), Px(240.0)),
        );
        let mut services = FakeServices::default();

        let root = fret_ui::declarative::render_root(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "tabs-on-value-change-with-source-roving",
            |cx| {
                let items = vec![
                    TabsItem::new("alpha", "Alpha", vec![]),
                    TabsItem::new("beta", "Beta", vec![]),
                    TabsItem::new("gamma", "Gamma", vec![]),
                ];
                vec![
                    Tabs::new(model.clone())
                        .activation_mode(TabsActivationMode::Automatic)
                        .on_value_change_with_source(Some(Arc::new(move |value, source| {
                            changed_for_handler
                                .lock()
                                .unwrap_or_else(|e| e.into_inner())
                                .push((value, source));
                        })))
                        .items(items)
                        .into_element(cx),
                ]
            },
        );

        ui.set_root(root);
        ui.request_semantics_snapshot();
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        let focusable = ui
            .first_focusable_descendant_including_declarative(&mut app, window, root)
            .expect("focusable tab");
        ui.set_focus(Some(focusable));

        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::KeyDown {
                key: fret_core::KeyCode::ArrowRight,
                modifiers: Modifiers::default(),
                repeat: false,
            },
        );

        let selected = app.models().get_cloned(&model).flatten();
        assert_eq!(selected.as_deref(), Some("beta"));

        let changed = changed.lock().unwrap_or_else(|e| e.into_inner());
        assert_eq!(changed.len(), 1);
        assert_eq!(changed[0].0.as_deref(), Some("beta"));
        assert_eq!(changed[0].1, TabsValueChangeSource::RovingActiveChange);
    }

    #[test]
    fn tabs_on_value_change_with_details_reports_activation_direction_on_pointer_down() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let model = app.models_mut().insert(Some(Arc::from("alpha")));
        let changed: Arc<std::sync::Mutex<Vec<(Option<Arc<str>>, TabsValueChangeDetails)>>> =
            Arc::new(std::sync::Mutex::new(Vec::new()));
        let changed_for_handler = changed.clone();

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            fret_core::Size::new(Px(400.0), Px(240.0)),
        );
        let mut services = FakeServices::default();

        let root = fret_ui::declarative::render_root(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "tabs-on-value-change-with-details-pointer",
            |cx| {
                let items = vec![
                    TabsItem::new("alpha", "Alpha", vec![]),
                    TabsItem::new("beta", "Beta", vec![]),
                    TabsItem::new("gamma", "Gamma", vec![]),
                ];
                vec![
                    Tabs::new(model.clone())
                        .activation_mode(TabsActivationMode::Manual)
                        .on_value_change_with_details(Some(Arc::new(move |value, details| {
                            changed_for_handler
                                .lock()
                                .unwrap_or_else(|e| e.into_inner())
                                .push((value, details));
                        })))
                        .items(items)
                        .into_element(cx),
                ]
            },
        );

        ui.set_root(root);
        ui.request_semantics_snapshot();
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        let snap = ui.semantics_snapshot().expect("semantics snapshot");
        let beta_tab = snap
            .nodes
            .iter()
            .find(|n| n.role == SemanticsRole::Tab && n.label.as_deref() == Some("Beta"))
            .expect("beta tab");

        let click = Point::new(
            Px(beta_tab.bounds.origin.x.0 + beta_tab.bounds.size.width.0 / 2.0),
            Px(beta_tab.bounds.origin.y.0 + beta_tab.bounds.size.height.0 / 2.0),
        );

        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::Pointer(fret_core::PointerEvent::Down {
                pointer_id: fret_core::PointerId(0),
                position: click,
                button: MouseButton::Left,
                modifiers: Modifiers::default(),
                pointer_type: PointerType::Mouse,
                click_count: 1,
            }),
        );

        let selected = app.models().get_cloned(&model).flatten();
        assert_eq!(selected.as_deref(), Some("beta"));

        let changed = changed.lock().unwrap_or_else(|e| e.into_inner());
        assert_eq!(changed.len(), 1);
        assert_eq!(changed[0].0.as_deref(), Some("beta"));
        assert_eq!(changed[0].1.source, TabsValueChangeSource::PointerDown);
        assert_eq!(
            changed[0].1.activation_direction,
            TabsActivationDirection::Right
        );
    }

    #[test]
    fn tabs_on_value_change_with_details_reports_activation_direction_on_roving_active_change() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let model = app.models_mut().insert(Some(Arc::from("alpha")));
        let changed: Arc<std::sync::Mutex<Vec<(Option<Arc<str>>, TabsValueChangeDetails)>>> =
            Arc::new(std::sync::Mutex::new(Vec::new()));
        let changed_for_handler = changed.clone();

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            fret_core::Size::new(Px(400.0), Px(240.0)),
        );
        let mut services = FakeServices::default();

        let root = fret_ui::declarative::render_root(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "tabs-on-value-change-with-details-roving",
            |cx| {
                let items = vec![
                    TabsItem::new("alpha", "Alpha", vec![]),
                    TabsItem::new("beta", "Beta", vec![]),
                    TabsItem::new("gamma", "Gamma", vec![]),
                ];
                vec![
                    Tabs::new(model.clone())
                        .activation_mode(TabsActivationMode::Automatic)
                        .on_value_change_with_details(Some(Arc::new(move |value, details| {
                            changed_for_handler
                                .lock()
                                .unwrap_or_else(|e| e.into_inner())
                                .push((value, details));
                        })))
                        .items(items)
                        .into_element(cx),
                ]
            },
        );

        ui.set_root(root);
        ui.request_semantics_snapshot();
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        let focusable = ui
            .first_focusable_descendant_including_declarative(&mut app, window, root)
            .expect("focusable tab");
        ui.set_focus(Some(focusable));

        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::KeyDown {
                key: fret_core::KeyCode::ArrowRight,
                modifiers: Modifiers::default(),
                repeat: false,
            },
        );

        let selected = app.models().get_cloned(&model).flatten();
        assert_eq!(selected.as_deref(), Some("beta"));

        let changed = changed.lock().unwrap_or_else(|e| e.into_inner());
        assert_eq!(changed.len(), 1);
        assert_eq!(changed[0].0.as_deref(), Some("beta"));
        assert_eq!(
            changed[0].1.source,
            TabsValueChangeSource::RovingActiveChange
        );
        assert_eq!(
            changed[0].1.activation_direction,
            TabsActivationDirection::Right
        );
    }

    #[test]
    fn tabs_on_value_change_with_event_details_can_cancel_model_update() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let model = app.models_mut().insert(Some(Arc::from("alpha")));
        let called: Arc<std::sync::Mutex<Vec<TabsValueChangeEventDetails>>> =
            Arc::new(std::sync::Mutex::new(Vec::new()));
        let called_for_handler = called.clone();
        let changed: Arc<std::sync::Mutex<Vec<(Option<Arc<str>>, TabsValueChangeSource)>>> =
            Arc::new(std::sync::Mutex::new(Vec::new()));
        let changed_for_source = changed.clone();

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            fret_core::Size::new(Px(400.0), Px(240.0)),
        );
        let mut services = FakeServices::default();

        let root = fret_ui::declarative::render_root(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "tabs-on-value-change-with-event-details-cancel",
            |cx| {
                let items = vec![
                    TabsItem::new("alpha", "Alpha", vec![]),
                    TabsItem::new("beta", "Beta", vec![]),
                    TabsItem::new("gamma", "Gamma", vec![]),
                ];
                vec![
                    Tabs::new(model.clone())
                        .activation_mode(TabsActivationMode::Manual)
                        .on_value_change_with_source(Some(Arc::new(move |value, source| {
                            changed_for_source
                                .lock()
                                .unwrap_or_else(|e| e.into_inner())
                                .push((value, source));
                        })))
                        .on_value_change_with_event_details(Some(Arc::new(
                            move |_value, details| {
                                details.prevent_default();
                                called_for_handler
                                    .lock()
                                    .unwrap_or_else(|e| e.into_inner())
                                    .push(*details);
                            },
                        )))
                        .items(items)
                        .into_element(cx),
                ]
            },
        );

        ui.set_root(root);
        ui.request_semantics_snapshot();
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        let snap = ui.semantics_snapshot().expect("semantics snapshot");
        let beta_tab = snap
            .nodes
            .iter()
            .find(|n| n.role == SemanticsRole::Tab && n.label.as_deref() == Some("Beta"))
            .expect("beta tab");

        let click = Point::new(
            Px(beta_tab.bounds.origin.x.0 + beta_tab.bounds.size.width.0 / 2.0),
            Px(beta_tab.bounds.origin.y.0 + beta_tab.bounds.size.height.0 / 2.0),
        );

        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::Pointer(fret_core::PointerEvent::Down {
                pointer_id: fret_core::PointerId(0),
                position: click,
                button: MouseButton::Left,
                modifiers: Modifiers::default(),
                pointer_type: PointerType::Mouse,
                click_count: 1,
            }),
        );

        let selected = app.models().get_cloned(&model).flatten();
        assert_eq!(
            selected.as_deref(),
            Some("alpha"),
            "expected cancelation to keep the previous selection"
        );

        let called = called.lock().unwrap_or_else(|e| e.into_inner());
        assert_eq!(called.len(), 1);
        assert_eq!(called[0].source, TabsValueChangeSource::PointerDown);
        assert!(called[0].is_canceled);

        let changed = changed.lock().unwrap_or_else(|e| e.into_inner());
        assert!(
            changed.is_empty(),
            "expected source callbacks to be suppressed when change is canceled"
        );
    }

    fn render_force_mount_frame(
        ui: &mut UiTree<App>,
        app: &mut App,
        services: &mut dyn fret_core::UiServices,
        window: AppWindowId,
        bounds: Rect,
        model: Model<Option<Arc<str>>>,
        force_mount: bool,
        alpha_content_id_out: &Cell<Option<GlobalElementId>>,
    ) {
        let root = fret_ui::declarative::render_root(
            ui,
            app,
            services,
            window,
            bounds,
            "tabs-force-mount",
            |cx| {
                let alpha_content = cx.pressable_with_id(
                    PressableProps {
                        layout: LayoutStyle::default(),
                        enabled: true,
                        focusable: true,
                        ..Default::default()
                    },
                    |_cx, _st, id| {
                        alpha_content_id_out.set(Some(id));
                        Vec::new()
                    },
                );

                let items = vec![
                    TabsItem::new("alpha", "Alpha", vec![alpha_content]),
                    TabsItem::new("beta", "Beta", vec![]),
                ];

                vec![
                    Tabs::new(model)
                        .force_mount_content(force_mount)
                        .items(items)
                        .into_element(cx),
                ]
            },
        );

        ui.set_root(root);
        ui.layout_all(app, services, bounds, 1.0);
    }

    #[test]
    fn tabs_manual_activation_does_not_change_model_on_arrow_navigation() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let model = app.models_mut().insert(Some(Arc::from("alpha")));

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            fret_core::Size::new(Px(400.0), Px(240.0)),
        );
        let mut services = FakeServices::default();

        let root = render(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            model.clone(),
            TabsActivationMode::Manual,
        );

        let focusable = ui
            .first_focusable_descendant_including_declarative(&mut app, window, root)
            .expect("focusable tab");
        ui.set_focus(Some(focusable));

        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::KeyDown {
                key: fret_core::KeyCode::ArrowRight,
                modifiers: Modifiers::default(),
                repeat: false,
            },
        );

        let _ = render(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            model.clone(),
            TabsActivationMode::Manual,
        );

        let selected = app.models().get_cloned(&model).flatten();
        assert_eq!(selected.as_deref(), Some("alpha"));

        let snap = ui.semantics_snapshot().expect("semantics snapshot");
        let focus = snap.focus.expect("focus");
        let focused_node = snap
            .nodes
            .iter()
            .find(|n| n.id == focus)
            .expect("focused node");
        assert_eq!(focused_node.role, SemanticsRole::Tab);
        assert_eq!(focused_node.label.as_deref(), Some("Beta"));

        assert!(
            snap.nodes.iter().any(|n| n.role == SemanticsRole::TabList),
            "tab list role"
        );

        let selected_tab = snap
            .nodes
            .iter()
            .find(|n| n.role == SemanticsRole::Tab && n.flags.selected)
            .expect("selected tab");
        assert_eq!(selected_tab.set_size, Some(3));
        assert_eq!(selected_tab.pos_in_set, Some(1));

        let mut other_tabs: Vec<_> = snap
            .nodes
            .iter()
            .filter(|n| n.role == SemanticsRole::Tab && !n.flags.selected)
            .collect();
        other_tabs.sort_by_key(|n| n.pos_in_set.unwrap_or(u32::MAX));
        assert_eq!(other_tabs.len(), 2);
        assert_eq!(other_tabs[0].set_size, Some(3));
        assert_eq!(other_tabs[1].set_size, Some(3));
        assert_eq!(other_tabs[0].pos_in_set, Some(2));
        assert_eq!(other_tabs[1].pos_in_set, Some(3));

        let panel = snap
            .nodes
            .iter()
            .find(|n| n.role == SemanticsRole::TabPanel)
            .expect("tab panel");
        assert_eq!(panel.label.as_deref(), Some("Alpha"));
        assert!(
            panel.labelled_by.iter().any(|id| *id == selected_tab.id),
            "tabpanel should be labelled by selected tab"
        );
        assert!(
            selected_tab.controls.iter().any(|id| *id == panel.id),
            "selected tab should control the active tabpanel"
        );
    }

    #[test]
    fn tabs_without_force_mount_allows_inactive_panel_nodes_to_be_swept() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let model = app.models_mut().insert(Some(Arc::from("alpha")));
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            fret_core::Size::new(Px(400.0), Px(240.0)),
        );
        let mut services = FakeServices::default();

        let alpha_content_id: Cell<Option<GlobalElementId>> = Cell::new(None);

        bump_frame(&mut app);
        render_force_mount_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            model.clone(),
            false,
            &alpha_content_id,
        );

        let alpha_content_id = alpha_content_id.get().expect("alpha content id");
        assert!(node_for_element(&mut app, window, alpha_content_id).is_some());

        let lag = app.with_global_mut(ElementRuntime::new, |rt, _app| rt.gc_lag_frames());

        let _ = app
            .models_mut()
            .update(&model, |v| *v = Some(Arc::from("beta")));

        for _ in 0..=lag {
            bump_frame(&mut app);
            render_force_mount_frame(
                &mut ui,
                &mut app,
                &mut services,
                window,
                bounds,
                model.clone(),
                false,
                &Cell::new(Some(alpha_content_id)),
            );
        }

        assert!(node_for_element(&mut app, window, alpha_content_id).is_none());
    }

    #[test]
    fn tabs_force_mount_keeps_inactive_panel_nodes_alive() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let model = app.models_mut().insert(Some(Arc::from("alpha")));
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            fret_core::Size::new(Px(400.0), Px(240.0)),
        );
        let mut services = FakeServices::default();

        let alpha_content_id: Cell<Option<GlobalElementId>> = Cell::new(None);

        bump_frame(&mut app);
        render_force_mount_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            model.clone(),
            true,
            &alpha_content_id,
        );

        let alpha_content_id = alpha_content_id.get().expect("alpha content id");
        assert!(node_for_element(&mut app, window, alpha_content_id).is_some());

        let lag = app.with_global_mut(ElementRuntime::new, |rt, _app| rt.gc_lag_frames());

        let _ = app
            .models_mut()
            .update(&model, |v| *v = Some(Arc::from("beta")));

        for _ in 0..=(lag + 2) {
            bump_frame(&mut app);
            render_force_mount_frame(
                &mut ui,
                &mut app,
                &mut services,
                window,
                bounds,
                model.clone(),
                true,
                &Cell::new(Some(alpha_content_id)),
            );
        }

        assert!(node_for_element(&mut app, window, alpha_content_id).is_some());
    }

    #[test]
    fn tabs_on_value_change_with_event_details_can_cancel_pointer_down() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let model = app.models_mut().insert(Some(Arc::from("alpha")));
        let changed: Arc<std::sync::Mutex<Vec<Option<Arc<str>>>>> =
            Arc::new(std::sync::Mutex::new(Vec::new()));
        let changed_for_handler = changed.clone();

        let events: Arc<std::sync::Mutex<Vec<(Option<Arc<str>>, TabsValueChangeEventDetails)>>> =
            Arc::new(std::sync::Mutex::new(Vec::new()));
        let events_for_handler = events.clone();

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            fret_core::Size::new(Px(400.0), Px(240.0)),
        );
        let mut services = FakeServices::default();

        let root = fret_ui::declarative::render_root(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "tabs-on-value-change-with-event-details-pointer-cancel",
            |cx| {
                let items = vec![
                    TabsItem::new("alpha", "Alpha", vec![]),
                    TabsItem::new("beta", "Beta", vec![]),
                    TabsItem::new("gamma", "Gamma", vec![]),
                ];
                vec![
                    Tabs::new(model.clone())
                        .activation_mode(TabsActivationMode::Manual)
                        .on_value_change(Some(Arc::new(move |value| {
                            changed_for_handler
                                .lock()
                                .unwrap_or_else(|e| e.into_inner())
                                .push(value);
                        })))
                        .on_value_change_with_event_details(Some(Arc::new(
                            move |value, event_details| {
                                if value.as_deref() == Some("beta") {
                                    event_details.prevent_default();
                                }
                                events_for_handler
                                    .lock()
                                    .unwrap_or_else(|e| e.into_inner())
                                    .push((value, *event_details));
                            },
                        )))
                        .items(items)
                        .into_element(cx),
                ]
            },
        );

        ui.set_root(root);
        ui.request_semantics_snapshot();
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        let snap = ui.semantics_snapshot().expect("semantics snapshot");
        let beta_tab = snap
            .nodes
            .iter()
            .find(|n| n.role == SemanticsRole::Tab && n.label.as_deref() == Some("Beta"))
            .expect("beta tab");

        let click = Point::new(
            Px(beta_tab.bounds.origin.x.0 + beta_tab.bounds.size.width.0 / 2.0),
            Px(beta_tab.bounds.origin.y.0 + beta_tab.bounds.size.height.0 / 2.0),
        );

        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::Pointer(fret_core::PointerEvent::Down {
                pointer_id: fret_core::PointerId(0),
                position: click,
                button: MouseButton::Left,
                modifiers: Modifiers::default(),
                pointer_type: PointerType::Mouse,
                click_count: 1,
            }),
        );

        let selected = app.models().get_cloned(&model).flatten();
        assert_eq!(selected.as_deref(), Some("alpha"));

        let changed = changed.lock().unwrap_or_else(|e| e.into_inner());
        assert!(changed.is_empty());

        let events = events.lock().unwrap_or_else(|e| e.into_inner());
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].0.as_deref(), Some("beta"));
        assert_eq!(events[0].1.source, TabsValueChangeSource::PointerDown);
        assert_eq!(
            events[0].1.activation_direction,
            TabsActivationDirection::Right
        );
        assert!(events[0].1.is_canceled());
    }

    #[test]
    fn tabs_on_value_change_with_event_details_can_cancel_roving_active_change() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let model = app.models_mut().insert(Some(Arc::from("alpha")));
        let events: Arc<std::sync::Mutex<Vec<(Option<Arc<str>>, TabsValueChangeEventDetails)>>> =
            Arc::new(std::sync::Mutex::new(Vec::new()));
        let events_for_handler = events.clone();

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            fret_core::Size::new(Px(400.0), Px(240.0)),
        );
        let mut services = FakeServices::default();

        let root = fret_ui::declarative::render_root(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "tabs-on-value-change-with-event-details-roving-cancel",
            |cx| {
                let items = vec![
                    TabsItem::new("alpha", "Alpha", vec![]),
                    TabsItem::new("beta", "Beta", vec![]),
                    TabsItem::new("gamma", "Gamma", vec![]),
                ];
                vec![
                    Tabs::new(model.clone())
                        .activation_mode(TabsActivationMode::Automatic)
                        .on_value_change_with_event_details(Some(Arc::new(
                            move |value, event_details| {
                                if value.as_deref() == Some("beta") {
                                    event_details.prevent_default();
                                }
                                events_for_handler
                                    .lock()
                                    .unwrap_or_else(|e| e.into_inner())
                                    .push((value, *event_details));
                            },
                        )))
                        .items(items)
                        .into_element(cx),
                ]
            },
        );

        ui.set_root(root);
        ui.request_semantics_snapshot();
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        let focusable = ui
            .first_focusable_descendant_including_declarative(&mut app, window, root)
            .expect("focusable tab");
        ui.set_focus(Some(focusable));

        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::KeyDown {
                key: fret_core::KeyCode::ArrowRight,
                modifiers: Modifiers::default(),
                repeat: false,
            },
        );

        let selected = app.models().get_cloned(&model).flatten();
        assert_eq!(selected.as_deref(), Some("alpha"));

        let events = events.lock().unwrap_or_else(|e| e.into_inner());
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].0.as_deref(), Some("beta"));
        assert_eq!(
            events[0].1.source,
            TabsValueChangeSource::RovingActiveChange
        );
        assert_eq!(
            events[0].1.activation_direction,
            TabsActivationDirection::Right
        );
        assert!(events[0].1.is_canceled());
    }
}
