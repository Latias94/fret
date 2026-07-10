#![cfg(all(not(target_arch = "wasm32"), feature = "desktop"))]

use fret::advanced::driver::{FretAppAdvancedExt, UiAppBuilderAdvancedExt, ViewElements, ui_app};
use fret::advanced::{KernelApp, raw::ModelStore};

fn assert_fret_app_advanced_ext<T: FretAppAdvancedExt>() {}

fn assert_ui_app_builder_advanced_ext<T: UiAppBuilderAdvancedExt>() {}

fn assert_root_builder_types(
    builder: fret::advanced::driver::UiAppBuilder<()>,
    driver: fret::advanced::driver::UiAppDriver<()>,
) -> (fret::UiAppBuilder<()>, fret::UiAppDriver<()>) {
    (builder, driver)
}

#[test]
fn advanced_facade_preserves_explicit_public_paths() {
    assert_fret_app_advanced_ext::<fret::FretApp>();
    assert_ui_app_builder_advanced_ext::<fret::UiAppBuilder<()>>();

    let _kernel_app = KernelApp::new();
    let _models = ModelStore::default();
    let _ui_app = ui_app::<()>;
    let _root_type_identity = assert_root_builder_types;
    let _view_elements: Option<ViewElements> = None;
}
