//! Material 3 tooltip.
//!
//! This module owns the Material tooltip recipe:
//! - floating placement via `fret-ui-kit` popper helpers
//! - Radix-aligned open delay + safe-hover corridor policies via `fret-ui-kit` tooltip primitives
//! - token-driven plain/rich tooltip container, text, accessibility, and motion outcomes

use std::sync::Arc;

use fret_core::{
    Axis, Color, Corners, Edges, KeyCode, PointerType, Px, Rect, SemanticsLive, Size, TextOverflow,
    TextStyle, TextWrap,
};
use fret_ui::element::{
    AnyElement, ContainerProps, ElementKind, Elements, FlexProps, HoverRegionProps, LayoutStyle,
    Length, PointerRegionProps, SemanticsProps, SpinnerProps, SvgIconProps, TextProps,
};
use fret_ui::overlay_placement::{Align, Side};
use fret_ui::{ElementContext, Theme, UiHost};
use fret_ui_kit::declarative::ModelWatchExt;
use fret_ui_kit::declarative::scheduling;
use fret_ui_kit::primitives::dismissable_layer as dismissable_layer_prim;
use fret_ui_kit::primitives::popper;
use fret_ui_kit::primitives::popper_content;
use fret_ui_kit::primitives::tooltip as tooltip_prim;
use fret_ui_kit::tooltip_provider;
use fret_ui_kit::{
    ColorRef, OverlayPresence, OverrideSlot, WidgetStateProperty, WidgetStates,
    merge_override_slot, resolve_override_slot_with,
};

use crate::foundation::context::material_layout_direction_in_scope;
use crate::foundation::overlay_motion::drive_overlay_open_close_motion;
use crate::foundation::surface::material_surface_style;
use crate::foundation::test_id::part_test_id;
use crate::motion::ms_to_frames;
use crate::tokens::tooltip as tooltip_tokens;

fn tooltip_part_id(test_id: &Option<Arc<str>>, part: &str) -> Option<Arc<str>> {
    test_id.as_ref().map(|id| part_test_id(id, part))
}

fn with_optional_test_id(mut element: AnyElement, test_id: Option<Arc<str>>) -> AnyElement {
    if let Some(test_id) = test_id {
        element = element.test_id(test_id);
    }
    element
}

fn tooltip_color_override(
    theme: &Theme,
    slot: &OverrideSlot<ColorRef>,
    states: WidgetStates,
    fallback: impl FnOnce() -> Color,
) -> Color {
    resolve_override_slot_with(
        slot.as_ref(),
        states,
        |color| color.resolve(theme),
        fallback,
    )
}

fn tooltip_metric_override(
    slot: &OverrideSlot<Px>,
    states: WidgetStates,
    fallback: impl FnOnce() -> Px,
) -> Px {
    resolve_override_slot_with(slot.as_ref(), states, |value| *value, fallback)
}

fn tooltip_edges_override(
    slot: &OverrideSlot<Edges>,
    states: WidgetStates,
    fallback: impl FnOnce() -> Edges,
) -> Edges {
    resolve_override_slot_with(slot.as_ref(), states, |value| *value, fallback)
}

fn tooltip_text_style_override(
    slot: &OverrideSlot<TextStyle>,
    states: WidgetStates,
    fallback: impl FnOnce() -> TextStyle,
) -> TextStyle {
    resolve_override_slot_with(slot.as_ref(), states, |style| style.clone(), fallback)
}

fn tooltip_content_root<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    test_id: Option<Arc<str>>,
    chrome: AnyElement,
) -> AnyElement {
    cx.semantics(
        SemanticsProps {
            role: fret_core::SemanticsRole::Tooltip,
            live: Some(SemanticsLive::Assertive),
            test_id,
            ..Default::default()
        },
        move |_cx| vec![chrome],
    )
}

