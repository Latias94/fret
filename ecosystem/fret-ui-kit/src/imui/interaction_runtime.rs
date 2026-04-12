//! Internal immediate-mode interaction runtime helpers.

use std::cell::Cell;
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::Arc;
use std::time::Duration;

use fret_core::{AppWindowId, CursorIcon, Modifiers, MouseButton, Point, Px};
use fret_interaction::drag::DragThreshold as InteractionDragThreshold;
use fret_interaction::runtime_drag::{DragMoveOutcome, update_thresholded_move};
use fret_ui::action::UiActionHostExt as _;
use fret_ui::{ElementContext, GlobalElementId, UiHost};

#[derive(Debug, Default)]
struct DragReportState {
    last_position: Option<Point>,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(super) struct LongPressSignalState {
    pub(super) timer: Option<fret_runtime::TimerToken>,
    pub(super) holding: bool,
}

#[derive(Default)]
struct ImUiContextMenuAnchorStore {
    by_element: HashMap<GlobalElementId, fret_runtime::Model<Option<Point>>>,
}

#[derive(Default)]
struct ImUiLongPressStore {
    by_element: HashMap<GlobalElementId, fret_runtime::Model<LongPressSignalState>>,
}

#[derive(Default)]
struct ImUiPointerClickModifiersStore {
    by_element: HashMap<GlobalElementId, fret_runtime::Model<Modifiers>>,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
struct ImUiSharedHoverDelayState {
    delay_short_met: bool,
    delay_normal_met: bool,
    short_timer: Option<fret_runtime::TimerToken>,
    normal_timer: Option<fret_runtime::TimerToken>,
    clear_timer: Option<fret_runtime::TimerToken>,
}

#[derive(Default)]
struct ImUiSharedHoverDelayStore {
    by_window: HashMap<AppWindowId, fret_runtime::Model<ImUiSharedHoverDelayState>>,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(super) struct ImUiActiveItemState {
    pub(super) active: Option<GlobalElementId>,
}

#[derive(Default)]
struct ImUiActiveItemStore {
    by_window: HashMap<AppWindowId, fret_runtime::Model<ImUiActiveItemState>>,
}

#[derive(Default)]
struct ImUiFloatWindowCollapsedStore {
    by_element: HashMap<GlobalElementId, fret_runtime::Model<bool>>,
}

#[derive(Default)]
struct ImUiDisabledScopeStore {
    depth: Rc<Cell<u32>>,
}

#[derive(Debug, Default, Clone, Copy)]
struct HoverQueryDelayLocalState {
    stationary_met: bool,
    delay_short_met: bool,
    delay_normal_met: bool,
}

#[derive(Debug, Default, Clone, Copy)]
pub(super) struct HoverQueryDelayRead {
    pub(super) stationary_met: bool,
    pub(super) delay_short_met: bool,
    pub(super) delay_normal_met: bool,
    pub(super) shared_delay_short_met: bool,
    pub(super) shared_delay_normal_met: bool,
}

const HOVER_TIMER_KIND_STATIONARY: u64 =
    super::fnv1a64(b"fret-ui-kit.imui.hover.timer.stationary.v1");
const HOVER_TIMER_KIND_DELAY_SHORT: u64 =
    super::fnv1a64(b"fret-ui-kit.imui.hover.timer.delay_short.v1");
const HOVER_TIMER_KIND_DELAY_NORMAL: u64 =
    super::fnv1a64(b"fret-ui-kit.imui.hover.timer.delay_normal.v1");

const SHARED_HOVER_CLEAR_DELAY: Duration = Duration::from_millis(250);

pub(super) fn context_menu_anchor_model_for<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    id: GlobalElementId,
) -> fret_runtime::Model<Option<Point>> {
    cx.app
        .with_global_mut_untracked(ImUiContextMenuAnchorStore::default, |st, app| {
            st.by_element
                .entry(id)
                .or_insert_with(|| app.models_mut().insert(None::<Point>))
                .clone()
        })
}

pub(super) fn long_press_signal_model_for<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    id: GlobalElementId,
) -> fret_runtime::Model<LongPressSignalState> {
    cx.app
        .with_global_mut_untracked(ImUiLongPressStore::default, |st, app| {
            st.by_element
                .entry(id)
                .or_insert_with(|| app.models_mut().insert(LongPressSignalState::default()))
                .clone()
        })
}

pub(super) fn pointer_click_modifiers_model_for<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    id: GlobalElementId,
) -> fret_runtime::Model<Modifiers> {
    cx.app
        .with_global_mut_untracked(ImUiPointerClickModifiersStore::default, |st, app| {
            st.by_element
                .entry(id)
                .or_insert_with(|| app.models_mut().insert(Modifiers::default()))
                .clone()
        })
}

