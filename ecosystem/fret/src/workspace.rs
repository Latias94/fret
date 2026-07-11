//! Workspace app-authoring entry points.
//!
//! This module is an explicit `fret/workspace` lane: `fret-workspace` still owns shell concepts,
//! while the `fret` facade owns desktop app startup, defaults, diagnostics, assets, and config
//! layering.

use std::sync::Arc;

use crate::{Defaults, FretApp, Result, UiAppBuilder, UiAppDriver};

pub use fret_workspace::{
    WorkspaceCommandScope, WorkspaceFrame, WorkspacePaneContentFocusTarget, WorkspaceTab,
    WorkspaceTabStrip, WorkspaceWorkbench, WorkspaceWorkbenchCommandOutcome,
    WorkspaceWorkbenchFocusFallback, WorkspaceWorkbenchFocusGuard, WorkspaceWorkbenchFocusRequest,
    workspace_pane_tree_element_with_resize,
};
pub use fret_workspace::{
    commands::{self, register_workspace_commands, typed_command_id},
    layout, menu, panes, tabs,
};

/// Install the default workspace command registry entries.
pub fn install(app: &mut crate::app::App) {
    register_workspace_commands(app.commands_mut());
}

#[cfg(all(not(target_arch = "wasm32"), feature = "desktop"))]
pub trait WorkspaceWindowState: crate::app::UiAppFrameStageSink {
    fn workspace_workbench(&self) -> &WorkspaceWorkbench;

    /// Persist every dirty item in `request` before a Save & Close decision is committed.
    ///
    /// The default is fail-closed: returning `false` keeps the prompt open and leaves the
    /// workspace layout unchanged.
    fn save_workspace_dirty_close(
        &mut self,
        _app: &mut crate::app::App,
        _window: crate::WindowId,
        _request: &fret_workspace::close_policy::WorkspaceDirtyCloseRequest,
    ) -> bool {
        false
    }

    /// Override the default Workbench response to a window-close request.
    ///
    /// Returning `Some` is authoritative, including an outcome that deliberately keeps the
    /// window open. Returning `None` delegates to `WorkspaceWorkbench`.
    fn handle_workspace_window_close(
        &mut self,
        _app: &mut crate::app::App,
        _window: crate::WindowId,
    ) -> Option<WorkspaceWorkbenchCommandOutcome> {
        None
    }

    fn handle_workspace_command(
        &mut self,
        _app: &mut crate::app::App,
        _window: crate::WindowId,
        _command: &fret_runtime::CommandId,
        _source: &fret_runtime::CommandDispatchSourceV1,
    ) -> Option<WorkspaceWorkbenchCommandOutcome> {
        None
    }

    fn handle_workspace_event(
        &mut self,
        _app: &mut crate::app::App,
        _window: crate::WindowId,
        _event: &fret_core::Event,
    ) {
    }

    fn handle_workspace_global_changes(
        &mut self,
        _app: &mut crate::app::App,
        _window: crate::WindowId,
        _changed: &[std::any::TypeId],
    ) {
    }
}

#[cfg(all(not(target_arch = "wasm32"), feature = "desktop"))]
fn resolve_workbench_window_close<S: WorkspaceWindowState>(
    app: &mut crate::app::App,
    window: crate::WindowId,
    state: &mut S,
) -> WorkspaceWorkbenchCommandOutcome {
    state
        .handle_workspace_window_close(app, window)
        .unwrap_or_else(|| {
            let workbench = state.workspace_workbench().clone();
            workbench.request_window_close(app, window)
        })
}

