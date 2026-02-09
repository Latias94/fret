use std::sync::Arc;
use std::{cell::Cell, rc::Rc};

use crate::popper_arrow::{self, DiamondArrowStyle};
use fret_core::{Edges, Point, Px, Rect, SemanticsRole, Size};
use fret_runtime::Model;
use fret_ui::action::{OnCloseAutoFocus, OnDismissRequest, OnOpenAutoFocus};
use fret_ui::element::{
    AnyElement, ContainerProps, ElementKind, HoverRegionProps, InteractivityGateProps, LayoutStyle,
    Length, OpacityProps, Overflow, SemanticsDecoration, VisualTransformProps,
};
use fret_ui::overlay_placement::{Align, Side};
use fret_ui::{ElementContext, Theme, UiHost};
use fret_ui_kit::declarative::model_watch::ModelWatchExt as _;
use fret_ui_kit::declarative::{scheduling, style as decl_style};
use fret_ui_kit::headless::safe_hover;
use fret_ui_kit::overlay;
use fret_ui_kit::primitives::direction as direction_prim;
use fret_ui_kit::primitives::focus_scope as focus_scope_prim;
use fret_ui_kit::primitives::hover_intent::HoverIntentConfig;
use fret_ui_kit::primitives::popover as radix_popover;
use fret_ui_kit::primitives::popper;
use fret_ui_kit::primitives::popper_content;
use fret_ui_kit::primitives::presence as radix_presence;
use fret_ui_kit::{ChromeRefinement, ColorRef, LayoutRefinement, OverlayPresence, Space, ui};

use crate::layout as shadcn_layout;
use crate::overlay_motion;
use crate::surface_slot::{ShadcnSurfaceSlot, with_surface_slot_provider};

#[derive(Debug, Clone, Copy)]
struct SizeHintPx {
    fixed_width: Option<Px>,
    fixed_height: Option<Px>,
    max_height: Option<Px>,
}

fn size_hint_px(element: &AnyElement) -> SizeHintPx {
    fn visit(node: &AnyElement, hint: &mut SizeHintPx) {
        let layout = match &node.kind {
            ElementKind::Container(ContainerProps { layout, .. }) => Some(layout),
            ElementKind::Scroll(fret_ui::element::ScrollProps { layout, .. }) => Some(layout),
            _ => None,
        };
        if let Some(layout) = layout {
            if let Length::Px(w) = layout.size.width {
                hint.fixed_width = Some(
                    hint.fixed_width
                        .map(|cur| if w.0 > cur.0 { w } else { cur })
                        .unwrap_or(w),
                );
            }
            if let Length::Px(h) = layout.size.height {
                hint.fixed_height = Some(
                    hint.fixed_height
                        .map(|cur| if h.0 > cur.0 { h } else { cur })
                        .unwrap_or(h),
                );
            }
            if let Some(max_h) = layout.size.max_height {
                hint.max_height = Some(
                    hint.max_height
                        .map(|cur| if max_h.0 > cur.0 { max_h } else { cur })
                        .unwrap_or(max_h),
                );
            }
        }

        for child in &node.children {
            visit(child, hint);
        }
    }

    let mut hint = SizeHintPx {
        fixed_width: None,
        fixed_height: None,
        max_height: None,
    };
    visit(element, &mut hint);
    hint
}