fn shared_hover_delay_model_for_window<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
) -> fret_runtime::Model<ImUiSharedHoverDelayState> {
    let window = cx.window;
    cx.app
        .with_global_mut_untracked(ImUiSharedHoverDelayStore::default, |st, app| {
            st.by_window
                .entry(window)
                .or_insert_with(|| {
                    app.models_mut()
                        .insert(ImUiSharedHoverDelayState::default())
                })
                .clone()
        })
}

pub(super) fn active_item_model_for_window<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
) -> fret_runtime::Model<ImUiActiveItemState> {
    let window = cx.window;
    cx.app
        .with_global_mut_untracked(ImUiActiveItemStore::default, |st, app| {
            st.by_window
                .entry(window)
                .or_insert_with(|| app.models_mut().insert(ImUiActiveItemState::default()))
                .clone()
        })
}

pub(super) fn hover_blocked_by_active_item_for<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    id: GlobalElementId,
    active_item_model: &fret_runtime::Model<ImUiActiveItemState>,
) -> bool {
    let active = cx
        .read_model(
            active_item_model,
            fret_ui::Invalidation::Paint,
            |_app, st| st.active,
        )
        .ok()
        .flatten();
    active.is_some() && active != Some(id)
}

pub(super) fn float_window_collapsed_model_for<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    id: GlobalElementId,
) -> fret_runtime::Model<bool> {
    cx.app
        .with_global_mut_untracked(ImUiFloatWindowCollapsedStore::default, |st, app| {
            st.by_element
                .entry(id)
                .or_insert_with(|| app.models_mut().insert(false))
                .clone()
        })
}

pub(super) fn drag_kind_for_element(element: GlobalElementId) -> fret_runtime::DragKindId {
    fret_runtime::DragKindId(super::DRAG_KIND_MASK | element.0)
}

pub(super) fn disabled_scope_depth_for<H: UiHost>(cx: &mut ElementContext<'_, H>) -> Rc<Cell<u32>> {
    cx.app
        .with_global_mut_untracked(ImUiDisabledScopeStore::default, |st, _app| st.depth.clone())
}

pub(super) fn imui_is_disabled<H: UiHost>(cx: &mut ElementContext<'_, H>) -> bool {
    disabled_scope_depth_for(cx).get() > 0
}

pub(super) fn disabled_alpha_for<H: UiHost>(cx: &ElementContext<'_, H>) -> f32 {
    let theme = fret_ui::Theme::global(&*cx.app);
    let v = theme
        .number_by_key(crate::theme_tokens::number::COMPONENT_IMUI_DISABLED_ALPHA)
        .unwrap_or(super::DEFAULT_DISABLED_ALPHA);
    v.clamp(0.0, 1.0)
}

pub(super) struct DisabledScopeGuard {
    depth: Rc<Cell<u32>>,
    active: bool,
}

impl DisabledScopeGuard {
    pub(super) fn push(depth: Rc<Cell<u32>>) -> Self {
        depth.set(depth.get().saturating_add(1));
        Self {
            depth,
            active: true,
        }
    }
}

impl Drop for DisabledScopeGuard {
    fn drop(&mut self) {
        if !self.active {
            return;
        }
        let v = self.depth.get();
        self.depth.set(v.saturating_sub(1));
    }
}

