const ADVANCED_RS: &str = include_str!("../../src/advanced.rs");
const APP_PRELUDE_RS: &str = include_str!("../../src/app/prelude.rs");
const ADVANCED_PRELUDE_RS: &str = include_str!("../../src/advanced/prelude.rs");
const ADVANCED_RAW_RS: &str = include_str!("../../src/advanced/raw.rs");
const DOCS_README: &str = include_str!("../../../../docs/README.md");
const ROADMAP: &str = include_str!("../../../../docs/roadmap.md");
const AUTHORING_GOLDEN_PATH: &str = include_str!("../../../../docs/authoring-golden-path.md");
const FEARLESS_REFACTORING: &str = include_str!("../../../../docs/fearless-refactoring.md");
const FIRST_HOUR: &str = include_str!("../../../../docs/first-hour.md");
const TODO_APP_GOLDEN_PATH: &str =
    include_str!("../../../../docs/examples/todo-app-golden-path.md");

fn app_prelude_slice() -> &'static str {
    APP_PRELUDE_RS
}

fn advanced_raw_slice() -> &'static str {
    ADVANCED_RAW_RS
}

fn advanced_prelude_slice() -> &'static str {
    ADVANCED_PRELUDE_RS
}

#[test]
fn raw_state_hook_is_exposed_on_the_advanced_surface() {
    let advanced_slice = ADVANCED_RS;
    let advanced_raw = advanced_raw_slice();
    let advanced_prelude = advanced_prelude_slice();
    let advanced_prelude_without_raw = advanced_prelude;
    assert!(!advanced_slice.contains("AppUiRawStateExt"));
    assert!(!advanced_raw.contains("AppUiRawStateExt"));
    for raw_symbol in [
        "AppUiRawModelExt",
        "AppUiRawActionNotifyExt",
        "LocalStateRawModelExt",
        "LocalStateModelStoreExt",
        "LocalStateElementContextExt",
        "TrackedModelExt",
    ] {
        assert!(
            advanced_raw.contains(raw_symbol),
            "`{raw_symbol}` should live on the explicit advanced raw surface"
        );
        assert!(
            !advanced_prelude_without_raw.contains(raw_symbol),
            "`{raw_symbol}` should stay out of the advanced wildcard prelude"
        );
    }
    assert!(advanced_raw.contains("pub use fret_runtime::{Model, ModelStore, ModelUpdateError};"));
    assert!(advanced_raw.contains("pub use fret_ui::UiTree;"));
    assert!(advanced_raw.contains("pub fn local_state_in<T>("));
    assert!(!advanced_prelude_without_raw.contains("UiTree"));
    assert!(!advanced_prelude_without_raw.contains("ModelStore"));
    assert!(!advanced_prelude_without_raw.contains("TrackedModelExt"));
    assert!(!app_prelude_slice().contains("AppUiRawModelExt"));
    assert!(!app_prelude_slice().contains("AppUiRawActionNotifyExt"));
    assert!(!app_prelude_slice().contains("LocalStateRawModelExt"));
    assert!(!app_prelude_slice().contains("LocalStateModelStoreExt"));
    assert!(!app_prelude_slice().contains("LocalStateElementContextExt"));
    assert!(!app_prelude_slice().contains("TrackedModelExt"));
}

#[test]
fn default_docs_keep_raw_state_as_an_explicit_advanced_seam() {
    assert!(!AUTHORING_GOLDEN_PATH.contains("AppUiRawStateExt"));
    assert!(AUTHORING_GOLDEN_PATH.contains("use fret::advanced::raw::AppUiRawModelExt;"));
    assert!(AUTHORING_GOLDEN_PATH.contains("cx.raw_model::<T>()"));
    assert!(!FEARLESS_REFACTORING.contains("AppUiRawStateExt"));
    assert!(FEARLESS_REFACTORING.contains("use fret::advanced::raw::AppUiRawModelExt;"));
    assert!(!FIRST_HOUR.contains("AppUiRawStateExt"));
    assert!(FIRST_HOUR.contains("use fret::advanced::raw::AppUiRawModelExt;"));
    assert!(FIRST_HOUR.contains("cx.raw_model::<T>()"));
    assert!(!TODO_APP_GOLDEN_PATH.contains("AppUiRawStateExt"));
    let old_raw_import = format!("use fret::advanced::{};", "AppUiRawModelExt");
    assert!(!TODO_APP_GOLDEN_PATH.contains(&old_raw_import));
    assert!(
        TODO_APP_GOLDEN_PATH
            .contains("move that code to an explicit advanced\nintegration document")
    );
}

#[test]
fn default_docs_prefer_render_context_access_for_new_helper_signatures() {
    assert!(AUTHORING_GOLDEN_PATH.contains("fret::app::AppRenderContext<'a>"));
    assert!(AUTHORING_GOLDEN_PATH.contains("fret::app::AppRenderCx<'_>"));
    assert!(FIRST_HOUR.contains("`fret::app::AppRenderContext<'a>`"));
    assert!(FIRST_HOUR.contains("`&mut fret::app::AppRenderCx<'_>`"));
    assert!(TODO_APP_GOLDEN_PATH.contains("`fret::app::AppRenderContext<'a>`"));
    assert!(TODO_APP_GOLDEN_PATH.contains("`&mut fret::app::AppRenderCx<'_>`"));
    assert!(!FIRST_HOUR.contains("give it `cx: &mut UiCx<'_>`"));
    assert!(!TODO_APP_GOLDEN_PATH.contains("give a helper `&mut UiCx<'_>`"));
}

#[test]
fn docs_indices_use_the_current_raw_model_name() {
    assert!(DOCS_README.contains("`AppUiRawModelExt::raw_model::<T>()`"));
    assert!(!DOCS_README.contains("keep\n    `use_state` as the explicit raw-model seam"));
    assert!(ROADMAP.contains("`AppUiRawModelExt::raw_model::<T>()`"));
    assert!(!ROADMAP.contains("`use_state` as the advanced raw-model seam"));
}
