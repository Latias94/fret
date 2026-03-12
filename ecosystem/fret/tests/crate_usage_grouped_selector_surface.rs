const CRATE_USAGE_GUIDE: &str = include_str!("../../../docs/crate-usage-guide.md");

#[test]
fn crate_usage_guide_keeps_selector_guidance_on_grouped_app_surfaces() {
    assert!(CRATE_USAGE_GUIDE.contains("`cx.data().selector(...)`"));
    assert!(CRATE_USAGE_GUIDE.contains("`fret::app::prelude::*` also re-exports `DepsBuilder` /"));
    assert!(
        CRATE_USAGE_GUIDE
            .contains("Enable `fret-selector/ui` only when you are working directly with")
    );
    assert!(CRATE_USAGE_GUIDE.contains("`ElementContext` in component/advanced surfaces."));
}