#[cfg(all(not(target_arch = "wasm32"), feature = "desktop"))]
fn apply_workbench_outcome(
    app: &mut crate::app::App,
    window: crate::WindowId,
    outcome: &WorkspaceWorkbenchCommandOutcome,
) {
    if outcome.close_window {
        app.push_effect(fret_runtime::Effect::Window(
            fret_app::WindowRequest::Close(window),
        ));
    }
    if let Some(focus) = outcome.focus.as_ref() {
        let fallback_command = focus.fallback.map(|fallback| fallback.command_id());
        fret_bootstrap::ui_app_driver::defer_ui_focus_until_after_frame(
            app,
            window,
            match focus.guard {
                WorkspaceWorkbenchFocusGuard::NoLiveFocus => {
                    fret_bootstrap::ui_app_driver::PostFrameUiFocusGuard::NoLiveFocus
                }
                WorkspaceWorkbenchFocusGuard::Unchanged(target) => {
                    fret_bootstrap::ui_app_driver::PostFrameUiFocusGuard::Unchanged(target)
                }
                WorkspaceWorkbenchFocusGuard::Authoritative => {
                    fret_bootstrap::ui_app_driver::PostFrameUiFocusGuard::Authoritative
                }
            },
            focus.target,
            fallback_command,
        );
    }
    if outcome.handled {
        app.request_redraw(window);
    }
}

#[cfg(all(not(target_arch = "wasm32"), feature = "desktop"))]
fn record_workbench_outcome(
    app: &mut crate::app::App,
    window: crate::WindowId,
    command: &fret_runtime::CommandId,
    outcome: &WorkspaceWorkbenchCommandOutcome,
) {
    if !outcome.handled {
        return;
    }
    app.with_global_mut(
        fret_runtime::WindowPendingCommandDispatchOutcomeService::default,
        |service, app| {
            service.record(
                window,
                app.tick_id(),
                command.clone(),
                fret_runtime::CommandDispatchOutcomeV1::from(outcome),
            );
        },
    );
}

#[cfg(all(not(target_arch = "wasm32"), feature = "desktop"))]
fn workspace_ui_has_modal(app: &crate::app::App, window: crate::WindowId) -> bool {
    app.global::<fret_runtime::WindowInputContextService>()
        .and_then(|service| service.snapshot(window))
        .is_some_and(|input| input.ui_has_modal)
}

#[cfg(all(not(target_arch = "wasm32"), feature = "desktop"))]
#[derive(Clone, Copy, Default)]
struct WorkspaceCommandRoutingContext {
    ui_has_modal: bool,
    source_is_within_active_modal: bool,
}

#[cfg(all(not(target_arch = "wasm32"), feature = "desktop"))]
impl WorkspaceCommandRoutingContext {
    fn active_modal_ui_action(self, source: &fret_runtime::CommandDispatchSourceV1) -> bool {
        self.source_is_within_active_modal
            && matches!(
                source.kind,
                fret_runtime::CommandDispatchSourceKindV1::Pointer
                    | fret_runtime::CommandDispatchSourceKindV1::Keyboard
            )
    }
}

#[cfg(all(not(target_arch = "wasm32"), feature = "desktop"))]
fn handle_workbench_command_with_routing<S: WorkspaceWindowState>(
    app: &mut crate::app::App,
    window: crate::WindowId,
    state: &mut S,
    command: &fret_runtime::CommandId,
    source: &fret_runtime::CommandDispatchSourceV1,
    routing: WorkspaceCommandRoutingContext,
) -> bool {
    let workbench = state.workspace_workbench().clone();
    let pending_dirty_close = workbench.pending_dirty_close(app.models());
    if routing.ui_has_modal {
        if fret_workspace::commands::is_workspace_dirty_close_resolution(command)
            || fret_workspace::commands::is_workspace_model_command(command)
        {
            if !routing.active_modal_ui_action(source) {
                return false;
            }
        } else if fret_workspace::commands::is_workspace_ui_command(command)
            || command.as_str() == "window.close"
        {
            return false;
        }
    }
    let outcome = if let Some(prompt) = pending_dirty_close {
        if command.as_str() == "window.close" {
            resolve_workbench_window_close(app, window, state)
        } else if command.as_str()
            == fret_workspace::commands::CMD_WORKSPACE_DIRTY_CLOSE_SAVE_AND_CLOSE
            && state.save_workspace_dirty_close(app, window, &prompt.request)
        {
            workbench.confirm_dirty_close_saved(app, source)
        } else {
            workbench.apply_command(app, window, command)
        }
    } else if command.as_str() == "window.close" {
        resolve_workbench_window_close(app, window, state)
    } else {
        state
            .handle_workspace_command(app, window, command, source)
            .unwrap_or_else(|| workbench.apply_command(app, window, command))
    };
    record_workbench_outcome(app, window, command, &outcome);
    apply_workbench_outcome(app, window, &outcome);
    outcome.handled
}

