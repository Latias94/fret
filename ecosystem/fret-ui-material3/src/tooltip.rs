//! Material 3 tooltip (MVP).
//!
//! This module currently targets the "plain tooltip" outcome:
//! - floating placement via `fret-ui-kit` popper helpers
//! - Radix-aligned open delay + safe-hover corridor policies via `fret-ui-kit` tooltip primitives
//! - token-driven container + text styling (`md.comp.plain-tooltip.*`)

use std::sync::Arc;

use fret_core::{
    Color, Corners, Edges, KeyCode, PointerType, Px, Rect, Size, TextOverflow, TextWrap,
};
use fret_ui::element::{
    AnyElement, ContainerProps, ElementKind, HoverRegionProps, LayoutStyle, PointerRegionProps,
    SemanticsProps, SpinnerProps, SvgIconProps, TextProps,
};
use fret_ui::overlay_placement::{Align, Side};
use fret_ui::theme::CubicBezier;
use fret_ui::{ElementContext, Theme, UiHost};
use fret_ui_kit::OverlayPresence;
use fret_ui_kit::declarative::ModelWatchExt;
use fret_ui_kit::declarative::scheduling;
use fret_ui_kit::declarative::transition;
use fret_ui_kit::primitives::direction as direction_prim;
use fret_ui_kit::primitives::dismissable_layer as dismissable_layer_prim;
use fret_ui_kit::primitives::popper;
use fret_ui_kit::primitives::popper_content;
use fret_ui_kit::primitives::tooltip as tooltip_prim;
use fret_ui_kit::tooltip_provider;

use crate::foundation::elevation::shadow_for_elevation_with_color;
use crate::foundation::token_resolver::MaterialTokenResolver;
use crate::motion::ms_to_frames;

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

    pub fn with<H: UiHost>(
        self,
        cx: &mut ElementContext<'_, H>,
        f: impl FnOnce(&mut ElementContext<'_, H>) -> Vec<AnyElement>,
    ) -> Vec<AnyElement> {
        tooltip_provider::with_tooltip_provider(
            cx,
            tooltip_provider::TooltipProviderConfig::new(
                self.delay_duration_frames as u64,
                self.skip_delay_duration_frames as u64,
            )
            .disable_hoverable_content(self.disable_hoverable_content),
            f,
        )
    }
}

#[derive(Clone)]
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
}

fn tooltip_trigger_event_models<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
) -> TooltipTriggerEventModels {
    #[derive(Default)]
    struct State {
        models: Option<TooltipTriggerEventModels>,
    }

    let existing = cx.with_state(State::default, |st| st.models.clone());
    if let Some(models) = existing {
        return models;
    }

    let models = TooltipTriggerEventModels {
        has_pointer_move_opened: cx.app.models_mut().insert(false),
        pointer_transit_geometry: tooltip_provider::pointer_transit_geometry_model(cx),
        suppress_hover_open: cx.app.models_mut().insert(false),
        suppress_focus_open: cx.app.models_mut().insert(false),
        close_requested: cx.app.models_mut().insert(false),
    };

    cx.with_state(State::default, |st| st.models = Some(models.clone()));
    models
}

#[derive(Debug, Default, Clone, Copy)]
struct TooltipTriggerHoverEdgeState {
    was_hovered: bool,
}

/// Material 3 Plain Tooltip (MVP).
///
/// This is a policy wrapper built on `fret-ui-kit` tooltip primitives.
#[derive(Clone)]
pub struct PlainTooltip {
    trigger: AnyElement,
    content: PlainTooltipContent,
    align: TooltipAlign,
    side: TooltipSide,
    side_offset: Px,
    window_margin: Px,
    hide_when_detached: bool,
    anchor_override: Option<fret_ui::elements::GlobalElementId>,
    open_delay_frames_override: Option<u32>,
    close_delay_frames_override: Option<u32>,
    disable_hoverable_content_override: Option<bool>,
}

impl std::fmt::Debug for PlainTooltip {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PlainTooltip")
            .field("trigger_id", &self.trigger.id)
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
            .finish()
    }
}

