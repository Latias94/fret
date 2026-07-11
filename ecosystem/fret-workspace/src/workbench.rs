use std::collections::HashSet;
use std::sync::Arc;

use fret_core::AppWindowId;
use fret_runtime::{ActionId, CommandDispatchSourceV1, CommandId, Model, ModelStore};
use fret_ui::UiHost;
use fret_ui::elements::GlobalElementId;

use crate::close_policy::{
    WorkspaceCloseReason, WorkspaceDirtyCloseDecision, WorkspaceDirtyClosePolicy,
    WorkspaceDirtyCloseRequest,
};
use crate::command_scope::{
    WorkspaceCommandScopeFocusLane, WorkspaceCommandScopeFocusSnapshot,
    workspace_command_scope_focus_snapshot,
};
use crate::commands::{
    CMD_WORKSPACE_DIRTY_CLOSE_CANCEL, CMD_WORKSPACE_DIRTY_CLOSE_SAVE_AND_CLOSE,
    is_typed_workspace_command, is_workspace_dirty_close_resolution, is_workspace_model_command,
};
use crate::layout::WorkspaceWindowLayout;

#[derive(Debug, Clone)]
pub struct WorkspaceDirtyClosePrompt {
    pub pane_id: Arc<str>,
    pub command: CommandId,
    pub target: Option<Arc<str>>,
    pub request: WorkspaceDirtyCloseRequest,
    pub focus_restore_target: Option<GlobalElementId>,
    pub focus_fallback: Option<WorkspaceWorkbenchFocusFallback>,
}

impl WorkspaceDirtyClosePrompt {
    fn window_close(
        request: WorkspaceDirtyCloseRequest,
        focus_restore_target: Option<GlobalElementId>,
        focus_fallback: Option<WorkspaceWorkbenchFocusFallback>,
    ) -> Self {
        Self {
            pane_id: Arc::from("<window>"),
            command: CommandId::from("window.close"),
            target: Some(Arc::from("window")),
            request,
            focus_restore_target,
            focus_fallback,
        }
    }

    fn tab_command(
        pane_id: Arc<str>,
        command: CommandId,
        target: Option<Arc<str>>,
        request: WorkspaceDirtyCloseRequest,
        focus_restore_target: Option<GlobalElementId>,
        focus_fallback: Option<WorkspaceWorkbenchFocusFallback>,
    ) -> Self {
        Self {
            pane_id,
            command,
            target,
            request,
            focus_restore_target,
            focus_fallback,
        }
    }

