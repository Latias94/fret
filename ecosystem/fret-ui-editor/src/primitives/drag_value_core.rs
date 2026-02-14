//! Drag-to-edit (scrub) primitive for numeric values.
//!
//! This is an editor-grade "hand feel" primitive:
//! - pointer down begins an edit session and captures the pre-edit value,
//! - pointer move scrubs horizontally with modifier-based multipliers,
//! - pointer up commits,
//! - Escape cancels to the pre-edit value.

use std::sync::{Arc, Mutex};

use fret_core::{KeyCode, Modifiers, MouseButton, Point, PointerId, Px};
use fret_ui::action::{
    ActionCx, PressablePointerDownResult, PressablePointerUpResult, UiActionHost,
};
use fret_ui::element::{AnyElement, LayoutStyle, Length, PressableA11y, PressableProps};
use fret_ui::{ElementContext, Theme, UiHost};

use super::{EditSession, EditorDensity, EditorTokenKeys};

pub trait DragValueScalar: Copy + 'static {
    fn to_f64(self) -> f64;
    fn from_f64(v: f64) -> Self;
}

impl DragValueScalar for f32 {
    fn to_f64(self) -> f64 {
        self as f64
    }

    fn from_f64(v: f64) -> Self {
        v as f32
    }
}

impl DragValueScalar for f64 {
    fn to_f64(self) -> f64 {
        self
    }

    fn from_f64(v: f64) -> Self {
        v
    }
}

impl DragValueScalar for i32 {
    fn to_f64(self) -> f64 {
        self as f64
    }

    fn from_f64(v: f64) -> Self {
        v.round().clamp(i32::MIN as f64, i32::MAX as f64) as i32
    }
}

#[derive(Debug, Clone, Copy)]
pub struct DragValueCoreOptions {
    pub layout: LayoutStyle,
    pub enabled: bool,
    pub scrub_on_double_click: bool,
    pub drag_threshold: Px,
    pub scrub_speed: f64,
    pub slow_multiplier: f64,
    pub fast_multiplier: f64,
}

