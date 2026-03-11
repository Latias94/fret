//! Shared app-integration contracts for reusable ecosystem bundles.
//!
//! Ordinary app code should keep using free installer functions with `FretApp::setup(...)`.
//! Implement [`InstallIntoApp`] when a crate wants to compose multiple installers into one
//! reusable app-facing bundle without widening the default app-author surface. For small app-local
//! composition, tuples are also supported directly.

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

macro_rules! impl_install_into_app_tuple {
    ($($ty:ident => $value:ident),+ $(,)?) => {
        impl<$($ty),+> InstallIntoApp for ($($ty,)+)
        where
            $($ty: InstallIntoApp,)+
        {
            fn install_into_app(self, app: &mut App) {
                let ($($value,)+) = self;
                $(
                    $value.install_into_app(app);
                )+
            }
        }
    };
}

impl_install_into_app_tuple!(A => a, B => b);
impl_install_into_app_tuple!(A => a, B => b, C => c);
impl_install_into_app_tuple!(A => a, B => b, C => c, D => d);

#[cfg(test)]
mod tests {
    use super::InstallIntoApp;
    use crate::app::App;

    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    struct Marker(u32);

    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    struct MarkerB(u32);

    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    struct MarkerC(u32);

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

    #[test]
    fn tuples_compose_small_app_local_installers() {
        let mut app = App::new();
        let install_a = |app: &mut App| app.set_global(Marker(1));
        let install_b = |app: &mut App| app.set_global(MarkerB(2));
        let install_c = |app: &mut App| app.set_global(MarkerC(3));

        (install_a, install_b, install_c).install_into_app(&mut app);

        assert_eq!(app.global::<Marker>().copied(), Some(Marker(1)));
        assert_eq!(app.global::<MarkerB>().copied(), Some(MarkerB(2)));
        assert_eq!(app.global::<MarkerC>().copied(), Some(MarkerC(3)));
    }
}
