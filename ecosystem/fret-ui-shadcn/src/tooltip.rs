use crate::layout as shadcn_layout;
use crate::popper_arrow::{self, DiamondArrowStyle};
use fret_ui_kit::declarative::ModelWatchExt;
use fret_ui_kit::declarative::scheduling;
use fret_ui_kit::declarative::style as decl_style;
use fret_ui_kit::overlay;
use fret_ui_kit::primitives::direction as direction_prim;
use fret_ui_kit::primitives::dismissable_layer as radix_dismissable_layer;
use fret_ui_kit::primitives::popper;
use fret_ui_kit::primitives::popper_content;
use fret_ui_kit::primitives::presence as radix_presence;
use fret_ui_kit::primitives::tooltip as radix_tooltip;
use fret_ui_kit::tooltip_provider;
use fret_ui_kit::{
    ChromeRefinement, ColorRef, LayoutRefinement, MetricRef, OverlayPresence, Radius, Space, ui,
};
use std::sync::Arc;

use fret_core::{KeyCode, PointerType, Px, Rect, Size, TextOverflow, TextStyle, TextWrap};
use fret_runtime::Model;
use fret_ui::element::{
    AnyElement, ElementKind, HoverRegionProps, Overflow, PointerRegionProps, SemanticsProps,
    SpinnerProps, SvgIconProps,
};
use fret_ui::overlay_placement::{Align, Side};
use fret_ui::{ElementContext, Theme, UiHost};

use crate::overlay_motion;

