const SHADCN_DECLARATIVE_PROGRESS: &str =
    include_str!("../../../docs/shadcn-declarative-progress.md");
const UI_BUILDER_EXT_BREADCRUMB_RS: &str =
    include_str!("../../fret-ui-shadcn/src/ui_builder_ext/breadcrumb.rs");
const UI_BUILDER_EXT_COLLAPSIBLE_RS: &str =
    include_str!("../../fret-ui-shadcn/src/ui_builder_ext/collapsible.rs");
const UI_BUILDER_EXT_COMMAND_DIALOG_RS: &str =
    include_str!("../../fret-ui-shadcn/src/ui_builder_ext/command_dialog.rs");
const UI_BUILDER_EXT_DATA_RS: &str =
    include_str!("../../fret-ui-shadcn/src/ui_builder_ext/data.rs");
const UI_BUILDER_EXT_MENUS_RS: &str =
    include_str!("../../fret-ui-shadcn/src/ui_builder_ext/menus.rs");
const UI_BUILDER_EXT_OVERLAY_ROOTS_RS: &str =
    include_str!("../../fret-ui-shadcn/src/ui_builder_ext/overlay_roots.rs");
const UI_EXT_SUPPORT_RS: &str = include_str!("../../fret-ui-shadcn/src/ui_ext/support.rs");
const UI_EXT_DATA_RS: &str = include_str!("../../fret-ui-shadcn/src/ui_ext/data.rs");

#[test]
fn shadcn_docs_keep_reusable_helper_guidance_on_unified_component_conversion_trait() {
    assert!(SHADCN_DECLARATIVE_PROGRESS.contains(
        "reusable generic helpers in `fret-ui-shadcn` / `fret-ui-kit` should converge on the unified"
    ));
    assert!(SHADCN_DECLARATIVE_PROGRESS.contains("`IntoUiElement<H>`"));
    assert!(SHADCN_DECLARATIVE_PROGRESS.contains("do not have to pre-land into `AnyElement`"));
}

#[test]
fn shadcn_ui_ext_glue_stays_on_unified_component_conversion_trait() {
    for (label, source) in [
        ("ui_ext/support.rs", UI_EXT_SUPPORT_RS),
        ("ui_ext/data.rs", UI_EXT_DATA_RS),
    ] {
        assert!(
            source.contains("IntoUiElement<H>"),
            "{label} should stay on IntoUiElement<H>"
        );
        assert!(
            !source.contains("::fret_ui_kit::UiIntoElement"),
            "{label} reintroduced direct UiIntoElement glue"
        );
    }
}

#[test]
fn shadcn_ui_builder_ext_prefers_unified_helper_inputs_over_prelanded_anyelement() {
    for (label, source, forbidden) in [
        (
            "ui_builder_ext/breadcrumb.rs",
            UI_BUILDER_EXT_BREADCRUMB_RS,
            "IntoIterator<Item = AnyElement>",
        ),
        (
            "ui_builder_ext/collapsible.rs",
            UI_BUILDER_EXT_COLLAPSIBLE_RS,
            "FnOnce(&mut ElementContext<'_, H>, bool) -> AnyElement",
        ),
        (
            "ui_builder_ext/command_dialog.rs",
            UI_BUILDER_EXT_COMMAND_DIALOG_RS,
            "FnOnce(&mut ElementContext<'_, H>) -> AnyElement",
        ),
        (
            "ui_builder_ext/data.rs",
            UI_BUILDER_EXT_DATA_RS,
            "FnMut(&mut ElementContext<'_, H>, usize, usize) -> AnyElement",
        ),
        (
            "ui_builder_ext/menus.rs",
            UI_BUILDER_EXT_MENUS_RS,
            "FnOnce(&mut ElementContext<'_, H>) -> AnyElement",
        ),
        (
            "ui_builder_ext/overlay_roots.rs",
            UI_BUILDER_EXT_OVERLAY_ROOTS_RS,
            "FnOnce(&mut ElementContext<'_, H>) -> AnyElement",
        ),
    ] {
        assert!(
            source.contains("IntoUiElement<H>"),
            "{label} should accept IntoUiElement<H>"
        );
        assert!(
            !source.contains(forbidden),
            "{label} reintroduced a pre-landed AnyElement helper input"
        );
    }
}