pub(super) fn sanitize_response_for_enabled(enabled: bool, response: &mut super::ResponseExt) {
    response.enabled = enabled;
    if enabled {
        return;
    }
    response.core.hovered = false;
    response.core.pressed = false;
    response.core.focused = false;
    response.core.clicked = false;
    response.core.changed = false;
    response.nav_highlighted = false;
    response.secondary_clicked = false;
    response.double_clicked = false;
    response.long_pressed = false;
    response.press_holding = false;
    response.context_menu_requested = false;
    response.context_menu_anchor = None;
    response.pointer_clicked = false;
    response.pointer_click_modifiers = Modifiers::default();
    response.drag = super::DragResponse::default();
}

fn hover_timer_token_for(kind: u64, element: GlobalElementId) -> fret_runtime::TimerToken {
    let mut hash = 0xcbf2_9ce4_8422_2325u64;
    for b in kind.to_le_bytes() {
        hash ^= b as u64;
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3u64);
    }
    for b in element.0.to_le_bytes() {
        hash ^= b as u64;
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3u64);
    }
    fret_runtime::TimerToken(hash)
}

fn shared_hover_delay_on_hover_change(
    host: &mut dyn fret_ui::action::UiActionHost,
    action_cx: fret_ui::action::ActionCx,
    hovered: bool,
    shared_model: &fret_runtime::Model<ImUiSharedHoverDelayState>,
) {
    let prev = host
        .models_mut()
        .read(shared_model, |st| *st)
        .ok()
        .unwrap_or_default();

    if hovered {
        if let Some(token) = prev.clear_timer {
            host.push_effect(fret_runtime::Effect::CancelTimer { token });
        }

        let mut next = prev;
        next.clear_timer = None;

        if !prev.delay_short_met && prev.short_timer.is_none() {
            let token = host.next_timer_token();
            next.short_timer = Some(token);
            host.push_effect(fret_runtime::Effect::SetTimer {
                window: Some(action_cx.window),
                token,
                after: super::HOVER_DELAY_SHORT,
                repeat: None,
            });
        }

        if !prev.delay_normal_met && prev.normal_timer.is_none() {
            let token = host.next_timer_token();
            next.normal_timer = Some(token);
            host.push_effect(fret_runtime::Effect::SetTimer {
                window: Some(action_cx.window),
                token,
                after: super::HOVER_DELAY_NORMAL,
                repeat: None,
            });
        }

        let _ = host.models_mut().update(shared_model, |st| *st = next);
        return;
    }

    if prev.clear_timer.is_some() {
        return;
    }

    let token = host.next_timer_token();
    host.push_effect(fret_runtime::Effect::SetTimer {
        window: Some(action_cx.window),
        token,
        after: SHARED_HOVER_CLEAR_DELAY,
        repeat: None,
    });

    let mut next = prev;
    next.clear_timer = Some(token);
    let _ = host.models_mut().update(shared_model, |st| *st = next);
}

fn shared_hover_delay_on_timer(
    host: &mut dyn fret_ui::action::UiFocusActionHost,
    action_cx: fret_ui::action::ActionCx,
    token: fret_runtime::TimerToken,
    shared_model: &fret_runtime::Model<ImUiSharedHoverDelayState>,
) -> bool {
    let prev = host
        .models_mut()
        .read(shared_model, |st| *st)
        .ok()
        .unwrap_or_default();

    if prev.short_timer == Some(token) {
        let mut next = prev;
        next.delay_short_met = true;
        next.short_timer = None;
        let _ = host.models_mut().update(shared_model, |st| *st = next);
        host.notify(action_cx);
        return true;
    }

    if prev.normal_timer == Some(token) {
        let mut next = prev;
        next.delay_normal_met = true;
        next.normal_timer = None;
        let _ = host.models_mut().update(shared_model, |st| *st = next);
        host.notify(action_cx);
        return true;
    }

    if prev.clear_timer == Some(token) {
        if let Some(token) = prev.short_timer {
            host.push_effect(fret_runtime::Effect::CancelTimer { token });
        }
        if let Some(token) = prev.normal_timer {
            host.push_effect(fret_runtime::Effect::CancelTimer { token });
        }
        let _ = host.models_mut().update(shared_model, |st| {
            *st = ImUiSharedHoverDelayState::default()
        });
        host.notify(action_cx);
        return true;
    }

    false
}

