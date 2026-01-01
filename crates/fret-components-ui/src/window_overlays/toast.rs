use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use fret_core::{AppWindowId, TimerToken};
use fret_runtime::{CommandId, Effect, Model};
use fret_ui::UiHost;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum ToastPosition {
    TopLeft,
    TopRight,
    BottomLeft,
    #[default]
    BottomRight,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum ToastVariant {
    #[default]
    Default,
    Destructive,
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

#[derive(Debug, Clone)]
pub(super) struct ToastEntry {
    pub(super) id: ToastId,
    pub(super) title: Arc<str>,
    pub(super) description: Option<Arc<str>>,
    pub(super) variant: ToastVariant,
    pub(super) action: Option<ToastAction>,
    pub(super) dismissible: bool,
    pub(super) token: Option<TimerToken>,
}

#[derive(Debug, Default)]
pub struct ToastStore {
    next_id: u64,
    by_window: HashMap<AppWindowId, Vec<ToastEntry>>,
    by_token: HashMap<TimerToken, (AppWindowId, ToastId)>,
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
        token: Option<TimerToken>,
    ) -> ToastId {
        if self.next_id == 0 {
            self.next_id = 1;
        }
        let id = ToastId(self.next_id);
        self.next_id = self.next_id.saturating_add(1);

        if let Some(token) = token {
            self.by_token.insert(token, (window, id));
        }

        self.by_window.entry(window).or_default().push(ToastEntry {
            id,
            title: request.title,
            description: request.description,
            variant: request.variant,
            action: request.action,
            dismissible: request.dismissible,
            token,
        });

        id
    }

    fn remove_toast(&mut self, window: AppWindowId, id: ToastId) -> Option<ToastEntry> {
        let toasts = self.by_window.get_mut(&window)?;
        let idx = toasts.iter().position(|t| t.id == id)?;
        let entry = toasts.remove(idx);
        if let Some(token) = entry.token {
            self.by_token.remove(&token);
        }
        Some(entry)
    }

    pub(super) fn remove_toast_by_token(
        &mut self,
        token: TimerToken,
    ) -> Option<(AppWindowId, ToastEntry)> {
        let (window, id) = self.by_token.remove(&token)?;
        let entry = self.remove_toast(window, id)?;
        Some((window, entry))
    }
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
    let removed = host
        .models_mut()
        .update(&store, |st| st.remove_toast(window, id))
        .ok();
    let Some(entry) = removed.flatten() else {
        return false;
    };

    if let Some(token) = entry.token {
        host.push_effect(Effect::CancelTimer { token });
    }
    true
}
