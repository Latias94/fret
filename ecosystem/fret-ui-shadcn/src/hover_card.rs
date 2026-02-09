use std::sync::Arc;

use crate::popper_arrow::{self, DiamondArrowStyle};
use fret_core::{Point, PointerType, Px, Rect, Size};
use fret_runtime::Model;
use fret_ui::action::{
    ActionCx, PointerCancelCx, PointerDownCx, PointerMoveCx, PointerUpCx, UiPointerActionHost,
};
use fret_ui::element::{
    AnyElement, ContainerProps, ElementKind, HoverRegionProps, Length, Overflow, PointerRegionProps,
};
use fret_ui::overlay_placement::{Align, Side};
use fret_ui::{ElementContext, Invalidation, Theme, UiHost};
use fret_ui_kit::declarative::ModelWatchExt as _;
use fret_ui_kit::declarative::{scheduling, style as decl_style};
use fret_ui_kit::headless::safe_hover;
use fret_ui_kit::overlay;
use fret_ui_kit::primitives::direction as direction_prim;
use fret_ui_kit::primitives::hover_card as radix_hover_card;
use fret_ui_kit::primitives::hover_intent::HoverIntentConfig;
use fret_ui_kit::primitives::popper;
use fret_ui_kit::primitives::popper_content;
use fret_ui_kit::primitives::presence as radix_presence;
use fret_ui_kit::{
    ChromeRefinement, ColorRef, LayoutRefinement, MetricRef, OverlayPresence, Radius, Space,
};

use crate::layout as shadcn_layout;
use crate::overlay_motion;

// Radix default delays: open=700ms, close=300ms. We approximate with 60fps ticks.
const HOVER_CARD_DEFAULT_OPEN_DELAY_FRAMES: u32 =
    (overlay_motion::SHADCN_MOTION_TICKS_500 + overlay_motion::SHADCN_MOTION_TICKS_200) as u32;
const HOVER_CARD_DEFAULT_CLOSE_DELAY_FRAMES: u32 = overlay_motion::SHADCN_MOTION_TICKS_300 as u32;
const HOVER_CARD_SAFE_CORRIDOR_BUFFER: Px = Px(5.0);
// A short lease prevents hover-driven close from firing immediately after pointer interactions
// (e.g. click/drag), while still allowing the card to close promptly once the interaction ends.
const HOVER_CARD_INTERACTION_LEASE_FRAMES: u32 = overlay_motion::SHADCN_MOTION_TICKS_300 as u32;

type OnOpenChange = Arc<dyn Fn(bool) + Send + Sync + 'static>;

#[derive(Default)]
struct HoverCardOpenChangeCallbackState {
    initialized: bool,
    last_open: bool,
    pending_complete: Option<bool>,
}

fn hover_card_open_change_events(
    state: &mut HoverCardOpenChangeCallbackState,
    open: bool,
    present: bool,
    animating: bool,
) -> (Option<bool>, Option<bool>) {
    let mut changed = None;
    let mut completed = None;

    if !state.initialized {
        state.initialized = true;
        state.last_open = open;
    } else if state.last_open != open {
        state.last_open = open;
        state.pending_complete = Some(open);
        changed = Some(open);
    }

    if state.pending_complete == Some(open) && present == open && !animating {
        state.pending_complete = None;
        completed = Some(open);
    }

    (changed, completed)
}

#[derive(Default)]
struct HoverCardLastPointerModelState {
    model: Option<Model<Option<Point>>>,
}

fn hover_card_last_pointer_model<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    hover_card_id: fret_ui::elements::GlobalElementId,
) -> Model<Option<Point>> {
    let existing = cx.with_state_for(
        hover_card_id,
        HoverCardLastPointerModelState::default,
        |st| st.model.clone(),
    );
    if let Some(model) = existing {
        model
    } else {
        let model = cx.app.models_mut().insert(None::<Point>);
        cx.with_state_for(
            hover_card_id,
            HoverCardLastPointerModelState::default,
            |st| {
                st.model = Some(model.clone());
            },
        );
        model
    }
}

fn fixed_size_hint_px(element: &AnyElement) -> Option<Size> {
    fn visit(node: &AnyElement, best: &mut Option<Size>) {
        if let ElementKind::Container(ContainerProps { layout, .. }) = &node.kind {
            if let Length::Px(w) = layout.size.width {
                let h = match layout.size.height {
                    Length::Px(h) => h,
                    _ => Px(120.0),
                };
                let candidate = Size::new(w, h);
                if best
                    .map(|cur| candidate.width.0 > cur.width.0)
                    .unwrap_or(true)
                {
                    *best = Some(candidate);
                }
            }
        }

        for child in &node.children {
            visit(child, best);
        }
    }

    let mut best: Option<Size> = None;
    visit(element, &mut best);
    best
}