pub(super) fn install_hover_query_hooks_for_pressable<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    id: GlobalElementId,
    hovered_raw: bool,
    long_press_signal_model: Option<fret_runtime::Model<LongPressSignalState>>,
) -> HoverQueryDelayRead {
    let shared_delay_model = shared_hover_delay_model_for_window(cx);
    let shared_delay_model_for_hover = shared_delay_model.clone();
    cx.pressable_on_hover_change(Arc::new(move |host, action_cx, hovered| {
        let stationary = hover_timer_token_for(HOVER_TIMER_KIND_STATIONARY, action_cx.target);
        let delay_short = hover_timer_token_for(HOVER_TIMER_KIND_DELAY_SHORT, action_cx.target);
        let delay_normal = hover_timer_token_for(HOVER_TIMER_KIND_DELAY_NORMAL, action_cx.target);

        if hovered {
            shared_hover_delay_on_hover_change(
                host,
                action_cx,
                true,
                &shared_delay_model_for_hover,
            );
            host.push_effect(fret_runtime::Effect::SetTimer {
                window: Some(action_cx.window),
                token: stationary,
                after: super::HOVER_STATIONARY_DELAY,
                repeat: None,
            });
            host.push_effect(fret_runtime::Effect::SetTimer {
                window: Some(action_cx.window),
                token: delay_short,
                after: super::HOVER_DELAY_SHORT,
                repeat: None,
            });
            host.push_effect(fret_runtime::Effect::SetTimer {
                window: Some(action_cx.window),
                token: delay_normal,
                after: super::HOVER_DELAY_NORMAL,
                repeat: None,
            });
            return;
        }

        shared_hover_delay_on_hover_change(host, action_cx, false, &shared_delay_model_for_hover);
        host.push_effect(fret_runtime::Effect::CancelTimer { token: stationary });
        host.push_effect(fret_runtime::Effect::CancelTimer { token: delay_short });
        host.push_effect(fret_runtime::Effect::CancelTimer {
            token: delay_normal,
        });
    }));

    let long_press_signal_model_for_timer = long_press_signal_model.clone();
    let shared_delay_model_for_timer = shared_delay_model.clone();
    cx.timer_on_timer_for(
        id,
        Arc::new(move |host, action_cx, token| {
            let stationary = hover_timer_token_for(HOVER_TIMER_KIND_STATIONARY, action_cx.target);
            if token == stationary {
                host.record_transient_event(action_cx, super::KEY_HOVER_STATIONARY_MET);
                host.notify(action_cx);
                return true;
            }
            let delay_short = hover_timer_token_for(HOVER_TIMER_KIND_DELAY_SHORT, action_cx.target);
            if token == delay_short {
                host.record_transient_event(action_cx, super::KEY_HOVER_DELAY_SHORT_MET);
                host.notify(action_cx);
                return true;
            }
            let delay_normal =
                hover_timer_token_for(HOVER_TIMER_KIND_DELAY_NORMAL, action_cx.target);
            if token == delay_normal {
                host.record_transient_event(action_cx, super::KEY_HOVER_DELAY_NORMAL_MET);
                host.notify(action_cx);
                return true;
            }

            if shared_hover_delay_on_timer(host, action_cx, token, &shared_delay_model_for_timer) {
                return true;
            }

            if let Some(model) = long_press_signal_model_for_timer.as_ref() {
                return emit_long_press_if_matching(host, action_cx, model, token);
            }

            false
        }),
    );

    let stationary_fired = cx.take_transient_for(id, super::KEY_HOVER_STATIONARY_MET);
    let delay_short_fired = cx.take_transient_for(id, super::KEY_HOVER_DELAY_SHORT_MET);
    let delay_normal_fired = cx.take_transient_for(id, super::KEY_HOVER_DELAY_NORMAL_MET);

    let local = cx.state_for(id, HoverQueryDelayLocalState::default, |st| {
        if stationary_fired {
            st.stationary_met = true;
        }
        if delay_short_fired {
            st.delay_short_met = true;
        }
        if delay_normal_fired {
            st.delay_normal_met = true;
        }

        if !hovered_raw {
            *st = HoverQueryDelayLocalState::default();
        }

        *st
    });

    let shared = cx
        .read_model(
            &shared_delay_model,
            fret_ui::Invalidation::Paint,
            |_app, st| (st.delay_short_met, st.delay_normal_met),
        )
        .unwrap_or((false, false));

    HoverQueryDelayRead {
        stationary_met: local.stationary_met,
        delay_short_met: local.delay_short_met,
        delay_normal_met: local.delay_normal_met,
        shared_delay_short_met: shared.0,
        shared_delay_normal_met: shared.1,
    }
}

