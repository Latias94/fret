use fret_core::UiServices;

/// Install `fret-node` at the UI services boundary.
///
/// This registers node-graph command metadata (for menus / command palette surfaces) and installs
/// any command-provided default keybindings into the app keymap.
pub fn install_with_ui_services(app: &mut fret_app::App, _services: &mut dyn UiServices) {
    let _ = _services;
    crate::app::install(app);
}