#[cfg(all(not(target_arch = "wasm32"), feature = "desktop"))]
#[cfg(test)]
fn handle_workbench_command<S: WorkspaceWindowState>(
    app: &mut crate::app::App,
    window: crate::WindowId,
    state: &mut S,
    command: &fret_runtime::CommandId,
    source: &fret_runtime::CommandDispatchSourceV1,
) -> bool {
    let ui_has_modal = workspace_ui_has_modal(app, window);
    handle_workbench_command_with_routing(
        app,
        window,
        state,
        command,
        source,
        WorkspaceCommandRoutingContext {
            ui_has_modal,
            source_is_within_active_modal: false,
        },
    )
}

#[cfg(all(not(target_arch = "wasm32"), feature = "desktop"))]
fn handle_workbench_command_from_context<S: WorkspaceWindowState>(
    app: &mut crate::app::App,
    window: crate::WindowId,
    state: &mut S,
    command: &fret_runtime::CommandId,
    context: fret_bootstrap::ui_app_driver::UiAppCommandBeforeUiContext<'_>,
) -> bool {
    handle_workbench_command_with_routing(
        app,
        window,
        state,
        command,
        context.source,
        WorkspaceCommandRoutingContext {
            ui_has_modal: context.ui_has_modal,
            source_is_within_active_modal: context.source_is_within_active_input_barrier_scope,
        },
    )
}

#[cfg(all(not(target_arch = "wasm32"), feature = "desktop"))]
fn handle_workbench_event<S: WorkspaceWindowState>(
    app: &mut crate::app::App,
    window: crate::WindowId,
    state: &mut S,
    event: &fret_core::Event,
) {
    state.handle_workspace_event(app, window, event);
    if matches!(event, fret_core::Event::WindowCloseRequested) {
        if workspace_ui_has_modal(app, window) {
            return;
        }
        let outcome = resolve_workbench_window_close(app, window, state);
        apply_workbench_outcome(app, window, &outcome);
    }
}

#[cfg(all(not(target_arch = "wasm32"), feature = "desktop"))]
fn handle_workbench_global_changes<S: WorkspaceWindowState>(
    app: &mut crate::app::App,
    window: crate::WindowId,
    state: &mut S,
    changed: &[std::any::TypeId],
) {
    state.handle_workspace_global_changes(app, window, changed);
}

#[derive(Clone)]
struct WorkspaceMenuInstaller {
    commands: fret_workspace::menu::WorkspaceMenuCommands,
}

impl crate::integration::InstallIntoApp for WorkspaceMenuInstaller {
    fn install_into_app(self, app: &mut crate::app::App) {
        let menu_bar = fret_workspace::menu::workspace_default_menu_bar(self.commands);
        fret_app::set_menu_bar_baseline(app, menu_bar);
        fret_app::sync_os_menu_bar(app);
    }
}

/// Builder facade for editor/workbench-style apps.
///
/// `WorkspaceApp` keeps ordinary workspace startup on a named lane:
/// - workspace command metadata is registered before default keybindings/config layering,
/// - the workspace menu baseline is installed from `fret-workspace`'s data-only menu model,
/// - frame, retained UI tree, diagnostics, and layout/paint lifecycle remain owned by
///   `UiAppDriver` through the final `view` / `ui` builder.
pub struct WorkspaceApp {
    app: FretApp,
    menu_commands: Option<fret_workspace::menu::WorkspaceMenuCommands>,
}

impl WorkspaceApp {
    /// Create a workspace app builder with default workspace commands and menu wiring.
    pub fn new(root_name: &'static str) -> Self {
        let mut menu_commands = fret_workspace::menu::WorkspaceMenuCommands::default();
        menu_commands.app_menu_title = Some(Arc::from(root_name));
        Self {
            app: FretApp::new(root_name).setup(install),
            menu_commands: Some(menu_commands),
        }
    }

    /// Wrap an existing `FretApp` in the workspace startup lane.
    pub fn from_app(app: FretApp) -> Self {
        Self {
            app: app.setup(install),
            menu_commands: Some(fret_workspace::menu::WorkspaceMenuCommands::default()),
        }
    }