pub(super) fn drag_threshold_for<H: UiHost>(
    cx: &ElementContext<'_, H>,
) -> InteractionDragThreshold {
    let theme = fret_ui::Theme::global(&*cx.app);
    let px = theme
        .metric_by_key(crate::theme_tokens::metric::COMPONENT_IMUI_DRAG_THRESHOLD_PX)
        .unwrap_or(Px(super::DEFAULT_DRAG_THRESHOLD_PX));
    InteractionDragThreshold::new(px)
}

pub(super) fn handle_pressable_drag_move_with_threshold(
    host: &mut dyn fret_ui::action::UiPointerActionHost,
    acx: fret_ui::action::ActionCx,
    mv: fret_ui::action::PointerMoveCx,
    active_item_model: &fret_runtime::Model<ImUiActiveItemState>,
    long_press_signal_model: &fret_runtime::Model<LongPressSignalState>,
    drag_kind: fret_runtime::DragKindId,
    drag_threshold: InteractionDragThreshold,
) -> bool {
    let (outcome, was_dragging) = {
        let Some(drag) = host.drag_mut(mv.pointer_id) else {
            return false;
        };
        if drag.kind != drag_kind || drag.source_window != acx.window {
            return false;
        }

        let was_dragging = drag.dragging;
        let outcome = update_thresholded_move(
            drag,
            acx.window,
            mv.position,
            mv.buttons.left,
            drag_threshold,
        );
        (outcome, was_dragging)
    };

    match outcome {
        DragMoveOutcome::Canceled => {
            if was_dragging {
                host.record_transient_event(acx, super::KEY_DRAG_STOPPED);
            }
            let _ = host.update_model(active_item_model, |st| {
                if st.active == Some(acx.target) {
                    st.active = None;
                }
            });
            host.cancel_drag(mv.pointer_id);
            cancel_long_press_timer_for(host, long_press_signal_model);
            host.notify(acx);
            false
        }
        DragMoveOutcome::StartedDragging => {
            cancel_long_press_timer_for(host, long_press_signal_model);
            host.record_transient_event(acx, super::KEY_DRAG_STARTED);
            host.notify(acx);
            false
        }
        DragMoveOutcome::Continue => {
            host.notify(acx);
            false
        }
    }
}

