//! Select helpers (Radix `@radix-ui/react-select` outcomes).
//!
//! Upstream Select composes:
//! - anchored floating placement (`@radix-ui/react-popper`)
//! - portal rendering (`@radix-ui/react-portal`)
//! - focus management + outside interaction blocking (`@radix-ui/react-focus-scope`, `DismissableLayer`)
//! - aria hiding + scroll lock while open (`aria-hidden`, `react-remove-scroll`)
//! - trigger open keys + typeahead selection while closed.
//!
//! In Fret, the "blocking outside interaction" outcome is typically modeled by installing the
//! select content in a modal overlay layer (barrier-backed) while keeping the content semantics
//! as `ListBox` rather than `Dialog`.
//!
//! This module is intentionally thin: it provides Radix-named entry points for trigger a11y and
//! overlay request wiring without forcing a visual skin.

use std::sync::{Arc, Mutex};
use std::time::Duration;

use fret_core::{AppWindowId, Edges, KeyCode, Modifiers, Point, PointerType, Px, Rect, Size};
use fret_runtime::{Effect, Model, TimerToken};
use fret_ui::action::{
    ActionCx, DismissReason, DismissRequestCx, OnDismissRequest, OnPointerUp, PointerDownCx,
    PointerMoveCx, PointerUpCx, PressablePointerUpResult, UiActionHost, UiPointerActionHost,
};
use fret_ui::element::{
    AnyElement, Elements, LayoutStyle, PointerRegionProps, PressableA11y, PressableProps,
    PressableState,
};
use fret_ui::elements::GlobalElementId;
use fret_ui::overlay_placement::Side;
use fret_ui::{ElementContext, UiHost};

use crate::declarative::ModelWatchExt;
use crate::headless::roving_focus;
pub use crate::headless::select_item_aligned::{
    SELECT_ITEM_ALIGNED_CONTENT_MARGIN, SelectItemAlignedInputs, SelectItemAlignedOutputs,
    select_item_aligned_position,
};
use crate::headless::typeahead;
use crate::overlay;
use crate::primitives::dialog;
use crate::primitives::popper;
use crate::primitives::popper_arrow;
use crate::primitives::trigger_a11y;
use crate::{OverlayController, OverlayPresence, OverlayRequest};

/// Stable per-overlay root naming convention for select overlays.
pub fn select_root_name(id: GlobalElementId) -> String {
    OverlayController::modal_root_name(id)
}

/// Returns a `Model<bool>` that behaves like Radix `useControllableState` for `open`.
///
/// This is a convenience helper for authoring Radix-shaped select roots:
/// - if `controlled_open` is provided, it is used directly
/// - otherwise an internal model is created (once) using `default_open` (Radix `defaultOpen`)
pub fn select_use_open_model<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    controlled_open: Option<Model<bool>>,
    default_open: impl FnOnce() -> bool,
) -> crate::primitives::controllable_state::ControllableModel<bool> {
    crate::primitives::open_state::open_use_model(cx, controlled_open, default_open)
}

/// Returns a `Model<Option<Arc<str>>>` that behaves like Radix `useControllableState` for `value`.
///
/// Radix models Select values as strings. Fret uses `Arc<str>` for stable, cheap-to-clone keys.
pub fn select_use_value_model<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    controlled_value: Option<Model<Option<Arc<str>>>>,
    default_value: impl FnOnce() -> Option<Arc<str>>,
) -> crate::primitives::controllable_state::ControllableModel<Option<Arc<str>>> {
    crate::primitives::controllable_state::use_controllable_model(
        cx,
        controlled_value,
        default_value,
    )
}

/// A Radix-shaped `Select` root configuration surface (open state only).
///
/// Upstream Select owns both `open` and `value` state. Fret's select primitive facade focuses on
/// input and overlay wiring; recipes typically own the selection model. This root helper exists to
/// standardize the controlled/uncontrolled open modeling (`open` / `defaultOpen`).
#[derive(Debug, Clone, Default)]
pub struct SelectRoot {
    open: Option<Model<bool>>,
    default_open: bool,
}

impl SelectRoot {
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the controlled `open` model (`Some`) or selects uncontrolled mode (`None`).
    pub fn open(mut self, open: Option<Model<bool>>) -> Self {
        self.open = open;
        self
    }

    /// Sets the uncontrolled initial open value (Radix `defaultOpen`).
    pub fn default_open(mut self, default_open: bool) -> Self {
        self.default_open = default_open;
        self
    }

    /// Returns a `Model<bool>` that behaves like Radix `useControllableState` for `open`.
    pub fn use_open_model<H: UiHost>(
        &self,
        cx: &mut ElementContext<'_, H>,
    ) -> crate::primitives::controllable_state::ControllableModel<bool> {
        select_use_open_model(cx, self.open.clone(), || self.default_open)
    }

    pub fn open_model<H: UiHost>(&self, cx: &mut ElementContext<'_, H>) -> Model<bool> {
        self.use_open_model(cx).model()
    }

    pub fn is_open<H: UiHost>(&self, cx: &mut ElementContext<'_, H>) -> bool {
        let open_model = self.open_model(cx);
        cx.watch_model(&open_model)
            .layout()
            .copied()
            .unwrap_or(false)
    }

    pub fn modal_request<H: UiHost>(
        &self,
        cx: &mut ElementContext<'_, H>,
        id: GlobalElementId,
        trigger: GlobalElementId,
        presence: OverlayPresence,
        children: Vec<AnyElement>,
    ) -> OverlayRequest {
        modal_select_request(id, trigger, self.open_model(cx), presence, children)
    }
}

/// Stamps Radix-like trigger semantics:
/// - `role=ComboBox`
/// - `expanded` mirrors `aria-expanded`
/// - `controls_element` mirrors `aria-controls` (by element id).
pub fn apply_select_trigger_a11y(
    trigger: AnyElement,
    expanded: bool,
    label: Option<Arc<str>>,
    listbox_element: Option<GlobalElementId>,
) -> AnyElement {
    trigger_a11y::apply_trigger_semantics(
        trigger,
        Some(fret_core::SemanticsRole::ComboBox),
        label,
        Some(expanded),
        listbox_element,
    )
}

/// A11y metadata for a Radix-style select trigger pressable.
pub fn select_trigger_a11y(
    label: Option<Arc<str>>,
    expanded: bool,
    listbox_element: Option<GlobalElementId>,
) -> PressableA11y {
    PressableA11y {
        role: Some(fret_core::SemanticsRole::ComboBox),
        label,
        expanded: Some(expanded),
        controls_element: listbox_element.map(|id| id.0),
        ..Default::default()
    }
}

fn select_listbox_semantics_id_in_scope<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
) -> GlobalElementId {
    select_listbox_pressable_with_id_props::<H>(cx, |_cx, _st, _id| {
        (
            PressableProps {
                layout: LayoutStyle::default(),
                enabled: true,
                focusable: false,
                ..Default::default()
            },
            Vec::new(),
        )
    })
    .id
}

/// Returns the stable semantics element id for a select listbox.
///
/// This mirrors Radix `SelectTrigger` / `SelectContent` behavior where the trigger advertises a
/// `controls` relationship (`aria-controls`) to the listbox content.
///
/// Callers should use this root-name-scoped helper rather than trying to capture the listbox id
/// from the mounted overlay subtree: the trigger needs a stable content id even when the listbox
/// is not mounted yet.
pub fn select_listbox_semantics_id<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    overlay_root_name: &str,
) -> GlobalElementId {
    cx.with_root_name(overlay_root_name, |cx| {
        select_listbox_semantics_id_in_scope::<H>(cx)
    })
}

/// Input-modality-gated initial focus targets for a select-like overlay.
///
/// This mirrors the Radix/shadcn "hand feel" contract:
/// - pointer-open focuses the content container (not the active/selected entry)
/// - keyboard-open focuses the selected/active entry when available
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct SelectInitialFocusTargets {
    pointer_content_focus: Option<GlobalElementId>,
    keyboard_entry_focus: Option<GlobalElementId>,
}

impl SelectInitialFocusTargets {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn pointer_content_focus(mut self, focus: Option<GlobalElementId>) -> Self {
        self.pointer_content_focus = focus;
        self
    }

    pub fn keyboard_entry_focus(mut self, focus: Option<GlobalElementId>) -> Self {
        self.keyboard_entry_focus = focus;
        self
    }

    pub fn resolve<H: UiHost>(
        self,
        cx: &mut ElementContext<'_, H>,
        window: AppWindowId,
    ) -> Option<GlobalElementId> {
        if fret_ui::input_modality::is_keyboard(cx.app, Some(window)) {
            self.keyboard_entry_focus.or(self.pointer_content_focus)
        } else {
            self.pointer_content_focus
        }
    }
}

/// Builds the select listbox element using a stable call path.
///
/// Use this instead of calling `ElementContext::pressable_with_id_props` directly when you need to
/// derive a stable listbox element id (e.g. for trigger `aria-controls` relationships).
pub fn select_listbox_pressable_with_id_props<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    f: impl FnOnce(
        &mut ElementContext<'_, H>,
        PressableState,
        GlobalElementId,
    ) -> (PressableProps, Vec<AnyElement>),
) -> AnyElement {
    cx.pressable_with_id_props(f)
}