fn apply_tooltip_inherited_fg(mut element: AnyElement, fg: Color) -> AnyElement {
    match &mut element.kind {
        ElementKind::Text(props) => {
            if props.color.is_none() {
                props.color = Some(fg);
            }
        }
        ElementKind::SvgIcon(SvgIconProps { color, .. }) => {
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
        ElementKind::Spinner(SpinnerProps { color, .. }) => {
            color.get_or_insert(fg);
        }
        _ => {}
    }

    element.children = element
        .children
        .into_iter()
        .map(|child| apply_tooltip_inherited_fg(child, fg))
        .collect();
    element
}

#[track_caller]
fn stabilize_popper_desired_size<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    desired: Size,
    scale_factor: f32,
) -> Size {
    #[derive(Default)]
    struct State {
        last: Option<Size>,
    }

    let eps = if scale_factor.is_finite() && scale_factor > 0.0 {
        // One physical pixel in logical px.
        Px(1.0 / scale_factor)
    } else {
        // Default to half a logical px if scale factor is unavailable.
        Px(0.5)
    };

    cx.slot_state(State::default, |st| {
        let next = match st.last {
            None => desired,
            Some(prev) => {
                let dw = (prev.width.0 - desired.width.0).abs();
                let dh = (prev.height.0 - desired.height.0).abs();
                if dw <= eps.0 && dh <= eps.0 {
                    prev
                } else {
                    desired
                }
            }
        };
        st.last = Some(next);
        next
    })
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum TooltipAlign {
    Start,
    #[default]
    Center,
    End,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum TooltipSide {
    #[default]
    Top,
    Right,
    Bottom,
    Left,
}

/// Material-like provider defaults for tooltip delay-group policy.
///
/// This mirrors Radix `TooltipProvider` behavior (delay-group), but the default timings are tuned
/// for desktop hover tooltips.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TooltipProvider {
    delay_duration_frames: u32,
    skip_delay_duration_frames: u32,
    disable_hoverable_content: bool,
}

impl Default for TooltipProvider {
    fn default() -> Self {
        Self {
            delay_duration_frames: 30,
            skip_delay_duration_frames: 6,
            disable_hoverable_content: false,
        }
    }
}

impl TooltipProvider {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn delay_duration_frames(mut self, frames: u32) -> Self {
        self.delay_duration_frames = frames;
        self
    }

    pub fn skip_delay_duration_frames(mut self, frames: u32) -> Self {
        self.skip_delay_duration_frames = frames;
        self
    }

    pub fn disable_hoverable_content(mut self, disable: bool) -> Self {
        self.disable_hoverable_content = disable;
        self
    }

    pub fn with_elements<H: UiHost, I>(
        self,
        cx: &mut ElementContext<'_, H>,
        f: impl FnOnce(&mut ElementContext<'_, H>) -> I,
    ) -> Elements
    where
        I: IntoIterator<Item = AnyElement>,
    {
        tooltip_provider::with_tooltip_provider(
            cx,
            tooltip_provider::TooltipProviderConfig::new(
                self.delay_duration_frames as u64,
                self.skip_delay_duration_frames as u64,
            )
            .disable_hoverable_content(self.disable_hoverable_content),
            |cx| f(cx).into_iter().collect::<Elements>(),
        )
    }
}

#[derive(Debug, Clone, Default)]
pub struct TooltipStyle {
    pub plain_container_background: OverrideSlot<ColorRef>,
    pub plain_supporting_text_color: OverrideSlot<ColorRef>,
    pub plain_supporting_text_style: OverrideSlot<TextStyle>,
    pub plain_container_corner_radius: OverrideSlot<Px>,
    pub plain_container_padding: OverrideSlot<Edges>,
    pub plain_container_max_width: OverrideSlot<Px>,
    pub rich_container_background: OverrideSlot<ColorRef>,
    pub rich_container_elevation: OverrideSlot<Px>,
    pub rich_container_shadow_color: OverrideSlot<ColorRef>,
    pub rich_title_color: OverrideSlot<ColorRef>,
    pub rich_supporting_text_color: OverrideSlot<ColorRef>,
    pub rich_title_text_style: OverrideSlot<TextStyle>,
    pub rich_supporting_text_style: OverrideSlot<TextStyle>,
    pub rich_container_corner_radius: OverrideSlot<Px>,
    pub rich_container_padding: OverrideSlot<Edges>,
    pub rich_container_max_width: OverrideSlot<Px>,
    pub rich_text_gap: OverrideSlot<Px>,
    pub container_min_width: OverrideSlot<Px>,
    pub container_min_height: OverrideSlot<Px>,
}

impl TooltipStyle {
    pub fn plain_container_background(
        mut self,
        color: WidgetStateProperty<Option<ColorRef>>,
    ) -> Self {
        self.plain_container_background = Some(color);
        self
    }

    pub fn plain_supporting_text_color(
        mut self,
        color: WidgetStateProperty<Option<ColorRef>>,
    ) -> Self {
        self.plain_supporting_text_color = Some(color);
        self
    }

    pub fn plain_supporting_text_style(
        mut self,
        style: WidgetStateProperty<Option<TextStyle>>,
    ) -> Self {
        self.plain_supporting_text_style = Some(style);
        self
    }

    pub fn plain_container_corner_radius(
        mut self,
        radius: WidgetStateProperty<Option<Px>>,
    ) -> Self {
        self.plain_container_corner_radius = Some(radius);
        self
    }

    pub fn plain_container_padding(mut self, padding: WidgetStateProperty<Option<Edges>>) -> Self {
        self.plain_container_padding = Some(padding);
        self
    }

    pub fn plain_container_max_width(mut self, width: WidgetStateProperty<Option<Px>>) -> Self {
        self.plain_container_max_width = Some(width);
        self
    }

    pub fn rich_container_background(
        mut self,
        color: WidgetStateProperty<Option<ColorRef>>,
    ) -> Self {
        self.rich_container_background = Some(color);
        self
    }

    pub fn rich_container_elevation(mut self, elevation: WidgetStateProperty<Option<Px>>) -> Self {
        self.rich_container_elevation = Some(elevation);
        self
    }

    pub fn rich_container_shadow_color(
        mut self,
        color: WidgetStateProperty<Option<ColorRef>>,
    ) -> Self {
        self.rich_container_shadow_color = Some(color);
        self
    }

    pub fn rich_title_color(mut self, color: WidgetStateProperty<Option<ColorRef>>) -> Self {
        self.rich_title_color = Some(color);
        self
    }

    pub fn rich_supporting_text_color(
        mut self,
        color: WidgetStateProperty<Option<ColorRef>>,
    ) -> Self {
        self.rich_supporting_text_color = Some(color);
        self
    }

    pub fn rich_title_text_style(mut self, style: WidgetStateProperty<Option<TextStyle>>) -> Self {
        self.rich_title_text_style = Some(style);
        self
    }

    pub fn rich_supporting_text_style(
        mut self,
        style: WidgetStateProperty<Option<TextStyle>>,
    ) -> Self {
        self.rich_supporting_text_style = Some(style);
        self
    }

    pub fn rich_container_corner_radius(mut self, radius: WidgetStateProperty<Option<Px>>) -> Self {
        self.rich_container_corner_radius = Some(radius);
        self
    }

    pub fn rich_container_padding(mut self, padding: WidgetStateProperty<Option<Edges>>) -> Self {
        self.rich_container_padding = Some(padding);
        self
    }

    pub fn rich_container_max_width(mut self, width: WidgetStateProperty<Option<Px>>) -> Self {
        self.rich_container_max_width = Some(width);
        self
    }

    pub fn rich_text_gap(mut self, gap: WidgetStateProperty<Option<Px>>) -> Self {
        self.rich_text_gap = Some(gap);
        self
    }

    pub fn container_min_width(mut self, width: WidgetStateProperty<Option<Px>>) -> Self {
        self.container_min_width = Some(width);
        self
    }

    pub fn container_min_height(mut self, height: WidgetStateProperty<Option<Px>>) -> Self {
        self.container_min_height = Some(height);
        self
    }

    pub fn merged(self, other: Self) -> Self {
        Self {
            plain_container_background: merge_override_slot(
                self.plain_container_background,
                other.plain_container_background,
            ),
            plain_supporting_text_color: merge_override_slot(
                self.plain_supporting_text_color,
                other.plain_supporting_text_color,
            ),
            plain_supporting_text_style: merge_override_slot(
                self.plain_supporting_text_style,
                other.plain_supporting_text_style,
            ),
            plain_container_corner_radius: merge_override_slot(
                self.plain_container_corner_radius,
                other.plain_container_corner_radius,
            ),
            plain_container_padding: merge_override_slot(
                self.plain_container_padding,
                other.plain_container_padding,
            ),
            plain_container_max_width: merge_override_slot(
                self.plain_container_max_width,
                other.plain_container_max_width,
            ),
            rich_container_background: merge_override_slot(
                self.rich_container_background,
                other.rich_container_background,
            ),
            rich_container_elevation: merge_override_slot(
                self.rich_container_elevation,
                other.rich_container_elevation,
            ),
            rich_container_shadow_color: merge_override_slot(
                self.rich_container_shadow_color,
                other.rich_container_shadow_color,
            ),
            rich_title_color: merge_override_slot(self.rich_title_color, other.rich_title_color),
            rich_supporting_text_color: merge_override_slot(
                self.rich_supporting_text_color,
                other.rich_supporting_text_color,
            ),
            rich_title_text_style: merge_override_slot(
                self.rich_title_text_style,
                other.rich_title_text_style,
            ),
            rich_supporting_text_style: merge_override_slot(
                self.rich_supporting_text_style,
                other.rich_supporting_text_style,
            ),
            rich_container_corner_radius: merge_override_slot(
                self.rich_container_corner_radius,
                other.rich_container_corner_radius,
            ),
            rich_container_padding: merge_override_slot(
                self.rich_container_padding,
                other.rich_container_padding,
            ),
            rich_container_max_width: merge_override_slot(
                self.rich_container_max_width,
                other.rich_container_max_width,
            ),
            rich_text_gap: merge_override_slot(self.rich_text_gap, other.rich_text_gap),
            container_min_width: merge_override_slot(
                self.container_min_width,
                other.container_min_width,
            ),
            container_min_height: merge_override_slot(
                self.container_min_height,
                other.container_min_height,
            ),
        }
    }
}

enum PlainTooltipContent {
    Text(Arc<str>),
    Element(AnyElement),
}

#[derive(Clone)]
struct TooltipTriggerEventModels {
    has_pointer_move_opened: fret_runtime::Model<bool>,
    pointer_transit_geometry: fret_runtime::Model<Option<(Rect, Rect)>>,
    suppress_hover_open: fret_runtime::Model<bool>,
    suppress_focus_open: fret_runtime::Model<bool>,
    close_requested: fret_runtime::Model<bool>,
    open: fret_runtime::Model<bool>,
}

#[track_caller]
fn tooltip_trigger_event_models<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
) -> TooltipTriggerEventModels {
    TooltipTriggerEventModels {
        has_pointer_move_opened: cx.local_model(|| false),
        pointer_transit_geometry: tooltip_provider::pointer_transit_geometry_model(cx),
        suppress_hover_open: cx.local_model(|| false),
        suppress_focus_open: cx.local_model(|| false),
        close_requested: cx.local_model(|| false),
        open: cx.local_model(|| false),
    }
}

#[derive(Debug, Default, Clone, Copy)]
struct TooltipTriggerHoverEdgeState {
    was_hovered: bool,
}

fn tooltip_policy_root<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    base_trigger: AnyElement,
    trigger_id: fret_ui::elements::GlobalElementId,
    anchor_id: fret_ui::elements::GlobalElementId,
    content: AnyElement,
    align: TooltipAlign,
    side: TooltipSide,
    side_offset: Px,
    window_margin: Px,
    hide_when_detached: bool,
    open_delay_frames_override: Option<u32>,
    close_delay_frames_override: Option<u32>,
    disable_hoverable_content_override: Option<bool>,
) -> AnyElement {
    let content_id = content.id;

    cx.hover_region(HoverRegionProps::default(), move |cx, hovered| {
        let focused = cx.is_focused_element(trigger_id);
        let event_models = tooltip_trigger_event_models(cx);

        let close_requested = cx
            .watch_model(&event_models.close_requested)
            .layout()
            .copied()
            .unwrap_or(false);
        let has_pointer_move_opened = cx
            .watch_model(&event_models.has_pointer_move_opened)
            .layout()
            .copied()
            .unwrap_or(false);
        let suppress_hover_open = cx
            .watch_model(&event_models.suppress_hover_open)
            .layout()
            .copied()
            .unwrap_or(false);
        let suppress_focus_open = cx
            .watch_model(&event_models.suppress_focus_open)
            .layout()
            .copied()
            .unwrap_or(false);

        let left_hover = cx.slot_state(TooltipTriggerHoverEdgeState::default, |st| {
            let left = st.was_hovered && !hovered;
            st.was_hovered = hovered;
            left
        });

        if left_hover && (has_pointer_move_opened || suppress_hover_open) {
            let _ = cx
                .app
                .models_mut()
                .update(&event_models.has_pointer_move_opened, |v| *v = false);
            let _ = cx
                .app
                .models_mut()
                .update(&event_models.suppress_hover_open, |v| *v = false);
        }

        if !focused && suppress_focus_open {
            let _ = cx
                .app
                .models_mut()
                .update(&event_models.suppress_focus_open, |v| *v = false);
        }

        if close_requested {
            if has_pointer_move_opened && !suppress_hover_open {
                let _ = cx
                    .app
                    .models_mut()
                    .update(&event_models.suppress_hover_open, |v| *v = true);
            }
            if focused && !suppress_focus_open {
                let _ = cx
                    .app
                    .models_mut()
                    .update(&event_models.suppress_focus_open, |v| *v = true);
            }
            let _ = cx
                .app
                .models_mut()
                .update(&event_models.close_requested, |v| *v = false);
        }

        let provider_cfg = tooltip_provider::current_config(cx);
        let disable_hoverable_content =
            disable_hoverable_content_override.unwrap_or(provider_cfg.disable_hoverable_content);
        let last_pointer = tooltip_prim::tooltip_last_pointer_model(cx);

        let primary_can_hover = fret_ui_kit::declarative::primary_pointer_can_hover(
            cx,
            fret_ui::Invalidation::Layout,
            true,
        );
        let trigger_hovered =
            primary_can_hover && hovered && has_pointer_move_opened && !suppress_hover_open;
        let trigger_focused = focused && !suppress_focus_open;

        let anchor_bounds = fret_ui_kit::overlay::anchor_bounds_for_element(cx, anchor_id);
        let floating_bounds = anchor_bounds.map(|anchor| {
            let last_content_size = cx.last_bounds_for_element(content_id).map(|r| r.size);
            let estimated_size = Size::new(Px(240.0), Px(32.0));
            let content_size = last_content_size.unwrap_or(estimated_size);

            let outer = fret_ui_kit::overlay::outer_bounds_with_window_margin_for_environment(
                cx,
                fret_ui::Invalidation::Layout,
                window_margin,
            );

            let align = match align {
                TooltipAlign::Start => Align::Start,
                TooltipAlign::Center => Align::Center,
                TooltipAlign::End => Align::End,
            };
            let side = match side {
                TooltipSide::Top => Side::Top,
                TooltipSide::Right => Side::Right,
                TooltipSide::Bottom => Side::Bottom,
                TooltipSide::Left => Side::Left,
            };

            let direction = material_layout_direction_in_scope(cx);
            let layout = popper::popper_content_layout_sized(
                outer,
                anchor,
                content_size,
                popper::PopperContentPlacement::new(direction, side, align, side_offset)
                    .with_shift_cross_axis(true),
            );

            layout.rect
        });

        let update = tooltip_prim::tooltip_update_interaction(
            cx,
            trigger_hovered,
            trigger_focused,
            close_requested,
            last_pointer.clone(),
            anchor_bounds,
            floating_bounds,
            tooltip_prim::TooltipInteractionConfig {
                disable_hoverable_content,
                open_delay_ticks_override: open_delay_frames_override.map(|v| v as u64),
                close_delay_ticks_override: close_delay_frames_override.map(|v| v as u64),
                safe_hover_buffer: Px(5.0),
            },
        );

        scheduling::set_continuous_frames(cx, update.wants_continuous_ticks);

        let open = event_models.open.clone();
        let open_now = cx.watch_model(&open).layout().copied().unwrap_or(false);
        if update.open != open_now {
            let _ = cx.app.models_mut().update(&open, |v| *v = update.open);
        }

        let trigger =
            tooltip_prim::apply_tooltip_trigger_a11y(base_trigger, update.open, content_id);

        cx.pressable_add_on_pointer_down_for(
            trigger_id,
            Arc::new({
                let close_requested = event_models.close_requested.clone();
                let suppress_focus_open = event_models.suppress_focus_open.clone();
                let has_pointer_move_opened = event_models.has_pointer_move_opened.clone();
                let suppress_hover_open = event_models.suppress_hover_open.clone();
                move |host, acx, down| {
                    if down.pointer_type != PointerType::Touch {
                        let _ = host.models_mut().update(&close_requested, |v| *v = true);
                    }
                    let _ = host
                        .models_mut()
                        .update(&suppress_focus_open, |v| *v = true);
                    let gate = host
                        .models_mut()
                        .read(&has_pointer_move_opened, |v| *v)
                        .ok()
                        .unwrap_or(false);
                    if gate {
                        let _ = host
                            .models_mut()
                            .update(&suppress_hover_open, |v| *v = true);
                    }
                    host.request_redraw(acx.window);
                    fret_ui::action::PressablePointerDownResult::Continue
                }
            }),
        );

        cx.pressable_add_on_activate_for(
            trigger_id,
            Arc::new({
                let close_requested = event_models.close_requested.clone();
                let suppress_focus_open = event_models.suppress_focus_open.clone();
                move |host, acx, _reason| {
                    let _ = host.models_mut().update(&close_requested, |v| *v = true);
                    let _ = host
                        .models_mut()
                        .update(&suppress_focus_open, |v| *v = true);
                    host.request_redraw(acx.window);
                }
            }),
        );

        cx.key_add_on_key_down_for(
            trigger_id,
            Arc::new({
                let close_requested = event_models.close_requested.clone();
                let suppress_focus_open = event_models.suppress_focus_open.clone();
                move |host, acx, down| {
                    if down.repeat || down.key != KeyCode::Escape {
                        return false;
                    }
                    let _ = host.models_mut().update(&close_requested, |v| *v = true);
                    let _ = host
                        .models_mut()
                        .update(&suppress_focus_open, |v| *v = true);
                    host.request_redraw(acx.window);
                    true
                }
            }),
        );

        let trigger = cx.pointer_region(PointerRegionProps::default(), move |cx| {
            cx.pointer_region_on_pointer_move(Arc::new({
                let has_pointer_move_opened = event_models.has_pointer_move_opened.clone();
                let pointer_transit_geometry = event_models.pointer_transit_geometry.clone();
                move |host, acx, mv| {
                    if mv.pointer_type == PointerType::Touch {
                        return false;
                    }

                    let geometry = host
                        .models_mut()
                        .read(&pointer_transit_geometry, |v| *v)
                        .ok()
                        .flatten();
                    if let Some((anchor, floating)) = geometry
                        && tooltip_prim::tooltip_pointer_in_transit(
                            mv.position,
                            anchor,
                            floating,
                            Px(5.0),
                        )
                    {
                        return false;
                    }

                    let already = host
                        .models_mut()
                        .read(&has_pointer_move_opened, |v| *v)
                        .ok()
                        .unwrap_or(false);
                    if !already {
                        let _ = host.models_mut().update(&has_pointer_move_opened, |v| {
                            *v = true;
                        });
                        host.request_redraw(acx.window);
                    }

                    false
                }
            }));

            vec![trigger]
        });

        let close_grace_frames = {
            let close_ms = {
                let theme = Theme::global(&*cx.app);
                tooltip_tokens::close_duration_ms(theme)
            };
            Some(ms_to_frames(close_ms))
        };
        let motion = drive_overlay_open_close_motion(cx, update.open, close_grace_frames);

        let overlay_presence = OverlayPresence {
            present: motion.present,
            interactive: update.open,
        };

        let out = vec![trigger];
        if !overlay_presence.present {
            return out;
        }

        let tooltip_id = cx.root_id();
        let overlay_root_name = tooltip_prim::tooltip_root_name(tooltip_id);
        let opacity = motion.alpha;
        let scale = motion.scale;
        let direction = material_layout_direction_in_scope(cx);

        let overlay_children = cx.with_root_name(&overlay_root_name, move |cx| {
            cx.provide(direction, |cx| {
                let anchor = fret_ui_kit::overlay::anchor_bounds_for_element(cx, anchor_id);
                let Some(anchor) = anchor else {
                    return Vec::new();
                };

                let scale_factor = cx.environment_scale_factor(fret_ui::Invalidation::Layout);
                let last_content_size = cx.last_bounds_for_element(content_id).map(|r| r.size);
                let estimated_size = Size::new(Px(240.0), Px(32.0));
                let content_size = stabilize_popper_desired_size(
                    cx,
                    last_content_size.unwrap_or(estimated_size),
                    scale_factor,
                );

                let outer = fret_ui_kit::overlay::outer_bounds_with_window_margin_for_environment(
                    cx,
                    fret_ui::Invalidation::Layout,
                    window_margin,
                );

                let align = match align {
                    TooltipAlign::Start => Align::Start,
                    TooltipAlign::Center => Align::Center,
                    TooltipAlign::End => Align::End,
                };
                let side = match side {
                    TooltipSide::Top => Side::Top,
                    TooltipSide::Right => Side::Right,
                    TooltipSide::Bottom => Side::Bottom,
                    TooltipSide::Left => Side::Left,
                };

                let placement =
                    popper::PopperContentPlacement::new(direction, side, align, side_offset)
                        .with_shift_cross_axis(true)
                        .with_hide_when_detached(hide_when_detached);
                let reference_hidden = placement.reference_hidden(outer, anchor);

                let layout =
                    popper::popper_content_layout_sized(outer, anchor, content_size, placement);
                let placed = layout.rect;

                let wrapper = popper_content::popper_wrapper_panel_at(
                    cx,
                    placed,
                    Edges::all(Px(0.0)),
                    fret_ui::element::Overflow::Visible,
                    move |_cx| vec![content],
                );

                let origin = popper::popper_content_transform_origin(&layout, anchor, None);
                let origin_inv = fret_core::Point::new(Px(-origin.x.0), Px(-origin.y.0));
                let transform = fret_core::Transform2D::translation(origin)
                    * fret_core::Transform2D::scale_uniform(scale)
                    * fret_core::Transform2D::translation(origin_inv);

                let opacity = if reference_hidden { 0.0 } else { opacity };
                vec![
                    fret_ui_kit::declarative::overlay_motion::wrap_opacity_and_render_transform_gated(
                        cx,
                        opacity,
                        transform,
                        !reference_hidden,
                        vec![wrapper],
                    ),
                ]
            })
        });

        let mut request =
            tooltip_prim::tooltip_request(tooltip_id, open, overlay_presence, overlay_children);
        request.trigger = Some(trigger_id);
        request.dismissible_on_dismiss_request = Some(dismissable_layer_prim::handler({
            let close_requested = event_models.close_requested.clone();
            move |host, acx, _reason| {
                let _ = host.models_mut().update(&close_requested, |v| *v = true);
                host.request_redraw(acx.window);
            }
        }));
        if !disable_hoverable_content {
            tooltip_prim::tooltip_install_pointer_move_tracker(&mut request, last_pointer);
        }
        tooltip_prim::request_tooltip(cx, request);

        out
    })
}

