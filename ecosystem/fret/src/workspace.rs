//! Workspace app-authoring entry points.
//!
//! This module is an explicit `fret/workspace` lane: `fret-workspace` still owns shell concepts,
//! while the `fret` facade owns desktop app startup, defaults, diagnostics, assets, and config
//! layering.

use std::sync::Arc;

use crate::{Defaults, FretApp, Result, UiAppBuilder, UiAppDriver};

pub use fret_workspace::{
    WorkspaceCommandScope, WorkspaceFrame, WorkspacePaneContentFocusTarget, WorkspaceTab,
    WorkspaceTabStrip, workspace_pane_tree_element_with_resize,
};
pub use fret_workspace::{
    commands::{self, register_workspace_commands, typed_command_id},
    layout, menu, panes, tabs,
};

/// Install the default workspace command registry entries.
pub fn install(app: &mut crate::app::App) {
    register_workspace_commands(app.commands_mut());
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
    pub fn ui<S: 'static>(
        self,
        init_window: fn(&mut crate::app::App, crate::WindowId) -> S,
        view: for<'a> fn(&mut crate::AppRenderCx<'a>, &mut S) -> crate::Ui,
    ) -> Result<UiAppBuilder<S>> {
        self.ui_with_hooks(init_window, view, |driver| driver)
    }

    /// Same as [`ui`](Self::ui), with driver hook configuration.
    pub fn ui_with_hooks<S: 'static>(
        self,
        init_window: fn(&mut crate::app::App, crate::WindowId) -> S,
        view: for<'a> fn(&mut crate::AppRenderCx<'a>, &mut S) -> crate::Ui,
        configure: fn(UiAppDriver<S>) -> UiAppDriver<S>,
    ) -> Result<UiAppBuilder<S>> {
        self.into_fret_app()
            .ui_driver_with_hooks(init_window, view, configure)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::integration::InstallIntoApp;
    use fret_app::effective_menu_bar;
    use fret_runtime::{CommandId, InputContext, KeyChord, KeymapService, Platform};

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
}