pub(super) fn finish_pressable_drag_on_pointer_up(
    host: &mut dyn fret_ui::action::UiPointerActionHost,
    acx: fret_ui::action::ActionCx,
    up: fret_ui::action::PointerUpCx,
    active_item_model: &fret_runtime::Model<ImUiActiveItemState>,
    long_press_signal_model: &fret_runtime::Model<LongPressSignalState>,
    drag_kind: fret_runtime::DragKindId,
) {
    if up.button == MouseButton::Left {
        let _ = host.update_model(active_item_model, |st| {
            if st.active == Some(acx.target) {
                st.active = None;
            }
        });
        cancel_long_press_timer_for(host, long_press_signal_model);
    }

    if let Some(drag) = host.drag(up.pointer_id)
        && drag.kind == drag_kind
        && drag.source_window == acx.window
    {
        if drag.dragging {
            host.record_transient_event(acx, super::KEY_DRAG_STOPPED);
        }
        host.cancel_drag(up.pointer_id);
        host.notify(acx);
    }
}

pub(super) fn populate_pressable_drag_response<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    id: GlobalElementId,
    response: &mut super::ResponseExt,
) {
    response.drag.started = cx.take_transient_for(id, super::KEY_DRAG_STARTED);
    response.drag.stopped = cx.take_transient_for(id, super::KEY_DRAG_STOPPED);
    response.drag.dragging = false;
    response.drag.delta = Point::default();
    response.drag.total = Point::default();

    let kind = drag_kind_for_element(id);
    let pointer_id = cx.app.find_drag_pointer_id(|d| {
        d.kind == kind && d.source_window == cx.window && d.current_window == cx.window
    });
    let drag_snapshot = pointer_id.and_then(|pointer_id| {
        cx.app
            .drag(pointer_id)
            .filter(|drag| drag.kind == kind)
            .map(|drag| (drag.dragging, drag.position, drag.start_position))
    });
    if let Some((dragging, current, start)) = drag_snapshot {
        response.drag.dragging = dragging;
        let (delta, total) = cx.state_for(id, DragReportState::default, |st| {
            let prev = st.last_position.unwrap_or(current);
            st.last_position = Some(current);
            (
                super::point_sub(current, prev),
                super::point_sub(current, start),
            )
        });
        if dragging {
            response.drag.delta = delta;
            response.drag.total = total;
        }
    } else {
        cx.state_for(id, DragReportState::default, |st| {
            st.last_position = None;
        });
    }
}

pub(super) fn mark_active_item_on_left_pointer_down(
    host: &mut dyn fret_ui::action::UiPointerActionHost,
    acx: fret_ui::action::ActionCx,
    button: MouseButton,
    active_item_model: &fret_runtime::Model<ImUiActiveItemState>,
    request_focus: bool,
) {
    if button != MouseButton::Left {
        return;
    }
    if request_focus {
        host.request_focus(acx.target);
    }
    let _ = host.update_model(active_item_model, |st| {
        st.active = Some(acx.target);
    });
    host.notify(acx);
}

pub(super) fn clear_active_item_on_left_pointer_up(
    host: &mut dyn fret_ui::action::UiPointerActionHost,
    acx: fret_ui::action::ActionCx,
    button: MouseButton,
    active_item_model: &fret_runtime::Model<ImUiActiveItemState>,
) {
    if button != MouseButton::Left {
        return;
    }
    let _ = host.update_model(active_item_model, |st| {
        if st.active == Some(acx.target) {
            st.active = None;
        }
    });
    host.notify(acx);
}

pub(super) fn prepare_pressable_drag_on_pointer_down(
    host: &mut dyn fret_ui::action::UiPointerActionHost,
    acx: fret_ui::action::ActionCx,
    down: fret_ui::action::PointerDownCx,
    active_item_model: &fret_runtime::Model<ImUiActiveItemState>,
    long_press_signal_model: &fret_runtime::Model<LongPressSignalState>,
    drag_kind: fret_runtime::DragKindId,
) {
    if down.button != MouseButton::Left {
        return;
    }

    let _ = host.update_model(active_item_model, |st| {
        st.active = Some(acx.target);
    });
    arm_long_press_timer_for(host, acx, long_press_signal_model);

    if host.drag(down.pointer_id).is_none() {
        host.begin_drag_with_kind(down.pointer_id, drag_kind, acx.window, down.position);
    }
}