/// Material 3 Plain Tooltip (MVP).
///
/// This is a policy wrapper built on `fret-ui-kit` tooltip primitives.
pub struct PlainTooltip {
    trigger: AnyElement,
    content: PlainTooltipContent,
    style: TooltipStyle,
    align: TooltipAlign,
    side: TooltipSide,
    side_offset: Px,
    window_margin: Px,
    hide_when_detached: bool,
    anchor_override: Option<fret_ui::elements::GlobalElementId>,
    open_delay_frames_override: Option<u32>,
    close_delay_frames_override: Option<u32>,
    disable_hoverable_content_override: Option<bool>,
    test_id: Option<Arc<str>>,
}

impl std::fmt::Debug for PlainTooltip {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PlainTooltip")
            .field("trigger_id", &self.trigger.id)
            .field("style", &self.style)
            .field("align", &self.align)
            .field("side", &self.side)
            .field("side_offset", &self.side_offset)
            .field("window_margin", &self.window_margin)
            .field("hide_when_detached", &self.hide_when_detached)
            .field("anchor_override", &self.anchor_override)
            .field(
                "open_delay_frames_override",
                &self.open_delay_frames_override,
            )
            .field(
                "close_delay_frames_override",
                &self.close_delay_frames_override,
            )
            .field(
                "disable_hoverable_content_override",
                &self.disable_hoverable_content_override,
            )
            .field("test_id", &self.test_id)
            .finish()
    }
}

