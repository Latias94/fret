use std::cell::Cell;
use std::sync::Arc;
use std::time::Duration;

use crate::test_id::test_id_slug;
use fret_core::window::ColorScheme;
use fret_core::{Color, Corners, DrawOrder, Edges, FontId, FontWeight, Px, TextStyle};
use fret_icons::IconId;
use fret_runtime::Model;
use fret_ui::element::{
    AnyElement, ContainerProps, CrossAlign, FlexProps, LayoutStyle, Length, MainAlign,
    PressableProps, RovingFlexProps, RovingFocusProps, ShadowLayerStyle, ShadowStyle, SpinnerProps,
    StackProps, SvgIconProps,
};
use fret_ui::elements::GlobalElementId;
use fret_ui::{ElementContext, Theme, ThemeSnapshot, UiHost};
use fret_ui_headless::motion::tolerance::Tolerance;
use fret_ui_kit::declarative::action_hooks::ActionHooksExt;
use fret_ui_kit::declarative::chrome::control_chrome_pressable_with_id_props;
use fret_ui_kit::declarative::current_color;
use fret_ui_kit::declarative::icon as decl_icon;
use fret_ui_kit::declarative::model_watch::ModelWatchExt as _;
use fret_ui_kit::declarative::motion::{
    drive_tween_color_for_element, drive_tween_f32_for_element,
};
use fret_ui_kit::declarative::motion_springs::shared_indicator_spring_description;
use fret_ui_kit::declarative::motion_value::{
    MotionToSpecF32, MotionValueF32Update, SpringSpecF32, drive_motion_value_f32,
};
use fret_ui_kit::declarative::style as decl_style;
use fret_ui_kit::declarative::transition;
use fret_ui_kit::typography;
use fret_ui_kit::{
    ChromeRefinement, ColorRef, LayoutRefinement, MetricRef, OverrideSlot, Radius, Space,
    WidgetState, WidgetStateProperty, WidgetStates, resolve_override_slot,
    resolve_override_slot_opt, ui,
};

use crate::overlay_motion;

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

fn tailwind_transition_ease_in_out(t: f32) -> f32 {
    // Tailwind default transition timing function: cubic-bezier(0.4, 0, 0.2, 1).
    // (Often described as `ease-in-out`-ish.)
    fret_ui_headless::easing::SHADCN_EASE.sample(t)
}

fn shadow_layer_with_presence(layer: ShadowLayerStyle, presence: f32) -> ShadowLayerStyle {
    let presence = presence.clamp(0.0, 1.0);
    ShadowLayerStyle {
        color: Color {
            a: (layer.color.a * presence).clamp(0.0, 1.0),
            ..layer.color
        },
        offset_x: Px(layer.offset_x.0 * presence),
        offset_y: Px(layer.offset_y.0 * presence),
        blur: Px(layer.blur.0 * presence),
        spread: Px(layer.spread.0 * presence),
    }
}

fn shadow_with_presence(shadow: ShadowStyle, presence: f32) -> ShadowStyle {
    ShadowStyle {
        primary: shadow_layer_with_presence(shadow.primary, presence),
        secondary: shadow
            .secondary
            .map(|layer| shadow_layer_with_presence(layer, presence)),
        corner_radii: shadow.corner_radii,
    }
}