pub(super) fn prepare_pointer_region_drag_on_left_down(
    host: &mut dyn fret_ui::action::UiPointerActionHost,
    acx: fret_ui::action::ActionCx,
    down: fret_ui::action::PointerDownCx,
    drag_kind: Option<fret_runtime::DragKindId>,
    cursor: Option<CursorIcon>,
) -> bool {
    if down.button != MouseButton::Left {
        return false;
    }

    host.request_focus(acx.target);
    if let Some(cursor) = cursor {
        host.set_cursor_icon(cursor);
    }
    if let Some(drag_kind) = drag_kind {
        host.capture_pointer();
        if host.drag(down.pointer_id).is_none() {
            host.begin_drag_with_kind(down.pointer_id, drag_kind, acx.window, down.position);
        }
    }
    true
}

pub(super) fn handle_pointer_region_drag_move_with_threshold(
    host: &mut dyn fret_ui::action::UiPointerActionHost,
    acx: fret_ui::action::ActionCx,
    mv: fret_ui::action::PointerMoveCx,
    drag_kind: fret_runtime::DragKindId,
    drag_threshold: InteractionDragThreshold,
) -> bool {
    let Some(drag) = host.drag_mut(mv.pointer_id) else {
        return false;
    };
    if drag.kind != drag_kind || drag.source_window != acx.window {
        return false;
    }

    let outcome = update_thresholded_move(
        drag,
        acx.window,
        mv.position,
        mv.buttons.left,
        drag_threshold,
    );
    if outcome == DragMoveOutcome::Canceled {
        host.cancel_drag(mv.pointer_id);
        host.release_pointer_capture();
        host.notify(acx);
        return false;
    }

    host.notify(acx);
    false
}

pub(super) fn finish_pointer_region_drag(
    host: &mut dyn fret_ui::action::UiPointerActionHost,
    acx: fret_ui::action::ActionCx,
    pointer_id: fret_core::PointerId,
    drag_kind: fret_runtime::DragKindId,
) -> bool {
    if let Some(drag) = host.drag(pointer_id)
        && drag.kind == drag_kind
        && drag.source_window == acx.window
    {
        host.cancel_drag(pointer_id);
    }
    host.release_pointer_capture();
    host.notify(acx);
    false
}

fn arm_long_press_timer_for(
    host: &mut dyn fret_ui::action::UiActionHost,
    action_cx: fret_ui::action::ActionCx,
    model: &fret_runtime::Model<LongPressSignalState>,
) {
    let token = host.next_timer_token();
    let previous = host
        .update_model(model, |state| {
            let previous = state.timer.take();
            state.timer = Some(token);
            state.holding = false;
            previous
        })
        .flatten();
    if let Some(previous) = previous {
        host.push_effect(fret_runtime::Effect::CancelTimer { token: previous });
    }
    host.push_effect(fret_runtime::Effect::SetTimer {
        window: Some(action_cx.window),
        token,
        after: super::LONG_PRESS_DELAY,
        repeat: None,
    });
}

fn cancel_long_press_timer_for(
    host: &mut dyn fret_ui::action::UiActionHost,
    model: &fret_runtime::Model<LongPressSignalState>,
) {
    let previous = host
        .update_model(model, |state| {
            let previous = state.timer.take();
            state.holding = false;
            previous
        })
        .flatten();
    if let Some(previous) = previous {
        host.push_effect(fret_runtime::Effect::CancelTimer { token: previous });
    }
}

fn emit_long_press_if_matching(
    host: &mut dyn fret_ui::action::UiActionHost,
    action_cx: fret_ui::action::ActionCx,
    model: &fret_runtime::Model<LongPressSignalState>,
    token: fret_runtime::TimerToken,
) -> bool {
    let fired = host
        .update_model(model, |state| {
            if state.timer != Some(token) {
                return false;
            }
            state.timer = None;
            state.holding = true;
            true
        })
        .unwrap_or(false);
    if fired {
        host.record_transient_event(action_cx, super::KEY_LONG_PRESSED);
        host.notify(action_cx);
    }
    fired
}