impl PlainTooltip {
    pub fn new(trigger: AnyElement, text: impl Into<Arc<str>>) -> Self {
        Self {
            trigger,
            content: PlainTooltipContent::Text(text.into()),
            style: TooltipStyle::default(),
            align: TooltipAlign::default(),
            side: TooltipSide::default(),
            side_offset: Px(4.0),
            window_margin: Px(0.0),
            hide_when_detached: false,
            anchor_override: None,
            open_delay_frames_override: None,
            close_delay_frames_override: None,
            disable_hoverable_content_override: None,
            test_id: None,
        }
    }

    pub fn content_element(mut self, content: AnyElement) -> Self {
        self.content = PlainTooltipContent::Element(content);
        self
    }

    pub fn style(mut self, style: TooltipStyle) -> Self {
        self.style = self.style.merged(style);
        self
    }

    pub fn align(mut self, align: TooltipAlign) -> Self {
        self.align = align;
        self
    }

    pub fn side(mut self, side: TooltipSide) -> Self {
        self.side = side;
        self
    }

    pub fn side_offset(mut self, offset: Px) -> Self {
        self.side_offset = offset;
        self
    }

    pub fn window_margin(mut self, margin: Px) -> Self {
        self.window_margin = margin;
        self
    }

    pub fn hide_when_detached(mut self, hide: bool) -> Self {
        self.hide_when_detached = hide;
        self
    }