/// Radix Select trigger "open keys" (`OPEN_KEYS`).
pub fn is_select_open_key(key: KeyCode) -> bool {
    matches!(
        key,
        KeyCode::Space | KeyCode::Enter | KeyCode::ArrowUp | KeyCode::ArrowDown
    )
}

/// Returns `true` when the open key is expected to also produce a click/activate event on key-up.
pub fn select_open_key_suppresses_activate(key: KeyCode) -> bool {
    matches!(key, KeyCode::Space | KeyCode::Enter)
}

/// Radix uses a 10px movement threshold to distinguish click-vs-drag outcomes after opening.
///
/// We reuse that threshold when emulating touch/pen click-to-open behavior for the trigger.
pub const SELECT_TRIGGER_CLICK_SLOP_PX: f32 = 10.0;

pub type SelectMouseOpenGuard = Arc<Mutex<SelectMouseOpenGuardState>>;

pub fn select_mouse_open_guard() -> SelectMouseOpenGuard {
    Arc::new(Mutex::new(SelectMouseOpenGuardState::default()))
}

pub fn select_mouse_open_guard_clear(guard: &SelectMouseOpenGuard) {
    let mut guard = guard.lock().unwrap_or_else(|e| e.into_inner());
    guard.clear();
}

pub fn select_mouse_open_guard_record_if_opened(
    guard: &SelectMouseOpenGuard,
    was_open: bool,
    now_open: bool,
    down_pos: Point,
) {
    let mut guard = guard.lock().unwrap_or_else(|e| e.into_inner());
    guard.record_if_opened(was_open, now_open, down_pos);
}

