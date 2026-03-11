use fret_core::UiServices;

/// Install `fret-node` into a `fret-app` application.
///
/// This registers node-graph command metadata (for menus / command palette surfaces) and installs
/// any command-provided default keybindings into the app keymap.
pub fn install(app: &mut fret_app::App) {
    #[cfg(feature = "fret-ui")]
    crate::ui::register_node_graph_commands(app.commands_mut());

    // Ensure `CommandMeta.default_keybindings` are reflected in the app keymap.
    fret_app::install_command_default_keybindings_into_keymap(app);
}

/// Install `fret-node` at the UI services boundary.
///
/// This registers node-graph command metadata (for menus / command palette surfaces) and installs
/// any command-provided default keybindings into the app keymap.
pub fn install_with_ui_services(app: &mut fret_app::App, _services: &mut dyn UiServices) {
    let _ = _services;
    install(app);
}