    pub fn anchor_element(mut self, element: fret_ui::elements::GlobalElementId) -> Self {
        self.anchor_override = Some(element);
        self
    }

    pub fn open_delay_frames(mut self, frames: Option<u32>) -> Self {
        self.open_delay_frames_override = frames;
        self
    }

    pub fn close_delay_frames(mut self, frames: Option<u32>) -> Self {
        self.close_delay_frames_override = frames;
        self
    }

    pub fn disable_hoverable_content(mut self, disable: Option<bool>) -> Self {
        self.disable_hoverable_content_override = disable;
        self
    }

    pub fn test_id(mut self, id: impl Into<Arc<str>>) -> Self {
        self.test_id = Some(id.into());
        self
    }

    #[track_caller]
    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let align = self.align;
        let side = self.side;
        let side_offset = self.side_offset;
        let window_margin = self.window_margin;
        let hide_when_detached = self.hide_when_detached;
        let anchor_override = self.anchor_override;
        let open_delay_frames_override = self.open_delay_frames_override;
        let close_delay_frames_override = self.close_delay_frames_override;
        let disable_hoverable_content_override = self.disable_hoverable_content_override;
        let test_id = self.test_id;
        let chrome_test_id = tooltip_part_id(&test_id, "chrome");
        let style = self.style;