fn has_height_constraint_px(hint: SizeHintPx) -> bool {
    hint.fixed_height.is_some() || hint.max_height.is_some()
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum PopoverAlign {
    Start,
    #[default]
    Center,
    End,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum PopoverSide {
    Top,
    Right,
    #[default]
    Bottom,
    Left,
    InlineStart,
    InlineEnd,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum PopoverModalMode {
    #[default]
    NonModal,
    Modal,
    TrapFocus,
}

const POPOVER_OPEN_ON_HOVER_DEFAULT_OPEN_DELAY_FRAMES: u32 =
    overlay_motion::SHADCN_MOTION_TICKS_300 as u32;
const POPOVER_OPEN_ON_HOVER_DEFAULT_CLOSE_DELAY_FRAMES: u32 = 0;
const POPOVER_OPEN_ON_HOVER_SAFE_CORRIDOR_BUFFER: Px = Px(5.0);

type OnOpenChange = Arc<dyn Fn(bool) + Send + Sync + 'static>;

#[derive(Default)]
struct PopoverOpenChangeCallbackState {
    initialized: bool,
    last_open: bool,
    pending_complete: Option<bool>,
}

fn popover_open_change_events(
    state: &mut PopoverOpenChangeCallbackState,
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
struct PopoverHoverLastPointerModelState {
    model: Option<Model<Option<Point>>>,
}

fn popover_hover_last_pointer_model<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    popover_id: fret_ui::elements::GlobalElementId,
) -> Model<Option<Point>> {
    let existing = cx.with_state_for(
        popover_id,
        PopoverHoverLastPointerModelState::default,
        |st| st.model.clone(),
    );
    if let Some(model) = existing {
        model
    } else {
        let model = cx.app.models_mut().insert(None::<Point>);
        cx.with_state_for(
            popover_id,
            PopoverHoverLastPointerModelState::default,
            |st| {
                st.model = Some(model.clone());
            },
        );
        model
    }
}

/// shadcn/ui `Popover` (v4).
///
/// This is a non-modal, dismissible overlay built on:
/// - per-window overlay roots (ADR 0067)
/// - click-through outside-press observer pass (ADR 0069)
#[derive(Clone)]
pub struct Popover {
    open: Model<bool>,
    trigger_override: Option<fret_ui::elements::GlobalElementId>,
    align: PopoverAlign,
    side: PopoverSide,
    align_offset: Px,
    side_offset: Px,
    shift_cross_axis: Option<bool>,
    window_margin_override: Option<Px>,
    collision_padding_override: Option<Edges>,
    collision_boundary_override: Option<Rect>,
    sticky_override: Option<popper::StickyMode>,
    arrow: bool,
    arrow_size_override: Option<Px>,
    arrow_padding_override: Option<Px>,
    hide_when_detached: bool,
    keep_mounted: bool,
    consume_outside_pointer_events: bool,
    modal_mode: PopoverModalMode,
    open_on_hover: bool,
    hover_open_delay_frames: u32,
    hover_close_delay_frames: u32,
    auto_focus: Option<bool>,
    initial_focus: Option<fret_ui::elements::GlobalElementId>,
    initial_focus_from_cell: Option<Rc<Cell<Option<fret_ui::elements::GlobalElementId>>>>,
    anchor_override: Option<fret_ui::elements::GlobalElementId>,
    on_dismiss_request: Option<OnDismissRequest>,
    on_open_auto_focus: Option<OnOpenAutoFocus>,
    on_close_auto_focus: Option<OnCloseAutoFocus>,
    on_open_change: Option<OnOpenChange>,
    on_open_change_complete: Option<OnOpenChange>,
}

impl std::fmt::Debug for Popover {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Popover")
            .field("open", &"<model>")
            .field("trigger_override", &self.trigger_override)
            .field("align", &self.align)
            .field("side", &self.side)
            .field("align_offset", &self.align_offset)
            .field("side_offset", &self.side_offset)
            .field("shift_cross_axis", &self.shift_cross_axis)
            .field("window_margin_override", &self.window_margin_override)
            .field(
                "collision_padding_override",
                &self.collision_padding_override,
            )
            .field(
                "collision_boundary_override",
                &self.collision_boundary_override,
            )
            .field("sticky_override", &self.sticky_override)
            .field("keep_mounted", &self.keep_mounted)
            .field("modal_mode", &self.modal_mode)
            .field("open_on_hover", &self.open_on_hover)
            .field("hover_open_delay_frames", &self.hover_open_delay_frames)
            .field("hover_close_delay_frames", &self.hover_close_delay_frames)
            .field("auto_focus", &self.auto_focus)
            .field("initial_focus", &self.initial_focus)
            .field(
                "initial_focus_from_cell",
                &self.initial_focus_from_cell.is_some(),
            )
            .field("on_dismiss_request", &self.on_dismiss_request.is_some())
            .field("on_open_auto_focus", &self.on_open_auto_focus.is_some())
            .field("on_close_auto_focus", &self.on_close_auto_focus.is_some())
            .field("on_open_change", &self.on_open_change.is_some())
            .field(
                "on_open_change_complete",
                &self.on_open_change_complete.is_some(),
            )
            .finish()
    }
}

impl Popover {
    pub fn new(open: Model<bool>) -> Self {
        Self {
            open,
            trigger_override: None,
            align: PopoverAlign::default(),
            side: PopoverSide::default(),
            align_offset: Px(0.0),
            side_offset: Px(4.0),
            shift_cross_axis: None,
            window_margin_override: None,
            collision_padding_override: None,
            collision_boundary_override: None,
            sticky_override: None,
            arrow: false,
            arrow_size_override: None,
            arrow_padding_override: None,
            hide_when_detached: false,
            keep_mounted: false,
            consume_outside_pointer_events: false,
            modal_mode: PopoverModalMode::NonModal,
            open_on_hover: false,
            hover_open_delay_frames: POPOVER_OPEN_ON_HOVER_DEFAULT_OPEN_DELAY_FRAMES,
            hover_close_delay_frames: POPOVER_OPEN_ON_HOVER_DEFAULT_CLOSE_DELAY_FRAMES,
            auto_focus: None,
            initial_focus: None,
            initial_focus_from_cell: None,
            anchor_override: None,
            on_dismiss_request: None,
            on_open_auto_focus: None,
            on_close_auto_focus: None,
            on_open_change: None,
            on_open_change_complete: None,
        }
    }

    /// Creates a popover with a controlled/uncontrolled open model (Radix `open` / `defaultOpen`).
    ///
    /// Note: If `open` is `None`, the internal model is stored in element state at the call site.
    /// Call this from a stable subtree (key the parent node if needed).
    pub fn new_controllable<H: UiHost>(
        cx: &mut ElementContext<'_, H>,
        open: Option<Model<bool>>,
        default_open: bool,
    ) -> Self {
        let open = radix_popover::PopoverRoot::new()
            .open(open)
            .default_open(default_open)
            .open_model(cx);
        Self::new(open)
    }

    pub fn align(mut self, align: PopoverAlign) -> Self {
        self.align = align;
        self
    }

    pub fn side(mut self, side: PopoverSide) -> Self {
        self.side = side;
        self
    }

    pub fn align_offset(mut self, offset: Px) -> Self {
        self.align_offset = offset;
        self
    }

    pub fn side_offset(mut self, offset: Px) -> Self {
        self.side_offset = offset;
        self
    }

    /// Controls whether the placement solver can shift the floating panel on the cross axis.
    ///
    /// Radix/Base UI default for Popover is effectively `false`.
    pub fn shift_cross_axis(mut self, shift_cross_axis: bool) -> Self {
        self.shift_cross_axis = Some(shift_cross_axis);
        self
    }

    pub fn window_margin(mut self, margin: Px) -> Self {
        self.window_margin_override = Some(margin);
        self
    }

    pub fn collision_padding(mut self, padding: Edges) -> Self {
        self.collision_padding_override = Some(padding);
        self
    }

    pub fn collision_boundary(mut self, boundary: Option<Rect>) -> Self {
        self.collision_boundary_override = boundary;
        self
    }

    pub fn sticky(mut self, sticky: popper::StickyMode) -> Self {
        self.sticky_override = Some(sticky);
        self
    }

    /// Enables a Popover arrow (Radix `PopoverArrow`-style).
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

    /// When `true`, the popover content becomes hidden and non-interactive if the anchor is fully
    /// clipped by the collision boundary (Radix `hideWhenDetached`).
    ///
    /// Default: `false`.
    pub fn hide_when_detached(mut self, hide: bool) -> Self {
        self.hide_when_detached = hide;
        self
    }

    /// Keeps the overlay subtree mounted while closed (Radix `forceMount`-style outcome).
    pub fn keep_mounted(mut self, keep_mounted: bool) -> Self {
        self.keep_mounted = keep_mounted;
        self
    }

    /// Alias for [`Popover::keep_mounted`], using Radix naming.
    pub fn force_mount(mut self, force_mount: bool) -> Self {
        self.keep_mounted = force_mount;
        self
    }

    /// When enabled, suppress hit-tested pointer-down dispatch to underlay widgets when this
    /// popover receives an outside-press observer event (ADR 0069).
    ///
    /// Default: `false` (click-through).
    pub fn consume_outside_pointer_events(mut self, consume: bool) -> Self {
        self.consume_outside_pointer_events = consume;
        self
    }

    /// Enables a Radix-style "modal popover" variant.
    ///
    /// This installs the popover content in the shared modal overlay layer, blocking interaction
    /// with the underlay.
    ///
    /// Default: `false` (non-modal popover).
    pub fn modal(mut self, modal: bool) -> Self {
        self.modal_mode = if modal {
            PopoverModalMode::Modal
        } else {
            PopoverModalMode::NonModal
        };
        self
    }

    /// Sets Base UI-compatible modal mode in one call (`modal` prop parity).
    ///
    /// - `PopoverModalMode::NonModal`: outside pointer interaction stays enabled.
    /// - `PopoverModalMode::Modal`: installs modal barrier + outside pointer disable.
    /// - `PopoverModalMode::TrapFocus`: traps focus only; outside pointer remains enabled.
    pub fn modal_mode(mut self, mode: PopoverModalMode) -> Self {
        self.modal_mode = mode;
        self
    }

    /// Base UI-style trap-focus mode: trap keyboard focus inside content while leaving outside
    /// pointer interactions enabled.
    ///
    /// This differs from `modal(true)` which installs a modal barrier.
    pub fn modal_trap_focus(mut self, trap: bool) -> Self {
        self.modal_mode = if trap {
            PopoverModalMode::TrapFocus
        } else {
            PopoverModalMode::NonModal
        };
        self
    }

    /// Associates this popover with an external trigger element (Base UI detached trigger-like).
    ///
    /// When set, this element acts as the dismissal/focus-restore trigger and source for trigger
    /// a11y state.
    pub fn trigger_element(mut self, element: fret_ui::elements::GlobalElementId) -> Self {
        self.trigger_override = Some(element);
        self
    }

    /// Enables optional hover-open behavior (Base UI `openOnHover`-style).
    ///
    /// Default: `false`.
    pub fn open_on_hover(mut self, open_on_hover: bool) -> Self {
        self.open_on_hover = open_on_hover;
        self
    }

    /// Configures hover-open delay in frames for `open_on_hover(true)`.
    pub fn hover_open_delay_frames(mut self, frames: u32) -> Self {
        self.hover_open_delay_frames = frames;
        self
    }

    /// Configures hover-close delay in frames for `open_on_hover(true)`.
    pub fn hover_close_delay_frames(mut self, frames: u32) -> Self {
        self.hover_close_delay_frames = frames;
        self
    }

    /// Controls whether the popover should auto-focus into content on open.
    ///
    /// By default this follows the trigger contract:
    /// - `PopoverTrigger`: auto-focus enabled (Radix/Base UI-like)
    /// - custom/manual trigger wiring: preserve previous behavior
    pub fn auto_focus(mut self, auto_focus: bool) -> Self {
        self.auto_focus = Some(auto_focus);
        self
    }

    pub fn initial_focus(mut self, element: fret_ui::elements::GlobalElementId) -> Self {
        self.initial_focus = Some(element);
        self
    }

    /// Uses an initial focus target that is only known while building the content subtree.
    ///
    /// This is useful for popovers that want to focus a specific descendant (e.g. a selected day
    /// in a date picker) without hard-coding element IDs.
    pub(crate) fn initial_focus_from_cell(
        mut self,
        cell: Rc<Cell<Option<fret_ui::elements::GlobalElementId>>>,
    ) -> Self {
        self.initial_focus_from_cell = Some(cell);
        self
    }

    /// Override the element used as the placement anchor.
    ///
    /// Notes:
    /// - Dismissal and focus-restore policies still treat the trigger as the "interactive branch".
    /// - The anchor bounds are resolved from `ElementCx::last_bounds_for_element` / visual bounds,
    ///   so it may take one frame to stabilize after layout changes (same as trigger anchoring).
    pub fn anchor_element(mut self, element: fret_ui::elements::GlobalElementId) -> Self {
        self.anchor_override = Some(element);
        self
    }

    /// Sets an optional dismiss request handler (Radix `DismissableLayer`).
    ///
    /// When set, Escape/outside-press dismissals route through this handler. To prevent default
    /// dismissal, call `req.prevent_default()`.
    pub fn on_dismiss_request(mut self, on_dismiss_request: Option<OnDismissRequest>) -> Self {
        self.on_dismiss_request = on_dismiss_request;
        self
    }

    /// Installs an open auto-focus hook (Radix `FocusScope` `onMountAutoFocus`).
    pub fn on_open_auto_focus(mut self, hook: Option<OnOpenAutoFocus>) -> Self {
        self.on_open_auto_focus = hook;
        self
    }

    /// Installs a close auto-focus hook (Radix `FocusScope` `onUnmountAutoFocus`).
    pub fn on_close_auto_focus(mut self, hook: Option<OnCloseAutoFocus>) -> Self {
        self.on_close_auto_focus = hook;
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

    pub fn into_element<H: UiHost>(
        self,
        cx: &mut ElementContext<'_, H>,
        trigger: impl FnOnce(&mut ElementContext<'_, H>) -> AnyElement,
        content: impl FnOnce(&mut ElementContext<'_, H>) -> AnyElement,
    ) -> AnyElement {
        self.into_element_with_anchor(cx, trigger, move |cx, _anchor| content(cx))
    }

    pub fn into_element_with_anchor<H: UiHost>(
        self,
        cx: &mut ElementContext<'_, H>,
        trigger: impl FnOnce(&mut ElementContext<'_, H>) -> AnyElement,
        content: impl FnOnce(&mut ElementContext<'_, H>, fret_core::Rect) -> AnyElement,
    ) -> AnyElement {
        cx.scope(|cx| {
            let theme = Theme::global(&*cx.app).clone();
            let popover_id = cx.root_id();

            let trigger = trigger(cx);
            let trigger_id = self.trigger_override.unwrap_or(trigger.id);

            let trigger_is_shadcn_trigger =
                cx.with_state_for(trigger_id, PopoverTriggerContract::default, |st| {
                    st.auto_toggle
                });

            if trigger_is_shadcn_trigger {
                cx.pressable_add_on_activate_for(
                    trigger_id,
                    Arc::new({
                        let open = self.open.clone();
                        move |host, _acx, _reason| {
                            let _ = host.models_mut().update(&open, |v| *v = !*v);
                        }
                    }),
                );
            }

            let open_on_hover = self.open_on_hover;
            let hover_last_pointer =
                open_on_hover.then(|| popover_hover_last_pointer_model(cx, popover_id));
            let hover_open_delay_frames = self.hover_open_delay_frames;
            let hover_close_delay_frames = self.hover_close_delay_frames;

            let trigger = if open_on_hover {
                let open = self.open.clone();
                let hover_last_pointer = hover_last_pointer
                    .clone()
                    .expect("hover pointer model should exist when open_on_hover=true");
                let cfg = HoverIntentConfig::new(
                    hover_open_delay_frames as u64,
                    hover_close_delay_frames as u64,
                );
                cx.hover_region(HoverRegionProps::default(), move |cx, trigger_hovered| {
                    let (overlay_hovered, anchor_bounds, floating_bounds) =
                        cx.with_state_for(popover_id, PopoverHoverSharedState::default, |st| {
                            (st.overlay_hovered, st.anchor_bounds, st.floating_bounds)
                        });
                    let pointer_in_corridor = cx
                        .watch_model(&hover_last_pointer)
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
                                POPOVER_OPEN_ON_HOVER_SAFE_CORRIDOR_BUFFER,
                            )
                        });

                    let hovered = trigger_hovered || overlay_hovered || pointer_in_corridor;

                    let open_now = cx.watch_model(&open).layout().copied().unwrap_or(false);
                    let frame_tick = cx.app.frame_id().0;
                    let update = cx.with_state_for(
                        popover_id,
                        PopoverHoverIntentDriverState::default,
                        |st| {
                            match st.last_frame_tick {
                                None => {
                                    st.last_frame_tick = Some(frame_tick);
                                    st.tick = frame_tick;
                                }
                                Some(prev) if prev != frame_tick => {
                                    st.last_frame_tick = Some(frame_tick);
                                    st.tick = frame_tick;
                                }
                                Some(_) => {
                                    st.tick = st.tick.saturating_add(1);
                                }
                            }

                            if st.intent.is_open() != open_now {
                                st.intent.set_open(open_now);
                            }

                            st.intent.update(hovered, st.tick, cfg)
                        },
                    );

                    scheduling::set_continuous_frames(cx, update.wants_continuous_ticks);
                    if update.open != open_now {
                        let _ = cx.app.models_mut().update(&open, |v| *v = update.open);
                    }

                    vec![trigger]
                })
            } else {
                trigger
            };

            let is_open = cx
                .watch_model(&self.open)
                .layout()
                .copied()
                .unwrap_or(false);

            let auto_focus = self
                .auto_focus
                .unwrap_or(trigger_is_shadcn_trigger && !open_on_hover);

            let anchor_id = self.anchor_override.unwrap_or(trigger_id);
            let modal = matches!(self.modal_mode, PopoverModalMode::Modal);
            let trap_focus_only = matches!(self.modal_mode, PopoverModalMode::TrapFocus);
            let overlay_root_name = if modal {
                radix_popover::popover_modal_root_name(trigger_id)
            } else {
                radix_popover::popover_root_name(trigger_id)
            };

            let motion = radix_presence::scale_fade_presence_with_durations_and_easing(
                cx,
                is_open,
                overlay_motion::SHADCN_MOTION_TICKS_100,
                overlay_motion::SHADCN_MOTION_TICKS_100,
                0.95,
                1.0,
                overlay_motion::shadcn_ease,
            );
            let (open_change, open_change_complete) =
                cx.with_state(PopoverOpenChangeCallbackState::default, |state| {
                    popover_open_change_events(state, is_open, motion.present, motion.animating)
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
            let overlay_presence = OverlayPresence {
                present: self.keep_mounted || motion.present,
                interactive: is_open,
            };
            let dialog_id_for_trigger: Rc<Cell<Option<fret_ui::elements::GlobalElementId>>> =
                Rc::new(Cell::new(None));

            if open_on_hover && !overlay_presence.present {
                cx.with_state_for(popover_id, PopoverHoverSharedState::default, |st| {
                    st.overlay_hovered = false;
                    st.anchor_bounds = None;
                    st.floating_bounds = None;
                });
            }

            if overlay_presence.present {
                let on_dismiss_request_for_children = self.on_dismiss_request.clone();
                let on_dismiss_request_for_request = self.on_dismiss_request.clone();

                let align = self.align;
                let side = self.side;
                let align_offset = self.align_offset;
                let side_offset = self.side_offset;
                let shift_cross_axis = self.shift_cross_axis.unwrap_or(true);
                let window_margin = self.window_margin_override.unwrap_or_else(|| {
                    theme
                        .metric_by_key("component.popover.window_margin")
                        .unwrap_or(Px(0.0))
                });
                let collision_padding = self
                    .collision_padding_override
                    .unwrap_or(Edges::all(Px(0.0)));
                let collision_boundary = self.collision_boundary_override;
                let sticky = self.sticky_override.unwrap_or(popper::StickyMode::Partial);
                let arrow = self.arrow;
                let arrow_size = self.arrow_size_override.unwrap_or_else(|| {
                    theme
                        .metric_by_key("component.popover.arrow_size")
                        .unwrap_or(Px(12.0))
                });
                let arrow_padding = self.arrow_padding_override.unwrap_or_else(|| {
                    theme
                        .metric_by_key("component.popover.arrow_padding")
                        .unwrap_or_else(|| theme.metric_required("metric.radius.md"))
                });

                let opacity = motion.opacity;
                let scale = motion.scale;
                let opening = is_open;
                let dialog_id_for_trigger = dialog_id_for_trigger.clone();
                let open_for_barrier = self.open.clone();
                let hide_when_detached = self.hide_when_detached;
                let direction = direction_prim::use_direction_in_scope(cx, None);
                let overlay_children = cx.with_root_name(&overlay_root_name, move |cx| {
                    let anchor_fallback = overlay::anchor_bounds_for_element(cx, anchor_id);
                    if anchor_fallback.is_none() {
                        if open_on_hover {
                            cx.with_state_for(popover_id, PopoverHoverSharedState::default, |st| {
                                st.overlay_hovered = false;
                                st.anchor_bounds = None;
                                st.floating_bounds = None;
                            });
                        }
                        if modal {
                            return [radix_popover::popover_modal_barrier_with_dismiss_handler(
                                cx,
                                open_for_barrier.clone(),
                                true,
                                on_dismiss_request_for_children.clone(),
                                Vec::new(),
                            )]
                            .into_iter()
                            .collect::<fret_ui::element::Elements>();
                        }
                        return std::iter::empty().collect::<fret_ui::element::Elements>();
                    }

                    let inner_id: Rc<Cell<Option<fret_ui::elements::GlobalElementId>>> =
                        Rc::new(Cell::new(None));
                    let inner_id_for_scope = inner_id.clone();
                    let inner_size_hint: Rc<Cell<Option<SizeHintPx>>> = Rc::new(Cell::new(None));
                    let inner_size_hint_for_scope = inner_size_hint.clone();
                    let content = radix_popover::popover_dialog_wrapper(cx, None, move |cx| {
                        let inner = with_surface_slot_provider(
                            cx,
                            ShadcnSurfaceSlot::PopoverContent,
                            |cx| content(cx, anchor_fallback.unwrap_or_default()),
                        );
                        inner_id_for_scope.set(Some(inner.id));
                        inner_size_hint_for_scope.set(Some(size_hint_px(&inner)));
                        vec![inner]
                    });
                    dialog_id_for_trigger.set(Some(content.id));

                    let measure_id = inner_id.get().unwrap_or(content.id);
                    let last_content_size = cx.last_bounds_for_element(measure_id).map(|r| r.size);
                    let estimated = Size::new(Px(288.0), Px(160.0));
                    let hint = inner_size_hint.get().unwrap_or(SizeHintPx {
                        fixed_width: None,
                        fixed_height: None,
                        max_height: None,
                    });
                    let hint_width = hint.fixed_width;
                    let hint_height = hint.fixed_height.or(hint.max_height);
                    let content_size = Size::new(
                        hint_width
                            .or_else(|| last_content_size.map(|s| s.width))
                            .unwrap_or(estimated.width),
                        hint_height
                            .or_else(|| last_content_size.map(|s| s.height))
                            .unwrap_or(estimated.height),
                    );

                    let align = match align {
                        PopoverAlign::Start => Align::Start,
                        PopoverAlign::Center => Align::Center,
                        PopoverAlign::End => Align::End,
                    };
                    let side = match side {
                        PopoverSide::Top => Side::Top,
                        PopoverSide::Right => Side::Right,
                        PopoverSide::Bottom => Side::Bottom,
                        PopoverSide::Left => Side::Left,
                        PopoverSide::InlineStart => {
                            if direction == direction_prim::LayoutDirection::Rtl {
                                Side::Right
                            } else {
                                Side::Left
                            }
                        }
                        PopoverSide::InlineEnd => {
                            if direction == direction_prim::LayoutDirection::Rtl {
                                Side::Left
                            } else {
                                Side::Right
                            }
                        }
                    };

                    let (arrow_options, arrow_protrusion) =
                        popper::diamond_arrow_options(arrow, arrow_size, arrow_padding);

                    let outer = overlay::outer_bounds_with_window_margin(cx.bounds, window_margin);
                    let placement =
                        popper::PopperContentPlacement::new(direction, side, align, side_offset)
                            .with_shift_cross_axis(shift_cross_axis)
                            .with_align_offset(align_offset)
                            .with_arrow(arrow_options, arrow_protrusion)
                            .with_collision_padding(collision_padding)
                            .with_collision_boundary(collision_boundary)
                            .with_sticky(sticky)
                            .with_hide_when_detached(hide_when_detached);
                    let reference_hidden = anchor_fallback
                        .is_some_and(|anchor| placement.reference_hidden(outer, anchor));

                    let bg = theme.color_required("popover.background");
                    let border = theme.color_required("border");

                    let anchor = anchor_fallback.unwrap_or_default();
                    let constrained_height = has_height_constraint_px(hint);
                    let layout = if constrained_height {
                        popper::popper_content_layout_sized(outer, anchor, content_size, placement)
                    } else {
                        popper::popper_content_layout_unclamped(
                            outer,
                            anchor,
                            content_size,
                            placement,
                        )
                    };
                    if open_on_hover {
                        cx.with_state_for(popover_id, PopoverHoverSharedState::default, |st| {
                            st.anchor_bounds = Some(anchor);
                            st.floating_bounds = Some(layout.rect);
                        });
                    }
                    let wrapper_insets = popper_arrow::wrapper_insets(&layout, arrow_protrusion);

                    let allow_intrinsic_wrapper = !arrow && !constrained_height;
                    let panel_or_content = if allow_intrinsic_wrapper {
                        content
                    } else {
                        popper_content::popper_panel_at(
                            cx,
                            layout.rect,
                            wrapper_insets,
                            Overflow::Visible,
                            move |_cx| vec![content],
                        )
                    };

                    let arrow_el = popper_arrow::diamond_arrow_element(
                        cx,
                        &layout,
                        wrapper_insets,
                        arrow_size,
                        DiamondArrowStyle {
                            bg,
                            border: Some(border),
                            border_width: Px(1.0),
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

                    // We want the layer root bounds to match the steady-state popper wrapper
                    // geometry (including arrow protrusion), but we don't want animation transforms
                    // to affect hit-testing.
                    //
                    // We model this by:
                    // - positioning the layer root via `InteractivityGate` (absolute layout),
                    // - applying opacity + `VisualTransform` on an inner fill node (paint-only).
                    let wrapper_layout = if allow_intrinsic_wrapper {
                        popper_content::popper_wrapper_layout_autosize(layout.rect.origin)
                    } else {
                        popper_content::popper_wrapper_layout(layout.rect, wrapper_insets)
                    };
                    let mut fill = LayoutStyle::default();
                    if !allow_intrinsic_wrapper {
                        fill.size.width = Length::Fill;
                        fill.size.height = Length::Fill;
                    }

                    let overlay_children = if let Some(arrow_el) = arrow_el {
                        vec![arrow_el, panel_or_content]
                    } else {
                        vec![panel_or_content]
                    };

                    let overlay_content = cx.interactivity_gate_props(
                        InteractivityGateProps {
                            layout: wrapper_layout,
                            present: true,
                            interactive: !reference_hidden,
                        },
                        move |cx| {
                            // `InteractivityGate` itself is pointer-transparent; we add a
                            // hit-testable wrapper container so the arrow protrusion region counts
                            // as "inside" the overlay for outside-press semantics.
                            vec![cx.container(
                                ContainerProps {
                                    layout: fill,
                                    ..Default::default()
                                },
                                move |cx| {
                                    vec![cx.opacity_props(
                                        OpacityProps {
                                            layout: fill,
                                            opacity,
                                        },
                                        move |cx| {
                                            vec![cx.visual_transform_props(
                                                VisualTransformProps {
                                                    layout: fill,
                                                    transform,
                                                },
                                                move |_cx| overlay_children,
                                            )]
                                        },
                                    )]
                                },
                            )]
                        },
                    );

                    let overlay_content = if open_on_hover {
                        cx.hover_region(HoverRegionProps::default(), move |cx, hovered| {
                            cx.with_state_for(popover_id, PopoverHoverSharedState::default, |st| {
                                st.overlay_hovered = hovered;
                            });
                            vec![overlay_content]
                        })
                    } else {
                        overlay_content
                    };

                    let overlay_content = if trap_focus_only {
                        focus_scope_prim::focus_trap(cx, move |_cx| vec![overlay_content])
                    } else {
                        overlay_content
                    };

                    if open_on_hover {
                        cx.with_state_for(popover_id, PopoverHoverSharedState::default, |st| {
                            st.overlay_hovered = false;
                        });
                    }

                    if modal {
                        radix_popover::popover_modal_layer_elements_with_dismiss_handler(
                            cx,
                            open_for_barrier.clone(),
                            on_dismiss_request_for_children.clone(),
                            [],
                            overlay_content,
                        )
                    } else {
                        [overlay_content]
                            .into_iter()
                            .collect::<fret_ui::element::Elements>()
                    }
                });

                let initial_focus = if let Some(id) = self.initial_focus {
                    Some(id)
                } else if let Some(cell) = self.initial_focus_from_cell.as_ref()
                    && let Some(id) = cell.get()
                {
                    Some(id)
                } else if auto_focus {
                    None
                } else {
                    Some(trigger_id)
                };

                let mut options = radix_popover::PopoverOptions::default()
                    .modal(modal)
                    .consume_outside_pointer_events(self.consume_outside_pointer_events)
                    .on_open_auto_focus(self.on_open_auto_focus.clone())
                    .on_close_auto_focus(self.on_close_auto_focus.clone());
                if let Some(initial_focus) = initial_focus {
                    options = options.initial_focus(initial_focus);
                }

                let mut request = radix_popover::popover_request_with_anchor_and_dismiss_handler(
                    trigger_id,
                    trigger_id,
                    Some(anchor_id),
                    self.open,
                    overlay_presence,
                    options,
                    on_dismiss_request_for_request,
                    overlay_children,
                );
                if open_on_hover {
                    let hover_last_pointer = hover_last_pointer
                        .clone()
                        .expect("hover pointer model should exist when open_on_hover=true");
                    request.dismissible_on_pointer_move = Some(Arc::new(move |host, _acx, mv| {
                        if mv.pointer_type == fret_core::PointerType::Touch {
                            return false;
                        }
                        let _ = host
                            .models_mut()
                            .update(&hover_last_pointer, |v| *v = Some(mv.position));
                        false
                    }));
                }
                radix_popover::request_popover(cx, request);
            }

            let dialog_id_for_trigger = dialog_id_for_trigger.get();
            radix_popover::apply_popover_trigger_a11y(trigger, is_open, dialog_id_for_trigger)
        })
    }
}

#[derive(Debug, Clone, Copy, Default)]
struct PopoverTriggerContract {
    auto_toggle: bool,
}

#[derive(Debug, Default, Clone, Copy)]
struct PopoverHoverIntentDriverState {
    last_frame_tick: Option<u64>,
    tick: u64,
    intent: fret_ui_kit::primitives::hover_intent::HoverIntentState,
}

#[derive(Debug, Default, Clone)]
struct PopoverHoverSharedState {
    overlay_hovered: bool,
    anchor_bounds: Option<Rect>,
    floating_bounds: Option<Rect>,
}

/// shadcn/ui `PopoverTrigger` (v4).
#[derive(Debug, Clone)]
pub struct PopoverTrigger {
    child: AnyElement,
    auto_toggle: bool,
}

impl PopoverTrigger {
    pub fn new(child: AnyElement) -> Self {
        Self {
            child,
            auto_toggle: true,
        }
    }

    /// Controls whether this trigger should toggle the associated `Popover` open model
    /// automatically.
    ///
    /// Default: `true` (Radix/shadcn Trigger-like behavior).
    pub fn auto_toggle(mut self, auto_toggle: bool) -> Self {
        self.auto_toggle = auto_toggle;
        self
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let child = self.child;
        let auto_toggle = self.auto_toggle;
        cx.with_state_for(child.id, PopoverTriggerContract::default, |st| {
            st.auto_toggle = auto_toggle;
        });
        child
    }
}

/// shadcn/ui `PopoverAnchor` (v4).
///
/// This is a layout-only helper. Use [`Popover::anchor_element`] to wire the anchor element ID
/// into placement.
#[derive(Debug, Clone)]
pub struct PopoverAnchor {
    child: AnyElement,
}

impl PopoverAnchor {
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

fn popover_content_chrome() -> ChromeRefinement {
    crate::ui_builder_ext::surfaces::popover_style_chrome()
}

/// shadcn/ui `PopoverContent` (v4).
#[derive(Debug, Clone)]
pub struct PopoverContent {
    children: Vec<AnyElement>,
    chrome: ChromeRefinement,
    layout: LayoutRefinement,
    a11y_label: Option<Arc<str>>,
}

impl PopoverContent {
    pub fn new(children: impl IntoIterator<Item = AnyElement>) -> Self {
        let children = children.into_iter().collect();
        Self {
            children,
            chrome: ChromeRefinement::default(),
            layout: LayoutRefinement::default()
                .w_px(Px(288.0))
                .min_w_0()
                .min_h_0(),
            a11y_label: None,
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

    pub fn a11y_label(mut self, label: impl Into<Arc<str>>) -> Self {
        self.a11y_label = Some(label.into());
        self
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let theme = Theme::global(&*cx.app);
        let chrome = popover_content_chrome().merge(self.chrome);
        let props = decl_style::container_props(theme, chrome, self.layout);
        let children = self.children;
        let label = self.a11y_label;

        let container = shadcn_layout::container_vstack_gap(cx, props, Space::N4, children);

        container.attach_semantics(SemanticsDecoration {
            role: Some(SemanticsRole::Panel),
            label,
            ..Default::default()
        })
    }
}

/// shadcn/ui `PopoverHeader` (v4).
#[derive(Debug, Clone)]
pub struct PopoverHeader {
    children: Vec<AnyElement>,
}

impl PopoverHeader {
    pub fn new(children: impl IntoIterator<Item = AnyElement>) -> Self {
        let children = children.into_iter().collect();
        Self { children }
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let props = decl_style::container_props(
            Theme::global(&*cx.app),
            ChromeRefinement::default(),
            LayoutRefinement::default().w_full().min_w_0(),
        );
        let children = self.children;
        shadcn_layout::container_vstack_gap(cx, props, Space::N1, children)
    }
}

/// shadcn/ui `PopoverTitle` (v4).
#[derive(Debug, Clone)]
pub struct PopoverTitle {
    text: Arc<str>,
}

impl PopoverTitle {
    pub fn new(text: impl Into<Arc<str>>) -> Self {
        Self { text: text.into() }
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let theme = Theme::global(&*cx.app).clone();
        let fg = theme.color_required("popover.foreground");

        let px = theme
            .metric_by_key("component.popover.title_px")
            .or_else(|| theme.metric_by_key("font.size"))
            .unwrap_or_else(|| theme.metric_required("font.size"));
        let line_height = theme
            .metric_by_key("component.popover.title_line_height")
            .or_else(|| theme.metric_by_key("font.line_height"))
            .unwrap_or_else(|| theme.metric_required("font.line_height"));

        ui::text(cx, self.text)
            .text_size_px(px)
            .line_height_px(line_height)
            .font_medium()
            .text_color(ColorRef::Color(fg))
            .nowrap()
            .into_element(cx)
    }
}

/// shadcn/ui `PopoverDescription` (v4).
#[derive(Debug, Clone)]
pub struct PopoverDescription {
    text: Arc<str>,
}

impl PopoverDescription {
    pub fn new(text: impl Into<Arc<str>>) -> Self {
        Self { text: text.into() }
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let theme = Theme::global(&*cx.app).clone();
        let fg = theme
            .color_by_key("muted.foreground")
            .or_else(|| theme.color_by_key("muted-foreground"))
            .unwrap_or_else(|| theme.color_required("muted.foreground"));

        let px = theme
            .metric_by_key("component.popover.description_px")
            .or_else(|| theme.metric_by_key("font.size"))
            .unwrap_or_else(|| theme.metric_required("font.size"));
        let line_height = theme
            .metric_by_key("component.popover.description_line_height")
            .or_else(|| theme.metric_by_key("font.line_height"))
            .unwrap_or_else(|| theme.metric_required("font.line_height"));

        ui::text(cx, self.text)
            .text_size_px(px)
            .line_height_px(line_height)
            .font_normal()
            .text_color(ColorRef::Color(fg))
            .into_element(cx)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::Cell;
    use std::rc::Rc;
    use std::sync::Mutex;
    use std::sync::atomic::{AtomicUsize, Ordering};

    use crate::test_support::render_overlay_frame;
    use fret_app::App;
    use fret_core::{
        AppWindowId, Corners, MouseButton, PathCommand, Point, Rect, Size as CoreSize, SvgId,
        SvgService,
    };
    use fret_core::{KeyCode, Modifiers};
    use fret_core::{PathConstraints, PathId, PathMetrics, PathService, PathStyle};
    use fret_core::{Px, TextBlobId, TextConstraints, TextMetrics, TextService};
    use fret_runtime::Effect;
    use fret_runtime::FrameId;
    use fret_ui::UiTree;
    use fret_ui::element::{LayoutStyle, Length, PressableProps};
    use fret_ui_kit::OverlayController;
    use fret_ui_kit::declarative::action_hooks::ActionHooksExt;

    #[test]
    fn popover_new_controllable_uses_controlled_model_when_provided() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            CoreSize::new(Px(200.0), Px(120.0)),
        );

        let controlled = app.models_mut().insert(true);

        fret_ui::elements::with_element_cx(&mut app, window, bounds, "test", |cx| {
            let popover = Popover::new_controllable(cx, Some(controlled.clone()), false);
            assert_eq!(popover.open, controlled);
        });
    }

    #[test]
    fn popover_new_controllable_applies_default_open() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            CoreSize::new(Px(200.0), Px(120.0)),
        );

        fret_ui::elements::with_element_cx(&mut app, window, bounds, "test", |cx| {
            let popover = Popover::new_controllable(cx, None, true);
            let open = cx
                .watch_model(&popover.open)
                .layout()
                .copied()
                .unwrap_or(false);
            assert!(open);
        });
    }

    #[test]
    fn popover_open_change_events_emit_change_and_complete_after_settle() {
        let mut state = PopoverOpenChangeCallbackState::default();

        let (changed, completed) = popover_open_change_events(&mut state, false, false, false);
        assert_eq!(changed, None);
        assert_eq!(completed, None);

        let (changed, completed) = popover_open_change_events(&mut state, true, true, true);
        assert_eq!(changed, Some(true));
        assert_eq!(completed, None);

        let (changed, completed) = popover_open_change_events(&mut state, true, true, false);
        assert_eq!(changed, None);
        assert_eq!(completed, Some(true));
    }

    #[test]
    fn popover_open_change_events_complete_without_animation() {
        let mut state = PopoverOpenChangeCallbackState::default();

        let _ = popover_open_change_events(&mut state, false, false, false);
        let (changed, completed) = popover_open_change_events(&mut state, true, true, false);

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
                    size: CoreSize::new(Px(0.0), Px(0.0)),
                    baseline: Px(0.0),
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

    fn render_popover_frame(
        ui: &mut UiTree<App>,
        app: &mut App,
        services: &mut dyn fret_core::UiServices,
        window: AppWindowId,
        bounds: Rect,
        open: Model<bool>,
        arrow: bool,
        underlay_id_out: Rc<Cell<Option<fret_ui::elements::GlobalElementId>>>,
        popover_focus_id_out: Rc<Cell<Option<fret_ui::elements::GlobalElementId>>>,
        popover_content_id_out: Rc<Cell<Option<fret_ui::elements::GlobalElementId>>>,
    ) -> fret_ui::elements::GlobalElementId {
        OverlayController::begin_frame(app, window);

        let mut trigger_id: Option<fret_ui::elements::GlobalElementId> = None;

        let root =
            fret_ui::declarative::render_root(ui, app, services, window, bounds, "test", |cx| {
                let underlay = cx.pressable_with_id(
                    PressableProps {
                        layout: {
                            let mut layout = LayoutStyle::default();
                            layout.size.width = Length::Px(Px(120.0));
                            layout.size.height = Length::Px(Px(40.0));
                            layout.inset.top = Some(Px(300.0));
                            layout.inset.left = Some(Px(400.0));
                            layout.position = fret_ui::element::PositionStyle::Absolute;
                            layout
                        },
                        enabled: true,
                        focusable: true,
                        ..Default::default()
                    },
                    |cx, _st, id| {
                        underlay_id_out.set(Some(id));
                        vec![cx.container(ContainerProps::default(), |_cx| Vec::new())]
                    },
                );

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
                        cx.pressable_toggle_bool(&open);
                        trigger_id = Some(id);
                        vec![cx.container(ContainerProps::default(), |_cx| Vec::new())]
                    },
                );

                let popover_focus_id_out = popover_focus_id_out.clone();
                let popover_content_id_out = popover_content_id_out.clone();
                let popover = Popover::new(open.clone())
                    .auto_focus(true)
                    .arrow(arrow)
                    .into_element(
                        cx,
                        |_cx| trigger,
                        move |cx| {
                            let focusable = cx.pressable_with_id(
                                PressableProps {
                                    layout: {
                                        let mut layout = LayoutStyle::default();
                                        layout.size.width = Length::Px(Px(160.0));
                                        layout.size.height = Length::Px(Px(44.0));
                                        layout
                                    },
                                    enabled: true,
                                    focusable: true,
                                    ..Default::default()
                                },
                                |cx, _st, id| {
                                    popover_focus_id_out.set(Some(id));
                                    vec![cx.container(ContainerProps::default(), |_cx| Vec::new())]
                                },
                            );
                            let content = PopoverContent::new(vec![focusable]).into_element(cx);
                            popover_content_id_out.set(Some(content.id));
                            content
                        },
                    );

                vec![underlay, popover]
            });

        ui.set_root(root);
        OverlayController::render(ui, app, services, window, bounds);
        trigger_id.expect("trigger id")
    }

    fn render_popover_frame_with_auto_focus_hooks(
        ui: &mut UiTree<App>,
        app: &mut App,
        services: &mut dyn fret_core::UiServices,
        window: AppWindowId,
        bounds: Rect,
        open: Model<bool>,
        arrow: bool,
        underlay_id_out: Rc<Cell<Option<fret_ui::elements::GlobalElementId>>>,
        underlay_id_cell: Option<Arc<Mutex<Option<fret_ui::elements::GlobalElementId>>>>,
        popover_focus_id_out: Rc<Cell<Option<fret_ui::elements::GlobalElementId>>>,
        popover_content_id_out: Rc<Cell<Option<fret_ui::elements::GlobalElementId>>>,
        on_open_auto_focus: Option<OnOpenAutoFocus>,
        on_close_auto_focus: Option<OnCloseAutoFocus>,
    ) -> fret_ui::elements::GlobalElementId {
        OverlayController::begin_frame(app, window);

        let mut trigger_id: Option<fret_ui::elements::GlobalElementId> = None;

        let root =
            fret_ui::declarative::render_root(ui, app, services, window, bounds, "test", |cx| {
                let underlay_id_cell = underlay_id_cell.clone();
                let underlay_id_out = underlay_id_out.clone();
                let underlay = cx.pressable_with_id(
                    PressableProps {
                        layout: {
                            let mut layout = LayoutStyle::default();
                            layout.size.width = Length::Px(Px(120.0));
                            layout.size.height = Length::Px(Px(40.0));
                            layout.inset.top = Some(Px(300.0));
                            layout.inset.left = Some(Px(400.0));
                            layout.position = fret_ui::element::PositionStyle::Absolute;
                            layout
                        },
                        enabled: true,
                        focusable: true,
                        ..Default::default()
                    },
                    move |cx, _st, id| {
                        underlay_id_out.set(Some(id));
                        if let Some(underlay_id_cell) = underlay_id_cell.as_ref() {
                            *underlay_id_cell.lock().unwrap_or_else(|e| e.into_inner()) = Some(id);
                        }
                        vec![cx.container(ContainerProps::default(), |_cx| Vec::new())]
                    },
                );

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
                        cx.pressable_toggle_bool(&open);
                        trigger_id = Some(id);
                        vec![cx.container(ContainerProps::default(), |_cx| Vec::new())]
                    },
                );

                let popover_focus_id_out = popover_focus_id_out.clone();
                let popover_content_id_out = popover_content_id_out.clone();
                let popover = Popover::new(open.clone())
                    .auto_focus(true)
                    .arrow(arrow)
                    .on_open_auto_focus(on_open_auto_focus.clone())
                    .on_close_auto_focus(on_close_auto_focus.clone())
                    .into_element(
                        cx,
                        |_cx| trigger,
                        move |cx| {
                            let focusable = cx.pressable_with_id(
                                PressableProps {
                                    layout: {
                                        let mut layout = LayoutStyle::default();
                                        layout.size.width = Length::Px(Px(160.0));
                                        layout.size.height = Length::Px(Px(44.0));
                                        layout
                                    },
                                    enabled: true,
                                    focusable: true,
                                    ..Default::default()
                                },
                                |cx, _st, id| {
                                    popover_focus_id_out.set(Some(id));
                                    vec![cx.container(ContainerProps::default(), |_cx| Vec::new())]
                                },
                            );
                            let content = PopoverContent::new(vec![focusable]).into_element(cx);
                            popover_content_id_out.set(Some(content.id));
                            content
                        },
                    );

                vec![underlay, popover]
            });

        ui.set_root(root);
        OverlayController::render(ui, app, services, window, bounds);
        trigger_id.expect("trigger id")
    }

    fn render_popover_frame_with_open_auto_focus_redirect_target(
        ui: &mut UiTree<App>,
        app: &mut App,
        services: &mut dyn fret_core::UiServices,
        window: AppWindowId,
        bounds: Rect,
        open: Model<bool>,
        arrow: bool,
        underlay_id_out: Rc<Cell<Option<fret_ui::elements::GlobalElementId>>>,
        underlay_id_cell: Option<Arc<Mutex<Option<fret_ui::elements::GlobalElementId>>>>,
        initial_focus_id_out: Rc<Cell<Option<fret_ui::elements::GlobalElementId>>>,
        redirect_focus_id_cell: Arc<Mutex<Option<fret_ui::elements::GlobalElementId>>>,
        redirect_focus_id_out: Rc<Cell<Option<fret_ui::elements::GlobalElementId>>>,
        on_open_auto_focus: Option<OnOpenAutoFocus>,
    ) -> fret_ui::elements::GlobalElementId {
        OverlayController::begin_frame(app, window);

        let mut trigger_id: Option<fret_ui::elements::GlobalElementId> = None;

        let root =
            fret_ui::declarative::render_root(ui, app, services, window, bounds, "test", |cx| {
                let underlay_id_cell = underlay_id_cell.clone();
                let underlay_id_out = underlay_id_out.clone();
                let underlay = cx.pressable_with_id(
                    PressableProps {
                        layout: {
                            let mut layout = LayoutStyle::default();
                            layout.size.width = Length::Px(Px(120.0));
                            layout.size.height = Length::Px(Px(40.0));
                            layout.inset.top = Some(Px(300.0));
                            layout.inset.left = Some(Px(400.0));
                            layout.position = fret_ui::element::PositionStyle::Absolute;
                            layout
                        },
                        enabled: true,
                        focusable: true,
                        ..Default::default()
                    },
                    move |cx, _st, id| {
                        underlay_id_out.set(Some(id));
                        if let Some(underlay_id_cell) = underlay_id_cell.as_ref() {
                            *underlay_id_cell.lock().unwrap_or_else(|e| e.into_inner()) = Some(id);
                        }
                        vec![cx.container(ContainerProps::default(), |_cx| Vec::new())]
                    },
                );

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
                        cx.pressable_toggle_bool(&open);
                        trigger_id = Some(id);
                        vec![cx.container(ContainerProps::default(), |_cx| Vec::new())]
                    },
                );

                let redirect_focus_id_cell = redirect_focus_id_cell.clone();
                let popover = Popover::new(open.clone())
                    .auto_focus(true)
                    .arrow(arrow)
                    .on_open_auto_focus(on_open_auto_focus.clone())
                    .into_element(
                        cx,
                        |_cx| trigger,
                        move |cx| {
                            let initial = cx.pressable_with_id(
                                PressableProps {
                                    layout: {
                                        let mut layout = LayoutStyle::default();
                                        layout.size.width = Length::Px(Px(160.0));
                                        layout.size.height = Length::Px(Px(44.0));
                                        layout
                                    },
                                    enabled: true,
                                    focusable: true,
                                    ..Default::default()
                                },
                                |cx, _st, id| {
                                    initial_focus_id_out.set(Some(id));
                                    vec![cx.container(ContainerProps::default(), |_cx| Vec::new())]
                                },
                            );

                            let redirect = cx.pressable_with_id(
                                PressableProps {
                                    layout: {
                                        let mut layout = LayoutStyle::default();
                                        layout.size.width = Length::Px(Px(160.0));
                                        layout.size.height = Length::Px(Px(44.0));
                                        layout
                                    },
                                    enabled: true,
                                    focusable: true,
                                    ..Default::default()
                                },
                                |cx, _st, id| {
                                    redirect_focus_id_out.set(Some(id));
                                    *redirect_focus_id_cell
                                        .lock()
                                        .unwrap_or_else(|e| e.into_inner()) = Some(id);
                                    vec![cx.container(ContainerProps::default(), |_cx| Vec::new())]
                                },
                            );

                            PopoverContent::new(vec![initial, redirect]).into_element(cx)
                        },
                    );

                vec![underlay, popover]
            });

        ui.set_root(root);
        OverlayController::render(ui, app, services, window, bounds);
        trigger_id.expect("trigger id")
    }

    #[test]
    fn popover_open_auto_focus_can_be_prevented() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let open = app.models_mut().insert(false);
        let underlay_id: Rc<Cell<Option<fret_ui::elements::GlobalElementId>>> =
            Rc::new(Cell::new(None));
        let popover_focus_id: Rc<Cell<Option<fret_ui::elements::GlobalElementId>>> =
            Rc::new(Cell::new(None));
        let popover_content_id: Rc<Cell<Option<fret_ui::elements::GlobalElementId>>> =
            Rc::new(Cell::new(None));

        let calls = Arc::new(AtomicUsize::new(0));
        let calls_for_handler = calls.clone();
        let handler: OnOpenAutoFocus = Arc::new(move |_host, _action_cx, req| {
            calls_for_handler.fetch_add(1, Ordering::SeqCst);
            req.prevent_default();
        });

        let mut services = FakeServices;
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            CoreSize::new(Px(800.0), Px(600.0)),
        );

        app.set_frame_id(FrameId(1));
        let trigger = render_popover_frame_with_auto_focus_hooks(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            open.clone(),
            false,
            underlay_id.clone(),
            None,
            popover_focus_id.clone(),
            popover_content_id.clone(),
            None,
            None,
        );
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        let trigger_node =
            fret_ui::elements::node_for_element(&mut app, window, trigger).expect("trigger");
        ui.set_focus(Some(trigger_node));

        let _ = app.models_mut().update(&open, |v| *v = true);

        app.set_frame_id(FrameId(2));
        let _ = render_popover_frame_with_auto_focus_hooks(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            open.clone(),
            false,
            underlay_id,
            None,
            popover_focus_id.clone(),
            popover_content_id,
            Some(handler),
            None,
        );
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        assert!(
            calls.load(Ordering::SeqCst) > 0,
            "expected on_open_auto_focus to run"
        );

        let popover_focus = popover_focus_id.get().expect("popover focus element");
        let popover_focus_node =
            fret_ui::elements::node_for_element(&mut app, window, popover_focus)
                .expect("focusable");
        assert_ne!(
            ui.focus(),
            Some(popover_focus_node),
            "expected preventDefault to suppress focusing the popover content"
        );
        assert_eq!(
            ui.focus(),
            Some(trigger_node),
            "expected focus to remain on trigger when open autofocus is prevented"
        );
    }

    #[test]
    fn popover_open_auto_focus_can_be_redirected() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let open = app.models_mut().insert(false);
        let underlay_id: Rc<Cell<Option<fret_ui::elements::GlobalElementId>>> =
            Rc::new(Cell::new(None));
        let initial_focus_id: Rc<Cell<Option<fret_ui::elements::GlobalElementId>>> =
            Rc::new(Cell::new(None));
        let redirect_focus_id_out: Rc<Cell<Option<fret_ui::elements::GlobalElementId>>> =
            Rc::new(Cell::new(None));
        let redirect_focus_id_cell: Arc<Mutex<Option<fret_ui::elements::GlobalElementId>>> =
            Arc::new(Mutex::new(None));
        let redirect_focus_id_for_handler = redirect_focus_id_cell.clone();

        let calls = Arc::new(AtomicUsize::new(0));
        let calls_for_handler = calls.clone();
        let handler: OnOpenAutoFocus = Arc::new(move |host, _action_cx, req| {
            calls_for_handler.fetch_add(1, Ordering::SeqCst);
            let id = redirect_focus_id_for_handler
                .lock()
                .unwrap_or_else(|e| e.into_inner())
                .clone();
            if let Some(id) = id {
                host.request_focus(id);
            }
            req.prevent_default();
        });

        let mut services = FakeServices;
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            CoreSize::new(Px(800.0), Px(600.0)),
        );

        app.set_frame_id(FrameId(1));
        let trigger = render_popover_frame_with_open_auto_focus_redirect_target(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            open.clone(),
            false,
            underlay_id.clone(),
            None,
            initial_focus_id.clone(),
            redirect_focus_id_cell.clone(),
            redirect_focus_id_out.clone(),
            None,
        );
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        let trigger_node =
            fret_ui::elements::node_for_element(&mut app, window, trigger).expect("trigger");
        ui.set_focus(Some(trigger_node));

        let _ = app.models_mut().update(&open, |v| *v = true);

        app.set_frame_id(FrameId(2));
        let _ = render_popover_frame_with_open_auto_focus_redirect_target(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            open.clone(),
            false,
            underlay_id,
            None,
            initial_focus_id.clone(),
            redirect_focus_id_cell,
            redirect_focus_id_out.clone(),
            Some(handler),
        );
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        assert!(
            calls.load(Ordering::SeqCst) > 0,
            "expected on_open_auto_focus to run"
        );

        let initial_focus = initial_focus_id.get().expect("initial focus element");
        let initial_focus_node =
            fret_ui::elements::node_for_element(&mut app, window, initial_focus)
                .expect("initial focus");
        let redirect_focus = redirect_focus_id_out.get().expect("redirect focus element");
        let redirect_focus_node =
            fret_ui::elements::node_for_element(&mut app, window, redirect_focus)
                .expect("redirect focus");
        assert_ne!(
            ui.focus(),
            Some(initial_focus_node),
            "expected redirect to override the default initial focus"
        );
        assert_eq!(
            ui.focus(),
            Some(redirect_focus_node),
            "expected open autofocus redirect to win when preventDefault is set"
        );
    }

    #[test]
    fn popover_close_auto_focus_can_be_prevented_and_redirected() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let open = app.models_mut().insert(true);
        let underlay_id: Rc<Cell<Option<fret_ui::elements::GlobalElementId>>> =
            Rc::new(Cell::new(None));
        let popover_focus_id: Rc<Cell<Option<fret_ui::elements::GlobalElementId>>> =
            Rc::new(Cell::new(None));
        let popover_content_id: Rc<Cell<Option<fret_ui::elements::GlobalElementId>>> =
            Rc::new(Cell::new(None));

        let underlay_id_cell: Arc<Mutex<Option<fret_ui::elements::GlobalElementId>>> =
            Arc::new(Mutex::new(None));
        let underlay_id_for_handler = underlay_id_cell.clone();

        let calls = Arc::new(AtomicUsize::new(0));
        let calls_for_handler = calls.clone();
        let handler: OnCloseAutoFocus = Arc::new(move |host, _action_cx, req| {
            calls_for_handler.fetch_add(1, Ordering::SeqCst);
            let id = underlay_id_for_handler
                .lock()
                .unwrap_or_else(|e| e.into_inner())
                .clone();
            if let Some(id) = id {
                host.request_focus(id);
            }
            req.prevent_default();
        });

        let mut services = FakeServices;
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            CoreSize::new(Px(800.0), Px(600.0)),
        );

        app.set_frame_id(FrameId(1));
        let _trigger = render_popover_frame_with_auto_focus_hooks(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            open.clone(),
            false,
            underlay_id.clone(),
            Some(underlay_id_cell.clone()),
            popover_focus_id.clone(),
            popover_content_id.clone(),
            None,
            Some(handler.clone()),
        );
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        // Popover content needs the trigger bounds from the previous frame.
        app.set_frame_id(FrameId(2));
        let _ = render_popover_frame_with_auto_focus_hooks(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            open.clone(),
            false,
            underlay_id.clone(),
            Some(underlay_id_cell.clone()),
            popover_focus_id.clone(),
            popover_content_id.clone(),
            None,
            Some(handler.clone()),
        );
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        let popover_focus = popover_focus_id.get().expect("popover focus element");
        let popover_focus_node =
            fret_ui::elements::node_for_element(&mut app, window, popover_focus)
                .expect("focusable");
        ui.set_focus(Some(popover_focus_node));

        let _ = app.models_mut().update(&open, |v| *v = false);

        let settle_frames =
            fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 as usize + 2;
        for i in 0..settle_frames {
            app.set_frame_id(FrameId(3 + i as u64));
            let _ = render_popover_frame_with_auto_focus_hooks(
                &mut ui,
                &mut app,
                &mut services,
                window,
                bounds,
                open.clone(),
                false,
                underlay_id.clone(),
                Some(underlay_id_cell.clone()),
                popover_focus_id.clone(),
                popover_content_id.clone(),
                None,
                Some(handler.clone()),
            );
            ui.layout_all(&mut app, &mut services, bounds, 1.0);
        }

        let underlay = underlay_id.get().expect("underlay element");
        let underlay_node =
            fret_ui::elements::node_for_element(&mut app, window, underlay).expect("underlay node");
        assert!(
            calls.load(Ordering::SeqCst) > 0,
            "expected on_close_auto_focus to run"
        );
        assert_eq!(
            ui.focus(),
            Some(underlay_node),
            "expected preventDefault close autofocus to allow redirecting focus"
        );
    }

    #[test]
    fn popover_can_consume_outside_pointer_down_events() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let open = app.models_mut().insert(true);
        let underlay_activated = app.models_mut().insert(false);

        let mut services = FakeServices;
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            CoreSize::new(Px(800.0), Px(600.0)),
        );

        OverlayController::begin_frame(&mut app, window);
        let root = fret_ui::declarative::render_root(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "test",
            |cx| {
                let underlay_activated = underlay_activated.clone();
                let underlay = cx.pressable(
                    PressableProps {
                        layout: {
                            let mut layout = LayoutStyle::default();
                            layout.size.width = Length::Px(Px(120.0));
                            layout.size.height = Length::Px(Px(40.0));
                            layout.inset.top = Some(Px(300.0));
                            layout.inset.left = Some(Px(400.0));
                            layout.position = fret_ui::element::PositionStyle::Absolute;
                            layout
                        },
                        enabled: true,
                        focusable: true,
                        ..Default::default()
                    },
                    move |cx, _st| {
                        cx.pressable_set_bool(&underlay_activated, true);
                        vec![cx.container(ContainerProps::default(), |_cx| Vec::new())]
                    },
                );

                let trigger = cx.pressable(
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
                    |cx, _st| {
                        cx.pressable_toggle_bool(&open);
                        vec![cx.container(ContainerProps::default(), |_cx| Vec::new())]
                    },
                );

                let popover = Popover::new(open.clone())
                    .consume_outside_pointer_events(true)
                    .into_element(
                        cx,
                        |_cx| trigger,
                        |cx| {
                            PopoverContent::new(vec![
                                cx.container(ContainerProps::default(), |_cx| Vec::new()),
                            ])
                            .into_element(cx)
                        },
                    );

                vec![underlay, popover]
            },
        );
        ui.set_root(root);
        OverlayController::render(&mut ui, &mut app, &mut services, window, bounds);
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        // Click "outside" the popover, on the underlay. The popover should close, but the underlay
        // should not activate because pointer-down dispatch is suppressed.
        let underlay_point = Point::new(Px(410.0), Px(310.0));
        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::Pointer(fret_core::PointerEvent::Down {
                pointer_id: fret_core::PointerId(0),
                position: underlay_point,
                button: MouseButton::Left,
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
                position: underlay_point,
                button: MouseButton::Left,
                modifiers: fret_core::Modifiers::default(),
                is_click: true,
                pointer_type: fret_core::PointerType::Mouse,
                click_count: 1,
            }),
        );

        assert_eq!(app.models().get_copied(&open), Some(false));
        assert_eq!(app.models().get_copied(&underlay_activated), Some(false));
    }

    #[test]
    fn popover_outside_press_can_be_intercepted() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let open = app.models_mut().insert(true);
        let underlay_activated = app.models_mut().insert(false);

        let dismiss_reason: Rc<Cell<Option<fret_ui::action::DismissReason>>> =
            Rc::new(Cell::new(None));
        let dismiss_reason_cell = dismiss_reason.clone();
        let handler: OnDismissRequest = Arc::new(move |_host, _cx, req| {
            dismiss_reason_cell.set(Some(req.reason));
            req.prevent_default();
        });

        let mut services = FakeServices;
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            CoreSize::new(Px(800.0), Px(600.0)),
        );

        OverlayController::begin_frame(&mut app, window);
        let root = fret_ui::declarative::render_root(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "test",
            |cx| {
                let underlay = {
                    let underlay_activated = underlay_activated.clone();
                    cx.pressable(
                        PressableProps {
                            layout: {
                                let mut layout = LayoutStyle::default();
                                layout.size.width = Length::Px(Px(120.0));
                                layout.size.height = Length::Px(Px(40.0));
                                layout.inset.top = Some(Px(300.0));
                                layout.inset.left = Some(Px(400.0));
                                layout.position = fret_ui::element::PositionStyle::Absolute;
                                layout
                            },
                            enabled: true,
                            focusable: true,
                            ..Default::default()
                        },
                        move |cx, _st| {
                            cx.pressable_set_bool(&underlay_activated, true);
                            vec![cx.container(ContainerProps::default(), |_cx| Vec::new())]
                        },
                    )
                };

                let trigger = cx.pressable(
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
                    |cx, _st| {
                        cx.pressable_toggle_bool(&open);
                        vec![cx.container(ContainerProps::default(), |_cx| Vec::new())]
                    },
                );

                let popover = Popover::new(open.clone())
                    .on_dismiss_request(Some(handler.clone()))
                    .into_element(
                        cx,
                        |_cx| trigger,
                        |cx| {
                            PopoverContent::new(vec![
                                cx.container(ContainerProps::default(), |_cx| Vec::new()),
                            ])
                            .into_element(cx)
                        },
                    );

                vec![underlay, popover]
            },
        );
        ui.set_root(root);
        OverlayController::render(&mut ui, &mut app, &mut services, window, bounds);
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        let underlay_point = Point::new(Px(410.0), Px(310.0));
        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::Pointer(fret_core::PointerEvent::Down {
                pointer_id: fret_core::PointerId(0),
                position: underlay_point,
                button: MouseButton::Left,
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
                position: underlay_point,
                button: MouseButton::Left,
                modifiers: fret_core::Modifiers::default(),
                is_click: true,
                pointer_type: fret_core::PointerType::Mouse,
                click_count: 1,
            }),
        );

        assert_eq!(app.models().get_copied(&open), Some(true));
        assert_eq!(
            app.models().get_copied(&underlay_activated),
            Some(true),
            "expected click-through outside press to still activate the underlay when dismissal is prevented"
        );
        let reason = dismiss_reason.get();
        let Some(fret_ui::action::DismissReason::OutsidePress { pointer }) = reason else {
            panic!("expected outside-press dismissal, got {reason:?}");
        };
        let Some(cx) = pointer else {
            panic!("expected pointer payload for outside-press dismissal");
        };
        assert_eq!(cx.pointer_id, fret_core::PointerId(0));
        assert_eq!(cx.pointer_type, fret_core::PointerType::Mouse);
        assert_eq!(cx.button, MouseButton::Left);
        assert_eq!(cx.modifiers, fret_core::Modifiers::default());
        assert_eq!(cx.click_count, 1);
    }

    #[test]
    fn popover_trigger_exposes_expanded_and_controls_semantics() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let open = app.models_mut().insert(false);
        let mut services = FakeServices;
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            CoreSize::new(Px(800.0), Px(600.0)),
        );

        let underlay_id: Rc<Cell<Option<fret_ui::elements::GlobalElementId>>> =
            Rc::new(Cell::new(None));
        let popover_focus_id: Rc<Cell<Option<fret_ui::elements::GlobalElementId>>> =
            Rc::new(Cell::new(None));
        let popover_content_id: Rc<Cell<Option<fret_ui::elements::GlobalElementId>>> =
            Rc::new(Cell::new(None));

        // Frame 1: closed.
        app.set_frame_id(FrameId(1));
        let trigger_element = render_popover_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            open.clone(),
            false,
            underlay_id,
            popover_focus_id,
            popover_content_id.clone(),
        );
        ui.request_semantics_snapshot();
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        let trigger_node = fret_ui::elements::node_for_element(&mut app, window, trigger_element)
            .expect("trigger");
        let snap = ui.semantics_snapshot().expect("semantics snapshot");
        let trigger_sem = snap
            .nodes
            .iter()
            .find(|n| n.id == trigger_node)
            .expect("trigger semantics node");
        assert_eq!(trigger_sem.flags.expanded, false);
        assert!(
            trigger_sem.controls.is_empty(),
            "closed popover should not expose controls to an unmounted content node"
        );

        // Open via trigger click.
        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::Pointer(fret_core::PointerEvent::Down {
                pointer_id: fret_core::PointerId(0),
                position: Point::new(Px(10.0), Px(10.0)),
                button: MouseButton::Left,
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
                position: Point::new(Px(10.0), Px(10.0)),
                button: MouseButton::Left,
                modifiers: fret_core::Modifiers::default(),
                is_click: true,
                pointer_type: fret_core::PointerType::Mouse,
                click_count: 1,
            }),
        );
        assert_eq!(app.models().get_copied(&open), Some(true));

        // Frame 2: open.
        app.set_frame_id(FrameId(2));
        let _ = render_popover_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            open.clone(),
            false,
            Rc::new(Cell::new(None)),
            Rc::new(Cell::new(None)),
            popover_content_id,
        );
        ui.request_semantics_snapshot();
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        let snap = ui.semantics_snapshot().expect("semantics snapshot");
        let trigger_sem = snap
            .nodes
            .iter()
            .find(|n| n.id == trigger_node)
            .expect("trigger semantics node");
        assert!(
            trigger_sem.flags.expanded,
            "trigger should set expanded=true while the popover is open"
        );
        let controlled = trigger_sem.controls.first().copied().expect("controls");
        let controlled_node = snap
            .nodes
            .iter()
            .find(|n| n.id == controlled)
            .expect("controlled node");
        assert_eq!(controlled_node.role, SemanticsRole::Dialog);
    }

    fn render_popover_in_clipped_surface_frame(
        ui: &mut UiTree<App>,
        app: &mut App,
        services: &mut dyn fret_core::UiServices,
        window: AppWindowId,
        bounds: Rect,
        open: Model<bool>,
        popover_content_id_out: Rc<Cell<Option<fret_ui::elements::GlobalElementId>>>,
    ) -> fret_ui::elements::GlobalElementId {
        OverlayController::begin_frame(app, window);

        let trigger_id_out: Rc<Cell<Option<fret_ui::elements::GlobalElementId>>> =
            Rc::new(Cell::new(None));

        let root =
            fret_ui::declarative::render_root(ui, app, services, window, bounds, "test", |cx| {
                let clipped_surface = cx.container(
                    ContainerProps {
                        layout: {
                            let mut layout = LayoutStyle::default();
                            layout.size.width = Length::Px(Px(200.0));
                            layout.size.height = Length::Px(Px(80.0));
                            layout.overflow = Overflow::Clip;
                            layout
                        },
                        corner_radii: Corners::all(Px(12.0)),
                        ..Default::default()
                    },
                    {
                        let trigger_id_out = trigger_id_out.clone();
                        move |cx| {
                            let popover_content_id_out = popover_content_id_out.clone();
                            vec![
                                Popover::new(open.clone())
                                    .side(PopoverSide::Bottom)
                                    .into_element(
                                        cx,
                                        |cx| {
                                            cx.pressable_with_id(
                                                PressableProps {
                                                    layout: {
                                                        let mut layout = LayoutStyle::default();
                                                        layout.size.width = Length::Px(Px(120.0));
                                                        layout.size.height = Length::Px(Px(40.0));
                                                        layout.position =
                                                        fret_ui::element::PositionStyle::Absolute;
                                                        layout.inset.top = Some(Px(20.0));
                                                        layout.inset.left = Some(Px(10.0));
                                                        layout
                                                    },
                                                    enabled: true,
                                                    focusable: true,
                                                    ..Default::default()
                                                },
                                                |cx, _st, id| {
                                                    cx.pressable_toggle_bool(&open);
                                                    trigger_id_out.set(Some(id));
                                                    vec![cx.container(
                                                        ContainerProps::default(),
                                                        |_cx| Vec::new(),
                                                    )]
                                                },
                                            )
                                        },
                                        move |cx| {
                                            let focusable = cx.pressable_with_id(
                                                PressableProps {
                                                    layout: {
                                                        let mut layout = LayoutStyle::default();
                                                        layout.size.width = Length::Px(Px(220.0));
                                                        layout.size.height = Length::Px(Px(120.0));
                                                        layout
                                                    },
                                                    enabled: true,
                                                    focusable: true,
                                                    ..Default::default()
                                                },
                                                |cx, _st, _id| {
                                                    vec![cx.container(
                                                        ContainerProps::default(),
                                                        |_cx| Vec::new(),
                                                    )]
                                                },
                                            );
                                            let content = PopoverContent::new(vec![focusable])
                                                .into_element(cx);
                                            popover_content_id_out.set(Some(content.id));
                                            content
                                        },
                                    ),
                            ]
                        }
                    },
                );
                vec![clipped_surface]
            });

        ui.set_root(root);
        OverlayController::render(ui, app, services, window, bounds);
        trigger_id_out.get().expect("trigger id")
    }

    #[test]
    fn popover_outside_press_closes_without_overriding_new_focus() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let open = app.models_mut().insert(false);
        let underlay_id: Rc<Cell<Option<fret_ui::elements::GlobalElementId>>> =
            Rc::new(Cell::new(None));
        let popover_focus_cell: Rc<Cell<Option<fret_ui::elements::GlobalElementId>>> =
            Rc::new(Cell::new(None));

        let mut services = FakeServices;
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            CoreSize::new(Px(800.0), Px(600.0)),
        );

        // First frame: closed, establish trigger bounds.
        app.set_frame_id(FrameId(1));
        let _trigger = render_popover_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            open.clone(),
            false,
            underlay_id.clone(),
            popover_focus_cell.clone(),
            Rc::new(Cell::new(None)),
        );
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        // Open via trigger click.
        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::Pointer(fret_core::PointerEvent::Down {
                pointer_id: fret_core::PointerId(0),
                position: Point::new(Px(10.0), Px(10.0)),
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
                position: Point::new(Px(10.0), Px(10.0)),
                button: fret_core::MouseButton::Left,
                modifiers: fret_core::Modifiers::default(),
                is_click: true,
                pointer_type: fret_core::PointerType::Mouse,
                click_count: 1,
            }),
        );
        assert_eq!(app.models().get_copied(&open), Some(true));

        // Second frame: open + auto-focus inside popover.
        app.set_frame_id(FrameId(2));
        let _trigger = render_popover_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            open.clone(),
            false,
            underlay_id.clone(),
            popover_focus_cell.clone(),
            Rc::new(Cell::new(None)),
        );
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        let popover_focus_element_id = popover_focus_cell.get().expect("popover focus element id");
        let popover_focus_node =
            fret_ui::elements::node_for_element(&mut app, window, popover_focus_element_id)
                .expect("popover focus node");
        assert_eq!(ui.focus(), Some(popover_focus_node));

        // Click the underlay while the popover is open: should close the popover (observer pass)
        // and still focus the underlay (click-through), without being overridden on close.
        let click = Point::new(Px(410.0), Px(310.0));
        let click_debug_before = ui.debug_hit_test(click);
        let click_hit_before = click_debug_before.hit;
        let click_path_before = click_hit_before
            .map(|hit| ui.debug_node_path(hit))
            .unwrap_or_default();
        let click_hit_bounds_before = click_hit_before.and_then(|hit| ui.debug_node_bounds(hit));
        let click_hit_visual_bounds_before =
            click_hit_before.and_then(|hit| ui.debug_node_visual_bounds(hit));
        let click_layers_before = ui.debug_layers_in_paint_order();
        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::Pointer(fret_core::PointerEvent::Down {
                pointer_id: fret_core::PointerId(0),
                position: click,
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
                position: click,
                button: fret_core::MouseButton::Left,
                modifiers: fret_core::Modifiers::default(),
                is_click: true,
                pointer_type: fret_core::PointerType::Mouse,
                click_count: 1,
            }),
        );
        assert_eq!(
            app.models().get_copied(&open),
            Some(false),
            "expected popover to close on outside press; click={click:?} hit_before={click_hit_before:?} hit_bounds_before={click_hit_bounds_before:?} hit_visual_bounds_before={click_hit_visual_bounds_before:?} path_before={click_path_before:?} active_roots_before={:?} barrier_root_before={:?} layers_before={click_layers_before:?}",
            click_debug_before.active_layer_roots,
            click_debug_before.barrier_root
        );

        // Third frame: popover hidden, focus should remain on the underlay.
        app.set_frame_id(FrameId(3));
        let _trigger = render_popover_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            open.clone(),
            false,
            underlay_id.clone(),
            popover_focus_cell.clone(),
            Rc::new(Cell::new(None)),
        );
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        let underlay_id = underlay_id.get().expect("underlay element id");
        let underlay_node =
            fret_ui::elements::node_for_element(&mut app, window, underlay_id).expect("underlay");
        assert_eq!(ui.focus(), Some(underlay_node));
    }

    #[test]
    fn popover_portal_escapes_overflow_clip_ancestor() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let open = app.models_mut().insert(false);
        let popover_content_cell: Rc<Cell<Option<fret_ui::elements::GlobalElementId>>> =
            Rc::new(Cell::new(None));

        let mut services = FakeServices;
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            CoreSize::new(Px(800.0), Px(600.0)),
        );

        // Frame 1: closed, establish trigger bounds for the placement solver.
        app.set_frame_id(FrameId(1));
        let trigger_id = render_popover_in_clipped_surface_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            open.clone(),
            popover_content_cell.clone(),
        );
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        // Open via trigger click.
        let trigger_node = fret_ui::elements::node_for_element(&mut app, window, trigger_id)
            .expect("trigger node");
        let trigger_bounds = ui.debug_node_bounds(trigger_node).expect("trigger bounds");
        let click_point = Point::new(
            Px(trigger_bounds.origin.x.0 + 2.0),
            Px(trigger_bounds.origin.y.0 + 2.0),
        );

        let pre_hit = ui.debug_hit_test(click_point).hit.expect("pre-hit");
        let pre_path = ui.debug_node_path(pre_hit);
        assert!(
            pre_path.contains(&trigger_node),
            "expected click point to hit trigger subtree; point={click_point:?} hit={pre_hit:?} trigger={trigger_node:?} path={pre_path:?}"
        );
        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::Pointer(fret_core::PointerEvent::Down {
                pointer_id: fret_core::PointerId(0),
                position: click_point,
                button: MouseButton::Left,
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
                position: click_point,
                button: MouseButton::Left,
                modifiers: fret_core::Modifiers::default(),
                is_click: true,
                pointer_type: fret_core::PointerType::Mouse,
                click_count: 1,
            }),
        );
        assert_eq!(app.models().get_copied(&open), Some(true));

        // Frame 2: open, compute popover bounds and hit-test outside the clipped surface.
        app.set_frame_id(FrameId(2));
        let _trigger = render_popover_in_clipped_surface_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            open.clone(),
            popover_content_cell.clone(),
        );
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        let content_id = popover_content_cell.get().expect("popover content id");
        let content_node = fret_ui::elements::node_for_element(&mut app, window, content_id)
            .expect("content node");
        let content_bounds = ui
            .debug_node_visual_bounds(content_node)
            .expect("content bounds");

        let clip_bottom = 80.0f32;
        let target_y = (clip_bottom + 5.0).max(content_bounds.origin.y.0 + 2.0);
        let point = Point::new(Px(content_bounds.origin.x.0 + 5.0), Px(target_y));
        assert!(
            content_bounds.contains(point),
            "expected point inside popover bounds; point={point:?} bounds={content_bounds:?}"
        );
        assert!(
            point.y.0 > clip_bottom,
            "expected point below the clipped surface; y={} clip_bottom={}",
            point.y.0,
            clip_bottom
        );

        let hit = ui
            .debug_hit_test(point)
            .hit
            .expect("expected hit in popover content outside clipped surface");
        let path = ui.debug_node_path(hit);
        assert!(
            path.contains(&content_node),
            "expected hit to be within popover content subtree; hit={hit:?} content={content_node:?} path={path:?}"
        );
    }

    #[test]
    fn popover_arrow_is_hit_testable_and_does_not_dismiss_on_click() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let open = app.models_mut().insert(false);
        let underlay_id: Rc<Cell<Option<fret_ui::elements::GlobalElementId>>> =
            Rc::new(Cell::new(None));
        let popover_focus_cell: Rc<Cell<Option<fret_ui::elements::GlobalElementId>>> =
            Rc::new(Cell::new(None));
        let popover_content_cell: Rc<Cell<Option<fret_ui::elements::GlobalElementId>>> =
            Rc::new(Cell::new(None));

        let mut services = FakeServices;
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            CoreSize::new(Px(800.0), Px(600.0)),
        );

        // Frame 1: closed, establish trigger bounds.
        app.set_frame_id(FrameId(1));
        let _trigger = render_popover_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            open.clone(),
            true,
            underlay_id.clone(),
            popover_focus_cell.clone(),
            popover_content_cell.clone(),
        );
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        // Open via trigger click.
        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::Pointer(fret_core::PointerEvent::Down {
                pointer_id: fret_core::PointerId(0),
                position: Point::new(Px(10.0), Px(10.0)),
                button: MouseButton::Left,
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
                position: Point::new(Px(10.0), Px(10.0)),
                button: MouseButton::Left,
                modifiers: fret_core::Modifiers::default(),
                is_click: true,
                pointer_type: fret_core::PointerType::Mouse,
                click_count: 1,
            }),
        );
        assert_eq!(app.models().get_copied(&open), Some(true));

        // Frame 2: open + arrow.
        app.set_frame_id(FrameId(2));
        let _trigger = render_popover_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            open.clone(),
            true,
            underlay_id.clone(),
            popover_focus_cell.clone(),
            popover_content_cell.clone(),
        );
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        let content_id = popover_content_cell.get().expect("popover content id");
        let content_node = fret_ui::elements::node_for_element(&mut app, window, content_id)
            .expect("content node");
        let content_bounds = ui
            .debug_node_visual_bounds(content_node)
            .expect("content bounds");

        // Click just above the panel: this should land on the arrow and not trigger outside-press
        // dismissal.
        let click = Point::new(
            Px(content_bounds.origin.x.0 + content_bounds.size.width.0 * 0.5),
            Px(content_bounds.origin.y.0 - 1.0),
        );
        let click_debug_before = ui.debug_hit_test(click);
        let click_hit_before = click_debug_before.hit;
        let click_path_before = click_hit_before
            .map(|hit| ui.debug_node_path(hit))
            .unwrap_or_default();
        let click_hit_bounds_before = click_hit_before.and_then(|hit| ui.debug_node_bounds(hit));
        let click_hit_visual_bounds_before =
            click_hit_before.and_then(|hit| ui.debug_node_visual_bounds(hit));
        let click_layers_before = ui.debug_layers_in_paint_order();

        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::Pointer(fret_core::PointerEvent::Down {
                pointer_id: fret_core::PointerId(0),
                position: click,
                button: MouseButton::Left,
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
                position: click,
                button: MouseButton::Left,
                modifiers: fret_core::Modifiers::default(),
                is_click: true,
                pointer_type: fret_core::PointerType::Mouse,
                click_count: 1,
            }),
        );

        assert_eq!(
            app.models().get_copied(&open),
            Some(true),
            "expected popover to remain open when clicking the arrow; click={click:?} hit_before={click_hit_before:?} hit_bounds_before={click_hit_bounds_before:?} hit_visual_bounds_before={click_hit_visual_bounds_before:?} path_before={click_path_before:?} active_roots_before={:?} barrier_root_before={:?} layers_before={click_layers_before:?}",
            click_debug_before.active_layer_roots,
            click_debug_before.barrier_root
        );
    }

    #[test]
    fn popover_anchor_override_changes_anchor_rect_passed_to_content() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let open_model = app.models_mut().insert(false);
        let anchor_id_out: Rc<Cell<Option<fret_ui::elements::GlobalElementId>>> =
            Rc::new(Cell::new(None));
        let anchor_rect_out: Rc<Cell<Option<Rect>>> = Rc::new(Cell::new(None));

        let mut services = FakeServices;
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            CoreSize::new(Px(800.0), Px(600.0)),
        );

        let render =
            |ui: &mut UiTree<App>, app: &mut App, services: &mut dyn fret_core::UiServices| {
                OverlayController::begin_frame(app, window);
                let anchor_id_out_for_frame = anchor_id_out.clone();
                let anchor_rect_out_for_frame = anchor_rect_out.clone();
                let open = open_model.clone();

                let root = fret_ui::declarative::render_root(
                    ui,
                    app,
                    services,
                    window,
                    bounds,
                    "test",
                    |cx| {
                        let anchor_id_out_for_anchor = anchor_id_out_for_frame.clone();
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

                        let anchor_id = anchor_id_out_for_frame.get().expect("anchor id");
                        let popover = Popover::new(open.clone())
                            .anchor_element(anchor_id)
                            .into_element_with_anchor(
                                cx,
                                move |cx| {
                                    let open = open.clone();
                                    cx.pressable(
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
                                        move |cx, _st| {
                                            cx.pressable_toggle_bool(&open);
                                            vec![]
                                        },
                                    )
                                },
                                move |cx, anchor_rect| {
                                    anchor_rect_out_for_frame.set(Some(anchor_rect));
                                    PopoverContent::new(vec![]).into_element(cx)
                                },
                            );

                        vec![anchor, popover]
                    },
                );

                ui.set_root(root);
                OverlayController::render(ui, app, services, window, bounds);
            };

        // Frame 1: closed, establish stable last-bounds for the anchor element.
        app.set_frame_id(FrameId(1));
        render(&mut ui, &mut app, &mut services);
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        // Open via trigger click.
        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::Pointer(fret_core::PointerEvent::Down {
                pointer_id: fret_core::PointerId(0),
                position: Point::new(Px(12.0), Px(12.0)),
                button: MouseButton::Left,
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
                position: Point::new(Px(12.0), Px(12.0)),
                button: MouseButton::Left,
                modifiers: fret_core::Modifiers::default(),
                is_click: true,
                pointer_type: fret_core::PointerType::Mouse,
                click_count: 1,
            }),
        );
        assert_eq!(app.models().get_copied(&open_model), Some(true));

        // Frame 2: open, content closure should observe the anchor override rect.
        app.set_frame_id(FrameId(2));
        render(&mut ui, &mut app, &mut services);
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        let anchor_rect = anchor_rect_out.get().expect("anchor rect");
        assert_eq!(
            anchor_rect,
            Rect::new(
                Point::new(Px(240.0), Px(120.0)),
                CoreSize::new(Px(50.0), Px(10.0))
            )
        );
    }

    #[test]
    fn popover_anchor_override_is_treated_as_dismissable_branch() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let open_model = app.models_mut().insert(false);
        let anchor_id_out: Rc<Cell<Option<fret_ui::elements::GlobalElementId>>> =
            Rc::new(Cell::new(None));

        let mut services = FakeServices;
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            CoreSize::new(Px(800.0), Px(600.0)),
        );

        let render =
            |ui: &mut UiTree<App>, app: &mut App, services: &mut dyn fret_core::UiServices| {
                OverlayController::begin_frame(app, window);
                let anchor_id_out_for_frame = anchor_id_out.clone();
                let open = open_model.clone();

                let root = fret_ui::declarative::render_root(
                    ui,
                    app,
                    services,
                    window,
                    bounds,
                    "test-branch",
                    |cx| {
                        let mut layout = LayoutStyle::default();
                        layout.size.width = Length::Px(Px(50.0));
                        layout.size.height = Length::Px(Px(10.0));
                        layout.inset.top = Some(Px(120.0));
                        layout.inset.left = Some(Px(240.0));
                        layout.position = fret_ui::element::PositionStyle::Absolute;

                        let anchor = cx.container(
                            ContainerProps {
                                layout,
                                ..Default::default()
                            },
                            |_cx| vec![],
                        );
                        anchor_id_out_for_frame.set(Some(anchor.id));

                        let anchor_id = anchor_id_out_for_frame.get().expect("anchor id");
                        let popover = Popover::new(open.clone())
                            .anchor_element(anchor_id)
                            .into_element(
                                cx,
                                move |cx| {
                                    let open = open.clone();
                                    cx.pressable(
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
                                        move |cx, _st| {
                                            cx.pressable_toggle_bool(&open);
                                            vec![]
                                        },
                                    )
                                },
                                move |cx| PopoverContent::new(vec![]).into_element(cx),
                            );

                        vec![anchor, popover]
                    },
                );

                ui.set_root(root);
                OverlayController::render(ui, app, services, window, bounds);
            };

        // Frame 1: closed.
        app.set_frame_id(FrameId(1));
        render(&mut ui, &mut app, &mut services);
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        // Open via trigger click.
        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::Pointer(fret_core::PointerEvent::Down {
                pointer_id: fret_core::PointerId(0),
                position: Point::new(Px(12.0), Px(12.0)),
                button: MouseButton::Left,
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
                position: Point::new(Px(12.0), Px(12.0)),
                button: MouseButton::Left,
                modifiers: fret_core::Modifiers::default(),
                is_click: true,
                pointer_type: fret_core::PointerType::Mouse,
                click_count: 1,
            }),
        );
        assert_eq!(app.models().get_copied(&open_model), Some(true));

        // Frame 2: open.
        app.set_frame_id(FrameId(2));
        render(&mut ui, &mut app, &mut services);
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        // Pointer down on the anchor element should NOT be treated as an outside press.
        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::Pointer(fret_core::PointerEvent::Down {
                pointer_id: fret_core::PointerId(0),
                position: Point::new(Px(245.0), Px(125.0)),
                button: MouseButton::Left,
                modifiers: fret_core::Modifiers::default(),
                pointer_type: fret_core::PointerType::Mouse,
                click_count: 1,
            }),
        );

        assert_eq!(app.models().get_copied(&open_model), Some(true));
    }

    #[test]
    fn modal_popover_outside_click_closes_without_activating_underlay() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let open = app.models_mut().insert(true);
        let underlay_activated = app.models_mut().insert(false);

        let mut services = FakeServices;
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            CoreSize::new(Px(800.0), Px(600.0)),
        );

        let next_frame = FrameId(app.frame_id().0.saturating_add(1));
        app.set_frame_id(next_frame);
        OverlayController::begin_frame(&mut app, window);
        let root = fret_ui::declarative::render_root(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "test",
            |cx| {
                let underlay_activated = underlay_activated.clone();
                let underlay = cx.pressable(
                    PressableProps {
                        layout: {
                            let mut layout = LayoutStyle::default();
                            layout.size.width = Length::Px(Px(120.0));
                            layout.size.height = Length::Px(Px(40.0));
                            layout.inset.top = Some(Px(300.0));
                            layout.inset.left = Some(Px(400.0));
                            layout.position = fret_ui::element::PositionStyle::Absolute;
                            layout
                        },
                        enabled: true,
                        focusable: true,
                        ..Default::default()
                    },
                    move |cx, _st| {
                        cx.pressable_set_bool(&underlay_activated, true);
                        vec![cx.container(ContainerProps::default(), |_cx| Vec::new())]
                    },
                );

                let trigger = cx.pressable(
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
                    |cx, _st| {
                        cx.pressable_toggle_bool(&open);
                        vec![cx.container(ContainerProps::default(), |_cx| Vec::new())]
                    },
                );

                let popover = Popover::new(open.clone())
                    .modal(true)
                    .auto_focus(true)
                    .into_element(
                        cx,
                        |_cx| trigger,
                        |cx| {
                            PopoverContent::new(vec![
                                cx.container(ContainerProps::default(), |_cx| Vec::new()),
                            ])
                            .into_element(cx)
                        },
                    );

                vec![underlay, popover]
            },
        );
        ui.set_root(root);
        OverlayController::render(&mut ui, &mut app, &mut services, window, bounds);
        ui.request_semantics_snapshot();
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        let snap = ui.semantics_snapshot().expect("semantics snapshot");
        let barrier_root = snap
            .barrier_root
            .expect("expected modal popover to install a modal barrier layer");
        assert!(
            snap.roots
                .iter()
                .any(|r| r.root == barrier_root && r.blocks_underlay_input),
            "expected barrier root to correspond to a blocks-underlay-input layer"
        );
        let base = snap
            .roots
            .iter()
            .find(|r| r.root == root)
            .expect("base layer root should appear in semantics snapshot");
        let barrier = snap
            .roots
            .iter()
            .find(|r| r.root == barrier_root)
            .expect("barrier root should appear in semantics snapshot");
        assert!(
            base.z_index < barrier.z_index,
            "expected modal barrier layer to be above the base layer: base_z={} barrier_z={}",
            base.z_index,
            barrier.z_index
        );

        // Click "outside" the popover, on the underlay. The modal barrier should catch the click:
        // the popover closes, and the underlay does not activate.
        let underlay_point = Point::new(Px(410.0), Px(310.0));
        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::Pointer(fret_core::PointerEvent::Down {
                pointer_id: fret_core::PointerId(0),
                position: underlay_point,
                button: MouseButton::Left,
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
                position: underlay_point,
                button: MouseButton::Left,
                modifiers: fret_core::Modifiers::default(),
                is_click: true,
                pointer_type: fret_core::PointerType::Mouse,
                click_count: 1,
            }),
        );

        assert_eq!(app.models().get_copied(&open), Some(false));
        assert_eq!(
            app.models().get_copied(&underlay_activated),
            Some(false),
            "underlay should not activate while modal popover is open"
        );
    }

    #[test]
    fn modal_popover_outside_click_can_be_intercepted() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let open = app.models_mut().insert(true);
        let underlay_activated = app.models_mut().insert(false);

        let dismiss_reason: Rc<Cell<Option<fret_ui::action::DismissReason>>> =
            Rc::new(Cell::new(None));
        let dismiss_reason_cell = dismiss_reason.clone();
        let handler: OnDismissRequest = Arc::new(move |_host, _cx, req| {
            dismiss_reason_cell.set(Some(req.reason));
            req.prevent_default();
        });

        let mut services = FakeServices;
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            CoreSize::new(Px(800.0), Px(600.0)),
        );

        let next_frame = FrameId(app.frame_id().0.saturating_add(1));
        app.set_frame_id(next_frame);
        OverlayController::begin_frame(&mut app, window);
        let root = fret_ui::declarative::render_root(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "test",
            |cx| {
                let underlay_activated = underlay_activated.clone();
                let underlay = cx.pressable(
                    PressableProps {
                        layout: {
                            let mut layout = LayoutStyle::default();
                            layout.size.width = Length::Px(Px(120.0));
                            layout.size.height = Length::Px(Px(40.0));
                            layout.inset.top = Some(Px(300.0));
                            layout.inset.left = Some(Px(400.0));
                            layout.position = fret_ui::element::PositionStyle::Absolute;
                            layout
                        },
                        enabled: true,
                        focusable: true,
                        ..Default::default()
                    },
                    move |cx, _st| {
                        cx.pressable_set_bool(&underlay_activated, true);
                        vec![cx.container(ContainerProps::default(), |_cx| Vec::new())]
                    },
                );

                let trigger = cx.pressable(
                    PressableProps {
                        layout: {
                            let mut layout = LayoutStyle::default();
                            layout.size.width = Length::Px(Px(120.0));
                            layout.size.height = Length::Px(Px(40.0));
                            layout.inset.top = Some(Px(0.0));
                            layout.inset.left = Some(Px(0.0));
                            layout.position = fret_ui::element::PositionStyle::Absolute;
                            layout
                        },
                        enabled: true,
                        focusable: true,
                        ..Default::default()
                    },
                    |cx, _st| vec![cx.container(ContainerProps::default(), |_cx| Vec::new())],
                );

                let popover = Popover::new(open.clone())
                    .modal(true)
                    .auto_focus(true)
                    .on_dismiss_request(Some(handler.clone()))
                    .into_element(
                        cx,
                        |_cx| trigger,
                        |cx| {
                            PopoverContent::new(vec![
                                cx.container(ContainerProps::default(), |_cx| Vec::new()),
                            ])
                            .into_element(cx)
                        },
                    );

                vec![underlay, popover]
            },
        );
        ui.set_root(root);
        OverlayController::render(&mut ui, &mut app, &mut services, window, bounds);
        ui.request_semantics_snapshot();
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        let snap = ui.semantics_snapshot().expect("semantics snapshot");
        let barrier_root = snap
            .barrier_root
            .expect("expected modal popover to install a modal barrier layer");
        assert!(
            snap.roots
                .iter()
                .any(|r| r.root == barrier_root && r.blocks_underlay_input),
            "expected barrier root to correspond to a blocks-underlay-input layer"
        );

        // Click "outside" the popover, on the underlay. The modal barrier should catch the click:
        // the popover stays open, and the underlay does not activate.
        let underlay_point = Point::new(Px(410.0), Px(310.0));
        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::Pointer(fret_core::PointerEvent::Down {
                pointer_id: fret_core::PointerId(0),
                position: underlay_point,
                button: MouseButton::Left,
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
                position: underlay_point,
                button: MouseButton::Left,
                modifiers: fret_core::Modifiers::default(),
                is_click: true,
                pointer_type: fret_core::PointerType::Mouse,
                click_count: 1,
            }),
        );

        assert_eq!(app.models().get_copied(&open), Some(true));
        assert_eq!(
            app.models().get_copied(&underlay_activated),
            Some(false),
            "underlay should not activate while modal popover is open"
        );
        let reason = dismiss_reason.get();
        let Some(fret_ui::action::DismissReason::OutsidePress { pointer }) = reason else {
            panic!("expected outside-press dismissal, got {reason:?}");
        };
        let Some(cx) = pointer else {
            panic!("expected pointer payload for outside-press dismissal");
        };
        assert_eq!(cx.pointer_id, fret_core::PointerId(0));
        assert_eq!(cx.pointer_type, fret_core::PointerType::Mouse);
        assert_eq!(cx.button, MouseButton::Left);
        assert_eq!(cx.modifiers, fret_core::Modifiers::default());
        assert_eq!(cx.click_count, 1);
    }

    fn apply_command_effects(ui: &mut UiTree<App>, app: &mut App, services: &mut FakeServices) {
        let effects = app.flush_effects();
        for effect in effects {
            let Effect::Command { window: _, command } = effect else {
                continue;
            };
            let _ = ui.dispatch_command(app, services, &command);
        }
    }

    #[test]
    fn modal_popover_traps_tab_navigation() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let open = app.models_mut().insert(false);
        let underlay_id: Rc<Cell<Option<fret_ui::elements::GlobalElementId>>> =
            Rc::new(Cell::new(None));
        let focusable_a_id: Rc<Cell<Option<fret_ui::elements::GlobalElementId>>> =
            Rc::new(Cell::new(None));
        let focusable_b_id: Rc<Cell<Option<fret_ui::elements::GlobalElementId>>> =
            Rc::new(Cell::new(None));

        let mut services = FakeServices;
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            CoreSize::new(Px(800.0), Px(600.0)),
        );

        let render_frame = |ui: &mut UiTree<App>, app: &mut App, services: &mut FakeServices| {
            OverlayController::begin_frame(app, window);

            let underlay_id = underlay_id.clone();
            let focusable_a_id = focusable_a_id.clone();
            let focusable_b_id = focusable_b_id.clone();

            let root = fret_ui::declarative::render_root(
                ui,
                app,
                services,
                window,
                bounds,
                "test",
                |cx| {
                    let underlay = cx.pressable_with_id(
                        PressableProps {
                            layout: {
                                let mut layout = LayoutStyle::default();
                                layout.size.width = Length::Px(Px(120.0));
                                layout.size.height = Length::Px(Px(40.0));
                                layout.inset.top = Some(Px(300.0));
                                layout.inset.left = Some(Px(400.0));
                                layout.position = fret_ui::element::PositionStyle::Absolute;
                                layout
                            },
                            enabled: true,
                            focusable: true,
                            ..Default::default()
                        },
                        |cx, _st, id| {
                            underlay_id.set(Some(id));
                            vec![cx.container(ContainerProps::default(), |_cx| Vec::new())]
                        },
                    );

                    let trigger = cx.pressable(
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
                        |cx, _st| {
                            cx.pressable_toggle_bool(&open);
                            vec![cx.container(ContainerProps::default(), |_cx| Vec::new())]
                        },
                    );

                    let popover = Popover::new(open.clone())
                        .modal(true)
                        .auto_focus(true)
                        .into_element(
                            cx,
                            |_cx| trigger,
                            |cx| {
                                let focusable_a =
                                    cx.pressable_with_id(
                                        PressableProps {
                                            layout: {
                                                let mut layout = LayoutStyle::default();
                                                layout.size.width = Length::Px(Px(200.0));
                                                layout.size.height = Length::Px(Px(44.0));
                                                layout
                                            },
                                            enabled: true,
                                            focusable: true,
                                            ..Default::default()
                                        },
                                        |cx, _st, id| {
                                            focusable_a_id.set(Some(id));
                                            vec![cx.container(ContainerProps::default(), |_cx| {
                                                Vec::new()
                                            })]
                                        },
                                    );

                                let focusable_b =
                                    cx.pressable_with_id(
                                        PressableProps {
                                            layout: {
                                                let mut layout = LayoutStyle::default();
                                                layout.size.width = Length::Px(Px(200.0));
                                                layout.size.height = Length::Px(Px(44.0));
                                                layout
                                            },
                                            enabled: true,
                                            focusable: true,
                                            ..Default::default()
                                        },
                                        |cx, _st, id| {
                                            focusable_b_id.set(Some(id));
                                            vec![cx.container(ContainerProps::default(), |_cx| {
                                                Vec::new()
                                            })]
                                        },
                                    );

                                PopoverContent::new(vec![focusable_a, focusable_b]).into_element(cx)
                            },
                        );

                    vec![underlay, popover]
                },
            );
            ui.set_root(root);
            OverlayController::render(ui, app, services, window, bounds);
        };

        // Frame 1: closed.
        app.set_frame_id(FrameId(1));
        render_frame(&mut ui, &mut app, &mut services);
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        // Frame 2: open.
        let _ = app.models_mut().update(&open, |v| *v = true);
        app.set_frame_id(FrameId(2));
        render_frame(&mut ui, &mut app, &mut services);
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        let underlay_element = underlay_id.get().expect("underlay element");
        let a_element = focusable_a_id.get().expect("focusable A element");
        let b_element = focusable_b_id.get().expect("focusable B element");

        let underlay_node = fret_ui::elements::node_for_element(&mut app, window, underlay_element)
            .expect("underlay");
        let a_node = fret_ui::elements::node_for_element(&mut app, window, a_element).expect("A");
        let b_node = fret_ui::elements::node_for_element(&mut app, window, b_element).expect("B");

        assert_eq!(ui.focus(), Some(a_node));

        // Tab -> next
        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::KeyDown {
                key: KeyCode::Tab,
                modifiers: Modifiers::default(),
                repeat: false,
            },
        );
        apply_command_effects(&mut ui, &mut app, &mut services);
        assert_ne!(ui.focus(), Some(underlay_node));
        assert!(
            ui.focus() == Some(a_node) || ui.focus() == Some(b_node),
            "trap-focus popover should keep focus inside content"
        );

        // Tab -> wrap
        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::KeyDown {
                key: KeyCode::Tab,
                modifiers: Modifiers::default(),
                repeat: false,
            },
        );
        apply_command_effects(&mut ui, &mut app, &mut services);
        assert_eq!(ui.focus(), Some(a_node));
        assert_ne!(ui.focus(), Some(underlay_node));

        // Shift+Tab -> previous (wrap)
        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::KeyDown {
                key: KeyCode::Tab,
                modifiers: Modifiers {
                    shift: true,
                    ..Modifiers::default()
                },
                repeat: false,
            },
        );
        apply_command_effects(&mut ui, &mut app, &mut services);
        assert_eq!(ui.focus(), Some(b_node));
        assert_ne!(ui.focus(), Some(underlay_node));
    }

    #[test]
    fn modal_popover_close_transition_keeps_modal_barrier_blocking_underlay() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let open = app.models_mut().insert(false);
        let underlay_activated = app.models_mut().insert(false);

        let mut services = FakeServices;
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            CoreSize::new(Px(800.0), Px(600.0)),
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
                    "modal-popover-close-transition-barrier",
                    |cx| {
                        let underlay_activated = underlay_activated.clone();
                        let underlay = cx.pressable(
                            PressableProps {
                                layout: {
                                    let mut layout = LayoutStyle::default();
                                    layout.size.width = Length::Fill;
                                    layout.size.height = Length::Fill;
                                    layout
                                },
                                enabled: true,
                                focusable: true,
                                ..Default::default()
                            },
                            move |cx, _st| {
                                cx.pressable_set_bool(&underlay_activated, true);
                                Vec::new()
                            },
                        );

                        let trigger = cx.pressable(
                            PressableProps {
                                layout: {
                                    let mut layout = LayoutStyle::default();
                                    layout.size.width = Length::Px(Px(120.0));
                                    layout.size.height = Length::Px(Px(40.0));
                                    layout.inset.left = Some(Px(20.0));
                                    layout.inset.top = Some(Px(20.0));
                                    layout.position = fret_ui::element::PositionStyle::Absolute;
                                    layout
                                },
                                enabled: true,
                                focusable: true,
                                ..Default::default()
                            },
                            |cx, _st| {
                                cx.pressable_toggle_bool(&open);
                                Vec::new()
                            },
                        );

                        let popover = Popover::new(open.clone())
                            .modal(true)
                            .auto_focus(false)
                            .into_element(
                                cx,
                                |_cx| trigger,
                                |cx| {
                                    PopoverContent::new(vec![
                                        cx.container(ContainerProps::default(), |_cx| Vec::new()),
                                    ])
                                    .into_element(cx)
                                },
                            );

                        vec![underlay, popover]
                    },
                );
                ui.set_root(root);
                OverlayController::render(ui, app, services, window, bounds);
                ui.request_semantics_snapshot();
                ui.layout_all(app, services, bounds, 1.0);
            };

        // Frame 1: closed.
        render_frame(&mut ui, &mut app, &mut services, 1);

        let _ = app.models_mut().update(&open, |v| *v = true);

        // Frame 2: open -> barrier root should exist.
        render_frame(&mut ui, &mut app, &mut services, 2);
        let snap = ui.semantics_snapshot().expect("semantics snapshot");
        assert!(
            snap.barrier_root.is_some(),
            "expected modal popover to install a modal barrier root"
        );

        let _ = app.models_mut().update(&open, |v| *v = false);

        // Frame 3: closing (present=true, interactive=false) -> barrier must remain active.
        render_frame(&mut ui, &mut app, &mut services, 3);
        let snap = ui.semantics_snapshot().expect("semantics snapshot");
        let barrier_root = snap
            .barrier_root
            .expect("expected barrier root to remain while the modal popover is closing");
        let barrier_layer = ui.node_layer(barrier_root).expect("barrier layer");
        let barrier = ui
            .debug_layers_in_paint_order()
            .into_iter()
            .find(|l| l.id == barrier_layer)
            .expect("barrier debug layer info");
        assert!(barrier.visible);
        assert!(barrier.hit_testable);
        assert!(
            barrier.blocks_underlay_input,
            "expected modal barrier layer to block underlay input"
        );

        // Click the underlay. The modal barrier should block the click-through while closing.
        let click = Point::new(Px(10.0), Px(10.0));
        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::Pointer(fret_core::PointerEvent::Down {
                pointer_id: fret_core::PointerId(0),
                position: click,
                button: MouseButton::Left,
                modifiers: Modifiers::default(),
                pointer_type: fret_core::PointerType::Mouse,
                click_count: 1,
            }),
        );
        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::Pointer(fret_core::PointerEvent::Up {
                pointer_id: fret_core::PointerId(0),
                position: click,
                button: MouseButton::Left,
                modifiers: Modifiers::default(),
                is_click: true,
                pointer_type: fret_core::PointerType::Mouse,
                click_count: 1,
            }),
        );
        assert_eq!(
            app.models().get_copied(&underlay_activated),
            Some(false),
            "underlay should remain inert while the modal popover is closing"
        );

        // After the exit transition settles, the barrier must drop and the underlay becomes
        // interactive again.
        let settle_frames = fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2;
        for i in 0..settle_frames {
            render_frame(&mut ui, &mut app, &mut services, 4 + i);
        }

        let snap = ui.semantics_snapshot().expect("semantics snapshot");
        assert!(
            snap.barrier_root.is_none(),
            "expected barrier root to clear once the exit transition completes"
        );

        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::Pointer(fret_core::PointerEvent::Down {
                pointer_id: fret_core::PointerId(1),
                position: click,
                button: MouseButton::Left,
                modifiers: Modifiers::default(),
                pointer_type: fret_core::PointerType::Mouse,
                click_count: 1,
            }),
        );
        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::Pointer(fret_core::PointerEvent::Up {
                pointer_id: fret_core::PointerId(1),
                position: click,
                button: MouseButton::Left,
                modifiers: Modifiers::default(),
                is_click: true,
                pointer_type: fret_core::PointerType::Mouse,
                click_count: 1,
            }),
        );
        assert_eq!(
            app.models().get_copied(&underlay_activated),
            Some(true),
            "underlay should activate once the barrier is removed"
        );
    }

    #[test]
    fn modal_popover_close_transition_restores_trigger_focus_while_barrier_blocks_underlay_pointer()
    {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let open = app.models_mut().insert(false);
        let underlay_activated = app.models_mut().insert(false);

        let trigger_id_out: Rc<Cell<Option<fret_ui::elements::GlobalElementId>>> =
            Rc::new(Cell::new(None));
        let focusable_id_out: Rc<Cell<Option<fret_ui::elements::GlobalElementId>>> =
            Rc::new(Cell::new(None));

        let mut services = FakeServices;
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            CoreSize::new(Px(800.0), Px(600.0)),
        );

        let render_frame = |ui: &mut UiTree<App>, app: &mut App, services: &mut FakeServices| {
            let _ = render_overlay_frame(
                ui,
                app,
                services,
                window,
                bounds,
                "modal-popover-close-transition-focus-restore",
                |cx| {
                    let underlay_activated = underlay_activated.clone();
                    let underlay = cx.pressable(
                        PressableProps {
                            layout: {
                                let mut layout = LayoutStyle::default();
                                layout.size.width = Length::Fill;
                                layout.size.height = Length::Fill;
                                layout
                            },
                            enabled: true,
                            focusable: true,
                            ..Default::default()
                        },
                        move |cx, _st| {
                            cx.pressable_set_bool(&underlay_activated, true);
                            Vec::new()
                        },
                    );

                    let trigger_id_out = trigger_id_out.clone();
                    let trigger = cx.pressable_with_id(
                        PressableProps {
                            layout: {
                                let mut layout = LayoutStyle::default();
                                layout.size.width = Length::Px(Px(120.0));
                                layout.size.height = Length::Px(Px(40.0));
                                layout.inset.left = Some(Px(20.0));
                                layout.inset.top = Some(Px(20.0));
                                layout.position = fret_ui::element::PositionStyle::Absolute;
                                layout
                            },
                            enabled: true,
                            focusable: true,
                            a11y: fret_ui::element::PressableA11y {
                                test_id: Some(Arc::from("trigger")),
                                ..Default::default()
                            },
                            ..Default::default()
                        },
                        |cx, _st, id| {
                            trigger_id_out.set(Some(id));
                            cx.pressable_toggle_bool(&open);
                            Vec::new()
                        },
                    );

                    let focusable_id_out = focusable_id_out.clone();
                    let popover = Popover::new(open.clone())
                        .modal(true)
                        .auto_focus(false)
                        .into_element(
                            cx,
                            |_cx| trigger,
                            move |cx| {
                                let focusable = cx.pressable_with_id(
                                    PressableProps {
                                        layout: {
                                            let mut layout = LayoutStyle::default();
                                            layout.size.width = Length::Px(Px(200.0));
                                            layout.size.height = Length::Px(Px(44.0));
                                            layout
                                        },
                                        enabled: true,
                                        focusable: true,
                                        a11y: fret_ui::element::PressableA11y {
                                            test_id: Some(Arc::from("popover-focusable")),
                                            ..Default::default()
                                        },
                                        ..Default::default()
                                    },
                                    move |_cx, _st, id| {
                                        focusable_id_out.set(Some(id));
                                        Vec::new()
                                    },
                                );
                                PopoverContent::new(vec![focusable]).into_element(cx)
                            },
                        );

                    vec![underlay, popover]
                },
            );
        };

        // Frame 1: closed.
        render_frame(&mut ui, &mut app, &mut services);

        let trigger_id = trigger_id_out.get().expect("trigger id");
        let trigger_node =
            fret_ui::elements::node_for_element(&mut app, window, trigger_id).expect("trigger");
        ui.set_focus(Some(trigger_node));

        let _ = app.models_mut().update(&open, |v| *v = true);

        // Frame 2: open -> barrier root should exist.
        render_frame(&mut ui, &mut app, &mut services);
        let snap = ui.semantics_snapshot().expect("semantics snapshot");
        assert!(
            snap.barrier_root.is_some(),
            "expected modal popover to install a modal barrier root"
        );

        let focusable_id = focusable_id_out.get().expect("popover focusable id");
        let focusable_node = fret_ui::elements::node_for_element(&mut app, window, focusable_id)
            .expect("popover focusable");
        ui.set_focus(Some(focusable_node));

        let _ = app.models_mut().update(&open, |v| *v = false);

        // Frame 3: closing (present=true, interactive=false) -> focus should be restored even
        // though pointer interactions remain blocked by the barrier.
        render_frame(&mut ui, &mut app, &mut services);

        let trigger_id = trigger_id_out.get().expect("trigger id");
        let trigger_node =
            fret_ui::elements::node_for_element(&mut app, window, trigger_id).expect("trigger");
        assert_eq!(
            ui.focus(),
            Some(trigger_node),
            "expected close transition to restore focus to the trigger"
        );

        let click = Point::new(Px(10.0), Px(10.0));
        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::Pointer(fret_core::PointerEvent::Down {
                pointer_id: fret_core::PointerId(0),
                position: click,
                button: MouseButton::Left,
                modifiers: Modifiers::default(),
                pointer_type: fret_core::PointerType::Mouse,
                click_count: 1,
            }),
        );
        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::Pointer(fret_core::PointerEvent::Up {
                pointer_id: fret_core::PointerId(0),
                position: click,
                button: MouseButton::Left,
                modifiers: Modifiers::default(),
                is_click: true,
                pointer_type: fret_core::PointerType::Mouse,
                click_count: 1,
            }),
        );
        assert_eq!(
            app.models().get_copied(&underlay_activated),
            Some(false),
            "underlay should remain inert while the modal popover is closing"
        );
    }

    #[test]
    fn modal_popover_close_transition_on_close_auto_focus_can_prevent_default_and_restore_focus() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let open = app.models_mut().insert(false);
        let underlay_activated = app.models_mut().insert(false);

        let trigger_id_cell: Arc<std::sync::Mutex<Option<fret_ui::elements::GlobalElementId>>> =
            Arc::new(std::sync::Mutex::new(None));
        let trigger_id_for_handler = trigger_id_cell.clone();

        let calls = Arc::new(AtomicUsize::new(0));
        let calls_for_handler = calls.clone();
        let handler: OnCloseAutoFocus = Arc::new(move |host, _action_cx, req| {
            calls_for_handler.fetch_add(1, Ordering::SeqCst);
            let trigger = trigger_id_for_handler
                .lock()
                .unwrap_or_else(|e| e.into_inner())
                .clone();
            if let Some(trigger) = trigger {
                host.request_focus(trigger);
            }
            req.prevent_default();
        });

        let trigger_id_out: Rc<Cell<Option<fret_ui::elements::GlobalElementId>>> =
            Rc::new(Cell::new(None));
        let focusable_id_out: Rc<Cell<Option<fret_ui::elements::GlobalElementId>>> =
            Rc::new(Cell::new(None));

        let mut services = FakeServices;
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            CoreSize::new(Px(800.0), Px(600.0)),
        );

        let render_frame = |ui: &mut UiTree<App>, app: &mut App, services: &mut FakeServices| {
            let _ = render_overlay_frame(
                ui,
                app,
                services,
                window,
                bounds,
                "modal-popover-close-transition-on-close-auto-focus",
                |cx| {
                    let underlay_activated = underlay_activated.clone();
                    let underlay = cx.pressable(
                        PressableProps {
                            layout: {
                                let mut layout = LayoutStyle::default();
                                layout.size.width = Length::Fill;
                                layout.size.height = Length::Fill;
                                layout
                            },
                            enabled: true,
                            focusable: true,
                            ..Default::default()
                        },
                        move |cx, _st| {
                            cx.pressable_set_bool(&underlay_activated, true);
                            Vec::new()
                        },
                    );

                    let trigger_id_out = trigger_id_out.clone();
                    let trigger_id_cell = trigger_id_cell.clone();
                    let open_for_trigger = open.clone();
                    let trigger = cx.pressable_with_id(
                        PressableProps {
                            layout: {
                                let mut layout = LayoutStyle::default();
                                layout.size.width = Length::Px(Px(120.0));
                                layout.size.height = Length::Px(Px(40.0));
                                layout.inset.left = Some(Px(20.0));
                                layout.inset.top = Some(Px(20.0));
                                layout.position = fret_ui::element::PositionStyle::Absolute;
                                layout
                            },
                            enabled: true,
                            focusable: true,
                            a11y: fret_ui::element::PressableA11y {
                                test_id: Some(Arc::from("trigger")),
                                ..Default::default()
                            },
                            ..Default::default()
                        },
                        move |cx, _st, id| {
                            trigger_id_out.set(Some(id));
                            *trigger_id_cell.lock().unwrap_or_else(|e| e.into_inner()) = Some(id);
                            cx.pressable_toggle_bool(&open_for_trigger);
                            Vec::new()
                        },
                    );

                    let focusable_id_out = focusable_id_out.clone();
                    let handler = handler.clone();
                    let popover = Popover::new(open.clone())
                        .modal(true)
                        .auto_focus(false)
                        .consume_outside_pointer_events(true)
                        .on_close_auto_focus(Some(handler))
                        .into_element(
                            cx,
                            |_cx| trigger,
                            move |cx| {
                                let focusable = cx.pressable_with_id(
                                    PressableProps {
                                        layout: {
                                            let mut layout = LayoutStyle::default();
                                            layout.size.width = Length::Px(Px(200.0));
                                            layout.size.height = Length::Px(Px(44.0));
                                            layout
                                        },
                                        enabled: true,
                                        focusable: true,
                                        a11y: fret_ui::element::PressableA11y {
                                            test_id: Some(Arc::from("popover-focusable")),
                                            ..Default::default()
                                        },
                                        ..Default::default()
                                    },
                                    move |_cx, _st, id| {
                                        focusable_id_out.set(Some(id));
                                        Vec::new()
                                    },
                                );
                                PopoverContent::new(vec![focusable]).into_element(cx)
                            },
                        );

                    vec![underlay, popover]
                },
            );
        };

        // Frame 1: closed.
        render_frame(&mut ui, &mut app, &mut services);

        let trigger_id = trigger_id_out.get().expect("trigger id");
        let trigger_node =
            fret_ui::elements::node_for_element(&mut app, window, trigger_id).expect("trigger");
        ui.set_focus(Some(trigger_node));

        let _ = app.models_mut().update(&open, |v| *v = true);

        // Frame 2: open -> barrier root should exist.
        render_frame(&mut ui, &mut app, &mut services);
        let snap = ui.semantics_snapshot().expect("semantics snapshot");
        assert!(
            snap.barrier_root.is_some(),
            "expected modal popover to install a modal barrier root"
        );

        let focusable_id = focusable_id_out.get().expect("focusable id");
        let focusable_node = fret_ui::elements::node_for_element(&mut app, window, focusable_id)
            .expect("focusable node");
        ui.set_focus(Some(focusable_node));

        let _ = app.models_mut().update(&open, |v| *v = false);

        // Frame 3: closing -> handler should be able to restore focus while barrier blocks pointer.
        render_frame(&mut ui, &mut app, &mut services);

        assert!(
            calls.load(Ordering::SeqCst) > 0,
            "expected on_close_auto_focus to run"
        );

        let trigger_id = trigger_id_out.get().expect("trigger id");
        let trigger_node =
            fret_ui::elements::node_for_element(&mut app, window, trigger_id).expect("trigger");
        assert_eq!(
            ui.focus(),
            Some(trigger_node),
            "expected on_close_auto_focus to restore focus to the trigger"
        );

        let click = Point::new(Px(10.0), Px(10.0));
        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::Pointer(fret_core::PointerEvent::Down {
                pointer_id: fret_core::PointerId(0),
                position: click,
                button: MouseButton::Left,
                modifiers: Modifiers::default(),
                pointer_type: fret_core::PointerType::Mouse,
                click_count: 1,
            }),
        );
        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::Pointer(fret_core::PointerEvent::Up {
                pointer_id: fret_core::PointerId(0),
                position: click,
                button: MouseButton::Left,
                modifiers: Modifiers::default(),
                is_click: true,
                pointer_type: fret_core::PointerType::Mouse,
                click_count: 1,
            }),
        );
        assert_eq!(
            app.models().get_copied(&underlay_activated),
            Some(false),
            "underlay should remain inert while the modal popover is closing"
        );
    }

    #[test]
    fn popover_close_transition_is_click_through_and_observer_inert() {
        use fret_core::{Event, MouseButtons};

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

        let mut services = FakeServices;
        let frame_bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            CoreSize::new(Px(800.0), Px(600.0)),
        );

        fn render_frame(
            ui: &mut UiTree<App>,
            app: &mut App,
            services: &mut dyn fret_core::UiServices,
            window: AppWindowId,
            frame_bounds: Rect,
            open: Model<bool>,
            underlay_clicked: Model<bool>,
            underlay_id_out: Rc<Cell<Option<fret_ui::elements::GlobalElementId>>>,
            content_id_out: Rc<Cell<Option<fret_ui::elements::GlobalElementId>>>,
            frame: u64,
        ) {
            app.set_frame_id(FrameId(frame));

            OverlayController::begin_frame(app, window);
            let root = fret_ui::declarative::render_root(
                ui,
                app,
                services,
                window,
                frame_bounds,
                "popover-close-transition-invariants",
                |cx| {
                    let underlay_id_out = underlay_id_out.clone();
                    let content_id_out = content_id_out.clone();
                    let underlay_clicked = underlay_clicked.clone();
                    let open = open.clone();

                    vec![cx.container(
                        ContainerProps {
                            layout: {
                                let mut layout = LayoutStyle::default();
                                layout.size.width = Length::Fill;
                                layout.size.height = Length::Fill;
                                layout
                            },
                            ..Default::default()
                        },
                        move |cx| {
                            let underlay =
                                cx.pressable_with_id(
                                    PressableProps {
                                        layout: {
                                            let mut layout = LayoutStyle::default();
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
                                            vec![cx.container(ContainerProps::default(), |_cx| {
                                                Vec::new()
                                            })]
                                        }
                                    },
                                );

                            let trigger = cx.pressable(
                                PressableProps {
                                    layout: {
                                        let mut layout = LayoutStyle::default();
                                        layout.size.width = Length::Px(Px(120.0));
                                        layout.size.height = Length::Px(Px(40.0));
                                        layout.inset.left = Some(Px(20.0));
                                        layout.inset.top = Some(Px(20.0));
                                        layout.position = fret_ui::element::PositionStyle::Absolute;
                                        layout
                                    },
                                    enabled: true,
                                    focusable: true,
                                    ..Default::default()
                                },
                                {
                                    let open = open.clone();
                                    move |cx, _st| {
                                        cx.pressable_toggle_bool(&open);
                                        Vec::new()
                                    }
                                },
                            );

                            let popover =
                                Popover::new(open.clone()).auto_focus(false).into_element(
                                    cx,
                                    |_cx| trigger,
                                    move |cx| {
                                        let content = PopoverContent::new(vec![
                                            ui::raw_text(cx, "content").into_element(cx),
                                        ])
                                        .into_element(cx);
                                        content_id_out.set(Some(content.id));
                                        content
                                    },
                                );

                            vec![underlay, popover]
                        },
                    )]
                },
            );
            ui.set_root(root);
            OverlayController::render(ui, app, services, window, frame_bounds);
            ui.request_semantics_snapshot();
            ui.layout_all(app, services, frame_bounds, 1.0);
        }

        // Frame 1: mount closed.
        render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            frame_bounds,
            open.clone(),
            underlay_clicked.clone(),
            underlay_id.clone(),
            content_id.clone(),
            1,
        );

        // Frame 2: open.
        let _ = app.models_mut().update(&open, |v| *v = true);
        render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            frame_bounds,
            open.clone(),
            underlay_clicked.clone(),
            underlay_id.clone(),
            content_id.clone(),
            2,
        );
        let content_element = content_id.get().expect("content element id");
        let content_node =
            fret_ui::elements::node_for_element(&mut app, window, content_element).expect("node");
        let snap = ui.semantics_snapshot().expect("semantics snapshot");
        let content_bounds = snap
            .nodes
            .iter()
            .find(|n| n.id == content_node)
            .map(|n| n.bounds)
            .expect("content bounds");
        let content_center = Point::new(
            Px(content_bounds.origin.x.0 + content_bounds.size.width.0 * 0.5),
            Px(content_bounds.origin.y.0 + content_bounds.size.height.0 * 0.5),
        );

        // Frame 3: close transition begins (present=true, interactive=false).
        let _ = app.models_mut().update(&open, |v| *v = false);
        render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            frame_bounds,
            open.clone(),
            underlay_clicked.clone(),
            underlay_id.clone(),
            content_id.clone(),
            3,
        );

        assert!(
            fret_ui::elements::node_for_element(&mut app, window, content_element).is_some(),
            "expected popover content to remain mounted during close transition"
        );

        let content_node =
            fret_ui::elements::node_for_element(&mut app, window, content_element).expect("node");
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

        // Even if pointer events hit where the popover is painted, they must go through while
        // closing.
        ui.dispatch_event(
            &mut app,
            &mut services,
            &Event::Pointer(fret_core::PointerEvent::Down {
                pointer_id: fret_core::PointerId(0),
                position: content_center,
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
                position: content_center,
                button: MouseButton::Left,
                modifiers: Modifiers::default(),
                is_click: true,
                pointer_type: fret_core::PointerType::Mouse,
                click_count: 1,
            }),
        );
        assert_eq!(
            app.models().get_copied(&underlay_clicked),
            Some(true),
            "expected close-transition clicks to reach the underlay"
        );

        // Pointer move should not install timers while closing (no hover policies running).
        ui.dispatch_event(
            &mut app,
            &mut services,
            &Event::Pointer(fret_core::PointerEvent::Move {
                pointer_id: fret_core::PointerId(0),
                position: Point::new(Px(10.0), Px(10.0)),
                buttons: MouseButtons::default(),
                modifiers: Modifiers::default(),
                pointer_type: fret_core::PointerType::Mouse,
            }),
        );
        let effects = app.flush_effects();
        assert!(
            !effects.iter().any(|e| matches!(e, Effect::SetTimer { .. })),
            "expected close transition to not arm timers; effects={effects:?}"
        );
    }

    #[test]
    fn popover_trigger_auto_toggle_opens_by_default() {
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

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            CoreSize::new(Px(800.0), Px(600.0)),
        );

        app.set_frame_id(FrameId(1));
        OverlayController::begin_frame(&mut app, window);
        let root = fret_ui::declarative::render_root(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "popover-auto-toggle-default",
            |cx| {
                let trigger_id = trigger_id.clone();
                let trigger = PopoverTrigger::new(cx.pressable_with_id(
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
                ))
                .into_element(cx);

                let popover = Popover::new(open.clone()).into_element(
                    cx,
                    |_cx| trigger,
                    |cx| {
                        PopoverContent::new(vec![
                            cx.container(ContainerProps::default(), |_cx| Vec::new()),
                        ])
                        .into_element(cx)
                    },
                );
                vec![popover]
            },
        );
        ui.set_root(root);
        OverlayController::render(&mut ui, &mut app, &mut services, window, bounds);
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        let trigger_element = trigger_id.get().expect("trigger element id");
        let trigger_node = fret_ui::elements::node_for_element(&mut app, window, trigger_element)
            .expect("trigger node");
        let trigger_bounds = ui.debug_node_bounds(trigger_node).expect("trigger bounds");

        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::Pointer(fret_core::PointerEvent::Down {
                pointer_id: fret_core::PointerId(0),
                position: center(trigger_bounds),
                button: MouseButton::Left,
                modifiers: Modifiers::default(),
                pointer_type: fret_core::PointerType::Mouse,
                click_count: 1,
            }),
        );
        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::Pointer(fret_core::PointerEvent::Up {
                pointer_id: fret_core::PointerId(0),
                position: center(trigger_bounds),
                button: MouseButton::Left,
                modifiers: Modifiers::default(),
                is_click: true,
                pointer_type: fret_core::PointerType::Mouse,
                click_count: 1,
            }),
        );

        assert_eq!(app.models().get_copied(&open), Some(true));
    }

    #[test]
    fn popover_trigger_auto_toggle_false_keeps_model_unchanged() {
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
        let trigger_activated = app.models_mut().insert(false);
        let trigger_id: Rc<Cell<Option<fret_ui::elements::GlobalElementId>>> =
            Rc::new(Cell::new(None));

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            CoreSize::new(Px(800.0), Px(600.0)),
        );

        app.set_frame_id(FrameId(1));
        OverlayController::begin_frame(&mut app, window);
        let root = fret_ui::declarative::render_root(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "popover-auto-toggle-off",
            |cx| {
                let trigger_id = trigger_id.clone();
                let trigger_activated = trigger_activated.clone();
                let trigger = PopoverTrigger::new(cx.pressable_with_id(
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
                        cx.pressable_set_bool(&trigger_activated, true);
                        vec![cx.container(ContainerProps::default(), |_cx| Vec::new())]
                    },
                ))
                .auto_toggle(false)
                .into_element(cx);

                let popover = Popover::new(open.clone()).into_element(
                    cx,
                    |_cx| trigger,
                    |cx| {
                        PopoverContent::new(vec![
                            cx.container(ContainerProps::default(), |_cx| Vec::new()),
                        ])
                        .into_element(cx)
                    },
                );
                vec![popover]
            },
        );
        ui.set_root(root);
        OverlayController::render(&mut ui, &mut app, &mut services, window, bounds);
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        let trigger_element = trigger_id.get().expect("trigger element id");
        let trigger_node = fret_ui::elements::node_for_element(&mut app, window, trigger_element)
            .expect("trigger node");
        let trigger_bounds = ui.debug_node_bounds(trigger_node).expect("trigger bounds");

        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::Pointer(fret_core::PointerEvent::Down {
                pointer_id: fret_core::PointerId(0),
                position: center(trigger_bounds),
                button: MouseButton::Left,
                modifiers: Modifiers::default(),
                pointer_type: fret_core::PointerType::Mouse,
                click_count: 1,
            }),
        );
        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::Pointer(fret_core::PointerEvent::Up {
                pointer_id: fret_core::PointerId(0),
                position: center(trigger_bounds),
                button: MouseButton::Left,
                modifiers: Modifiers::default(),
                is_click: true,
                pointer_type: fret_core::PointerType::Mouse,
                click_count: 1,
            }),
        );

        assert_eq!(app.models().get_copied(&open), Some(false));
        assert_eq!(app.models().get_copied(&trigger_activated), Some(true));
    }

    #[test]
    fn popover_open_on_hover_toggles_open_without_click() {
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

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            CoreSize::new(Px(800.0), Px(600.0)),
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
                    "popover-open-on-hover",
                    |cx| {
                        let trigger_id = trigger_id.clone();
                        let trigger = PopoverTrigger::new(cx.pressable_with_id(
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
                        ))
                        .auto_toggle(false)
                        .into_element(cx);

                        let popover = Popover::new(open.clone())
                            .open_on_hover(true)
                            .hover_open_delay_frames(0)
                            .hover_close_delay_frames(0)
                            .into_element(
                                cx,
                                |_cx| trigger,
                                |cx| {
                                    PopoverContent::new(vec![
                                        cx.container(ContainerProps::default(), |_cx| Vec::new()),
                                    ])
                                    .into_element(cx)
                                },
                            );
                        vec![popover]
                    },
                );
                ui.set_root(root);
                OverlayController::render(ui, app, services, window, bounds);
                ui.layout_all(app, services, bounds, 1.0);
            };

        render_frame(&mut ui, &mut app, &mut services, 1);

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
                modifiers: Modifiers::default(),
                pointer_type: fret_core::PointerType::Mouse,
            }),
        );

        render_frame(&mut ui, &mut app, &mut services, 2);
        assert_eq!(app.models().get_copied(&open), Some(true));

        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::Pointer(fret_core::PointerEvent::Move {
                pointer_id: fret_core::PointerId(0),
                position: Point::new(Px(760.0), Px(560.0)),
                buttons: fret_core::MouseButtons::default(),
                modifiers: Modifiers::default(),
                pointer_type: fret_core::PointerType::Mouse,
            }),
        );

        render_frame(&mut ui, &mut app, &mut services, 3);
        assert_eq!(app.models().get_copied(&open), Some(false));
    }

    #[test]
    fn popover_open_on_hover_does_not_move_focus_into_content_by_default() {
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
        let content_focusable_id: Rc<Cell<Option<fret_ui::elements::GlobalElementId>>> =
            Rc::new(Cell::new(None));

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            CoreSize::new(Px(800.0), Px(600.0)),
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
                    "popover-open-on-hover-focus-behavior",
                    |cx| {
                        let trigger_id = trigger_id.clone();
                        let content_focusable_id = content_focusable_id.clone();
                        let trigger = PopoverTrigger::new(cx.pressable_with_id(
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
                        ))
                        .auto_toggle(false)
                        .into_element(cx);

                        let popover = Popover::new(open.clone())
                            .open_on_hover(true)
                            .hover_open_delay_frames(0)
                            .hover_close_delay_frames(0)
                            .into_element(
                                cx,
                                |_cx| trigger,
                                |cx| {
                                    let focusable = cx.pressable_with_id(
                                        PressableProps {
                                            enabled: true,
                                            focusable: true,
                                            ..Default::default()
                                        },
                                        move |cx, _st, id| {
                                            content_focusable_id.set(Some(id));
                                            vec![cx.container(ContainerProps::default(), |_cx| {
                                                Vec::new()
                                            })]
                                        },
                                    );
                                    PopoverContent::new(vec![focusable]).into_element(cx)
                                },
                            );
                        vec![popover]
                    },
                );
                ui.set_root(root);
                OverlayController::render(ui, app, services, window, bounds);
                ui.layout_all(app, services, bounds, 1.0);
            };

        render_frame(&mut ui, &mut app, &mut services, 1);

        let trigger_element = trigger_id.get().expect("trigger element id");
        let trigger_node = fret_ui::elements::node_for_element(&mut app, window, trigger_element)
            .expect("trigger node");
        let trigger_bounds = ui.debug_node_bounds(trigger_node).expect("trigger bounds");

        ui.set_focus(Some(trigger_node));

        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::Pointer(fret_core::PointerEvent::Move {
                pointer_id: fret_core::PointerId(0),
                position: center(trigger_bounds),
                buttons: fret_core::MouseButtons::default(),
                modifiers: Modifiers::default(),
                pointer_type: fret_core::PointerType::Mouse,
            }),
        );

        render_frame(&mut ui, &mut app, &mut services, 2);
        assert_eq!(app.models().get_copied(&open), Some(true));

        let content_focusable_element = content_focusable_id.get().expect("content focusable id");
        let content_focusable_node =
            fret_ui::elements::node_for_element(&mut app, window, content_focusable_element)
                .expect("content focusable node");

        assert_eq!(
            ui.focus(),
            Some(trigger_node),
            "hover-open should not move focus into popover content by default"
        );
        assert_ne!(
            ui.focus(),
            Some(content_focusable_node),
            "hover-open should keep content unfocused unless auto_focus is explicitly enabled"
        );
    }

    #[test]
    fn popover_open_on_hover_keeps_open_while_pointer_moves_through_safe_corridor() {
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
            CoreSize::new(Px(800.0), Px(600.0)),
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
                    "popover-open-on-hover-safe-corridor",
                    |cx| {
                        let trigger_id = trigger_id.clone();
                        let content_probe_id = content_probe_id.clone();
                        let trigger = PopoverTrigger::new(cx.pressable_with_id(
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
                        ))
                        .auto_toggle(false)
                        .into_element(cx);

                        let popover = Popover::new(open.clone())
                            .open_on_hover(true)
                            .hover_open_delay_frames(0)
                            .hover_close_delay_frames(0)
                            .into_element(
                                cx,
                                |_cx| trigger,
                                |cx| {
                                    let probe = cx.pressable_with_id(
                                        PressableProps {
                                            layout: {
                                                let mut layout = LayoutStyle::default();
                                                layout.size.width = Length::Px(Px(120.0));
                                                layout.size.height = Length::Px(Px(40.0));
                                                layout
                                            },
                                            enabled: true,
                                            focusable: false,
                                            ..Default::default()
                                        },
                                        move |cx, _st, id| {
                                            content_probe_id.set(Some(id));
                                            vec![cx.container(ContainerProps::default(), |_cx| {
                                                Vec::new()
                                            })]
                                        },
                                    );
                                    PopoverContent::new(vec![probe]).into_element(cx)
                                },
                            );
                        vec![popover]
                    },
                );
                ui.set_root(root);
                OverlayController::render(ui, app, services, window, bounds);
                ui.layout_all(app, services, bounds, 1.0);
            };

        render_frame(&mut ui, &mut app, &mut services, 1);

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
                modifiers: Modifiers::default(),
                pointer_type: fret_core::PointerType::Mouse,
            }),
        );

        render_frame(&mut ui, &mut app, &mut services, 2);
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
            &fret_core::Event::Pointer(fret_core::PointerEvent::Move {
                pointer_id: fret_core::PointerId(0),
                position: transit_point,
                buttons: fret_core::MouseButtons::default(),
                modifiers: Modifiers::default(),
                pointer_type: fret_core::PointerType::Mouse,
            }),
        );

        render_frame(&mut ui, &mut app, &mut services, 3);
        assert_eq!(app.models().get_copied(&open), Some(true));

        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::Pointer(fret_core::PointerEvent::Move {
                pointer_id: fret_core::PointerId(0),
                position: Point::new(Px(760.0), Px(560.0)),
                buttons: fret_core::MouseButtons::default(),
                modifiers: Modifiers::default(),
                pointer_type: fret_core::PointerType::Mouse,
            }),
        );

        render_frame(&mut ui, &mut app, &mut services, 4);
        assert_eq!(app.models().get_copied(&open), Some(false));
    }

    #[test]
    fn popover_supports_detached_trigger_element() {
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
        let detached_trigger_id: Rc<Cell<Option<fret_ui::elements::GlobalElementId>>> =
            Rc::new(Cell::new(None));

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            CoreSize::new(Px(800.0), Px(600.0)),
        );

        app.set_frame_id(FrameId(1));
        OverlayController::begin_frame(&mut app, window);
        let root = fret_ui::declarative::render_root(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "popover-detached-trigger",
            |cx| {
                let detached_trigger_id_out = detached_trigger_id.clone();
                let detached_trigger = PopoverTrigger::new(cx.pressable_with_id(
                    PressableProps {
                        layout: {
                            let mut layout = LayoutStyle::default();
                            layout.size.width = Length::Px(Px(120.0));
                            layout.size.height = Length::Px(Px(40.0));
                            layout.inset.left = Some(Px(40.0));
                            layout.inset.top = Some(Px(40.0));
                            layout.position = fret_ui::element::PositionStyle::Absolute;
                            layout
                        },
                        enabled: true,
                        focusable: true,
                        ..Default::default()
                    },
                    move |cx, _st, id| {
                        detached_trigger_id_out.set(Some(id));
                        vec![cx.container(ContainerProps::default(), |_cx| Vec::new())]
                    },
                ))
                .into_element(cx);

                let detached_id = detached_trigger.id;
                let popover = Popover::new(open.clone())
                    .trigger_element(detached_id)
                    .anchor_element(detached_id)
                    .into_element(
                        cx,
                        |cx| {
                            cx.container(
                                ContainerProps {
                                    layout: {
                                        let mut layout = LayoutStyle::default();
                                        layout.size.width = Length::Px(Px(1.0));
                                        layout.size.height = Length::Px(Px(1.0));
                                        layout
                                    },
                                    ..Default::default()
                                },
                                |_cx| Vec::new(),
                            )
                        },
                        |cx| {
                            PopoverContent::new(vec![
                                cx.container(ContainerProps::default(), |_cx| Vec::new()),
                            ])
                            .into_element(cx)
                        },
                    );

                vec![detached_trigger, popover]
            },
        );
        ui.set_root(root);
        OverlayController::render(&mut ui, &mut app, &mut services, window, bounds);
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        let trigger_element = detached_trigger_id.get().expect("detached trigger id");
        let trigger_node = fret_ui::elements::node_for_element(&mut app, window, trigger_element)
            .expect("trigger node");
        let trigger_bounds = ui.debug_node_bounds(trigger_node).expect("trigger bounds");

        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::Pointer(fret_core::PointerEvent::Down {
                pointer_id: fret_core::PointerId(0),
                position: center(trigger_bounds),
                button: MouseButton::Left,
                modifiers: Modifiers::default(),
                pointer_type: fret_core::PointerType::Mouse,
                click_count: 1,
            }),
        );
        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::Pointer(fret_core::PointerEvent::Up {
                pointer_id: fret_core::PointerId(0),
                position: center(trigger_bounds),
                button: MouseButton::Left,
                modifiers: Modifiers::default(),
                is_click: true,
                pointer_type: fret_core::PointerType::Mouse,
                click_count: 1,
            }),
        );

        assert_eq!(app.models().get_copied(&open), Some(true));
    }

    #[test]
    fn popover_modal_trap_focus_traps_tab_but_keeps_outside_click_through() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let open = app.models_mut().insert(false);
        let underlay_activated = app.models_mut().insert(false);
        let underlay_id: Rc<Cell<Option<fret_ui::elements::GlobalElementId>>> =
            Rc::new(Cell::new(None));
        let focusable_a_id: Rc<Cell<Option<fret_ui::elements::GlobalElementId>>> =
            Rc::new(Cell::new(None));
        let focusable_b_id: Rc<Cell<Option<fret_ui::elements::GlobalElementId>>> =
            Rc::new(Cell::new(None));
        let trigger_id: Rc<Cell<Option<fret_ui::elements::GlobalElementId>>> =
            Rc::new(Cell::new(None));

        let mut services = FakeServices;
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            CoreSize::new(Px(800.0), Px(600.0)),
        );

        let render_frame =
            |ui: &mut UiTree<App>, app: &mut App, services: &mut FakeServices, frame: u64| {
                app.set_frame_id(FrameId(frame));
                OverlayController::begin_frame(app, window);

                let underlay_id = underlay_id.clone();
                let focusable_a_id = focusable_a_id.clone();
                let focusable_b_id = focusable_b_id.clone();
                let trigger_id = trigger_id.clone();
                let underlay_activated = underlay_activated.clone();

                let root = fret_ui::declarative::render_root(
                    ui,
                    app,
                    services,
                    window,
                    bounds,
                    "popover-trap-focus",
                    |cx| {
                        let underlay_id = underlay_id.clone();
                        let underlay_activated = underlay_activated.clone();
                        let underlay = cx.pressable_with_id(
                            PressableProps {
                                layout: {
                                    let mut layout = LayoutStyle::default();
                                    layout.size.width = Length::Fill;
                                    layout.size.height = Length::Fill;
                                    layout
                                },
                                enabled: true,
                                focusable: true,
                                ..Default::default()
                            },
                            move |cx, _st, id| {
                                underlay_id.set(Some(id));
                                cx.pressable_set_bool(&underlay_activated, true);
                                vec![cx.container(ContainerProps::default(), |_cx| Vec::new())]
                            },
                        );

                        let trigger_id = trigger_id.clone();
                        let trigger = PopoverTrigger::new(cx.pressable_with_id(
                            PressableProps {
                                layout: {
                                    let mut layout = LayoutStyle::default();
                                    layout.size.width = Length::Px(Px(120.0));
                                    layout.size.height = Length::Px(Px(40.0));
                                    layout.inset.left = Some(Px(20.0));
                                    layout.inset.top = Some(Px(20.0));
                                    layout.position = fret_ui::element::PositionStyle::Absolute;
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
                        ))
                        .into_element(cx);

                        let focusable_a_id = focusable_a_id.clone();
                        let focusable_b_id = focusable_b_id.clone();
                        let popover = Popover::new(open.clone())
                            .modal_trap_focus(true)
                            .auto_focus(true)
                            .into_element(
                                cx,
                                |_cx| trigger,
                                move |cx| {
                                    let a = cx.pressable_with_id(
                                        PressableProps {
                                            layout: {
                                                let mut layout = LayoutStyle::default();
                                                layout.size.width = Length::Px(Px(180.0));
                                                layout.size.height = Length::Px(Px(44.0));
                                                layout
                                            },
                                            enabled: true,
                                            focusable: true,
                                            ..Default::default()
                                        },
                                        move |cx, _st, id| {
                                            focusable_a_id.set(Some(id));
                                            vec![cx.container(ContainerProps::default(), |_cx| {
                                                Vec::new()
                                            })]
                                        },
                                    );

                                    let b = cx.pressable_with_id(
                                        PressableProps {
                                            layout: {
                                                let mut layout = LayoutStyle::default();
                                                layout.size.width = Length::Px(Px(180.0));
                                                layout.size.height = Length::Px(Px(44.0));
                                                layout
                                            },
                                            enabled: true,
                                            focusable: true,
                                            ..Default::default()
                                        },
                                        move |cx, _st, id| {
                                            focusable_b_id.set(Some(id));
                                            vec![cx.container(ContainerProps::default(), |_cx| {
                                                Vec::new()
                                            })]
                                        },
                                    );

                                    PopoverContent::new(vec![a, b]).into_element(cx)
                                },
                            );

                        vec![underlay, popover]
                    },
                );
                ui.set_root(root);
                OverlayController::render(ui, app, services, window, bounds);
                ui.layout_all(app, services, bounds, 1.0);
            };

        render_frame(&mut ui, &mut app, &mut services, 1);

        let trigger_element = trigger_id.get().expect("trigger id");
        let trigger_node = fret_ui::elements::node_for_element(&mut app, window, trigger_element)
            .expect("trigger node");
        let trigger_bounds = ui.debug_node_bounds(trigger_node).expect("trigger bounds");
        let trigger_center = Point::new(
            Px(trigger_bounds.origin.x.0 + trigger_bounds.size.width.0 * 0.5),
            Px(trigger_bounds.origin.y.0 + trigger_bounds.size.height.0 * 0.5),
        );

        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::Pointer(fret_core::PointerEvent::Down {
                pointer_id: fret_core::PointerId(0),
                position: trigger_center,
                button: MouseButton::Left,
                modifiers: Modifiers::default(),
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
                button: MouseButton::Left,
                modifiers: Modifiers::default(),
                is_click: true,
                pointer_type: fret_core::PointerType::Mouse,
                click_count: 1,
            }),
        );

        render_frame(&mut ui, &mut app, &mut services, 2);
        assert_eq!(app.models().get_copied(&open), Some(true));

        let underlay_element = underlay_id.get().expect("underlay id");
        let a_id = focusable_a_id.get().expect("focusable a id");
        let b_id = focusable_b_id.get().expect("focusable b id");
        let underlay_node = fret_ui::elements::node_for_element(&mut app, window, underlay_element)
            .expect("underlay node");
        let a_node = fret_ui::elements::node_for_element(&mut app, window, a_id).expect("a node");
        let b_node = fret_ui::elements::node_for_element(&mut app, window, b_id).expect("b node");
        assert_eq!(ui.focus(), Some(a_node));

        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::KeyDown {
                key: KeyCode::Tab,
                modifiers: Modifiers::default(),
                repeat: false,
            },
        );
        apply_command_effects(&mut ui, &mut app, &mut services);
        assert_ne!(ui.focus(), Some(underlay_node));
        assert!(
            ui.focus() == Some(a_node) || ui.focus() == Some(b_node),
            "trap-focus popover should keep focus inside content"
        );
        assert_ne!(ui.focus(), Some(underlay_node));

        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::Pointer(fret_core::PointerEvent::Down {
                pointer_id: fret_core::PointerId(0),
                position: Point::new(Px(10.0), Px(10.0)),
                button: MouseButton::Left,
                modifiers: Modifiers::default(),
                pointer_type: fret_core::PointerType::Mouse,
                click_count: 1,
            }),
        );
        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::Pointer(fret_core::PointerEvent::Up {
                pointer_id: fret_core::PointerId(0),
                position: Point::new(Px(10.0), Px(10.0)),
                button: MouseButton::Left,
                modifiers: Modifiers::default(),
                is_click: true,
                pointer_type: fret_core::PointerType::Mouse,
                click_count: 1,
            }),
        );

        assert_eq!(app.models().get_copied(&underlay_activated), Some(true));
    }

    #[test]
    fn popover_force_mount_alias_sets_keep_mounted() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let open = app.models_mut().insert(false);
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            CoreSize::new(Px(200.0), Px(120.0)),
        );

        fret_ui::elements::with_element_cx(&mut app, window, bounds, "test", |_cx| {
            let a = Popover::new(open.clone()).keep_mounted(true);
            let b = Popover::new(open.clone()).force_mount(true);
            assert!(a.keep_mounted);
            assert!(b.keep_mounted);
        });
    }

    #[test]
    fn popover_shift_cross_axis_defaults_to_true_and_can_be_disabled() {
        let mut app = App::new();
        let open = app.models_mut().insert(false);
        let a = Popover::new(open.clone());
        let b = Popover::new(open).shift_cross_axis(false);
        assert_eq!(a.shift_cross_axis.unwrap_or(true), true);
        assert_eq!(b.shift_cross_axis.unwrap_or(true), false);
    }

    #[test]
    fn popover_modal_mode_alias_sets_expected_mode() {
        let mut app = App::new();
        let open = app.models_mut().insert(false);

        let non_modal = Popover::new(open.clone()).modal_mode(PopoverModalMode::NonModal);
        assert_eq!(non_modal.modal_mode, PopoverModalMode::NonModal);

        let modal = Popover::new(open.clone()).modal_mode(PopoverModalMode::Modal);
        assert_eq!(modal.modal_mode, PopoverModalMode::Modal);

        let trap_focus = Popover::new(open).modal_mode(PopoverModalMode::TrapFocus);
        assert_eq!(trap_focus.modal_mode, PopoverModalMode::TrapFocus);
    }
}
