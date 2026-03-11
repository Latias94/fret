//! Shared app-integration contracts for reusable ecosystem bundles.
//!
//! Ordinary app code should keep using free installer functions with `FretApp::setup(...)`.
//! Implement [`InstallIntoApp`] when a crate wants to compose multiple installers into one
//! reusable app-facing bundle without widening the default app-author surface.

use crate::app::App;

/// Shared app-level integration seam for reusable ecosystem bundles.
///
/// This trait belongs at the `fret` ecosystem facade layer rather than in `crates/fret-ui`.
/// Keep the default app-author story boring: free installer functions remain valid and are still
/// the first thing docs should teach. Implement this trait when a crate needs to package multiple
/// installers behind one reusable `.setup(...)` value.
pub trait InstallIntoApp {
    /// Install app-owned globals, commands, and other early app wiring.
    fn install_into_app(self, app: &mut App);
}

impl<F> InstallIntoApp for F
where
    F: FnOnce(&mut App),
{
    fn install_into_app(self, app: &mut App) {
        (self)(app);
    }
}

#[cfg(test)]
mod tests {
    use super::InstallIntoApp;
    use crate::app::App;

    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    struct Marker(u32);

    struct BundleInstaller;

    impl InstallIntoApp for BundleInstaller {
        fn install_into_app(self, app: &mut App) {
            app.set_global(Marker(7));
        }
    }

    #[test]
    fn function_installers_implement_install_into_app() {
        let mut app = App::new();
        let install = |app: &mut App| app.set_global(Marker(1));

        install.install_into_app(&mut app);

        assert_eq!(app.global::<Marker>().copied(), Some(Marker(1)));
    }

    #[test]
    fn bundle_types_can_implement_install_into_app() {
        let mut app = App::new();

        BundleInstaller.install_into_app(&mut app);

        assert_eq!(app.global::<Marker>().copied(), Some(Marker(7)));
    }
}