        let base_trigger = self.trigger;
        let content_spec = self.content;
        let trigger_id = base_trigger.id;
        let anchor_id = anchor_override.unwrap_or(trigger_id);

        let (
            container_bg,
            shadow,
            supporting_text_style,
            content_max_width,
            content_min_width,
            content_min_height,
            container_padding,
            corner_radii,
            text_fg,
        ) = {
            let theme = Theme::global(&*cx.app);
            let states = WidgetStates::empty();

            let container_bg =
                tooltip_color_override(theme, &style.plain_container_background, states, || {
                    tooltip_tokens::plain_container_background(theme)
                });
            let text_fg =
                tooltip_color_override(theme, &style.plain_supporting_text_color, states, || {
                    tooltip_tokens::plain_supporting_text_color(theme)
                });
            let radius =
                tooltip_metric_override(&style.plain_container_corner_radius, states, || {
                    tooltip_tokens::plain_container_shape_radius(theme)
                });
            let corner_radii = Corners::all(radius);
            // Material Web v30 plain tooltip tokens do not include elevation; keep it flat by default.
            let elevation = Px(0.0);
            let shadow_color = tooltip_tokens::shadow_color(theme);
            let surface = material_surface_style(
                theme,
                container_bg,
                elevation,
                Some(shadow_color),
                corner_radii,
            );

            let supporting_text_style =
                tooltip_text_style_override(&style.plain_supporting_text_style, states, || {
                    tooltip_tokens::plain_supporting_text_style(theme)
                });
            let content_max_width =
                tooltip_metric_override(&style.plain_container_max_width, states, || {
                    tooltip_tokens::plain_container_max_width(theme)
                });
            let content_min_width =
                tooltip_metric_override(&style.container_min_width, states, || {
                    tooltip_tokens::container_min_width(theme)
                });
            let content_min_height =
                tooltip_metric_override(&style.container_min_height, states, || {
                    tooltip_tokens::container_min_height(theme)
                });
            let container_padding =
                tooltip_edges_override(&style.plain_container_padding, states, || {
                    tooltip_tokens::plain_container_padding(theme)
                });

            (
                surface.background,
                surface.shadow,
                supporting_text_style,
                content_max_width,
                content_min_width,
                content_min_height,
                container_padding,
                corner_radii,
                text_fg,
            )
        };

