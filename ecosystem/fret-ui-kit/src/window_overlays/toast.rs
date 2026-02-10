use std::collections::HashMap;
use std::sync::Arc;
use std::sync::Mutex;
use std::time::Duration;

use fret_core::{AppWindowId, Point, Px, TimerToken};
use fret_runtime::{CommandId, Effect, Model};
use fret_ui::UiHost;
use fret_ui::action::UiActionHostAdapter;
use fret_ui::elements::GlobalElementId;

pub(super) const TOAST_CLOSE_DURATION: Duration = Duration::from_millis(200);
pub(super) const TOAST_AUTO_CLOSE_TICK: Duration = Duration::from_millis(100);
pub const DEFAULT_MAX_TOASTS: usize = 3;
pub const DEFAULT_SWIPE_THRESHOLD_PX: f32 = 50.0;
pub const DEFAULT_SWIPE_MAX_DRAG_PX: f32 = 240.0;
pub const DEFAULT_SWIPE_DRAGGING_THRESHOLD_PX: f32 = 4.0;
pub const DEFAULT_VISIBLE_TOASTS: usize = 3;
pub const DEFAULT_TOAST_DURATION: Duration = Duration::from_millis(4000);

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum ToastPosition {
    TopLeft,
    TopCenter,
    TopRight,
    BottomLeft,
    BottomCenter,
    #[default]
    BottomRight,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum ToastVariant {
    #[default]
    Default,
    Destructive,
    Success,
    Info,
    Warning,
    Error,
    Loading,
}

/// Mirrors Radix toast `swipeDirection` (`left`/`right`/`up`/`down`).
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ToastSwipeDirection {
    Left,
    #[default]
    Right,
    Up,
    Down,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ToastSwipeConfig {
    pub direction: ToastSwipeDirection,
    pub threshold: Px,
    pub max_drag: Px,
    pub dragging_threshold: Px,
}

impl Default for ToastSwipeConfig {
    fn default() -> Self {
        Self {
            direction: ToastSwipeDirection::default(),
            threshold: Px(DEFAULT_SWIPE_THRESHOLD_PX),
            max_drag: Px(DEFAULT_SWIPE_MAX_DRAG_PX),
            dragging_threshold: Px(DEFAULT_SWIPE_DRAGGING_THRESHOLD_PX),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ToastAction {
    pub label: Arc<str>,
    pub command: CommandId,
}

#[derive(Debug, Clone)]
pub struct ToastRequest {
    pub id: Option<ToastId>,
    pub toaster_id: Option<Arc<str>>,
    pub title: Arc<str>,
    pub description: Option<Arc<str>>,
    pub duration: Option<Duration>,
    pub variant: ToastVariant,
    pub action: Option<ToastAction>,
    pub cancel: Option<ToastAction>,
    pub dismissible: bool,
    pub position: Option<ToastPosition>,
    pub rich_colors: Option<bool>,
    pub invert: bool,
    pub test_id: Option<Arc<str>>,
}

impl ToastRequest {
    pub fn new(title: impl Into<Arc<str>>) -> Self {
        Self {
            id: None,
            toaster_id: None,
            title: title.into(),
            description: None,
            duration: Some(DEFAULT_TOAST_DURATION),
            variant: ToastVariant::default(),
            action: None,
            cancel: None,
            dismissible: true,
            position: None,
            rich_colors: None,
            invert: false,
            test_id: None,
        }
    }

    pub fn description(mut self, description: impl Into<Arc<str>>) -> Self {
        self.description = Some(description.into());
        self
    }

    pub fn id(mut self, id: ToastId) -> Self {
        self.id = Some(id);
        self
    }

    pub fn toaster_id(mut self, id: impl Into<Arc<str>>) -> Self {
        self.toaster_id = Some(id.into());
        self
    }

    pub fn toaster_id_opt(mut self, id: Option<Arc<str>>) -> Self {
        self.toaster_id = id;
        self
    }

    pub fn duration(mut self, duration: Option<Duration>) -> Self {
        self.duration = duration;
        self
    }

    pub fn variant(mut self, variant: ToastVariant) -> Self {
        self.variant = variant;
        self
    }

    pub fn action(mut self, action: ToastAction) -> Self {
        self.action = Some(action);
        self
    }

    pub fn cancel(mut self, cancel: ToastAction) -> Self {
        self.cancel = Some(cancel);
        self
    }

    pub fn dismissible(mut self, dismissible: bool) -> Self {
        self.dismissible = dismissible;
        self
    }

    pub fn position(mut self, position: ToastPosition) -> Self {
        self.position = Some(position);
        self
    }

    pub fn rich_colors(mut self, rich_colors: bool) -> Self {
        self.rich_colors = Some(rich_colors);
        self
    }

    pub fn invert(mut self, invert: bool) -> Self {
        self.invert = invert;
        self
    }

    pub fn test_id(mut self, test_id: impl Into<Arc<str>>) -> Self {
        self.test_id = Some(test_id.into());
        self
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ToastId(pub u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ToastTimerKind {
    AutoClose,
    RemoveAfterClose,
}

#[derive(Debug, Clone, Copy)]
struct ToastTimerRef {
    window: AppWindowId,
    toast: ToastId,
    kind: ToastTimerKind,
}

#[derive(Debug, Clone)]
pub(super) struct ToastEntry {
    pub(super) id: ToastId,
    pub(super) toaster_id: Option<Arc<str>>,
    pub(super) title: Arc<str>,
    pub(super) description: Option<Arc<str>>,
    pub(super) duration: Option<Duration>,
    pub(super) auto_close_remaining: Option<Duration>,
    pub(super) variant: ToastVariant,
    pub(super) action: Option<ToastAction>,
    pub(super) cancel: Option<ToastAction>,
    pub(super) dismissible: bool,
    pub(super) position: Option<ToastPosition>,
    pub(super) rich_colors: Option<bool>,
    pub(super) invert: bool,
    pub(super) test_id: Option<Arc<str>>,
    pub(super) measured_height: Option<Px>,
    pub(super) open: bool,
    pub(super) auto_close_token: Option<TimerToken>,
    pub(super) remove_token: Option<TimerToken>,
    pub(super) drag_start: Option<Point>,
    pub(super) drag_offset: Point,
    pub(super) settle_from: Option<Point>,
    pub(super) dragging: bool,
}

#[derive(Debug, Clone)]
pub(super) struct ToastUpsertOutcome {
    pub(super) id: ToastId,
    pub(super) cancel_auto: Option<TimerToken>,
    pub(super) schedule_auto: Option<(TimerToken, Duration)>,
    pub(super) evicted: Vec<ToastId>,
}

#[derive(Debug, Default)]
pub struct ToastStore {
    next_id: u64,
    by_window: HashMap<AppWindowId, Vec<ToastEntry>>,
    by_token: HashMap<TimerToken, ToastTimerRef>,
    max_toasts_by_window: HashMap<AppWindowId, usize>,
    swipe_by_window: HashMap<AppWindowId, ToastSwipeConfig>,
    toaster_state: HashMap<(AppWindowId, GlobalElementId), ToasterState>,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub(crate) struct ToasterState {
    pub(crate) hovered: bool,
    pub(crate) interacting: bool,
    pub(crate) hotkey_expanded: bool,
}

impl ToastStore {
    pub fn set_window_max_toasts(
        &mut self,
        window: AppWindowId,
        max_toasts: Option<usize>,
    ) -> bool {
        let max_toasts = max_toasts.unwrap_or(0);
        let prev = self.max_toasts_by_window.get(&window).copied().unwrap_or(0);
        if prev == max_toasts {
            return false;
        }
        if max_toasts == 0 {
            self.max_toasts_by_window.remove(&window);
        } else {
            self.max_toasts_by_window.insert(window, max_toasts);
        }
        true
    }

    pub(super) fn toasts_for_window(&self, window: AppWindowId) -> &[ToastEntry] {
        self.by_window
            .get(&window)
            .map(Vec::as_slice)
            .unwrap_or(&[])
    }

    fn max_toasts_for_window(&self, window: AppWindowId) -> Option<usize> {
        self.max_toasts_by_window.get(&window).copied()
    }

    pub fn set_window_swipe_config(
        &mut self,
        window: AppWindowId,
        direction: ToastSwipeDirection,
        threshold: Px,
    ) -> bool {
        let cfg = ToastSwipeConfig {
            direction,
            threshold: Px(threshold.0.max(1.0)),
            ..Default::default()
        };
        let prev = self.swipe_by_window.get(&window).copied();
        if prev == Some(cfg) {
            return false;
        }
        self.swipe_by_window.insert(window, cfg);
        true
    }

    pub fn set_window_swipe_config_ex(
        &mut self,
        window: AppWindowId,
        config: ToastSwipeConfig,
    ) -> bool {
        let prev = self.swipe_by_window.get(&window).copied();
        if prev == Some(config) {
            return false;
        }
        self.swipe_by_window.insert(window, config);
        true
    }

    fn swipe_config_for_window(&self, window: AppWindowId) -> ToastSwipeConfig {
        self.swipe_by_window
            .get(&window)
            .copied()
            .unwrap_or_default()
    }

    fn add_toast(
        &mut self,
        window: AppWindowId,
        request: ToastRequest,
        auto_close_token: Option<TimerToken>,
    ) -> ToastId {
        let wants_timer = request.duration.filter(|d| d.as_secs_f32() > 0.0);
        if self.next_id == 0 {
            self.next_id = 1;
        }
        let id = ToastId(self.next_id);
        self.next_id = self.next_id.saturating_add(1);

        if let Some(token) = auto_close_token {
            self.by_token.insert(
                token,
                ToastTimerRef {
                    window,
                    toast: id,
                    kind: ToastTimerKind::AutoClose,
                },
            );
        }

        self.by_window.entry(window).or_default().push(ToastEntry {
            id,
            toaster_id: request.toaster_id,
            title: request.title,
            description: request.description,
            duration: request.duration,
            auto_close_remaining: wants_timer,
            variant: request.variant,
            action: request.action,
            cancel: request.cancel,
            dismissible: request.dismissible,
            position: request.position,
            rich_colors: request.rich_colors,
            invert: request.invert,
            test_id: request.test_id,
            measured_height: None,
            open: true,
            auto_close_token,
            remove_token: None,
            drag_start: None,
            drag_offset: Point::new(Px(0.0), Px(0.0)),
            settle_from: None,
            dragging: false,
        });

        id
    }

    pub(super) fn upsert_toast(
        &mut self,
        window: AppWindowId,
        request: ToastRequest,
        auto_close_token: Option<TimerToken>,
    ) -> ToastUpsertOutcome {
        let wants_timer = request.duration.filter(|d| d.as_secs_f32() > 0.0);

        if let Some(id) = request.id {
            if let Some(toasts) = self.by_window.get_mut(&window) {
                if let Some(toast) = toasts
                    .iter_mut()
                    .find(|t| t.id == id && t.open && t.remove_token.is_none())
                {
                    let cancel_auto = toast.auto_close_token.take();
                    if let Some(token) = cancel_auto {
                        self.by_token.remove(&token);
                    }

                    toast.title = request.title;
                    toast.description = request.description;
                    toast.duration = request.duration;
                    toast.auto_close_remaining = wants_timer;
                    toast.variant = request.variant;
                    toast.action = request.action;
                    toast.cancel = request.cancel;
                    toast.dismissible = request.dismissible;
                    toast.toaster_id = request.toaster_id;
                    toast.position = request.position;
                    toast.rich_colors = request.rich_colors;
                    toast.invert = request.invert;
                    toast.test_id = request.test_id;
                    toast.drag_start = None;
                    toast.drag_offset = Point::new(Px(0.0), Px(0.0));
                    toast.settle_from = None;
                    toast.dragging = false;
                    toast.measured_height = None;

                    let schedule_auto = match (wants_timer, auto_close_token) {
                        (Some(after), Some(token)) => {
                            toast.auto_close_token = Some(token);
                            self.by_token.insert(
                                token,
                                ToastTimerRef {
                                    window,
                                    toast: id,
                                    kind: ToastTimerKind::AutoClose,
                                },
                            );
                            Some((token, auto_close_next_after(after)))
                        }
                        _ => None,
                    };

                    return ToastUpsertOutcome {
                        id,
                        cancel_auto,
                        schedule_auto,
                        evicted: Vec::new(),
                    };
                }
            }
        }

        let id = self.add_toast(window, request, auto_close_token);
        let schedule_auto = match (wants_timer, auto_close_token) {
            (Some(after), Some(token)) => Some((token, auto_close_next_after(after))),
            _ => None,
        };
        let evicted = self.evict_excess_toasts(window, id);

        ToastUpsertOutcome {
            id,
            cancel_auto: None,
            schedule_auto,
            evicted,
        }
    }

    fn evict_excess_toasts(&self, window: AppWindowId, keep: ToastId) -> Vec<ToastId> {
        let Some(max) = self.max_toasts_for_window(window) else {
            return Vec::new();
        };
        if max == 0 {
            return Vec::new();
        }

        let Some(toasts) = self.by_window.get(&window) else {
            return Vec::new();
        };

        let active: Vec<&ToastEntry> = toasts
            .iter()
            .filter(|t| t.open && t.remove_token.is_none())
            .collect();

        let mut need = active.len().saturating_sub(max);
        if need == 0 {
            return Vec::new();
        }

        let mut evicted = Vec::new();

        // Prefer evicting auto-closing toasts first; keep pinned toasts around when possible.
        for toast in &active {
            if need == 0 {
                break;
            }
            if toast.id == keep || toast.auto_close_remaining.is_none() {
                continue;
            }
            evicted.push(toast.id);
            need = need.saturating_sub(1);
        }

        for toast in &active {
            if need == 0 {
                break;
            }
            if toast.id == keep || toast.auto_close_remaining.is_some() {
                continue;
            }
            evicted.push(toast.id);
            need = need.saturating_sub(1);
        }
        evicted
    }

    fn remove_toast(&mut self, window: AppWindowId, id: ToastId) -> Option<ToastEntry> {
        let toasts = self.by_window.get_mut(&window)?;
        let idx = toasts.iter().position(|t| t.id == id)?;
        let entry = toasts.remove(idx);
        if let Some(token) = entry.auto_close_token {
            self.by_token.remove(&token);
        }
        if let Some(token) = entry.remove_token {
            self.by_token.remove(&token);
        }
        Some(entry)
    }

    pub(super) fn begin_close(
        &mut self,
        window: AppWindowId,
        id: ToastId,
        remove_token: TimerToken,
    ) -> Option<ToastClosePlan> {
        let toasts = self.by_window.get_mut(&window)?;
        let toast = toasts.iter_mut().find(|t| t.id == id)?;
        if toast.remove_token.is_some() {
            return Some(ToastClosePlan {
                cancel_auto: None,
                schedule_remove: None,
            });
        }

        toast.open = false;
        toast.auto_close_remaining = None;
        toast.drag_start = None;
        toast.drag_offset = Point::new(Px(0.0), Px(0.0));
        toast.settle_from = None;
        toast.dragging = false;
        let cancel_auto = toast.auto_close_token.take();
        if let Some(token) = cancel_auto {
            self.by_token.remove(&token);
        }

        toast.remove_token = Some(remove_token);
        self.by_token.insert(
            remove_token,
            ToastTimerRef {
                window,
                toast: id,
                kind: ToastTimerKind::RemoveAfterClose,
            },
        );

        Some(ToastClosePlan {
            cancel_auto,
            schedule_remove: Some(remove_token),
        })
    }

    pub(super) fn pause_auto_close(
        &mut self,
        window: AppWindowId,
        id: ToastId,
    ) -> Option<TimerToken> {
        let toasts = self.by_window.get_mut(&window)?;
        let toast = toasts.iter_mut().find(|t| t.id == id)?;
        let token = toast.auto_close_token.take()?;
        self.by_token.remove(&token);
        Some(token)
    }

    pub(super) fn resume_auto_close(
        &mut self,
        window: AppWindowId,
        id: ToastId,
        token: TimerToken,
    ) -> Option<Duration> {
        let toasts = self.by_window.get_mut(&window)?;
        let toast = toasts.iter_mut().find(|t| t.id == id)?;
        if !toast.open || toast.auto_close_token.is_some() || toast.remove_token.is_some() {
            return None;
        }
        let remaining = toast
            .auto_close_remaining
            .filter(|d| d.as_secs_f32() > 0.0)?;
        toast.auto_close_token = Some(token);
        self.by_token.insert(
            token,
            ToastTimerRef {
                window,
                toast: id,
                kind: ToastTimerKind::AutoClose,
            },
        );
        Some(auto_close_next_after(remaining))
    }

    pub(super) fn begin_drag(&mut self, window: AppWindowId, id: ToastId, start: Point) -> bool {
        let Some(toasts) = self.by_window.get_mut(&window) else {
            return false;
        };
        let Some(toast) = toasts.iter_mut().find(|t| t.id == id) else {
            return false;
        };
        if !toast.open || toast.remove_token.is_some() || !toast.dismissible {
            return false;
        }
        toast.drag_start = Some(start);
        toast.drag_offset = Point::new(Px(0.0), Px(0.0));
        toast.settle_from = None;
        toast.dragging = false;
        true
    }

    pub(super) fn clear_settle(&mut self, window: AppWindowId, id: ToastId) -> bool {
        let Some(toasts) = self.by_window.get_mut(&window) else {
            return false;
        };
        let Some(toast) = toasts.iter_mut().find(|t| t.id == id) else {
            return false;
        };
        if toast.settle_from.is_none() {
            return false;
        }
        toast.settle_from = None;
        true
    }

    fn toast_drag_offset(start: Point, position: Point, config: ToastSwipeConfig) -> Point {
        let dx = Px(position.x.0 - start.x.0);
        let dy = Px(position.y.0 - start.y.0);
        let max = config.max_drag.0.max(1.0);

        match config.direction {
            ToastSwipeDirection::Left => {
                let dx = Px(dx.0.clamp(-max, 0.0));
                Point::new(dx, Px(0.0))
            }
            ToastSwipeDirection::Right => {
                let dx = Px(dx.0.clamp(0.0, max));
                Point::new(dx, Px(0.0))
            }
            ToastSwipeDirection::Up => {
                let dy = Px(dy.0.clamp(-max, 0.0));
                Point::new(Px(0.0), dy)
            }
            ToastSwipeDirection::Down => {
                let dy = Px(dy.0.clamp(0.0, max));
                Point::new(Px(0.0), dy)
            }
        }
    }

    fn toast_drag_is_active(offset: Point, config: ToastSwipeConfig) -> bool {
        let px = match config.direction {
            ToastSwipeDirection::Left | ToastSwipeDirection::Right => offset.x.0.abs(),
            ToastSwipeDirection::Up | ToastSwipeDirection::Down => offset.y.0.abs(),
        };
        px >= config.dragging_threshold.0.max(0.0)
    }

    fn toast_drag_should_dismiss(offset: Point, config: ToastSwipeConfig) -> bool {
        let threshold = config.threshold.0.max(1.0);
        match config.direction {
            ToastSwipeDirection::Left => offset.x.0 <= -threshold,
            ToastSwipeDirection::Right => offset.x.0 >= threshold,
            ToastSwipeDirection::Up => offset.y.0 <= -threshold,
            ToastSwipeDirection::Down => offset.y.0 >= threshold,
        }
    }

    pub(super) fn drag_move(
        &mut self,
        window: AppWindowId,
        id: ToastId,
        position: Point,
    ) -> Option<ToastDragMove> {
        let cfg = self.swipe_config_for_window(window);
        let toasts = self.by_window.get_mut(&window)?;
        let toast = toasts.iter_mut().find(|t| t.id == id)?;
        let start = toast.drag_start?;
        if !toast.open || toast.remove_token.is_some() {
            return None;
        }

        let offset = Self::toast_drag_offset(start, position, cfg);
        let was_dragging = toast.dragging;
        if !toast.dragging && Self::toast_drag_is_active(offset, cfg) {
            toast.dragging = true;
        }
        toast.drag_offset = offset;

        Some(ToastDragMove {
            dragging: toast.dragging,
            capture_pointer: toast.dragging && !was_dragging,
        })
    }

    pub(super) fn end_drag(&mut self, window: AppWindowId, id: ToastId) -> Option<ToastDragEnd> {
        let cfg = self.swipe_config_for_window(window);
        let toasts = self.by_window.get_mut(&window)?;
        let toast = toasts.iter_mut().find(|t| t.id == id)?;
        if toast.drag_start.is_none() {
            return None;
        }

        let dismiss = toast.dragging && Self::toast_drag_should_dismiss(toast.drag_offset, cfg);
        let settle_from = (!dismiss && toast.dragging).then_some(toast.drag_offset);
        let result = ToastDragEnd {
            dragging: toast.dragging,
            dismiss,
        };
        toast.drag_start = None;
        toast.drag_offset = Point::new(Px(0.0), Px(0.0));
        toast.dragging = false;
        toast.settle_from = settle_from;
        Some(result)
    }

    pub(super) fn on_timer(
        &mut self,
        token: TimerToken,
        remove_token: TimerToken,
    ) -> ToastTimerOutcome {
        let Some(timer) = self.by_token.get(&token).copied() else {
            return ToastTimerOutcome::Noop;
        };

        match timer.kind {
            ToastTimerKind::AutoClose => {
                self.on_auto_close_tick(token, timer.window, timer.toast, remove_token)
            }
            ToastTimerKind::RemoveAfterClose => {
                self.by_token.remove(&token);
                let removed = self.remove_toast(timer.window, timer.toast).is_some();
                if removed {
                    ToastTimerOutcome::Removed {
                        window: timer.window,
                    }
                } else {
                    ToastTimerOutcome::Noop
                }
            }
        }
    }

    fn on_auto_close_tick(
        &mut self,
        token: TimerToken,
        window: AppWindowId,
        toast_id: ToastId,
        remove_token: TimerToken,
    ) -> ToastTimerOutcome {
        let Some(toasts) = self.by_window.get_mut(&window) else {
            self.by_token.remove(&token);
            return ToastTimerOutcome::Noop;
        };
        let Some(toast) = toasts.iter_mut().find(|t| t.id == toast_id) else {
            self.by_token.remove(&token);
            return ToastTimerOutcome::Noop;
        };

        if !toast.open || toast.remove_token.is_some() || toast.auto_close_token != Some(token) {
            self.by_token.remove(&token);
            return ToastTimerOutcome::Noop;
        }

        let Some(mut remaining) = toast.auto_close_remaining else {
            toast.auto_close_token = None;
            self.by_token.remove(&token);
            return ToastTimerOutcome::Noop;
        };

        let step = remaining.min(TOAST_AUTO_CLOSE_TICK);
        remaining = remaining.saturating_sub(step);
        toast.auto_close_remaining = (!remaining.is_zero()).then_some(remaining);

        if !remaining.is_zero() {
            ToastTimerOutcome::RescheduleAuto {
                window,
                token,
                after: auto_close_next_after(remaining),
            }
        } else {
            let plan = self.begin_close(window, toast_id, remove_token);
            let Some(plan) = plan else {
                return ToastTimerOutcome::Noop;
            };
            if plan.schedule_remove.is_some() {
                ToastTimerOutcome::BeganClose {
                    window,
                    remove_token,
                }
            } else {
                ToastTimerOutcome::Noop
            }
        }
    }

    pub fn set_toaster_hovered(
        &mut self,
        window: AppWindowId,
        toaster: GlobalElementId,
        hovered: bool,
    ) -> bool {
        let st = self.toaster_state.entry((window, toaster)).or_default();
        if st.hovered == hovered {
            return false;
        }
        st.hovered = hovered;
        true
    }

    pub fn set_toaster_interacting(
        &mut self,
        window: AppWindowId,
        toaster: GlobalElementId,
        interacting: bool,
    ) -> bool {
        let st = self.toaster_state.entry((window, toaster)).or_default();
        if st.interacting == interacting {
            return false;
        }
        st.interacting = interacting;
        true
    }

    pub fn set_toaster_hotkey_expanded(
        &mut self,
        window: AppWindowId,
        toaster: GlobalElementId,
        expanded: bool,
    ) -> bool {
        let st = self.toaster_state.entry((window, toaster)).or_default();
        if st.hotkey_expanded == expanded {
            return false;
        }
        st.hotkey_expanded = expanded;
        true
    }

    pub(crate) fn toaster_state(
        &self,
        window: AppWindowId,
        toaster: GlobalElementId,
    ) -> ToasterState {
        self.toaster_state
            .get(&(window, toaster))
            .copied()
            .unwrap_or_default()
    }

    pub fn set_toast_measured_height(
        &mut self,
        window: AppWindowId,
        id: ToastId,
        height: Px,
    ) -> bool {
        let Some(toasts) = self.by_window.get_mut(&window) else {
            return false;
        };
        let Some(toast) = toasts.iter_mut().find(|t| t.id == id) else {
            return false;
        };
        if toast.measured_height == Some(height) {
            return false;
        }
        toast.measured_height = Some(height);
        true
    }
}

fn auto_close_next_after(remaining: Duration) -> Duration {
    remaining.min(TOAST_AUTO_CLOSE_TICK)
}

#[derive(Debug, Clone, Copy)]
pub(super) struct ToastClosePlan {
    pub(super) cancel_auto: Option<TimerToken>,
    pub(super) schedule_remove: Option<TimerToken>,
}

#[derive(Debug, Clone, Copy)]
pub(super) struct ToastDragMove {
    pub(super) dragging: bool,
    pub(super) capture_pointer: bool,
}

#[derive(Debug, Clone, Copy)]
pub(super) struct ToastDragEnd {
    pub(super) dragging: bool,
    pub(super) dismiss: bool,
}

#[derive(Debug, Clone, Copy)]
pub(super) enum ToastTimerOutcome {
    Noop,
    RescheduleAuto {
        window: AppWindowId,
        token: TimerToken,
        after: Duration,
    },
    BeganClose {
        window: AppWindowId,
        remove_token: TimerToken,
    },
    Removed {
        window: AppWindowId,
    },
}

#[derive(Default)]
struct ToastService {
    store: Option<Model<ToastStore>>,
}

pub fn toast_store<H: UiHost>(app: &mut H) -> Model<ToastStore> {
    app.with_global_mut_untracked(ToastService::default, |svc, app| {
        svc.store
            .get_or_insert_with(|| app.models_mut().insert(ToastStore::default()))
            .clone()
    })
}

pub fn toast_action(
    host: &mut dyn fret_ui::action::UiActionHost,
    store: Model<ToastStore>,
    window: AppWindowId,
    request: ToastRequest,
) -> ToastId {
    let wants_timer = request.duration.filter(|d| d.as_secs_f32() > 0.0);
    let token = wants_timer.map(|_| host.next_timer_token());

    let outcome = host
        .models_mut()
        .update(&store, |st| st.upsert_toast(window, request, token))
        .ok();

    let Some(outcome) = outcome else {
        return ToastId(0);
    };

    if let Some(token) = outcome.cancel_auto {
        host.push_effect(Effect::CancelTimer { token });
    }

    if let Some((token, after)) = outcome.schedule_auto {
        host.push_effect(Effect::SetTimer {
            window: Some(window),
            token,
            after,
            repeat: None,
        });
    }

    for id in outcome.evicted {
        let remove_token = host.next_timer_token();
        let plan = host
            .models_mut()
            .update(&store, |st| st.begin_close(window, id, remove_token))
            .ok()
            .flatten();

        let Some(plan) = plan else {
            continue;
        };

        if let Some(token) = plan.cancel_auto {
            host.push_effect(Effect::CancelTimer { token });
        }

        if plan.schedule_remove.is_some() {
            host.push_effect(Effect::SetTimer {
                window: Some(window),
                token: remove_token,
                after: TOAST_CLOSE_DURATION,
                repeat: None,
            });
        }
    }

    host.request_redraw(window);
    outcome.id
}

pub fn dismiss_toast_action(
    host: &mut dyn fret_ui::action::UiActionHost,
    store: Model<ToastStore>,
    window: AppWindowId,
    id: ToastId,
) -> bool {
    let remove_token = host.next_timer_token();
    let plan = host
        .models_mut()
        .update(&store, |st| st.begin_close(window, id, remove_token))
        .ok();
    let Some(plan) = plan.flatten() else {
        return false;
    };

    if let Some(token) = plan.cancel_auto {
        host.push_effect(Effect::CancelTimer { token });
    }

    if plan.schedule_remove.is_some() {
        host.push_effect(Effect::SetTimer {
            window: Some(window),
            token: remove_token,
            after: TOAST_CLOSE_DURATION,
            repeat: None,
        });
    }

    host.request_redraw(window);
    true
}

#[derive(Default)]
struct ToastAsyncQueue {
    inner: Arc<Mutex<Vec<ToastAsyncMsg>>>,
}

/// Thread-safe handle for scheduling toast upserts/dismissals from background work.
///
/// Messages are applied on the UI thread during the window overlays render pass.
#[derive(Clone, Debug)]
pub struct ToastAsyncQueueHandle {
    inner: Arc<Mutex<Vec<ToastAsyncMsg>>>,
}

#[derive(Clone, Debug)]
pub enum ToastAsyncMsg {
    Upsert {
        window: AppWindowId,
        request: ToastRequest,
    },
    Dismiss {
        window: AppWindowId,
        id: ToastId,
    },
}

impl ToastAsyncQueueHandle {
    pub fn push(&self, msg: ToastAsyncMsg) {
        let mut lock = self.inner.lock().unwrap_or_else(|p| p.into_inner());
        lock.push(msg);
    }

    pub fn upsert(&self, window: AppWindowId, request: ToastRequest) {
        self.push(ToastAsyncMsg::Upsert { window, request });
    }

    pub fn dismiss(&self, window: AppWindowId, id: ToastId) {
        self.push(ToastAsyncMsg::Dismiss { window, id });
    }
}

pub fn toast_async_queue<H: UiHost>(app: &mut H) -> ToastAsyncQueueHandle {
    app.with_global_mut_untracked(ToastAsyncQueue::default, |queue, _app| {
        ToastAsyncQueueHandle {
            inner: queue.inner.clone(),
        }
    })
}

pub(super) fn drain_toast_async_queue<H: UiHost>(app: &mut H) {
    let msgs = app.with_global_mut_untracked(ToastAsyncQueue::default, |queue, _app| {
        let mut lock = queue.inner.lock().unwrap_or_else(|p| p.into_inner());
        std::mem::take(&mut *lock)
    });

    if msgs.is_empty() {
        return;
    }

    let store = toast_store(app);
    let mut host = UiActionHostAdapter { app };

    for msg in msgs {
        match msg {
            ToastAsyncMsg::Upsert { window, request } => {
                let _ = toast_action(&mut host, store.clone(), window, request);
            }
            ToastAsyncMsg::Dismiss { window, id } => {
                let _ = dismiss_toast_action(&mut host, store.clone(), window, id);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn toast_pause_resume_and_removal_flow() {
        let window = AppWindowId::default();
        let mut store = ToastStore::default();

        let request = ToastRequest::new("Hello").duration(Some(Duration::from_millis(250)));
        let id = store.add_toast(window, request, Some(TimerToken(1)));

        let paused = store.pause_auto_close(window, id);
        assert_eq!(paused, Some(TimerToken(1)));

        let resumed = store.resume_auto_close(window, id, TimerToken(2));
        assert_eq!(resumed, Some(Duration::from_millis(100)));

        let outcome = store.on_timer(TimerToken(2), TimerToken(3));
        match outcome {
            ToastTimerOutcome::RescheduleAuto {
                window: w, after, ..
            } => {
                assert_eq!(w, window);
                assert_eq!(after, Duration::from_millis(100));
            }
            _ => panic!("expected RescheduleAuto"),
        }

        let paused = store.pause_auto_close(window, id);
        assert_eq!(paused, Some(TimerToken(2)));

        let resumed = store.resume_auto_close(window, id, TimerToken(4));
        assert_eq!(resumed, Some(Duration::from_millis(100)));

        let outcome = store.on_timer(TimerToken(4), TimerToken(5));
        match outcome {
            ToastTimerOutcome::RescheduleAuto {
                window: w, after, ..
            } => {
                assert_eq!(w, window);
                assert_eq!(after, Duration::from_millis(50));
            }
            _ => panic!("expected RescheduleAuto"),
        }

        let outcome = store.on_timer(TimerToken(4), TimerToken(6));
        match outcome {
            ToastTimerOutcome::BeganClose { window: w, .. } => assert_eq!(w, window),
            _ => panic!("expected BeganClose"),
        }

        let outcome = store.on_timer(TimerToken(6), TimerToken(7));
        match outcome {
            ToastTimerOutcome::Removed { window: w } => assert_eq!(w, window),
            _ => panic!("expected Removed"),
        }

        assert!(store.toasts_for_window(window).is_empty());
    }

    #[test]
    fn toast_drag_sets_and_resets_offset() {
        let window = AppWindowId::default();
        let mut store = ToastStore::default();

        let request = ToastRequest::new("Drag").duration(None);
        let id = store.add_toast(window, request, None);

        store.set_window_swipe_config(window, ToastSwipeDirection::Right, Px(50.0));
        assert!(store.begin_drag(window, id, Point::new(Px(10.0), Px(10.0))));

        let moved = store.drag_move(window, id, Point::new(Px(30.0), Px(10.0)));
        assert!(moved.is_some());
        assert!(store.toasts_for_window(window)[0].drag_offset.x.0 > 0.0);

        let end = store.end_drag(window, id);
        assert!(end.is_some());
        assert_eq!(
            store.toasts_for_window(window)[0].drag_offset,
            Point::new(Px(0.0), Px(0.0))
        );
        assert_eq!(store.toasts_for_window(window)[0].drag_start, None);
    }

    #[test]
    fn toast_drag_cancel_records_settle_offset() {
        let window = AppWindowId::default();
        let mut store = ToastStore::default();

        let id = store.add_toast(window, ToastRequest::new("Cancel").duration(None), None);
        store.set_window_swipe_config_ex(
            window,
            ToastSwipeConfig {
                direction: ToastSwipeDirection::Right,
                threshold: Px(50.0),
                max_drag: Px(240.0),
                dragging_threshold: Px(0.0),
            },
        );

        assert!(store.begin_drag(window, id, Point::new(Px(10.0), Px(10.0))));
        assert!(
            store
                .drag_move(window, id, Point::new(Px(30.0), Px(10.0)))
                .is_some()
        );
        let end = store.end_drag(window, id).expect("end");
        assert!(end.dragging);
        assert!(!end.dismiss, "expected below threshold to not dismiss");

        let toast = store
            .toasts_for_window(window)
            .iter()
            .find(|t| t.id == id)
            .expect("toast entry");
        assert_eq!(toast.drag_offset, Point::new(Px(0.0), Px(0.0)));
        assert_eq!(toast.settle_from, Some(Point::new(Px(20.0), Px(0.0))));

        assert!(store.clear_settle(window, id));
        let toast = store
            .toasts_for_window(window)
            .iter()
            .find(|t| t.id == id)
            .expect("toast entry");
        assert_eq!(toast.settle_from, None);
        assert!(!store.clear_settle(window, id));
    }

    #[test]
    fn toast_drag_dismiss_does_not_record_settle_offset() {
        let window = AppWindowId::default();
        let mut store = ToastStore::default();

        let id = store.add_toast(window, ToastRequest::new("Dismiss").duration(None), None);
        store.set_window_swipe_config_ex(
            window,
            ToastSwipeConfig {
                direction: ToastSwipeDirection::Right,
                threshold: Px(50.0),
                max_drag: Px(240.0),
                dragging_threshold: Px(0.0),
            },
        );

        assert!(store.begin_drag(window, id, Point::new(Px(10.0), Px(10.0))));
        assert!(
            store
                .drag_move(window, id, Point::new(Px(80.0), Px(10.0)))
                .is_some()
        );
        let end = store.end_drag(window, id).expect("end");
        assert!(end.dragging);
        assert!(end.dismiss);

        let toast = store
            .toasts_for_window(window)
            .iter()
            .find(|t| t.id == id)
            .expect("toast entry");
        assert_eq!(toast.settle_from, None);
    }

    #[test]
    fn toast_drag_dismiss_uses_swipe_config_direction_and_threshold() {
        let window = AppWindowId::default();
        let mut store = ToastStore::default();

        let id = store.add_toast(window, ToastRequest::new("Swipe").duration(None), None);

        store.set_window_swipe_config(window, ToastSwipeDirection::Right, Px(50.0));
        assert!(store.begin_drag(window, id, Point::new(Px(10.0), Px(10.0))));
        assert!(
            store
                .drag_move(window, id, Point::new(Px(70.0), Px(10.0)))
                .is_some()
        );
        let end = store.end_drag(window, id).expect("end");
        assert!(end.dragging);
        assert!(end.dismiss, "expected swipe-right to dismiss");

        let id = store.add_toast(window, ToastRequest::new("Swipe2").duration(None), None);
        store.set_window_swipe_config(window, ToastSwipeDirection::Left, Px(50.0));
        assert!(store.begin_drag(window, id, Point::new(Px(60.0), Px(10.0))));
        assert!(
            store
                .drag_move(window, id, Point::new(Px(20.0), Px(10.0)))
                .is_some()
        );
        let end = store.end_drag(window, id).expect("end");
        assert!(end.dragging);
        assert!(
            !end.dismiss,
            "expected swipe-left below threshold to not dismiss"
        );

        let id = store.add_toast(window, ToastRequest::new("Swipe3").duration(None), None);
        store.set_window_swipe_config(window, ToastSwipeDirection::Up, Px(50.0));
        assert!(store.begin_drag(window, id, Point::new(Px(10.0), Px(60.0))));
        assert!(
            store
                .drag_move(window, id, Point::new(Px(10.0), Px(0.0)))
                .is_some()
        );
        assert!(
            store
                .toasts_for_window(window)
                .iter()
                .find(|t| t.id == id)
                .expect("toast entry")
                .drag_offset
                .y
                .0
                < 0.0
        );
        let end = store.end_drag(window, id).expect("end");
        assert!(end.dragging);
        assert!(end.dismiss, "expected swipe-up to dismiss");

        let id = store.add_toast(window, ToastRequest::new("Swipe4").duration(None), None);
        store.set_window_swipe_config(window, ToastSwipeDirection::Down, Px(50.0));
        assert!(store.begin_drag(window, id, Point::new(Px(10.0), Px(10.0))));
        assert!(
            store
                .drag_move(window, id, Point::new(Px(10.0), Px(70.0)))
                .is_some()
        );
        assert!(
            store
                .toasts_for_window(window)
                .iter()
                .find(|t| t.id == id)
                .expect("toast entry")
                .drag_offset
                .y
                .0
                > 0.0
        );
        let end = store.end_drag(window, id).expect("end");
        assert!(end.dragging);
        assert!(end.dismiss, "expected swipe-down to dismiss");
    }

    #[test]
    fn toast_drag_clamps_to_max_drag_for_swipe_axis() {
        let window = AppWindowId::default();
        let mut store = ToastStore::default();

        let id = store.add_toast(window, ToastRequest::new("Clamp").duration(None), None);
        store.set_window_swipe_config_ex(
            window,
            ToastSwipeConfig {
                direction: ToastSwipeDirection::Right,
                threshold: Px(50.0),
                max_drag: Px(16.0),
                dragging_threshold: Px(0.0),
            },
        );

        assert!(store.begin_drag(window, id, Point::new(Px(10.0), Px(10.0))));
        assert!(
            store
                .drag_move(window, id, Point::new(Px(200.0), Px(200.0)))
                .is_some()
        );

        let toast = store
            .toasts_for_window(window)
            .iter()
            .find(|t| t.id == id)
            .expect("toast entry");
        assert_eq!(toast.drag_offset.x, Px(16.0));
        assert_eq!(toast.drag_offset.y, Px(0.0));
    }

    #[test]
    fn toast_dragging_threshold_controls_capture_arming() {
        let window = AppWindowId::default();
        let mut store = ToastStore::default();

        let id = store.add_toast(window, ToastRequest::new("Threshold").duration(None), None);
        store.set_window_swipe_config_ex(
            window,
            ToastSwipeConfig {
                direction: ToastSwipeDirection::Right,
                threshold: Px(50.0),
                max_drag: Px(240.0),
                dragging_threshold: Px(40.0),
            },
        );

        assert!(store.begin_drag(window, id, Point::new(Px(10.0), Px(10.0))));
        let moved = store.drag_move(window, id, Point::new(Px(45.0), Px(10.0)));
        assert!(
            moved.is_some_and(|m| !m.dragging),
            "expected below dragging threshold"
        );

        let moved = store.drag_move(window, id, Point::new(Px(55.0), Px(10.0)));
        assert!(
            moved.is_some_and(|m| m.dragging && m.capture_pointer),
            "expected to arm pointer capture once dragging begins"
        );
    }

    #[test]
    fn toast_upsert_updates_existing_entry_and_resets_timer() {
        let window = AppWindowId::default();
        let mut store = ToastStore::default();

        let out0 = store.upsert_toast(
            window,
            ToastRequest::new("Loading")
                .variant(ToastVariant::Loading)
                .duration(None),
            None,
        );
        let id = out0.id;

        let out1 = store.upsert_toast(
            window,
            ToastRequest::new("Done")
                .id(id)
                .variant(ToastVariant::Success)
                .duration(Some(Duration::from_secs(2)))
                .action(ToastAction {
                    label: Arc::from("Undo"),
                    command: CommandId::from("toast.undo"),
                })
                .cancel(ToastAction {
                    label: Arc::from("Cancel"),
                    command: CommandId::from("toast.cancel"),
                }),
            Some(TimerToken(10)),
        );
        assert_eq!(out1.id, id);
        assert_eq!(out1.cancel_auto, None);
        assert_eq!(
            out1.schedule_auto,
            Some((TimerToken(10), TOAST_AUTO_CLOSE_TICK))
        );

        let toast = store.toasts_for_window(window)[0].clone();
        assert_eq!(toast.id, id);
        assert_eq!(toast.title.as_ref(), "Done");
        assert_eq!(toast.variant, ToastVariant::Success);
        assert_eq!(toast.auto_close_token, Some(TimerToken(10)));
        assert_eq!(
            toast.action.as_ref().map(|a| a.label.as_ref()),
            Some("Undo")
        );
        assert_eq!(
            toast.cancel.as_ref().map(|a| a.label.as_ref()),
            Some("Cancel")
        );
    }

    #[test]
    fn toast_upsert_noops_swipe_for_non_dismissible_toasts() {
        let window = AppWindowId::default();
        let mut store = ToastStore::default();

        let id = store.add_toast(
            window,
            ToastRequest::new("Pinned")
                .duration(None)
                .dismissible(false),
            None,
        );

        assert!(!store.begin_drag(window, id, Point::new(Px(10.0), Px(10.0))));
    }

    #[test]
    fn toast_max_toasts_evicts_oldest_open_toasts() {
        let window = AppWindowId::default();
        let mut store = ToastStore::default();
        store.set_window_max_toasts(window, Some(2));

        let out0 = store.upsert_toast(window, ToastRequest::new("A").duration(None), None);
        let out1 = store.upsert_toast(window, ToastRequest::new("B").duration(None), None);
        let out2 = store.upsert_toast(window, ToastRequest::new("C").duration(None), None);

        assert_eq!(out0.evicted, Vec::new());
        assert_eq!(out1.evicted, Vec::new());
        assert_eq!(out2.evicted, vec![out0.id]);
    }

    #[test]
    fn toast_max_toasts_prefers_evicting_auto_closing_toasts_over_pinned() {
        let window = AppWindowId::default();
        let mut store = ToastStore::default();
        store.set_window_max_toasts(window, Some(2));

        let pinned = store.upsert_toast(window, ToastRequest::new("Pinned").duration(None), None);
        let auto0 = store.upsert_toast(
            window,
            ToastRequest::new("Auto0").duration(Some(Duration::from_secs(3))),
            None,
        );
        let auto1 = store.upsert_toast(
            window,
            ToastRequest::new("Auto1").duration(Some(Duration::from_secs(3))),
            None,
        );

        assert_eq!(pinned.evicted, Vec::new());
        assert_eq!(auto0.evicted, Vec::new());
        assert_eq!(auto1.evicted, vec![auto0.id]);
    }

    #[test]
    fn toast_max_toasts_evicts_pinned_when_all_are_pinned() {
        let window = AppWindowId::default();
        let mut store = ToastStore::default();
        store.set_window_max_toasts(window, Some(1));

        let a = store.upsert_toast(window, ToastRequest::new("A").duration(None), None);
        let b = store.upsert_toast(window, ToastRequest::new("B").duration(None), None);

        assert_eq!(a.evicted, Vec::new());
        assert_eq!(b.evicted, vec![a.id]);
    }
}
