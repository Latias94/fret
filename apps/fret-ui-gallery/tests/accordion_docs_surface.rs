fn normalize_ws(source: &str) -> String {
    source.split_whitespace().collect()
}

#[test]
fn accordion_page_documents_docs_path_and_children_api_decision() {
    let source = include_str!("../src/ui/pages/accordion.rs");

    for needle in [
        "`accordion_single_uncontrolled(cx, default, |cx| ..)` and `accordion_multiple_uncontrolled(cx, default, |cx| ..)` stay the terse builder helpers for the upstream Demo/Basic/Multiple lanes.",
        "`AccordionRoot::children([...])` plus `AccordionItemPart`, `AccordionTriggerPart`, and `AccordionContentPart` now provide the curated typed children lane on the facade, so the copyable `Usage` section no longer needs the raw `shadcn::raw::accordion::composable` escape hatch.",
        "Radix and Base UI remain the semantics truth for trigger-expanded state, region labelling, and roving navigation; the remaining accordion drift here is first-party docs/public-surface alignment rather than a `fret-ui` mechanism bug.",
        "A broader untyped JSX-style heterogeneous children API is not currently warranted beyond the typed root lane: accordion still only needs explicit item/trigger/content ownership, and widening further would add more naming ambiguity than capability.",
        "Preview mirrors the shadcn Accordion docs path first: Demo, Usage, Basic, Multiple, Disabled, Borders, Card, RTL, and API Reference. The usage lane keeps the typed `AccordionRoot::children([...])` surface copyable on the curated facade, while the builder helpers cover the compact docs examples.",
        "DocSection::build(cx, \"Demo\", demo)",
        "DocSection::build(cx, \"Usage\", usage)",
        "DocSection::build(cx, \"Basic\", basic)",
        "DocSection::build(cx, \"Multiple\", multiple)",
        "DocSection::build(cx, \"Disabled\", disabled)",
        "DocSection::build(cx, \"Borders\", borders)",
        "DocSection::build(cx, \"Card\", card)",
        "DocSection::build(cx, \"RTL\", rtl)",
        "DocSection::build(cx, \"API Reference\", api_reference)",
    ] {
        assert!(
            source.contains(needle),
            "accordion page should document the docs path and children-API decision; missing `{needle}`",
        );
    }

    let normalized = normalize_ws(source);
    let ordered_sections = normalize_ws(
        r#"
        vec![
            demo,
            usage,
            basic,
            multiple,
            disabled,
            borders,
            card,
            rtl,
            api_reference,
        ]
        "#,
    );
    assert!(
        normalized.contains(&ordered_sections),
        "accordion page should keep the docs-path section order before the API reference follow-up",
    );
}

#[test]
fn accordion_usage_snippet_stays_copyable_and_children_oriented() {
    let usage = include_str!("../src/ui/snippets/accordion/usage.rs");

    for needle in [
        "use fret::{UiChild, AppComponentCx};",
        "use fret_ui_shadcn::{facade as shadcn, prelude::*};",
        "shadcn::AccordionRoot::single_uncontrolled(Some(\"item-1\"))",
        ".children([shadcn::AccordionItemPart::new(\"item-1\")",
        "shadcn::AccordionTriggerPart::new(vec![cx.text(\"Is it accessible?\")])",
        "shadcn::AccordionContentPart::new(ui::children![",
        "\"ui-gallery-accordion-usage-trigger\"",
        "\"ui-gallery-accordion-usage-panel\"",
        "\"ui-gallery-accordion-usage\"",
    ] {
        assert!(
            usage.contains(needle),
            "accordion usage snippet should remain a complete copyable typed-children example; missing `{needle}`",
        );
    }

    assert!(
        !usage.contains("shadcn::raw::accordion::composable"),
        "accordion usage snippet should stay on the curated facade instead of the raw composable escape hatch",
    );
    assert!(
        !usage.contains(".item(item)"),
        "accordion usage snippet should prefer the typed `children([...])` root lane over the older single-item staging pattern",
    );
}

#[test]
fn accordion_docs_diag_scripts_cover_docs_path_and_usage_gate() {
    let docs_smoke = include_str!(
        "../../../tools/diag-scripts/ui-gallery/accordion/ui-gallery-accordion-docs-smoke.json"
    );
    let usage_toggle = include_str!(
        "../../../tools/diag-scripts/ui-gallery/accordion/ui-gallery-accordion-usage-toggle.json"
    );
    let conformance_suite =
        include_str!("../../../tools/diag-scripts/suites/ui-gallery-shadcn-conformance/suite.json");
    let runtime_evidence_suite = include_str!(
        "../../../tools/diag-scripts/suites/ui-gallery-shadcn-runtime-evidence/suite.json"
    );

    for needle in [
        "\"ui-gallery-accordion-demo-content\"",
        "\"ui-gallery-accordion-usage-content\"",
        "\"ui-gallery-accordion-basic-content\"",
        "\"ui-gallery-accordion-multiple-content\"",
        "\"ui-gallery-accordion-disabled-content\"",
        "\"ui-gallery-accordion-borders-content\"",
        "\"ui-gallery-accordion-card-content\"",
        "\"ui-gallery-accordion-rtl-content\"",
        "\"ui-gallery-accordion-api-reference-content\"",
    ] {
        assert!(
            docs_smoke.contains(needle),
            "accordion docs smoke script should cover the docs path sections; missing `{needle}`",
        );
    }

    for needle in [
        "\"ui-gallery-accordion-usage-trigger\"",
        "\"ui-gallery-accordion-usage-panel\"",
        "\"ui-gallery-accordion-usage-toggle-open\"",
        "\"ui-gallery-accordion-usage-toggle-closed\"",
    ] {
        assert!(
            usage_toggle.contains(needle),
            "accordion usage toggle script should keep the interaction gate on the typed-children usage lane; missing `{needle}`",
        );
    }

    assert!(
        conformance_suite.contains("tools/diag-scripts/ui-gallery-accordion-docs-smoke.json"),
        "accordion conformance suite should continue to reference the docs-smoke stub",
    );
    assert!(
        runtime_evidence_suite.contains(
            "tools/diag-scripts/ui-gallery/accordion/ui-gallery-accordion-usage-toggle.json"
        ),
        "accordion runtime-evidence suite should continue to reference the usage-toggle interaction artifact",
    );
}
