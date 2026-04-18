fn normalize_ws(source: &str) -> String {
    source.split_whitespace().collect()
}

#[test]
fn select_page_documents_source_axes_and_children_api_decision() {
    let source = include_str!("../src/ui/pages/select.rs");

    for needle in [
        "Reference stack: shadcn Select docs on the Radix and Base UI lanes.",
        "The current visual/chrome baseline comes from the default shadcn registry recipe; semantics/headless references come from Radix Primitives Select and Base UI Select.",
        "`Select::into_element_parts(...)` plus `SelectContent::with_entries(...)` is the typed docs-parity seam for the upstream nested `SelectTrigger` / `SelectValue` / `SelectContent` children lane; a generic root `children([...])` / `compose()` API is not warranted because the option tree is already typed as `SelectEntry` (`SelectGroup` / `SelectItem` / `SelectLabel` / `SelectSeparator`).",
        "Base UI-style object values and multi-select remain separate public-surface work rather than a recipe/mechanism bug.",
        "Preview now mirrors the upstream shadcn/Base UI Select docs path first after collapsing the top `ComponentPreview` into `Demo` and skipping `Installation`: `Demo`, `Usage`, `Align Item With Trigger`, `Groups`, `Scrollable`, `Disabled`, `Invalid`, `RTL`, and `API Reference`.",
        "DocSection::build(cx, \"Composable Parts (Fret)\", parts)",
        "DocSection::build(cx, \"Rich Items (Fret)\", rich_items)",
        "DocSection::build(cx, \"API Reference\", api_reference)",
    ] {
        assert!(
            source.contains(needle),
            "select page should document source axes and the children-api decision; missing `{needle}`",
        );
    }

    let normalized = normalize_ws(source);
    let ordered_sections = normalize_ws(
        r#"
        vec![
            demo,
            usage,
            align_item,
            groups,
            scrollable,
            disabled,
            invalid,
            rtl,
            api_reference,
            parts,
            rich_items,
            label,
            field_association,
            diag_surface,
            notes,
        ]
        "#,
    );
    assert!(
        normalized.contains(&ordered_sections),
        "select page should keep the docs-path sections before the Fret-only follow-ups",
    );
}

#[test]
fn select_snippets_keep_the_default_root_lane_and_explicit_parts_adapter() {
    let usage = include_str!("../src/ui/snippets/select/usage.rs");
    let parts = include_str!("../src/ui/snippets/select/parts.rs");
    let rich_items = include_str!("../src/ui/snippets/select/rich_items.rs");

    for needle in [
        "use fret::{UiChild, AppComponentCx};",
        "use fret_ui_shadcn::{facade as shadcn, prelude::*};",
        "shadcn::Select::new_controllable(cx, None, None::<Arc<str>>, None, false)",
        ".trigger(",
        ".value(shadcn::SelectValue::new().placeholder(\"Select a fruit\"))",
        ".content(shadcn::SelectContent::new())",
        ".entries([shadcn::SelectGroup::new([",
        ".test_id_prefix(\"ui-gallery-select-usage\")",
    ] {
        assert!(
            usage.contains(needle),
            "select usage snippet should remain the copyable default root lane; missing `{needle}`",
        );
    }

    for needle in [
        ".into_element_parts(",
        "shadcn::SelectContent::new().with_entries([shadcn::SelectGroup::new([",
        "shadcn::SelectTrigger::new()",
        "shadcn::SelectValue::new().placeholder(\"Select a fruit\")",
        ".test_id_prefix(\"ui-gallery-select-composable-parts\")",
    ] {
        assert!(
            parts.contains(needle),
            "select parts snippet should keep the typed docs-parity adapter visible; missing `{needle}`",
        );
    }

    assert!(
        !parts.contains(".children(["),
        "select parts snippet should not widen into a generic root children API"
    );

    for needle in [
        "shadcn::SelectItemText::new([",
        "shadcn::SelectTextRun::new(",
        ".label_policy(shadcn::SelectTriggerLabelPolicy::Value)",
        "shadcn::SelectContent::new()",
        ".position(shadcn::raw::select::SelectPosition::Popper)",
    ] {
        assert!(
            rich_items.contains(needle),
            "select rich-items snippet should keep the typed rich-text lane explicit; missing `{needle}`",
        );
    }
}

#[test]
fn select_docs_diag_script_covers_docs_path_then_fret_followups() {
    let script = include_str!(
        "../../../tools/diag-scripts/ui-gallery/select/ui-gallery-select-docs-screenshots.json"
    );

    for needle in [
        "ui-gallery-select-demo-content",
        "ui-gallery-select-usage-content",
        "ui-gallery-select-align-item-content",
        "ui-gallery-select-groups-content",
        "ui-gallery-select-scrollable-content",
        "ui-gallery-select-disabled-content",
        "ui-gallery-select-invalid-content",
        "ui-gallery-select-rtl-content",
        "ui-gallery-select-api-reference-content",
        "ui-gallery-select-composable-parts-content",
        "ui-gallery-select-composable-parts-tabs-trigger-code",
        "ui-gallery-select-rich-items-content",
        "ui-gallery-select-rich-items-tabs-trigger-code",
    ] {
        assert!(
            script.contains(needle),
            "select docs diag script should cover the docs path and Fret follow-ups; missing `{needle}`",
        );
    }
}