fn hover_card_content_chrome(theme: &Theme) -> ChromeRefinement {
    let bg = theme.color_required("popover");
    let border = theme.color_required("border");

    ChromeRefinement::default()
        .rounded(Radius::Md)
        .border_1()
        .bg(ColorRef::Color(bg))
        .border_color(ColorRef::Color(border))
        .p(Space::N4)
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum HoverCardAlign {
    Start,
    #[default]
    Center,
    End,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum HoverCardSide {
    Top,
    Right,
    #[default]
    Bottom,
    Left,
}

#[derive(Debug, Default, Clone)]
struct HoverCardSharedState {
    overlay_hovered: bool,
    anchor_bounds: Option<Rect>,
    floating_bounds: Option<Rect>,
}

/// shadcn/ui `HoverCard` root (v4).
///
/// This is a floating hover surface anchored to a trigger. In Radix/shadcn this uses a portal;
/// in Fret this is implemented as a component-layer policy built on runtime substrate primitives:
/// - `HoverRegion` (hover tracking)
/// - cross-frame geometry queries (`elements::bounds_for_element`)
/// - placement solver (`overlay_placement`)
#[derive(Clone)]
pub struct HoverCard {
    open: Option<Model<bool>>,
    default_open: bool,
    trigger: AnyElement,
    content: AnyElement,
    align: HoverCardAlign,
    side: HoverCardSide,
    side_offset: Px,
    window_margin_override: Option<Px>,
    arrow: bool,
    arrow_size_override: Option<Px>,
    arrow_padding_override: Option<Px>,
    open_delay_frames: u32,
    close_delay_frames: u32,
    layout: LayoutRefinement,
    anchor_override: Option<fret_ui::elements::GlobalElementId>,
    on_open_change: Option<OnOpenChange>,
    on_open_change_complete: Option<OnOpenChange>,
}

impl std::fmt::Debug for HoverCard {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("HoverCard")
            .field("open", &"<model>")
            .field("default_open", &self.default_open)
            .field("align", &self.align)
            .field("side", &self.side)
            .field("side_offset", &self.side_offset)
            .field("window_margin_override", &self.window_margin_override)
            .field("arrow", &self.arrow)
            .field("open_delay_frames", &self.open_delay_frames)
            .field("close_delay_frames", &self.close_delay_frames)
            .field("layout", &self.layout)
            .field("anchor_override", &self.anchor_override)
            .field("on_open_change", &self.on_open_change.is_some())
            .field(
                "on_open_change_complete",
                &self.on_open_change_complete.is_some(),
            )
            .finish()
    }
}

impl HoverCard {
    pub fn new(trigger: AnyElement, content: AnyElement) -> Self {
        Self {
            open: None,
            default_open: false,
            trigger,
            content,
            align: HoverCardAlign::default(),
            side: HoverCardSide::default(),
            side_offset: Px(4.0),
            window_margin_override: None,
            arrow: false,
            arrow_size_override: None,
            arrow_padding_override: None,
            open_delay_frames: HOVER_CARD_DEFAULT_OPEN_DELAY_FRAMES,
            close_delay_frames: HOVER_CARD_DEFAULT_CLOSE_DELAY_FRAMES,
            layout: LayoutRefinement::default(),
            anchor_override: None,
            on_open_change: None,
            on_open_change_complete: None,
        }
    }

    /// Creates a hover card with a controlled/uncontrolled open model (Radix `open` / `defaultOpen`).
    ///
    /// Note: If `open` is `None`, the internal model is stored in element state at the call site.
    /// Call this from a stable subtree (key the parent node if needed).
    pub fn new_controllable<H: UiHost>(
        cx: &mut ElementContext<'_, H>,
        open: Option<Model<bool>>,
        default_open: bool,
        trigger: AnyElement,
        content: AnyElement,
    ) -> Self {
        let open = radix_hover_card::HoverCardRoot::new()
            .open(open)
            .default_open(default_open)
            .open_model(cx);
        Self::new(trigger, content).open(Some(open))
    }

    /// Sets the controlled `open` model (`Some`) or selects uncontrolled mode (`None`).
    pub fn open(mut self, open: Option<Model<bool>>) -> Self {
        self.open = open;
        self
    }

    /// Sets the uncontrolled initial open value (Radix `defaultOpen`).
    ///
    /// Note: If a controlled `open` model is provided, this value is ignored.
    pub fn default_open(mut self, default_open: bool) -> Self {
        self.default_open = default_open;
        self
    }

    pub fn align(mut self, align: HoverCardAlign) -> Self {
        self.align = align;
        self
    }

    pub fn side(mut self, side: HoverCardSide) -> Self {
        self.side = side;
        self
    }

    pub fn side_offset(mut self, offset: Px) -> Self {
        self.side_offset = offset;
        self
    }

    pub fn open_delay_frames(mut self, frames: u32) -> Self {
        self.open_delay_frames = frames;
        self
    }

    pub fn close_delay_frames(mut self, frames: u32) -> Self {
        self.close_delay_frames = frames;
        self
    }

    pub fn window_margin(mut self, margin: Px) -> Self {
        self.window_margin_override = Some(margin);
        self
    }

    /// Enables a HoverCard arrow (Radix `HoverCardArrow`-style).
    ///
    /// Default: `false`.
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

    /// Override the element used as the placement anchor.
    ///
    /// Notes:
    /// - Hover tracking still uses the trigger element.
    /// - The anchor bounds are resolved from last-frame layout/visual bounds.
    pub fn anchor_element(mut self, element: fret_ui::elements::GlobalElementId) -> Self {
        self.anchor_override = Some(element);
        self
    }

    /// Called when the open state changes (Base UI `onOpenChange`).
    pub fn on_open_change(mut self, on_open_change: Option<OnOpenChange>) -> Self {
        self.on_open_change = on_open_change;
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

    pub fn refine_layout(mut self, layout: LayoutRefinement) -> Self {
        self.layout = self.layout.merge(layout);
        self
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let theme = Theme::global(&*cx.app).clone();

        let layout = decl_style::layout_style(&theme, self.layout);
        let side_offset = if self.side_offset == Px(4.0) {
            theme
                .metric_by_key("component.hover_card.side_offset")
                .unwrap_or(self.side_offset)
        } else {
            self.side_offset
        };
        let window_margin = self.window_margin_override.unwrap_or_else(|| {
            theme
                .metric_by_key("component.hover_card.window_margin")
                .unwrap_or(Px(0.0))
        });

        let align = self.align;
        let side = self.side;
        let open_delay_frames = self.open_delay_frames;
        let close_delay_frames = self.close_delay_frames;
        let arrow = self.arrow;
        let arrow_size = self.arrow_size_override.unwrap_or_else(|| {
            theme
                .metric_by_key("component.hover_card.arrow_size")
                .unwrap_or(Px(12.0))
        });
        let arrow_padding = self.arrow_padding_override.unwrap_or_else(|| {
            theme
                .metric_by_key("component.hover_card.arrow_padding")
                .unwrap_or_else(|| MetricRef::radius(Radius::Md).resolve(&theme))
        });
        let arrow_bg = theme.color_required("popover");
        let arrow_border = theme.color_required("border");

        let uncontrolled_default_open = self.open.is_none() && self.default_open;
        let open_root = radix_hover_card::HoverCardRoot::new()
            .open(self.open)
            .default_open(self.default_open);
        // Store the uncontrolled `open` model at the HoverCard call site so `default_open` behaves
        // like Radix `defaultOpen` across frames (rather than being tied to an internal wrapper).
        let open = open_root.open_model(cx);
        let trigger = self.trigger;
        let content = self.content;
        let content_size_hint = fixed_size_hint_px(&content);
        let trigger_id = trigger.id;
        let content_id = content.id;
        let anchor_id = self.anchor_override.unwrap_or(trigger_id);
        let debug_trace = cfg!(test) && std::env::var_os("FRET_DEBUG_HOVERCARD").is_some();
        let pointer_can_hover =
            fret_ui_kit::declarative::primary_pointer_can_hover(cx, Invalidation::Layout, true);
        cx.hover_region(HoverRegionProps { layout }, move |cx, hovered| {
            let hovered = hovered && pointer_can_hover;
            let hover_card_id = cx.root_id();
            let open = open.clone();
            let last_pointer = hover_card_last_pointer_model(cx, hover_card_id);
            let mut open_now = cx.watch_model(&open).layout().copied().unwrap_or(false);
            if uncontrolled_default_open {
                #[derive(Default)]
                struct HoverCardDefaultOpenInit {
                    applied: bool,
                }
                let should_apply =
                    cx.with_state_for(hover_card_id, HoverCardDefaultOpenInit::default, |st| {
                        if st.applied {
                            false
                        } else {
                            st.applied = true;
                            true
                        }
                    });
                if should_apply && !open_now {
                    let _ = cx.app.models_mut().update(&open, |v| *v = true);
                    open_now = true;
                }
            }

            #[derive(Default)]
            struct HoverCardPointerDownModelState {
                model: Option<Model<bool>>,
                lease: Option<Model<u32>>,
            }

            let pointer_down_on_content = cx.with_state_for(
                hover_card_id,
                HoverCardPointerDownModelState::default,
                |st| st.model.clone(),
            );
            let pointer_down_on_content = if let Some(model) = pointer_down_on_content {
                model
            } else {
                let model = cx.app.models_mut().insert(false);
                cx.with_state_for(
                    hover_card_id,
                    HoverCardPointerDownModelState::default,
                    |st| {
                        st.model = Some(model.clone());
                    },
                );
                model
            };
            let mut pointer_down_on_content_now = cx
                .watch_model(&pointer_down_on_content)
                .layout()
                .copied()
                .unwrap_or(false);

            let interaction_lease = cx.with_state_for(
                hover_card_id,
                HoverCardPointerDownModelState::default,
                |st| st.lease.clone(),
            );
            let interaction_lease = if let Some(model) = interaction_lease {
                model
            } else {
                let model = cx.app.models_mut().insert(0u32);
                cx.with_state_for(
                    hover_card_id,
                    HoverCardPointerDownModelState::default,
                    |st| {
                        st.lease = Some(model.clone());
                    },
                );
                model
            };
            let interaction_lease_now = cx
                .watch_model(&interaction_lease)
                .layout()
                .copied()
                .unwrap_or(0);
            let interaction_lease_active = interaction_lease_now > 0;
            if pointer_down_on_content_now && !interaction_lease_active {
                // If we miss `PointerUp` (e.g. due to a descendant capturing pointer), treat an
                // expired lease as a conservative “no longer interacting” signal.
                let _ = cx
                    .app
                    .models_mut()
                    .update(&pointer_down_on_content, |v| *v = false);
                pointer_down_on_content_now = false;
            }

            let (overlay_hovered, anchor_bounds, floating_bounds) =
                cx.with_state_for(hover_card_id, HoverCardSharedState::default, |st| {
                    (st.overlay_hovered, st.anchor_bounds, st.floating_bounds)
                });
            let focused = cx.is_focused_element(trigger_id);
            let keyboard_focused =
                focused && fret_ui::input_modality::is_keyboard(&mut *cx.app, Some(cx.window));

            let pointer_in_corridor = cx
                .watch_model(&last_pointer)
                .layout()
                .copied()
                .unwrap_or(None)
                .zip(anchor_bounds)
                .zip(floating_bounds)
                .is_some_and(|((pointer, anchor), floating)| {
                    safe_hover::safe_hover_contains(
                        pointer,
                        anchor,
                        floating,
                        HOVER_CARD_SAFE_CORRIDOR_BUFFER,
                    )
                });

            let hovered =
                radix_hover_card::hover_card_hovered(hovered, overlay_hovered, keyboard_focused)
                    || pointer_in_corridor;

            let overlay_root_name = radix_hover_card::hover_card_root_name(hover_card_id);
            let overlay_root_id = fret_ui::elements::global_root(cx.window, &overlay_root_name);
            let has_text_selection = cx.has_active_text_selection_in_root(overlay_root_id);

            #[derive(Default)]
            struct HoverCardSelectionPointerState {
                saw_selection_while_pointer_down: bool,
            }

            let clear_stale_pointer_down_after_selection = cx.with_state_for(
                hover_card_id,
                HoverCardSelectionPointerState::default,
                |st| {
                    if pointer_down_on_content_now && has_text_selection {
                        st.saw_selection_while_pointer_down = true;
                        return false;
                    }

                    if pointer_down_on_content_now
                        && !has_text_selection
                        && st.saw_selection_while_pointer_down
                    {
                        st.saw_selection_while_pointer_down = false;
                        return true;
                    }

                    if !pointer_down_on_content_now {
                        st.saw_selection_while_pointer_down = false;
                    }

                    false
                },
            );

            if clear_stale_pointer_down_after_selection {
                let _ = cx
                    .app
                    .models_mut()
                    .update(&pointer_down_on_content, |v| *v = false);
                pointer_down_on_content_now = false;
            }

            let cfg = HoverIntentConfig::new(open_delay_frames as u64, close_delay_frames as u64);
            let update = radix_hover_card::hover_card_update_interaction(
                cx,
                open_now,
                hovered,
                pointer_down_on_content_now,
                has_text_selection,
                cfg,
            );

            scheduling::set_continuous_frames(
                cx,
                update.wants_continuous_ticks || interaction_lease_active,
            );

            if interaction_lease_active {
                let next = interaction_lease_now.saturating_sub(1);
                let _ = cx
                    .app
                    .models_mut()
                    .update(&interaction_lease, |v| *v = next);
            }

            if update.open != open_now {
                let _ = cx.app.models_mut().update(&open, |v| *v = update.open);
            }

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
            let (open_change, open_change_complete) =
                cx.with_state(HoverCardOpenChangeCallbackState::default, |state| {
                    hover_card_open_change_events(state, opening, motion.present, motion.animating)
                });
            if let (Some(open), Some(on_open_change)) = (open_change, self.on_open_change.as_ref())
            {
                on_open_change(open);
            }
            if let (Some(open), Some(on_open_change_complete)) =
                (open_change_complete, self.on_open_change_complete.as_ref())
            {
                on_open_change_complete(open);
            }
            let opacity = motion.opacity;
            let scale = motion.scale;
            let overlay_presence = OverlayPresence {
                present: motion.present,
                // Keep the hover card interactive for the full duration it is mounted. Otherwise
                // the card can remain visible during close transitions but become click-through,
                // which breaks “interactive hover card” outcomes (pager buttons, links, etc.).
                interactive: motion.present,
            };

            let out = vec![trigger];
            if debug_trace {
                eprintln!(
                    "hover_card trace frame_id={} open_now={} update_open={} present={} hovered={} overlay_hovered={} pointer_down_on_content={} interaction_lease={} has_text_selection={}",
                    cx.frame_id.0,
                    open_now,
                    update.open,
                    motion.present,
                    hovered,
                    overlay_hovered,
                    pointer_down_on_content_now,
                    interaction_lease_now,
                    has_text_selection,
                );
            }
            if !motion.present {
                cx.with_state_for(hover_card_id, HoverCardSharedState::default, |st| {
                    st.overlay_hovered = false;
                    st.anchor_bounds = None;
                    st.floating_bounds = None;
                });
                if pointer_down_on_content_now {
                    let _ = cx
                        .app
                        .models_mut()
                        .update(&pointer_down_on_content, |v| *v = false);
                }
                if interaction_lease_active {
                    let _ = cx.app.models_mut().update(&interaction_lease, |v| *v = 0);
                }
                return out;
            }

            let direction = direction_prim::use_direction_in_scope(cx, None);
            let overlay_children = cx.with_root_name(&overlay_root_name, move |cx| {
                let anchor = overlay::anchor_bounds_for_element(cx, anchor_id);
                let Some(anchor) = anchor else {
                    cx.with_state_for(hover_card_id, HoverCardSharedState::default, |st| {
                        st.overlay_hovered = false;
                        st.anchor_bounds = None;
                        st.floating_bounds = None;
                    });
                    return Vec::new();
                };

                let last_content_size = cx.last_bounds_for_element(content_id).map(|r| r.size);
                let estimated_size = Size::new(Px(256.0), Px(120.0));
                let content_size = content_size_hint
                    .or(last_content_size)
                    .unwrap_or(estimated_size);

                let outer = overlay::outer_bounds_with_window_margin(cx.bounds, window_margin);

                let align = match align {
                    HoverCardAlign::Start => Align::Start,
                    HoverCardAlign::Center => Align::Center,
                    HoverCardAlign::End => Align::End,
                };

                let placement_side = match side {
                    HoverCardSide::Top => Side::Top,
                    HoverCardSide::Right => Side::Right,
                    HoverCardSide::Bottom => Side::Bottom,
                    HoverCardSide::Left => Side::Left,
                };

                let (arrow_options, arrow_protrusion) =
                    popper::diamond_arrow_options(arrow, arrow_size, arrow_padding);

                let layout = popper::popper_content_layout_sized(
                    outer,
                    anchor,
                    content_size,
                    popper::PopperContentPlacement::new(
                        direction,
                        placement_side,
                        align,
                        side_offset,
                    )
                    .with_shift_cross_axis(true)
                    .with_arrow(arrow_options, arrow_protrusion),
                );
                cx.with_state_for(hover_card_id, HoverCardSharedState::default, |st| {
                    st.anchor_bounds = Some(anchor);
                    st.floating_bounds = Some(layout.rect);
                });

                let placed = layout.rect;
                let mut wrapper_insets = popper_arrow::wrapper_insets(&layout, arrow_protrusion);
                let slide_insets = overlay_motion::shadcn_slide_insets(layout.side);
                wrapper_insets.top.0 += slide_insets.top.0;
                wrapper_insets.right.0 += slide_insets.right.0;
                wrapper_insets.bottom.0 += slide_insets.bottom.0;
                wrapper_insets.left.0 += slide_insets.left.0;

                let origin = popper::popper_content_transform_origin(
                    &layout,
                    anchor,
                    arrow.then_some(arrow_size),
                );
                let transform = overlay_motion::shadcn_popper_presence_transform(
                    layout.side,
                    origin,
                    opacity,
                    scale,
                    opening,
                );

                let pointer_down_on_content_model = pointer_down_on_content.clone();
                let interaction_lease_model = interaction_lease.clone();
                let content_for_panel = content.clone();
                let wrapper = cx.hover_region(
                    HoverRegionProps {
                        layout: popper_content::popper_wrapper_layout(placed, wrapper_insets),
                    },
                    move |cx, hovered| {
                        cx.with_state_for(hover_card_id, HoverCardSharedState::default, |st| {
                            st.overlay_hovered = hovered;
                        });

                        let panel_layout = popper_content::popper_panel_layout(
                            placed,
                            wrapper_insets,
                            Overflow::Visible,
                        );
                        let panel = cx.pointer_region(
                            PointerRegionProps {
                                layout: panel_layout,
                                enabled: true,
                            },
                            move |cx| {
                                let pointer_down_model_for_down =
                                    pointer_down_on_content_model.clone();
                                let interaction_lease_model_for_down =
                                    interaction_lease_model.clone();
                                cx.pointer_region_on_pointer_down(Arc::new(
                                    move |host: &mut dyn UiPointerActionHost,
                                          cx: ActionCx,
                                          _down: PointerDownCx| {
                                        host.capture_pointer();
                                        let _ = host
                                            .models_mut()
                                            .update(&pointer_down_model_for_down, |v| *v = true);
                                        let _ = host
                                            .models_mut()
                                            .update(&interaction_lease_model_for_down, |v| {
                                                *v = HOVER_CARD_INTERACTION_LEASE_FRAMES;
                                            });
                                        host.request_redraw(cx.window);
                                        false
                                    },
                                ));

                                let pointer_down_model_for_up =
                                    pointer_down_on_content_model.clone();
                                let interaction_lease_model_for_up =
                                    interaction_lease_model.clone();
                                cx.pointer_region_on_pointer_up(Arc::new(
                                    move |host: &mut dyn UiPointerActionHost,
                                          cx: ActionCx,
                                          _up: PointerUpCx| {
                                        host.release_pointer_capture();
                                        let _ = host
                                            .models_mut()
                                            .update(&pointer_down_model_for_up, |v| *v = false);
                                        let _ = host.models_mut().update(
                                            &interaction_lease_model_for_up,
                                            |v| {
                                                *v = HOVER_CARD_INTERACTION_LEASE_FRAMES;
                                            },
                                        );
                                        host.request_redraw(cx.window);
                                        false
                                    },
                                ));

                                let pointer_down_model_for_move =
                                    pointer_down_on_content_model.clone();
                                let interaction_lease_model_for_move = interaction_lease_model.clone();
                                cx.pointer_region_on_pointer_move(Arc::new(
                                    move |host: &mut dyn UiPointerActionHost,
                                          cx: ActionCx,
                                          mv: PointerMoveCx| {
                                        if mv.buttons.left || mv.buttons.right || mv.buttons.middle {
                                            return false;
                                        }

                                        let is_down = host
                                            .models_mut()
                                            .read(&pointer_down_model_for_move, |v| *v)
                                            .ok()
                                            .unwrap_or(false);
                                        if is_down {
                                            // If a descendant captures pointer, this region may miss
                                            // the `PointerUp` hook. Use a no-buttons move as a
                                            // conservative “interaction ended” signal.
                                            host.release_pointer_capture();
                                            let _ = host
                                                .models_mut()
                                                .update(&pointer_down_model_for_move, |v| *v = false);
                                            let _ = host.models_mut().update(
                                                &interaction_lease_model_for_move,
                                                |v| {
                                                    *v = HOVER_CARD_INTERACTION_LEASE_FRAMES;
                                                },
                                            );
                                            host.request_redraw(cx.window);
                                        }
                                        false
                                    },
                                ));

                                let pointer_down_model_for_cancel =
                                    pointer_down_on_content_model.clone();
                                let interaction_lease_model_for_cancel = interaction_lease_model.clone();
                                cx.pointer_region_on_pointer_cancel(Arc::new(
                                    move |host: &mut dyn UiPointerActionHost,
                                          cx: ActionCx,
                                          _cancel: PointerCancelCx| {
                                        host.release_pointer_capture();
                                        let _ = host
                                            .models_mut()
                                            .update(&pointer_down_model_for_cancel, |v| *v = false);
                                        let _ = host.models_mut().update(
                                            &interaction_lease_model_for_cancel,
                                            |v| {
                                                *v = HOVER_CARD_INTERACTION_LEASE_FRAMES;
                                            },
                                        );
                                        host.request_redraw(cx.window);
                                        false
                                    },
                                ));

                                vec![content_for_panel.clone()]
                            },
                        );

                        let arrow_el = popper_arrow::diamond_arrow_element(
                            cx,
                            &layout,
                            wrapper_insets,
                            arrow_size,
                            DiamondArrowStyle {
                                bg: arrow_bg,
                                border: Some(arrow_border),
                                border_width: Px(1.0),
                            },
                        );

                        if let Some(arrow_el) = arrow_el {
                            vec![arrow_el, panel]
                        } else {
                            vec![panel]
                        }
                    },
                );

                vec![overlay_motion::wrap_opacity_and_render_transform(
                    cx,
                    opacity,
                    transform,
                    vec![wrapper],
                )]
            });

            let request = radix_hover_card::hover_card_request(
                hover_card_id,
                trigger_id,
                open.clone(),
                overlay_presence,
                overlay_children,
            );
            let mut request = request;
            let last_pointer_for_move = last_pointer.clone();
            request.dismissible_on_pointer_move = Some(Arc::new(move |host, _acx, mv| {
                if mv.pointer_type == PointerType::Touch {
                    return false;
                }
                let _ = host
                    .models_mut()
                    .update(&last_pointer_for_move, |v| *v = Some(mv.position));
                false
            }));
            radix_hover_card::request_hover_card(cx, request);

            out
        })
    }
}

/// shadcn/ui `HoverCardTrigger` (v4).
///
/// In the DOM this is a context-aware wrapper that does not impose layout. In Fret's declarative
/// authoring, the trigger is expressed as the first child passed to `HoverCard::new(...)`.
#[derive(Debug, Clone)]
pub struct HoverCardTrigger {
    child: AnyElement,
}

impl HoverCardTrigger {
    pub fn new(child: AnyElement) -> Self {
        Self { child }
    }

    pub fn into_element<H: UiHost>(self, _cx: &mut ElementContext<'_, H>) -> AnyElement {
        self.child
    }
}

/// Optional layout-only anchor for advanced hover card placement recipes.
///
/// Use [`HoverCard::anchor_element`] to wire the anchor element ID into placement.
#[derive(Debug, Clone)]
pub struct HoverCardAnchor {
    child: AnyElement,
}

impl HoverCardAnchor {
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

/// shadcn/ui `HoverCardContent` (v4).
#[derive(Debug, Clone)]
pub struct HoverCardContent {
    children: Vec<AnyElement>,
    chrome: ChromeRefinement,
    layout: LayoutRefinement,
}

impl HoverCardContent {
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

        let base_layout = LayoutRefinement::default().w_px(Px(256.0)).flex_shrink_0();

        let chrome = hover_card_content_chrome(&theme).merge(self.chrome);
        let radius = MetricRef::radius(Radius::Md).resolve(&theme);
        let mut props = decl_style::container_props(&theme, chrome, base_layout.merge(self.layout));
        props.shadow = Some(decl_style::shadow_md(&theme, radius));
        let children = self.children;
        shadcn_layout::container_flow(cx, props, children)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::Cell;
    use std::rc::Rc;

    use fret_app::App;
    use fret_core::{
        AppWindowId, Event, Modifiers, MouseButton, MouseButtons, PathCommand, PathConstraints,
        PathId, PathMetrics, PathService, PathStyle, Point, Px, Rect, SemanticsRole, SvgId,
        SvgService, TextBlobId, TextConstraints, TextMetrics, TextService,
    };
    use fret_runtime::{FrameId, TickId};
    use fret_ui::element::{
        ContainerProps, LayoutStyle, Length, PositionStyle, PressableProps, SemanticsProps,
    };
    use fret_ui::overlay_placement;
    use fret_ui::tree::UiTree;
    use fret_ui_kit::prelude::ActionHooksExt;
    use fret_ui_kit::{OverlayController, ui};

    #[test]
    fn hover_card_open_change_events_emit_change_and_complete_after_settle() {
        let mut state = HoverCardOpenChangeCallbackState::default();

        let (changed, completed) = hover_card_open_change_events(&mut state, false, false, false);
        assert_eq!(changed, None);
        assert_eq!(completed, None);

        let (changed, completed) = hover_card_open_change_events(&mut state, true, true, true);
        assert_eq!(changed, Some(true));
        assert_eq!(completed, None);

        let (changed, completed) = hover_card_open_change_events(&mut state, true, true, false);
        assert_eq!(changed, None);
        assert_eq!(completed, Some(true));
    }

    #[test]
    fn hover_card_open_change_events_complete_without_animation() {
        let mut state = HoverCardOpenChangeCallbackState::default();

        let _ = hover_card_open_change_events(&mut state, false, false, false);
        let (changed, completed) = hover_card_open_change_events(&mut state, true, true, false);

        assert_eq!(changed, Some(true));
        assert_eq!(completed, Some(true));
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
                    size: fret_core::Size::new(Px(10.0), Px(10.0)),
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

    fn render_hover_card_frame(
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

        let root =
            fret_ui::declarative::render_root(ui, app, services, window, bounds, "test", |cx| {
                let anchor_id_out_for_anchor = anchor_id_out.clone();
                let anchor = cx.pressable_with_id(
                    PressableProps {
                        layout: {
                            let mut layout = LayoutStyle::default();
                            layout.size.width = Length::Px(Px(50.0));
                            layout.size.height = Length::Px(Px(10.0));
                            layout.inset.top = Some(Px(120.0));
                            layout.inset.left = Some(Px(240.0));
                            layout.position = PositionStyle::Absolute;
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
                            HoverCardContent::new(vec![ui::raw_text(cx, "card").into_element(cx)])
                                .into_element(cx),
                        ]
                    },
                );
                content_id_out.set(Some(content.id));

                vec![
                    anchor,
                    HoverCard::new(trigger, content)
                        .anchor_element(anchor_id)
                        .align(HoverCardAlign::Start)
                        .open_delay_frames(0)
                        .side_offset(Px(8.0))
                        .window_margin(Px(0.0))
                        .into_element(cx),
                ]
            });

        ui.set_root(root);
        OverlayController::render(ui, app, services, window, bounds);
    }

    #[test]
    fn hover_card_anchor_override_uses_anchor_bounds_for_placement() {
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

        let mut services = FakeServices::default();
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            fret_core::Size::new(Px(800.0), Px(600.0)),
        );

        // Frame 1: establish bounds for the anchor + element/node mappings.
        app.set_frame_id(FrameId(1));
        app.set_tick_id(TickId(1));
        render_hover_card_frame(
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

        // Move pointer over the trigger to open.
        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::Pointer(fret_core::PointerEvent::Move {
                pointer_id: fret_core::PointerId(0),
                position: Point::new(Px(12.0), Px(12.0)),
                buttons: MouseButtons::default(),
                modifiers: fret_core::Modifiers::default(),
                pointer_type: fret_core::PointerType::Mouse,
            }),
        );

        // Frame 2: hover should request the overlay and mount the content.
        app.set_frame_id(FrameId(2));
        app.set_tick_id(TickId(2));
        render_hover_card_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            anchor_id.clone(),
            trigger_id.clone(),
            content_id.clone(),
        );
        ui.request_semantics_snapshot();
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        // The hover card uses a render-transform for shadcn-style open motion. Semantics bounds
        // track the transformed geometry, so advance a few frames to reach steady state before
        // asserting placement.
        let settle_frames: u64 = overlay_motion::SHADCN_MOTION_TICKS_100 + 2;
        for step in 0..settle_frames {
            let tick = 3 + step;
            app.set_frame_id(FrameId(tick));
            app.set_tick_id(TickId(tick));
            render_hover_card_frame(
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
        let anchor_bounds =
            fret_ui::elements::visual_bounds_for_element(&mut app, window, anchor_element)
                .or_else(|| fret_ui::elements::bounds_for_element(&mut app, window, anchor_element))
                .expect("anchor bounds");
        let desired = fret_core::Size::new(Px(256.0), Px(120.0));

        let layout = popper::popper_content_layout_sized(
            bounds,
            anchor_bounds,
            desired,
            popper::PopperContentPlacement::new(
                direction_prim::LayoutDirection::default(),
                overlay_placement::Side::Bottom,
                overlay_placement::Align::Start,
                Px(8.0),
            )
            .with_shift_cross_axis(true),
        );
        let expected = layout.rect;

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

    fn render_hover_card_focus_frame(
        ui: &mut UiTree<App>,
        app: &mut App,
        services: &mut dyn fret_core::UiServices,
        window: AppWindowId,
        bounds: Rect,
        trigger_id_out: Rc<Cell<Option<fret_ui::elements::GlobalElementId>>>,
        content_id_out: Rc<Cell<Option<fret_ui::elements::GlobalElementId>>>,
        after_id_out: Rc<Cell<Option<fret_ui::elements::GlobalElementId>>>,
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
                            HoverCardContent::new(vec![ui::raw_text(cx, "card").into_element(cx)])
                                .into_element(cx),
                        ]
                    },
                );
                content_id_out.set(Some(content.id));

                let after = cx.pressable_with_id(
                    PressableProps {
                        layout: {
                            let mut layout = LayoutStyle::default();
                            layout.size.width = Length::Px(Px(120.0));
                            layout.size.height = Length::Px(Px(40.0));
                            layout.inset.top = Some(Px(60.0));
                            layout.position = PositionStyle::Absolute;
                            layout
                        },
                        enabled: true,
                        focusable: true,
                        ..Default::default()
                    },
                    |_cx, _st, id| {
                        after_id_out.set(Some(id));
                        Vec::new()
                    },
                );

                vec![
                    HoverCard::new(trigger, content)
                        .open_delay_frames(0)
                        .close_delay_frames(0)
                        .into_element(cx),
                    after,
                ]
            });

        ui.set_root(root);
        OverlayController::render(ui, app, services, window, bounds);
    }

    #[test]
    fn hover_card_default_open_mounts_without_hover() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let trigger_id: Rc<Cell<Option<fret_ui::elements::GlobalElementId>>> =
            Rc::new(Cell::new(None));
        let content_id: Rc<Cell<Option<fret_ui::elements::GlobalElementId>>> =
            Rc::new(Cell::new(None));

        let mut services = FakeServices::default();
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            fret_core::Size::new(Px(800.0), Px(600.0)),
        );

        let render = |ui: &mut UiTree<App>,
                      app: &mut App,
                      services: &mut dyn fret_core::UiServices,
                      frame: u64| {
            let trigger_id_out = trigger_id.clone();
            let content_id_out = content_id.clone();
            app.set_frame_id(FrameId(frame));
            OverlayController::begin_frame(app, window);
            let root = fret_ui::declarative::render_root(
                ui,
                app,
                services,
                window,
                bounds,
                "test",
                move |cx| {
                    let trigger = cx.pressable_with_id_props(move |cx, _st, id| {
                        trigger_id_out.set(Some(id));
                        (
                            PressableProps {
                                layout: {
                                    let mut layout = LayoutStyle::default();
                                    layout.size.width = Length::Px(Px(120.0));
                                    layout.size.height = Length::Px(Px(40.0));
                                    layout
                                },
                                enabled: true,
                                focusable: true,
                                ..Default::default()
                            },
                            vec![cx.container(ContainerProps::default(), |_cx| Vec::new())],
                        )
                    });

                    let content = cx.semantics(
                        SemanticsProps {
                            role: SemanticsRole::Panel,
                            ..Default::default()
                        },
                        |cx| {
                            vec![
                                HoverCardContent::new(vec![
                                    ui::raw_text(cx, "card").into_element(cx),
                                ])
                                .into_element(cx),
                            ]
                        },
                    );
                    content_id_out.set(Some(content.id));

                    vec![
                        HoverCard::new(trigger, content)
                            .default_open(true)
                            .open_delay_frames(0)
                            .close_delay_frames(0)
                            .into_element(cx),
                    ]
                },
            );
            ui.set_root(root);
            OverlayController::render(ui, app, services, window, bounds);
        };

        // Frame 1: establish trigger bounds (placement resolves from last-frame layout).
        render(&mut ui, &mut app, &mut services, 1);
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        // Frame 2: default_open should mount the overlay.
        render(&mut ui, &mut app, &mut services, 2);
        ui.request_semantics_snapshot();
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        let trigger_element = trigger_id.get().expect("trigger element id");
        let last_trigger_bounds = fret_ui::elements::with_element_cx(
            &mut app,
            window,
            bounds,
            "hover-card-default-open-probe",
            |cx| cx.last_bounds_for_element(trigger_element),
        );
        assert!(
            last_trigger_bounds.is_some(),
            "expected trigger to have last-frame bounds for placement"
        );

        let content_element = content_id.get().expect("content element id");
        let content_node = fret_ui::elements::node_for_element(&mut app, window, content_element)
            .expect("content node");
        let snap = ui.semantics_snapshot().expect("semantics snapshot");
        assert!(
            snap.nodes.iter().any(|n| n.id == content_node),
            "expected hover card content to mount when default_open=true"
        );
    }

    #[test]
    fn hover_card_opens_on_focus_and_closes_on_blur() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let trigger_id: Rc<Cell<Option<fret_ui::elements::GlobalElementId>>> =
            Rc::new(Cell::new(None));
        let content_id: Rc<Cell<Option<fret_ui::elements::GlobalElementId>>> =
            Rc::new(Cell::new(None));
        let after_id: Rc<Cell<Option<fret_ui::elements::GlobalElementId>>> =
            Rc::new(Cell::new(None));

        let mut services = FakeServices::default();
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            fret_core::Size::new(Px(800.0), Px(600.0)),
        );

        // Frame 1: mount trigger/after and resolve element/node mappings.
        app.set_frame_id(FrameId(1));
        render_hover_card_focus_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            trigger_id.clone(),
            content_id.clone(),
            after_id.clone(),
        );
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        let trigger_element = trigger_id.get().expect("trigger element id");
        let trigger_node = fret_ui::elements::node_for_element(&mut app, window, trigger_element)
            .expect("trigger node");
        // Focus-driven hover cards are a keyboard affordance; mirror the runtime input-modality
        // signal that Radix would receive via key interaction (e.g. tabbing).
        let _ = fret_ui::input_modality::update_for_event(
            &mut app,
            window,
            &fret_core::Event::KeyDown {
                key: fret_core::KeyCode::Tab,
                modifiers: fret_core::Modifiers::default(),
                repeat: false,
            },
        );
        ui.set_focus(Some(trigger_node));

        // Frame 2: focus should open the overlay and mount the content.
        app.set_frame_id(FrameId(2));
        render_hover_card_focus_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            trigger_id.clone(),
            content_id.clone(),
            after_id.clone(),
        );
        ui.request_semantics_snapshot();
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        let content_element = content_id.get().expect("content element id");
        let content_node = fret_ui::elements::node_for_element(&mut app, window, content_element)
            .expect("content node");
        let snap = ui.semantics_snapshot().expect("semantics snapshot");
        assert!(
            snap.nodes.iter().any(|n| n.id == content_node),
            "expected hover card content to mount when trigger is focused"
        );

        // Blur by moving focus elsewhere, then wait for the exit animation to complete.
        let after_element = after_id.get().expect("after element id");
        let after_node = fret_ui::elements::node_for_element(&mut app, window, after_element)
            .expect("after node");
        ui.set_focus(Some(after_node));
        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::Pointer(fret_core::PointerEvent::Move {
                pointer_id: fret_core::PointerId(0),
                position: Point::new(Px(2000.0), Px(2000.0)),
                buttons: MouseButtons::default(),
                modifiers: fret_core::Modifiers::default(),
                pointer_type: fret_core::PointerType::Mouse,
            }),
        );

        let settle_frames = crate::overlay_motion::SHADCN_MOTION_TICKS_100 + 1;
        for i in 0..settle_frames {
            app.set_frame_id(FrameId(3 + i));
            render_hover_card_focus_frame(
                &mut ui,
                &mut app,
                &mut services,
                window,
                bounds,
                trigger_id.clone(),
                content_id.clone(),
                after_id.clone(),
            );
            ui.layout_all(&mut app, &mut services, bounds, 1.0);
        }

        ui.request_semantics_snapshot();
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        let snap = ui.semantics_snapshot().expect("semantics snapshot");
        assert!(
            !snap.nodes.iter().any(|n| n.id == content_node),
            "expected hover card content to unmount after blur"
        );
    }

    #[test]
    fn hover_card_does_not_close_while_pointer_down_on_content() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let open = app.models_mut().insert(false);

        let content_id: Rc<Cell<Option<fret_ui::elements::GlobalElementId>>> =
            Rc::new(Cell::new(None));

        let mut services = FakeServices::default();
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            fret_core::Size::new(Px(800.0), Px(600.0)),
        );

        let render = |ui: &mut UiTree<App>,
                      app: &mut App,
                      services: &mut dyn fret_core::UiServices,
                      frame: u64| {
            let content_id_out = content_id.clone();
            let open = open.clone();
            app.set_frame_id(FrameId(frame));
            OverlayController::begin_frame(app, window);
            let root = fret_ui::declarative::render_root(
                ui,
                app,
                services,
                window,
                bounds,
                "test",
                move |cx| {
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
                            ..Default::default()
                        },
                        |cx, _st, _id| {
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
                                HoverCardContent::new(vec![
                                    ui::raw_text(cx, "card").into_element(cx),
                                ])
                                .into_element(cx),
                            ]
                        },
                    );
                    content_id_out.set(Some(content.id));

                    vec![
                        HoverCard::new(trigger, content)
                            .open(Some(open))
                            .open_delay_frames(0)
                            .close_delay_frames(0)
                            .refine_layout(
                                LayoutRefinement::default().w_px(Px(120.0)).h_px(Px(40.0)),
                            )
                            .window_margin(Px(0.0))
                            .into_element(cx),
                    ]
                },
            );
            ui.set_root(root);
            OverlayController::render(ui, app, services, window, bounds);
        };

        // Frame 1: mount trigger and establish element/node mappings.
        render(&mut ui, &mut app, &mut services, 1);
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        // Hover trigger to open (open_delay=0).
        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::Pointer(fret_core::PointerEvent::Move {
                pointer_id: fret_core::PointerId(0),
                position: Point::new(Px(12.0), Px(12.0)),
                buttons: MouseButtons::default(),
                modifiers: fret_core::Modifiers::default(),
                pointer_type: fret_core::PointerType::Mouse,
            }),
        );

        // Frame 2: open model should flip to true.
        render(&mut ui, &mut app, &mut services, 2);
        ui.request_semantics_snapshot();
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        let is_open = app.models().read(&open, |v| *v).expect("open");
        assert!(is_open, "expected hover card to open on hover");

        let content_element = content_id.get().expect("content element id");
        let content_node = fret_ui::elements::node_for_element(&mut app, window, content_element)
            .expect("content node");
        let snap = ui.semantics_snapshot().expect("semantics snapshot");
        let content_bounds = snap
            .nodes
            .iter()
            .find(|n| n.id == content_node)
            .map(|n| n.bounds)
            .expect("content bounds");

        // Pointer down on the content, then drag out.
        //
        // Note: the hover card subtree may be wrapped in a render transform (motion), so the
        // semantics snapshot bounds are not guaranteed to map 1:1 to interactive hit testing.
        // Find an actual hit-testable point within the content subtree.
        let mut down_pos: Option<Point> = None;
        for y in (0..=bounds.size.height.0 as i32).step_by(8) {
            for x in (0..=bounds.size.width.0 as i32).step_by(8) {
                let pos = Point::new(Px(x as f32), Px(y as f32));
                let Some(hit) = ui.debug_hit_test(pos).hit else {
                    continue;
                };
                if ui.debug_node_path(hit).contains(&content_node) {
                    down_pos = Some(pos);
                    break;
                }
            }
            if down_pos.is_some() {
                break;
            }
        }
        let down_pos = down_pos.unwrap_or_else(|| {
            let fallback = Point::new(
                Px(content_bounds.origin.x.0 + 1.0),
                Px(content_bounds.origin.y.0 + 1.0),
            );
            panic!(
                "failed to find hover card content hit target; fallback={:?} hit={:?}",
                fallback,
                ui.debug_hit_test(fallback)
            );
        });
        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::Pointer(fret_core::PointerEvent::Down {
                pointer_id: fret_core::PointerId(0),
                position: down_pos,
                button: fret_core::MouseButton::Left,
                modifiers: fret_core::Modifiers::default(),
                click_count: 1,
                pointer_type: fret_core::PointerType::Mouse,
            }),
        );
        let outside = Point::new(Px(400.0), Px(400.0));
        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::Pointer(fret_core::PointerEvent::Move {
                pointer_id: fret_core::PointerId(0),
                position: outside,
                buttons: MouseButtons {
                    left: true,
                    right: false,
                    middle: false,
                },
                modifiers: fret_core::Modifiers::default(),
                pointer_type: fret_core::PointerType::Mouse,
            }),
        );

        // Frame 3: should remain open (close_delay=0, but pointer is down).
        render(&mut ui, &mut app, &mut services, 3);
        ui.layout_all(&mut app, &mut services, bounds, 1.0);
        let is_open = app.models().read(&open, |v| *v).expect("open");
        assert!(
            is_open,
            "expected hover card to remain open while pointer is down"
        );

        // Pointer up outside should not immediately close; Radix does not schedule close during pointer down.
        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::Pointer(fret_core::PointerEvent::Up {
                pointer_id: fret_core::PointerId(0),
                position: outside,
                button: fret_core::MouseButton::Left,
                modifiers: fret_core::Modifiers::default(),
                is_click: true,
                click_count: 1,
                pointer_type: fret_core::PointerType::Mouse,
            }),
        );
        render(&mut ui, &mut app, &mut services, 4);
        ui.layout_all(&mut app, &mut services, bounds, 1.0);
        let is_open = app.models().read(&open, |v| *v).expect("open");
        assert!(
            is_open,
            "expected hover card to keep open after drag out release"
        );

        // Re-enter the hover card content, then leave to close (close_delay=0).
        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::Pointer(fret_core::PointerEvent::Move {
                pointer_id: fret_core::PointerId(0),
                position: down_pos,
                buttons: MouseButtons::default(),
                modifiers: fret_core::Modifiers::default(),
                pointer_type: fret_core::PointerType::Mouse,
            }),
        );
        render(&mut ui, &mut app, &mut services, 5);
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::Pointer(fret_core::PointerEvent::Move {
                pointer_id: fret_core::PointerId(0),
                position: outside,
                buttons: MouseButtons::default(),
                modifiers: fret_core::Modifiers::default(),
                pointer_type: fret_core::PointerType::Mouse,
            }),
        );
        render(&mut ui, &mut app, &mut services, 6);
        ui.layout_all(&mut app, &mut services, bounds, 1.0);
        // The overlay-hover flag is updated while rendering overlay children, so allow one extra
        // frame for the leave to reflect in the root driver before asserting close.
        render(&mut ui, &mut app, &mut services, 7);
        ui.layout_all(&mut app, &mut services, bounds, 1.0);
        let is_open = app.models().read(&open, |v| *v).expect("open");
        assert!(!is_open, "expected hover card to close after leave edge");
    }

    #[test]
    fn hover_card_does_not_close_while_text_selection_active() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let open = app.models_mut().insert(false);

        let content_id: Rc<Cell<Option<fret_ui::elements::GlobalElementId>>> =
            Rc::new(Cell::new(None));
        let selectable_id: Rc<Cell<Option<fret_ui::elements::GlobalElementId>>> =
            Rc::new(Cell::new(None));

        let mut services = FakeServices::default();
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            fret_core::Size::new(Px(800.0), Px(600.0)),
        );

        let render = |ui: &mut UiTree<App>,
                      app: &mut App,
                      services: &mut dyn fret_core::UiServices,
                      frame: u64| {
            let content_id_out = content_id.clone();
            let selectable_id_out = selectable_id.clone();
            let open = open.clone();
            app.set_frame_id(FrameId(frame));
            OverlayController::begin_frame(app, window);
            let root = fret_ui::declarative::render_root(
                ui,
                app,
                services,
                window,
                bounds,
                "hover-card-text-selection-guard",
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
                            ..Default::default()
                        },
                        |cx, _st, _id| {
                            vec![cx.container(ContainerProps::default(), |_cx| Vec::new())]
                        },
                    );

                    let text = Arc::<str>::from("hello world");
                    let spans =
                        Arc::<[fret_core::TextSpan]>::from([fret_core::TextSpan::new(text.len())]);
                    let rich = fret_core::AttributedText::new(text, spans);

                    let content = cx.semantics(
                        SemanticsProps {
                            role: SemanticsRole::Panel,
                            ..Default::default()
                        },
                        |cx| {
                            vec![
                                HoverCardContent::new(vec![{
                                    let selectable = cx.selectable_text(rich.clone());
                                    selectable_id_out.set(Some(selectable.id));
                                    selectable
                                }])
                                .into_element(cx),
                            ]
                        },
                    );
                    content_id_out.set(Some(content.id));

                    vec![
                        HoverCard::new(trigger, content)
                            .open(Some(open))
                            .open_delay_frames(0)
                            .close_delay_frames(0)
                            .refine_layout(
                                LayoutRefinement::default().w_px(Px(120.0)).h_px(Px(40.0)),
                            )
                            .window_margin(Px(0.0))
                            .into_element(cx),
                    ]
                },
            );
            ui.set_root(root);
            OverlayController::render(ui, app, services, window, bounds);
        };

        // Frame 1: mount trigger and establish element/node mappings.
        render(&mut ui, &mut app, &mut services, 1);
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        // Hover trigger to open (open_delay=0).
        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::Pointer(fret_core::PointerEvent::Move {
                pointer_id: fret_core::PointerId(0),
                position: Point::new(Px(12.0), Px(12.0)),
                buttons: MouseButtons::default(),
                modifiers: fret_core::Modifiers::default(),
                pointer_type: fret_core::PointerType::Mouse,
            }),
        );

        // Frame 2: open model should flip to true.
        render(&mut ui, &mut app, &mut services, 2);
        ui.request_semantics_snapshot();
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        let is_open = app.models().read(&open, |v| *v).expect("open");
        assert!(is_open, "expected hover card to open on hover");

        let selectable_element = selectable_id.get().expect("selectable element id");
        let selectable_node =
            fret_ui::elements::node_for_element(&mut app, window, selectable_element)
                .expect("selectable node");
        let snap = ui.semantics_snapshot().expect("semantics snapshot");
        let selectable_bounds = ui
            .debug_node_visual_bounds(selectable_node)
            .or_else(|| ui.debug_node_bounds(selectable_node))
            .or_else(|| {
                snap.nodes
                    .iter()
                    .find(|n| n.id == selectable_node)
                    .map(|n| n.bounds)
            })
            .expect("selectable bounds");

        // Select text (double click selects the first word), then leave the hover card.
        let select_pos = Point::new(
            Px(selectable_bounds.origin.x.0 + 1.0),
            Px(selectable_bounds.origin.y.0 + 1.0),
        );
        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::Pointer(fret_core::PointerEvent::Down {
                pointer_id: fret_core::PointerId(0),
                position: select_pos,
                button: fret_core::MouseButton::Left,
                modifiers: fret_core::Modifiers::default(),
                click_count: 2,
                pointer_type: fret_core::PointerType::Mouse,
            }),
        );
        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::Pointer(fret_core::PointerEvent::Up {
                pointer_id: fret_core::PointerId(0),
                position: select_pos,
                button: fret_core::MouseButton::Left,
                modifiers: fret_core::Modifiers::default(),
                is_click: true,
                click_count: 2,
                pointer_type: fret_core::PointerType::Mouse,
            }),
        );
        let (anchor, caret) = fret_ui::elements::with_element_state(
            &mut app,
            window,
            selectable_element,
            fret_ui::element::SelectableTextState::default,
            |state| (state.selection_anchor, state.caret),
        );
        assert_ne!(
            anchor, caret,
            "expected selectable text to have an active selection after double click"
        );
        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::Pointer(fret_core::PointerEvent::Up {
                pointer_id: fret_core::PointerId(0),
                position: select_pos,
                button: fret_core::MouseButton::Left,
                modifiers: fret_core::Modifiers::default(),
                is_click: true,
                click_count: 2,
                pointer_type: fret_core::PointerType::Mouse,
            }),
        );

        let outside = Point::new(Px(400.0), Px(400.0));
        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::Pointer(fret_core::PointerEvent::Move {
                pointer_id: fret_core::PointerId(0),
                position: outside,
                buttons: MouseButtons::default(),
                modifiers: fret_core::Modifiers::default(),
                pointer_type: fret_core::PointerType::Mouse,
            }),
        );

        // Frame 3/4: allow leave to propagate; selection should keep the hover card open.
        render(&mut ui, &mut app, &mut services, 3);
        ui.layout_all(&mut app, &mut services, bounds, 1.0);
        render(&mut ui, &mut app, &mut services, 4);
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        let is_open = app.models().read(&open, |v| *v).expect("open");
        assert!(
            is_open,
            "expected hover card to remain open while a text selection is active"
        );

        // Collapse selection; the hover card should close on the next frame.
        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::SetTextSelection {
                anchor: 0,
                focus: 0,
            },
        );
        let (anchor, caret) = fret_ui::elements::with_element_state(
            &mut app,
            window,
            selectable_element,
            fret_ui::element::SelectableTextState::default,
            |state| (state.selection_anchor, state.caret),
        );
        assert_eq!(
            (anchor, caret),
            (0, 0),
            "expected selection to collapse before asserting hover card close"
        );
        render(&mut ui, &mut app, &mut services, 5);
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        let is_open = app.models().read(&open, |v| *v).expect("open");
        assert!(
            !is_open,
            "expected hover card to close after selection is cleared"
        );

        // Keep IDs live to avoid surprising drop-order side effects in future refactors.
        let _ = content_id.get().expect("content element id");
    }

    #[test]
    fn hover_card_close_transition_is_click_through() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let open = app.models_mut().insert(false);
        let underlay_clicked = app.models_mut().insert(false);

        let underlay_id: Rc<Cell<Option<fret_ui::elements::GlobalElementId>>> =
            Rc::new(Cell::new(None));
        let content_id: Rc<Cell<Option<fret_ui::elements::GlobalElementId>>> =
            Rc::new(Cell::new(None));

        let mut services = FakeServices::default();
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            fret_core::Size::new(Px(800.0), Px(600.0)),
        );

        fn render_frame(
            ui: &mut UiTree<App>,
            app: &mut App,
            services: &mut dyn fret_core::UiServices,
            window: AppWindowId,
            bounds: Rect,
            open: Model<bool>,
            underlay_clicked: Model<bool>,
            underlay_id: Rc<Cell<Option<fret_ui::elements::GlobalElementId>>>,
            content_id: Rc<Cell<Option<fret_ui::elements::GlobalElementId>>>,
            frame: u64,
        ) {
            app.set_frame_id(FrameId(frame));
            app.set_tick_id(TickId(frame));

            OverlayController::begin_frame(app, window);
            let root = fret_ui::declarative::render_root(
                ui,
                app,
                services,
                window,
                bounds,
                "hover-card-close-transition-click-through",
                |cx| {
                    let underlay_id_out = underlay_id.clone();
                    let content_id_out = content_id.clone();
                    let underlay_clicked = underlay_clicked.clone();
                    let open = open.clone();

                    vec![cx.container(
                        ContainerProps {
                            layout: {
                                let mut layout = LayoutStyle::default();
                                layout.position = PositionStyle::Relative;
                                layout.size.width = Length::Fill;
                                layout.size.height = Length::Fill;
                                layout
                            },
                            ..Default::default()
                        },
                        move |cx| {
                            let underlay = cx.pressable_with_id(
                                PressableProps {
                                    layout: {
                                        let mut layout = LayoutStyle::default();
                                        layout.position = PositionStyle::Absolute;
                                        layout.inset.left = Some(Px(0.0));
                                        layout.inset.top = Some(Px(0.0));
                                        layout.size.width = Length::Fill;
                                        layout.size.height = Length::Fill;
                                        layout
                                    },
                                    enabled: true,
                                    focusable: true,
                                    ..Default::default()
                                },
                                {
                                    let underlay_id_out = underlay_id_out.clone();
                                    let underlay_clicked = underlay_clicked.clone();
                                    move |cx, _st, id| {
                                        underlay_id_out.set(Some(id));
                                        cx.pressable_toggle_bool(&underlay_clicked);
                                        Vec::new()
                                    }
                                },
                            );

                            let trigger = cx.pressable(
                                PressableProps {
                                    layout: {
                                        let mut layout = LayoutStyle::default();
                                        layout.position = PositionStyle::Absolute;
                                        layout.inset.left = Some(Px(20.0));
                                        layout.inset.top = Some(Px(20.0));
                                        layout.size.width = Length::Px(Px(120.0));
                                        layout.size.height = Length::Px(Px(40.0));
                                        layout
                                    },
                                    enabled: true,
                                    focusable: true,
                                    ..Default::default()
                                },
                                |_cx, _st| Vec::new(),
                            );

                            let content = cx.semantics(
                                SemanticsProps {
                                    role: SemanticsRole::Panel,
                                    ..Default::default()
                                },
                                |cx| {
                                    vec![
                                        HoverCardContent::new(vec![
                                            ui::raw_text(cx, "card").into_element(cx),
                                        ])
                                        .into_element(cx),
                                    ]
                                },
                            );
                            content_id_out.set(Some(content.id));

                            vec![
                                underlay,
                                HoverCard::new(trigger, content)
                                    .open(Some(open))
                                    .open_delay_frames(0)
                                    .close_delay_frames(0)
                                    .refine_layout(
                                        LayoutRefinement::default().w_px(Px(120.0)).h_px(Px(40.0)),
                                    )
                                    .window_margin(Px(0.0))
                                    .into_element(cx),
                            ]
                        },
                    )]
                },
            );

            ui.set_root(root);
            OverlayController::render(ui, app, services, window, bounds);
            ui.request_semantics_snapshot();
            ui.layout_all(app, services, bounds, 1.0);
        }

        // Frame 1: closed; establish element->node mapping for the underlay.
        render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            open.clone(),
            underlay_clicked.clone(),
            underlay_id.clone(),
            content_id.clone(),
            1,
        );
        let underlay_element = underlay_id.get().expect("underlay element id");
        let underlay_node = fret_ui::elements::node_for_element(&mut app, window, underlay_element)
            .expect("underlay node");

        // Frame 2: open and capture content bounds.
        let _ = app.models_mut().update(&open, |v| *v = true);
        render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            open.clone(),
            underlay_clicked.clone(),
            underlay_id.clone(),
            content_id.clone(),
            2,
        );
        let content_element = content_id.get().expect("content element id");
        let content_node = fret_ui::elements::node_for_element(&mut app, window, content_element)
            .expect("content node");
        let snap = ui.semantics_snapshot().expect("semantics snapshot");
        let content_bounds = snap
            .nodes
            .iter()
            .find(|n| n.id == content_node)
            .map(|n| n.bounds)
            .expect("content bounds");

        let click_pos = Point::new(
            Px(content_bounds.origin.x.0 + content_bounds.size.width.0 * 0.5),
            Px(content_bounds.origin.y.0 + content_bounds.size.height.0 * 0.5),
        );

        // Frame 3: start close transition (present=true, interactive=false).
        let _ = app.models_mut().update(&open, |v| *v = false);
        render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            open.clone(),
            underlay_clicked.clone(),
            underlay_id.clone(),
            content_id.clone(),
            3,
        );

        // Sanity: content is still mounted during the fade-out.
        assert!(
            fret_ui::elements::node_for_element(&mut app, window, content_element).is_some(),
            "expected hover card content to remain mounted during close transition"
        );
        let content_node = fret_ui::elements::node_for_element(&mut app, window, content_element)
            .expect("content node");
        let content_layer = ui.node_layer(content_node).expect("content layer");
        let layer = ui
            .debug_layers_in_paint_order()
            .into_iter()
            .find(|l| l.id == content_layer)
            .expect("overlay layer");
        assert!(layer.visible);
        assert!(!layer.hit_testable);
        assert_eq!(
            layer.pointer_occlusion,
            fret_ui::tree::PointerOcclusion::None
        );
        assert!(!layer.wants_pointer_move_events);
        assert!(!layer.wants_timer_events);

        ui.dispatch_event(
            &mut app,
            &mut services,
            &Event::Pointer(fret_core::PointerEvent::Down {
                pointer_id: fret_core::PointerId(0),
                position: click_pos,
                button: MouseButton::Left,
                modifiers: Modifiers::default(),
                pointer_type: fret_core::PointerType::Mouse,
                click_count: 1,
            }),
        );
        ui.dispatch_event(
            &mut app,
            &mut services,
            &Event::Pointer(fret_core::PointerEvent::Up {
                pointer_id: fret_core::PointerId(0),
                position: click_pos,
                button: MouseButton::Left,
                modifiers: Modifiers::default(),
                is_click: true,
                pointer_type: fret_core::PointerType::Mouse,
                click_count: 1,
            }),
        );

        // Frame 4: observe click-through effects via the underlay pressable helper.
        render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            open.clone(),
            underlay_clicked.clone(),
            underlay_id,
            content_id,
            4,
        );

        assert_eq!(
            app.models().get_copied(&open),
            Some(false),
            "expected hover card to remain closed after click-through while closing"
        );
        assert_eq!(
            app.models().get_copied(&underlay_clicked),
            Some(true),
            "expected close transition click to reach the underlay"
        );
        assert_eq!(
            ui.focus(),
            Some(underlay_node),
            "expected focus to move to the underlay during click-through close transition"
        );
    }

    #[test]
    fn hover_card_outside_press_is_click_through_and_closes_on_leave() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let open = app.models_mut().insert(false);
        let underlay_clicked = app.models_mut().insert(false);
        let underlay_id: Rc<Cell<Option<fret_ui::elements::GlobalElementId>>> =
            Rc::new(Cell::new(None));

        let mut services = FakeServices::default();
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            fret_core::Size::new(Px(800.0), Px(600.0)),
        );

        let render = |ui: &mut UiTree<App>,
                      app: &mut App,
                      services: &mut dyn fret_core::UiServices,
                      frame: u64| {
            app.set_frame_id(FrameId(frame));
            app.set_tick_id(TickId(frame));

            OverlayController::begin_frame(app, window);
            let root = fret_ui::declarative::render_root(
                ui,
                app,
                services,
                window,
                bounds,
                "hover-card-outside-press-click-through",
                |cx| {
                    let underlay_id_out = underlay_id.clone();
                    let underlay_clicked = underlay_clicked.clone();
                    let open = open.clone();

                    vec![cx.container(
                        ContainerProps {
                            layout: {
                                let mut layout = LayoutStyle::default();
                                layout.position = PositionStyle::Relative;
                                layout.size.width = Length::Fill;
                                layout.size.height = Length::Fill;
                                layout
                            },
                            ..Default::default()
                        },
                        move |cx| {
                            let underlay = cx.pressable_with_id(
                                PressableProps {
                                    layout: {
                                        let mut layout = LayoutStyle::default();
                                        layout.position = PositionStyle::Absolute;
                                        layout.inset.left = Some(Px(600.0));
                                        layout.inset.top = Some(Px(420.0));
                                        layout.size.width = Length::Px(Px(160.0));
                                        layout.size.height = Length::Px(Px(80.0));
                                        layout
                                    },
                                    enabled: true,
                                    focusable: true,
                                    ..Default::default()
                                },
                                {
                                    let underlay_id_out = underlay_id_out.clone();
                                    let underlay_clicked = underlay_clicked.clone();
                                    move |cx, _st, id| {
                                        underlay_id_out.set(Some(id));
                                        cx.pressable_toggle_bool(&underlay_clicked);
                                        Vec::new()
                                    }
                                },
                            );

                            let trigger = cx.pressable(
                                PressableProps {
                                    layout: {
                                        let mut layout = LayoutStyle::default();
                                        layout.position = PositionStyle::Absolute;
                                        layout.inset.left = Some(Px(0.0));
                                        layout.inset.top = Some(Px(0.0));
                                        layout.size.width = Length::Px(Px(120.0));
                                        layout.size.height = Length::Px(Px(40.0));
                                        layout
                                    },
                                    enabled: true,
                                    focusable: true,
                                    ..Default::default()
                                },
                                |_cx, _st| Vec::new(),
                            );

                            let content = cx.semantics(
                                SemanticsProps {
                                    role: SemanticsRole::Panel,
                                    ..Default::default()
                                },
                                |cx| {
                                    vec![
                                        HoverCardContent::new(vec![
                                            ui::raw_text(cx, "card").into_element(cx),
                                        ])
                                        .into_element(cx),
                                    ]
                                },
                            );

                            vec![
                                underlay,
                                HoverCard::new(trigger, content)
                                    .open(Some(open))
                                    .open_delay_frames(0)
                                    .close_delay_frames(0)
                                    .refine_layout(
                                        LayoutRefinement::default().w_px(Px(120.0)).h_px(Px(40.0)),
                                    )
                                    .window_margin(Px(0.0))
                                    .into_element(cx),
                            ]
                        },
                    )]
                },
            );

            ui.set_root(root);
            OverlayController::render(ui, app, services, window, bounds);
        };

        // Frame 1: mount and establish element/node mappings.
        render(&mut ui, &mut app, &mut services, 1);
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        let underlay_element = underlay_id.get().expect("underlay element id");
        let underlay_node = fret_ui::elements::node_for_element(&mut app, window, underlay_element)
            .expect("underlay node");

        // Hover trigger to open (open_delay=0).
        ui.dispatch_event(
            &mut app,
            &mut services,
            &Event::Pointer(fret_core::PointerEvent::Move {
                pointer_id: fret_core::PointerId(0),
                position: Point::new(Px(12.0), Px(12.0)),
                buttons: MouseButtons::default(),
                modifiers: Modifiers::default(),
                pointer_type: fret_core::PointerType::Mouse,
            }),
        );

        // Frame 2: open model should flip to true.
        render(&mut ui, &mut app, &mut services, 2);
        ui.layout_all(&mut app, &mut services, bounds, 1.0);
        assert_eq!(
            app.models().get_copied(&open),
            Some(true),
            "expected hover card to open on hover"
        );

        let underlay_pos = Point::new(Px(680.0), Px(460.0));
        ui.dispatch_event(
            &mut app,
            &mut services,
            &Event::Pointer(fret_core::PointerEvent::Move {
                pointer_id: fret_core::PointerId(0),
                position: underlay_pos,
                buttons: MouseButtons::default(),
                modifiers: Modifiers::default(),
                pointer_type: fret_core::PointerType::Mouse,
            }),
        );
        ui.dispatch_event(
            &mut app,
            &mut services,
            &Event::Pointer(fret_core::PointerEvent::Down {
                pointer_id: fret_core::PointerId(0),
                position: underlay_pos,
                button: MouseButton::Left,
                modifiers: Modifiers::default(),
                pointer_type: fret_core::PointerType::Mouse,
                click_count: 1,
            }),
        );
        ui.dispatch_event(
            &mut app,
            &mut services,
            &Event::Pointer(fret_core::PointerEvent::Up {
                pointer_id: fret_core::PointerId(0),
                position: underlay_pos,
                button: MouseButton::Left,
                modifiers: Modifiers::default(),
                is_click: true,
                pointer_type: fret_core::PointerType::Mouse,
                click_count: 1,
            }),
        );

        // Frame 3/4: allow leave to propagate; hover card should close and the outside click should
        // activate + focus the underlay (click-through, non-modal).
        render(&mut ui, &mut app, &mut services, 3);
        ui.layout_all(&mut app, &mut services, bounds, 1.0);
        render(&mut ui, &mut app, &mut services, 4);
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        assert_eq!(
            app.models().get_copied(&underlay_clicked),
            Some(true),
            "expected outside press to reach the underlay (click-through)"
        );
        assert_eq!(
            ui.focus(),
            Some(underlay_node),
            "expected focus to move to the underlay on click-through outside press"
        );
        assert_eq!(
            app.models().get_copied(&open),
            Some(false),
            "expected hover card to close after leaving the hover region"
        );

        let arbitration = OverlayController::arbitration_snapshot(&ui);
        assert_eq!(
            arbitration.pointer_occlusion,
            fret_ui::tree::PointerOcclusion::None
        );
    }

    #[test]
    fn hover_card_keeps_open_while_pointer_moves_through_safe_corridor() {
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
        let mut services = FakeServices;

        let open = app.models_mut().insert(false);
        let trigger_id: Rc<Cell<Option<fret_ui::elements::GlobalElementId>>> =
            Rc::new(Cell::new(None));
        let content_probe_id: Rc<Cell<Option<fret_ui::elements::GlobalElementId>>> =
            Rc::new(Cell::new(None));

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            fret_core::Size::new(Px(800.0), Px(600.0)),
        );

        let render_frame =
            |ui: &mut UiTree<App>, app: &mut App, services: &mut FakeServices, frame: u64| {
                app.set_frame_id(FrameId(frame));
                OverlayController::begin_frame(app, window);
                let root = fret_ui::declarative::render_root(
                    ui,
                    app,
                    services,
                    window,
                    bounds,
                    "hover-card-safe-corridor",
                    |cx| {
                        let trigger_id = trigger_id.clone();
                        let content_probe_id = content_probe_id.clone();

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
                                ..Default::default()
                            },
                            move |cx, _st, id| {
                                trigger_id.set(Some(id));
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
                                    HoverCardContent::new(vec![
                                        ui::raw_text(cx, "card").into_element(cx),
                                    ])
                                    .into_element(cx),
                                ]
                            },
                        );
                        content_probe_id.set(Some(content.id));

                        vec![
                            HoverCard::new(trigger, content)
                                .open(Some(open.clone()))
                                .open_delay_frames(0)
                                .close_delay_frames(0)
                                .side(HoverCardSide::Bottom)
                                .side_offset(Px(8.0))
                                .window_margin(Px(0.0))
                                .into_element(cx),
                        ]
                    },
                );
                ui.set_root(root);
                OverlayController::render(ui, app, services, window, bounds);
            };

        render_frame(&mut ui, &mut app, &mut services, 1);
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        let trigger_element = trigger_id.get().expect("trigger element id");
        let trigger_node = fret_ui::elements::node_for_element(&mut app, window, trigger_element)
            .expect("trigger node");
        let trigger_bounds = ui.debug_node_bounds(trigger_node).expect("trigger bounds");

        ui.dispatch_event(
            &mut app,
            &mut services,
            &Event::Pointer(fret_core::PointerEvent::Move {
                pointer_id: fret_core::PointerId(0),
                position: center(trigger_bounds),
                buttons: MouseButtons::default(),
                modifiers: Modifiers::default(),
                pointer_type: fret_core::PointerType::Mouse,
            }),
        );

        render_frame(&mut ui, &mut app, &mut services, 2);
        ui.layout_all(&mut app, &mut services, bounds, 1.0);
        assert_eq!(app.models().get_copied(&open), Some(true));

        let content_probe_element = content_probe_id.get().expect("content probe element id");
        let content_probe_node =
            fret_ui::elements::node_for_element(&mut app, window, content_probe_element)
                .expect("content probe node");
        let content_probe_bounds = ui
            .debug_node_bounds(content_probe_node)
            .expect("content probe bounds");

        let transit_point = Point::new(
            Px(trigger_bounds.origin.x.0 + trigger_bounds.size.width.0 * 0.5),
            Px(trigger_bounds.origin.y.0 + trigger_bounds.size.height.0 + 2.0),
        );
        assert!(
            !trigger_bounds.contains(transit_point),
            "transit point should be outside trigger bounds"
        );
        assert!(
            !content_probe_bounds.contains(transit_point),
            "transit point should be outside floating content bounds"
        );

        ui.dispatch_event(
            &mut app,
            &mut services,
            &Event::Pointer(fret_core::PointerEvent::Move {
                pointer_id: fret_core::PointerId(0),
                position: transit_point,
                buttons: MouseButtons::default(),
                modifiers: Modifiers::default(),
                pointer_type: fret_core::PointerType::Mouse,
            }),
        );

        render_frame(&mut ui, &mut app, &mut services, 3);
        ui.layout_all(&mut app, &mut services, bounds, 1.0);
        assert_eq!(app.models().get_copied(&open), Some(true));

        ui.dispatch_event(
            &mut app,
            &mut services,
            &Event::Pointer(fret_core::PointerEvent::Move {
                pointer_id: fret_core::PointerId(0),
                position: Point::new(Px(760.0), Px(560.0)),
                buttons: MouseButtons::default(),
                modifiers: Modifiers::default(),
                pointer_type: fret_core::PointerType::Mouse,
            }),
        );

        render_frame(&mut ui, &mut app, &mut services, 4);
        ui.layout_all(&mut app, &mut services, bounds, 1.0);
        render_frame(&mut ui, &mut app, &mut services, 5);
        ui.layout_all(&mut app, &mut services, bounds, 1.0);
        assert_eq!(app.models().get_copied(&open), Some(false));
    }
}