        let content = cx.named("content", move |cx| {
            let child = match content_spec {
                PlainTooltipContent::Text(text) => cx.text_props(TextProps {
                    layout: LayoutStyle::default(),
                    text,
                    style: Some(supporting_text_style),
                    color: Some(text_fg),
                    wrap: TextWrap::Word,
                    overflow: TextOverflow::Clip,
                    align: fret_core::TextAlign::Start,
                    ink_overflow: Default::default(),
                }),
                PlainTooltipContent::Element(el) => apply_tooltip_inherited_fg(el, text_fg),
            };

            let mut layout = LayoutStyle::default();
            layout.size.max_width = Some(Length::Px(content_max_width));
            layout.size.min_width = Some(Length::Px(content_min_width));
            layout.size.min_height = Some(Length::Px(content_min_height));

            let chrome = cx.container(
                ContainerProps {
                    layout,
                    padding: container_padding.into(),
                    background: Some(container_bg),
                    shadow,
                    corner_radii,
                    ..Default::default()
                },
                move |_cx| vec![child],
            );
            tooltip_content_root(
                cx,
                test_id.clone(),
                with_optional_test_id(chrome, chrome_test_id.clone()),
            )
        });

        tooltip_policy_root(
            cx,
            base_trigger,
            trigger_id,
            anchor_id,
            content,
            align,
            side,
            side_offset,
            window_margin,
            hide_when_detached,
            open_delay_frames_override,
            close_delay_frames_override,
            disable_hoverable_content_override,
        )
    }
}

enum RichTooltipContent {
    Text {
        title: Option<Arc<str>>,
        supporting_text: Arc<str>,
    },
    Element(AnyElement),
}

/// Material 3 Rich Tooltip (MVP).
///
/// Notes:
/// - Rich tooltips remain click-through because `OverlayKind::Tooltip` is not hit-testable in Fret.
/// - Action rows are therefore out-of-scope until we have a concrete consumer that requires an
///   interactive outcome (mechanism follow-up candidate).
pub struct RichTooltip {
    trigger: AnyElement,
    content: RichTooltipContent,
    style: TooltipStyle,
    align: TooltipAlign,
    side: TooltipSide,
    side_offset: Px,
    window_margin: Px,
    hide_when_detached: bool,
    anchor_override: Option<fret_ui::elements::GlobalElementId>,
    open_delay_frames_override: Option<u32>,
    close_delay_frames_override: Option<u32>,
    disable_hoverable_content_override: Option<bool>,
    test_id: Option<Arc<str>>,
}

impl std::fmt::Debug for RichTooltip {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RichTooltip")
            .field("trigger_id", &self.trigger.id)
            .field("style", &self.style)
            .field("align", &self.align)
            .field("side", &self.side)
            .field("side_offset", &self.side_offset)
            .field("window_margin", &self.window_margin)
            .field("hide_when_detached", &self.hide_when_detached)
            .field("anchor_override", &self.anchor_override)
            .field(
                "open_delay_frames_override",
                &self.open_delay_frames_override,
            )
            .field(
                "close_delay_frames_override",
                &self.close_delay_frames_override,
            )
            .field(
                "disable_hoverable_content_override",
                &self.disable_hoverable_content_override,
            )
            .field("test_id", &self.test_id)
            .finish()
    }
}

impl RichTooltip {
    pub fn new(trigger: AnyElement, supporting_text: impl Into<Arc<str>>) -> Self {
        Self {
            trigger,
            content: RichTooltipContent::Text {
                title: None,
                supporting_text: supporting_text.into(),
            },
            style: TooltipStyle::default(),
            align: TooltipAlign::default(),
            side: TooltipSide::default(),
            side_offset: Px(4.0),
            window_margin: Px(0.0),
            hide_when_detached: false,
            anchor_override: None,
            open_delay_frames_override: None,
            close_delay_frames_override: None,
            disable_hoverable_content_override: None,
            test_id: None,
        }
    }

    pub fn title(mut self, title: impl Into<Arc<str>>) -> Self {
        if let RichTooltipContent::Text { title: t, .. } = &mut self.content {
            *t = Some(title.into());
        }
        self
    }

    pub fn content_element(mut self, content: AnyElement) -> Self {
        self.content = RichTooltipContent::Element(content);
        self
    }

    pub fn style(mut self, style: TooltipStyle) -> Self {
        self.style = self.style.merged(style);
        self
    }

    pub fn align(mut self, align: TooltipAlign) -> Self {
        self.align = align;
        self
    }

    pub fn side(mut self, side: TooltipSide) -> Self {
        self.side = side;
        self
    }

    pub fn side_offset(mut self, offset: Px) -> Self {
        self.side_offset = offset;
        self
    }

    pub fn window_margin(mut self, margin: Px) -> Self {
        self.window_margin = margin;
        self
    }

    pub fn hide_when_detached(mut self, hide: bool) -> Self {
        self.hide_when_detached = hide;
        self
    }

    pub fn anchor_element(mut self, element: fret_ui::elements::GlobalElementId) -> Self {
        self.anchor_override = Some(element);
        self
    }

    pub fn open_delay_frames(mut self, frames: Option<u32>) -> Self {
        self.open_delay_frames_override = frames;
        self
    }

    pub fn close_delay_frames(mut self, frames: Option<u32>) -> Self {
        self.close_delay_frames_override = frames;
        self
    }

    pub fn disable_hoverable_content(mut self, disable: Option<bool>) -> Self {
        self.disable_hoverable_content_override = disable;
        self
    }

    pub fn test_id(mut self, id: impl Into<Arc<str>>) -> Self {
        self.test_id = Some(id.into());
        self
    }

    #[track_caller]
    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let align = self.align;
        let side = self.side;
        let side_offset = self.side_offset;
        let window_margin = self.window_margin;
        let hide_when_detached = self.hide_when_detached;
        let anchor_override = self.anchor_override;
        let open_delay_frames_override = self.open_delay_frames_override;
        let close_delay_frames_override = self.close_delay_frames_override;
        let disable_hoverable_content_override = self.disable_hoverable_content_override;
        let test_id = self.test_id;
        let chrome_test_id = tooltip_part_id(&test_id, "chrome");
        let title_test_id = tooltip_part_id(&test_id, "title");
        let supporting_text_test_id = tooltip_part_id(&test_id, "supporting-text");
        let style = self.style;