/// Returns `true` when a mouse pointer-up should be treated as part of the original trigger click.
///
/// Upstream Radix installs a one-shot "pointer up guard" after opening on mouse `pointerdown` to
/// avoid immediately selecting an item or dismissing when the pointer is released without moving.
pub fn select_mouse_open_is_within_click_slop(down: Point, up: Point) -> bool {
    let dx = (down.x.0 - up.x.0).abs();
    let dy = (down.y.0 - up.y.0).abs();
    dx <= SELECT_TRIGGER_CLICK_SLOP_PX && dy <= SELECT_TRIGGER_CLICK_SLOP_PX
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SelectMouseOpenGuardPointerUpDecision {
    NoGuard,
    Suppress,
    Allow,
}

pub fn select_mouse_open_guard_pointer_up_decision(
    guard: &mut SelectMouseOpenGuardState,
    up: PointerUpCx,
) -> SelectMouseOpenGuardPointerUpDecision {
    if up.button != fret_core::MouseButton::Left {
        return SelectMouseOpenGuardPointerUpDecision::NoGuard;
    }
    if !matches!(up.pointer_type, PointerType::Mouse | PointerType::Unknown) {
        return SelectMouseOpenGuardPointerUpDecision::NoGuard;
    }

    let Some(down) = guard.take() else {
        return SelectMouseOpenGuardPointerUpDecision::NoGuard;
    };

    if select_mouse_open_is_within_click_slop(down, up.position) {
        SelectMouseOpenGuardPointerUpDecision::Suppress
    } else {
        SelectMouseOpenGuardPointerUpDecision::Allow
    }
}

pub fn select_mouse_open_guard_pointer_up_decision_shared(
    guard: &SelectMouseOpenGuard,
    up: PointerUpCx,
) -> SelectMouseOpenGuardPointerUpDecision {
    let mut guard = guard.lock().unwrap_or_else(|e| e.into_inner());
    select_mouse_open_guard_pointer_up_decision(&mut guard, up)
}

/// Returns `true` when a pointer-up event should be treated as the original trigger click release
/// after opening via mouse `pointerdown`.
///
/// Upstream Radix installs a one-shot guard to avoid the `pointerup` immediately selecting an
/// option or dismissing the overlay. We model that guard via `SelectMouseOpenGuardState`.
pub fn select_mouse_open_guard_should_suppress_pointer_up(
    guard: &mut SelectMouseOpenGuardState,
    up: PointerUpCx,
) -> bool {
    select_mouse_open_guard_pointer_up_decision(guard, up)
        == SelectMouseOpenGuardPointerUpDecision::Suppress
}

pub fn select_mouse_open_guard_should_suppress_pointer_up_shared(
    guard: &SelectMouseOpenGuard,
    up: PointerUpCx,
) -> bool {
    select_mouse_open_guard_pointer_up_decision_shared(guard, up)
        == SelectMouseOpenGuardPointerUpDecision::Suppress
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SelectItemAlignedLayout {
    pub outputs: SelectItemAlignedOutputs,
    pub rect: Rect,
    pub side: Side,
}

/// Returns a window-space content rect for Radix Select's `item-aligned` positioning mode.
///
/// Upstream Radix computes CSS `left/right` + `top/bottom` style values. In Fret we convert that
/// output into a concrete `Rect` in window space so renderer backends and non-shadcn recipes can
/// reuse the same contract without reimplementing the mapping logic.
pub fn select_item_aligned_layout(inputs: SelectItemAlignedInputs) -> SelectItemAlignedLayout {
    let outputs = select_item_aligned_position(inputs);

    let margin = SELECT_ITEM_ALIGNED_CONTENT_MARGIN;
    let window_left = inputs.window.origin.x;
    let window_top = inputs.window.origin.y;
    let window_right = Px(window_left.0 + inputs.window.size.width.0);
    let window_bottom = Px(window_top.0 + inputs.window.size.height.0);

    let clamp_y = |y: Px| {
        let min_y = Px(window_top.0 + margin.0);
        let max_y = Px((window_bottom.0 - margin.0 - outputs.height.0).max(min_y.0));
        Px(y.0.clamp(min_y.0, max_y.0))
    };

    let trigger_mid_y = Px(inputs.trigger.origin.y.0 + inputs.trigger.size.height.0 / 2.0);

    let x = if let Some(left) = outputs.left {
        left
    } else if let Some(right) = outputs.right {
        Px(window_right.0 - right.0 - outputs.width.0)
    } else {
        Px(window_left.0 + margin.0)
    };

    let y = if outputs.top.is_some() {
        Px(window_top.0 + margin.0)
    } else if outputs.bottom.is_some() {
        // Radix positions the wrapper relative to the window edges (top/bottom), but the visible
        // listbox content itself is aligned so the selected item's midpoint matches the trigger's
        // midpoint (when it fits without top overflow). Map that outcome directly into window
        // space so the resulting rect matches the web goldens for short lists.
        let selected_item_half_h = Px(inputs.selected_item.size.height.0 / 2.0);
        // Keep the window-space mapping consistent with `select_item_aligned_position`:
        // `selectedItem.offsetTop` is relative to the viewport padding edge (padding-inclusive).
        let selected_item_mid_offset = Px((inputs.selected_item.origin.y.0
            - inputs.viewport.origin.y.0)
            + selected_item_half_h.0);
        let content_top_to_item_mid = Px(inputs.content_border_top.0
            + inputs.content_padding_top.0
            + selected_item_mid_offset.0);
        clamp_y(Px(trigger_mid_y.0 - content_top_to_item_mid.0))
    } else {
        Px(window_top.0 + margin.0)
    };

    let side = if outputs.bottom.is_some() {
        Side::Bottom
    } else {
        Side::Top
    };

    SelectItemAlignedLayout {
        outputs,
        rect: Rect::new(Point::new(x, y), Size::new(outputs.width, outputs.height)),
        side,
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SelectItemAlignedElementInputs {
    pub direction: popper::LayoutDirection,
    pub window: Rect,
    pub trigger: Rect,

    /// Minimum content width (matches CSS `min-width` on the content element).
    pub content_min_width: Px,

    pub content_border_top: Px,
    pub content_padding_top: Px,
    pub content_border_bottom: Px,
    pub content_padding_bottom: Px,

    pub viewport_padding_top: Px,
    pub viewport_padding_bottom: Px,

    pub selected_item_is_first: bool,
    pub selected_item_is_last: bool,

    pub value_node: GlobalElementId,
    pub viewport: GlobalElementId,
    pub listbox: GlobalElementId,
    pub content_panel: GlobalElementId,
    /// Optional probe element that represents the intrinsic content width (e.g. max item label).
    ///
    /// When present, the measured width is used as an additional lower bound for the item-aligned
    /// solver. This mirrors Radix's behavior where the content can grow beyond the trigger width
    /// when items require more space.
    pub content_width_probe: Option<GlobalElementId>,
    pub selected_item: GlobalElementId,
    pub selected_item_text: GlobalElementId,
}

pub fn select_item_aligned_layout_from_elements<H: UiHost>(
    cx: &ElementContext<'_, H>,
    inputs: SelectItemAlignedElementInputs,
) -> Option<SelectItemAlignedLayout> {
    let value_node = overlay::anchor_bounds_for_element(cx, inputs.value_node)?;
    let viewport = overlay::anchor_bounds_for_element(cx, inputs.viewport)?;
    // Item-aligned select positioning uses `offsetTop`-like measurements that must remain stable
    // as the viewport scrolls. Prefer layout bounds for scrolled descendants (they do not include
    // the scroll render transform) so wheel scrolling cannot cause the overlay to "chase" the
    // selected item and drift off-screen.
    let listbox = cx
        .last_bounds_for_element(inputs.listbox)
        .or_else(|| overlay::anchor_bounds_for_element(cx, inputs.listbox))?;
    let mut content = cx
        .last_bounds_for_element(inputs.content_panel)
        .or_else(|| overlay::anchor_bounds_for_element(cx, inputs.content_panel))?;
    content.size.width = Px(content.size.width.0.max(inputs.content_min_width.0));
    let selected_item = cx
        .last_bounds_for_element(inputs.selected_item)
        .or_else(|| overlay::anchor_bounds_for_element(cx, inputs.selected_item))?;
    let selected_item_text = cx
        .last_bounds_for_element(inputs.selected_item_text)
        .or_else(|| overlay::anchor_bounds_for_element(cx, inputs.selected_item_text))?;
    // The headless solver expects `items_height` to match Radix `viewport.scrollHeight`.
    //
    // In our shadcn ports `inputs.listbox` typically points at the element that lays out the full
    // listbox content (including the `SelectViewport` padding such as `p-1`). Because this element
    // grows to fit all rows even when clipped by the scroll viewport, its laid-out height is the
    // closest analogue to `scrollHeight`.
    let items_height = listbox.size.height;

    if let Some(probe_id) = inputs.content_width_probe
        && let Some(probe) = cx.last_bounds_for_element(probe_id)
        && probe.size.width.0.is_finite()
        && probe.size.width.0 > 0.0
    {
        // The solver expects the "content" rect to reflect the last known content panel width.
        // Inflate it with an intrinsic width measurement so the panel can grow to fit long labels.
        //
        // We reuse the vertical border thickness as the horizontal border thickness (shadcn/radix
        // uses a uniform border width).
        let border_extra = Px(inputs.content_border_top.0 * 2.0);
        let probed_width = Px(probe.size.width.0 + border_extra.0);
        content.size.width = Px(content.size.width.0.max(probed_width.0));
    }

    Some(select_item_aligned_layout(SelectItemAlignedInputs {
        direction: inputs.direction,
        window: inputs.window,
        trigger: inputs.trigger,
        content,
        value_node,
        selected_item_text,
        selected_item,
        viewport,
        content_border_top: inputs.content_border_top,
        content_padding_top: inputs.content_padding_top,
        content_border_bottom: inputs.content_border_bottom,
        content_padding_bottom: inputs.content_padding_bottom,
        viewport_padding_top: inputs.viewport_padding_top,
        viewport_padding_bottom: inputs.viewport_padding_bottom,
        selected_item_is_first: inputs.selected_item_is_first,
        selected_item_is_last: inputs.selected_item_is_last,
        items_height,
    }))
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SelectResolvedContentPlacement {
    pub placement: SelectContentPlacement,
    pub item_aligned_layout: Option<SelectItemAlignedLayout>,
}

pub fn select_resolve_content_placement(
    anchor: Rect,
    outer: Rect,
    desired: Size,
    popper_placement: popper::PopperContentPlacement,
    arrow_size: Option<Px>,
    item_aligned_layout: Option<SelectItemAlignedLayout>,
) -> SelectResolvedContentPlacement {
    if let Some(item_aligned_layout) = item_aligned_layout {
        return SelectResolvedContentPlacement {
            placement: select_content_placement_item_aligned(anchor, item_aligned_layout),
            item_aligned_layout: Some(item_aligned_layout),
        };
    }

    SelectResolvedContentPlacement {
        placement: select_content_placement_popper(
            outer,
            anchor,
            desired,
            popper_placement,
            arrow_size,
        ),
        item_aligned_layout: None,
    }
}

pub fn select_resolve_content_placement_from_elements<H: UiHost>(
    cx: &ElementContext<'_, H>,
    anchor: Rect,
    outer: Rect,
    desired: Size,
    popper_placement: popper::PopperContentPlacement,
    arrow_size: Option<Px>,
    item_aligned: Option<SelectItemAlignedElementInputs>,
) -> SelectResolvedContentPlacement {
    let item_aligned_layout =
        item_aligned.and_then(|inputs| select_item_aligned_layout_from_elements(cx, inputs));
    select_resolve_content_placement(
        anchor,
        outer,
        desired,
        popper_placement,
        arrow_size,
        item_aligned_layout,
    )
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SelectContentPlacement {
    pub placed: Rect,
    pub wrapper_insets: Edges,
    pub side: Side,
    pub transform_origin: Point,
    pub popper_layout: Option<fret_ui::overlay_placement::AnchoredPanelLayout>,
}

pub fn select_content_placement_item_aligned(
    anchor: Rect,
    layout: SelectItemAlignedLayout,
) -> SelectContentPlacement {
    let pseudo_layout = fret_ui::overlay_placement::AnchoredPanelLayout {
        rect: layout.rect,
        side: layout.side,
        align: popper::Align::Center,
        arrow: None,
    };

    SelectContentPlacement {
        placed: layout.rect,
        wrapper_insets: Edges::all(Px(0.0)),
        side: layout.side,
        transform_origin: popper::popper_content_transform_origin(&pseudo_layout, anchor, None),
        popper_layout: None,
    }
}

pub fn select_content_placement_popper(
    outer: Rect,
    anchor: Rect,
    desired: Size,
    placement: popper::PopperContentPlacement,
    arrow_size: Option<Px>,
) -> SelectContentPlacement {
    // Radix Select (position="popper") relies on Popper for *placement* but uses CSS max-height
    // (via `--radix-select-content-available-height`) to constrain the listbox. Floating UI does
    // not clamp the floating rect size as part of collision shifting, so the content can overflow
    // the collision boundary when it is taller than the available space.
    //
    // Keep the desired size intact and let collision logic affect only the origin.
    let layout = popper::popper_content_layout_unclamped(outer, anchor, desired, placement);
    let wrapper_insets = popper_arrow::wrapper_insets(&layout, placement.arrow_protrusion);
    let transform_origin = popper::popper_content_transform_origin(&layout, anchor, arrow_size);

    SelectContentPlacement {
        placed: layout.rect,
        wrapper_insets,
        side: layout.side,
        transform_origin,
        popper_layout: Some(layout),
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SelectPopperVars {
    pub available_width: Px,
    pub available_height: Px,
    pub trigger_width: Px,
    pub trigger_height: Px,
}

pub fn select_popper_desired_width(outer: Rect, anchor: Rect, min_width: Px) -> Px {
    popper::popper_desired_width(outer, anchor, min_width)
}

/// Compute Radix-like "select popper vars" (`--radix-select-*`) for recipes.
///
/// Upstream Radix wires these through from `@radix-ui/react-popper`:
/// - `--radix-select-content-available-width`
/// - `--radix-select-content-available-height`
/// - `--radix-select-trigger-width`
/// - `--radix-select-trigger-height`
///
/// In Fret, we compute the same concepts as a structured return value so recipes can size and
/// constrain the listbox without relying on CSS variables.
pub fn select_popper_vars(
    outer: Rect,
    anchor: Rect,
    min_width: Px,
    placement: popper::PopperContentPlacement,
) -> SelectPopperVars {
    let metrics =
        popper::popper_available_metrics_for_placement(outer, anchor, min_width, placement);
    SelectPopperVars {
        available_width: metrics.available_width,
        available_height: metrics.available_height,
        trigger_width: metrics.anchor_width,
        trigger_height: metrics.anchor_height,
    }
}

/// Compute a Radix-like default max height for select popper content.
///
/// Upstream Radix sets `max-height: var(--radix-select-content-available-height)` for shadcn
/// recipes. In Fret, we compute the same concept using `popper_available_metrics(...)` so recipes
/// can size the listbox without relying on CSS variables.
pub fn select_popper_available_height(
    outer: Rect,
    anchor: Rect,
    min_width: Px,
    placement: popper::PopperContentPlacement,
) -> Px {
    select_popper_vars(outer, anchor, min_width, placement).available_height
}

/// Radix-like select typeahead clear timeout (in milliseconds).
///
/// Upstream Radix resets the typeahead search 1 second after it was last updated.
pub const SELECT_TYPEAHEAD_CLEAR_TIMEOUT_MS: u64 = 1000;

/// Timer-driven typeahead query state (Radix-style).
#[derive(Debug, Default)]
pub struct TimedTypeaheadState {
    query: String,
    clear_token: Option<TimerToken>,
}

impl TimedTypeaheadState {
    pub fn query(&self) -> &str {
        self.query.as_str()
    }

    pub fn clear_and_cancel(&mut self, host: &mut dyn UiActionHost) {
        if let Some(token) = self.clear_token.take() {
            host.push_effect(Effect::CancelTimer { token });
        }
        self.query.clear();
    }

    pub fn on_timer(&mut self, token: TimerToken) -> bool {
        if self.clear_token == Some(token) {
            self.clear_token = None;
            self.query.clear();
            return true;
        }
        false
    }

    pub fn push_key_and_arm_timer(
        &mut self,
        host: &mut dyn UiActionHost,
        window: AppWindowId,
        key: KeyCode,
        timeout: Duration,
    ) -> Option<char> {
        let ch = fret_core::keycode_to_ascii_lowercase(key)?;
        self.query.push(ch);
        if let Some(token) = self.clear_token.take() {
            host.push_effect(Effect::CancelTimer { token });
        }
        let token = host.next_timer_token();
        self.clear_token = Some(token);
        host.push_effect(Effect::SetTimer {
            window: Some(window),
            token,
            after: timeout,
            repeat: None,
        });
        Some(ch)
    }
}

/// Closed-state trigger policy for Radix-style select.
///
/// This models two coupled Radix outcomes:
/// - Trigger open keys open the listbox on key-down (and suppress the ensuing key-up activation).
/// - While closed, alphanumeric typeahead updates the selected value without opening.
#[derive(Debug, Default)]
pub struct SelectTriggerKeyState {
    suppress_next_activate: bool,
    typeahead: TimedTypeaheadState,
}

impl SelectTriggerKeyState {
    pub fn take_suppress_next_activate(&mut self) -> bool {
        let v = self.suppress_next_activate;
        self.suppress_next_activate = false;
        v
    }

    pub fn clear_typeahead(&mut self, host: &mut dyn UiActionHost) {
        self.typeahead.clear_and_cancel(host);
    }

    pub fn reset_typeahead_buffer(&mut self) {
        self.typeahead.query.clear();
        self.typeahead.clear_token = None;
    }

    pub fn typeahead_query(&self) -> &str {
        self.typeahead.query()
    }

    pub fn push_typeahead_key_and_arm_timer(
        &mut self,
        host: &mut dyn UiActionHost,
        window: AppWindowId,
        key: KeyCode,
    ) -> Option<char> {
        let timeout = Duration::from_millis(SELECT_TYPEAHEAD_CLEAR_TIMEOUT_MS);
        self.typeahead
            .push_key_and_arm_timer(host, window, key, timeout)
    }

    pub fn on_timer(&mut self, token: TimerToken) -> bool {
        self.typeahead.on_timer(token)
    }

    pub fn handle_key_down_when_closed(
        &mut self,
        host: &mut dyn UiActionHost,
        window: AppWindowId,
        open: &Model<bool>,
        value: &Model<Option<Arc<str>>>,
        values: &[Arc<str>],
        labels: &[Arc<str>],
        disabled: &[bool],
        key: KeyCode,
        modifiers: Modifiers,
        repeat: bool,
    ) -> bool {
        if repeat {
            return false;
        }

        let is_open = host.models_mut().get_copied(open).unwrap_or(false);
        if is_open {
            return false;
        }

        let is_modifier_key = modifiers.ctrl || modifiers.alt || modifiers.meta || modifiers.alt_gr;
        if is_modifier_key {
            return false;
        }

        if key == KeyCode::Space && !self.typeahead.query().is_empty() {
            return true;
        }

        if is_select_open_key(key) {
            if select_open_key_suppresses_activate(key) {
                self.suppress_next_activate = true;
            }
            self.typeahead.clear_and_cancel(host);
            let _ = host.models_mut().update(open, |v| *v = true);
            host.request_redraw(window);
            return true;
        }

        let timeout = Duration::from_millis(SELECT_TYPEAHEAD_CLEAR_TIMEOUT_MS);
        let Some(_ch) = self
            .typeahead
            .push_key_and_arm_timer(host, window, key, timeout)
        else {
            return false;
        };

        let current = host.models_mut().read(value, |v| v.clone()).ok().flatten();
        let current_idx = current
            .as_ref()
            .and_then(|v| values.iter().position(|it| it.as_ref() == v.as_ref()));

        if let Some(next) = typeahead::match_prefix_arc_str(
            labels,
            disabled,
            self.typeahead.query(),
            current_idx,
            true,
        ) && let Some(next_value) = values.get(next).cloned()
        {
            let _ = host.models_mut().update(value, |v| *v = Some(next_value));
            host.request_redraw(window);
        }

        true
    }
}

/// One-shot pointer-up guard used when a select is opened via mouse `pointerdown`.
///
/// This mirrors Radix Select's behavior: the pointer-up that completes the click should not
/// immediately select an item nor dismiss the content.
#[derive(Debug, Default)]
pub struct SelectMouseOpenGuardState {
    mouse_open_down_pos: Option<Point>,
}

impl SelectMouseOpenGuardState {
    pub fn clear(&mut self) {
        self.mouse_open_down_pos = None;
    }

    pub fn record_if_opened(&mut self, was_open: bool, now_open: bool, down_pos: Point) {
        if !was_open && now_open {
            self.mouse_open_down_pos = Some(down_pos);
        } else {
            self.mouse_open_down_pos = None;
        }
    }

    pub fn take(&mut self) -> Option<Point> {
        self.mouse_open_down_pos.take()
    }
}

/// Pointer policy for Radix-style select triggers.
///
/// Upstream Radix opens on `pointerdown` for mouse (and prevents the trigger from stealing focus),
/// while touch/pen devices open on click to avoid scroll-to-open.
#[derive(Debug, Default)]
pub struct SelectTriggerPointerState {
    down_pos: Option<Point>,
    moved: bool,
    captured: bool,
}

impl SelectTriggerPointerState {
    fn reset(&mut self) {
        self.down_pos = None;
        self.moved = false;
        self.captured = false;
    }

    fn moved_beyond_slop(&self, current: Point) -> bool {
        let Some(down) = self.down_pos else {
            return false;
        };
        (down.x.0 - current.x.0).abs() > SELECT_TRIGGER_CLICK_SLOP_PX
            || (down.y.0 - current.y.0).abs() > SELECT_TRIGGER_CLICK_SLOP_PX
    }

    pub fn handle_pointer_down(
        &mut self,
        host: &mut dyn UiPointerActionHost,
        action_cx: ActionCx,
        down: PointerDownCx,
        open: &Model<bool>,
        enabled: bool,
    ) -> bool {
        if !enabled {
            return false;
        }
        if down.button != fret_core::MouseButton::Left {
            return false;
        }

        let is_macos_ctrl_click = cfg!(target_os = "macos")
            && down.modifiers.ctrl
            && down.pointer_type == PointerType::Mouse;
        if is_macos_ctrl_click {
            return false;
        }

        match down.pointer_type {
            PointerType::Mouse | PointerType::Unknown => {
                let _ = host.models_mut().update(open, |v| *v = true);
                host.request_redraw(action_cx.window);
                true
            }
            PointerType::Touch | PointerType::Pen => {
                self.down_pos = Some(down.position);
                self.moved = false;
                self.captured = true;
                host.capture_pointer();
                true
            }
        }
    }

    pub fn handle_pointer_move(
        &mut self,
        _host: &mut dyn UiPointerActionHost,
        _action_cx: ActionCx,
        mv: PointerMoveCx,
    ) -> bool {
        if !self.captured {
            return false;
        }
        if !self.moved && self.moved_beyond_slop(mv.position) {
            self.moved = true;
        }
        true
    }

    pub fn handle_pointer_up(
        &mut self,
        host: &mut dyn UiPointerActionHost,
        action_cx: ActionCx,
        up: PointerUpCx,
        open: &Model<bool>,
        enabled: bool,
    ) -> bool {
        if !enabled {
            self.reset();
            return false;
        }
        if up.button != fret_core::MouseButton::Left {
            self.reset();
            return false;
        }
        if !self.captured {
            self.reset();
            return false;
        }

        host.release_pointer_capture();
        self.captured = false;

        let should_open = !self.moved
            && self.down_pos.is_some()
            && !self.moved_beyond_slop(up.position)
            && host.bounds().contains(up.position);

        self.reset();

        if should_open {
            let _ = host.models_mut().update(open, |v| *v = true);
            host.request_redraw(action_cx.window);
        }
        true
    }
}

/// Open-state listbox policy for Radix-style select content.
///
/// This mirrors Radix outcomes inside `SelectContent`:
/// - `Escape` closes.
/// - `Tab` is suppressed (select should not be navigated using Tab).
/// - `Home/End/ArrowUp/ArrowDown` move the active option (skipping disabled).
/// - `Enter/Space` commits the active option and closes.
/// - Typeahead search moves the active option (with repeated-search normalization).
#[derive(Debug, Default)]
pub struct SelectContentKeyState {
    active_row: Option<usize>,
    typeahead: TimedTypeaheadState,
}

impl SelectContentKeyState {
    pub fn active_row(&self) -> Option<usize> {
        self.active_row
    }

    pub fn set_active_row(&mut self, row: Option<usize>) {
        self.active_row = row;
    }

    pub fn reset_on_open(&mut self, initial_active_row: Option<usize>) {
        self.active_row = initial_active_row;
        self.typeahead.query.clear();
        self.typeahead.clear_token = None;
    }

    pub fn clear_typeahead(&mut self, host: &mut dyn UiActionHost) {
        self.typeahead.clear_and_cancel(host);
    }

    pub fn on_timer(&mut self, token: TimerToken) -> bool {
        self.typeahead.on_timer(token)
    }

    pub fn handle_key_down_when_open(
        &mut self,
        host: &mut dyn UiActionHost,
        window: AppWindowId,
        open: &Model<bool>,
        value: &Model<Option<Arc<str>>>,
        values_by_row: &[Option<Arc<str>>],
        labels_by_row: &[Arc<str>],
        disabled_by_row: &[bool],
        key: KeyCode,
        repeat: bool,
        loop_navigation: bool,
    ) -> bool {
        if repeat {
            return false;
        }

        let is_open = host.models_mut().get_copied(open).unwrap_or(false);
        if !is_open {
            return false;
        }

        if key == KeyCode::Space && !self.typeahead.query().is_empty() {
            return true;
        }

        let current = self
            .active_row
            .or_else(|| roving_focus::first_enabled(disabled_by_row));

        match key {
            KeyCode::Tab => true,
            KeyCode::Escape => {
                let _ = host.models_mut().update(open, |v| *v = false);
                host.request_redraw(window);
                true
            }
            KeyCode::Home => {
                self.active_row = roving_focus::first_enabled(disabled_by_row);
                host.request_redraw(window);
                true
            }
            KeyCode::End => {
                self.active_row = roving_focus::last_enabled(disabled_by_row);
                host.request_redraw(window);
                true
            }
            KeyCode::ArrowDown | KeyCode::ArrowUp => {
                let Some(current) = current else {
                    return true;
                };
                let forward = key == KeyCode::ArrowDown;
                self.active_row =
                    roving_focus::next_enabled(disabled_by_row, current, forward, loop_navigation)
                        .or(Some(current));
                host.request_redraw(window);
                true
            }
            KeyCode::Enter | KeyCode::Space => {
                let Some(active_row) = current else {
                    return true;
                };
                let is_disabled = disabled_by_row.get(active_row).copied().unwrap_or(true);
                if is_disabled {
                    return true;
                }
                if let Some(chosen_value) = values_by_row.get(active_row).cloned().flatten() {
                    let _ = host
                        .models_mut()
                        .update(value, |v| *v = Some(chosen_value.clone()));
                    let _ = host.models_mut().update(open, |v| *v = false);
                    host.request_redraw(window);
                }
                true
            }
            _ => {
                let timeout = Duration::from_millis(SELECT_TYPEAHEAD_CLEAR_TIMEOUT_MS);
                let Some(_ch) = self
                    .typeahead
                    .push_key_and_arm_timer(host, window, key, timeout)
                else {
                    return false;
                };

                let next = typeahead::match_prefix_arc_str(
                    labels_by_row,
                    disabled_by_row,
                    self.typeahead.query(),
                    current,
                    true,
                );
                if next != self.active_row {
                    self.active_row = next;
                    host.request_redraw(window);
                }
                true
            }
        }
    }
}

/// Layout used for a Radix-like select modal barrier element.
///
/// This is a re-export of the shared modal barrier layout from `primitives::dialog`.
pub fn select_modal_barrier_layout() -> LayoutStyle {
    dialog::modal_barrier_layout()
}

/// Builds a full-window modal barrier for Radix-like select overlays.
///
/// This is a thin wrapper over `primitives::dialog::modal_barrier` so non-shadcn users can reuse
/// the same "disable outside pointer events" outcome without depending on dialog primitives.
pub fn select_modal_barrier<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    open: Model<bool>,
    dismiss_on_press: bool,
    children: impl IntoIterator<Item = AnyElement>,
) -> AnyElement {
    select_modal_barrier_with_dismiss_handler(cx, open, dismiss_on_press, None, children)
}

/// Builds a full-window modal barrier for Radix-like select overlays while routing dismissals
/// through an optional dismiss handler.
///
/// When `on_dismiss_request` is provided and `dismiss_on_press` is enabled, barrier presses invoke
/// the handler with `DismissReason::OutsidePress` and do not close `open` automatically.
pub fn select_modal_barrier_with_dismiss_handler<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    open: Model<bool>,
    dismiss_on_press: bool,
    on_dismiss_request: Option<OnDismissRequest>,
    children: impl IntoIterator<Item = AnyElement>,
) -> AnyElement {
    dialog::modal_barrier_with_dismiss_handler(
        cx,
        open,
        dismiss_on_press,
        on_dismiss_request,
        children,
    )
}

/// Convenience helper to assemble select modal overlay children in a Radix-like order: barrier then
/// content.
pub fn select_modal_layer_elements<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    open: Model<bool>,
    dismiss_on_press: bool,
    barrier_children: impl IntoIterator<Item = AnyElement>,
    content: AnyElement,
) -> Elements {
    Elements::from([
        select_modal_barrier(cx, open, dismiss_on_press, barrier_children),
        content,
    ])
}

/// Convenience helper to assemble select modal overlay children in a Radix-like order (barrier then
/// content), while routing barrier presses through an optional dismiss handler.
pub fn select_modal_layer_elements_with_dismiss_handler<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    open: Model<bool>,
    dismiss_on_press: bool,
    on_dismiss_request: Option<OnDismissRequest>,
    barrier_children: impl IntoIterator<Item = AnyElement>,
    content: AnyElement,
) -> Elements {
    Elements::from([
        select_modal_barrier_with_dismiss_handler(
            cx,
            open,
            dismiss_on_press,
            on_dismiss_request,
            barrier_children,
        ),
        content,
    ])
}

/// Builds a pointer region that guards the next mouse `pointerup` after opening on `pointerdown`.
///
/// This element should be installed as a top-most sibling in the overlay (i.e. after the content)
/// so it can swallow the click-release that opened the select, matching Radix's one-shot
/// pointer-up guard even when the release lands over the content.
pub fn select_modal_barrier_pointer_up_guard<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    _open: Model<bool>,
    guard: SelectMouseOpenGuard,
) -> AnyElement {
    let down = guard
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .mouse_open_down_pos;
    let enabled = down.is_some();
    let layout = if let Some(down) = down {
        // Only cover the click-slop rect around the trigger down position. If the pointer-up lands
        // outside this rect, we should not intercept it (drag-to-select / outside-dismiss).
        let slop = SELECT_TRIGGER_CLICK_SLOP_PX;
        let size = Px(slop * 2.0);
        let left = Px((down.x.0 - slop).max(0.0));
        let top = Px((down.y.0 - slop).max(0.0));

        let mut layout = LayoutStyle::default();
        layout.position = fret_ui::element::PositionStyle::Absolute;
        layout.inset = fret_ui::element::InsetStyle {
            left: Some(left),
            right: None,
            top: Some(top),
            bottom: None,
        };
        layout.size.width = fret_ui::element::Length::Px(size);
        layout.size.height = fret_ui::element::Length::Px(size);
        layout
    } else {
        select_modal_barrier_layout()
    };
    cx.pointer_region(PointerRegionProps { layout, enabled }, move |cx| {
        let guard_for_pointer_up = guard.clone();
        cx.pointer_region_on_pointer_up(Arc::new(move |_host, _action_cx, up: PointerUpCx| {
            match select_mouse_open_guard_pointer_up_decision_shared(&guard_for_pointer_up, up) {
                SelectMouseOpenGuardPointerUpDecision::NoGuard => false,
                SelectMouseOpenGuardPointerUpDecision::Suppress => true,
                // Outside the click slop this element should not be hit, but keep the behavior
                // conservative in case the host routing changes.
                SelectMouseOpenGuardPointerUpDecision::Allow => false,
            }
        }));
        Vec::new()
    })
}

/// Convenience helper to assemble select modal overlay children with a pointer-up guard installed
/// inside the barrier (Radix-like behavior when opening on mouse `pointerdown`).
pub fn select_modal_layer_elements_with_pointer_up_guard<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    open: Model<bool>,
    dismiss_on_press: bool,
    guard: SelectMouseOpenGuard,
    barrier_children: impl IntoIterator<Item = AnyElement>,
    content: AnyElement,
) -> Elements {
    let guard_el = select_modal_barrier_pointer_up_guard(cx, open.clone(), guard);
    Elements::from([
        select_modal_barrier(cx, open, dismiss_on_press, barrier_children),
        content,
        guard_el,
    ])
}

/// Convenience helper to assemble select modal overlay children with a pointer-up guard installed
/// inside the barrier (Radix behavior when opening on mouse `pointerdown`), while routing barrier
/// presses through an optional dismiss handler.
pub fn select_modal_layer_elements_with_pointer_up_guard_and_dismiss_handler<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    open: Model<bool>,
    dismiss_on_press: bool,
    on_dismiss_request: Option<OnDismissRequest>,
    guard: SelectMouseOpenGuard,
    barrier_children: impl IntoIterator<Item = AnyElement>,
    content: AnyElement,
) -> Elements {
    let guard_el = select_modal_barrier_pointer_up_guard(cx, open.clone(), guard.clone());
    let barrier_children: Vec<AnyElement> = barrier_children.into_iter().collect();
    let barrier = if dismiss_on_press {
        let open_for_pressable = open.clone();
        let guard_for_pressable = guard;
        let on_dismiss_request_for_pressable = on_dismiss_request.clone();
        cx.pressable(
            PressableProps {
                layout: select_modal_barrier_layout(),
                enabled: true,
                focusable: false,
                ..Default::default()
            },
            move |cx, _st| {
                let open_for_pointer_up = open_for_pressable.clone();
                let guard_for_pointer_up = guard_for_pressable.clone();
                let on_dismiss_request_for_pointer_up = on_dismiss_request_for_pressable.clone();
                cx.pressable_add_on_pointer_up(Arc::new(move |host, action_cx, up| {
                    match select_mouse_open_guard_pointer_up_decision_shared(
                        &guard_for_pointer_up,
                        up,
                    ) {
                        SelectMouseOpenGuardPointerUpDecision::Suppress => {
                            host.request_redraw(action_cx.window);
                            return PressablePointerUpResult::SkipActivate;
                        }
                        SelectMouseOpenGuardPointerUpDecision::Allow
                        | SelectMouseOpenGuardPointerUpDecision::NoGuard => {}
                    }

                    if let Some(on_dismiss_request) = on_dismiss_request_for_pointer_up.as_ref() {
                        let mut req = DismissRequestCx::new(DismissReason::OutsidePress {
                            pointer: Some(fret_ui::action::OutsidePressCx {
                                pointer_id: up.pointer_id,
                                pointer_type: up.pointer_type,
                                button: up.button,
                                modifiers: up.modifiers,
                                click_count: up.click_count,
                            }),
                        });
                        on_dismiss_request(host, action_cx, &mut req);
                        if !req.default_prevented() {
                            let _ = host
                                .models_mut()
                                .update(&open_for_pointer_up, |v| *v = false);
                        }
                    } else {
                        let _ = host
                            .models_mut()
                            .update(&open_for_pointer_up, |v| *v = false);
                    }

                    PressablePointerUpResult::SkipActivate
                }));

                barrier_children
            },
        )
    } else {
        cx.container(
            fret_ui::element::ContainerProps {
                layout: select_modal_barrier_layout(),
                ..Default::default()
            },
            move |_cx| barrier_children,
        )
    };
    Elements::from([barrier, content, guard_el])
}

/// Returns an item-level pointer-up handler that respects the "open via mouse pointerdown" guard.
///
/// When a select is opened on mouse `pointerdown`, the click-release `pointerup` can land on top of
/// the content. Radix installs a one-shot pointer-up guard to ensure that release does not
/// immediately select an item. This helper mirrors that behavior for listbox options.
pub fn select_item_pointer_up_handler(
    open: Model<bool>,
    value: Model<Option<Arc<str>>>,
    item_value: Arc<str>,
    item_disabled: bool,
    mouse_open_guard: SelectMouseOpenGuard,
) -> OnPointerUp {
    Arc::new(move |host, action_cx, up: PointerUpCx| {
        if up.button != fret_core::MouseButton::Left {
            return false;
        }
        if !matches!(up.pointer_type, PointerType::Mouse | PointerType::Unknown) {
            return false;
        }
        if item_disabled {
            return true;
        }
        if select_mouse_open_guard_should_suppress_pointer_up_shared(&mouse_open_guard, up) {
            return true;
        }

        let _ = host
            .models_mut()
            .update(&value, |v| *v = Some(item_value.clone()));
        let _ = host.models_mut().update(&open, |v| *v = false);
        host.request_redraw(action_cx.window);
        true
    })
}

/// Builds an overlay request for a Radix-style select content overlay.
///
/// This uses a modal overlay layer to approximate Radix Select's outside interaction blocking.
pub fn modal_select_request(
    id: GlobalElementId,
    trigger: GlobalElementId,
    open: Model<bool>,
    presence: OverlayPresence,
    children: impl IntoIterator<Item = AnyElement>,
) -> OverlayRequest {
    let children: Vec<AnyElement> = children.into_iter().collect();
    let mut request = OverlayRequest::modal(id, Some(trigger), open, presence, children);
    request.close_on_window_focus_lost = true;
    request.close_on_window_resize = true;
    request.root_name = Some(select_root_name(id));
    request
}

/// Builds an overlay request for a Radix-style select content overlay while routing dismissals
/// through an optional dismiss handler (Radix `DismissableLayer` "preventDefault" outcome).
pub fn modal_select_request_with_dismiss_handler(
    id: GlobalElementId,
    trigger: GlobalElementId,
    open: Model<bool>,
    presence: OverlayPresence,
    on_dismiss_request: Option<OnDismissRequest>,
    children: impl IntoIterator<Item = AnyElement>,
) -> OverlayRequest {
    let mut request = modal_select_request(id, trigger, open, presence, children);
    request.dismissible_on_dismiss_request = on_dismiss_request;
    request
}

/// Requests a select overlay for the current window.
pub fn request_select<H: UiHost>(cx: &mut ElementContext<'_, H>, request: OverlayRequest) {
    OverlayController::request(cx, request);
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::cell::Cell;

    use fret_app::App;
    use fret_core::{
        AppWindowId, Event, Modifiers, MouseButtons, Point, PointerEvent, PointerId, PointerType,
        Px, Rect, Size,
    };
    use fret_ui::action::{UiActionHostAdapter, UiFocusActionHost, UiPointerActionHost};
    use fret_ui::element::{ElementKind, LayoutStyle, PressableProps};
    use std::time::Duration;

    fn bounds() -> Rect {
        Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(200.0), Px(120.0)),
        )
    }

    #[test]
    fn select_root_open_model_uses_controlled_model() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let b = bounds();

        let controlled = app.models_mut().insert(true);

        fret_ui::elements::with_element_cx(&mut app, window, b, "test", |cx| {
            let root = SelectRoot::new()
                .open(Some(controlled.clone()))
                .default_open(false);
            assert_eq!(root.open_model(cx), controlled);
        });
    }

    #[test]
    fn select_initial_focus_targets_gate_by_input_modality() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let b = bounds();

        fret_ui::elements::with_element_cx(&mut app, window, b, "test", |cx| {
            let pointer_focus = GlobalElementId(0x111);
            let keyboard_focus = GlobalElementId(0x222);

            // Pointer modality: prefer pointer content focus.
            fret_ui::input_modality::update_for_event(
                cx.app,
                window,
                &Event::Pointer(PointerEvent::Move {
                    position: Point::new(Px(1.0), Px(2.0)),
                    buttons: MouseButtons::default(),
                    modifiers: Modifiers::default(),
                    pointer_id: PointerId(0),
                    pointer_type: PointerType::Mouse,
                }),
            );
            assert_eq!(
                SelectInitialFocusTargets::new()
                    .pointer_content_focus(Some(pointer_focus))
                    .keyboard_entry_focus(Some(keyboard_focus))
                    .resolve(cx, window),
                Some(pointer_focus)
            );

            // Keyboard modality: prefer keyboard entry focus.
            fret_ui::input_modality::update_for_event(
                cx.app,
                window,
                &Event::KeyDown {
                    key: fret_core::KeyCode::KeyA,
                    modifiers: Modifiers::default(),
                    repeat: false,
                },
            );
            assert_eq!(
                SelectInitialFocusTargets::new()
                    .pointer_content_focus(Some(pointer_focus))
                    .keyboard_entry_focus(Some(keyboard_focus))
                    .resolve(cx, window),
                Some(keyboard_focus)
            );
        });
    }

    #[test]
    fn select_use_value_model_prefers_controlled_and_does_not_call_default() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let b = bounds();

        let controlled = app.models_mut().insert(Some(Arc::from("a")));
        let called = Cell::new(0);

        fret_ui::elements::with_element_cx(&mut app, window, b, "test", |cx| {
            let out = select_use_value_model(cx, Some(controlled.clone()), || {
                called.set(called.get() + 1);
                None
            });
            assert!(out.is_controlled());
            assert_eq!(out.model(), controlled);
        });

        assert_eq!(called.get(), 0);
    }

    struct PointerHost<'a> {
        app: &'a mut App,
        bounds: Rect,
    }

    impl fret_ui::action::UiActionHost for PointerHost<'_> {
        fn models_mut(&mut self) -> &mut fret_runtime::ModelStore {
            self.app.models_mut()
        }

        fn push_effect(&mut self, effect: Effect) {
            self.app.push_effect(effect);
        }

        fn request_redraw(&mut self, window: AppWindowId) {
            self.app.request_redraw(window);
        }

        fn next_timer_token(&mut self) -> TimerToken {
            self.app.next_timer_token()
        }

        fn next_clipboard_token(&mut self) -> fret_runtime::ClipboardToken {
            self.app.next_clipboard_token()
        }
    }

    impl UiFocusActionHost for PointerHost<'_> {
        fn request_focus(&mut self, _target: GlobalElementId) {}
    }

    impl fret_ui::action::UiDragActionHost for PointerHost<'_> {
        fn begin_drag_with_kind(
            &mut self,
            _pointer_id: fret_core::PointerId,
            _kind: fret_runtime::DragKindId,
            _source_window: fret_core::AppWindowId,
            _start: fret_core::Point,
        ) {
        }

        fn begin_cross_window_drag_with_kind(
            &mut self,
            _pointer_id: fret_core::PointerId,
            _kind: fret_runtime::DragKindId,
            _source_window: fret_core::AppWindowId,
            _start: fret_core::Point,
        ) {
        }

        fn drag(&self, _pointer_id: fret_core::PointerId) -> Option<&fret_runtime::DragSession> {
            None
        }

        fn drag_mut(
            &mut self,
            _pointer_id: fret_core::PointerId,
        ) -> Option<&mut fret_runtime::DragSession> {
            None
        }

        fn cancel_drag(&mut self, _pointer_id: fret_core::PointerId) {}
    }

    impl UiPointerActionHost for PointerHost<'_> {
        fn bounds(&self) -> Rect {
            self.bounds
        }

        fn capture_pointer(&mut self) {}

        fn release_pointer_capture(&mut self) {}

        fn prevent_default(&mut self, _action: fret_runtime::DefaultAction) {}

        fn set_cursor_icon(&mut self, _icon: fret_core::CursorIcon) {}
    }

    #[test]
    fn apply_select_trigger_a11y_sets_role_expanded_and_controls() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let b = bounds();

        fret_ui::elements::with_element_cx(&mut app, window, b, "test", |cx| {
            let trigger = cx.pressable(
                PressableProps {
                    layout: LayoutStyle::default(),
                    enabled: true,
                    focusable: true,
                    ..Default::default()
                },
                |_cx, _st| Vec::new(),
            );

            let listbox = GlobalElementId(0xbeef);
            let trigger =
                apply_select_trigger_a11y(trigger, true, Some(Arc::from("Select")), Some(listbox));

            let ElementKind::Pressable(PressableProps { a11y, .. }) = &trigger.kind else {
                panic!("expected pressable trigger");
            };
            assert_eq!(a11y.role, Some(fret_core::SemanticsRole::ComboBox));
            assert_eq!(a11y.expanded, Some(true));
            assert_eq!(a11y.controls_element, Some(listbox.0));
            assert_eq!(a11y.label.as_deref(), Some("Select"));
        });
    }

    #[test]
    fn select_listbox_semantics_id_matches_mounted_listbox_id() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let b = bounds();

        fret_ui::elements::with_element_cx(&mut app, window, b, "test", |cx| {
            let overlay_root_name = "select-overlay";
            let expected = select_listbox_semantics_id::<App>(cx, overlay_root_name);
            let actual = cx.with_root_name(overlay_root_name, |cx| {
                select_listbox_pressable_with_id_props::<App>(cx, |_cx, _st, _id| {
                    (
                        PressableProps {
                            layout: LayoutStyle::default(),
                            enabled: true,
                            focusable: false,
                            ..Default::default()
                        },
                        Vec::new(),
                    )
                })
                .id
            });
            assert_eq!(expected, actual);
        });
    }

    #[test]
    fn modal_select_request_sets_default_root_name() {
        let mut app = App::new();
        let open = app.models_mut().insert(false);
        let id = GlobalElementId(0x123);
        let trigger = GlobalElementId(0x456);

        let req = modal_select_request(
            id,
            trigger,
            open,
            OverlayPresence::instant(true),
            Vec::new(),
        );
        let expected = select_root_name(id);
        assert_eq!(req.root_name.as_deref(), Some(expected.as_str()));
    }

    #[test]
    fn select_content_placement_item_aligned_has_no_wrapper_insets_and_origin_on_rect_edge() {
        let window = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(300.0), Px(200.0)),
        );
        let anchor = Rect::new(
            Point::new(Px(100.0), Px(80.0)),
            Size::new(Px(80.0), Px(24.0)),
        );

        let item_layout = select_item_aligned_layout(SelectItemAlignedInputs {
            direction: popper::LayoutDirection::Ltr,
            window,
            trigger: anchor,
            content: Rect::new(
                Point::new(Px(0.0), Px(0.0)),
                Size::new(Px(160.0), Px(120.0)),
            ),
            value_node: Rect::new(
                Point::new(Px(110.0), Px(84.0)),
                Size::new(Px(60.0), Px(16.0)),
            ),
            selected_item_text: Rect::new(
                Point::new(Px(20.0), Px(40.0)),
                Size::new(Px(80.0), Px(16.0)),
            ),
            selected_item: Rect::new(
                Point::new(Px(10.0), Px(36.0)),
                Size::new(Px(140.0), Px(24.0)),
            ),
            viewport: Rect::new(
                Point::new(Px(10.0), Px(30.0)),
                Size::new(Px(160.0), Px(120.0)),
            ),
            content_border_top: Px(1.0),
            content_padding_top: Px(0.0),
            content_border_bottom: Px(1.0),
            content_padding_bottom: Px(0.0),
            viewport_padding_top: Px(4.0),
            viewport_padding_bottom: Px(4.0),
            selected_item_is_first: false,
            selected_item_is_last: false,
            items_height: Px(240.0),
        });

        let placement = select_content_placement_item_aligned(anchor, item_layout);
        assert_eq!(placement.wrapper_insets, Edges::all(Px(0.0)));
        assert!(placement.popper_layout.is_none());

        match placement.side {
            Side::Bottom => assert_eq!(placement.transform_origin.y, placement.placed.origin.y),
            Side::Top => {
                assert_eq!(
                    placement.transform_origin.y,
                    Px(placement.placed.origin.y.0 + placement.placed.size.height.0)
                )
            }
            Side::Left | Side::Right => {}
        }
    }

    #[test]
    fn select_content_placement_popper_exposes_layout_and_wrapper_insets_when_arrow_enabled() {
        let outer = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(300.0), Px(200.0)),
        );
        let anchor = Rect::new(
            Point::new(Px(120.0), Px(40.0)),
            Size::new(Px(80.0), Px(24.0)),
        );
        let desired = Size::new(Px(180.0), Px(120.0));

        let (arrow_options, arrow_protrusion) =
            popper::diamond_arrow_options(true, Px(12.0), Px(4.0));
        let placement = popper::PopperContentPlacement::new(
            popper::LayoutDirection::Ltr,
            Side::Bottom,
            popper::Align::Start,
            Px(6.0),
        )
        .with_align_offset(Px(0.0))
        .with_arrow(arrow_options, arrow_protrusion);

        let out =
            select_content_placement_popper(outer, anchor, desired, placement, Some(Px(12.0)));
        assert!(out.popper_layout.is_some());
        assert_eq!(out.placed, out.popper_layout.unwrap().rect);
        assert!(
            out.wrapper_insets.top.0 > 0.0
                || out.wrapper_insets.bottom.0 > 0.0
                || out.wrapper_insets.left.0 > 0.0
                || out.wrapper_insets.right.0 > 0.0
        );
    }

    #[test]
    fn select_resolve_content_placement_prefers_item_aligned_layout_when_provided() {
        let outer = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(300.0), Px(200.0)),
        );
        let anchor = Rect::new(
            Point::new(Px(120.0), Px(40.0)),
            Size::new(Px(80.0), Px(24.0)),
        );
        let desired = Size::new(Px(180.0), Px(120.0));

        let item_layout = select_item_aligned_layout(SelectItemAlignedInputs {
            direction: popper::LayoutDirection::Ltr,
            window: outer,
            trigger: anchor,
            content: Rect::new(Point::new(Px(0.0), Px(0.0)), desired),
            value_node: Rect::new(
                Point::new(Px(130.0), Px(44.0)),
                Size::new(Px(60.0), Px(16.0)),
            ),
            selected_item_text: Rect::new(
                Point::new(Px(20.0), Px(40.0)),
                Size::new(Px(80.0), Px(16.0)),
            ),
            selected_item: Rect::new(
                Point::new(Px(10.0), Px(36.0)),
                Size::new(Px(140.0), Px(24.0)),
            ),
            viewport: Rect::new(
                Point::new(Px(10.0), Px(30.0)),
                Size::new(Px(160.0), Px(120.0)),
            ),
            content_border_top: Px(1.0),
            content_padding_top: Px(0.0),
            content_border_bottom: Px(1.0),
            content_padding_bottom: Px(0.0),
            viewport_padding_top: Px(4.0),
            viewport_padding_bottom: Px(4.0),
            selected_item_is_first: false,
            selected_item_is_last: false,
            items_height: Px(240.0),
        });

        let popper_placement = popper::PopperContentPlacement::new(
            popper::LayoutDirection::Ltr,
            Side::Bottom,
            popper::Align::Start,
            Px(6.0),
        );
        let resolved = select_resolve_content_placement(
            anchor,
            outer,
            desired,
            popper_placement,
            None,
            Some(item_layout),
        );

        assert!(resolved.item_aligned_layout.is_some());
        assert!(resolved.placement.popper_layout.is_none());
    }

    #[test]
    fn select_open_keys_match_radix_defaults() {
        assert!(is_select_open_key(KeyCode::Enter));
        assert!(is_select_open_key(KeyCode::Space));
        assert!(is_select_open_key(KeyCode::ArrowDown));
        assert!(is_select_open_key(KeyCode::ArrowUp));
        assert!(!is_select_open_key(KeyCode::Escape));

        assert!(select_open_key_suppresses_activate(KeyCode::Enter));
        assert!(select_open_key_suppresses_activate(KeyCode::Space));
        assert!(!select_open_key_suppresses_activate(KeyCode::ArrowDown));
        assert!(!select_open_key_suppresses_activate(KeyCode::ArrowUp));
    }

    #[test]
    fn select_popper_available_height_tracks_flipped_side_space() {
        let outer = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(100.0), Px(100.0)),
        );
        let anchor = Rect::new(
            Point::new(Px(10.0), Px(70.0)),
            Size::new(Px(30.0), Px(10.0)),
        );

        // Preferred bottom won't fit for a tall list; the solver should flip to top, and the
        // available height should match the top space.
        let placement = popper::PopperContentPlacement::new(
            popper::LayoutDirection::Ltr,
            Side::Bottom,
            popper::Align::Start,
            Px(0.0),
        );

        let vars = select_popper_vars(outer, anchor, Px(0.0), placement);
        assert!(vars.available_height.0 > 60.0 && vars.available_height.0 < 80.0);
    }

    #[test]
    fn select_popper_desired_width_respects_min_width_and_outer_bounds() {
        let outer = Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(80.0), Px(100.0)));
        let anchor = Rect::new(
            Point::new(Px(10.0), Px(10.0)),
            Size::new(Px(24.0), Px(10.0)),
        );

        assert_eq!(
            select_popper_desired_width(outer, anchor, Px(0.0)),
            Px(24.0)
        );
        assert_eq!(
            select_popper_desired_width(outer, anchor, Px(40.0)),
            Px(40.0)
        );
        assert_eq!(
            select_popper_desired_width(outer, anchor, Px(100.0)),
            Px(80.0)
        );
    }

    #[test]
    fn trigger_typeahead_updates_value_without_opening() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let open = app.models_mut().insert(false);
        let value = app.models_mut().insert(None::<Arc<str>>);

        let values: Vec<Arc<str>> = vec![Arc::from("alpha"), Arc::from("beta")];
        let labels: Vec<Arc<str>> = vec![Arc::from("Alpha"), Arc::from("Beta")];
        let disabled = vec![false, false];

        let mut state = SelectTriggerKeyState::default();
        let mut host = UiActionHostAdapter { app: &mut app };
        assert!(state.handle_key_down_when_closed(
            &mut host,
            window,
            &open,
            &value,
            &values,
            &labels,
            &disabled,
            KeyCode::KeyB,
            Modifiers::default(),
            false,
        ));

        assert!(!app.models().get_copied(&open).unwrap_or(false));
        assert_eq!(
            app.models().get_cloned(&value).flatten().as_deref(),
            Some("beta")
        );

        let effects = app.flush_effects();
        assert!(
            effects.iter().any(|e| matches!(
                e,
                Effect::SetTimer { after, .. }
                    if *after == Duration::from_millis(SELECT_TYPEAHEAD_CLEAR_TIMEOUT_MS)
            )),
            "expected a typeahead clear timer"
        );
    }

    #[test]
    fn trigger_open_key_opens_and_suppresses_activate() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let open = app.models_mut().insert(false);
        let value = app.models_mut().insert(None::<Arc<str>>);

        let values: Vec<Arc<str>> = vec![Arc::from("alpha")];
        let labels: Vec<Arc<str>> = vec![Arc::from("Alpha")];
        let disabled = vec![false];

        let mut state = SelectTriggerKeyState::default();
        let mut host = UiActionHostAdapter { app: &mut app };
        assert!(state.handle_key_down_when_closed(
            &mut host,
            window,
            &open,
            &value,
            &values,
            &labels,
            &disabled,
            KeyCode::Enter,
            Modifiers::default(),
            false,
        ));

        assert!(app.models().get_copied(&open).unwrap_or(false));
        assert!(state.take_suppress_next_activate());
    }

    #[test]
    fn content_arrow_navigation_updates_active_row() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let open = app.models_mut().insert(true);
        let value = app.models_mut().insert(None::<Arc<str>>);

        let values_by_row: Vec<Option<Arc<str>>> = vec![
            Some(Arc::from("alpha")),
            Some(Arc::from("beta")),
            Some(Arc::from("gamma")),
        ];
        let labels_by_row: Vec<Arc<str>> =
            vec![Arc::from("Alpha"), Arc::from("Beta"), Arc::from("Gamma")];
        let disabled_by_row = vec![false, true, false];

        let mut state = SelectContentKeyState::default();
        let mut host = UiActionHostAdapter { app: &mut app };

        assert!(state.handle_key_down_when_open(
            &mut host,
            window,
            &open,
            &value,
            &values_by_row,
            &labels_by_row,
            &disabled_by_row,
            KeyCode::ArrowDown,
            false,
            true,
        ));
        // Skips disabled row 1, so we land on row 2.
        assert_eq!(state.active_row(), Some(2));
    }

    #[test]
    fn content_tab_is_suppressed() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let open = app.models_mut().insert(true);
        let value = app.models_mut().insert(None::<Arc<str>>);

        let values_by_row: Vec<Option<Arc<str>>> = vec![Some(Arc::from("beta"))];
        let labels_by_row: Vec<Arc<str>> = vec![Arc::from("Beta")];
        let disabled_by_row = vec![false];

        let mut state = SelectContentKeyState::default();
        let mut host = UiActionHostAdapter { app: &mut app };
        assert!(state.handle_key_down_when_open(
            &mut host,
            window,
            &open,
            &value,
            &values_by_row,
            &labels_by_row,
            &disabled_by_row,
            KeyCode::Tab,
            false,
            true,
        ));

        assert!(app.models().get_copied(&open).unwrap_or(false));
        assert_eq!(state.active_row(), None);
        assert_eq!(app.models().get_cloned(&value).flatten().as_deref(), None);
    }

    #[test]
    fn content_enter_commits_value_and_closes() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let open = app.models_mut().insert(true);
        let value = app.models_mut().insert(None::<Arc<str>>);

        let values_by_row: Vec<Option<Arc<str>>> = vec![Some(Arc::from("beta"))];
        let labels_by_row: Vec<Arc<str>> = vec![Arc::from("Beta")];
        let disabled_by_row = vec![false];

        let mut state = SelectContentKeyState::default();
        state.set_active_row(Some(0));

        let mut host = UiActionHostAdapter { app: &mut app };
        assert!(state.handle_key_down_when_open(
            &mut host,
            window,
            &open,
            &value,
            &values_by_row,
            &labels_by_row,
            &disabled_by_row,
            KeyCode::Enter,
            false,
            true,
        ));

        assert!(!app.models().get_copied(&open).unwrap_or(false));
        assert_eq!(
            app.models().get_cloned(&value).flatten().as_deref(),
            Some("beta")
        );
    }

    #[test]
    fn trigger_pointer_mouse_down_opens() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let open = app.models_mut().insert(false);

        let mut state = SelectTriggerPointerState::default();
        let mut host = PointerHost {
            app: &mut app,
            bounds: bounds(),
        };

        assert!(state.handle_pointer_down(
            &mut host,
            ActionCx {
                window,
                target: GlobalElementId(1),
            },
            PointerDownCx {
                pointer_id: fret_core::PointerId(0),
                position: Point::new(Px(10.0), Px(12.0)),
                tick_id: fret_runtime::TickId(0),
                pixels_per_point: 1.0,
                button: fret_core::MouseButton::Left,
                modifiers: Modifiers::default(),
                click_count: 1,
                pointer_type: PointerType::Mouse,
            },
            &open,
            true,
        ));
        assert!(host.models_mut().get_copied(&open).unwrap_or(false));
    }

    #[test]
    fn trigger_pointer_touch_opens_on_click_like_up() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let open = app.models_mut().insert(false);

        let mut state = SelectTriggerPointerState::default();
        let mut host = PointerHost {
            app: &mut app,
            bounds: bounds(),
        };

        assert!(state.handle_pointer_down(
            &mut host,
            ActionCx {
                window,
                target: GlobalElementId(1),
            },
            PointerDownCx {
                pointer_id: fret_core::PointerId(0),
                position: Point::new(Px(10.0), Px(12.0)),
                tick_id: fret_runtime::TickId(0),
                pixels_per_point: 1.0,
                button: fret_core::MouseButton::Left,
                modifiers: Modifiers::default(),
                click_count: 1,
                pointer_type: PointerType::Touch,
            },
            &open,
            true,
        ));
        assert!(!host.models_mut().get_copied(&open).unwrap_or(false));

        assert!(state.handle_pointer_up(
            &mut host,
            ActionCx {
                window,
                target: GlobalElementId(1),
            },
            PointerUpCx {
                pointer_id: fret_core::PointerId(0),
                position: Point::new(Px(13.0), Px(15.0)),
                tick_id: fret_runtime::TickId(0),
                pixels_per_point: 1.0,
                button: fret_core::MouseButton::Left,
                modifiers: Modifiers::default(),
                is_click: true,
                click_count: 1,
                pointer_type: PointerType::Touch,
            },
            &open,
            true,
        ));
        assert!(host.models_mut().get_copied(&open).unwrap_or(false));
    }

    #[test]
    fn trigger_pointer_touch_drag_does_not_open() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let open = app.models_mut().insert(false);

        let mut state = SelectTriggerPointerState::default();
        let mut host = PointerHost {
            app: &mut app,
            bounds: bounds(),
        };

        assert!(state.handle_pointer_down(
            &mut host,
            ActionCx {
                window,
                target: GlobalElementId(1),
            },
            PointerDownCx {
                pointer_id: fret_core::PointerId(0),
                position: Point::new(Px(10.0), Px(12.0)),
                tick_id: fret_runtime::TickId(0),
                pixels_per_point: 1.0,
                button: fret_core::MouseButton::Left,
                modifiers: Modifiers::default(),
                click_count: 1,
                pointer_type: PointerType::Touch,
            },
            &open,
            true,
        ));
        assert!(state.handle_pointer_move(
            &mut host,
            ActionCx {
                window,
                target: GlobalElementId(1),
            },
            PointerMoveCx {
                pointer_id: fret_core::PointerId(0),
                position: Point::new(Px(40.0), Px(12.0)),
                tick_id: fret_runtime::TickId(0),
                pixels_per_point: 1.0,
                buttons: fret_core::MouseButtons {
                    left: true,
                    right: false,
                    middle: false,
                },
                modifiers: Modifiers::default(),
                pointer_type: PointerType::Touch,
            },
        ));
        assert!(state.handle_pointer_up(
            &mut host,
            ActionCx {
                window,
                target: GlobalElementId(1),
            },
            PointerUpCx {
                pointer_id: fret_core::PointerId(0),
                position: Point::new(Px(40.0), Px(12.0)),
                tick_id: fret_runtime::TickId(0),
                pixels_per_point: 1.0,
                button: fret_core::MouseButton::Left,
                modifiers: Modifiers::default(),
                is_click: false,
                click_count: 1,
                pointer_type: PointerType::Touch,
            },
            &open,
            true,
        ));
        assert!(!host.models_mut().get_copied(&open).unwrap_or(false));
    }
}