fn apply_trigger_inherited_style(
    mut element: AnyElement,
    fg: Color,
    text_style: &TextStyle,
    default_icon_color: Color,
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
            // Heuristic:
            // - Older callsites may build an `SvgIcon` with the default white color.
            // - Newer callsites that use `declarative::icon::icon(...)` (outside a currentColor
            //   provider) resolve `muted-foreground` eagerly.
            //
            // In a TabsTrigger, both shapes should track the trigger foreground by default.
            let is_default_white = *color
                == Color {
                    r: 1.0,
                    g: 1.0,
                    b: 1.0,
                    a: 1.0,
                };
            let is_default_muted_fg = *color == default_icon_color;
            if is_default_white || is_default_muted_fg {
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
        .map(|child| apply_trigger_inherited_style(child, fg, text_style, default_icon_color))
        .collect();
    element
}

fn tabs_gap(theme: &ThemeSnapshot) -> Px {
    theme
        .metric_by_key("component.tabs.gap")
        .unwrap_or_else(|| MetricRef::space(Space::N2).resolve(theme))
}

fn tabs_list_height(theme: &ThemeSnapshot) -> Px {
    theme
        .metric_by_key("component.tabs.list_height")
        .unwrap_or(Px(36.0))
}

fn tabs_list_padding(theme: &ThemeSnapshot) -> Px {
    theme
        .metric_by_key("component.tabs.list_padding")
        .unwrap_or(Px(3.0))
}

fn tabs_list_bg(theme: &ThemeSnapshot) -> Color {
    theme.color_token("muted")
}

fn tabs_list_fg_muted(theme: &ThemeSnapshot) -> Color {
    theme.color_token("muted-foreground")
}

#[derive(Debug, Clone)]
pub struct TabsListVariants {
    pub chrome: ChromeRefinement,
    pub padding_px: Px,
    pub trigger_row_gap_px: Px,
}

pub fn tabs_list_variants(theme: &ThemeSnapshot, variant: TabsListVariant) -> TabsListVariants {
    match variant {
        TabsListVariant::Default => TabsListVariants {
            chrome: ChromeRefinement::default()
                .rounded(Radius::Lg)
                .bg(ColorRef::Color(tabs_list_bg(theme)))
                .text_color(ColorRef::Color(tabs_list_fg_muted(theme))),
            padding_px: tabs_list_padding(theme),
            trigger_row_gap_px: Px(0.0),
        },
        TabsListVariant::Line => TabsListVariants {
            chrome: ChromeRefinement::default()
                .bg(ColorRef::Color(Color::TRANSPARENT))
                .text_color(ColorRef::Color(tabs_list_fg_muted(theme))),
            // Upstream (radix-* registry variants) keeps `p-[3px]` even for `variant=line`.
            // Keeping the padding preserves the same trigger inset as the default variant while
            // switching the "active" affordance from a pill to a shared line indicator.
            padding_px: tabs_list_padding(theme),
            trigger_row_gap_px: Px(4.0),
        },
    }
}

fn tabs_trigger_text_style(theme: &ThemeSnapshot) -> TextStyle {
    let px = theme
        .metric_by_key("component.tabs.trigger.text_px")
        .or_else(|| theme.metric_by_key("font.size"))
        .unwrap_or_else(|| theme.metric_token("font.size"));
    let line_height = theme
        .metric_by_key("component.tabs.trigger.line_height")
        .or_else(|| theme.metric_by_key("font.line_height"))
        .unwrap_or_else(|| theme.metric_token("font.line_height"));
    let mut style = typography::fixed_line_box_style(FontId::ui(), px, line_height);
    style.weight = FontWeight::MEDIUM;
    style
}

fn tabs_trigger_radius(theme: &ThemeSnapshot) -> Px {
    theme
        .metric_by_key("component.tabs.trigger.radius")
        .unwrap_or_else(|| MetricRef::radius(Radius::Md).resolve(theme))
}

fn tabs_trigger_bg_active(theme: &ThemeSnapshot) -> Color {
    theme
        .color_by_key("component.tabs.trigger.bg_active")
        .unwrap_or_else(|| {
            if theme.color_scheme == Some(ColorScheme::Dark) {
                theme
                    .color_by_key("component.input.bg")
                    .unwrap_or_else(|| theme.color_token("background"))
            } else {
                theme.color_token("background")
            }
        })
}

fn tabs_trigger_border_active(theme: &ThemeSnapshot) -> Color {
    theme
        .color_by_key("component.tabs.trigger.border_active")
        .unwrap_or_else(|| {
            if theme.color_scheme == Some(ColorScheme::Dark) {
                theme
                    .color_by_key("input")
                    .or_else(|| theme.color_by_key("border"))
                    .unwrap_or_else(|| theme.color_token("border"))
            } else {
                Color::TRANSPARENT
            }
        })
}

fn tabs_trigger_border_width(theme: &ThemeSnapshot) -> Px {
    theme
        .metric_by_key("component.tabs.trigger.border_width")
        .unwrap_or(Px(1.0))
}

use fret_ui_kit::primitives::tabs as radix_tabs;
pub use fret_ui_kit::primitives::tabs::{TabsActivationMode, TabsOrientation};

/// Style variants for shadcn/ui `TabsList` (v4).
///
/// Mirrors the upstream `tabsListVariants({ variant })` helper exported by shadcn/ui.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TabsListVariant {
    #[default]
    Default,
    Line,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TabsListHeightOverride {
    Auto,
    Px(Px),
}

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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TabsSharedIndicatorKind {
    Pill,
    Line,
}

fn tabs_shared_indicator<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    container_id: GlobalElementId,
    orientation: TabsOrientation,
    kind: TabsSharedIndicatorKind,
    tab_count: usize,
    selected_idx: Option<usize>,
    indicator_test_id: Option<Arc<str>>,
    disabled: bool,
    indicator_line_color_override: Option<ColorRef>,
    indicator_line_outset_override: Option<Px>,
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
            line_thickness,
        ) = {
            let theme = Theme::global(&*cx.app).snapshot();

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
            let fg_inactive = crate::theme_variants::tabs_trigger_inactive_fg(&theme);
            let fg_active = ColorRef::Color(theme.color_token("foreground"));
            let fg_disabled = ColorRef::Color(alpha_mul(theme.color_token("foreground"), 0.5));

            let bg_active = ColorRef::Color(tabs_trigger_bg_active(&theme));
            let border_active = ColorRef::Color(tabs_trigger_border_active(&theme));
            let border_w = tabs_trigger_border_width(&theme);
            let radius = tabs_trigger_radius(&theme);

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
            .map(|bg| bg.resolve(&theme))
            .unwrap_or(Color::TRANSPARENT);
            let border_color = resolve_override_slot_opt(
                style_override.trigger_border_color.as_ref(),
                &default_trigger_border,
                states,
            )
            .map(|border| border.resolve(&theme))
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

            let spring = shared_indicator_spring_description(&*cx.app);

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
                line_thickness,
            ) = match kind {
                TabsSharedIndicatorKind::Pill => (
                    target_x,
                    target_y,
                    target_width,
                    target_height,
                    bg,
                    border_color,
                    border_w,
                    (!disabled && selected_idx.is_some())
                        .then(|| decl_style::shadow_sm(&theme, radius)),
                    radius,
                    0.0,
                ),
                TabsSharedIndicatorKind::Line => {
                    let thickness = theme
                        .metric_by_key("component.tabs.indicator.line_thickness")
                        .unwrap_or(Px(2.0))
                        .0
                        .max(0.0);
                    let (target_x, target_y, target_width, target_height) = if thickness > 0.0 {
                        match orientation {
                            TabsOrientation::Horizontal => {
                                (target_x, target_y, target_width, thickness)
                            }
                            TabsOrientation::Vertical => {
                                (target_x, target_y, thickness, target_height)
                            }
                        }
                    } else {
                        (target_x, target_y, target_width, target_height)
                    };

                    let bg = if !disabled && selected_idx.is_some() {
                        indicator_line_color_override
                            .as_ref()
                            .map(|color| color.resolve(&theme))
                            .unwrap_or_else(|| theme.color_token("foreground"))
                    } else {
                        Color::TRANSPARENT
                    };
                    (
                        target_x,
                        target_y,
                        target_width,
                        target_height,
                        bg,
                        Color::TRANSPARENT,
                        Px(0.0),
                        None,
                        Px(0.0),
                        thickness,
                    )
                }
            };

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
                line_thickness,
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
        props.layout.inset.top = Some(Px(0.0)).into();
        props.layout.inset.right = Some(Px(0.0)).into();
        props.layout.inset.bottom = Some(Px(0.0)).into();
        props.layout.inset.left = Some(Px(0.0)).into();
        if kind == TabsSharedIndicatorKind::Line {
            // shadcn v4 draws the line "outside" the trigger via negative offsets:
            // - horizontal: `bottom-[-5px]`
            // - vertical: `-right-1` (4px)
            //
            // Extend the indicator canvas so we can paint into that extra area.
            let outset = indicator_line_outset_override.unwrap_or_else(|| match orientation {
                TabsOrientation::Horizontal => Px(5.0),
                TabsOrientation::Vertical => Px(4.0),
            });
            let outset = Px(outset.0.max(0.0));
            match orientation {
                TabsOrientation::Horizontal => {
                    if outset.0 > 0.0 {
                        props.layout.inset.bottom = Some(Px(-outset.0)).into();
                    }
                }
                TabsOrientation::Vertical => {
                    if outset.0 > 0.0 {
                        props.layout.inset.right = Some(Px(-outset.0)).into();
                    }
                }
            }
        }

        let mut indicator = cx.canvas(props, move |p| {
            let bounds = p.bounds();
            // The shared indicator targets trigger bounds tracked relative to the *list container*
            // element (`container_bounds`). Depending on how absolute-positioned children are
            // resolved in the current layout backend, the canvas bounds may be anchored to the
            // container's padding box or content box. Convert from container-local offsets to
            // canvas-local offsets so the indicator stays aligned under list padding.
            let dx = container_bounds.origin.x.0 - bounds.origin.x.0;
            let dy = container_bounds.origin.y.0 - bounds.origin.y.0;

            let x_base = (x.value + dx).clamp(0.0, bounds.size.width.0);
            let y_base = (y.value + dy).clamp(0.0, bounds.size.height.0);
            let max_width_base = (bounds.size.width.0 - x_base).max(0.0);
            let max_height_base = (bounds.size.height.0 - y_base).max(0.0);

            let (x_px, y_px, width_px, height_px) = match kind {
                TabsSharedIndicatorKind::Line => {
                    let thickness = line_thickness.max(0.0);
                    if thickness <= 0.0 {
                        return;
                    }

                    match orientation {
                        TabsOrientation::Horizontal => {
                            let y_px = (bounds.size.height.0 - thickness).max(0.0);
                            let max_height = (bounds.size.height.0 - y_px).max(0.0);
                            let height_px = thickness.clamp(0.0, max_height);
                            let width_px = width.value.clamp(0.0, max_width_base);
                            (x_base, y_px, width_px, height_px)
                        }
                        TabsOrientation::Vertical => {
                            let x_px = (bounds.size.width.0 - thickness).max(0.0);
                            let max_width = (bounds.size.width.0 - x_px).max(0.0);
                            let width_px = thickness.clamp(0.0, max_width);
                            let height_px = height.value.clamp(0.0, max_height_base);
                            (x_px, y_base, width_px, height_px)
                        }
                    }
                }
                TabsSharedIndicatorKind::Pill => {
                    let width_px = width.value.clamp(0.0, max_width_base);
                    let height_px = height.value.clamp(0.0, max_height_base);
                    (x_base, y_base, width_px, height_px)
                }
            };

            if height_px <= 0.0 || width_px <= 0.0 || bg.a <= 0.0 {
                return;
            }

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

        cx.hit_test_gate(false, move |_cx| [indicator])
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
#[derive(Debug)]
pub struct TabsTrigger {
    value: Arc<str>,
    label: Arc<str>,
    children: Option<Vec<AnyElement>>,
    test_id: Option<Arc<str>>,
    disabled: bool,
}

impl TabsTrigger {
    pub fn new(value: impl Into<Arc<str>>, label: impl Into<Arc<str>>) -> Self {
        Self {
            value: value.into(),
            label: label.into(),
            children: None,
            test_id: None,
            disabled: false,
        }
    }

    pub fn test_id(mut self, test_id: impl Into<Arc<str>>) -> Self {
        self.test_id = Some(test_id.into());
        self
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
#[derive(Debug, Default)]
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
#[derive(Debug)]
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
pub struct TabsRoot {
    model: Option<Model<Option<Arc<str>>>>,
    default_value: Option<Arc<str>>,
    list: TabsList,
    contents: Vec<TabsContent>,
    disabled: bool,
    orientation: TabsOrientation,
    activation_mode: TabsActivationMode,
    loop_navigation: bool,
    list_variant: TabsListVariant,
    gap_px_override: Option<Px>,
    list_padding_px_override: Option<Px>,
    list_height_override: Option<TabsListHeightOverride>,
    style: TabsStyle,
    indicator_line_color_override: Option<ColorRef>,
    indicator_line_outset_override: Option<Px>,
    trigger_padding_override: Option<(Px, Px)>,
    trigger_radius_override: Option<Px>,
    trigger_border_width_override: Option<Px>,
    chrome: ChromeRefinement,
    layout: LayoutRefinement,
    force_mount_content: bool,
    list_full_width: bool,
    list_bar_borders: bool,
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
            .field("list_variant", &self.list_variant)
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
            list_variant: TabsListVariant::default(),
            gap_px_override: None,
            list_padding_px_override: None,
            list_height_override: None,
            style: TabsStyle::default(),
            indicator_line_color_override: None,
            indicator_line_outset_override: None,
            trigger_padding_override: None,
            trigger_radius_override: None,
            trigger_border_width_override: None,
            chrome: ChromeRefinement::default(),
            layout: LayoutRefinement::default(),
            force_mount_content: false,
            list_full_width: false,
            list_bar_borders: false,
            content_fill_remaining: true,
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
            list_variant: TabsListVariant::default(),
            gap_px_override: None,
            list_padding_px_override: None,
            list_height_override: None,
            style: TabsStyle::default(),
            indicator_line_color_override: None,
            indicator_line_outset_override: None,
            trigger_padding_override: None,
            trigger_radius_override: None,
            trigger_border_width_override: None,
            chrome: ChromeRefinement::default(),
            layout: LayoutRefinement::default(),
            force_mount_content: false,
            list_full_width: false,
            list_bar_borders: false,
            content_fill_remaining: true,
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

    pub fn list_variant(mut self, variant: TabsListVariant) -> Self {
        self.list_variant = variant;
        self
    }

    pub fn gap_px(mut self, gap_px: Px) -> Self {
        self.gap_px_override = Some(gap_px);
        self
    }

    pub fn list_padding_px(mut self, padding_px: Px) -> Self {
        self.list_padding_px_override = Some(padding_px);
        self
    }

    pub fn list_height_override(mut self, height: TabsListHeightOverride) -> Self {
        self.list_height_override = Some(height);
        self
    }

    pub fn indicator_line_color(mut self, color: ColorRef) -> Self {
        self.indicator_line_color_override = Some(color);
        self
    }

    pub fn indicator_line_outset(mut self, outset: Px) -> Self {
        self.indicator_line_outset_override = Some(outset);
        self
    }

    pub fn trigger_padding(mut self, pad_x: Px, pad_y: Px) -> Self {
        self.trigger_padding_override = Some((pad_x, pad_y));
        self
    }

    pub fn trigger_radius(mut self, radius: Px) -> Self {
        self.trigger_radius_override = Some(radius);
        self
    }

    pub fn trigger_border_width(mut self, border_width: Px) -> Self {
        self.trigger_border_width_override = Some(border_width);
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

    /// When `true`, paints 1px top/bottom border lines across the full available width around the
    /// tab list (AI Elements `SandboxTabsBar` affordance).
    pub fn list_bar_borders(mut self, enabled: bool) -> Self {
        self.list_bar_borders = enabled;
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
        let Self {
            model,
            default_value,
            list,
            contents,
            disabled,
            orientation,
            activation_mode,
            loop_navigation,
            list_variant,
            gap_px_override,
            list_padding_px_override,
            list_height_override,
            style,
            indicator_line_color_override,
            indicator_line_outset_override,
            trigger_padding_override,
            trigger_radius_override,
            trigger_border_width_override,
            chrome,
            layout,
            force_mount_content,
            list_full_width,
            list_bar_borders,
            content_fill_remaining,
            shared_indicator_motion,
            content_presence_motion,
            test_id,
            on_value_change,
            on_value_change_with_source,
            on_value_change_with_details,
            on_value_change_with_event_details,
        } = self;

        let mut content_by_value: std::collections::HashMap<Arc<str>, Vec<AnyElement>> =
            std::collections::HashMap::new();
        for content in contents {
            content_by_value.insert(content.value.clone(), content.children);
        }

        let items: Vec<TabsItem> = list
            .triggers
            .into_iter()
            .map(|trigger| {
                let content = content_by_value
                    .remove(trigger.value.as_ref())
                    .unwrap_or_default();
                let mut item = TabsItem::new(trigger.value, trigger.label, content)
                    .trigger_children(trigger.children.unwrap_or_default())
                    .disabled(trigger.disabled);
                if let Some(test_id) = trigger.test_id {
                    item = item.trigger_test_id(test_id);
                }
                item
            })
            .collect();

        let tabs = if let Some(model) = model {
            Tabs::new(model)
        } else {
            Tabs::uncontrolled(default_value)
        };

        tabs.disabled(disabled)
            .orientation(orientation)
            .activation_mode(activation_mode)
            .loop_navigation(loop_navigation)
            .list_variant(list_variant)
            .gap_px_opt(gap_px_override)
            .list_padding_px_opt(list_padding_px_override)
            .list_height_override_opt(list_height_override)
            .indicator_line_color_opt(indicator_line_color_override)
            .indicator_line_outset_opt(indicator_line_outset_override)
            .trigger_padding_opt(trigger_padding_override)
            .trigger_radius_opt(trigger_radius_override)
            .trigger_border_width_opt(trigger_border_width_override)
            .on_value_change(on_value_change)
            .on_value_change_with_source(on_value_change_with_source)
            .on_value_change_with_details(on_value_change_with_details)
            .on_value_change_with_event_details(on_value_change_with_event_details)
            .style(style)
            .refine_style(chrome)
            .refine_layout(layout)
            .force_mount_content(force_mount_content)
            .list_full_width(list_full_width)
            .list_bar_borders(list_bar_borders)
            .content_fill_remaining(content_fill_remaining)
            .shared_indicator_motion(shared_indicator_motion)
            .content_presence_motion(content_presence_motion)
            .test_id_opt(test_id)
            .items(items)
            .into_element(cx)
    }
}

#[derive(Debug)]
pub struct TabsItem {
    value: Arc<str>,
    label: Arc<str>,
    content: Vec<AnyElement>,
    trigger: Option<Vec<AnyElement>>,
    trigger_leading_icon: Option<IconId>,
    trigger_trailing_icon: Option<IconId>,
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
            trigger_leading_icon: None,
            trigger_trailing_icon: None,
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

    /// Adds a leading icon to the default trigger contents (icon + label).
    ///
    /// This is preferred over pre-built icon elements because `declarative::icon::icon(...)`
    /// resolves its final color during `into_element(cx)`. By deferring icon construction to the
    /// `Tabs` host, the icon inherits the trigger foreground via the `currentColor` provider.
    pub fn trigger_leading_icon(mut self, icon: IconId) -> Self {
        self.trigger_leading_icon = Some(icon);
        self
    }

    /// Adds a trailing icon to the default trigger contents (label + icon).
    pub fn trigger_trailing_icon(mut self, icon: IconId) -> Self {
        self.trigger_trailing_icon = Some(icon);
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

pub struct Tabs {
    model: Option<Model<Option<Arc<str>>>>,
    default_value: Option<Arc<str>>,
    items: Vec<TabsItem>,
    disabled: bool,
    orientation: TabsOrientation,
    activation_mode: TabsActivationMode,
    loop_navigation: bool,
    list_variant: TabsListVariant,
    gap_px_override: Option<Px>,
    list_padding_px_override: Option<Px>,
    list_height_override: Option<TabsListHeightOverride>,
    style: TabsStyle,
    indicator_line_color_override: Option<ColorRef>,
    indicator_line_outset_override: Option<Px>,
    trigger_padding_override: Option<(Px, Px)>,
    trigger_radius_override: Option<Px>,
    trigger_border_width_override: Option<Px>,
    chrome: ChromeRefinement,
    layout: LayoutRefinement,
    force_mount_content: bool,
    list_full_width: bool,
    list_bar_borders: bool,
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
            .field("list_variant", &self.list_variant)
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
            list_variant: TabsListVariant::default(),
            gap_px_override: None,
            list_padding_px_override: None,
            list_height_override: None,
            style: TabsStyle::default(),
            indicator_line_color_override: None,
            indicator_line_outset_override: None,
            trigger_padding_override: None,
            trigger_radius_override: None,
            trigger_border_width_override: None,
            chrome: ChromeRefinement::default(),
            layout: LayoutRefinement::default(),
            force_mount_content: false,
            list_full_width: false,
            list_bar_borders: false,
            content_fill_remaining: true,
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
            list_variant: TabsListVariant::default(),
            gap_px_override: None,
            list_padding_px_override: None,
            list_height_override: None,
            style: TabsStyle::default(),
            indicator_line_color_override: None,
            indicator_line_outset_override: None,
            trigger_padding_override: None,
            trigger_radius_override: None,
            trigger_border_width_override: None,
            chrome: ChromeRefinement::default(),
            layout: LayoutRefinement::default(),
            force_mount_content: false,
            list_full_width: false,
            list_bar_borders: false,
            content_fill_remaining: true,
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

    pub fn list_variant(mut self, variant: TabsListVariant) -> Self {
        self.list_variant = variant;
        self
    }

    pub fn gap_px_opt(mut self, gap_px: Option<Px>) -> Self {
        self.gap_px_override = gap_px;
        self
    }

    pub fn list_padding_px_opt(mut self, padding_px: Option<Px>) -> Self {
        self.list_padding_px_override = padding_px;
        self
    }

    pub fn list_height_override_opt(mut self, height: Option<TabsListHeightOverride>) -> Self {
        self.list_height_override = height;
        self
    }

    pub fn indicator_line_color_opt(mut self, color: Option<ColorRef>) -> Self {
        self.indicator_line_color_override = color;
        self
    }

    pub fn indicator_line_outset_opt(mut self, outset: Option<Px>) -> Self {
        self.indicator_line_outset_override = outset;
        self
    }

    pub fn trigger_padding_opt(mut self, padding: Option<(Px, Px)>) -> Self {
        self.trigger_padding_override = padding;
        self
    }

    pub fn trigger_radius_opt(mut self, radius: Option<Px>) -> Self {
        self.trigger_radius_override = radius;
        self
    }

    pub fn trigger_border_width_opt(mut self, border_width: Option<Px>) -> Self {
        self.trigger_border_width_override = border_width;
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

    /// When `true`, paints 1px top/bottom border lines across the full available width around the
    /// tab list (AI Elements `SandboxTabsBar` affordance).
    pub fn list_bar_borders(mut self, enabled: bool) -> Self {
        self.list_bar_borders = enabled;
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
        let list_variant = self.list_variant;
        let gap_px_override = self.gap_px_override;
        let list_padding_px_override = self.list_padding_px_override;
        let list_height_override = self.list_height_override;
        let style_override = self.style;
        let indicator_line_color_override = self.indicator_line_color_override;
        let indicator_line_outset_override = self.indicator_line_outset_override;
        let trigger_padding_override = self.trigger_padding_override;
        let trigger_radius_override = self.trigger_radius_override;
        let trigger_border_width_override = self.trigger_border_width_override;
        let chrome = self.chrome;
        let layout = self.layout;
        let force_mount_content = self.force_mount_content;
        let list_full_width = self.list_full_width;
        let list_bar_borders = self.list_bar_borders;
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

        let theme = Theme::global(&*cx.app).snapshot();
        let mut gap = tabs_gap(&theme);
        if let Some(gap_px_override) = gap_px_override {
            gap = gap_px_override;
        }
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
        let TabsListVariants {
            chrome: list_chrome,
            padding_px: list_padding,
            trigger_row_gap_px: trigger_row_gap,
        } = tabs_list_variants(&theme, list_variant);
        let list_padding = list_padding_px_override.unwrap_or(list_padding);
        // shadcn new-york-v4:
        // - horizontal: `group-data-[orientation=horizontal]/tabs:h-9`
        // - vertical: `group-data-[orientation=vertical]/tabs:h-fit`
        //
        // The horizontal list has a fixed height so the active pill can be vertically centered.
        // In vertical orientation, the list must grow to fit all triggers.
        let list_height_opt = match orientation {
            TabsOrientation::Horizontal => match list_height_override {
                Some(TabsListHeightOverride::Auto) => None,
                Some(TabsListHeightOverride::Px(height)) => Some(height),
                None => Some(list_height),
            },
            TabsOrientation::Vertical => None,
        };
        let list_layout = match (orientation, list_height_opt) {
            (TabsOrientation::Horizontal, Some(height)) => LayoutRefinement::default().h_px(height),
            _ => LayoutRefinement::default(),
        };
        let mut list_props = decl_style::container_props(&theme, list_chrome, list_layout);
        list_props.padding = Edges::all(list_padding).into();
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
                // shadcn/ui uses `flex-1` on `TabsContent`. In practice that intent is "fill the
                // remaining main-axis space when the parent is a flex container".
                //
                // Note: Avoid Tailwind's `flex: 1 1 0%` (`basis=0`) here. Taffy currently
                // shrink-wraps some auto-sized flex containers around the sum of flex bases, which
                // can collapse panels in unconstrained compositions. `flex: 1 1 auto` keeps
                // intrinsic sizing stable while still allowing fill in the common case.
                refinement = refinement.flex_grow(1.0).flex_shrink(1.0);
            }
            decl_style::layout_style(&theme, refinement)
        };

        struct TabsItemFrame {
            value: Arc<str>,
            label: Arc<str>,
            trigger: Option<Vec<AnyElement>>,
            trigger_leading_icon: Option<IconId>,
            trigger_trailing_icon: Option<IconId>,
            trigger_test_id: Option<Arc<str>>,
        }

        let items_len = items.len();
        let mut items_frame: Vec<TabsItemFrame> = Vec::with_capacity(items_len);
        let mut content_by_value: std::collections::HashMap<Arc<str>, Vec<AnyElement>> =
            std::collections::HashMap::new();
        for item in items {
            content_by_value.insert(item.value.clone(), item.content);
            items_frame.push(TabsItemFrame {
                value: item.value,
                label: item.label,
                trigger: item.trigger,
                trigger_leading_icon: item.trigger_leading_icon,
                trigger_trailing_icon: item.trigger_trailing_icon,
                trigger_test_id: item.trigger_test_id,
            });
        }

        let active_label = active_idx
            .and_then(|active| items_frame.get(active))
            .map(|item| item.label.clone())
            .unwrap_or_else(|| Arc::from(""));
        let active_value = active_idx
            .and_then(|active| items_frame.get(active))
            .map(|item| item.value.clone());

        let tab_list_bar_border_color = theme
            .color_by_key("border")
            .unwrap_or_else(|| theme.color_token("border"));

        let root_props = decl_style::container_props(&theme, chrome, layout);

        let mut root = cx.container(root_props, move |cx| {
            let mut items = items_frame;
            let mut content_by_value = content_by_value;

            let selected_tab_element: Cell<Option<u64>> = Cell::new(None);
            let selected_tab_element = &selected_tab_element;
            let tab_trigger_elements: Vec<Cell<Option<u64>>> =
                (0..items_len).map(|_| Cell::new(None)).collect();
            let tab_trigger_elements = &tab_trigger_elements;
            let mut children: Vec<AnyElement> = Vec::new();
            let content_stage_test_id = root_test_id_for_children
                .as_ref()
                .map(|id| Arc::<str>::from(format!("{id}-content-stage")));

            let tab_list_semantics =
                radix_tabs::tab_list_semantics_props(list_props.layout, orientation);
            let tab_list = cx.semantics(tab_list_semantics, |cx| {
                vec![cx.container(list_props, |cx| {
                    let list_container_id = cx.root_id();
                    let indicator_test_id = root_test_id_for_children
                        .as_ref()
                        .map(|id| Arc::<str>::from(format!("{id}-shared-indicator")));

                    let mut list_children: Vec<AnyElement> = Vec::new();
                        let indicator_kind = if list_variant == TabsListVariant::Line {
                            Some(TabsSharedIndicatorKind::Line)
                        } else if shared_indicator_motion {
                            Some(TabsSharedIndicatorKind::Pill)
                        } else {
                            None
                        };
                        if let Some(indicator_kind) = indicator_kind
                            && indicator_kind != TabsSharedIndicatorKind::Line
                        {
                            // Pill highlight should paint under the triggers.
                            list_children.push(tabs_shared_indicator(
                                cx,
                                list_container_id,
                                orientation,
                                indicator_kind,
                                items_len,
                                active_idx,
                                indicator_test_id.clone(),
                                tabs_disabled,
                                indicator_line_color_override.clone(),
                                indicator_line_outset_override,
                                &style_override,
                            ));
                        }

                        // Vertical tabs should keep triggers at a shared width (shadcn's `w-full`
                        // outcome relative to the widest trigger). Our layout engine does not
                        // fully match CSS percentage sizing in shrink-to-fit containers, so we
                        // stabilize this by reusing the previous frame's measured trigger width.
                        let vertical_trigger_width_px: Option<Px> =
                            if orientation == TabsOrientation::Vertical {
                                let trigger_ids = cx.with_state_for(
                                    list_container_id,
                                    TabsListLayoutRuntime::default,
                                    |rt| rt.triggers.clone(),
                                );
                                let mut max_w: Option<f32> = None;
                                for id in trigger_ids {
                                    if let Some(bounds) = cx.last_bounds_for_element(id) {
                                        max_w = Some(
                                            max_w
                                                .map_or(bounds.size.width.0, |w| w.max(bounds.size.width.0)),
                                        );
                                    }
                                }
                                max_w.map(Px)
                            } else {
                                None
                            };

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
                                        layout.size.height = match orientation {
                                            TabsOrientation::Horizontal => Length::Fill,
                                            TabsOrientation::Vertical => Length::Auto,
                                        };
                                        if list_full_width {
                                            layout.size.width = Length::Fill;
                                        }
                                        layout
                                    },
                                    direction: match orientation {
                                        TabsOrientation::Horizontal => fret_core::Axis::Horizontal,
                                        TabsOrientation::Vertical => fret_core::Axis::Vertical,
                                    },
                                    gap: trigger_row_gap.into(),
                                    padding: Edges::all(Px(0.0)).into(),
                                    // NOTE: Taffy currently shrink-wraps auto-sized flex containers
                                    // around the sum of flex bases, not the min-content widths.
                                    // With Tailwind-like `flex-1` (`basis=0`), that yields a
                                    // zero-width container and a negative "centered" offset.
                                    // `justify-start` matches shadcn for `flex-1` triggers while
                                    // avoiding negative positions in shrink-wrapped lists.
                                    justify: MainAlign::Start,
                                    align: match orientation {
                                        TabsOrientation::Horizontal => CrossAlign::Center,
                                        TabsOrientation::Vertical => CrossAlign::Stretch,
                                    },
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
                                let fg_inactive =
                                    crate::theme_variants::tabs_trigger_inactive_fg(&theme);
                                let fg_active = ColorRef::Color(theme.color_token("foreground"));
                                let fg_disabled =
                                    ColorRef::Color(alpha_mul(theme.color_token("foreground"), 0.5));
                                let radius =
                                    trigger_radius_override.unwrap_or_else(|| tabs_trigger_radius(&theme));
                                let ring = decl_style::focus_ring(&theme, radius);
                                let bg_active =
                                    ColorRef::Color(tabs_trigger_bg_active(&theme));
                                let border_active =
                                    ColorRef::Color(tabs_trigger_border_active(&theme));
                                let border_w = trigger_border_width_override
                                    .unwrap_or_else(|| tabs_trigger_border_width(&theme));

                                let default_trigger_fg = WidgetStateProperty::new(fg_inactive)
                                    .when(WidgetStates::SELECTED, fg_active)
                                    .when(WidgetStates::DISABLED, fg_disabled);
                                let default_trigger_bg = if list_variant == TabsListVariant::Line {
                                    WidgetStateProperty::new(None)
                                } else {
                                    WidgetStateProperty::new(None)
                                        .when(WidgetStates::SELECTED, Some(bg_active))
                                };
                                let default_trigger_border = if list_variant == TabsListVariant::Line
                                {
                                    WidgetStateProperty::new(None)
                                } else {
                                    WidgetStateProperty::new(None)
                                        .when(WidgetStates::SELECTED, Some(border_active))
                                };

                                let (pad_x, pad_y) =
                                    trigger_padding_override.unwrap_or_else(|| {
                                        (
                                            MetricRef::space(Space::N2).resolve(&theme),
                                            MetricRef::space(Space::N1).resolve(&theme),
                                        )
                                    });
                                // new-york-v4: trigger uses `h-[calc(100%-1px)]` relative to the list
                                // content box (after list padding).
                                //
                                // In a GPU-first renderer, the 1px delta is a frequent source of
                                // rounding drift at non-integer scale factors. More importantly,
                                // shadcn's trigger chrome includes `py-1` + a 1px border; a 2px
                                // height reduction can starve the inner content box enough to read
                                // as bottom-heavy (the label overflows into the bottom padding).
                                //
                                // Prefer filling the list content box by default; this preserves a
                                // stable baseline for vertical centering while keeping the list's
                                // outer padding (`p-[3px]`) as the primary visual inset.
                                let trigger_h = list_height_opt.map(|list_height| {
                                    Px((list_height.0 - list_padding.0 * 2.0).max(0.0))
                                });
                                let mut trigger_refinement = LayoutRefinement::default();
                                if list_full_width {
                                    trigger_refinement = trigger_refinement.flex_1();
                                }
                                if let Some(trigger_h) = trigger_h {
                                    trigger_refinement = trigger_refinement.h_px(trigger_h);
                                }
                                let trigger_layout =
                                    decl_style::layout_style(&theme, trigger_refinement);

                                let list_item_count = items.len();
                                let mut out: Vec<AnyElement> =
                                    Vec::with_capacity(disabled_flags.len());
                                for (idx, item) in items.iter_mut().enumerate() {
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

                                    let value = item.value.clone();
                                    let label = item.label.clone();
                                    let trigger_children = item.trigger.take();
                                    let trigger_leading_icon = item.trigger_leading_icon.clone();
                                    let trigger_trailing_icon = item.trigger_trailing_icon.clone();
                                    let trigger_test_id = item.trigger_test_id.clone().or_else(|| {
                                        root_test_id_for_children.as_ref().map(|id| {
                                            Arc::<str>::from(format!(
                                                "{id}-trigger-{}",
                                                test_id_slug(item.value.as_ref())
                                            ))
                                        })
                                    });
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
                                        let duration = overlay_motion::shadcn_motion_duration_150(cx);

                                        let focus_visible =
                                            states.contains(WidgetStates::FOCUS_VISIBLE);

                                        let fg_motion = drive_tween_color_for_element(
                                            cx,
                                            id,
                                            "tabs.trigger.fg",
                                            fg_ref.resolve(&theme),
                                            duration,
                                            tailwind_transition_ease_in_out,
                                        );
                                        let fg = fg_motion.value;
                                        let fg_ref = ColorRef::Color(fg);

                                        let default_icon_color = theme
                                            .color_by_key("muted-foreground")
                                            .unwrap_or_else(|| theme.color_token("muted-foreground"));
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

                                        let border_motion = drive_tween_color_for_element(
                                            cx,
                                            id,
                                            "tabs.trigger.border",
                                            if focus_visible {
                                                theme.color_token("ring")
                                            } else {
                                                border
                                            },
                                            duration,
                                            tailwind_transition_ease_in_out,
                                        );
                                        let border = border_motion.value;

                                        let ring_alpha = drive_tween_f32_for_element(
                                            cx,
                                            id,
                                            "tabs.trigger.ring.alpha",
                                            if focus_visible { 1.0 } else { 0.0 },
                                            duration,
                                            tailwind_transition_ease_in_out,
                                        );
                                        let mut ring = ring;
                                        ring.color.a = (ring.color.a * ring_alpha.value)
                                            .clamp(0.0, 1.0);
                                        if let Some(offset_color) = ring.offset_color {
                                            ring.offset_color = Some(Color {
                                                a: (offset_color.a * ring_alpha.value)
                                                    .clamp(0.0, 1.0),
                                                ..offset_color
                                            });
                                        }

                                        let shadow_presence = drive_tween_f32_for_element(
                                            cx,
                                            id,
                                            "tabs.trigger.shadow.presence",
                                            if !shared_indicator_motion
                                                && list_variant == TabsListVariant::Default
                                                && active
                                                && !item_disabled
                                            {
                                                1.0
                                            } else {
                                                0.0
                                            },
                                            duration,
                                            tailwind_transition_ease_in_out,
                                        );
                                        let wants_shadow =
                                            shadow_presence.animating || shadow_presence.value > 0.0;
                                        let shadow = wants_shadow.then(|| {
                                            shadow_with_presence(
                                                decl_style::shadow_sm(&theme, radius),
                                                shadow_presence.value,
                                            )
                                        });

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

                                        let mut props = PressableProps {
                                            layout: trigger_layout,
                                            enabled: !item_disabled,
                                            focusable: tab_stop || st.focused,
                                            focus_ring: Some(ring),
                                            focus_ring_always_paint: ring_alpha.animating,
                                            a11y,
                                            ..Default::default()
                                        };

                                        if orientation == TabsOrientation::Vertical {
                                            if let Some(w) = vertical_trigger_width_px {
                                                props.layout.size.width = Length::Px(w);
                                            }
                                            props.layout.flex.align_self = Some(CrossAlign::Stretch);
                                        }

                                        let mut chrome = ContainerProps {
                                            padding: Edges {
                                                top: pad_y,
                                                right: pad_x,
                                                bottom: pad_y,
                                                left: pad_x,
                                            }.into(),
                                            background: bg,
                                            shadow,
                                            border: Edges::all(border_w),
                                            border_color: Some(border),
                                            corner_radii: Corners::all(radius),
                                            ..Default::default()
                                        };
                                        if orientation == TabsOrientation::Vertical {
                                            chrome.layout.size.width = Length::Fill;
                                        }

                                        let mut trigger_children = trigger_children;
                                        let content = move |cx: &mut ElementContext<'_, H>| {
                                            current_color::scope_children(
                                                cx,
                                                fg_ref.clone(),
                                                |cx| {
                                                    let base = trigger_children.take().unwrap_or_else(|| {
                                                        let style = text_style.clone();
                                                        let mut out: Vec<AnyElement> = Vec::new();
                                                        if let Some(icon) = trigger_leading_icon.clone() {
                                                            out.push(decl_icon::icon(cx, icon));
                                                        }

                                                        let mut text = ui::label( label.clone())
                                                            .text_size_px(style.size)
                                                            .font_weight(style.weight)
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
                                                        out.push(text.into_element(cx));

                                                        if let Some(icon) =
                                                            trigger_trailing_icon.clone()
                                                        {
                                                            out.push(decl_icon::icon(cx, icon));
                                                        }
                                                        out
                                                    });

                                                    let styled: Vec<AnyElement> = base
                                                        .into_iter()
                                                        .map(|child| {
                                                            apply_trigger_inherited_style(
                                                                child,
                                                                fg,
                                                                &text_style,
                                                                default_icon_color,
                                                            )
                                                        })
                                                        .collect();

                                                    vec![cx.flex(
                                                        FlexProps {
                                                            layout: {
                                                                let mut layout =
                                                                    LayoutStyle::default();
                                                                // Avoid `width: Fill` in shrink-wrapped trigger chrome:
                                                                // some layout backends (Taffy) can resolve percent sizing
                                                                // against an auto-sized containing block as 0px, causing the
                                                                // trigger background to collapse while its children overflow.
                                                                //
                                                                // Only opt into fill sizing when the trigger itself is
                                                                // expected to have a definite width (e.g. `list_full_width`
                                                                // `flex-1` triggers).
                                                                layout.size.width = if list_full_width {
                                                                    Length::Fill
                                                                } else {
                                                                    Length::Auto
                                                                };
                                                                layout.size.height = Length::Fill;
                                                                layout
                                                            },
                                                            direction: fret_core::Axis::Horizontal,
                                                            gap: Px(6.0).into(),
                                                            padding: Edges::all(Px(0.0)).into(),
                                                            justify: match orientation {
                                                                TabsOrientation::Horizontal => {
                                                                    MainAlign::Center
                                                                }
                                                                TabsOrientation::Vertical => {
                                                                    MainAlign::Start
                                                                }
                                                            },
                                                            align: CrossAlign::Center,
                                                            wrap: false,
                                                        },
                                                        move |_cx| styled,
                                                    )]
                                                },
                                            )
                                        };

                                        (props, chrome, content)
                                        })
                                     }));
                                 }
                             out
                         },
                        ));
                        if let Some(indicator_kind) = indicator_kind
                            && indicator_kind == TabsSharedIndicatorKind::Line
                        {
                            // shadcn's line indicator is visually "above" the triggers.
                            // Keep it last in the list so it paints on top.
                            list_children.push(tabs_shared_indicator(
                                cx,
                                list_container_id,
                                orientation,
                                indicator_kind,
                                items_len,
                                active_idx,
                                indicator_test_id,
                                tabs_disabled,
                                indicator_line_color_override.clone(),
                                indicator_line_outset_override,
                                &style_override,
                            ));
                        }
                    list_children
                })]
            });

            if list_bar_borders {
                let border_color = tab_list_bar_border_color;
                let bar = cx.container(
                    ContainerProps {
                        layout: {
                            let mut layout = LayoutStyle::default();
                            layout.size.width = Length::Fill;
                            layout
                        },
                        ..Default::default()
                    },
                    move |cx| {
                        let mut out: Vec<AnyElement> = Vec::with_capacity(3);
                        let border_top = border_color;
                        let border_bottom = border_color;

                        out.push(cx.named("tabs_list_bar_border_top", move |cx| {
                            let mut layout = LayoutStyle::default();
                            layout.position = fret_ui::element::PositionStyle::Absolute;
                            layout.inset.top = Some(Px(0.0)).into();
                            layout.inset.left = Some(Px(0.0)).into();
                            layout.inset.right = Some(Px(0.0)).into();
                            layout.size.width = Length::Fill;
                            layout.size.height = Length::Px(Px(1.0));

                            let line = cx.container(
                                ContainerProps {
                                    layout,
                                    background: Some(border_top),
                                    ..Default::default()
                                },
                                |_cx| Vec::new(),
                            );

                            cx.hit_test_gate(false, move |_cx| [line])
                        }));

                        out.push(cx.named("tabs_list_bar_border_bottom", move |cx| {
                            let mut layout = LayoutStyle::default();
                            layout.position = fret_ui::element::PositionStyle::Absolute;
                            layout.inset.bottom = Some(Px(0.0)).into();
                            layout.inset.left = Some(Px(0.0)).into();
                            layout.inset.right = Some(Px(0.0)).into();
                            layout.size.width = Length::Fill;
                            layout.size.height = Length::Px(Px(1.0));

                            let line = cx.container(
                                ContainerProps {
                                    layout,
                                    background: Some(border_bottom),
                                    ..Default::default()
                                },
                                |_cx| Vec::new(),
                            );

                            cx.hit_test_gate(false, move |_cx| [line])
                        }));

                        out.push(tab_list);
                        out
                    },
                );
                children.push(bar);
            } else {
                children.push(tab_list);
            }

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
                        cx.keyed_slot_state("content_presence", TabsContentPresenceRuntime::default, |st| {
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
                        let Some(content) = content_by_value.remove(value.as_ref()) else {
                            prune.push(value);
                            continue;
                        };

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
                        if let Some(content) = content_by_value.remove(active_value.as_ref()) {
                            let labelled_by_element = selected_tab_element.get();
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
                        cx.keyed_slot_state("content_presence", TabsContentPresenceRuntime::default, |st| {
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
                } else {
                    let active_children = active_value
                        .as_ref()
                        .and_then(|value| content_by_value.remove(value.as_ref()))
                        .unwrap_or_default();

                    if let Some(panel) = radix_tabs::tab_panel_with_gate(
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
            }

            if force_mount_content {
                for (idx, item) in items.iter().enumerate() {
                    let active = active_idx.is_some_and(|a| a == idx);
                    let labelled_by_element = tab_trigger_elements
                        .get(idx)
                        .and_then(|cell| cell.get());
                    let label = item.label.clone();
                    let content = content_by_value
                        .remove(item.value.as_ref())
                        .unwrap_or_default();

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
                    gap: gap.into(),
                    padding: Edges::all(Px(0.0)).into(),
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
        AppWindowId, Color, Modifiers, MouseButton, Point, PointerType, Px, Rect, SemanticsRole,
        Size, SvgId, SvgService,
    };
    use fret_core::{PathCommand, PathConstraints, PathId, PathMetrics, PathService, PathStyle};
    use fret_core::{TextBlobId, TextConstraints, TextMetrics, TextService};
    use fret_runtime::{FrameId, TickId};
    use fret_ui::element::ColumnProps;
    use fret_ui::elements::{ElementRuntime, GlobalElementId, node_for_element};
    use fret_ui::tree::UiTree;
    use fret_ui_kit::ColorRef;

    fn contains_foreground_scope(el: &AnyElement) -> bool {
        matches!(el.kind, fret_ui::element::ElementKind::ForegroundScope(_))
            || el.children.iter().any(contains_foreground_scope)
    }

    fn find_first_inherited_foreground_node(el: &AnyElement) -> Option<&AnyElement> {
        if el.inherited_foreground.is_some() {
            return Some(el);
        }
        el.children
            .iter()
            .find_map(find_first_inherited_foreground_node)
    }

    fn find_pressable_element_with_test_id<'a>(
        el: &'a AnyElement,
        test_id: &str,
    ) -> Option<&'a AnyElement> {
        match &el.kind {
            fret_ui::element::ElementKind::Pressable(props) => {
                if props.a11y.test_id.as_deref() == Some(test_id) {
                    return Some(el);
                }
            }
            _ => {}
        }
        el.children
            .iter()
            .find_map(|child| find_pressable_element_with_test_id(child, test_id))
    }

    #[test]
    fn tabs_list_variants_match_upstream_default_and_line_intent() {
        let mut app = App::new();
        crate::shadcn_themes::apply_shadcn_new_york(
            &mut app,
            crate::shadcn_themes::ShadcnBaseColor::Slate,
            crate::shadcn_themes::ShadcnColorScheme::Light,
        );
        let theme = Theme::global(&app).snapshot();

        let default = tabs_list_variants(&theme, TabsListVariant::Default);
        assert!(default.padding_px.0 > 0.0);
        assert_eq!(default.trigger_row_gap_px, Px(0.0));
        match default.chrome.background {
            Some(ColorRef::Color(c)) => assert_ne!(c, Color::TRANSPARENT),
            other => panic!("expected default tabs list to have a background, got {other:?}"),
        }

        let line = tabs_list_variants(&theme, TabsListVariant::Line);
        assert!(line.padding_px.0 > 0.0);
        assert_eq!(line.trigger_row_gap_px, Px(4.0));
        match line.chrome.background {
            Some(ColorRef::Color(c)) => assert_eq!(c, Color::TRANSPARENT),
            other => panic!("expected line tabs list background to be transparent, got {other:?}"),
        }
    }

    #[test]
    fn tabs_content_defaults_to_flex_grow_fill_like_shadcn() {
        let window = AppWindowId::default();
        let mut app = App::new();
        crate::shadcn_themes::apply_shadcn_new_york(
            &mut app,
            crate::shadcn_themes::ShadcnBaseColor::Slate,
            crate::shadcn_themes::ShadcnColorScheme::Light,
        );

        let model = app.models_mut().insert(Some(Arc::from("alpha")));
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(400.0), Px(240.0)),
        );

        let el = fret_ui::elements::with_element_cx(&mut app, window, bounds, "tabs-fill", |cx| {
            Tabs::new(model.clone())
                .items([
                    TabsItem::new("alpha", "Alpha", [cx.text("Panel")]),
                    TabsItem::new("beta", "Beta", [cx.text("Panel")]),
                ])
                .into_element(cx)
        });

        fn find_tab_panel_semantics<'a>(
            el: &'a AnyElement,
        ) -> Option<&'a fret_ui::element::SemanticsProps> {
            match &el.kind {
                fret_ui::element::ElementKind::Semantics(props)
                    if props.role == SemanticsRole::TabPanel =>
                {
                    return Some(props);
                }
                _ => {}
            }
            el.children
                .iter()
                .find_map(|child| find_tab_panel_semantics(child))
        }

        let panel = find_tab_panel_semantics(&el).expect("expected TabsContent tabpanel semantics");
        assert_eq!(panel.layout.flex.grow, 1.0);
        assert_eq!(panel.layout.flex.shrink, 1.0);
        assert_eq!(panel.layout.size.width, Length::Fill);
        assert_eq!(panel.layout.size.min_width, Some(Length::Px(Px(0.0))));
    }

    #[test]
    fn tabs_trigger_content_attaches_foreground_without_wrapper() {
        let window = AppWindowId::default();
        let mut app = App::new();
        crate::shadcn_themes::apply_shadcn_new_york(
            &mut app,
            crate::shadcn_themes::ShadcnBaseColor::Slate,
            crate::shadcn_themes::ShadcnColorScheme::Light,
        );

        let model = app.models_mut().insert(Some(Arc::from("alpha")));
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(360.0), Px(200.0)),
        );

        let el = fret_ui::elements::with_element_cx(&mut app, window, bounds, "tabs-fg", |cx| {
            Tabs::new(model.clone())
                .items([
                    TabsItem::new("alpha", "Alpha", Vec::<AnyElement>::new())
                        .trigger_test_id("tabs.trigger.alpha")
                        .trigger_leading_icon(IconId::new_static("lucide.star"))
                        .trigger_trailing_icon(IconId::new_static("lucide.chevron-right")),
                    TabsItem::new("beta", "Beta", Vec::<AnyElement>::new()),
                ])
                .into_element(cx)
        });

        let pressable = find_pressable_element_with_test_id(&el, "tabs.trigger.alpha")
            .expect("tabs trigger pressable with test_id");
        let inherited = find_first_inherited_foreground_node(pressable)
            .expect("expected tabs trigger subtree to carry inherited foreground");

        assert!(matches!(
            inherited.kind,
            fret_ui::element::ElementKind::Flex(_)
        ));
        assert!(
            !contains_foreground_scope(pressable),
            "expected tabs trigger content to attach inherited foreground without inserting a ForegroundScope"
        );
    }

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

    #[test]
    fn tabs_vertical_orientation_does_not_clip_triggers() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let model = app.models_mut().insert(Some(Arc::from("preview")));
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
            "tabs-vertical-no-clip",
            |cx| {
                vec![
                    Tabs::new(model.clone())
                        .orientation(TabsOrientation::Vertical)
                        .items([
                            TabsItem::new("preview", "Preview", Vec::<AnyElement>::new()),
                            TabsItem::new("code", "Code", Vec::<AnyElement>::new()),
                        ])
                        .into_element(cx),
                ]
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
        let mut tabs: Vec<_> = snap
            .nodes
            .iter()
            .filter(|n| n.role == SemanticsRole::Tab)
            .collect();
        assert_eq!(tabs.len(), 2, "expected two tab triggers");

        tabs.sort_by(|a, b| {
            a.bounds
                .origin
                .y
                .0
                .partial_cmp(&b.bounds.origin.y.0)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        let list_top = tab_list.bounds.origin.y.0;
        let list_bottom = tab_list.bounds.origin.y.0 + tab_list.bounds.size.height.0;
        let eps = 0.51;
        for tab in tabs.iter() {
            let top = tab.bounds.origin.y.0;
            let bottom = tab.bounds.origin.y.0 + tab.bounds.size.height.0;
            assert!(
                top >= list_top - eps,
                "tab should be within tablist: tab_top={top:.3}, list_top={list_top:.3}"
            );
            assert!(
                bottom <= list_bottom + eps,
                "tab should be within tablist: tab_bottom={bottom:.3}, list_bottom={list_bottom:.3}"
            );
        }

        let first_top = tabs[0].bounds.origin.y.0;
        let second_top = tabs[1].bounds.origin.y.0;
        assert!(
            second_top > first_top + 1.0,
            "expected vertical stacking: first_top={first_top:.3}, second_top={second_top:.3}"
        );

        let w0 = tabs[0].bounds.size.width.0;
        let w1 = tabs[1].bounds.size.width.0;
        let wdiff = (w0 - w1).abs();
        assert!(
            wdiff <= 0.51,
            "expected vertical triggers to share list width: w0={w0:.3}, w1={w1:.3}, diff={wdiff:.3}"
        );
    }

    #[test]
    fn tabs_vertical_line_variant_stretches_triggers_to_shared_width() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let model = app.models_mut().insert(Some(Arc::from("preview")));
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(400.0), Px(240.0)),
        );
        let mut services = FakeServices::default();

        let mut render = || {
            let root = fret_ui::declarative::render_root(
                &mut ui,
                &mut app,
                &mut services,
                window,
                bounds,
                "tabs-vertical-line-stretch",
                |cx| {
                    vec![
                        Tabs::new(model.clone())
                            .orientation(TabsOrientation::Vertical)
                            .list_variant(TabsListVariant::Line)
                            .items([
                                TabsItem::new("preview", "Preview", Vec::<AnyElement>::new()),
                                TabsItem::new("code", "Code", Vec::<AnyElement>::new()),
                            ])
                            .into_element(cx),
                    ]
                },
            );
            ui.set_root(root);
            ui.request_semantics_snapshot();
            ui.layout_all(&mut app, &mut services, bounds, 1.0);
        };

        // Two frames so the vertical width stabilization can reuse the previous frame's measured
        // trigger widths.
        render();
        render();

        let snap = ui.semantics_snapshot().expect("semantics snapshot");
        let mut tabs: Vec<_> = snap
            .nodes
            .iter()
            .filter(|n| n.role == SemanticsRole::Tab)
            .collect();
        assert_eq!(tabs.len(), 2, "expected two tab triggers");

        tabs.sort_by(|a, b| {
            a.bounds
                .origin
                .y
                .0
                .partial_cmp(&b.bounds.origin.y.0)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        let w0 = tabs[0].bounds.size.width.0;
        let w1 = tabs[1].bounds.size.width.0;
        let wdiff = (w0 - w1).abs();
        assert!(
            wdiff <= 0.51,
            "expected vertical line variant triggers to share width: w0={w0:.3}, w1={w1:.3}, diff={wdiff:.3}"
        );
    }
    #[test]
    fn tabs_root_test_id_derives_trigger_test_ids() {
        fn find_pressable_with_test_id<'a>(
            el: &'a AnyElement,
            test_id: &str,
        ) -> Option<&'a PressableProps> {
            match &el.kind {
                fret_ui::element::ElementKind::Pressable(props) => {
                    if props.a11y.test_id.as_deref() == Some(test_id) {
                        return Some(props);
                    }
                }
                _ => {}
            }
            el.children
                .iter()
                .find_map(|child| find_pressable_with_test_id(child, test_id))
        }

        let window = AppWindowId::default();
        let mut app = App::new();
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            fret_core::Size::new(Px(320.0), Px(200.0)),
        );
        let model = app.models_mut().insert(Some(Arc::<str>::from("alpha")));

        let el = fret_ui::elements::with_element_cx(
            &mut app,
            window,
            bounds,
            "tabs-derived-trigger-id",
            |cx| {
                Tabs::new(model)
                    .test_id("tabs-demo")
                    .items([
                        TabsItem::new("alpha", "Alpha", Vec::<AnyElement>::new()),
                        TabsItem::new("beta", "Beta", Vec::<AnyElement>::new()),
                    ])
                    .into_element(cx)
            },
        );

        assert!(find_pressable_with_test_id(&el, "tabs-demo-trigger-alpha").is_some());
        assert!(find_pressable_with_test_id(&el, "tabs-demo-trigger-beta").is_some());
    }

    #[test]
    fn tabs_trigger_focus_ring_tweens_in_and_out_like_a_transition() {
        use std::cell::Cell;
        use std::rc::Rc;
        use std::time::Duration;

        use fret_core::KeyCode;
        use fret_ui_kit::declarative::transition::ticks_60hz_for_duration;

        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        crate::shadcn_themes::apply_shadcn_new_york(
            &mut app,
            crate::shadcn_themes::ShadcnBaseColor::Neutral,
            crate::shadcn_themes::ShadcnColorScheme::Light,
        );

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(320.0), Px(200.0)),
        );
        let mut services = FakeServices::default();
        let model = app.models_mut().insert(Some(Arc::from("alpha")));

        let ring_alpha_out: Rc<Cell<Option<f32>>> = Rc::new(Cell::new(None));
        let always_paint_out: Rc<Cell<Option<bool>>> = Rc::new(Cell::new(None));

        fn find_pressable_with_test_id<'a>(
            el: &'a AnyElement,
            test_id: &str,
        ) -> Option<&'a PressableProps> {
            match &el.kind {
                fret_ui::element::ElementKind::Pressable(props) => {
                    if props.a11y.test_id.as_deref() == Some(test_id) {
                        return Some(props);
                    }
                }
                _ => {}
            }
            el.children
                .iter()
                .find_map(|child| find_pressable_with_test_id(child, test_id))
        }

        fn render_frame(
            ui: &mut UiTree<App>,
            app: &mut App,
            services: &mut dyn fret_core::UiServices,
            window: AppWindowId,
            bounds: Rect,
            model: Model<Option<Arc<str>>>,
            ring_alpha_out: Rc<Cell<Option<f32>>>,
            always_paint_out: Rc<Cell<Option<bool>>>,
        ) -> fret_core::NodeId {
            let root = fret_ui::declarative::render_root(
                ui,
                app,
                services,
                window,
                bounds,
                "tabs-trigger-focus-ring-tween",
                move |cx| {
                    let el = Tabs::new(model)
                        .items([
                            TabsItem::new("alpha", "Alpha", Vec::<AnyElement>::new())
                                .trigger_test_id("tabs.trigger.alpha"),
                            TabsItem::new("beta", "Beta", Vec::<AnyElement>::new()),
                        ])
                        .into_element(cx);

                    let pressable = find_pressable_with_test_id(&el, "tabs.trigger.alpha")
                        .expect("pressable with trigger test_id");
                    let ring = pressable.focus_ring.expect("focus ring");
                    ring_alpha_out.set(Some(ring.color.a));
                    always_paint_out.set(Some(pressable.focus_ring_always_paint));

                    vec![el]
                },
            );
            ui.set_root(root);
            ui.request_semantics_snapshot();
            ui.layout_all(app, services, bounds, 1.0);
            root
        }

        // Frame 1: baseline render (no focus-visible), ring alpha should be 0.
        app.set_frame_id(FrameId(1));
        let root = render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            model.clone(),
            ring_alpha_out.clone(),
            always_paint_out.clone(),
        );
        let a0 = ring_alpha_out.get().expect("a0");
        assert!(
            a0.abs() <= 1e-6,
            "expected ring alpha to start at 0, got {a0}"
        );

        // Focus the trigger and mark focus-visible via a navigation key.
        let focusable = ui
            .first_focusable_descendant_including_declarative(&mut app, window, root)
            .expect("focusable tab trigger");
        ui.set_focus(Some(focusable));
        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::KeyDown {
                key: KeyCode::Tab,
                modifiers: Modifiers::default(),
                repeat: false,
            },
        );

        // Frame 2: ring should be in-between (not snapped).
        app.set_frame_id(FrameId(2));
        let _ = render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            model.clone(),
            ring_alpha_out.clone(),
            always_paint_out.clone(),
        );
        let a1 = ring_alpha_out.get().expect("a1");
        assert!(
            a1 > 0.0,
            "expected ring alpha to start animating in, got {a1}"
        );

        // Advance frames until the default 150ms transition settles.
        let settle = ticks_60hz_for_duration(Duration::from_millis(150)) + 2;
        for i in 0..settle {
            app.set_frame_id(FrameId(3 + i));
            let _ = render_frame(
                &mut ui,
                &mut app,
                &mut services,
                window,
                bounds,
                model.clone(),
                ring_alpha_out.clone(),
                always_paint_out.clone(),
            );
        }
        let a_focused = ring_alpha_out.get().expect("a_focused");
        assert!(
            a_focused > a1 + 1e-4,
            "expected ring alpha to increase over time, got a1={a1} a_focused={a_focused}"
        );

        // Blur and ensure ring animates out while still being painted.
        ui.set_focus(None);
        app.set_frame_id(FrameId(1000));
        let _ = render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            model.clone(),
            ring_alpha_out.clone(),
            always_paint_out.clone(),
        );
        let a_blur = ring_alpha_out.get().expect("a_blur");
        let always_paint = always_paint_out.get().expect("always_paint");
        assert!(
            a_blur > 0.0 && a_blur < a_focused,
            "expected ring alpha to be intermediate after blur, got a_blur={a_blur} a_focused={a_focused}"
        );
        assert!(
            always_paint,
            "expected focus ring to request painting while animating out"
        );

        for i in 0..settle {
            app.set_frame_id(FrameId(1001 + i));
            let _ = render_frame(
                &mut ui,
                &mut app,
                &mut services,
                window,
                bounds,
                model.clone(),
                ring_alpha_out.clone(),
                always_paint_out.clone(),
            );
        }
        let a_final = ring_alpha_out.get().expect("a_final");
        let always_paint_final = always_paint_out.get().expect("always_paint_final");
        assert!(
            a_final.abs() <= 1e-4,
            "expected ring alpha to settle at 0, got {a_final}"
        );
        assert!(
            !always_paint_final,
            "expected focus ring to stop requesting painting after settling"
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
                page.padding = Edges::all(Px(16.0)).into();

                vec![cx.container(page, |cx| {
                    let items = vec![
                        TabsItem::new("alpha", "Alpha", vec![cx.text("Panel")]),
                        TabsItem::new("beta", "Beta", vec![cx.text("Panel")]),
                        TabsItem::new("gamma", "Gamma", vec![cx.text("Panel")]),
                    ];

                    let mut col = ColumnProps::default();
                    col.layout.size.width = Length::Fill;
                    col.layout.size.height = Length::Auto;
                    col.gap = Px(16.0).into();

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

    fn render_presence_motion_frame(
        ui: &mut UiTree<App>,
        app: &mut App,
        services: &mut dyn fret_core::UiServices,
        window: AppWindowId,
        bounds: Rect,
        model: Model<Option<Arc<str>>>,
        alpha_content_id_out: &Cell<Option<GlobalElementId>>,
        beta_content_id_out: &Cell<Option<GlobalElementId>>,
    ) {
        let root = fret_ui::declarative::render_root(
            ui,
            app,
            services,
            window,
            bounds,
            "tabs-presence-motion",
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
                let beta_content = cx.pressable_with_id(
                    PressableProps {
                        layout: LayoutStyle::default(),
                        enabled: true,
                        focusable: true,
                        ..Default::default()
                    },
                    |_cx, _st, id| {
                        beta_content_id_out.set(Some(id));
                        Vec::new()
                    },
                );

                let items = vec![
                    TabsItem::new("alpha", "Alpha", vec![alpha_content]),
                    TabsItem::new("beta", "Beta", vec![beta_content]),
                ];

                vec![
                    Tabs::new(model)
                        .content_presence_motion(true)
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
    fn tabs_presence_motion_keeps_outgoing_panel_until_transition_settles() {
        use std::time::Duration;

        use fret_ui_kit::declarative::transition::ticks_60hz_for_duration;

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
        let beta_content_id: Cell<Option<GlobalElementId>> = Cell::new(None);

        bump_frame(&mut app);
        render_presence_motion_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            model.clone(),
            &alpha_content_id,
            &beta_content_id,
        );

        let alpha_content_id = alpha_content_id.get().expect("alpha content id");
        assert!(node_for_element(&mut app, window, alpha_content_id).is_some());

        let _ = app
            .models_mut()
            .update(&model, |v| *v = Some(Arc::from("beta")));

        let alpha_after_switch = Cell::new(Some(alpha_content_id));
        let beta_after_switch = Cell::new(None);

        bump_frame(&mut app);
        render_presence_motion_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            model.clone(),
            &alpha_after_switch,
            &beta_after_switch,
        );

        let beta_content_id = beta_after_switch.get().expect("beta content id");
        assert!(
            node_for_element(&mut app, window, alpha_content_id).is_some(),
            "outgoing panel should remain mounted during the exit transition"
        );
        assert!(
            node_for_element(&mut app, window, beta_content_id).is_some(),
            "incoming panel should mount immediately when selection changes"
        );

        let settle = ticks_60hz_for_duration(Duration::from_millis(150)) + 2;
        for _ in 0..settle {
            bump_frame(&mut app);
            render_presence_motion_frame(
                &mut ui,
                &mut app,
                &mut services,
                window,
                bounds,
                model.clone(),
                &Cell::new(Some(alpha_content_id)),
                &Cell::new(Some(beta_content_id)),
            );
        }

        assert!(
            node_for_element(&mut app, window, alpha_content_id).is_none(),
            "outgoing panel should be pruned after the exit transition settles"
        );
        assert!(node_for_element(&mut app, window, beta_content_id).is_some());
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
