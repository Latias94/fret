const CRATE_USAGE_GUIDE: &str = include_str!("../../../docs/crate-usage-guide.md");
const COMPONENT_AUTHOR_GUIDE: &str = include_str!("../../../docs/component-author-guide.md");
const COMPONENT_AUTHORING_CONTRACTS: &str =
    include_str!("../../../docs/component-authoring-contracts.md");

#[test]
fn curated_docs_prefer_unified_component_conversion_vocabulary() {
    for (label, source) in [
        ("docs/crate-usage-guide.md", CRATE_USAGE_GUIDE),
        ("docs/component-author-guide.md", COMPONENT_AUTHOR_GUIDE),
        (
            "docs/component-authoring-contracts.md",
            COMPONENT_AUTHORING_CONTRACTS,
        ),
    ] {
        assert!(
            source.contains("IntoUiElement<H>"),
            "{label} should teach IntoUiElement<H> on the curated component surface"
        );
    }
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
}