    /// Return the underlying app builder after installing workspace menu wiring.
    pub fn into_fret_app(self) -> FretApp {
        let Self { app, menu_commands } = self;
        if let Some(commands) = menu_commands {
            app.setup(WorkspaceMenuInstaller { commands })
        } else {
            app
        }
    }

    /// Override the workspace menu command model.
    pub fn menu_commands(mut self, commands: fret_workspace::menu::WorkspaceMenuCommands) -> Self {
        self.menu_commands = Some(commands);
        self
    }

    /// Mutate the default workspace menu command model in place.
    pub fn map_menu_commands(
        mut self,
        f: impl FnOnce(&mut fret_workspace::menu::WorkspaceMenuCommands),
    ) -> Self {
        let mut commands = self.menu_commands.unwrap_or_default();
        f(&mut commands);
        self.menu_commands = Some(commands);
        self
    }

    /// Disable the default workspace menu baseline.
    pub fn without_default_menu(mut self) -> Self {
        self.menu_commands = None;
        self
    }

    /// Override runtime defaults applied by the `fret` facade.
    pub fn defaults(mut self, defaults: Defaults) -> Self {
        self.app = self.app.defaults(defaults);
        self
    }

    /// Run app-level setup during bootstrap.
    pub fn setup<T>(mut self, setup: T) -> Self
    where
        T: crate::integration::InstallIntoApp + 'static,
    {
        self.app = self.app.setup(setup);
        self
    }
}

#[cfg(all(not(target_arch = "wasm32"), feature = "desktop"))]
impl WorkspaceApp {
    /// Enable the command palette if the `command-palette` feature is available.
    #[cfg(feature = "command-palette")]
    pub fn command_palette(mut self, enabled: bool) -> Self {
        self.app = self.app.command_palette(enabled);
        self
    }

    /// Configure the main window title and initial size.
    pub fn window(mut self, title: impl Into<String>, size: (f64, f64)) -> Self {
        self.app = self.app.window(title, size);
        self
    }

    /// Configure the minimum logical surface size for the main window.
    pub fn window_min_size(mut self, size: (f64, f64)) -> Self {
        self.app = self.app.window_min_size(size);
        self
    }

    /// Configure the maximum logical surface size for the main window.
    pub fn window_max_size(mut self, size: (f64, f64)) -> Self {
        self.app = self.app.window_max_size(size);
        self
    }

    /// Configure the logical surface resize increments for the main window.
    pub fn window_resize_increments(mut self, size: (f64, f64)) -> Self {
        self.app = self.app.window_resize_increments(size);
        self
    }

    /// Build a `View`-runtime workspace app.
    pub fn view<V: crate::view::View>(
        self,
    ) -> Result<UiAppBuilder<crate::view::ViewWindowState<V>>> {
        self.into_fret_app().view::<V>()
    }

    /// Build a `View`-runtime workspace app while preserving driver hook configuration.
    pub fn view_with_hooks<V: crate::view::View>(
        self,
        configure: fn(
            UiAppDriver<crate::view::ViewWindowState<V>>,
        ) -> UiAppDriver<crate::view::ViewWindowState<V>>,
    ) -> Result<UiAppBuilder<crate::view::ViewWindowState<V>>> {
        self.into_fret_app().view_with_hooks::<V>(configure)
    }

