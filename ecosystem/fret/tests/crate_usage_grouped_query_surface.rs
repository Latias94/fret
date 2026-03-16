const CRATE_USAGE_GUIDE: &str = include_str!("../../../docs/crate-usage-guide.md");

#[test]
fn crate_usage_guide_keeps_query_guidance_on_grouped_app_surfaces() {
    assert!(CRATE_USAGE_GUIDE.contains("`cx.data().query_async(...)`"));
    assert!(CRATE_USAGE_GUIDE.contains("`cx.data().query_async_local(...)`"));
    assert!(CRATE_USAGE_GUIDE.contains("`fret::query::{QueryKey, QueryPolicy, QueryState, ...}`"));
    assert!(CRATE_USAGE_GUIDE.contains("Extracted `UiCx` helpers keep that same grouped"));
    assert!(CRATE_USAGE_GUIDE.contains("`UiCxActionsExt`"));
    assert!(CRATE_USAGE_GUIDE.contains("`UiCxDataExt`"));
    assert!(CRATE_USAGE_GUIDE.contains("working directly with low-level"));
    assert!(CRATE_USAGE_GUIDE.contains("generic writer extensions outside the"));
    assert!(CRATE_USAGE_GUIDE.contains("app-facing `fret` facades."));
    assert!(!CRATE_USAGE_GUIDE.contains("`ElementContext` helpers like `cx.use_query_async(...)`"));
    assert!(!CRATE_USAGE_GUIDE.contains("`cx.use_query_async(...)`"));
}
