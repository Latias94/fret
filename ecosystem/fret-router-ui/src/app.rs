use fret_app::App;

/// Register the recommended router commands on the app surface.
///
/// Use this from `FretApp::setup(...)` so config/keymap layering can see the router commands
/// before defaults are installed.
pub fn install(app: &mut App) {
    crate::register_router_commands(app.commands_mut());
}
