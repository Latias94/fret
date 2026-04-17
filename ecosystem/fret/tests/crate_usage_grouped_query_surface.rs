const CRATE_USAGE_GUIDE: &str = include_str!("../../../docs/crate-usage-guide.md");

#[test]
fn crate_usage_guide_keeps_query_guidance_on_grouped_app_surfaces() {
    assert!(CRATE_USAGE_GUIDE.contains("`cx.data().query_async(...)`"));
    assert!(CRATE_USAGE_GUIDE.contains("`cx.data().query_async_local(...)`"));
    assert!(CRATE_USAGE_GUIDE.contains("`handle.read_layout(cx)`"));
    assert!(CRATE_USAGE_GUIDE.contains("`cx.data().invalidate_query(...)`"));
    assert!(CRATE_USAGE_GUIDE.contains("`cx.data().invalidate_query_namespace(...)`"));
    assert!(CRATE_USAGE_GUIDE.contains("`cx.data().cancel_query(...)`"));
    assert!(CRATE_USAGE_GUIDE.contains("`cx.data().query_snapshot_entry(...)`"));
    assert!(CRATE_USAGE_GUIDE.contains("`fret::query::{QueryKey, QueryPolicy, QueryState, ...}`"));
    assert!(
        CRATE_USAGE_GUIDE
            .contains("Extracted helpers on the default lane should usually be generic over")
    );
    assert!(CRATE_USAGE_GUIDE.contains("`fret::app::AppRenderContext<'a>`"));
    assert!(CRATE_USAGE_GUIDE.contains("`&mut fret::app::AppRenderCx<'_>`"));
    assert!(CRATE_USAGE_GUIDE.contains("`UiCxActionsExt`"));
    assert!(CRATE_USAGE_GUIDE.contains("`UiCxDataExt`"));
    assert!(CRATE_USAGE_GUIDE.contains("compatibility old-name alias"));
    assert!(CRATE_USAGE_GUIDE.contains("`fret::env::{...}`"));
    assert!(CRATE_USAGE_GUIDE.contains("`ContainerQueryHysteresis`"));
    assert!(CRATE_USAGE_GUIDE.contains("`ViewportQueryHysteresis`"));
    assert!(CRATE_USAGE_GUIDE.contains("`ViewportOrientation`"));
    assert!(
        CRATE_USAGE_GUIDE
            .contains("Keep `fret::query::with_query_client(...)` for pure app/driver code")
    );
    assert!(CRATE_USAGE_GUIDE.contains("working directly with low-level"));
    assert!(CRATE_USAGE_GUIDE.contains("generic writer extensions outside the"));
    assert!(CRATE_USAGE_GUIDE.contains("app-facing `fret` facades."));
    assert!(!CRATE_USAGE_GUIDE.contains("`ElementContext` helpers like `cx.use_query_async(...)`"));
    assert!(!CRATE_USAGE_GUIDE.contains("`cx.use_query_async(...)`"));
}