        let base_trigger = self.trigger;
        let content_spec = self.content;
        let trigger_id = base_trigger.id;
        let anchor_id = anchor_override.unwrap_or(trigger_id);
        let has_title = matches!(
            &content_spec,
            RichTooltipContent::Text { title: Some(_), .. }
        );

        let (
            container_bg,
            shadow,
            subhead_fg,
            supporting_fg,
            corner_radii,
            subhead_style,
            supporting_style,
            content_max_width,
            content_min_width,
            content_min_height,
            container_padding,
            text_gap,
        ) = {
            let theme = Theme::global(&*cx.app);
            let states = WidgetStates::empty();

            let container_bg =
                tooltip_color_override(theme, &style.rich_container_background, states, || {
                    tooltip_tokens::rich_container_background(theme)
                });
            let subhead_fg = tooltip_color_override(theme, &style.rich_title_color, states, || {
                tooltip_tokens::rich_subhead_color(theme)
            });
            let supporting_fg =
                tooltip_color_override(theme, &style.rich_supporting_text_color, states, || {
                    tooltip_tokens::rich_supporting_text_color(theme)
                });
            let radius =
                tooltip_metric_override(&style.rich_container_corner_radius, states, || {
                    tooltip_tokens::rich_container_shape_radius(theme)
                });
            let corner_radii = Corners::all(radius);
            let elevation =
                tooltip_metric_override(&style.rich_container_elevation, states, || {
                    tooltip_tokens::rich_container_elevation(theme)
                });
            let shadow_color =
                tooltip_color_override(theme, &style.rich_container_shadow_color, states, || {
                    tooltip_tokens::rich_container_shadow_color(theme)
                });
            let surface = material_surface_style(
                theme,
                container_bg,
                elevation,
                Some(shadow_color),
                corner_radii,
            );

            let subhead_style =
                tooltip_text_style_override(&style.rich_title_text_style, states, || {
                    tooltip_tokens::rich_subhead_text_style(theme)
                });
            let supporting_style =
                tooltip_text_style_override(&style.rich_supporting_text_style, states, || {
                    tooltip_tokens::rich_supporting_text_style(theme)
                });

            let content_max_width =
                tooltip_metric_override(&style.rich_container_max_width, states, || {
                    tooltip_tokens::rich_container_max_width(theme)
                });
            let content_min_width =
                tooltip_metric_override(&style.container_min_width, states, || {
                    tooltip_tokens::container_min_width(theme)
                });
            let content_min_height =
                tooltip_metric_override(&style.container_min_height, states, || {
                    tooltip_tokens::container_min_height(theme)
                });
            let container_padding =
                tooltip_edges_override(&style.rich_container_padding, states, || {
                    tooltip_tokens::rich_container_padding(theme, has_title)
                });
            let text_gap = tooltip_metric_override(&style.rich_text_gap, states, || {
                tooltip_tokens::rich_text_gap(theme)
            });

            (
                surface.background,
                surface.shadow,
                subhead_fg,
                supporting_fg,
                corner_radii,
                subhead_style,
                supporting_style,
                content_max_width,
                content_min_width,
                content_min_height,
                container_padding,
                text_gap,
            )
        };

        let content = cx.named("content", move |cx| {
            let child = match content_spec {
                RichTooltipContent::Text {
                    title,
                    supporting_text,
                } => {
                    let mut props = FlexProps::default();
                    props.direction = Axis::Vertical;
                    props.gap = text_gap.into();

                    cx.flex(props, move |cx| {
                        let mut children = Vec::new();
                        if let Some(title) = title.clone() {
                            let title = cx.text_props(TextProps {
                                layout: LayoutStyle::default(),
                                text: title,
                                style: Some(subhead_style),
                                color: Some(subhead_fg),
                                wrap: TextWrap::Word,
                                overflow: TextOverflow::Clip,
                                align: fret_core::TextAlign::Start,
                                ink_overflow: Default::default(),
                            });
                            children.push(with_optional_test_id(title, title_test_id.clone()));
                        }
                        let supporting_text = cx.text_props(TextProps {
                            layout: LayoutStyle::default(),
                            text: supporting_text.clone(),
                            style: Some(supporting_style),
                            color: Some(supporting_fg),
                            wrap: TextWrap::Word,
                            overflow: TextOverflow::Clip,
                            align: fret_core::TextAlign::Start,
                            ink_overflow: Default::default(),
                        });
                        children.push(with_optional_test_id(
                            supporting_text,
                            supporting_text_test_id.clone(),
                        ));
                        children
                    })
                }
                RichTooltipContent::Element(el) => apply_tooltip_inherited_fg(el, supporting_fg),
            };

            let mut layout = LayoutStyle::default();
            layout.size.max_width = Some(Length::Px(content_max_width));
            layout.size.min_width = Some(Length::Px(content_min_width));
            layout.size.min_height = Some(Length::Px(content_min_height));

            let chrome = cx.container(
                ContainerProps {
                    layout,
                    padding: container_padding.into(),
                    background: Some(container_bg),
                    shadow,
                    corner_radii,
                    ..Default::default()
                },
                move |_cx| vec![child],
            );

            tooltip_content_root(
                cx,
                test_id.clone(),
                with_optional_test_id(chrome, chrome_test_id.clone()),
            )
        });

        tooltip_policy_root(
            cx,
            base_trigger,
            trigger_id,
            anchor_id,
            content,
            align,
            side,
            side_offset,
            window_margin,
            hide_when_detached,
            open_delay_frames_override,
            close_delay_frames_override,
            disable_hoverable_content_override,
        )
    }
}