impl PlainTooltip {
    pub fn new(trigger: AnyElement, text: impl Into<Arc<str>>) -> Self {
        Self {
            trigger,
            content: PlainTooltipContent::Text(text.into()),
            align: TooltipAlign::default(),
            side: TooltipSide::default(),
            side_offset: Px(4.0),
            window_margin: Px(0.0),
            hide_when_detached: false,
            anchor_override: None,
            open_delay_frames_override: None,
            close_delay_frames_override: None,
            disable_hoverable_content_override: None,
        }
    }

    pub fn content_element(mut self, content: AnyElement) -> Self {
        self.content = PlainTooltipContent::Element(content);
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

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let theme = Theme::global(&*cx.app).clone();
        let resolver = MaterialTokenResolver::new(&theme);

        let align = self.align;
        let side = self.side;
        let side_offset = self.side_offset;
        let window_margin = self.window_margin;
        let hide_when_detached = self.hide_when_detached;
        let anchor_override = self.anchor_override;
        let open_delay_frames_override = self.open_delay_frames_override;
        let close_delay_frames_override = self.close_delay_frames_override;
        let disable_hoverable_content_override = self.disable_hoverable_content_override;

        let base_trigger = self.trigger;
        let content_spec = self.content;
        let trigger_id = base_trigger.id;
        let anchor_id = anchor_override.unwrap_or(trigger_id);

        let container_bg = resolver.color_comp_or_sys(
            "md.comp.plain-tooltip.container.color",
            "md.sys.color.inverse-surface",
        );
        let text_fg = resolver.color_comp_or_sys(
            "md.comp.plain-tooltip.supporting-text.color",
            "md.sys.color.inverse-on-surface",
        );
        let radius = theme
            .metric_by_key("md.comp.plain-tooltip.container.shape")
            .unwrap_or(Px(4.0));
        let corner_radii = Corners::all(radius);
        let elevation = theme
            .metric_by_key("md.comp.plain-tooltip.container.elevation")
            .or_else(|| theme.metric_by_key("md.comp.rich-tooltip.container.elevation"))
            .unwrap_or(Px(0.0));
        let shadow_color = theme
            .color_by_key("md.comp.plain-tooltip.container.shadow-color")
            .or_else(|| theme.color_by_key("md.comp.rich-tooltip.container.shadow-color"))
            .unwrap_or_else(|| resolver.color_sys("md.sys.color.shadow"));
        let shadow =
            shadow_for_elevation_with_color(&theme, elevation, Some(shadow_color), corner_radii);

        let body_small = theme
            .text_style_by_key("md.sys.typescale.body-small")
            .unwrap_or_default();

        let content = cx.named("content", move |cx| {
            let child = match content_spec {
                PlainTooltipContent::Text(text) => cx.text_props(TextProps {
                    layout: LayoutStyle::default(),
                    text,
                    style: Some(body_small),
                    color: Some(text_fg),
                    wrap: TextWrap::Word,
                    overflow: TextOverflow::Clip,
                }),
                PlainTooltipContent::Element(el) => apply_tooltip_inherited_fg(el, text_fg),
            };

            let mut layout = LayoutStyle::default();
            layout.size.max_width = Some(Px(240.0));

            let container = cx.container(
                ContainerProps {
                    layout,
                    padding: Edges {
                        left: Px(8.0),
                        right: Px(8.0),
                        top: Px(4.0),
                        bottom: Px(4.0),
                    },
                    background: Some(container_bg),
                    shadow,
                    corner_radii,
                    ..Default::default()
                },
                move |_cx| vec![child],
            );

            cx.semantics(
                SemanticsProps {
                    role: fret_core::SemanticsRole::Tooltip,
                    ..Default::default()
                },
                move |_cx| vec![container],
            )
        });
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

            let left_hover = cx.with_state(TooltipTriggerHoverEdgeState::default, |st| {
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
            let disable_hoverable_content = disable_hoverable_content_override
                .unwrap_or(provider_cfg.disable_hoverable_content);
            let last_pointer = tooltip_prim::tooltip_last_pointer_model(cx);

            let trigger_hovered = hovered && has_pointer_move_opened && !suppress_hover_open;
            let trigger_focused = focused && !suppress_focus_open;

            let anchor_bounds = fret_ui_kit::overlay::anchor_bounds_for_element(cx, anchor_id);
            let floating_bounds = anchor_bounds.and_then(|anchor| {
                let last_content_size = cx.last_bounds_for_element(content_id).map(|r| r.size);
                let estimated_size = Size::new(Px(240.0), Px(32.0));
                let content_size = last_content_size.unwrap_or(estimated_size);

                let outer = fret_ui_kit::overlay::outer_bounds_with_window_margin(
                    cx.bounds,
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

                let direction = direction_prim::use_direction_in_scope(cx, None);
                let layout = popper::popper_content_layout_sized(
                    outer,
                    anchor,
                    content_size,
                    popper::PopperContentPlacement::new(direction, side, align, side_offset)
                        .with_shift_cross_axis(true),
                );

                Some(layout.rect)
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

            let trigger = tooltip_prim::apply_tooltip_trigger_a11y(
                base_trigger.clone(),
                update.open,
                content_id,
            );

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
                            let _ =
                                host.models_mut().update(&suppress_hover_open, |v| *v = true);
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

            let opening = update.open;
            let open_ticks = ms_to_frames(
                theme
                    .duration_ms_by_key("md.sys.motion.duration.short2")
                    .unwrap_or(100),
            );
            let close_ticks = ms_to_frames(
                theme
                    .duration_ms_by_key("md.sys.motion.duration.short1")
                    .unwrap_or(50),
            );
            let easing = theme
                .easing_by_key("md.sys.motion.easing.emphasized")
                .or_else(|| theme.easing_by_key("md.sys.motion.easing.standard"))
                .unwrap_or(CubicBezier {
                    x1: 0.0,
                    y1: 0.0,
                    x2: 1.0,
                    y2: 1.0,
                });
            let motion = transition::drive_transition_with_durations_and_cubic_bezier(
                cx,
                opening,
                open_ticks,
                close_ticks,
                easing,
            );

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
            let opacity = motion.progress;
            let scale = 0.92 + 0.08 * motion.progress;
            let direction = direction_prim::use_direction_in_scope(cx, None);

            let overlay_children = cx.with_root_name(&overlay_root_name, move |cx| {
                let anchor = fret_ui_kit::overlay::anchor_bounds_for_element(cx, anchor_id);
                let Some(anchor) = anchor else {
                    return Vec::new();
                };

                let last_content_size = cx.last_bounds_for_element(content_id).map(|r| r.size);
                let estimated_size = Size::new(Px(240.0), Px(32.0));
                let content_size = last_content_size.unwrap_or(estimated_size);

                let outer = fret_ui_kit::overlay::outer_bounds_with_window_margin(
                    cx.bounds,
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

                let layout = popper::popper_content_layout_sized(outer, anchor, content_size, placement);
                let placed = layout.rect;

                let wrapper = popper_content::popper_wrapper_panel_at(
                    cx,
                    placed,
                    Edges::all(Px(0.0)),
                    fret_ui::element::Overflow::Visible,
                    move |_cx| vec![content.clone()],
                );

                let origin = popper::popper_content_transform_origin(&layout, anchor, None);
                let origin_inv = fret_core::Point::new(Px(-origin.x.0), Px(-origin.y.0));
                let transform = fret_core::Transform2D::translation(origin)
                    * fret_core::Transform2D::scale_uniform(scale)
                    * fret_core::Transform2D::translation(origin_inv);

                let opacity = if reference_hidden { 0.0 } else { opacity };
                vec![fret_ui_kit::declarative::overlay_motion::wrap_opacity_and_render_transform_gated(
                    cx,
                    opacity,
                    transform,
                    !reference_hidden,
                    vec![wrapper],
                )]
            });

            let mut request = tooltip_prim::tooltip_request(tooltip_id, overlay_presence, overlay_children);
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
}