    pub fn is_window_close(&self) -> bool {
        self.request.reason == WorkspaceCloseReason::CloseWindow
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WorkspaceWorkbenchFocusFallback {
    ActiveTabStrip,
    ActivePaneContent,
}

impl WorkspaceWorkbenchFocusFallback {
    pub fn command_id(self) -> CommandId {
        match self {
            Self::ActiveTabStrip => crate::commands::typed_command_id::<
                crate::commands::act::WorkspacePaneFocusTabStrip,
            >(),
            Self::ActivePaneContent => crate::commands::typed_command_id::<
                crate::commands::act::WorkspacePaneFocusContent,
            >(),
        }
    }
}

/// Window-owner policy for close commands that target a pane's only tab.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum WorkspaceLastTabClosePolicy {
    /// Allow the workspace model to represent an empty pane.
    #[default]
    AllowEmptyPane,
    /// Keep the pane's final tab open and report the close as a handled no-op.
    PreserveLastTab,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WorkspaceWorkbenchFocusGuard {
    NoLiveFocus,
    Unchanged(GlobalElementId),
    Authoritative,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkspaceWorkbenchFocusRequest {
    pub guard: WorkspaceWorkbenchFocusGuard,
    pub target: Option<GlobalElementId>,
    pub fallback: Option<WorkspaceWorkbenchFocusFallback>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkspaceWorkbenchCommandOutcome {
    pub handled: bool,
    pub action_id: Option<ActionId>,
    pub target: Option<Arc<str>>,
    pub applied: bool,
    pub blocked_dirty_close: bool,
    pub close_window: bool,
    pub focus: Option<WorkspaceWorkbenchFocusRequest>,
}

impl WorkspaceWorkbenchCommandOutcome {
    pub fn unhandled() -> Self {
        Self {
            handled: false,
            action_id: None,
            target: None,
            applied: false,
            blocked_dirty_close: false,
            close_window: false,
            focus: None,
        }
    }
}

impl From<&WorkspaceWorkbenchCommandOutcome> for fret_runtime::CommandDispatchOutcomeV1 {
    fn from(outcome: &WorkspaceWorkbenchCommandOutcome) -> Self {
        Self {
            action_id: outcome.action_id.clone(),
            target: outcome.target.clone(),
            applied: outcome.applied,
            blocked_dirty_close: outcome.blocked_dirty_close,
        }
    }
}

struct ConfiguredDirtyClosePolicy {
    block: bool,
}

impl WorkspaceDirtyClosePolicy for ConfiguredDirtyClosePolicy {
    fn decide_dirty_close(
        &mut self,
        _request: &WorkspaceDirtyCloseRequest,
    ) -> WorkspaceDirtyCloseDecision {
        if self.block {
            WorkspaceDirtyCloseDecision::Block
        } else {
            WorkspaceDirtyCloseDecision::Allow
        }
    }
}

/// App-facing owner for workspace layout commands and dirty-close transactions.
#[derive(Clone)]
pub struct WorkspaceWorkbench {
    window_layout: Model<WorkspaceWindowLayout>,
    dirty_close_prompt_open: Model<bool>,
    dirty_close_prompt: Model<Option<WorkspaceDirtyClosePrompt>>,
    block_dirty_close: bool,
    last_tab_close_policy: WorkspaceLastTabClosePolicy,
}

impl WorkspaceWorkbench {
    pub fn new(
        models: &mut ModelStore,
        window_layout: Model<WorkspaceWindowLayout>,
        block_dirty_close: bool,
    ) -> Self {
        Self {
            window_layout,
            dirty_close_prompt_open: models.insert(false),
            dirty_close_prompt: models.insert(None),
            block_dirty_close,
            last_tab_close_policy: WorkspaceLastTabClosePolicy::default(),
        }
    }

    pub fn with_last_tab_close_policy(mut self, policy: WorkspaceLastTabClosePolicy) -> Self {
        self.last_tab_close_policy = policy;
        self
    }

    pub fn window_layout(&self) -> &Model<WorkspaceWindowLayout> {
        &self.window_layout
    }

    pub fn dirty_close_prompt_open(&self) -> &Model<bool> {
        &self.dirty_close_prompt_open
    }

    pub fn dirty_close_prompt(&self) -> &Model<Option<WorkspaceDirtyClosePrompt>> {
        &self.dirty_close_prompt
    }

    /// Returns `true` when prompt ownership cannot be read so mutation entry points fail closed.
    pub fn has_pending_dirty_close(&self, models: &ModelStore) -> bool {
        models
            .read(&self.dirty_close_prompt, |prompt| prompt.is_some())
            .unwrap_or(true)
    }

    pub fn pending_dirty_close(&self, models: &ModelStore) -> Option<WorkspaceDirtyClosePrompt> {
        models.get_cloned(&self.dirty_close_prompt).flatten()
    }

    pub fn confirm_dirty_close_saved<H: UiHost>(
        &self,
        app: &mut H,
        source: &CommandDispatchSourceV1,
    ) -> WorkspaceWorkbenchCommandOutcome {
        self.resolve_dirty_close(
            app.models_mut(),
            &CommandId::from(CMD_WORKSPACE_DIRTY_CLOSE_SAVE_AND_CLOSE),
            true,
            source.element.map(GlobalElementId),
        )
    }

    pub fn apply_command<H: UiHost>(
        &self,
        app: &mut H,
        window: AppWindowId,
        command: &CommandId,
    ) -> WorkspaceWorkbenchCommandOutcome {
        if !is_workspace_dirty_close_resolution(command) && !is_workspace_model_command(command) {
            return WorkspaceWorkbenchCommandOutcome::unhandled();
        }
        let focus = workspace_command_scope_focus_snapshot(app, window);
        self.apply_command_with_focus(app.models_mut(), command, focus)
    }

    fn apply_command_with_focus(
        &self,
        models: &mut ModelStore,
        command: &CommandId,
        focus: WorkspaceCommandScopeFocusSnapshot,
    ) -> WorkspaceWorkbenchCommandOutcome {
        if is_workspace_dirty_close_resolution(command) {
            return self.resolve_dirty_close(models, command, false, focus.target);
        }
        if let Some(outcome) = self.blocked_by_pending_dirty_close(models, command) {
            return outcome;
        }
        if !is_workspace_model_command(command) {
            return WorkspaceWorkbenchCommandOutcome::unhandled();
        }

        let mut policy = ConfiguredDirtyClosePolicy {
            block: self.block_dirty_close,
        };
        let update = models.update(&self.window_layout, |layout| {
            let target = workspace_command_target(layout, command);
            let active_pane_before = layout.active_pane.clone();
            let command_pane = active_pane_before
                .clone()
                .or_else(|| layout.pane_tree.first_leaf_id().cloned());
            let active_tab_before = active_tab_id(layout);
            let tab_close_command = is_tab_close_command(command);
            let focus_fallback =
                focus_fallback(&focus, active_pane_before.as_deref()).or_else(|| {
                    tab_close_command.then_some(WorkspaceWorkbenchFocusFallback::ActiveTabStrip)
                });
            let preserve_last_tab = command_pane.as_deref().is_some_and(|pane_id| {
                self.last_tab_close_policy == WorkspaceLastTabClosePolicy::PreserveLastTab
                    && tab_close_would_empty_pane(layout, pane_id, command)
            });
            let outcome = if preserve_last_tab {
                crate::tabs::WorkspaceApplyCommandOutcome::applied(false)
            } else {
                layout.apply_command_with_close_policy(command, Some(&mut policy))
            };
            let active_context_changed = active_pane_before != layout.active_pane
                || active_tab_before != active_tab_id(layout);
            let follow_focus = outcome.applied
                && focus_fallback.is_some()
                && (tab_close_command || active_context_changed);
            let focus_request = follow_focus.then(|| WorkspaceWorkbenchFocusRequest {
                guard: focus
                    .target
                    .map(WorkspaceWorkbenchFocusGuard::Unchanged)
                    .unwrap_or(WorkspaceWorkbenchFocusGuard::NoLiveFocus),
                target: tab_close_command.then_some(focus.target).flatten(),
                fallback: focus_fallback,
            });
            (
                target,
                active_pane_before,
                focus_fallback,
                outcome,
                focus_request,
            )
        });
        let Ok((target, active_pane, focus_fallback, outcome, focus_request)) = update else {
            return WorkspaceWorkbenchCommandOutcome::unhandled();
        };

        if let Some(request) = outcome.blocked_dirty_close.clone()
            && let Some(pane_id) = active_pane
        {
            self.open_dirty_close_prompt(
                models,
                WorkspaceDirtyClosePrompt::tab_command(
                    pane_id,
                    command.clone(),
                    target.clone(),
                    request,
                    focus.target,
                    focus_fallback,
                ),
            );
        }

        WorkspaceWorkbenchCommandOutcome {
            handled: true,
            action_id: is_typed_workspace_command(command).then(|| command.clone()),
            target,
            applied: outcome.applied,
            blocked_dirty_close: outcome.blocked_dirty_close.is_some(),
            close_window: false,
            focus: focus_request,
        }
    }

    pub fn apply_command_in_pane<H: UiHost>(
        &self,
        app: &mut H,
        window: AppWindowId,
        pane_id: impl Into<Arc<str>>,
        command: &CommandId,
    ) -> WorkspaceWorkbenchCommandOutcome {
        let pane_id = pane_id.into();
        if let Some(outcome) = self.blocked_by_pending_dirty_close(app.models_mut(), command) {
            return outcome;
        }
        if is_workspace_dirty_close_resolution(command) {
            return self.apply_command_with_focus(
                app.models_mut(),
                command,
                WorkspaceCommandScopeFocusSnapshot::default(),
            );
        }
        if !is_workspace_model_command(command) {
            return WorkspaceWorkbenchCommandOutcome::unhandled();
        }
        let focus = workspace_command_scope_focus_snapshot(app, window);
        let models = app.models_mut();
        let pane_exists = models
            .read(&self.window_layout, |layout| {
                layout.pane_tree.find_pane(pane_id.as_ref()).is_some()
            })
            .unwrap_or(false);
        if !pane_exists {
            return WorkspaceWorkbenchCommandOutcome {
                handled: true,
                action_id: is_typed_workspace_command(command).then(|| command.clone()),
                target: Some(pane_id),
                applied: false,
                blocked_dirty_close: false,
                close_window: false,
                focus: None,
            };
        }
        let _ = models.update(&self.window_layout, |layout| {
            let _ = layout.activate_pane(pane_id.as_ref());
        });
        self.apply_command_with_focus(models, command, focus)
    }

    pub fn request_window_close<H: UiHost>(
        &self,
        app: &mut H,
        window: AppWindowId,
    ) -> WorkspaceWorkbenchCommandOutcome {
        let focus = workspace_command_scope_focus_snapshot(app, window);
        let models = app.models_mut();
        let blocked_window_close = || WorkspaceWorkbenchCommandOutcome {
            handled: true,
            action_id: None,
            target: Some(Arc::from("window")),
            applied: false,
            blocked_dirty_close: true,
            close_window: false,
            focus: None,
        };
        if self.has_pending_dirty_close(models) {
            return blocked_window_close();
        }
        let mut policy = ConfiguredDirtyClosePolicy {
            block: self.block_dirty_close,
        };
        let outcome = match models.read(&self.window_layout, |layout| {
            layout.can_close_window_with_policy(Some(&mut policy))
        }) {
            Ok(outcome) => outcome,
            Err(_) => return blocked_window_close(),
        };
        let target = Some(Arc::from("window"));
        let active_pane = models
            .read(&self.window_layout, |layout| {
                layout.active_pane_id().cloned()
            })
            .ok()
            .flatten();
        let focus_fallback = focus_fallback(&focus, active_pane.as_deref());

        if let Some(request) = outcome.blocked_dirty_close.clone() {
            self.open_dirty_close_prompt(
                models,
                WorkspaceDirtyClosePrompt::window_close(request, focus.target, focus_fallback),
            );
        }

        WorkspaceWorkbenchCommandOutcome {
            handled: true,
            action_id: None,
            target,
            applied: outcome.applied,
            blocked_dirty_close: outcome.blocked_dirty_close.is_some(),
            close_window: outcome.applied,
            focus: None,
        }
    }

    fn resolve_dirty_close(
        &self,
        models: &mut ModelStore,
        command: &CommandId,
        save_confirmed: bool,
        command_focus: Option<GlobalElementId>,
    ) -> WorkspaceWorkbenchCommandOutcome {
        let prompt = models.get_cloned(&self.dirty_close_prompt).flatten();
        let Some(prompt) = prompt else {
            return WorkspaceWorkbenchCommandOutcome {
                handled: true,
                action_id: Some(command.clone()),
                target: None,
                applied: false,
                blocked_dirty_close: false,
                close_window: false,
                focus: None,
            };
        };
        let target = prompt_target(&prompt);
        let cancel = command.as_str() == CMD_WORKSPACE_DIRTY_CLOSE_CANCEL;
        let save = command.as_str() == CMD_WORKSPACE_DIRTY_CLOSE_SAVE_AND_CLOSE;
        if save && !save_confirmed {
            return WorkspaceWorkbenchCommandOutcome {
                handled: true,
                action_id: Some(command.clone()),
                target,
                applied: false,
                blocked_dirty_close: true,
                close_window: false,
                focus: None,
            };
        }
        if !cancel {
            let refreshed_request = models
                .read(&self.window_layout, |layout| {
                    dirty_close_request_with_new_dirty_tabs(layout, &prompt)
                })
                .ok()
                .flatten();
            if let Some(refreshed_request) = refreshed_request {
                let _ = models.update(&self.dirty_close_prompt, |slot| {
                    if let Some(current) = slot.as_mut() {
                        current.request = refreshed_request;
                    }
                });
                return WorkspaceWorkbenchCommandOutcome {
                    handled: true,
                    action_id: Some(command.clone()),
                    target,
                    applied: false,
                    blocked_dirty_close: true,
                    close_window: false,
                    focus: None,
                };
            }
        }
        let applied = if cancel {
            false
        } else {
            models
                .update(&self.window_layout, |layout| {
                    let mut candidate = layout.clone();
                    let preserve_last_tab = !prompt.is_window_close()
                        && self.last_tab_close_policy
                            == WorkspaceLastTabClosePolicy::PreserveLastTab
                        && dirty_close_targets_would_empty_pane(&candidate, &prompt);
                    let applied = !preserve_last_tab
                        && apply_dirty_close_resolution(&mut candidate, &prompt, save);
                    if applied {
                        *layout = candidate;
                    }
                    applied
                })
                .unwrap_or(false)
        };
        if !cancel && !applied {
            return WorkspaceWorkbenchCommandOutcome {
                handled: true,
                action_id: Some(command.clone()),
                target,
                applied: false,
                blocked_dirty_close: true,
                close_window: false,
                focus: None,
            };
        }
        self.clear_dirty_close_prompt(models);

        let restore_after_cancel =
            cancel && (prompt.focus_restore_target.is_some() || prompt.focus_fallback.is_some());
        let follow_after_tab_close = !prompt.is_window_close()
            && applied
            && (prompt.focus_restore_target.is_some() || prompt.focus_fallback.is_some());
        WorkspaceWorkbenchCommandOutcome {
            handled: true,
            action_id: Some(command.clone()),
            target,
            applied,
            blocked_dirty_close: false,
            close_window: prompt.is_window_close() && applied,
            focus: (restore_after_cancel || follow_after_tab_close).then_some(
                WorkspaceWorkbenchFocusRequest {
                    guard: if cancel {
                        command_focus
                            .map(WorkspaceWorkbenchFocusGuard::Unchanged)
                            .unwrap_or(WorkspaceWorkbenchFocusGuard::NoLiveFocus)
                    } else {
                        WorkspaceWorkbenchFocusGuard::Authoritative
                    },
                    target: cancel.then_some(prompt.focus_restore_target).flatten(),
                    fallback: prompt.focus_fallback,
                },
            ),
        }
    }

    fn blocked_by_pending_dirty_close(
        &self,
        models: &mut ModelStore,
        command: &CommandId,
    ) -> Option<WorkspaceWorkbenchCommandOutcome> {
        if !is_workspace_model_command(command) || !self.has_pending_dirty_close(models) {
            return None;
        }
        let target = models
            .read(&self.window_layout, |layout| {
                workspace_command_target(layout, command)
            })
            .ok()
            .flatten();
        Some(WorkspaceWorkbenchCommandOutcome {
            handled: true,
            action_id: is_typed_workspace_command(command).then(|| command.clone()),
            target,
            applied: false,
            blocked_dirty_close: true,
            close_window: false,
            focus: None,
        })
    }

    fn open_dirty_close_prompt(&self, models: &mut ModelStore, prompt: WorkspaceDirtyClosePrompt) {
        let _ = models.update(&self.dirty_close_prompt, |slot| *slot = Some(prompt));
        let _ = models.update(&self.dirty_close_prompt_open, |open| *open = true);
    }

    fn clear_dirty_close_prompt(&self, models: &mut ModelStore) {
        let _ = models.update(&self.dirty_close_prompt, |slot| *slot = None);
        let _ = models.update(&self.dirty_close_prompt_open, |open| *open = false);
    }
}

fn workspace_target(layout: &WorkspaceWindowLayout) -> Option<Arc<str>> {
    let pane_id = layout.active_pane_id()?.clone();
    let tab_id = active_tab_id(layout);
    Some(match tab_id {
        Some(tab_id) => Arc::from(format!("{pane_id}/{tab_id}")),
        None => pane_id,
    })
}

fn active_tab_id(layout: &WorkspaceWindowLayout) -> Option<Arc<str>> {
    layout
        .active_pane_id()
        .and_then(|pane_id| layout.pane_tree.find_pane(pane_id.as_ref()))
        .and_then(|pane| pane.tabs.active().cloned())
}

fn focus_fallback(
    focus: &WorkspaceCommandScopeFocusSnapshot,
    active_pane: Option<&str>,
) -> Option<WorkspaceWorkbenchFocusFallback> {
    let lane = focus.lane.as_ref()?;
    if active_pane != Some(lane.pane_id().as_ref()) {
        return None;
    }
    Some(match lane {
        WorkspaceCommandScopeFocusLane::TabStrip(_) => {
            WorkspaceWorkbenchFocusFallback::ActiveTabStrip
        }
        WorkspaceCommandScopeFocusLane::PaneContent(_) => {
            WorkspaceWorkbenchFocusFallback::ActivePaneContent
        }
    })
}

fn is_tab_close_command(command: &CommandId) -> bool {
    matches!(
        command.as_str(),
        crate::commands::CMD_WORKSPACE_TAB_CLOSE
            | crate::commands::CMD_WORKSPACE_TAB_CLOSE_OTHERS
            | crate::commands::CMD_WORKSPACE_TAB_CLOSE_LEFT
            | crate::commands::CMD_WORKSPACE_TAB_CLOSE_RIGHT
    ) || command
        .as_str()
        .strip_prefix(crate::commands::CMD_WORKSPACE_TAB_CLOSE_PREFIX)
        .is_some_and(|tab_id| !tab_id.trim().is_empty())
}

fn tab_close_would_empty_pane(
    layout: &WorkspaceWindowLayout,
    pane_id: &str,
    command: &CommandId,
) -> bool {
    let Some(pane) = layout.pane_tree.find_pane(pane_id) else {
        return false;
    };
    let [only_tab] = pane.tabs.tabs() else {
        return false;
    };

    if command.as_str() == crate::commands::CMD_WORKSPACE_TAB_CLOSE {
        return pane
            .tabs
            .active()
            .is_some_and(|active| active.as_ref() == only_tab.as_ref());
    }
    if matches!(
        command.as_str(),
        crate::commands::CMD_WORKSPACE_TAB_CLOSE_OTHERS
            | crate::commands::CMD_WORKSPACE_TAB_CLOSE_LEFT
            | crate::commands::CMD_WORKSPACE_TAB_CLOSE_RIGHT
    ) {
        return false;
    }

    command
        .as_str()
        .strip_prefix(crate::commands::CMD_WORKSPACE_TAB_CLOSE_PREFIX)
        .map(str::trim)
        .is_some_and(|tab_id| !tab_id.is_empty() && tab_id == only_tab.as_ref())
}

fn dirty_close_targets_would_empty_pane(
    layout: &WorkspaceWindowLayout,
    prompt: &WorkspaceDirtyClosePrompt,
) -> bool {
    let Some(pane) = layout.pane_tree.find_pane(prompt.pane_id.as_ref()) else {
        return false;
    };
    !pane.tabs.tabs().is_empty()
        && pane
            .tabs
            .tabs()
            .iter()
            .all(|tab| prompt.request.target_tabs_in_order.contains(tab))
}

fn workspace_command_target(
    layout: &WorkspaceWindowLayout,
    command: &CommandId,
) -> Option<Arc<str>> {
    let pane_id = layout.active_pane_id()?.clone();
    for prefix in [
        crate::commands::CMD_WORKSPACE_TAB_ACTIVATE_PREFIX,
        crate::commands::CMD_WORKSPACE_TAB_CLOSE_PREFIX,
        crate::commands::CMD_WORKSPACE_TAB_PIN_PREFIX,
        crate::commands::CMD_WORKSPACE_TAB_UNPIN_PREFIX,
        crate::commands::CMD_WORKSPACE_TAB_OPEN_PREVIEW_PREFIX,
    ] {
        if let Some(tab_id) = command.as_str().strip_prefix(prefix)
            && !tab_id.is_empty()
            && !matches!(tab_id, "others" | "left" | "right")
        {
            return Some(Arc::from(format!("{pane_id}/{tab_id}")));
        }
    }
    if let Some(pane_id) = command
        .as_str()
        .strip_prefix(crate::commands::CMD_WORKSPACE_PANE_ACTIVATE_PREFIX)
        .filter(|pane_id| !pane_id.is_empty())
    {
        return Some(Arc::from(pane_id));
    }
    workspace_target(layout)
}

fn prompt_target(prompt: &WorkspaceDirtyClosePrompt) -> Option<Arc<str>> {
    prompt.target.clone()
}

fn dirty_close_request_with_new_dirty_tabs(
    layout: &WorkspaceWindowLayout,
    prompt: &WorkspaceDirtyClosePrompt,
) -> Option<WorkspaceDirtyCloseRequest> {
    let current = if prompt.is_window_close() {
        layout.dirty_close_request_for_window_close()?
    } else {
        let pane = layout.pane_tree.find_pane(prompt.pane_id.as_ref())?;
        let dirty_tabs_in_order = prompt
            .request
            .target_tabs_in_order
            .iter()
            .filter(|tab| pane.tabs.is_dirty(tab.as_ref()))
            .cloned()
            .collect();
        WorkspaceDirtyCloseRequest {
            reason: prompt.request.reason,
            target_tabs_in_order: prompt.request.target_tabs_in_order.clone(),
            dirty_tabs_in_order,
            active_tab_id: prompt.request.active_tab_id.clone(),
        }
    };
    let confirmed_dirty: HashSet<&str> = prompt
        .request
        .dirty_tabs_in_order
        .iter()
        .map(AsRef::as_ref)
        .collect();
    current
        .dirty_tabs_in_order
        .iter()
        .any(|dirty| !confirmed_dirty.contains(dirty.as_ref()))
        .then_some(current)
}

fn apply_dirty_close_resolution(
    layout: &mut WorkspaceWindowLayout,
    prompt: &WorkspaceDirtyClosePrompt,
    save: bool,
) -> bool {
    if prompt.is_window_close() {
        if save {
            let mut pane_ids = Vec::new();
            layout.pane_tree.collect_leaf_ids(&mut pane_ids);
            for dirty_id in &prompt.request.dirty_tabs_in_order {
                for pane_id in &pane_ids {
                    if let Some(pane) = layout.pane_tree.find_pane_mut(pane_id.as_ref()) {
                        pane.tabs.set_dirty(dirty_id.clone(), false);
                    }
                }
            }
        }
        return true;
    }

    layout.active_pane = Some(prompt.pane_id.clone());
    let Some(pane) = layout.pane_tree.find_pane_mut(prompt.pane_id.as_ref()) else {
        return false;
    };
    if prompt.request.target_tabs_in_order.is_empty()
        || !prompt
            .request
            .target_tabs_in_order
            .iter()
            .all(|target| pane.tabs.tabs().contains(target))
    {
        return false;
    }
    if let Some(active) = prompt.request.active_tab_id.clone()
        && !pane.tabs.activate(active)
    {
        return false;
    }
    if save {
        for id in &prompt.request.dirty_tabs_in_order {
            pane.tabs.set_dirty(id.clone(), false);
        }
    }
    prompt
        .request
        .target_tabs_in_order
        .iter()
        .all(|target| pane.tabs.close(target.as_ref()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::commands::{CMD_WORKSPACE_DIRTY_CLOSE_DISCARD, act, typed_command_id};
    use crate::layout::WorkspacePaneTree;
    use fret_core::Axis;

    fn workbench(block_dirty_close: bool) -> (fret_app::App, WorkspaceWorkbench) {
        let mut app = fret_app::App::new();
        let mut layout = WorkspaceWindowLayout::new("window", "pane-a");
        layout.pane_tree = WorkspacePaneTree::leaf("pane-a");
        let pane = layout.pane_tree.find_pane_mut("pane-a").unwrap();
        pane.tabs.open_and_activate(Arc::from("doc-a"));
        pane.tabs.open_and_activate(Arc::from("doc-b"));
        let layout = app.models_mut().insert(layout);
        let workbench = WorkspaceWorkbench::new(app.models_mut(), layout, block_dirty_close);
        (app, workbench)
    }

    fn two_pane_workbench() -> (fret_app::App, WorkspaceWorkbench) {
        let mut app = fret_app::App::new();
        let mut layout = WorkspaceWindowLayout::new("window", "pane-a");
        layout.pane_tree = WorkspacePaneTree::split(
            Axis::Horizontal,
            0.5,
            WorkspacePaneTree::leaf("pane-a"),
            WorkspacePaneTree::leaf("pane-b"),
        );
        layout.active_pane = Some(Arc::from("pane-a"));
        layout
            .pane_tree
            .find_pane_mut("pane-a")
            .unwrap()
            .tabs
            .open_and_activate(Arc::from("doc-a"));
        layout
            .pane_tree
            .find_pane_mut("pane-b")
            .unwrap()
            .tabs
            .open_and_activate(Arc::from("doc-b"));
        let layout = app.models_mut().insert(layout);
        let workbench = WorkspaceWorkbench::new(app.models_mut(), layout, false);
        (app, workbench)
    }

    #[test]
    fn command_outcome_converts_to_generic_dispatch_diagnostics() {
        let command = typed_command_id::<act::WorkspaceTabNext>();
        let outcome = WorkspaceWorkbenchCommandOutcome {
            handled: true,
            action_id: Some(command.clone()),
            target: Some(Arc::from("pane-a/doc-a")),
            applied: true,
            blocked_dirty_close: false,
            close_window: false,
            focus: None,
        };

        assert_eq!(
            fret_runtime::CommandDispatchOutcomeV1::from(&outcome),
            fret_runtime::CommandDispatchOutcomeV1 {
                action_id: Some(command),
                target: Some(Arc::from("pane-a/doc-a")),
                applied: true,
                blocked_dirty_close: false,
            }
        );
    }

    #[test]
    fn dirty_close_cancel_and_confirmed_save_are_owned_by_the_workbench() {
        let (mut app, workbench) = workbench(true);
        let window = AppWindowId::default();
        let _ = app
            .models_mut()
            .update(workbench.window_layout(), |layout| {
                let pane = layout.pane_tree.find_pane_mut("pane-a").unwrap();
                pane.tabs.set_dirty(Arc::from("doc-b"), true);
            });

        let close = typed_command_id::<act::WorkspaceTabClose>();
        let blocked = workbench.apply_command(&mut app, window, &close);
        assert!(blocked.handled);
        assert!(blocked.blocked_dirty_close);
        assert!(!blocked.applied);

        let cancel = CommandId::from(CMD_WORKSPACE_DIRTY_CLOSE_CANCEL);
        let canceled = workbench.apply_command(&mut app, window, &cancel);
        assert!(canceled.handled);
        assert!(!canceled.applied);
        assert_eq!(
            app.models().get_copied(workbench.dirty_close_prompt_open()),
            Some(false)
        );

        let _ = workbench.apply_command(&mut app, window, &close);
        let save = CommandId::from(CMD_WORKSPACE_DIRTY_CLOSE_SAVE_AND_CLOSE);
        let refused = workbench.apply_command(&mut app, window, &save);
        assert!(!refused.applied);
        assert!(workbench.has_pending_dirty_close(app.models()));

        let save_source = CommandDispatchSourceV1 {
            kind: fret_runtime::CommandDispatchSourceKindV1::Keyboard,
            element: Some(77),
            test_id: Some(Arc::from("dirty-close.save")),
        };
        let saved = workbench.confirm_dirty_close_saved(&mut app, &save_source);
        assert!(saved.applied);
        assert_eq!(
            saved.focus.as_ref().map(|focus| focus.guard),
            Some(WorkspaceWorkbenchFocusGuard::Authoritative),
            "a committed close owns the post-frame focus transaction"
        );
        assert_eq!(
            saved.focus.as_ref().map(|focus| focus.target),
            Some(None),
            "a committed close must focus the surviving workspace target via fallback"
        );
        let active_dirty = app
            .models()
            .read(workbench.window_layout(), |layout| {
                layout
                    .pane_tree
                    .find_pane("pane-a")
                    .unwrap()
                    .tabs
                    .dirty_in_tab_order()
            })
            .unwrap();
        assert!(active_dirty.is_empty());
    }

    #[test]
    fn typed_command_outcome_keeps_action_and_target_identity() {
        let (mut app, workbench) = workbench(false);
        let window = AppWindowId::default();
        let command = typed_command_id::<act::WorkspaceTabNext>();

        let outcome = workbench.apply_command(&mut app, window, &command);

        assert!(outcome.handled);
        assert!(outcome.applied);
        assert_eq!(outcome.action_id, Some(command));
        assert_eq!(outcome.target.as_deref(), Some("pane-a/doc-b"));
    }

    #[test]
    fn recognized_noop_is_handled_with_an_unapplied_outcome() {
        let (mut app, workbench) = workbench(false);
        let command = typed_command_id::<act::WorkspacePaneFocusRight>();

        let outcome = workbench.apply_command(&mut app, AppWindowId::default(), &command);

        assert!(outcome.handled);
        assert!(!outcome.applied);
        assert_eq!(outcome.action_id, Some(command));
        assert_eq!(outcome.target.as_deref(), Some("pane-a/doc-b"));
    }

    #[test]
    fn default_last_tab_close_policy_allows_an_empty_pane() {
        let (mut app, workbench) = workbench(false);
        let close = typed_command_id::<act::WorkspaceTabClose>();

        assert!(
            workbench
                .apply_command(&mut app, AppWindowId::default(), &close)
                .applied
        );
        assert!(
            workbench
                .apply_command(&mut app, AppWindowId::default(), &close)
                .applied
        );

        let tabs = app
            .models()
            .read(workbench.window_layout(), |layout| {
                layout
                    .pane_tree
                    .find_pane("pane-a")
                    .unwrap()
                    .tabs
                    .tabs()
                    .to_vec()
            })
            .unwrap();
        assert!(tabs.is_empty());
    }

    #[test]
    fn preserve_last_tab_policy_closes_to_one_then_reports_a_handled_noop() {
        let (mut app, workbench) = workbench(false);
        let workbench =
            workbench.with_last_tab_close_policy(WorkspaceLastTabClosePolicy::PreserveLastTab);
        let close = typed_command_id::<act::WorkspaceTabClose>();

        assert!(
            workbench
                .apply_command(&mut app, AppWindowId::default(), &close)
                .applied
        );
        let _ = app
            .models_mut()
            .update(workbench.window_layout(), |layout| {
                layout
                    .pane_tree
                    .find_pane_mut("pane-a")
                    .unwrap()
                    .tabs
                    .set_dirty(Arc::from("doc-a"), true);
            });
        let preserved = workbench.apply_command(&mut app, AppWindowId::default(), &close);

        assert!(preserved.handled);
        assert!(!preserved.applied);
        assert!(!preserved.blocked_dirty_close);
        assert_eq!(preserved.action_id, Some(close));
        assert_eq!(preserved.target.as_deref(), Some("pane-a/doc-a"));
        assert!(preserved.focus.is_none());
        assert!(!workbench.has_pending_dirty_close(app.models()));
        let tabs = app
            .models()
            .read(workbench.window_layout(), |layout| {
                layout
                    .pane_tree
                    .find_pane("pane-a")
                    .unwrap()
                    .tabs
                    .tabs()
                    .to_vec()
            })
            .unwrap();
        assert_eq!(tabs, vec![Arc::<str>::from("doc-a")]);
    }

    #[test]
    fn preserve_last_tab_policy_applies_to_close_by_id_in_a_specific_pane() {
        let (mut app, workbench) = two_pane_workbench();
        let workbench =
            workbench.with_last_tab_close_policy(WorkspaceLastTabClosePolicy::PreserveLastTab);
        let close = crate::commands::tab_close_command("doc-b").unwrap();

        let preserved = workbench.apply_command_in_pane(
            &mut app,
            AppWindowId::default(),
            Arc::from("pane-b"),
            &close,
        );

        assert!(preserved.handled);
        assert!(!preserved.applied);
        assert!(!preserved.blocked_dirty_close);
        assert_eq!(preserved.target.as_deref(), Some("pane-b/doc-b"));
        let tabs = app
            .models()
            .read(workbench.window_layout(), |layout| {
                layout
                    .pane_tree
                    .find_pane("pane-b")
                    .unwrap()
                    .tabs
                    .tabs()
                    .to_vec()
            })
            .unwrap();
        assert_eq!(tabs, vec![Arc::<str>::from("doc-b")]);
    }

    #[test]
    fn preserve_last_tab_policy_rechecks_a_pending_dirty_close_resolution() {
        let (mut app, workbench) = workbench(true);
        let workbench =
            workbench.with_last_tab_close_policy(WorkspaceLastTabClosePolicy::PreserveLastTab);
        let window = AppWindowId::default();
        let _ = app
            .models_mut()
            .update(workbench.window_layout(), |layout| {
                layout
                    .pane_tree
                    .find_pane_mut("pane-a")
                    .unwrap()
                    .tabs
                    .set_dirty(Arc::from("doc-b"), true);
            });
        let close = typed_command_id::<act::WorkspaceTabClose>();
        assert!(
            workbench
                .apply_command(&mut app, window, &close)
                .blocked_dirty_close
        );
        let _ = app
            .models_mut()
            .update(workbench.window_layout(), |layout| {
                assert!(
                    layout
                        .pane_tree
                        .find_pane_mut("pane-a")
                        .unwrap()
                        .tabs
                        .close("doc-a")
                );
            });

        let discard = CommandId::from(CMD_WORKSPACE_DIRTY_CLOSE_DISCARD);
        let preserved = workbench.apply_command(&mut app, window, &discard);

        assert!(preserved.handled);
        assert!(!preserved.applied);
        assert!(preserved.blocked_dirty_close);
        assert!(workbench.has_pending_dirty_close(app.models()));
        let tabs = app
            .models()
            .read(workbench.window_layout(), |layout| {
                layout
                    .pane_tree
                    .find_pane("pane-a")
                    .unwrap()
                    .tabs
                    .tabs()
                    .to_vec()
            })
            .unwrap();
        assert_eq!(tabs, vec![Arc::<str>::from("doc-b")]);
    }

    #[test]
    fn pending_dirty_close_blocks_model_commands_before_any_target_mutation() {
        let (mut app, workbench) = workbench(true);
        let window = AppWindowId::default();
        let _ = app
            .models_mut()
            .update(workbench.window_layout(), |layout| {
                layout
                    .pane_tree
                    .find_pane_mut("pane-a")
                    .unwrap()
                    .tabs
                    .set_dirty(Arc::from("doc-b"), true);
            });
        let close = typed_command_id::<act::WorkspaceTabClose>();
        assert!(
            workbench
                .apply_command(&mut app, window, &close)
                .blocked_dirty_close
        );

        let next = typed_command_id::<act::WorkspaceTabNext>();
        let blocked = workbench.apply_command_in_pane(&mut app, window, Arc::from("pane-b"), &next);

        assert!(blocked.handled);
        assert!(!blocked.applied);
        assert_eq!(
            app.models()
                .read(workbench.window_layout(), |layout| layout
                    .active_pane
                    .clone())
                .unwrap()
                .as_deref(),
            Some("pane-a")
        );
    }

    #[test]
    fn stale_dirty_close_replay_reports_failure_without_mutating_replacement_layout() {
        let (mut app, workbench) = workbench(true);
        let window = AppWindowId::default();
        let _ = app
            .models_mut()
            .update(workbench.window_layout(), |layout| {
                layout
                    .pane_tree
                    .find_pane_mut("pane-a")
                    .unwrap()
                    .tabs
                    .set_dirty(Arc::from("doc-b"), true);
            });
        let close = typed_command_id::<act::WorkspaceTabClose>();
        let _ = workbench.apply_command(&mut app, window, &close);
        let _ = app
            .models_mut()
            .update(workbench.window_layout(), |layout| {
                layout.pane_tree = WorkspacePaneTree::leaf("replacement");
                layout.active_pane = Some(Arc::from("replacement"));
                layout
                    .pane_tree
                    .find_pane_mut("replacement")
                    .unwrap()
                    .tabs
                    .open_and_activate(Arc::from("replacement-doc"));
            });

        let outcome =
            workbench.confirm_dirty_close_saved(&mut app, &CommandDispatchSourceV1::programmatic());

        assert!(!outcome.applied);
        assert!(!outcome.close_window);
        assert!(outcome.focus.is_none());
        assert!(outcome.blocked_dirty_close);
        assert!(workbench.has_pending_dirty_close(app.models()));
        assert_eq!(
            app.models()
                .read(workbench.window_layout(), |layout| {
                    layout
                        .pane_tree
                        .find_pane("replacement")
                        .unwrap()
                        .tabs
                        .tabs()
                        .to_vec()
                })
                .unwrap(),
            vec![Arc::<str>::from("replacement-doc")]
        );
    }

    #[test]
    fn dirty_close_replay_closes_only_the_frozen_close_others_targets() {
        let (mut app, workbench) = workbench(true);
        let window = AppWindowId::default();
        let _ = app
            .models_mut()
            .update(workbench.window_layout(), |layout| {
                layout
                    .pane_tree
                    .find_pane_mut("pane-a")
                    .unwrap()
                    .tabs
                    .set_dirty(Arc::from("doc-a"), true);
            });
        let close_others = typed_command_id::<act::WorkspaceTabCloseOthers>();
        assert!(
            workbench
                .apply_command(&mut app, window, &close_others)
                .blocked_dirty_close
        );
        let _ = app
            .models_mut()
            .update(workbench.window_layout(), |layout| {
                let tabs = &mut layout.pane_tree.find_pane_mut("pane-a").unwrap().tabs;
                tabs.open_and_activate(Arc::from("doc-c"));
                assert!(tabs.activate(Arc::from("doc-b")));
            });

        let discard = CommandId::from(CMD_WORKSPACE_DIRTY_CLOSE_DISCARD);
        let outcome = workbench.apply_command(&mut app, window, &discard);

        assert!(outcome.applied);
        assert!(!outcome.blocked_dirty_close);
        assert_eq!(
            app.models()
                .read(workbench.window_layout(), |layout| {
                    layout
                        .pane_tree
                        .find_pane("pane-a")
                        .unwrap()
                        .tabs
                        .tabs()
                        .to_vec()
                })
                .unwrap(),
            vec![Arc::<str>::from("doc-b"), Arc::<str>::from("doc-c")]
        );
    }

    #[test]
    fn window_dirty_close_replay_refuses_a_new_dirty_tab() {
        let (mut app, workbench) = workbench(true);
        let _ = app
            .models_mut()
            .update(workbench.window_layout(), |layout| {
                layout
                    .pane_tree
                    .find_pane_mut("pane-a")
                    .unwrap()
                    .tabs
                    .set_dirty(Arc::from("doc-b"), true);
            });
        let blocked = workbench.request_window_close(&mut app, AppWindowId::default());
        assert!(blocked.blocked_dirty_close);
        let _ = app
            .models_mut()
            .update(workbench.window_layout(), |layout| {
                layout
                    .pane_tree
                    .find_pane_mut("pane-a")
                    .unwrap()
                    .tabs
                    .set_dirty(Arc::from("doc-a"), true);
            });

        let outcome =
            workbench.confirm_dirty_close_saved(&mut app, &CommandDispatchSourceV1::programmatic());

        assert!(!outcome.applied);
        assert!(!outcome.close_window);
        assert!(outcome.blocked_dirty_close);
        assert!(workbench.has_pending_dirty_close(app.models()));
        assert_eq!(
            workbench
                .pending_dirty_close(app.models())
                .unwrap()
                .request
                .dirty_tabs_in_order,
            vec![Arc::<str>::from("doc-a"), Arc::<str>::from("doc-b")]
        );
        assert_eq!(
            app.models()
                .read(workbench.window_layout(), |layout| layout
                    .dirty_close_request_for_window_close()
                    .unwrap()
                    .dirty_tabs_in_order)
                .unwrap(),
            vec![Arc::<str>::from("doc-a"), Arc::<str>::from("doc-b")]
        );
    }

    #[test]
    fn window_close_fails_closed_while_the_layout_is_leased() {
        let (mut app, workbench) = workbench(true);
        let window = AppWindowId::default();
        let layout = workbench.window_layout().clone();
        let _ = app.models_mut().update(&layout, |layout| {
            layout
                .pane_tree
                .find_pane_mut("pane-a")
                .unwrap()
                .tabs
                .set_dirty(Arc::from("doc-a"), true);
        });

        let outcome = layout
            .update(&mut app, |_layout, cx| {
                workbench.request_window_close(cx.app(), window)
            })
            .expect("outer layout update must complete");

        assert!(outcome.handled);
        assert!(!outcome.applied);
        assert!(outcome.blocked_dirty_close);
        assert!(!outcome.close_window);
    }

    #[test]
    fn window_close_fails_closed_while_the_dirty_close_prompt_is_leased() {
        let (mut app, workbench) = workbench(true);
        let window = AppWindowId::default();
        let _ = app
            .models_mut()
            .update(workbench.window_layout(), |layout| {
                layout
                    .pane_tree
                    .find_pane_mut("pane-a")
                    .unwrap()
                    .tabs
                    .set_dirty(Arc::from("doc-a"), true);
            });
        assert!(
            workbench
                .request_window_close(&mut app, window)
                .blocked_dirty_close
        );
        let _ = app
            .models_mut()
            .update(workbench.window_layout(), |layout| {
                layout
                    .pane_tree
                    .find_pane_mut("pane-a")
                    .unwrap()
                    .tabs
                    .set_dirty(Arc::from("doc-a"), false);
            });

        let prompt = workbench.dirty_close_prompt().clone();
        let next = typed_command_id::<act::WorkspaceTabNext>();
        let (outcome, blocked_command) = prompt
            .update(&mut app, |_prompt, cx| {
                let close = workbench.request_window_close(cx.app(), window);
                let command = workbench.apply_command(cx.app(), window, &next);
                (close, command)
            })
            .expect("outer dirty-close prompt update must complete");

        assert!(outcome.handled);
        assert!(!outcome.applied);
        assert!(outcome.blocked_dirty_close);
        assert!(!outcome.close_window);
        assert!(blocked_command.handled);
        assert!(!blocked_command.applied);
        assert!(blocked_command.blocked_dirty_close);
    }

    #[test]
    fn dirty_close_resolution_preserves_an_inactive_close_by_id_target() {
        let (mut app, workbench) = workbench(true);
        let window = AppWindowId::default();
        let _ = app
            .models_mut()
            .update(workbench.window_layout(), |layout| {
                layout
                    .pane_tree
                    .find_pane_mut("pane-a")
                    .unwrap()
                    .tabs
                    .set_dirty(Arc::from("doc-a"), true);
            });
        let close_doc_a = crate::commands::tab_close_command("doc-a").unwrap();

        let blocked = workbench.apply_command(&mut app, window, &close_doc_a);
        assert!(blocked.blocked_dirty_close);
        assert_eq!(blocked.target.as_deref(), Some("pane-a/doc-a"));
        assert_eq!(
            workbench
                .pending_dirty_close(app.models())
                .unwrap()
                .target
                .as_deref(),
            Some("pane-a/doc-a")
        );

        let discarded = workbench.apply_command(
            &mut app,
            window,
            &CommandId::from(CMD_WORKSPACE_DIRTY_CLOSE_DISCARD),
        );
        assert!(discarded.applied);
        assert_eq!(discarded.target.as_deref(), Some("pane-a/doc-a"));
        assert_eq!(
            app.models()
                .read(workbench.window_layout(), |layout| {
                    layout
                        .pane_tree
                        .find_pane("pane-a")
                        .unwrap()
                        .tabs
                        .tabs()
                        .to_vec()
                })
                .unwrap(),
            vec![Arc::<str>::from("doc-b")]
        );
    }

    #[test]
    fn window_dirty_close_cancel_restores_the_captured_focus() {
        let (mut app, workbench) = workbench(true);
        let original_focus = GlobalElementId(41);
        let modal_focus = GlobalElementId(42);
        let _ = app
            .models_mut()
            .update(workbench.window_layout(), |layout| {
                layout
                    .pane_tree
                    .find_pane_mut("pane-a")
                    .unwrap()
                    .tabs
                    .set_dirty(Arc::from("doc-b"), true);
            });
        let request = app
            .models()
            .read(workbench.window_layout(), |layout| {
                layout.dirty_close_request_for_window_close().unwrap()
            })
            .unwrap();
        workbench.open_dirty_close_prompt(
            app.models_mut(),
            WorkspaceDirtyClosePrompt {
                pane_id: Arc::from("<window>"),
                command: CommandId::from("window.close"),
                target: Some(Arc::from("window")),
                request,
                focus_restore_target: Some(original_focus),
                focus_fallback: None,
            },
        );

        let canceled = workbench.resolve_dirty_close(
            app.models_mut(),
            &CommandId::from(CMD_WORKSPACE_DIRTY_CLOSE_CANCEL),
            false,
            Some(modal_focus),
        );

        assert!(!canceled.applied);
        assert_eq!(
            canceled.focus,
            Some(WorkspaceWorkbenchFocusRequest {
                guard: WorkspaceWorkbenchFocusGuard::Unchanged(modal_focus),
                target: Some(original_focus),
                fallback: None,
            })
        );
    }

    #[test]
    fn closing_focused_active_tab_requests_focus_on_the_survivor() {
        let (mut app, workbench) = workbench(false);
        let close = typed_command_id::<act::WorkspaceTabClose>();

        let outcome = workbench.apply_command_with_focus(
            app.models_mut(),
            &close,
            WorkspaceCommandScopeFocusSnapshot {
                target: None,
                lane: Some(WorkspaceCommandScopeFocusLane::TabStrip(Arc::from(
                    "pane-a",
                ))),
            },
        );

        assert!(outcome.applied);
        assert_eq!(
            outcome.focus,
            Some(WorkspaceWorkbenchFocusRequest {
                guard: WorkspaceWorkbenchFocusGuard::NoLiveFocus,
                target: None,
                fallback: Some(WorkspaceWorkbenchFocusFallback::ActiveTabStrip),
            })
        );
    }

    #[test]
    fn dirty_tab_close_without_a_focus_lane_keeps_a_survivor_fallback() {
        let (mut app, workbench) = workbench(true);
        let _ = app
            .models_mut()
            .update(workbench.window_layout(), |layout| {
                layout
                    .pane_tree
                    .find_pane_mut("pane-a")
                    .unwrap()
                    .tabs
                    .set_dirty(Arc::from("doc-b"), true);
            });
        let close = typed_command_id::<act::WorkspaceTabClose>();
        let modal_focus = GlobalElementId(44);

        let blocked = workbench.apply_command_with_focus(
            app.models_mut(),
            &close,
            WorkspaceCommandScopeFocusSnapshot {
                target: Some(modal_focus),
                lane: None,
            },
        );
        assert!(blocked.blocked_dirty_close);

        let saved =
            workbench.confirm_dirty_close_saved(&mut app, &CommandDispatchSourceV1::programmatic());
        assert_eq!(
            saved.focus,
            Some(WorkspaceWorkbenchFocusRequest {
                guard: WorkspaceWorkbenchFocusGuard::Authoritative,
                target: None,
                fallback: Some(WorkspaceWorkbenchFocusFallback::ActiveTabStrip),
            })
        );
    }

    #[test]
    fn overlay_close_by_id_restores_the_active_tabstrip_after_the_menu_unmounts() {
        let (mut app, workbench) = workbench(false);
        let close = crate::commands::tab_close_command("doc-a").unwrap();
        let overlay_target = GlobalElementId(41);

        let outcome = workbench.apply_command_with_focus(
            app.models_mut(),
            &close,
            WorkspaceCommandScopeFocusSnapshot {
                target: Some(overlay_target),
                lane: Some(WorkspaceCommandScopeFocusLane::TabStrip(Arc::from(
                    "pane-a",
                ))),
            },
        );

        assert!(outcome.applied);
        assert_eq!(
            outcome.focus,
            Some(WorkspaceWorkbenchFocusRequest {
                guard: WorkspaceWorkbenchFocusGuard::Unchanged(overlay_target),
                target: Some(overlay_target),
                fallback: Some(WorkspaceWorkbenchFocusFallback::ActiveTabStrip),
            })
        );
    }

    #[test]
    fn dirty_close_cancel_keeps_the_overlay_target_and_workspace_lane_fallback() {
        let (mut app, workbench) = workbench(true);
        let _ = app
            .models_mut()
            .update(workbench.window_layout(), |layout| {
                layout
                    .pane_tree
                    .find_pane_mut("pane-a")
                    .unwrap()
                    .tabs
                    .set_dirty(Arc::from("doc-b"), true);
            });
        let close = typed_command_id::<act::WorkspaceTabClose>();
        let overlay_target = GlobalElementId(42);
        let blocked = workbench.apply_command_with_focus(
            app.models_mut(),
            &close,
            WorkspaceCommandScopeFocusSnapshot {
                target: Some(overlay_target),
                lane: Some(WorkspaceCommandScopeFocusLane::TabStrip(Arc::from(
                    "pane-a",
                ))),
            },
        );
        assert!(blocked.blocked_dirty_close);
        let modal_focus = GlobalElementId(43);

        let canceled = workbench.apply_command_with_focus(
            app.models_mut(),
            &CommandId::from(CMD_WORKSPACE_DIRTY_CLOSE_CANCEL),
            WorkspaceCommandScopeFocusSnapshot {
                target: Some(modal_focus),
                lane: None,
            },
        );

        assert_eq!(
            canceled.focus,
            Some(WorkspaceWorkbenchFocusRequest {
                guard: WorkspaceWorkbenchFocusGuard::Unchanged(modal_focus),
                target: Some(overlay_target),
                fallback: Some(WorkspaceWorkbenchFocusFallback::ActiveTabStrip),
            })
        );
        assert!(!workbench.has_pending_dirty_close(app.models()));
    }

    #[test]
    fn typed_pane_focus_follows_the_tabstrip_lane_to_the_new_active_pane() {
        let (mut app, workbench) = two_pane_workbench();
        let command = typed_command_id::<act::WorkspacePaneFocusRight>();
        let original_focus = GlobalElementId(43);

        let outcome = workbench.apply_command_with_focus(
            app.models_mut(),
            &command,
            WorkspaceCommandScopeFocusSnapshot {
                target: Some(original_focus),
                lane: Some(WorkspaceCommandScopeFocusLane::TabStrip(Arc::from(
                    "pane-a",
                ))),
            },
        );

        assert!(outcome.applied);
        assert_eq!(
            app.models()
                .read(workbench.window_layout(), |layout| layout
                    .active_pane_id()
                    .cloned())
                .unwrap()
                .as_deref(),
            Some("pane-b")
        );
        assert_eq!(
            outcome.focus,
            Some(WorkspaceWorkbenchFocusRequest {
                guard: WorkspaceWorkbenchFocusGuard::Unchanged(original_focus),
                target: None,
                fallback: Some(WorkspaceWorkbenchFocusFallback::ActiveTabStrip),
            })
        );
    }

    #[test]
    fn typed_tab_move_follows_the_content_lane_to_the_target_pane() {
        let (mut app, workbench) = two_pane_workbench();
        let command = typed_command_id::<act::WorkspacePaneMoveActiveTabRight>();

        let outcome = workbench.apply_command_with_focus(
            app.models_mut(),
            &command,
            WorkspaceCommandScopeFocusSnapshot {
                target: None,
                lane: Some(WorkspaceCommandScopeFocusLane::PaneContent(Arc::from(
                    "pane-a",
                ))),
            },
        );

        assert!(outcome.applied);
        assert_eq!(
            app.models()
                .read(workbench.window_layout(), |layout| layout
                    .active_pane_id()
                    .cloned())
                .unwrap()
                .as_deref(),
            Some("pane-b")
        );
        assert_eq!(
            outcome.focus,
            Some(WorkspaceWorkbenchFocusRequest {
                guard: WorkspaceWorkbenchFocusGuard::NoLiveFocus,
                target: None,
                fallback: Some(WorkspaceWorkbenchFocusFallback::ActivePaneContent),
            })
        );
    }
}