    /// Build a workspace app from a retained state initializer and render function.
    ///
    /// This keeps workspace authors off raw `FnDriver` / `UiTree` / frame lifecycle ownership
    /// while allowing editor-grade shells to keep explicit app-owned state models.
    pub fn ui<S: WorkspaceWindowState + 'static>(
        self,
        init_window: fn(&mut crate::app::App, crate::WindowId) -> S,
        view: for<'a> fn(&mut crate::AppRenderCx<'a>, &mut S) -> crate::Ui,
    ) -> Result<UiAppBuilder<S>> {
        self.into_fret_app()
            .ui_driver_with_hooks(init_window, view, |driver| {
                driver
                    .close_on_window_close_requested(false)
                    .on_app_event(handle_workbench_event::<S>)
                    .on_app_command_before_ui(handle_workbench_command_from_context::<S>)
                    .on_app_global_changes(handle_workbench_global_changes::<S>)
                    .record_frame_stages()
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::integration::InstallIntoApp;
    use fret_app::effective_menu_bar;
    use fret_runtime::{CommandId, InputContext, KeyChord, KeymapService, Platform};

    struct WorkbenchTestState {
        workbench: WorkspaceWorkbench,
        save_succeeds: bool,
        override_window_close: bool,
        custom_command_count: usize,
    }

    impl crate::app::UiAppFrameStageSink for WorkbenchTestState {
        fn record_frame_stage(&mut self, _observation: crate::app::UiAppFrameObservation) {}
    }

    impl WorkspaceWindowState for WorkbenchTestState {
        fn workspace_workbench(&self) -> &WorkspaceWorkbench {
            &self.workbench
        }

        fn save_workspace_dirty_close(
            &mut self,
            _app: &mut crate::app::App,
            _window: crate::WindowId,
            _request: &fret_workspace::close_policy::WorkspaceDirtyCloseRequest,
        ) -> bool {
            self.save_succeeds
        }

        fn handle_workspace_window_close(
            &mut self,
            _app: &mut crate::app::App,
            _window: crate::WindowId,
        ) -> Option<WorkspaceWorkbenchCommandOutcome> {
            self.override_window_close
                .then(|| WorkspaceWorkbenchCommandOutcome {
                    handled: true,
                    action_id: None,
                    target: Some(Arc::from("window")),
                    applied: false,
                    blocked_dirty_close: false,
                    close_window: false,
                    focus: None,
                })
        }

        fn handle_workspace_command(
            &mut self,
            _app: &mut crate::app::App,
            _window: crate::WindowId,
            command: &fret_runtime::CommandId,
            _source: &fret_runtime::CommandDispatchSourceV1,
        ) -> Option<WorkspaceWorkbenchCommandOutcome> {
            (command.as_str() == "app.custom").then(|| {
                self.custom_command_count += 1;
                WorkspaceWorkbenchCommandOutcome {
                    handled: true,
                    action_id: None,
                    target: Some(Arc::from("custom")),
                    applied: true,
                    blocked_dirty_close: false,
                    close_window: false,
                    focus: None,
                }
            })
        }
    }

    #[test]
    fn workspace_install_registers_commands_before_default_keybindings() {
        let mut app = crate::app::App::new();
        install(&mut app);
        fret_app::install_command_default_keybindings_into_keymap(&mut app);

        let command = fret_workspace::commands::typed_command_id::<
            fret_workspace::commands::act::WorkspacePaneToggleTabStripFocus,
        >();
        assert!(app.commands().get(command.clone()).is_some());

        let ctx = InputContext::fallback(Platform::Macos, Default::default());
        let chord = KeyChord::new(
            fret_core::KeyCode::F6,
            fret_core::Modifiers {
                ctrl: true,
                ..Default::default()
            },
        );
        let resolved = app
            .global::<KeymapService>()
            .and_then(|svc| svc.keymap.resolve(&ctx, chord));
        assert_eq!(resolved.as_ref(), Some(&command));
    }

    #[test]
    fn workspace_menu_installer_sets_menu_baseline() {
        let mut app = crate::app::App::new();
        let mut commands = fret_workspace::menu::WorkspaceMenuCommands::default();
        commands.app_menu_title = Some(Arc::from("Workbench"));

        WorkspaceMenuInstaller { commands }.install_into_app(&mut app);

        let menu_bar = effective_menu_bar(&app).expect("workspace menu baseline");
        assert!(
            menu_bar
                .menus
                .iter()
                .flat_map(|menu| &menu.items)
                .any(|item| matches!(
                    item,
                    fret_runtime::MenuItem::Command { command, .. }
                        if command == &CommandId::new(fret_workspace::commands::CMD_WORKSPACE_TAB_NEXT)
                ))
        );
    }

    #[test]
    fn workbench_command_bridge_records_typed_target_and_dirty_close_outcome() {
        let mut app = crate::app::App::new();
        let window = crate::WindowId::default();
        let mut layout = fret_workspace::layout::WorkspaceWindowLayout::new("window", "pane-a");
        layout.pane_tree = fret_workspace::layout::WorkspacePaneTree::leaf("pane-a");
        let pane = layout.pane_tree.find_pane_mut("pane-a").unwrap();
        pane.tabs.open_and_activate(Arc::from("doc-a"));
        pane.tabs.set_dirty(Arc::from("doc-a"), true);
        let layout = app.models_mut().insert(layout);
        let workbench = WorkspaceWorkbench::new(app.models_mut(), layout, true);
        let mut state = WorkbenchTestState {
            workbench,
            save_succeeds: false,
            override_window_close: false,
            custom_command_count: 0,
        };
        let command = fret_workspace::commands::typed_command_id::<
            fret_workspace::commands::act::WorkspaceTabClose,
        >();
        let source = fret_runtime::CommandDispatchSourceV1 {
            kind: fret_runtime::CommandDispatchSourceKindV1::Shortcut,
            element: None,
            test_id: None,
        };

        assert!(handle_workbench_command(
            &mut app, window, &mut state, &command, &source,
        ));

        let outcome = app.with_global_mut(
            fret_runtime::WindowPendingCommandDispatchOutcomeService::default,
            |service, app| service.consume(window, app.tick_id(), &command),
        );
        assert_eq!(
            outcome,
            Some(fret_runtime::CommandDispatchOutcomeV1 {
                action_id: Some(command),
                target: Some(Arc::from("pane-a/doc-a")),
                applied: false,
                blocked_dirty_close: true,
            })
        );
    }

    #[test]
    fn save_and_close_requires_app_persistence_success() {
        let mut app = crate::app::App::new();
        let window = crate::WindowId::default();
        let mut layout = fret_workspace::layout::WorkspaceWindowLayout::new("window", "pane-a");
        layout.pane_tree = fret_workspace::layout::WorkspacePaneTree::leaf("pane-a");
        let pane = layout.pane_tree.find_pane_mut("pane-a").unwrap();
        pane.tabs.open_and_activate(Arc::from("doc-a"));
        pane.tabs.set_dirty(Arc::from("doc-a"), true);
        let layout = app.models_mut().insert(layout);
        let workbench = WorkspaceWorkbench::new(app.models_mut(), layout, true);
        let mut state = WorkbenchTestState {
            workbench,
            save_succeeds: false,
            override_window_close: false,
            custom_command_count: 0,
        };
        let close = fret_workspace::commands::typed_command_id::<
            fret_workspace::commands::act::WorkspaceTabClose,
        >();
        let save = fret_workspace::commands::typed_command_id::<
            fret_workspace::commands::act::WorkspaceDirtyCloseSaveAndClose,
        >();
        let source = fret_runtime::CommandDispatchSourceV1::programmatic();
        assert!(handle_workbench_command(
            &mut app, window, &mut state, &close, &source,
        ));

        let window_close = CommandId::from("window.close");
        assert!(handle_workbench_command(
            &mut app,
            window,
            &mut state,
            &window_close,
            &source,
        ));
        assert!(state.workbench.has_pending_dirty_close(app.models()));

        assert!(handle_workbench_command(
            &mut app, window, &mut state, &save, &source,
        ));
        assert!(state.workbench.has_pending_dirty_close(app.models()));
        let still_open = app
            .models()
            .read(state.workbench.window_layout(), |layout| {
                layout
                    .pane_tree
                    .find_pane("pane-a")
                    .unwrap()
                    .tabs
                    .tabs()
                    .len()
            })
            .unwrap();
        assert_eq!(still_open, 1);

        state.save_succeeds = true;
        assert!(handle_workbench_command(
            &mut app, window, &mut state, &save, &source,
        ));
        assert!(!state.workbench.has_pending_dirty_close(app.models()));
        let remaining = app
            .models()
            .read(state.workbench.window_layout(), |layout| {
                layout
                    .pane_tree
                    .find_pane("pane-a")
                    .unwrap()
                    .tabs
                    .tabs()
                    .len()
            })
            .unwrap();
        assert_eq!(remaining, 0);
    }

    #[test]
    fn modal_snapshot_defers_external_workspace_commands_but_allows_active_ui_actions() {
        let mut app = crate::app::App::new();
        let window = crate::WindowId::default();
        let mut layout = fret_workspace::layout::WorkspaceWindowLayout::new("window", "pane-a");
        layout.pane_tree = fret_workspace::layout::WorkspacePaneTree::leaf("pane-a");
        let pane = layout.pane_tree.find_pane_mut("pane-a").unwrap();
        pane.tabs.open_and_activate(Arc::from("doc-a"));
        pane.tabs.open_and_activate(Arc::from("doc-b"));
        pane.tabs.set_dirty(Arc::from("doc-b"), true);
        let layout = app.models_mut().insert(layout);
        let workbench = WorkspaceWorkbench::new(app.models_mut(), layout, true);
        let mut state = WorkbenchTestState {
            workbench,
            save_succeeds: false,
            override_window_close: false,
            custom_command_count: 0,
        };
        app.with_global_mut(
            fret_runtime::WindowInputContextService::default,
            |service, _app| {
                service.set_snapshot(
                    window,
                    fret_runtime::InputContext {
                        ui_has_modal: true,
                        ..Default::default()
                    },
                );
            },
        );
        let next = fret_workspace::commands::typed_command_id::<
            fret_workspace::commands::act::WorkspaceTabNext,
        >();

        assert!(!handle_workbench_command(
            &mut app,
            window,
            &mut state,
            &next,
            &fret_runtime::CommandDispatchSourceV1::programmatic(),
        ));
        assert_eq!(
            app.models()
                .read(state.workbench.window_layout(), |layout| {
                    layout
                        .pane_tree
                        .find_pane("pane-a")
                        .unwrap()
                        .tabs
                        .active()
                        .cloned()
                })
                .unwrap()
                .as_deref(),
            Some("doc-b")
        );

        let shortcut_source = fret_runtime::CommandDispatchSourceV1 {
            kind: fret_runtime::CommandDispatchSourceKindV1::Shortcut,
            element: Some(42),
            test_id: Some(Arc::from("modal-shortcut-source")),
        };
        assert!(!handle_workbench_command_with_routing(
            &mut app,
            window,
            &mut state,
            &next,
            &shortcut_source,
            WorkspaceCommandRoutingContext {
                ui_has_modal: true,
                source_is_within_active_modal: true,
            },
        ));

        let menu_source = fret_runtime::CommandDispatchSourceV1 {
            kind: fret_runtime::CommandDispatchSourceKindV1::Pointer,
            element: Some(43),
            test_id: Some(Arc::from("modal-menu-item")),
        };
        assert!(handle_workbench_command_with_routing(
            &mut app,
            window,
            &mut state,
            &next,
            &menu_source,
            WorkspaceCommandRoutingContext {
                ui_has_modal: true,
                source_is_within_active_modal: true,
            },
        ));
        assert_eq!(
            app.models()
                .read(state.workbench.window_layout(), |layout| {
                    layout
                        .pane_tree
                        .find_pane("pane-a")
                        .unwrap()
                        .tabs
                        .active()
                        .cloned()
                })
                .unwrap()
                .as_deref(),
            Some("doc-a")
        );

        let custom = CommandId::from("app.custom");
        assert!(handle_workbench_command(
            &mut app,
            window,
            &mut state,
            &custom,
            &fret_runtime::CommandDispatchSourceV1::programmatic(),
        ));
        assert_eq!(state.custom_command_count, 1);

        handle_workbench_event(
            &mut app,
            window,
            &mut state,
            &fret_core::Event::WindowCloseRequested,
        );
        assert!(!state.workbench.has_pending_dirty_close(app.models()));
    }

    #[test]
    fn pending_dirty_close_resolution_requires_active_modal_provenance() {
        let mut app = crate::app::App::new();
        let window = crate::WindowId::default();
        let mut layout = fret_workspace::layout::WorkspaceWindowLayout::new("window", "pane-a");
        layout.pane_tree = fret_workspace::layout::WorkspacePaneTree::leaf("pane-a");
        let pane = layout.pane_tree.find_pane_mut("pane-a").unwrap();
        pane.tabs.open_and_activate(Arc::from("doc-a"));
        pane.tabs.set_dirty(Arc::from("doc-a"), true);
        let layout = app.models_mut().insert(layout);
        let workbench = WorkspaceWorkbench::new(app.models_mut(), layout, true);
        let mut state = WorkbenchTestState {
            workbench,
            save_succeeds: true,
            override_window_close: false,
            custom_command_count: 0,
        };
        let close = fret_workspace::commands::typed_command_id::<
            fret_workspace::commands::act::WorkspaceTabClose,
        >();
        assert!(handle_workbench_command(
            &mut app,
            window,
            &mut state,
            &close,
            &fret_runtime::CommandDispatchSourceV1::programmatic(),
        ));
        assert!(state.workbench.has_pending_dirty_close(app.models()));

        let external_routing = WorkspaceCommandRoutingContext {
            ui_has_modal: true,
            source_is_within_active_modal: false,
        };
        for command in [
            fret_workspace::commands::CMD_WORKSPACE_DIRTY_CLOSE_CANCEL,
            fret_workspace::commands::CMD_WORKSPACE_DIRTY_CLOSE_DISCARD,
            fret_workspace::commands::CMD_WORKSPACE_DIRTY_CLOSE_SAVE_AND_CLOSE,
        ] {
            assert!(!handle_workbench_command_with_routing(
                &mut app,
                window,
                &mut state,
                &CommandId::from(command),
                &fret_runtime::CommandDispatchSourceV1::programmatic(),
                external_routing,
            ));
            assert!(state.workbench.has_pending_dirty_close(app.models()));
        }

        let discard = CommandId::from(fret_workspace::commands::CMD_WORKSPACE_DIRTY_CLOSE_DISCARD);
        assert!(!handle_workbench_command_with_routing(
            &mut app,
            window,
            &mut state,
            &discard,
            &fret_runtime::CommandDispatchSourceV1 {
                kind: fret_runtime::CommandDispatchSourceKindV1::Shortcut,
                element: Some(42),
                test_id: Some(Arc::from("dirty-close.shortcut")),
            },
            WorkspaceCommandRoutingContext {
                ui_has_modal: true,
                source_is_within_active_modal: true,
            },
        ));
        assert!(state.workbench.has_pending_dirty_close(app.models()));

        assert!(handle_workbench_command_with_routing(
            &mut app,
            window,
            &mut state,
            &discard,
            &fret_runtime::CommandDispatchSourceV1 {
                kind: fret_runtime::CommandDispatchSourceKindV1::Pointer,
                element: Some(43),
                test_id: Some(Arc::from("dirty-close.discard")),
            },
            WorkspaceCommandRoutingContext {
                ui_has_modal: true,
                source_is_within_active_modal: true,
            },
        ));
        assert!(!state.workbench.has_pending_dirty_close(app.models()));
        let remaining = app
            .models()
            .read(state.workbench.window_layout(), |layout| {
                layout
                    .pane_tree
                    .find_pane("pane-a")
                    .unwrap()
                    .tabs
                    .tabs()
                    .len()
            })
            .unwrap();
        assert_eq!(remaining, 0);
    }

    #[test]
    fn app_window_close_override_prevents_default_dirty_close_prompt() {
        let mut app = crate::app::App::new();
        let window = crate::WindowId::default();
        let mut layout = fret_workspace::layout::WorkspaceWindowLayout::new("window", "pane-a");
        layout.pane_tree = fret_workspace::layout::WorkspacePaneTree::leaf("pane-a");
        let pane = layout.pane_tree.find_pane_mut("pane-a").unwrap();
        pane.tabs.open_and_activate(Arc::from("doc-a"));
        pane.tabs.set_dirty(Arc::from("doc-a"), true);
        let layout = app.models_mut().insert(layout);
        let workbench = WorkspaceWorkbench::new(app.models_mut(), layout, true);
        let mut state = WorkbenchTestState {
            workbench,
            save_succeeds: false,
            override_window_close: true,
            custom_command_count: 0,
        };

        assert!(handle_workbench_command(
            &mut app,
            window,
            &mut state,
            &CommandId::from("window.close"),
            &fret_runtime::CommandDispatchSourceV1::programmatic(),
        ));
        assert!(!state.workbench.has_pending_dirty_close(app.models()));

        handle_workbench_event(
            &mut app,
            window,
            &mut state,
            &fret_core::Event::WindowCloseRequested,
        );

        assert!(!state.workbench.has_pending_dirty_close(app.models()));
    }
}
