use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use fret_core::{AppWindowId, TimerToken};
use fret_runtime::{CommandId, Effect, Model};
use fret_ui::UiHost;

pub(super) const TOAST_CLOSE_DURATION: Duration = Duration::from_millis(200);

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

#[derive(Debug, Clone)]
pub struct ToastAction {
    pub label: Arc<str>,
    pub command: CommandId,
}

#[derive(Debug, Clone)]
pub struct ToastRequest {
    pub title: Arc<str>,
    pub description: Option<Arc<str>>,
    pub duration: Option<Duration>,
    pub variant: ToastVariant,
    pub action: Option<ToastAction>,
    pub dismissible: bool,
}

impl ToastRequest {
    pub fn new(title: impl Into<Arc<str>>) -> Self {
        Self {
            title: title.into(),
            description: None,
            duration: Some(Duration::from_secs(3)),
            variant: ToastVariant::default(),
            action: None,
            dismissible: true,
        }
    }

    pub fn description(mut self, description: impl Into<Arc<str>>) -> Self {
        self.description = Some(description.into());
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

    pub fn dismissible(mut self, dismissible: bool) -> Self {
        self.dismissible = dismissible;
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
    pub(super) title: Arc<str>,
    pub(super) description: Option<Arc<str>>,
    pub(super) variant: ToastVariant,
    pub(super) action: Option<ToastAction>,
    pub(super) dismissible: bool,
    pub(super) open: bool,
    pub(super) auto_close_token: Option<TimerToken>,
    pub(super) remove_token: Option<TimerToken>,
}

#[derive(Debug, Default)]
pub struct ToastStore {
    next_id: u64,
    by_window: HashMap<AppWindowId, Vec<ToastEntry>>,
    by_token: HashMap<TimerToken, ToastTimerRef>,
}

impl ToastStore {
    pub(super) fn toasts_for_window(&self, window: AppWindowId) -> &[ToastEntry] {
        self.by_window
            .get(&window)
            .map(Vec::as_slice)
            .unwrap_or(&[])
    }

    fn add_toast(
        &mut self,
        window: AppWindowId,
        request: ToastRequest,
        auto_close_token: Option<TimerToken>,
    ) -> ToastId {
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
            title: request.title,
            description: request.description,
            variant: request.variant,
            action: request.action,
            dismissible: request.dismissible,
            open: true,
            auto_close_token,
            remove_token: None,
        });

        id
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

    pub(super) fn on_timer(
        &mut self,
        token: TimerToken,
        remove_token: TimerToken,
    ) -> ToastTimerOutcome {
        let Some(timer) = self.by_token.remove(&token) else {
            return ToastTimerOutcome::Noop;
        };

        match timer.kind {
            ToastTimerKind::AutoClose => {
                let plan = self.begin_close(timer.window, timer.toast, remove_token);
                let Some(plan) = plan else {
                    return ToastTimerOutcome::Noop;
                };
                if plan.schedule_remove.is_some() {
                    ToastTimerOutcome::BeganClose {
                        window: timer.window,
                        remove_token,
                    }
                } else {
                    ToastTimerOutcome::Noop
                }
            }
            ToastTimerKind::RemoveAfterClose => {
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
}

#[derive(Debug, Clone, Copy)]
pub(super) struct ToastClosePlan {
    pub(super) cancel_auto: Option<TimerToken>,
    pub(super) schedule_remove: Option<TimerToken>,
}

#[derive(Debug, Clone, Copy)]
pub(super) enum ToastTimerOutcome {
    Noop,
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
    app.with_global_mut(ToastService::default, |svc, app| {
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
    let token = request
        .duration
        .filter(|d| d.as_secs_f32() > 0.0)
        .map(|after| {
            let token = host.next_timer_token();
            host.push_effect(Effect::SetTimer {
                window: Some(window),
                token,
                after,
                repeat: None,
            });
            token
        });

    let result = host
        .models_mut()
        .update(&store, |st| st.add_toast(window, request, token));

    let Ok(id) = result else {
        if let Some(token) = token {
            host.push_effect(Effect::CancelTimer { token });
        }
        return ToastId(0);
    };

    host.request_redraw(window);
    id
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