fn apply_tooltip_inherited_fg(mut element: AnyElement, fg: fret_core::Color) -> AnyElement {
    match &mut element.kind {
        ElementKind::Text(props) => {
            if props.color.is_none() {
                props.color = Some(fg);
            }
        }
        ElementKind::SvgIcon(SvgIconProps { color, .. }) => {
            let is_default = *color
                == fret_core::Color {
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

fn tooltip_text_fg(theme: &Theme) -> fret_core::Color {
    theme.color_required("background")
}

fn tooltip_text_style(theme: &Theme) -> TextStyle {
    // new-york-v4 uses `text-xs` for tooltips (base is `text-sm`).
    let base_px = theme.metric_required("font.size");
    let base_line_height = theme.metric_required("font.line_height");

    let px = theme
        .metric_by_key("component.tooltip.text_px")
        .unwrap_or(Px((base_px.0 - 2.0).max(10.0)));
    let line_height = theme
        .metric_by_key("component.tooltip.line_height")
        .unwrap_or(Px((base_line_height.0 - 4.0).max(12.0)));

    TextStyle {
        font: fret_core::FontId::default(),
        size: px,
        weight: fret_core::FontWeight::NORMAL,
        slant: Default::default(),
        line_height: Some(line_height),
        letter_spacing_em: None,
    }
}

fn tooltip_content_chrome(theme: &Theme) -> ChromeRefinement {
    // shadcn/ui v4 (2025-09-22): tooltip uses `bg-foreground text-background`.
    let bg = theme.color_required("foreground");

    ChromeRefinement::default()
        .rounded(Radius::Md)
        .bg(ColorRef::Color(bg))
        .px(Space::N3)
        .py(Space::N1p5)
}

#[derive(Clone)]
struct TooltipTriggerEventModels {
    has_pointer_move_opened: Model<bool>,
    pointer_transit_geometry: Model<Option<(Rect, Rect)>>,
    suppress_hover_open: Model<bool>,
    suppress_focus_open: Model<bool>,
    close_requested: Model<bool>,
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

/// shadcn/ui `TooltipProvider` (v4).
///
/// In Radix/shadcn this is a context provider used to share open-delay behavior across tooltip
/// instances. In Fret, this is implemented as a declarative scoping helper that persists provider
/// state (delay group) across frames.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct TooltipProvider {
    delay_duration_frames: u32,
    skip_delay_duration_frames: u32,
    disable_hoverable_content: bool,
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

/// shadcn/ui `Tooltip` root (v4).
///
/// This is implemented as a component-layer policy built on runtime substrate primitives:
/// - `HoverRegion` (hover tracking)
/// - cross-frame geometry queries (`elements::bounds_for_element`)
/// - placement solver (`overlay_placement`)
///
/// Note: This uses a per-window overlay root, so it is not clipped by ancestors with
/// `overflow: Clip`.
#[derive(Debug, Clone)]
pub struct Tooltip {
    trigger: AnyElement,
    content: AnyElement,
    align: TooltipAlign,
    side: TooltipSide,
    side_offset: Px,
    window_margin_override: Option<Px>,
    arrow: bool,
    arrow_size_override: Option<Px>,
    arrow_padding_override: Option<Px>,
    hide_when_detached: bool,
    open_delay_frames_override: Option<u32>,
    close_delay_frames_override: Option<u32>,
    disable_hoverable_content_override: Option<bool>,
    layout: LayoutRefinement,
    anchor_override: Option<fret_ui::elements::GlobalElementId>,
}

impl Tooltip {
    pub fn new(trigger: AnyElement, content: AnyElement) -> Self {
        Self {
            trigger,
            content,
            align: TooltipAlign::default(),
            side: TooltipSide::default(),
            side_offset: Px(0.0),
            window_margin_override: None,
            arrow: true,
            arrow_size_override: None,
            arrow_padding_override: None,
            hide_when_detached: false,
            open_delay_frames_override: None,
            close_delay_frames_override: None,
            disable_hoverable_content_override: None,
            layout: LayoutRefinement::default(),
            anchor_override: None,
        }
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

    pub fn open_delay_frames(mut self, frames: u32) -> Self {
        self.open_delay_frames_override = Some(frames);
        self
    }

    pub fn close_delay_frames(mut self, frames: u32) -> Self {
        self.close_delay_frames_override = Some(frames);
        self
    }

    /// When `true`, hovering the tooltip content does not keep it open (Radix `disableHoverableContent`).
    ///
    /// Default: inherited from `TooltipProvider`.
    pub fn disable_hoverable_content(mut self, disable: bool) -> Self {
        self.disable_hoverable_content_override = Some(disable);
        self
    }

    pub fn window_margin(mut self, margin: Px) -> Self {
        self.window_margin_override = Some(margin);
        self
    }

    /// Enables a Tooltip arrow (Radix `TooltipArrow`-style).
    ///
    /// Default: `true`.
    pub fn arrow(mut self, arrow: bool) -> Self {
        self.arrow = arrow;
        self
    }

    pub fn arrow_size(mut self, size: Px) -> Self {
        self.arrow_size_override = Some(size);
        self
    }

    pub fn arrow_padding(mut self, padding: Px) -> Self {
        self.arrow_padding_override = Some(padding);
        self
    }

    /// When `true`, the tooltip content becomes hidden and non-interactive if the anchor is fully
    /// clipped by the collision boundary (Radix `hideWhenDetached`).
    ///
    /// Default: `false`.
    pub fn hide_when_detached(mut self, hide: bool) -> Self {
        self.hide_when_detached = hide;
        self
    }

    /// Override the element used as the placement anchor.
    ///
    /// Notes:
    /// - Hover/focus tracking still uses the trigger element.
    /// - The anchor bounds are resolved from last-frame layout/visual bounds (same as Popover).
    pub fn anchor_element(mut self, element: fret_ui::elements::GlobalElementId) -> Self {
        self.anchor_override = Some(element);
        self
    }

    pub fn refine_layout(mut self, layout: LayoutRefinement) -> Self {
        self.layout = self.layout.merge(layout);
        self
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let theme = Theme::global(&*cx.app).clone();

        let layout = decl_style::layout_style(&theme, self.layout);
        let side_offset = if self.side_offset == Px(0.0) {
            theme
                .metric_by_key("component.tooltip.side_offset")
                .unwrap_or(self.side_offset)
        } else {
            self.side_offset
        };
        let window_margin = self.window_margin_override.unwrap_or_else(|| {
            theme
                .metric_by_key("component.tooltip.window_margin")
                .unwrap_or(Px(0.0))
        });
        let arrow = self.arrow;
        let arrow_size = self.arrow_size_override.unwrap_or_else(|| {
            theme
                .metric_by_key("component.tooltip.arrow_size")
                .unwrap_or(Px(10.0))
        });
        let arrow_padding = self.arrow_padding_override.unwrap_or_else(|| {
            theme
                .metric_by_key("component.tooltip.arrow_padding")
                .unwrap_or_else(|| MetricRef::radius(Radius::Sm).resolve(&theme))
        });
        let arrow_bg = theme.color_required("foreground");
        let hide_when_detached = self.hide_when_detached;

        let align = self.align;
        let side = self.side;
        let open_delay_frames_override = self.open_delay_frames_override;
        let close_delay_frames_override = self.close_delay_frames_override;
        let disable_hoverable_content_override = self.disable_hoverable_content_override;

        let base_trigger = self.trigger;
        let content = self.content;
        let trigger_id = base_trigger.id;
        let content_id = content.id;
        let anchor_id = self.anchor_override.unwrap_or(trigger_id);

        cx.hover_region(HoverRegionProps { layout }, move |cx, hovered| {
            let focused = cx.is_focused_element(trigger_id);
            let event_models = tooltip_trigger_event_models(cx);
            let tooltip_id = cx.root_id();

            #[derive(Default)]
            struct TooltipOpenModelState {
                model: Option<Model<bool>>,
            }

            let open = cx.with_state_for(tooltip_id, TooltipOpenModelState::default, |st| {
                st.model.clone()
            });
            let open = if let Some(model) = open {
                model
            } else {
                let model = cx.app.models_mut().insert(false);
                cx.with_state_for(tooltip_id, TooltipOpenModelState::default, |st| {
                    st.model = Some(model.clone());
                });
                model
            };
            let mut open_now = cx.watch_model(&open).layout().copied().unwrap_or(false);

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
            let last_pointer = radix_tooltip::tooltip_last_pointer_model(cx);

            let trigger_hovered = hovered && has_pointer_move_opened && !suppress_hover_open;
            let trigger_focused = focused && !suppress_focus_open;

            let anchor_bounds = overlay::anchor_bounds_for_element(cx, anchor_id);
            let floating_bounds = anchor_bounds.and_then(|anchor| {
                let last_content_size = cx.last_bounds_for_element(content_id).map(|r| r.size);
                let estimated_size = Size::new(Px(240.0), Px(44.0));
                let content_size = last_content_size.unwrap_or(estimated_size);

                let outer = overlay::outer_bounds_with_window_margin(cx.bounds, window_margin);

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

                let (arrow_options, arrow_protrusion) =
                    popper::diamond_arrow_options(arrow, arrow_size, arrow_padding);
                let direction = direction_prim::use_direction_in_scope(cx, None);

                let layout = popper::popper_content_layout_sized(
                    outer,
                    anchor,
                    content_size,
                    popper::PopperContentPlacement::new(direction, side, align, side_offset)
                        .with_shift_cross_axis(true)
                        .with_arrow(arrow_options, arrow_protrusion),
                );

                // Use the panel rect (not the wrapper rect that includes motion insets) for hover
                // and pointer-transit policies. This keeps Radix-like grace areas from becoming
                // overly permissive during shadcn enter/exit motion.
                Some(layout.rect)
            });

            let update = radix_tooltip::tooltip_update_interaction(
                cx,
                trigger_hovered,
                trigger_focused,
                close_requested,
                last_pointer.clone(),
                anchor_bounds,
                floating_bounds,
                radix_tooltip::TooltipInteractionConfig {
                    disable_hoverable_content,
                    open_delay_ticks_override: open_delay_frames_override.map(|v| v as u64),
                    close_delay_ticks_override: close_delay_frames_override.map(|v| v as u64),
                    safe_hover_buffer: Px(5.0),
                },
            );

            scheduling::set_continuous_frames(cx, update.wants_continuous_ticks);

            if update.open != open_now {
                let _ = cx.app.models_mut().update(&open, |v| *v = update.open);
                open_now = update.open;
            }

            let trigger = radix_tooltip::apply_tooltip_trigger_a11y(
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
                            && radix_tooltip::tooltip_pointer_in_transit(
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
            let motion = radix_presence::scale_fade_presence_with_durations_and_easing(
                cx,
                opening,
                overlay_motion::SHADCN_MOTION_TICKS_100,
                overlay_motion::SHADCN_MOTION_TICKS_100,
                0.95,
                1.0,
                overlay_motion::shadcn_ease,
            );
            let overlay_presence = OverlayPresence {
                present: motion.present,
                interactive: update.open,
            };

            let out = vec![trigger];
            if !overlay_presence.present {
                return out;
            }

            let overlay_root_name = radix_tooltip::tooltip_root_name(tooltip_id);
            let opacity = motion.opacity;
            let scale = motion.scale;
            let direction = direction_prim::use_direction_in_scope(cx, None);

            let overlay_children = cx.with_root_name(&overlay_root_name, move |cx| {
                let anchor = overlay::anchor_bounds_for_element(cx, anchor_id);
                let Some(anchor) = anchor else {
                    return Vec::new();
                };

                let last_content_size = cx.last_bounds_for_element(content_id).map(|r| r.size);
                let estimated_size = Size::new(Px(240.0), Px(44.0));
                let content_size = last_content_size.unwrap_or(estimated_size);

                let outer = overlay::outer_bounds_with_window_margin(cx.bounds, window_margin);

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

                let (arrow_options, arrow_protrusion) =
                    popper::diamond_arrow_options(arrow, arrow_size, arrow_padding);

                let placement =
                    popper::PopperContentPlacement::new(direction, side, align, side_offset)
                        .with_shift_cross_axis(true)
                        .with_arrow(arrow_options, arrow_protrusion)
                        .with_hide_when_detached(hide_when_detached);
                let reference_hidden = placement.reference_hidden(outer, anchor);

                let layout =
                    popper::popper_content_layout_sized(outer, anchor, content_size, placement);

                let placed = layout.rect;
                let mut wrapper_insets = popper_arrow::wrapper_insets(&layout, arrow_protrusion);
                let slide_insets = overlay_motion::shadcn_slide_insets(layout.side);
                wrapper_insets.top.0 += slide_insets.top.0;
                wrapper_insets.right.0 += slide_insets.right.0;
                wrapper_insets.bottom.0 += slide_insets.bottom.0;
                wrapper_insets.left.0 += slide_insets.left.0;

                let wrapper = popper_content::popper_wrapper_at_with_panel(
                    cx,
                    placed,
                    wrapper_insets,
                    Overflow::Visible,
                    move |_cx| vec![content],
                    move |cx, content| {
                        // new-york-v4: `size-2.5 rotate-45 rounded-[2px] translate-y-[calc(-50%_-_2px)]`
                        // (i.e. a slightly outset, lightly rounded diamond).
                        let arrow_el = popper_arrow::diamond_arrow_element_refined(
                            cx,
                            &layout,
                            wrapper_insets,
                            arrow_size,
                            DiamondArrowStyle {
                                bg: arrow_bg,
                                border: None,
                                border_width: Px(0.0),
                            },
                            Px(2.0),
                            Px(2.0),
                        );

                        if let Some(arrow_el) = arrow_el {
                            vec![arrow_el, content]
                        } else {
                            vec![content]
                        }
                    },
                );

                let origin = popper::popper_content_transform_origin(
                    &layout,
                    anchor,
                    arrow.then_some(arrow_size),
                );
                let opacity = if reference_hidden { 0.0 } else { opacity };
                let transform = overlay_motion::shadcn_popper_presence_transform(
                    layout.side,
                    origin,
                    opacity,
                    scale,
                    opening,
                );

                vec![overlay_motion::wrap_opacity_and_render_transform_gated(
                    cx,
                    opacity,
                    transform,
                    !reference_hidden,
                    vec![wrapper],
                )]
            });

            let mut request = radix_tooltip::tooltip_request(
                tooltip_id,
                open.clone(),
                overlay_presence,
                overlay_children,
            );
            request.trigger = Some(trigger_id);
            request.dismissible_on_dismiss_request = Some(radix_dismissable_layer::handler({
                let close_requested = event_models.close_requested.clone();
                move |host, acx, _reason| {
                    let _ = host.models_mut().update(&close_requested, |v| *v = true);
                    host.request_redraw(acx.window);
                }
            }));
            if !disable_hoverable_content {
                radix_tooltip::tooltip_install_pointer_move_tracker(&mut request, last_pointer);
            }
            radix_tooltip::request_tooltip(cx, request);

            out
        })
    }
}

/// shadcn/ui `TooltipTrigger` (v4).
#[derive(Debug, Clone)]
pub struct TooltipTrigger {
    child: AnyElement,
}

impl TooltipTrigger {
    pub fn new(child: AnyElement) -> Self {
        Self { child }
    }

    pub fn into_element<H: UiHost>(self, _cx: &mut ElementContext<'_, H>) -> AnyElement {
        self.child
    }
}

/// Optional layout-only anchor for advanced tooltip placement recipes.
///
/// Use [`Tooltip::anchor_element`] to wire the anchor element ID into placement.
#[derive(Debug, Clone)]
pub struct TooltipAnchor {
    child: AnyElement,
}

impl TooltipAnchor {
    pub fn new(child: AnyElement) -> Self {
        Self { child }
    }

    pub fn element_id(&self) -> fret_ui::elements::GlobalElementId {
        self.child.id
    }

    pub fn into_element<H: UiHost>(self, _cx: &mut ElementContext<'_, H>) -> AnyElement {
        self.child
    }
}

/// shadcn/ui `TooltipContent` (v4).
#[derive(Debug, Clone)]
pub struct TooltipContent {
    children: Vec<AnyElement>,
    chrome: ChromeRefinement,
    layout: LayoutRefinement,
}

impl TooltipContent {
    pub fn new(children: impl IntoIterator<Item = AnyElement>) -> Self {
        let children = children.into_iter().collect();
        Self {
            children,
            chrome: ChromeRefinement::default(),
            layout: LayoutRefinement::default(),
        }
    }

    pub fn text<H: UiHost>(
        cx: &mut ElementContext<'_, H>,
        text: impl Into<Arc<str>>,
    ) -> AnyElement {
        let theme = Theme::global(&*cx.app).clone();
        let text = text.into();

        let text_style = tooltip_text_style(&theme);
        let fg = tooltip_text_fg(&theme);

        ui::text(cx, text)
            .text_size_px(text_style.size)
            .line_height_px(
                text_style
                    .line_height
                    .unwrap_or_else(|| theme.metric_required("font.line_height")),
            )
            .font_weight(text_style.weight)
            .wrap(TextWrap::Word)
            .overflow(TextOverflow::Clip)
            .text_color(ColorRef::Color(fg))
            .into_element(cx)
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

        let base_layout = LayoutRefinement::default().flex_shrink_0();
        let chrome = tooltip_content_chrome(&theme).merge(self.chrome);
        let props = decl_style::container_props(&theme, chrome, base_layout.merge(self.layout));
        let fg = tooltip_text_fg(&theme);
        let children = self
            .children
            .into_iter()
            .map(|child| apply_tooltip_inherited_fg(child, fg))
            .collect();
        let container = shadcn_layout::container_flow(cx, props, children);
        cx.semantics(
            SemanticsProps {
                role: fret_core::SemanticsRole::Tooltip,
                ..Default::default()
            },
            move |_cx| vec![container],
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::Cell;
    use std::rc::Rc;

    use fret_app::App;
    use fret_core::{
        AppWindowId, PathCommand, PathConstraints, PathId, PathMetrics, PathService, PathStyle,
        Point, Px, Rect, SemanticsRole, Size as CoreSize, SvgId, SvgService, TextBlobId,
        TextConstraints, TextMetrics, TextService, TextStyle as CoreTextStyle,
    };
    use fret_runtime::{FrameId, TickId};
    use fret_ui::element::{
        ContainerProps, FlexProps, LayoutStyle, Length, PressableA11y, PressableProps,
        SemanticsProps, TextProps,
    };
    use fret_ui::overlay_placement::{Align, Side, anchored_panel_bounds_sized};
    use fret_ui::tree::UiTree;
    use fret_ui_kit::OverlayController;

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

    #[test]
    fn tooltip_content_applies_default_fg_to_descendant_text() {
        use fret_core::Color;
        use fret_ui::element::ElementKind;
        use fret_ui::elements::GlobalElementId;

        let fg = Color {
            r: 0.1,
            g: 0.2,
            b: 0.3,
            a: 0.4,
        };

        let text_no_color = AnyElement::new(
            GlobalElementId(1),
            ElementKind::Text(TextProps::new("tip")),
            Vec::new(),
        );
        let mut text_with_color = TextProps::new("fixed");
        text_with_color.color = Some(Color {
            r: 0.9,
            g: 0.8,
            b: 0.7,
            a: 1.0,
        });
        let text_with_color = AnyElement::new(
            GlobalElementId(2),
            ElementKind::Text(text_with_color),
            Vec::new(),
        );

        let root = AnyElement::new(
            GlobalElementId(3),
            ElementKind::Container(ContainerProps::default()),
            vec![text_no_color, text_with_color],
        );

        let out = apply_tooltip_inherited_fg(root, fg);
        let colors: Vec<Option<Color>> = out
            .children
            .iter()
            .map(|child| match &child.kind {
                ElementKind::Text(t) => t.color,
                _ => None,
            })
            .collect();

        assert_eq!(colors.len(), 2);
        assert_eq!(colors[0], Some(fg));
        assert_ne!(colors[1], Some(fg));
    }

    fn render_tooltip_frame(
        ui: &mut UiTree<App>,
        app: &mut App,
        services: &mut dyn fret_core::UiServices,
        window: AppWindowId,
        bounds: Rect,
        trigger_id_out: Rc<Cell<Option<fret_ui::elements::GlobalElementId>>>,
        content_id_out: Rc<Cell<Option<fret_ui::elements::GlobalElementId>>>,
    ) {
        OverlayController::begin_frame(app, window);

        let root =
            fret_ui::declarative::render_root(ui, app, services, window, bounds, "test", |cx| {
                let trigger = cx.pressable_with_id(
                    PressableProps {
                        layout: {
                            let mut layout = LayoutStyle::default();
                            layout.size.width = Length::Px(Px(120.0));
                            layout.size.height = Length::Px(Px(40.0));
                            layout
                        },
                        enabled: true,
                        focusable: true,
                        a11y: PressableA11y {
                            role: Some(SemanticsRole::Button),
                            label: Some(Arc::from("trigger")),
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                    |cx, _st, id| {
                        trigger_id_out.set(Some(id));
                        vec![cx.container(ContainerProps::default(), |_cx| Vec::new())]
                    },
                );

                let content = TooltipContent::new(vec![ui::raw_text(cx, "tip").into_element(cx)])
                    .into_element(cx);
                content_id_out.set(Some(content.id));

                vec![
                    Tooltip::new(trigger, content)
                        .open_delay_frames(30)
                        .close_delay_frames(30)
                        .into_element(cx),
                ]
            });

        ui.set_root(root);
        OverlayController::render(ui, app, services, window, bounds);
    }

    #[test]
    fn tooltip_opens_on_keyboard_focus_without_hover() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let trigger_id: Rc<Cell<Option<fret_ui::elements::GlobalElementId>>> =
            Rc::new(Cell::new(None));
        let content_id: Rc<Cell<Option<fret_ui::elements::GlobalElementId>>> =
            Rc::new(Cell::new(None));

        let mut services = FakeServices;
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            CoreSize::new(Px(800.0), Px(600.0)),
        );

        // Frame 1: establish element->node mappings.
        app.set_frame_id(FrameId(1));
        render_tooltip_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            trigger_id.clone(),
            content_id.clone(),
        );
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        let trigger_element = trigger_id.get().expect("trigger element id");
        let trigger_node = fret_ui::elements::node_for_element(&mut app, window, trigger_element)
            .expect("trigger node");
        ui.set_focus(Some(trigger_node));

        // Frame 2: focus should cause the tooltip overlay to be requested and mounted.
        app.set_frame_id(FrameId(2));
        render_tooltip_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            trigger_id.clone(),
            content_id.clone(),
        );
        ui.request_semantics_snapshot();
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        let content_element = content_id.get().expect("content element id");
        let content_node = fret_ui::elements::node_for_element(&mut app, window, content_element);
        assert!(
            content_node.is_some(),
            "expected tooltip content to be mounted when focused"
        );

        let snap = ui.semantics_snapshot().expect("semantics snapshot");
        let content_node = content_node.expect("content node");

        let trigger_node = snap
            .nodes
            .iter()
            .find(|n| n.label.as_deref() == Some("trigger"))
            .expect("trigger semantics node");
        let tooltip_node = snap
            .nodes
            .iter()
            .find(|n| n.id == content_node)
            .expect("tooltip semantics node");

        assert_eq!(tooltip_node.role, SemanticsRole::Tooltip);
        assert!(
            trigger_node
                .described_by
                .iter()
                .any(|id| *id == tooltip_node.id),
            "trigger should be described by the tooltip content"
        );
    }

    #[test]
    fn tooltip_opens_after_delay_and_closes_after_close_delay() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let trigger_id: Rc<Cell<Option<fret_ui::elements::GlobalElementId>>> =
            Rc::new(Cell::new(None));
        let content_id: Rc<Cell<Option<fret_ui::elements::GlobalElementId>>> =
            Rc::new(Cell::new(None));

        let mut services = FakeServices;
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            CoreSize::new(Px(800.0), Px(600.0)),
        );

        fn render_frame(
            ui: &mut UiTree<App>,
            app: &mut App,
            services: &mut dyn fret_core::UiServices,
            window: AppWindowId,
            bounds: Rect,
            trigger_id_out: Rc<Cell<Option<fret_ui::elements::GlobalElementId>>>,
            content_id_out: Rc<Cell<Option<fret_ui::elements::GlobalElementId>>>,
        ) {
            OverlayController::begin_frame(app, window);
            let root = fret_ui::declarative::render_root(
                ui,
                app,
                services,
                window,
                bounds,
                "test",
                |cx| {
                    TooltipProvider::new()
                        .delay_duration_frames(1)
                        .skip_delay_duration_frames(0)
                        .with(cx, |cx| {
                            let trigger = cx.pressable_with_id(
                                PressableProps {
                                    layout: {
                                        let mut layout = LayoutStyle::default();
                                        layout.size.width = Length::Px(Px(120.0));
                                        layout.size.height = Length::Px(Px(40.0));
                                        layout
                                    },
                                    enabled: true,
                                    focusable: true,
                                    a11y: PressableA11y {
                                        role: Some(SemanticsRole::Button),
                                        label: Some(Arc::from("trigger")),
                                        ..Default::default()
                                    },
                                    ..Default::default()
                                },
                                |cx, _st, id| {
                                    trigger_id_out.set(Some(id));
                                    vec![cx.container(ContainerProps::default(), |_cx| Vec::new())]
                                },
                            );

                            let content =
                                TooltipContent::new(vec![ui::raw_text(cx, "tip").into_element(cx)])
                                    .into_element(cx);
                            content_id_out.set(Some(content.id));

                            vec![
                                Tooltip::new(trigger, content)
                                    .close_delay_frames(2)
                                    .into_element(cx),
                            ]
                        })
                },
            );
            ui.set_root(root);
            OverlayController::render(ui, app, services, window, bounds);
        }

        // Frame 1: build and establish mappings.
        app.set_frame_id(FrameId(1));
        render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            trigger_id.clone(),
            content_id.clone(),
        );
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        // Ensure pointer starts outside.
        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::Pointer(fret_core::PointerEvent::Move {
                pointer_id: fret_core::PointerId(0),
                position: Point::new(Px(200.0), Px(200.0)),
                buttons: fret_core::MouseButtons::default(),
                modifiers: fret_core::Modifiers::default(),
                pointer_type: fret_core::PointerType::Mouse,
            }),
        );

        // Hover trigger.
        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::Pointer(fret_core::PointerEvent::Move {
                pointer_id: fret_core::PointerId(0),
                position: Point::new(Px(10.0), Px(10.0)),
                buttons: fret_core::MouseButtons::default(),
                modifiers: fret_core::Modifiers::default(),
                pointer_type: fret_core::PointerType::Mouse,
            }),
        );

        // Frame 2: hovered, but delay not yet elapsed.
        app.set_frame_id(FrameId(2));
        render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            trigger_id.clone(),
            content_id.clone(),
        );
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        let content_element = content_id.get().expect("content element id");
        assert!(
            fret_ui::elements::node_for_element(&mut app, window, content_element).is_none(),
            "expected tooltip to still be closed before delay elapses"
        );

        // Frame 3: delay elapsed -> open.
        app.set_frame_id(FrameId(3));
        render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            trigger_id.clone(),
            content_id.clone(),
        );
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        let content_node = fret_ui::elements::node_for_element(&mut app, window, content_element)
            .expect("expected tooltip content node to exist after delay elapses");
        let tooltip_layer_root = *ui
            .debug_node_path(content_node)
            .first()
            .expect("tooltip node path root");
        assert!(
            ui.debug_layers_in_paint_order()
                .iter()
                .find(|layer| layer.root == tooltip_layer_root)
                .is_some_and(|layer| layer.visible),
            "expected tooltip layer to be visible after delay elapses"
        );

        // Leave hover.
        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::Pointer(fret_core::PointerEvent::Move {
                pointer_id: fret_core::PointerId(0),
                position: Point::new(Px(200.0), Px(200.0)),
                buttons: fret_core::MouseButtons::default(),
                modifiers: fret_core::Modifiers::default(),
                pointer_type: fret_core::PointerType::Mouse,
            }),
        );

        // Frame 4/5: close delay not yet elapsed -> still open.
        for frame in 4..=5 {
            app.set_frame_id(FrameId(frame));
            render_frame(
                &mut ui,
                &mut app,
                &mut services,
                window,
                bounds,
                trigger_id.clone(),
                content_id.clone(),
            );
            ui.layout_all(&mut app, &mut services, bounds, 1.0);
            assert!(
                fret_ui::elements::node_for_element(&mut app, window, content_element).is_some(),
                "expected tooltip to remain mounted during close delay"
            );
            assert!(
                ui.debug_layers_in_paint_order()
                    .iter()
                    .find(|layer| layer.root == tooltip_layer_root)
                    .is_some_and(|layer| layer.visible),
                "expected tooltip layer to remain visible during close delay"
            );
        }

        // Frame 6: close delay elapsed -> closed.
        app.set_frame_id(FrameId(6));
        render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            trigger_id.clone(),
            content_id.clone(),
        );
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        // Closing begins after the close delay, but we keep the tooltip mounted during the fade-out
        // transition (Radix Presence-style behavior).
        assert!(
            fret_ui::elements::node_for_element(&mut app, window, content_element).is_some(),
            "expected tooltip to remain mounted during fade-out"
        );
        assert!(
            ui.debug_layers_in_paint_order()
                .iter()
                .find(|layer| layer.root == tooltip_layer_root)
                .is_some_and(|layer| layer.visible),
            "expected tooltip layer to remain visible during fade-out"
        );

        // Close delay elapsed on frame 6, then Presence keeps the layer mounted while fading out.
        // Assert that it becomes hidden by the end of the fade-out window.
        let settle_frame = 6 + crate::overlay_motion::SHADCN_MOTION_TICKS_100 + 1;
        for frame in 7..=settle_frame {
            app.set_frame_id(FrameId(frame));
            render_frame(
                &mut ui,
                &mut app,
                &mut services,
                window,
                bounds,
                trigger_id.clone(),
                content_id.clone(),
            );
            ui.layout_all(&mut app, &mut services, bounds, 1.0);
        }

        let tooltip_layer = ui
            .debug_layers_in_paint_order()
            .into_iter()
            .find(|layer| layer.root == tooltip_layer_root);
        assert!(
            tooltip_layer.is_none_or(|layer| !layer.visible),
            "expected tooltip layer to be hidden after close delay + fade-out elapses"
        );
    }

    #[test]
    fn tooltip_closes_on_pointer_down_and_does_not_reopen_until_hover_leave() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let trigger_id: Rc<Cell<Option<fret_ui::elements::GlobalElementId>>> =
            Rc::new(Cell::new(None));
        let content_id: Rc<Cell<Option<fret_ui::elements::GlobalElementId>>> =
            Rc::new(Cell::new(None));

        let mut services = FakeServices;
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            CoreSize::new(Px(800.0), Px(600.0)),
        );

        fn render_frame(
            ui: &mut UiTree<App>,
            app: &mut App,
            services: &mut dyn fret_core::UiServices,
            window: AppWindowId,
            bounds: Rect,
            trigger_id_out: Rc<Cell<Option<fret_ui::elements::GlobalElementId>>>,
            content_id_out: Rc<Cell<Option<fret_ui::elements::GlobalElementId>>>,
        ) {
            OverlayController::begin_frame(app, window);
            let root = fret_ui::declarative::render_root(
                ui,
                app,
                services,
                window,
                bounds,
                "test",
                |cx| {
                    let trigger = cx.pressable_with_id(
                        PressableProps {
                            layout: {
                                let mut layout = LayoutStyle::default();
                                layout.size.width = Length::Px(Px(120.0));
                                layout.size.height = Length::Px(Px(40.0));
                                layout
                            },
                            enabled: true,
                            focusable: true,
                            a11y: PressableA11y {
                                role: Some(SemanticsRole::Button),
                                label: Some(Arc::from("trigger")),
                                ..Default::default()
                            },
                            ..Default::default()
                        },
                        |cx, _st, id| {
                            trigger_id_out.set(Some(id));
                            vec![cx.container(ContainerProps::default(), |_cx| Vec::new())]
                        },
                    );

                    let content =
                        TooltipContent::new(vec![ui::raw_text(cx, "tip").into_element(cx)])
                            .into_element(cx);
                    content_id_out.set(Some(content.id));

                    vec![
                        Tooltip::new(trigger, content)
                            .open_delay_frames(0)
                            .close_delay_frames(0)
                            .disable_hoverable_content(false)
                            .into_element(cx),
                    ]
                },
            );
            ui.set_root(root);
            OverlayController::render(ui, app, services, window, bounds);
        }

        // Frame 1: establish trigger bounds.
        app.set_frame_id(FrameId(1));
        render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            trigger_id.clone(),
            content_id.clone(),
        );
        ui.request_semantics_snapshot();
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        let snap = ui.semantics_snapshot().expect("semantics snapshot");
        let trigger_node = snap
            .nodes
            .iter()
            .find(|n| n.label.as_deref() == Some("trigger"))
            .expect("trigger node");
        let trigger_bounds = trigger_node.bounds;
        let trigger_center = Point::new(
            Px(trigger_bounds.origin.x.0 + trigger_bounds.size.width.0 * 0.5),
            Px(trigger_bounds.origin.y.0 + trigger_bounds.size.height.0 * 0.5),
        );

        // Hover trigger (pointermove gating should allow open after this move).
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

        // Frame 2: tooltip opens immediately.
        app.set_frame_id(FrameId(2));
        render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            trigger_id.clone(),
            content_id.clone(),
        );
        ui.request_semantics_snapshot();
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        let content_element = content_id.get().expect("content element id");
        assert!(
            fret_ui::elements::node_for_element(&mut app, window, content_element).is_some(),
            "expected tooltip content to be mounted after hover"
        );

        // Pointer down should close (and suppress focus-driven reopen).
        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::Pointer(fret_core::PointerEvent::Down {
                pointer_id: fret_core::PointerId(0),
                position: trigger_center,
                button: fret_core::MouseButton::Left,
                modifiers: fret_core::Modifiers::default(),
                pointer_type: fret_core::PointerType::Mouse,
                click_count: 1,
            }),
        );
        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::Pointer(fret_core::PointerEvent::Up {
                pointer_id: fret_core::PointerId(0),
                position: trigger_center,
                button: fret_core::MouseButton::Left,
                modifiers: fret_core::Modifiers::default(),
                is_click: true,
                pointer_type: fret_core::PointerType::Mouse,
                click_count: 1,
            }),
        );

        // Frame 3: described-by should be cleared.
        app.set_frame_id(FrameId(3));
        render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            trigger_id.clone(),
            content_id.clone(),
        );
        ui.request_semantics_snapshot();
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        let snap = ui.semantics_snapshot().expect("semantics snapshot");
        let trigger_node = snap
            .nodes
            .iter()
            .find(|n| n.label.as_deref() == Some("trigger"))
            .expect("trigger node");
        let content_node = fret_ui::elements::node_for_element(&mut app, window, content_element);
        if let Some(content_node) = content_node {
            assert!(
                !trigger_node
                    .described_by
                    .iter()
                    .any(|id| *id == content_node),
                "expected aria-describedby to be cleared after pointerdown close"
            );
        } else {
            assert!(
                trigger_node.described_by.is_empty(),
                "expected aria-describedby to be cleared after pointerdown close"
            );
        }

        // Frame 4: should stay closed even if focus remains on the trigger.
        app.set_frame_id(FrameId(4));
        render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            trigger_id.clone(),
            content_id.clone(),
        );
        ui.request_semantics_snapshot();
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        let snap = ui.semantics_snapshot().expect("semantics snapshot");
        let trigger_node = snap
            .nodes
            .iter()
            .find(|n| n.label.as_deref() == Some("trigger"))
            .expect("trigger node");
        assert!(
            trigger_node.described_by.is_empty(),
            "expected tooltip to remain closed while focused after pointerdown"
        );

        // Leave hover to reset trigger gates, then re-hover to open again.
        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::Pointer(fret_core::PointerEvent::Move {
                pointer_id: fret_core::PointerId(0),
                position: Point::new(Px(200.0), Px(200.0)),
                buttons: fret_core::MouseButtons::default(),
                modifiers: fret_core::Modifiers::default(),
                pointer_type: fret_core::PointerType::Mouse,
            }),
        );

        app.set_frame_id(FrameId(5));
        render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            trigger_id.clone(),
            content_id.clone(),
        );
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

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

        app.set_frame_id(FrameId(6));
        render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            trigger_id.clone(),
            content_id.clone(),
        );
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        assert!(
            fret_ui::elements::node_for_element(&mut app, window, content_element).is_some(),
            "expected tooltip to reopen after leaving and re-hovering"
        );
    }

    #[test]
    fn tooltip_provider_skips_delay_after_recent_close() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        // This test "fast-forwards" by jumping `App::frame_id` without rendering intermediate
        // frames. Keep element state alive across that jump so we can validate provider delay
        // semantics rather than `ElementRuntime` GC behavior.
        app.with_global_mut(fret_ui::elements::ElementRuntime::new, |rt, _app| {
            rt.set_gc_lag_frames(128);
        });

        let content_1_id: Rc<Cell<Option<fret_ui::elements::GlobalElementId>>> =
            Rc::new(Cell::new(None));
        let content_2_id: Rc<Cell<Option<fret_ui::elements::GlobalElementId>>> =
            Rc::new(Cell::new(None));

        let mut services = FakeServices;
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            CoreSize::new(Px(800.0), Px(600.0)),
        );

        fn render_frame(
            ui: &mut UiTree<App>,
            app: &mut App,
            services: &mut dyn fret_core::UiServices,
            window: AppWindowId,
            bounds: Rect,
            content_1_id_out: Rc<Cell<Option<fret_ui::elements::GlobalElementId>>>,
            content_2_id_out: Rc<Cell<Option<fret_ui::elements::GlobalElementId>>>,
        ) {
            OverlayController::begin_frame(app, window);

            let root =
                fret_ui::declarative::render_root(
                    ui,
                    app,
                    services,
                    window,
                    bounds,
                    "test",
                    |cx| {
                        TooltipProvider::new()
                            .delay_duration_frames(10)
                            .skip_delay_duration_frames(30)
                            .with(cx, |cx| {
                                vec![cx.column(fret_ui::element::ColumnProps::default(), |cx| {
                                    let trigger_1 = cx.pressable_with_id(
                                        PressableProps {
                                            layout: {
                                                let mut layout = LayoutStyle::default();
                                                layout.size.width = Length::Px(Px(120.0));
                                                layout.size.height = Length::Px(Px(40.0));
                                                layout
                                            },
                                            enabled: true,
                                            focusable: true,
                                            a11y: PressableA11y {
                                                role: Some(SemanticsRole::Button),
                                                label: Some(Arc::from("trigger_1")),
                                                ..Default::default()
                                            },
                                            ..Default::default()
                                        },
                                        |cx, _st, _id| {
                                            vec![cx.container(ContainerProps::default(), |_cx| {
                                                Vec::new()
                                            })]
                                        },
                                    );

                                    let trigger_2 = cx.pressable_with_id(
                                        PressableProps {
                                            layout: {
                                                let mut layout = LayoutStyle::default();
                                                layout.size.width = Length::Px(Px(120.0));
                                                layout.size.height = Length::Px(Px(40.0));
                                                layout
                                            },
                                            enabled: true,
                                            focusable: true,
                                            a11y: PressableA11y {
                                                role: Some(SemanticsRole::Button),
                                                label: Some(Arc::from("trigger_2")),
                                                ..Default::default()
                                            },
                                            ..Default::default()
                                        },
                                        |cx, _st, _id| {
                                            vec![cx.container(ContainerProps::default(), |_cx| {
                                                Vec::new()
                                            })]
                                        },
                                    );

                                    let content_1 = TooltipContent::new(vec![
                                        ui::raw_text(cx, "tip_1").into_element(cx),
                                    ])
                                    .into_element(cx);
                                    content_1_id_out.set(Some(content_1.id));

                                    let content_2 = TooltipContent::new(vec![
                                        ui::raw_text(cx, "tip_2").into_element(cx),
                                    ])
                                    .into_element(cx);
                                    content_2_id_out.set(Some(content_2.id));

                                    vec![
                                        Tooltip::new(trigger_1, content_1).into_element(cx),
                                        Tooltip::new(trigger_2, content_2).into_element(cx),
                                    ]
                                })]
                            })
                    },
                );

            ui.set_root(root);
            OverlayController::render(ui, app, services, window, bounds);
        }

        // Frame 1: build.
        app.set_frame_id(FrameId(1));
        render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            content_1_id.clone(),
            content_2_id.clone(),
        );
        ui.request_semantics_snapshot();
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        let snap = ui.semantics_snapshot().expect("semantics snapshot");
        let trigger_1 = snap
            .nodes
            .iter()
            .find(|n| n.label.as_deref() == Some("trigger_1"))
            .expect("trigger_1 node");
        let trigger_2 = snap
            .nodes
            .iter()
            .find(|n| n.label.as_deref() == Some("trigger_2"))
            .expect("trigger_2 node");

        let trigger_1_node = trigger_1.id;
        let trigger_1_bounds = trigger_1.bounds;
        let trigger_2_bounds = trigger_2.bounds;

        let trigger_1_point = Point::new(
            Px(trigger_1_bounds.origin.x.0 + trigger_1_bounds.size.width.0 * 0.5),
            Px(trigger_1_bounds.origin.y.0 + trigger_1_bounds.size.height.0 * 0.5),
        );
        let trigger_2_point = Point::new(
            Px(trigger_2_bounds.origin.x.0 + trigger_2_bounds.size.width.0 * 0.5),
            Px(trigger_2_bounds.origin.y.0 + trigger_2_bounds.size.height.0 * 0.5),
        );

        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::Pointer(fret_core::PointerEvent::Move {
                pointer_id: fret_core::PointerId(0),
                position: trigger_1_point,
                buttons: fret_core::MouseButtons::default(),
                modifiers: fret_core::Modifiers::default(),
                pointer_type: fret_core::PointerType::Mouse,
            }),
        );

        // Frame 2: focus should open tooltip 1 immediately (regardless of provider delay).
        ui.set_focus(Some(trigger_1_node));

        app.set_frame_id(FrameId(2));
        render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            content_1_id.clone(),
            content_2_id.clone(),
        );
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        let content_1_element = content_1_id.get().expect("content_1 element id");
        assert!(
            fret_ui::elements::node_for_element(&mut app, window, content_1_element).is_some(),
            "expected tooltip 1 to be open when focused"
        );

        // Blur + move to trigger 2, then render: provider should skip delay for the new tooltip.
        ui.set_focus(None);
        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::Pointer(fret_core::PointerEvent::Move {
                pointer_id: fret_core::PointerId(0),
                position: trigger_2_point,
                buttons: fret_core::MouseButtons::default(),
                modifiers: fret_core::Modifiers::default(),
                pointer_type: fret_core::PointerType::Mouse,
            }),
        );

        app.set_frame_id(FrameId(3));
        render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            content_1_id.clone(),
            content_2_id.clone(),
        );
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        let content_2_element = content_2_id.get().expect("content_2 element id");
        assert!(
            fret_ui::elements::node_for_element(&mut app, window, content_2_element).is_some(),
            "expected tooltip 2 to open without delay under the provider skip window"
        );
    }

    #[test]
    fn tooltip_opening_another_tooltip_closes_previous() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let content_1_id: Rc<Cell<Option<fret_ui::elements::GlobalElementId>>> =
            Rc::new(Cell::new(None));
        let content_2_id: Rc<Cell<Option<fret_ui::elements::GlobalElementId>>> =
            Rc::new(Cell::new(None));

        let mut services = FakeServices;
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            CoreSize::new(Px(800.0), Px(600.0)),
        );

        fn render_frame(
            ui: &mut UiTree<App>,
            app: &mut App,
            services: &mut dyn fret_core::UiServices,
            window: AppWindowId,
            bounds: Rect,
            content_1_id_out: Rc<Cell<Option<fret_ui::elements::GlobalElementId>>>,
            content_2_id_out: Rc<Cell<Option<fret_ui::elements::GlobalElementId>>>,
        ) {
            OverlayController::begin_frame(app, window);

            let root =
                fret_ui::declarative::render_root(
                    ui,
                    app,
                    services,
                    window,
                    bounds,
                    "test",
                    |cx| {
                        TooltipProvider::new()
                            .delay_duration_frames(0)
                            .skip_delay_duration_frames(0)
                            .with(cx, |cx| {
                                vec![cx.column(fret_ui::element::ColumnProps::default(), |cx| {
                                    let trigger_1 = cx.pressable_with_id(
                                        PressableProps {
                                            layout: {
                                                let mut layout = LayoutStyle::default();
                                                layout.size.width = Length::Px(Px(120.0));
                                                layout.size.height = Length::Px(Px(40.0));
                                                layout
                                            },
                                            enabled: true,
                                            focusable: true,
                                            a11y: PressableA11y {
                                                role: Some(SemanticsRole::Button),
                                                label: Some(Arc::from("trigger_1")),
                                                ..Default::default()
                                            },
                                            ..Default::default()
                                        },
                                        |cx, _st, _id| {
                                            vec![cx.container(ContainerProps::default(), |_cx| {
                                                Vec::new()
                                            })]
                                        },
                                    );
                                    let content_1 = TooltipContent::new(vec![
                                        ui::raw_text(cx, "tip1").into_element(cx),
                                    ])
                                    .into_element(cx);
                                    content_1_id_out.set(Some(content_1.id));

                                    let trigger_2 = cx.pressable_with_id(
                                        PressableProps {
                                            layout: {
                                                let mut layout = LayoutStyle::default();
                                                layout.size.width = Length::Px(Px(120.0));
                                                layout.size.height = Length::Px(Px(40.0));
                                                layout
                                            },
                                            enabled: true,
                                            focusable: true,
                                            a11y: PressableA11y {
                                                role: Some(SemanticsRole::Button),
                                                label: Some(Arc::from("trigger_2")),
                                                ..Default::default()
                                            },
                                            ..Default::default()
                                        },
                                        |cx, _st, _id| {
                                            vec![cx.container(ContainerProps::default(), |_cx| {
                                                Vec::new()
                                            })]
                                        },
                                    );
                                    let content_2 = TooltipContent::new(vec![
                                        ui::raw_text(cx, "tip2").into_element(cx),
                                    ])
                                    .into_element(cx);
                                    content_2_id_out.set(Some(content_2.id));

                                    vec![
                                        Tooltip::new(trigger_1, content_1)
                                            .open_delay_frames(0)
                                            .close_delay_frames(100)
                                            .disable_hoverable_content(false)
                                            .into_element(cx),
                                        Tooltip::new(trigger_2, content_2)
                                            .open_delay_frames(0)
                                            .close_delay_frames(0)
                                            .disable_hoverable_content(false)
                                            .into_element(cx),
                                    ]
                                })]
                            })
                    },
                );
            ui.set_root(root);
            OverlayController::render(ui, app, services, window, bounds);
        }

        // Frame 1: mount and snapshot trigger bounds.
        app.set_frame_id(FrameId(1));
        render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            content_1_id.clone(),
            content_2_id.clone(),
        );
        ui.request_semantics_snapshot();
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        let snap = ui.semantics_snapshot().expect("semantics snapshot");
        let trigger_1 = snap
            .nodes
            .iter()
            .find(|n| n.label.as_deref() == Some("trigger_1"))
            .expect("trigger_1 node");
        let trigger_2 = snap
            .nodes
            .iter()
            .find(|n| n.label.as_deref() == Some("trigger_2"))
            .expect("trigger_2 node");

        let trigger_1_bounds = trigger_1.bounds;
        let trigger_2_bounds = trigger_2.bounds;
        let trigger_1_point = Point::new(
            Px(trigger_1_bounds.origin.x.0 + trigger_1_bounds.size.width.0 * 0.5),
            Px(trigger_1_bounds.origin.y.0 + trigger_1_bounds.size.height.0 * 0.5),
        );
        let trigger_2_point = Point::new(
            Px(trigger_2_bounds.origin.x.0 + trigger_2_bounds.size.width.0 * 0.5),
            Px(trigger_2_bounds.origin.y.0 + trigger_2_bounds.size.height.0 * 0.5),
        );

        // Hover trigger 1.
        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::Pointer(fret_core::PointerEvent::Move {
                pointer_id: fret_core::PointerId(0),
                position: trigger_1_point,
                buttons: fret_core::MouseButtons::default(),
                modifiers: fret_core::Modifiers::default(),
                pointer_type: fret_core::PointerType::Mouse,
            }),
        );

        // Frame 2: tooltip 1 opens immediately.
        app.set_frame_id(FrameId(2));
        render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            content_1_id.clone(),
            content_2_id.clone(),
        );
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        let content_1_element = content_1_id.get().expect("content_1 element id");
        assert!(
            fret_ui::elements::node_for_element(&mut app, window, content_1_element).is_some(),
            "expected tooltip 1 to be open after hovering trigger_1"
        );

        // Move to trigger 2 (tooltip 1 would normally remain open due to close delay).
        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::Pointer(fret_core::PointerEvent::Move {
                pointer_id: fret_core::PointerId(0),
                position: trigger_2_point,
                buttons: fret_core::MouseButtons::default(),
                modifiers: fret_core::Modifiers::default(),
                pointer_type: fret_core::PointerType::Mouse,
            }),
        );

        // Frame 3: tooltip 2 opens immediately; tooltip 1 is still open (close delay).
        app.set_frame_id(FrameId(3));
        render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            content_1_id.clone(),
            content_2_id.clone(),
        );
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        let content_2_element = content_2_id.get().expect("content_2 element id");
        assert!(
            fret_ui::elements::node_for_element(&mut app, window, content_2_element).is_some(),
            "expected tooltip 2 to open after hovering trigger_2"
        );
        assert!(
            fret_ui::elements::node_for_element(&mut app, window, content_1_element).is_some(),
            "expected tooltip 1 to still be mounted due to close delay"
        );

        // Frame 4: tooltip 1 should close because tooltip 2 opened (Radix TOOLTIP_OPEN outcome).
        app.set_frame_id(FrameId(4));
        render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            content_1_id.clone(),
            content_2_id.clone(),
        );
        ui.request_semantics_snapshot();
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        let content_1_node =
            fret_ui::elements::node_for_element(&mut app, window, content_1_element);
        let snap = ui.semantics_snapshot().expect("semantics snapshot");
        let trigger_1 = snap
            .nodes
            .iter()
            .find(|n| n.label.as_deref() == Some("trigger_1"))
            .expect("trigger_1 node");

        if let Some(content_1_node) = content_1_node {
            assert!(
                !trigger_1
                    .described_by
                    .iter()
                    .any(|id| *id == content_1_node),
                "expected tooltip 1 to close (aria-describedby cleared) after tooltip 2 opens"
            );
        } else {
            assert!(
                trigger_1.described_by.is_empty(),
                "expected tooltip 1 to close (aria-describedby cleared) after tooltip 2 opens"
            );
        }
    }

    #[test]
    fn tooltip_closes_on_outside_press() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let trigger_id: Rc<Cell<Option<fret_ui::elements::GlobalElementId>>> =
            Rc::new(Cell::new(None));
        let content_id: Rc<Cell<Option<fret_ui::elements::GlobalElementId>>> =
            Rc::new(Cell::new(None));

        let mut services = FakeServices;
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            CoreSize::new(Px(800.0), Px(600.0)),
        );

        fn center(rect: Rect) -> Point {
            Point::new(
                Px(rect.origin.x.0 + rect.size.width.0 * 0.5),
                Px(rect.origin.y.0 + rect.size.height.0 * 0.5),
            )
        }

        fn render_frame(
            ui: &mut UiTree<App>,
            app: &mut App,
            services: &mut dyn fret_core::UiServices,
            window: AppWindowId,
            bounds: Rect,
            trigger_id_out: Rc<Cell<Option<fret_ui::elements::GlobalElementId>>>,
            content_id_out: Rc<Cell<Option<fret_ui::elements::GlobalElementId>>>,
        ) {
            OverlayController::begin_frame(app, window);
            let root = fret_ui::declarative::render_root(
                ui,
                app,
                services,
                window,
                bounds,
                "test",
                |cx| {
                    TooltipProvider::new()
                        .delay_duration_frames(0)
                        .skip_delay_duration_frames(0)
                        .with(cx, |cx| {
                            let trigger = cx.pressable_with_id(
                                PressableProps {
                                    layout: {
                                        let mut layout = LayoutStyle::default();
                                        layout.size.width = Length::Px(Px(120.0));
                                        layout.size.height = Length::Px(Px(40.0));
                                        layout
                                    },
                                    enabled: true,
                                    focusable: true,
                                    a11y: PressableA11y {
                                        role: Some(SemanticsRole::Button),
                                        label: Some(Arc::from("trigger")),
                                        ..Default::default()
                                    },
                                    ..Default::default()
                                },
                                |cx, _st, id| {
                                    trigger_id_out.set(Some(id));
                                    vec![cx.container(ContainerProps::default(), |_cx| Vec::new())]
                                },
                            );

                            let content =
                                TooltipContent::new(vec![ui::raw_text(cx, "tip").into_element(cx)])
                                    .into_element(cx);
                            content_id_out.set(Some(content.id));

                            vec![
                                Tooltip::new(trigger, content)
                                    .open_delay_frames(0)
                                    .close_delay_frames(0)
                                    .disable_hoverable_content(false)
                                    .into_element(cx),
                            ]
                        })
                },
            );
            ui.set_root(root);
            OverlayController::render(ui, app, services, window, bounds);
        }

        // Frame 1: layout and read trigger bounds.
        app.set_frame_id(FrameId(1));
        render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            trigger_id.clone(),
            content_id.clone(),
        );
        ui.request_semantics_snapshot();
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        let snap = ui.semantics_snapshot().expect("semantics snapshot");
        let trigger_node = snap
            .nodes
            .iter()
            .find(|n| n.label.as_deref() == Some("trigger"))
            .expect("trigger node");
        let trigger_point = center(trigger_node.bounds);

        // Hover trigger to open tooltip.
        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::Pointer(fret_core::PointerEvent::Move {
                pointer_id: fret_core::PointerId(0),
                position: trigger_point,
                buttons: fret_core::MouseButtons::default(),
                modifiers: fret_core::Modifiers::default(),
                pointer_type: fret_core::PointerType::Mouse,
            }),
        );

        // Frame 2: tooltip should be open.
        app.set_frame_id(FrameId(2));
        render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            trigger_id.clone(),
            content_id.clone(),
        );
        ui.request_semantics_snapshot();
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        let content_element = content_id.get().expect("content element id");
        assert!(
            fret_ui::elements::node_for_element(&mut app, window, content_element).is_some(),
            "expected tooltip content to be mounted after opening"
        );

        // Outside press should close the tooltip (Radix DismissableLayer onDismiss outcome).
        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::Pointer(fret_core::PointerEvent::Down {
                pointer_id: fret_core::PointerId(0),
                position: Point::new(Px(700.0), Px(500.0)),
                button: fret_core::MouseButton::Left,
                modifiers: fret_core::Modifiers::default(),
                pointer_type: fret_core::PointerType::Mouse,
                click_count: 1,
            }),
        );
        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::Pointer(fret_core::PointerEvent::Up {
                pointer_id: fret_core::PointerId(0),
                position: Point::new(Px(700.0), Px(500.0)),
                button: fret_core::MouseButton::Left,
                modifiers: fret_core::Modifiers::default(),
                is_click: true,
                pointer_type: fret_core::PointerType::Mouse,
                click_count: 1,
            }),
        );

        // Frame 3: trigger described-by should be cleared (open=false).
        app.set_frame_id(FrameId(3));
        render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            trigger_id.clone(),
            content_id.clone(),
        );
        ui.request_semantics_snapshot();
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        let snap = ui.semantics_snapshot().expect("semantics snapshot");
        let trigger_node = snap
            .nodes
            .iter()
            .find(|n| n.label.as_deref() == Some("trigger"))
            .expect("trigger node");
        assert!(
            trigger_node.described_by.is_empty(),
            "expected aria-describedby to be cleared after outside press dismissal"
        );
    }

    #[test]
    fn tooltip_closes_on_escape_while_focused() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let trigger_id: Rc<Cell<Option<fret_ui::elements::GlobalElementId>>> =
            Rc::new(Cell::new(None));
        let content_id: Rc<Cell<Option<fret_ui::elements::GlobalElementId>>> =
            Rc::new(Cell::new(None));

        let mut services = FakeServices;
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            CoreSize::new(Px(800.0), Px(600.0)),
        );

        fn render_frame(
            ui: &mut UiTree<App>,
            app: &mut App,
            services: &mut dyn fret_core::UiServices,
            window: AppWindowId,
            bounds: Rect,
            trigger_id_out: Rc<Cell<Option<fret_ui::elements::GlobalElementId>>>,
            content_id_out: Rc<Cell<Option<fret_ui::elements::GlobalElementId>>>,
        ) {
            OverlayController::begin_frame(app, window);
            let root = fret_ui::declarative::render_root(
                ui,
                app,
                services,
                window,
                bounds,
                "test",
                |cx| {
                    TooltipProvider::new()
                        .delay_duration_frames(0)
                        .skip_delay_duration_frames(0)
                        .with(cx, |cx| {
                            let trigger = cx.pressable_with_id(
                                PressableProps {
                                    layout: {
                                        let mut layout = LayoutStyle::default();
                                        layout.size.width = Length::Px(Px(120.0));
                                        layout.size.height = Length::Px(Px(40.0));
                                        layout
                                    },
                                    enabled: true,
                                    focusable: true,
                                    a11y: PressableA11y {
                                        role: Some(SemanticsRole::Button),
                                        label: Some(Arc::from("trigger")),
                                        ..Default::default()
                                    },
                                    ..Default::default()
                                },
                                |cx, _st, id| {
                                    trigger_id_out.set(Some(id));
                                    vec![cx.container(ContainerProps::default(), |_cx| Vec::new())]
                                },
                            );

                            let content =
                                TooltipContent::new(vec![ui::raw_text(cx, "tip").into_element(cx)])
                                    .into_element(cx);
                            content_id_out.set(Some(content.id));

                            vec![
                                Tooltip::new(trigger, content)
                                    .open_delay_frames(0)
                                    .close_delay_frames(0)
                                    .disable_hoverable_content(false)
                                    .into_element(cx),
                            ]
                        })
                },
            );
            ui.set_root(root);
            OverlayController::render(ui, app, services, window, bounds);
        }

        // Frame 1: mount and focus the trigger.
        app.set_frame_id(FrameId(1));
        render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            trigger_id.clone(),
            content_id.clone(),
        );
        ui.request_semantics_snapshot();
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        let trigger_element = trigger_id.get().expect("trigger element id");
        let trigger_node =
            fret_ui::elements::node_for_element(&mut app, window, trigger_element).expect("node");
        ui.set_focus(Some(trigger_node));

        // Frame 2: focus opens tooltip.
        app.set_frame_id(FrameId(2));
        render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            trigger_id.clone(),
            content_id.clone(),
        );
        ui.request_semantics_snapshot();
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        let content_element = content_id.get().expect("content element id");
        assert!(
            fret_ui::elements::node_for_element(&mut app, window, content_element).is_some(),
            "expected tooltip content to be mounted after focus open"
        );

        // Escape closes and suppresses focus-driven reopen.
        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::KeyDown {
                key: fret_core::KeyCode::Escape,
                modifiers: fret_core::Modifiers::default(),
                repeat: false,
            },
        );

        // Frame 3: described-by should be cleared and tooltip should stay closed while focused.
        app.set_frame_id(FrameId(3));
        render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            trigger_id.clone(),
            content_id.clone(),
        );
        ui.request_semantics_snapshot();
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        let snap = ui.semantics_snapshot().expect("semantics snapshot");
        let trigger_node = snap
            .nodes
            .iter()
            .find(|n| n.label.as_deref() == Some("trigger"))
            .expect("trigger node");
        assert!(
            trigger_node.described_by.is_empty(),
            "expected aria-describedby to be cleared after Escape dismissal"
        );

        app.set_frame_id(FrameId(4));
        render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            trigger_id.clone(),
            content_id.clone(),
        );
        ui.request_semantics_snapshot();
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        let snap = ui.semantics_snapshot().expect("semantics snapshot");
        let trigger_node = snap
            .nodes
            .iter()
            .find(|n| n.label.as_deref() == Some("trigger"))
            .expect("trigger node");
        assert!(
            trigger_node.described_by.is_empty(),
            "expected tooltip to remain closed while focused after Escape dismissal"
        );
    }

    #[test]
    fn tooltip_closes_when_trigger_scrolls() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let trigger_id: Rc<Cell<Option<fret_ui::elements::GlobalElementId>>> =
            Rc::new(Cell::new(None));
        let content_id: Rc<Cell<Option<fret_ui::elements::GlobalElementId>>> =
            Rc::new(Cell::new(None));
        let scroll_handle = fret_ui::scroll::ScrollHandle::default();

        let mut services = FakeServices;
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            CoreSize::new(Px(800.0), Px(600.0)),
        );

        fn center(rect: Rect) -> Point {
            Point::new(
                Px(rect.origin.x.0 + rect.size.width.0 * 0.5),
                Px(rect.origin.y.0 + rect.size.height.0 * 0.5),
            )
        }

        fn render_scroll_frame(
            ui: &mut UiTree<App>,
            app: &mut App,
            services: &mut dyn fret_core::UiServices,
            window: AppWindowId,
            bounds: Rect,
            trigger_id_out: Rc<Cell<Option<fret_ui::elements::GlobalElementId>>>,
            content_id_out: Rc<Cell<Option<fret_ui::elements::GlobalElementId>>>,
            scroll_handle: &fret_ui::scroll::ScrollHandle,
        ) {
            OverlayController::begin_frame(app, window);
            let root = fret_ui::declarative::render_root(
                ui,
                app,
                services,
                window,
                bounds,
                "test",
                |cx| {
                    TooltipProvider::new()
                        .delay_duration_frames(0)
                        .skip_delay_duration_frames(0)
                        .with(cx, |cx| {
                            let scroll = cx.scroll(
                                fret_ui::element::ScrollProps {
                                    layout: {
                                        let mut layout = LayoutStyle::default();
                                        layout.size.width = Length::Px(Px(320.0));
                                        layout.size.height = Length::Px(Px(180.0));
                                        layout
                                    },
                                    axis: fret_ui::element::ScrollAxis::Y,
                                    scroll_handle: Some(scroll_handle.clone()),
                                    probe_unbounded: true,
                                },
                                |cx| {
                                    let trigger = cx.pressable_with_id(
                                        PressableProps {
                                            layout: {
                                                let mut layout = LayoutStyle::default();
                                                layout.size.width = Length::Px(Px(120.0));
                                                layout.size.height = Length::Px(Px(40.0));
                                                layout
                                            },
                                            enabled: true,
                                            focusable: true,
                                            a11y: PressableA11y {
                                                role: Some(SemanticsRole::Button),
                                                label: Some(Arc::from("trigger")),
                                                ..Default::default()
                                            },
                                            ..Default::default()
                                        },
                                        |cx, _st, id| {
                                            trigger_id_out.set(Some(id));
                                            vec![cx.container(ContainerProps::default(), |_cx| {
                                                Vec::new()
                                            })]
                                        },
                                    );

                                    let content = TooltipContent::new(vec![
                                        ui::raw_text(cx, "tip").into_element(cx),
                                    ])
                                    .into_element(cx);
                                    content_id_out.set(Some(content.id));

                                    vec![cx.column(
                                        fret_ui::element::ColumnProps {
                                            gap: Px(0.0),
                                            ..Default::default()
                                        },
                                        |cx| {
                                            let mut out: Vec<AnyElement> = Vec::new();
                                            out.push(
                                                Tooltip::new(trigger, content)
                                                    .open_delay_frames(0)
                                                    .close_delay_frames(0)
                                                    .disable_hoverable_content(false)
                                                    .into_element(cx),
                                            );
                                            for _ in 0..50 {
                                                out.push(cx.text("filler"));
                                            }
                                            out
                                        },
                                    )]
                                },
                            );

                            vec![scroll]
                        })
                },
            );
            ui.set_root(root);
            OverlayController::render(ui, app, services, window, bounds);
        }

        // Frame 1: layout and read trigger bounds.
        app.set_frame_id(FrameId(1));
        render_scroll_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            trigger_id.clone(),
            content_id.clone(),
            &scroll_handle,
        );
        ui.request_semantics_snapshot();
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        let snap = ui.semantics_snapshot().expect("semantics snapshot");
        let trigger_node = snap
            .nodes
            .iter()
            .find(|n| n.label.as_deref() == Some("trigger"))
            .expect("trigger node");
        let trigger_point = center(trigger_node.bounds);

        // Hover trigger to open tooltip.
        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::Pointer(fret_core::PointerEvent::Move {
                pointer_id: fret_core::PointerId(0),
                position: trigger_point,
                buttons: fret_core::MouseButtons::default(),
                modifiers: fret_core::Modifiers::default(),
                pointer_type: fret_core::PointerType::Mouse,
            }),
        );

        // Frame 2: tooltip should be open.
        app.set_frame_id(FrameId(2));
        render_scroll_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            trigger_id.clone(),
            content_id.clone(),
            &scroll_handle,
        );
        ui.request_semantics_snapshot();
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        let content_element = content_id.get().expect("content element id");
        assert!(
            fret_ui::elements::node_for_element(&mut app, window, content_element).is_some(),
            "expected tooltip content to be mounted after opening"
        );

        // Scrolling the trigger's scroll container should close the tooltip (Radix scroll target
        // contains trigger).
        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::Pointer(fret_core::PointerEvent::Wheel {
                pointer_id: fret_core::PointerId(0),
                position: trigger_point,
                delta: Point::new(Px(0.0), Px(-40.0)),
                modifiers: fret_core::Modifiers::default(),
                pointer_type: fret_core::PointerType::Mouse,
            }),
        );
        assert!(
            scroll_handle.offset().y.0 > 0.01,
            "expected the trigger scroll container to consume wheel input"
        );

        app.set_frame_id(FrameId(3));
        render_scroll_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            trigger_id.clone(),
            content_id.clone(),
            &scroll_handle,
        );
        ui.request_semantics_snapshot();
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        let snap = ui.semantics_snapshot().expect("semantics snapshot");
        let trigger_node = snap
            .nodes
            .iter()
            .find(|n| n.label.as_deref() == Some("trigger"))
            .expect("trigger node");
        assert!(
            trigger_node.described_by.is_empty(),
            "expected aria-describedby to be cleared after scroll dismissal"
        );
    }

    #[test]
    fn tooltip_does_not_close_on_unrelated_scroll() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let trigger_id: Rc<Cell<Option<fret_ui::elements::GlobalElementId>>> =
            Rc::new(Cell::new(None));
        let content_id: Rc<Cell<Option<fret_ui::elements::GlobalElementId>>> =
            Rc::new(Cell::new(None));
        let scroll_id: Rc<Cell<Option<fret_ui::elements::GlobalElementId>>> =
            Rc::new(Cell::new(None));
        let scroll_with_trigger_handle = fret_ui::scroll::ScrollHandle::default();
        let other_scroll_handle = fret_ui::scroll::ScrollHandle::default();

        let mut services = FakeServices;
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            CoreSize::new(Px(800.0), Px(600.0)),
        );

        fn center(rect: Rect) -> Point {
            Point::new(
                Px(rect.origin.x.0 + rect.size.width.0 * 0.5),
                Px(rect.origin.y.0 + rect.size.height.0 * 0.5),
            )
        }

        fn render_frame(
            ui: &mut UiTree<App>,
            app: &mut App,
            services: &mut dyn fret_core::UiServices,
            window: AppWindowId,
            bounds: Rect,
            trigger_id_out: Rc<Cell<Option<fret_ui::elements::GlobalElementId>>>,
            content_id_out: Rc<Cell<Option<fret_ui::elements::GlobalElementId>>>,
            scroll_id_out: Rc<Cell<Option<fret_ui::elements::GlobalElementId>>>,
            scroll_with_trigger_handle: fret_ui::scroll::ScrollHandle,
            other_scroll_handle: fret_ui::scroll::ScrollHandle,
        ) {
            OverlayController::begin_frame(app, window);
            let root = fret_ui::declarative::render_root(
                ui,
                app,
                services,
                window,
                bounds,
                "test",
                |cx| {
                    TooltipProvider::new()
                        .delay_duration_frames(0)
                        .skip_delay_duration_frames(0)
                        .with(cx, |cx| {
                            let mut flex_layout = LayoutStyle::default();
                            flex_layout.size.width = Length::Px(Px(800.0));
                            flex_layout.size.height = Length::Px(Px(600.0));

                            vec![cx.flex(
                                FlexProps {
                                    layout: flex_layout,
                                    direction: fret_core::Axis::Vertical,
                                    gap: Px(24.0),
                                    ..Default::default()
                                },
                                |cx| {
                                    let scroll_with_trigger = cx.scroll(
                                        fret_ui::element::ScrollProps {
                                            layout: {
                                                let mut layout = LayoutStyle::default();
                                                layout.size.width = Length::Px(Px(320.0));
                                                layout.size.height = Length::Px(Px(180.0));
                                                layout
                                            },
                                            axis: fret_ui::element::ScrollAxis::Y,
                                            scroll_handle: Some(scroll_with_trigger_handle.clone()),
                                            probe_unbounded: true,
                                        },
                                        |cx| {
                                            let trigger = cx.pressable_with_id(
                                                PressableProps {
                                                    layout: {
                                                        let mut layout = LayoutStyle::default();
                                                        layout.size.width = Length::Px(Px(120.0));
                                                        layout.size.height = Length::Px(Px(40.0));
                                                        layout
                                                    },
                                                    enabled: true,
                                                    focusable: true,
                                                    a11y: PressableA11y {
                                                        role: Some(SemanticsRole::Button),
                                                        label: Some(Arc::from("trigger")),
                                                        ..Default::default()
                                                    },
                                                    ..Default::default()
                                                },
                                                |cx, _st, id| {
                                                    trigger_id_out.set(Some(id));
                                                    vec![cx.container(
                                                        ContainerProps::default(),
                                                        |_cx| Vec::new(),
                                                    )]
                                                },
                                            );

                                            let content = TooltipContent::new(vec![
                                                ui::raw_text(cx, "tip").into_element(cx),
                                            ])
                                            .into_element(cx);
                                            content_id_out.set(Some(content.id));

                                            vec![cx.column(
                                                fret_ui::element::ColumnProps {
                                                    gap: Px(0.0),
                                                    ..Default::default()
                                                },
                                                |cx| {
                                                    let mut out: Vec<AnyElement> = Vec::new();
                                                    out.push(
                                                        Tooltip::new(trigger, content)
                                                            .open_delay_frames(0)
                                                            .close_delay_frames(0)
                                                            .disable_hoverable_content(false)
                                                            .into_element(cx),
                                                    );
                                                    for _ in 0..50 {
                                                        out.push(cx.text("filler"));
                                                    }
                                                    out
                                                },
                                            )]
                                        },
                                    );

                                    let other_scroll = cx.scroll(
                                        fret_ui::element::ScrollProps {
                                            layout: {
                                                let mut layout = LayoutStyle::default();
                                                layout.size.width = Length::Px(Px(320.0));
                                                layout.size.height = Length::Px(Px(180.0));
                                                layout
                                            },
                                            axis: fret_ui::element::ScrollAxis::Y,
                                            scroll_handle: Some(other_scroll_handle.clone()),
                                            probe_unbounded: true,
                                        },
                                        |cx| {
                                            vec![cx.column(
                                                fret_ui::element::ColumnProps {
                                                    gap: Px(0.0),
                                                    ..Default::default()
                                                },
                                                |cx| {
                                                    let mut out: Vec<AnyElement> = Vec::new();
                                                    for _ in 0..50 {
                                                        out.push(cx.text("filler"));
                                                    }
                                                    out
                                                },
                                            )]
                                        },
                                    );
                                    scroll_id_out.set(Some(other_scroll.id));

                                    vec![scroll_with_trigger, other_scroll]
                                },
                            )]
                        })
                },
            );
            ui.set_root(root);
            OverlayController::render(ui, app, services, window, bounds);
        }

        // Frame 1: layout and read trigger/scroll bounds.
        app.set_frame_id(FrameId(1));
        render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            trigger_id.clone(),
            content_id.clone(),
            scroll_id.clone(),
            scroll_with_trigger_handle.clone(),
            other_scroll_handle.clone(),
        );
        ui.request_semantics_snapshot();
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        let snap = ui.semantics_snapshot().expect("semantics snapshot");
        let trigger_node = snap
            .nodes
            .iter()
            .find(|n| n.label.as_deref() == Some("trigger"))
            .expect("trigger node");
        let trigger_point = center(trigger_node.bounds);

        let other_scroll_element = scroll_id.get().expect("other scroll element id");
        let other_scroll_node =
            fret_ui::elements::node_for_element(&mut app, window, other_scroll_element)
                .expect("other scroll node");
        let other_scroll_bounds = ui
            .debug_node_bounds(other_scroll_node)
            .expect("other scroll bounds");
        let other_scroll_point = center(other_scroll_bounds);

        // Hover trigger to open tooltip.
        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::Pointer(fret_core::PointerEvent::Move {
                pointer_id: fret_core::PointerId(0),
                position: trigger_point,
                buttons: fret_core::MouseButtons::default(),
                modifiers: fret_core::Modifiers::default(),
                pointer_type: fret_core::PointerType::Mouse,
            }),
        );

        // Frame 2: tooltip should be open.
        app.set_frame_id(FrameId(2));
        render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            trigger_id.clone(),
            content_id.clone(),
            scroll_id.clone(),
            scroll_with_trigger_handle.clone(),
            other_scroll_handle.clone(),
        );
        ui.request_semantics_snapshot();
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        let content_element = content_id.get().expect("content element id");
        assert!(
            fret_ui::elements::node_for_element(&mut app, window, content_element).is_some(),
            "expected tooltip content to be mounted after opening"
        );

        // Scrolling a different scroll container should not close the tooltip.
        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::Pointer(fret_core::PointerEvent::Wheel {
                pointer_id: fret_core::PointerId(0),
                position: other_scroll_point,
                delta: Point::new(Px(0.0), Px(-40.0)),
                modifiers: fret_core::Modifiers::default(),
                pointer_type: fret_core::PointerType::Mouse,
            }),
        );
        assert!(
            other_scroll_handle.offset().y.0 > 0.01,
            "expected the unrelated scroll container to consume wheel input"
        );

        app.set_frame_id(FrameId(3));
        render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            trigger_id.clone(),
            content_id.clone(),
            scroll_id.clone(),
            scroll_with_trigger_handle.clone(),
            other_scroll_handle.clone(),
        );
        ui.request_semantics_snapshot();
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        assert!(
            fret_ui::elements::node_for_element(&mut app, window, content_element).is_some(),
            "expected tooltip to remain open when an unrelated scroll container scrolls"
        );
    }

    #[test]
    fn tooltip_remains_open_while_pointer_moves_over_content() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let trigger_id: Rc<Cell<Option<fret_ui::elements::GlobalElementId>>> =
            Rc::new(Cell::new(None));
        let content_id: Rc<Cell<Option<fret_ui::elements::GlobalElementId>>> =
            Rc::new(Cell::new(None));

        let mut services = FakeServices;
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            CoreSize::new(Px(800.0), Px(600.0)),
        );

        fn center(rect: Rect) -> Point {
            Point::new(
                Px(rect.origin.x.0 + rect.size.width.0 * 0.5),
                Px(rect.origin.y.0 + rect.size.height.0 * 0.5),
            )
        }

        fn render_frame(
            ui: &mut UiTree<App>,
            app: &mut App,
            services: &mut dyn fret_core::UiServices,
            window: AppWindowId,
            bounds: Rect,
            trigger_id_out: Rc<Cell<Option<fret_ui::elements::GlobalElementId>>>,
            content_id_out: Rc<Cell<Option<fret_ui::elements::GlobalElementId>>>,
        ) {
            OverlayController::begin_frame(app, window);
            let root = fret_ui::declarative::render_root(
                ui,
                app,
                services,
                window,
                bounds,
                "test",
                |cx| {
                    TooltipProvider::new()
                        .delay_duration_frames(0)
                        .skip_delay_duration_frames(0)
                        .with(cx, |cx| {
                            let trigger = cx.pressable_with_id(
                                PressableProps {
                                    layout: {
                                        let mut layout = LayoutStyle::default();
                                        layout.size.width = Length::Px(Px(120.0));
                                        layout.size.height = Length::Px(Px(40.0));
                                        layout
                                    },
                                    enabled: true,
                                    focusable: true,
                                    a11y: PressableA11y {
                                        role: Some(SemanticsRole::Button),
                                        label: Some(Arc::from("trigger")),
                                        ..Default::default()
                                    },
                                    ..Default::default()
                                },
                                |cx, _st, id| {
                                    trigger_id_out.set(Some(id));
                                    vec![cx.container(ContainerProps::default(), |_cx| Vec::new())]
                                },
                            );

                            let content =
                                TooltipContent::new(vec![ui::raw_text(cx, "tip").into_element(cx)])
                                    .into_element(cx);
                            content_id_out.set(Some(content.id));

                            vec![
                                Tooltip::new(trigger, content)
                                    .open_delay_frames(0)
                                    .close_delay_frames(0)
                                    .disable_hoverable_content(false)
                                    .into_element(cx),
                            ]
                        })
                },
            );

            ui.set_root(root);
            OverlayController::render(ui, app, services, window, bounds);
        }

        // Frame 1: establish element->node mappings and layout.
        app.set_frame_id(FrameId(1));
        render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            trigger_id.clone(),
            content_id.clone(),
        );
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        // Hover trigger to open tooltip (open_delay=0).
        let trigger_element = trigger_id.get().expect("trigger element id");
        let trigger_node = fret_ui::elements::node_for_element(&mut app, window, trigger_element)
            .expect("trigger node");
        let trigger_bounds = ui.debug_node_bounds(trigger_node).expect("trigger bounds");
        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::Pointer(fret_core::PointerEvent::Move {
                pointer_id: fret_core::PointerId(0),
                position: center(trigger_bounds),
                buttons: fret_core::MouseButtons::default(),
                modifiers: fret_core::Modifiers::default(),
                pointer_type: fret_core::PointerType::Mouse,
            }),
        );

        // Frame 2: tooltip should be open and mounted.
        app.set_frame_id(FrameId(2));
        render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            trigger_id.clone(),
            content_id.clone(),
        );
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        let content_element = content_id.get().expect("content element id");
        let content_node = fret_ui::elements::node_for_element(&mut app, window, content_element)
            .expect("expected tooltip content to be mounted after opening");
        let content_bounds = ui
            .debug_node_bounds(content_node)
            .expect("tooltip content bounds");

        // Move pointer to the tooltip content bounds (trigger is no longer hovered).
        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::Pointer(fret_core::PointerEvent::Move {
                pointer_id: fret_core::PointerId(0),
                position: center(content_bounds),
                buttons: fret_core::MouseButtons::default(),
                modifiers: fret_core::Modifiers::default(),
                pointer_type: fret_core::PointerType::Mouse,
            }),
        );

        // Frame 3: tooltip stays open because hoverable content grace area considers the pointer.
        app.set_frame_id(FrameId(3));
        render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            trigger_id.clone(),
            content_id.clone(),
        );
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        assert!(
            fret_ui::elements::node_for_element(&mut app, window, content_element).is_some(),
            "expected tooltip content to remain mounted while pointer is over content"
        );
    }

    #[test]
    fn tooltip_pointer_in_transit_suppresses_other_trigger_hover_open() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let content_1_id: Rc<Cell<Option<fret_ui::elements::GlobalElementId>>> =
            Rc::new(Cell::new(None));
        let content_2_id: Rc<Cell<Option<fret_ui::elements::GlobalElementId>>> =
            Rc::new(Cell::new(None));

        let mut services = FakeServices;
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            CoreSize::new(Px(800.0), Px(600.0)),
        );

        fn center(rect: Rect) -> Point {
            Point::new(
                Px(rect.origin.x.0 + rect.size.width.0 * 0.5),
                Px(rect.origin.y.0 + rect.size.height.0 * 0.5),
            )
        }

        fn render_frame(
            ui: &mut UiTree<App>,
            app: &mut App,
            services: &mut dyn fret_core::UiServices,
            window: AppWindowId,
            bounds: Rect,
            content_1_id_out: Rc<Cell<Option<fret_ui::elements::GlobalElementId>>>,
            content_2_id_out: Rc<Cell<Option<fret_ui::elements::GlobalElementId>>>,
        ) {
            OverlayController::begin_frame(app, window);
            let root = fret_ui::declarative::render_root(
                ui,
                app,
                services,
                window,
                bounds,
                "test",
                |cx| {
                    TooltipProvider::new()
                        .delay_duration_frames(0)
                        .skip_delay_duration_frames(0)
                        .disable_hoverable_content(false)
                        .with(cx, |cx| {
                            vec![cx.row(
                                fret_ui::element::RowProps {
                                    gap: Px(20.0),
                                    ..Default::default()
                                },
                                |cx| {
                                    let trigger_1 = cx.pressable_with_id(
                                        PressableProps {
                                            layout: {
                                                let mut layout = LayoutStyle::default();
                                                layout.size.width = Length::Px(Px(60.0));
                                                layout.size.height = Length::Px(Px(40.0));
                                                layout
                                            },
                                            enabled: true,
                                            focusable: true,
                                            a11y: PressableA11y {
                                                role: Some(SemanticsRole::Button),
                                                label: Some(Arc::from("trigger_1")),
                                                ..Default::default()
                                            },
                                            ..Default::default()
                                        },
                                        |cx, _st, _id| {
                                            vec![cx.container(ContainerProps::default(), |_cx| {
                                                Vec::new()
                                            })]
                                        },
                                    );

                                    let trigger_2 = cx.pressable_with_id(
                                        PressableProps {
                                            layout: {
                                                let mut layout = LayoutStyle::default();
                                                layout.size.width = Length::Px(Px(60.0));
                                                layout.size.height = Length::Px(Px(40.0));
                                                layout
                                            },
                                            enabled: true,
                                            focusable: true,
                                            a11y: PressableA11y {
                                                role: Some(SemanticsRole::Button),
                                                label: Some(Arc::from("trigger_2")),
                                                ..Default::default()
                                            },
                                            ..Default::default()
                                        },
                                        |cx, _st, _id| {
                                            vec![cx.container(ContainerProps::default(), |_cx| {
                                                Vec::new()
                                            })]
                                        },
                                    );

                                    let content_1 = TooltipContent::new(vec![
                                        ui::raw_text(cx, "tip1").into_element(cx),
                                    ])
                                    .into_element(cx);
                                    content_1_id_out.set(Some(content_1.id));

                                    let content_2 = TooltipContent::new(vec![
                                        ui::raw_text(cx, "tip2").into_element(cx),
                                    ])
                                    .into_element(cx);
                                    content_2_id_out.set(Some(content_2.id));

                                    vec![
                                        Tooltip::new(trigger_1, content_1)
                                            .open_delay_frames(0)
                                            .close_delay_frames(0)
                                            .disable_hoverable_content(false)
                                            .side(TooltipSide::Right)
                                            .side_offset(Px(120.0))
                                            .into_element(cx),
                                        Tooltip::new(trigger_2, content_2)
                                            .open_delay_frames(0)
                                            .close_delay_frames(0)
                                            .disable_hoverable_content(false)
                                            .side(TooltipSide::Right)
                                            .into_element(cx),
                                    ]
                                },
                            )]
                        })
                },
            );
            ui.set_root(root);
            OverlayController::render(ui, app, services, window, bounds);
        }

        // Frame 1: mount and read trigger bounds.
        app.set_frame_id(FrameId(1));
        render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            content_1_id.clone(),
            content_2_id.clone(),
        );
        ui.request_semantics_snapshot();
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        let snap = ui.semantics_snapshot().expect("semantics snapshot");
        let trigger_1 = snap
            .nodes
            .iter()
            .find(|n| n.label.as_deref() == Some("trigger_1"))
            .expect("trigger_1 node");
        let trigger_2 = snap
            .nodes
            .iter()
            .find(|n| n.label.as_deref() == Some("trigger_2"))
            .expect("trigger_2 node");

        let trigger_1_point = center(trigger_1.bounds);
        let trigger_2_point = center(trigger_2.bounds);

        // Hover trigger_1 to open tooltip 1.
        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::Pointer(fret_core::PointerEvent::Move {
                pointer_id: fret_core::PointerId(0),
                position: trigger_1_point,
                buttons: fret_core::MouseButtons::default(),
                modifiers: fret_core::Modifiers::default(),
                pointer_type: fret_core::PointerType::Mouse,
            }),
        );

        app.set_frame_id(FrameId(2));
        render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            content_1_id.clone(),
            content_2_id.clone(),
        );
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        let content_1_element = content_1_id.get().expect("content_1 element id");
        let content_2_element = content_2_id.get().expect("content_2 element id");

        assert!(
            fret_ui::elements::node_for_element(&mut app, window, content_1_element).is_some(),
            "expected tooltip 1 to be open after hover"
        );
        assert!(
            fret_ui::elements::node_for_element(&mut app, window, content_2_element).is_none(),
            "expected tooltip 2 to remain closed initially"
        );

        // Move pointer to trigger_2 while tooltip_1 is open. The trigger_2 point lies between the
        // trigger_1 anchor and the right-side content bounds, so tooltip_1 sets pointer-in-transit
        // and tooltip_2 should not open.
        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::Pointer(fret_core::PointerEvent::Move {
                pointer_id: fret_core::PointerId(0),
                position: trigger_2_point,
                buttons: fret_core::MouseButtons::default(),
                modifiers: fret_core::Modifiers::default(),
                pointer_type: fret_core::PointerType::Mouse,
            }),
        );

        app.set_frame_id(FrameId(3));
        render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            content_1_id.clone(),
            content_2_id.clone(),
        );
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        assert!(
            fret_ui::elements::node_for_element(&mut app, window, content_1_element).is_some(),
            "expected tooltip 1 to remain open while in transit"
        );
        assert!(
            fret_ui::elements::node_for_element(&mut app, window, content_2_element).is_none(),
            "expected tooltip 2 to remain closed while pointer-in-transit is active"
        );
    }

    #[test]
    fn tooltip_closes_when_hoverable_content_disabled() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let trigger_id: Rc<Cell<Option<fret_ui::elements::GlobalElementId>>> =
            Rc::new(Cell::new(None));
        let content_id: Rc<Cell<Option<fret_ui::elements::GlobalElementId>>> =
            Rc::new(Cell::new(None));

        let mut services = FakeServices;
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            CoreSize::new(Px(800.0), Px(600.0)),
        );

        fn center(rect: Rect) -> Point {
            Point::new(
                Px(rect.origin.x.0 + rect.size.width.0 * 0.5),
                Px(rect.origin.y.0 + rect.size.height.0 * 0.5),
            )
        }

        fn render_frame(
            ui: &mut UiTree<App>,
            app: &mut App,
            services: &mut dyn fret_core::UiServices,
            window: AppWindowId,
            bounds: Rect,
            trigger_id_out: Rc<Cell<Option<fret_ui::elements::GlobalElementId>>>,
            content_id_out: Rc<Cell<Option<fret_ui::elements::GlobalElementId>>>,
        ) {
            OverlayController::begin_frame(app, window);
            let root = fret_ui::declarative::render_root(
                ui,
                app,
                services,
                window,
                bounds,
                "test",
                |cx| {
                    TooltipProvider::new()
                        .delay_duration_frames(0)
                        .skip_delay_duration_frames(0)
                        .with(cx, |cx| {
                            let trigger = cx.pressable_with_id(
                                PressableProps {
                                    layout: {
                                        let mut layout = LayoutStyle::default();
                                        layout.size.width = Length::Px(Px(120.0));
                                        layout.size.height = Length::Px(Px(40.0));
                                        layout
                                    },
                                    enabled: true,
                                    focusable: true,
                                    a11y: PressableA11y {
                                        role: Some(SemanticsRole::Button),
                                        label: Some(Arc::from("trigger")),
                                        ..Default::default()
                                    },
                                    ..Default::default()
                                },
                                |cx, _st, id| {
                                    trigger_id_out.set(Some(id));
                                    vec![cx.container(ContainerProps::default(), |_cx| Vec::new())]
                                },
                            );

                            let content =
                                TooltipContent::new(vec![ui::raw_text(cx, "tip").into_element(cx)])
                                    .into_element(cx);
                            content_id_out.set(Some(content.id));

                            vec![
                                Tooltip::new(trigger, content)
                                    .open_delay_frames(0)
                                    .close_delay_frames(0)
                                    .side(TooltipSide::Top)
                                    .side_offset(Px(120.0))
                                    .disable_hoverable_content(true)
                                    .into_element(cx),
                            ]
                        })
                },
            );

            ui.set_root(root);
            OverlayController::render(ui, app, services, window, bounds);
        }

        // Frame 1: establish element->node mappings and layout.
        app.set_frame_id(FrameId(1));
        render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            trigger_id.clone(),
            content_id.clone(),
        );
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        // Hover trigger to open tooltip (open_delay=0).
        let trigger_element = trigger_id.get().expect("trigger element id");
        let trigger_node = fret_ui::elements::node_for_element(&mut app, window, trigger_element)
            .expect("trigger node");
        let trigger_bounds = ui.debug_node_bounds(trigger_node).expect("trigger bounds");
        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::Pointer(fret_core::PointerEvent::Move {
                pointer_id: fret_core::PointerId(0),
                position: center(trigger_bounds),
                buttons: fret_core::MouseButtons::default(),
                modifiers: fret_core::Modifiers::default(),
                pointer_type: fret_core::PointerType::Mouse,
            }),
        );

        // Frame 2: tooltip should be open and mounted.
        app.set_frame_id(FrameId(2));
        render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            trigger_id.clone(),
            content_id.clone(),
        );
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        let content_element = content_id.get().expect("content element id");
        let content_node = fret_ui::elements::node_for_element(&mut app, window, content_element)
            .expect("expected tooltip content to be mounted after opening");
        let content_bounds = ui
            .debug_node_bounds(content_node)
            .expect("tooltip content bounds");
        let tooltip_layer_root = *ui
            .debug_node_path(content_node)
            .first()
            .expect("tooltip node path root");

        // Move pointer onto the tooltip content, but ensure the coordinate is outside the trigger
        // bounds (otherwise the trigger is still "hovered" and no close is expected).
        let trigger_element = trigger_id.get().expect("trigger element id");
        let trigger_node = fret_ui::elements::node_for_element(&mut app, window, trigger_element)
            .expect("trigger node");
        let trigger_bounds = ui.debug_node_bounds(trigger_node).expect("trigger bounds");
        let candidates = [
            Point::new(
                Px(content_bounds.origin.x.0 + 1.0),
                Px(content_bounds.origin.y.0 + 1.0),
            ),
            Point::new(
                Px(content_bounds.origin.x.0 + content_bounds.size.width.0 - 1.0),
                Px(content_bounds.origin.y.0 + 1.0),
            ),
            Point::new(
                Px(content_bounds.origin.x.0 + 1.0),
                Px(content_bounds.origin.y.0 + content_bounds.size.height.0 - 1.0),
            ),
            Point::new(
                Px(content_bounds.origin.x.0 + content_bounds.size.width.0 - 1.0),
                Px(content_bounds.origin.y.0 + content_bounds.size.height.0 - 1.0),
            ),
            center(content_bounds),
        ];
        let content_point = candidates
            .into_iter()
            .find(|p| !trigger_bounds.contains(*p))
            .unwrap_or(center(content_bounds));
        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::Pointer(fret_core::PointerEvent::Move {
                pointer_id: fret_core::PointerId(0),
                position: content_point,
                buttons: fret_core::MouseButtons::default(),
                modifiers: fret_core::Modifiers::default(),
                pointer_type: fret_core::PointerType::Mouse,
            }),
        );

        // Frame 3: close begins immediately (close_delay=0), but Presence keeps the layer mounted
        // while fading out. Assert that it becomes hidden by the end of the fade-out.
        let settle_frames = crate::overlay_motion::SHADCN_MOTION_TICKS_100 + 1;
        for frame in 3..=(2 + settle_frames) {
            app.set_frame_id(FrameId(frame));
            render_frame(
                &mut ui,
                &mut app,
                &mut services,
                window,
                bounds,
                trigger_id.clone(),
                content_id.clone(),
            );
            ui.layout_all(&mut app, &mut services, bounds, 1.0);
        }

        let tooltip_layer = ui
            .debug_layers_in_paint_order()
            .into_iter()
            .find(|layer| layer.root == tooltip_layer_root);
        assert!(
            tooltip_layer.is_none_or(|layer| !layer.visible),
            "expected tooltip to become hidden after hoverable content is disabled"
        );
    }

    #[test]
    fn tooltip_anchor_override_uses_anchor_bounds_for_placement() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let anchor_id: Rc<Cell<Option<fret_ui::elements::GlobalElementId>>> =
            Rc::new(Cell::new(None));
        let trigger_id: Rc<Cell<Option<fret_ui::elements::GlobalElementId>>> =
            Rc::new(Cell::new(None));
        let content_id: Rc<Cell<Option<fret_ui::elements::GlobalElementId>>> =
            Rc::new(Cell::new(None));

        let mut services = FakeServices;
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            CoreSize::new(Px(800.0), Px(600.0)),
        );

        fn render_frame(
            ui: &mut UiTree<App>,
            app: &mut App,
            services: &mut dyn fret_core::UiServices,
            window: AppWindowId,
            bounds: Rect,
            anchor_id_out: Rc<Cell<Option<fret_ui::elements::GlobalElementId>>>,
            trigger_id_out: Rc<Cell<Option<fret_ui::elements::GlobalElementId>>>,
            content_id_out: Rc<Cell<Option<fret_ui::elements::GlobalElementId>>>,
        ) {
            OverlayController::begin_frame(app, window);

            let root = fret_ui::declarative::render_root(
                ui,
                app,
                services,
                window,
                bounds,
                "test",
                |cx| {
                    let anchor_id_out_for_anchor = anchor_id_out.clone();
                    let anchor = cx.pressable_with_id(
                        PressableProps {
                            layout: {
                                let mut layout = LayoutStyle::default();
                                layout.size.width = Length::Px(Px(50.0));
                                layout.size.height = Length::Px(Px(10.0));
                                layout.inset.top = Some(Px(120.0));
                                layout.inset.left = Some(Px(240.0));
                                layout.position = fret_ui::element::PositionStyle::Absolute;
                                layout
                            },
                            enabled: false,
                            focusable: false,
                            ..Default::default()
                        },
                        move |_cx, _st, id| {
                            anchor_id_out_for_anchor.set(Some(id));
                            vec![]
                        },
                    );

                    let anchor_id = anchor_id_out.get().expect("anchor element id");

                    let trigger = cx.pressable_with_id(
                        PressableProps {
                            layout: {
                                let mut layout = LayoutStyle::default();
                                layout.size.width = Length::Px(Px(120.0));
                                layout.size.height = Length::Px(Px(40.0));
                                layout
                            },
                            enabled: true,
                            focusable: true,
                            a11y: PressableA11y {
                                role: Some(SemanticsRole::Button),
                                label: Some(Arc::from("trigger")),
                                ..Default::default()
                            },
                            ..Default::default()
                        },
                        |cx, _st, id| {
                            trigger_id_out.set(Some(id));
                            vec![cx.container(ContainerProps::default(), |_cx| Vec::new())]
                        },
                    );

                    let content = cx.semantics(
                        SemanticsProps {
                            role: SemanticsRole::Panel,
                            ..Default::default()
                        },
                        |cx| {
                            vec![
                                TooltipContent::new(vec![ui::raw_text(cx, "tip").into_element(cx)])
                                    .into_element(cx),
                            ]
                        },
                    );
                    content_id_out.set(Some(content.id));

                    vec![
                        anchor,
                        Tooltip::new(trigger, content)
                            .anchor_element(anchor_id)
                            .side(TooltipSide::Bottom)
                            .align(TooltipAlign::Start)
                            .side_offset(Px(8.0))
                            .window_margin(Px(0.0))
                            .arrow(false)
                            .open_delay_frames(0)
                            .close_delay_frames(0)
                            .into_element(cx),
                    ]
                },
            );

            ui.set_root(root);
            OverlayController::render(ui, app, services, window, bounds);
        }

        // Frame 1: establish bounds for the anchor + element/node mappings.
        app.set_frame_id(FrameId(1));
        app.set_tick_id(TickId(1));
        render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            anchor_id.clone(),
            trigger_id.clone(),
            content_id.clone(),
        );
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::Pointer(fret_core::PointerEvent::Move {
                pointer_id: fret_core::PointerId(0),
                position: Point::new(Px(12.0), Px(12.0)),
                buttons: fret_core::MouseButtons::default(),
                modifiers: fret_core::Modifiers::default(),
                pointer_type: fret_core::PointerType::Mouse,
            }),
        );

        // Frame 2: hover should open the tooltip, and placement should use the anchor override.
        app.set_frame_id(FrameId(2));
        app.set_tick_id(TickId(2));
        render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            anchor_id.clone(),
            trigger_id.clone(),
            content_id.clone(),
        );

        // Tooltip uses render-transform motion on open; advance a few frames to reach steady state.
        let settle_frames: u64 = crate::overlay_motion::SHADCN_MOTION_TICKS_100 + 2;
        for step in 0..settle_frames {
            let tick = 3 + step;
            app.set_frame_id(FrameId(tick));
            app.set_tick_id(TickId(tick));
            render_frame(
                &mut ui,
                &mut app,
                &mut services,
                window,
                bounds,
                anchor_id.clone(),
                trigger_id.clone(),
                content_id.clone(),
            );
            ui.layout_all(&mut app, &mut services, bounds, 1.0);
        }
        ui.request_semantics_snapshot();
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        let content_element = content_id.get().expect("content element id");

        let anchor_element = anchor_id.get().expect("anchor element id");
        let anchor_bounds = fret_ui::elements::bounds_for_element(&mut app, window, anchor_element)
            .expect("anchor bounds");

        let expected = anchored_panel_bounds_sized(
            bounds,
            anchor_bounds,
            CoreSize::new(Px(240.0), Px(44.0)),
            Px(8.0),
            Side::Bottom,
            Align::Start,
        );

        let snap = ui.semantics_snapshot().expect("semantics snapshot");
        let content_node = fret_ui::elements::node_for_element(&mut app, window, content_element)
            .expect("content node");
        let content_bounds = snap
            .nodes
            .iter()
            .find(|n| n.id == content_node)
            .map(|n| n.bounds)
            .expect("content bounds");

        assert_eq!(content_bounds.origin, expected.origin);
    }
}
