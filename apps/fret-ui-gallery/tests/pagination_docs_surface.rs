#[test]
fn pagination_page_documents_source_axes_and_children_api_decision() {
    let source = include_str!("../src/ui/pages/pagination.rs");

    for needle in [
        "Reference stack: shadcn Pagination docs and the default registry recipe.",
        "No direct `Pagination` primitive exists in Radix Primitives or Base UI",
        "Routing / Next.js",
        "Custom Text / Changelog",
        "explicit composable children",
        "Compact Builder",
    ] {
        assert!(
            source.contains(needle),
            "pagination page should document source axes and the children-api decision; missing `{}`",
            needle,
        );
    }
}

#[test]
fn pagination_page_keeps_docs_order_after_installation() {
    let source = include_str!("../src/ui/pages/pagination.rs");

    for needle in [
        "DocSection::build(cx, \"Demo\", demo)",
        "DocSection::build(cx, \"Usage\", usage)",
        "DocSection::build(cx, \"Simple\", simple)",
        "DocSection::build(cx, \"Icons Only\", icons_only)",
        "DocSection::build(cx, \"Routing / Next.js\", routing)",
        "DocSection::build(cx, \"RTL\", rtl)",
        "DocSection::build(cx, \"Custom Text / Changelog\", custom_text)",
        "DocSection::build(cx, \"Compact Builder\", compact_builder)",
    ] {
        assert!(
            source.contains(needle),
            "pagination page should keep the shadcn docs-aligned section order before Fret follow-ups; missing `{}`",
            needle,
        );
    }
}
