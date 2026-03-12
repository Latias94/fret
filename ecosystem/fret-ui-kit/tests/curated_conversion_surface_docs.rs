const CRATE_USAGE_GUIDE: &str = include_str!("../../../docs/crate-usage-guide.md");
const COMPONENT_AUTHOR_GUIDE: &str = include_str!("../../../docs/component-author-guide.md");
const COMPONENT_AUTHORING_CONTRACTS: &str =
    include_str!("../../../docs/component-authoring-contracts.md");
const FIRST_HOUR: &str = include_str!("../../../docs/first-hour.md");

#[test]
fn curated_docs_prefer_unified_component_conversion_vocabulary() {
    for (label, source) in [
        ("docs/crate-usage-guide.md", CRATE_USAGE_GUIDE),
        ("docs/component-author-guide.md", COMPONENT_AUTHOR_GUIDE),
        (
            "docs/component-authoring-contracts.md",
            COMPONENT_AUTHORING_CONTRACTS,
        ),
        ("docs/first-hour.md", FIRST_HOUR),
    ] {
        assert!(
            source.contains("IntoUiElement<H>"),
            "{label} should teach IntoUiElement<H> on the curated component surface"
        );
    }

    assert!(
        COMPONENT_AUTHORING_CONTRACTS.contains("ui_component_render_once!"),
        "docs/component-authoring-contracts.md should teach the renamed RenderOnce macro"
    );
}

#[test]
fn curated_docs_avoid_legacy_conversion_trait_names() {
    for (label, source) in [
        ("docs/crate-usage-guide.md", CRATE_USAGE_GUIDE),
        ("docs/component-author-guide.md", COMPONENT_AUTHOR_GUIDE),
        (
            "docs/component-authoring-contracts.md",
            COMPONENT_AUTHORING_CONTRACTS,
        ),
        ("docs/first-hour.md", FIRST_HOUR),
    ] {
        for legacy_name in [
            "UiIntoElement",
            "UiChildIntoElement",
            "UiHostBoundIntoElement",
            "UiBuilderHostBoundIntoElementExt",
        ] {
            assert!(
                !source.contains(legacy_name),
                "{label} reintroduced legacy conversion name `{legacy_name}` into a curated doc"
            );
        }
    }

    assert!(
        !COMPONENT_AUTHORING_CONTRACTS.contains("ui_into_element_render_once!"),
        "docs/component-authoring-contracts.md reintroduced the legacy RenderOnce macro name"
    );
}