impl Default for DragValueCoreOptions {
    fn default() -> Self {
        Self {
            layout: LayoutStyle {
                size: fret_ui::element::SizeStyle {
                    width: Length::Fill,
                    height: Length::Auto,
                    ..Default::default()
                },
                ..Default::default()
            },
            enabled: true,
            scrub_on_double_click: true,
            drag_threshold: Px(4.0),
            scrub_speed: 0.02,
            slow_multiplier: 0.1,
            fast_multiplier: 10.0,
        }
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct DragValueCoreResponse {
    pub dragging: bool,
    pub canceled: bool,
    pub committed: bool,
}

#[derive(Debug)]
struct DragState<T> {
    session: EditSession<T>,
    armed: bool,
    dragging: bool,
    pointer_id: Option<PointerId>,
    down_pos: Point,
    start_x: f64,
    start_value: T,
}

impl<T: Copy + Default> Default for DragState<T> {
    fn default() -> Self {
        Self {
            session: EditSession::default(),
            armed: false,
            dragging: false,
            pointer_id: None,
            down_pos: Point::new(Px(0.0), Px(0.0)),
            start_x: 0.0,
            start_value: T::default(),
        }
    }
}

#[derive(Clone)]
pub struct DragValueCore<T> {
    value: T,
    on_change_live: Arc<dyn Fn(&mut dyn UiActionHost, ActionCx, T) + 'static>,
    on_commit: Option<Arc<dyn Fn(&mut dyn UiActionHost, ActionCx) + 'static>>,
    on_cancel: Option<Arc<dyn Fn(&mut dyn UiActionHost, ActionCx) + 'static>>,
    a11y_label: Option<Arc<str>>,
    options: DragValueCoreOptions,
}

impl<T> DragValueCore<T>
where
    T: DragValueScalar + Default,
{
    pub fn new(
        value: T,
        on_change_live: Arc<dyn Fn(&mut dyn UiActionHost, ActionCx, T) + 'static>,
    ) -> Self {
        Self {
            value,
            on_change_live,
            on_commit: None,
            on_cancel: None,
            a11y_label: None,
            options: DragValueCoreOptions::default(),
        }
    }

    pub fn on_commit(
        mut self,
        on_commit: Option<Arc<dyn Fn(&mut dyn UiActionHost, ActionCx) + 'static>>,
    ) -> Self {
        self.on_commit = on_commit;
        self
    }

    pub fn on_cancel(
        mut self,
        on_cancel: Option<Arc<dyn Fn(&mut dyn UiActionHost, ActionCx) + 'static>>,
    ) -> Self {
        self.on_cancel = on_cancel;
        self
    }

    pub fn a11y_label(mut self, label: impl Into<Arc<str>>) -> Self {
        self.a11y_label = Some(label.into());
        self
    }

    pub fn options(mut self, options: DragValueCoreOptions) -> Self {
        self.options = options;
        self
    }

    #[track_caller]
    pub fn into_element<H: UiHost>(
        self,
        cx: &mut ElementContext<'_, H>,
        children: impl FnOnce(&mut ElementContext<'_, H>, DragValueCoreResponse) -> Vec<AnyElement>,
    ) -> AnyElement {
        let theme = Theme::global(&*cx.app);
        let density = EditorDensity::resolve(theme);

        let opts = resolve_options(theme, self.options);
        let state: Arc<Mutex<DragState<T>>> = cx.with_state(
            || Arc::new(Mutex::new(DragState::<T>::default())),
            |s| s.clone(),
        );

        if !opts.enabled {
            let mut st = state.lock().unwrap_or_else(|e| e.into_inner());
            st.armed = false;
            st.dragging = false;
            st.pointer_id = None;
            let _ = st.session.commit();
        }

        let response = {
            let st = state.lock().unwrap_or_else(|e| e.into_inner());
            DragValueCoreResponse {
                dragging: st.dragging,
                ..Default::default()
            }
        };

        let on_change_live = self.on_change_live.clone();
        let on_commit = self.on_commit.clone();
        let on_cancel = self.on_cancel.clone();

        let enabled = opts.enabled;
        let a11y_label = self.a11y_label.clone();
        let value = self.value;

        let mut layout = opts.layout;
        if layout.size.min_height.is_none() {
            layout.size.min_height = Some(density.row_height);
        }

        let pressable = cx.pressable(
            PressableProps {
                enabled,
                layout,
                a11y: PressableA11y {
                    label: a11y_label,
                    ..Default::default()
                },
                ..Default::default()
            },
            move |cx, _st| {
                let state_for_down = state.clone();
                cx.pressable_add_on_pointer_down(Arc::new(move |_host, _action_cx, down| {
                    if down.button != MouseButton::Left {
                        return PressablePointerDownResult::Continue;
                    }

                    if !opts.scrub_on_double_click && down.click_count >= 2 {
                        return PressablePointerDownResult::SkipDefaultAndStopPropagation;
                    }

                    let mut st = state_for_down.lock().unwrap_or_else(|e| e.into_inner());
                    st.session.begin(value);
                    st.armed = true;
                    st.dragging = false;
                    st.pointer_id = Some(down.pointer_id);
                    st.down_pos = down.position_local;
                    st.start_x = down.position_local.x.0 as f64;
                    st.start_value = value;
                    PressablePointerDownResult::Continue
                }));

                let state_for_move = state.clone();
                let on_change_live_for_move = on_change_live.clone();
                let on_commit_for_move = on_commit.clone();
                let on_cancel_for_move = on_cancel.clone();
                cx.pressable_add_on_pointer_move(Arc::new(move |host, action_cx, mv| {
                    enum MoveAction<T> {
                        None,
                        Live(T),
                        Commit,
                        Cancel(Option<T>),
                    }

                    let action = {
                        let mut st = state_for_move.lock().unwrap_or_else(|e| e.into_inner());
                        if !st.armed || st.pointer_id != Some(mv.pointer_id) {
                            return false;
                        }

                        // Best-effort cleanup for unexpected end-of-stream:
                        // If we are armed but the runtime reports no pressed left button,
                        // treat it like "pointer up/cancel" to avoid stuck sessions.
                        if !mv.buttons.left {
                            let was_dragging = st.dragging;
                            st.armed = false;
                            st.dragging = false;
                            st.pointer_id = None;
                            if st.session.is_active() {
                                if was_dragging {
                                    let _ = st.session.commit();
                                    MoveAction::Commit
                                } else {
                                    let pre = st.session.cancel();
                                    MoveAction::Cancel(pre)
                                }
                            } else {
                                MoveAction::None
                            }
                        } else {
                            if !st.dragging {
                                let dx = mv.position_local.x.0 - st.down_pos.x.0;
                                let dy = mv.position_local.y.0 - st.down_pos.y.0;
                                let dist2 = (dx as f64) * (dx as f64) + (dy as f64) * (dy as f64);
                                let th = opts.drag_threshold.0 as f64;
                                if dist2 < th * th {
                                    return false;
                                }

                                st.dragging = true;
                                // Reset the delta origin when crossing the threshold to avoid an initial jump.
                                st.start_x = mv.position_local.x.0 as f64;
                                st.down_pos = mv.position_local;
                            }

                            let delta_x = mv.position_local.x.0 as f64 - st.start_x;
                            let multiplier = resolve_multiplier(
                                mv.modifiers,
                                opts.slow_multiplier,
                                opts.fast_multiplier,
                            );
                            let delta = delta_x * opts.scrub_speed * multiplier;
                            let next = T::from_f64(st.start_value.to_f64() + delta);
                            MoveAction::Live(next)
                        }
                    };

                    match action {
                        MoveAction::None => false,
                        MoveAction::Live(next) => {
                            (on_change_live_for_move)(host, action_cx, next);
                            true
                        }
                        MoveAction::Commit => {
                            host.release_pointer_capture();
                            if let Some(cb) = on_commit_for_move.as_ref() {
                                cb(host, action_cx);
                            }
                            host.request_redraw(action_cx.window);
                            true
                        }
                        MoveAction::Cancel(pre) => {
                            host.release_pointer_capture();
                            if let Some(pre) = pre {
                                (on_change_live_for_move)(host, action_cx, pre);
                            }
                            if let Some(cb) = on_cancel_for_move.as_ref() {
                                cb(host, action_cx);
                            }
                            host.request_redraw(action_cx.window);
                            true
                        }
                    }
                }));

                let state_for_up = state.clone();
                let on_commit_for_up = on_commit.clone();
                cx.pressable_add_on_pointer_up(Arc::new(move |host, action_cx, up| {
                    if up.button != MouseButton::Left {
                        return PressablePointerUpResult::Continue;
                    }

                    let mut st = state_for_up.lock().unwrap_or_else(|e| e.into_inner());
                    if st.pointer_id.is_some() && st.pointer_id != Some(up.pointer_id) {
                        return PressablePointerUpResult::Continue;
                    }
                    let was_dragging = st.dragging;
                    st.armed = false;
                    st.dragging = false;
                    st.pointer_id = None;
                    if st.session.is_active() {
                        st.session.commit();
                    }
                    if was_dragging && let Some(cb) = on_commit_for_up.as_ref() {
                        cb(host, action_cx);
                    }
                    PressablePointerUpResult::Continue
                }));

                let state_for_key = state.clone();
                let on_cancel_for_key = on_cancel.clone();
                cx.key_add_on_key_down_capture_for(
                    cx.root_id(),
                    Arc::new(move |host, action_cx, key| {
                        if key.key != KeyCode::Escape {
                            return false;
                        }

                        let mut st = state_for_key.lock().unwrap_or_else(|e| e.into_inner());
                        if !st.session.is_active() {
                            return false;
                        }

                        st.armed = false;
                        st.dragging = false;
                        st.pointer_id = None;
                        if let Some(pre) = st.session.cancel() {
                            (on_change_live)(host, action_cx, pre);
                        }
                        if let Some(cb) = on_cancel_for_key.as_ref() {
                            cb(host, action_cx);
                        }
                        true
                    }),
                );

                children(cx, response)
            },
        );

        pressable
    }
}

fn resolve_multiplier(mods: Modifiers, slow: f64, fast: f64) -> f64 {
    let mut out = 1.0;
    if mods.shift {
        out *= slow;
    }
    if mods.alt {
        out *= fast;
    }
    out
}

fn resolve_options(theme: &Theme, mut opts: DragValueCoreOptions) -> DragValueCoreOptions {
    let scrub_speed = theme
        .metric_by_key(EditorTokenKeys::NUMERIC_SCRUB_SPEED)
        .map(|m| m.0 as f64)
        .unwrap_or(opts.scrub_speed);
    let slow_multiplier = theme
        .metric_by_key(EditorTokenKeys::NUMERIC_SCRUB_SLOW_MULTIPLIER)
        .map(|m| m.0 as f64)
        .unwrap_or(opts.slow_multiplier);
    let fast_multiplier = theme
        .metric_by_key(EditorTokenKeys::NUMERIC_SCRUB_FAST_MULTIPLIER)
        .map(|m| m.0 as f64)
        .unwrap_or(opts.fast_multiplier);
    let drag_threshold = theme
        .metric_by_key(EditorTokenKeys::NUMERIC_SCRUB_DRAG_THRESHOLD)
        .unwrap_or(opts.drag_threshold);

    if !scrub_speed.is_finite() {
        opts.scrub_speed = 0.02;
    } else {
        opts.scrub_speed = scrub_speed.max(0.0);
    }
    opts.slow_multiplier = if slow_multiplier.is_finite() {
        slow_multiplier.max(0.0)
    } else {
        0.1
    };
    opts.fast_multiplier = if fast_multiplier.is_finite() {
        fast_multiplier.max(0.0)
    } else {
        10.0
    };
    opts.drag_threshold = if drag_threshold.0.is_finite() {
        Px(drag_threshold.0.max(0.0))
    } else {
        Px(4.0)
    };
    opts
}
