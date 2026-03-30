fn normalize_ws(source: &str) -> String {
    source.split_whitespace().collect()
}

#[test]
fn tooltip_page_documents_source_axes_and_children_api_decision() {
    let source = include_str!("../src/ui/pages/tooltip.rs");

    for needle in [
        "Reference stack: shadcn Tooltip docs, the default registry chrome, Radix Primitives tooltip semantics/lifecycle, and Base UI tooltip lifecycle.",
        "`Tooltip::new(cx, trigger, content)` already acts as the default copyable root lane",
        "`TooltipTrigger::build(...)` and `TooltipContent::build(cx, ...)` cover the typed compound-parts lane for copyable first-party snippets, while `TooltipContent::new([...])` remains the landed-content follow-up when you already own the children.",
        "Tooltip hover/focus, Escape/outside-press dismissal, scroll-close, and Radix web parity are already covered by the existing tooltip tests in `ecosystem/fret-ui-shadcn`; the remaining work here is docs/public-surface alignment rather than a `fret-ui` mechanism bug.",
        "No extra generic `children([...])` / `compose()` / `asChild` root API is currently warranted",
        "Preview mirrors the shadcn/base Tooltip docs path after collapsing the top `ComponentPreview` into `Demo` and skipping `Installation`: `Demo`, `Usage`, `Side`, `With Keyboard Shortcut`, `Disabled Button`, `RTL`, and `API Reference`. `Long Content`, `Keyboard Focus`, and `Notes` stay as explicit Fret follow-ups.",
        ".test_id_prefix(\"ui-gallery-tooltip-side\")",
        ".test_id_prefix(\"ui-gallery-tooltip-keyboard-shortcut\")",
        ".test_id_prefix(\"ui-gallery-tooltip-disabled-button\")",
        ".test_id_prefix(\"ui-gallery-tooltip-keyboard-focus\")",
    ] {
        assert!(
            source.contains(needle),
            "tooltip page should document the source axes and children-api decision; missing `{needle}`",
        );
    }

    let normalized = normalize_ws(source);
    let ordered_sections = normalize_ws(
        r#"
        vec![
            demo_tooltip,
            usage,
            side_row,
            keyboard_tooltip,
            disabled_tooltip,
            rtl_row,
            api_reference,
            long_content_tooltip,
            focus_row,
            notes,
        ]
        "#,
    );
    assert!(
        normalized.contains(&ordered_sections),
        "tooltip page should keep the docs-path sections before the Fret follow-ups and notes",
    );
}

#[test]
fn tooltip_snippets_stay_copyable_and_docs_aligned() {
    let demo = include_str!("../src/ui/snippets/tooltip/demo.rs");
    let usage = include_str!("../src/ui/snippets/tooltip/usage.rs");
    let long_content = include_str!("../src/ui/snippets/tooltip/long_content.rs");
    let keyboard_focus = include_str!("../src/ui/snippets/tooltip/keyboard_focus.rs");

    for needle in [
        "use fret::{UiChild, UiCx};",
        "use fret_ui_shadcn::facade as shadcn;",
        "shadcn::TooltipProvider::new()",
        "shadcn::TooltipTrigger::build(",
        "shadcn::TooltipContent::build(cx, |_cx| {",
        "shadcn::Tooltip::new(",
        "shadcn::Button::new(\"Hover\").variant(shadcn::ButtonVariant::Outline)",
    ] {
        assert!(
            usage.contains(needle),
            "tooltip usage snippet should remain a complete copyable docs-path example; missing `{needle}`",
        );
    }

    for needle in [
        ".delay(Duration::ZERO)",
        ".timeout_duration(Duration::from_millis(400))",
        ".arrow(true)",
        ".panel_test_id(\"ui-gallery-tooltip-demo-panel\")",
        ".test_id(\"ui-gallery-tooltip-demo\")",
    ] {
        assert!(
            demo.contains(needle),
            "tooltip demo snippet should keep the docs-aligned preview surface and stable test ids; missing `{needle}`",
        );
    }

    for needle in [
        "\"This tooltip demonstrates long content wrapping at the max width boundary without collapsing to min-content.\"",
        ".test_id(\"ui-gallery-tooltip-long-content-text\")",
        ".panel_test_id(\"ui-gallery-tooltip-long-content-panel\")",
    ] {
        assert!(
            long_content.contains(needle),
            "tooltip long-content snippet should keep the explicit max-width regression surface; missing `{needle}`",
        );
    }

    for needle in [
        "\"Opens on keyboard focus\"",
        ".test_id(\"ui-gallery-tooltip-focus-trigger\")",
        ".panel_test_id(\"ui-gallery-tooltip-focus-panel\")",
        ".test_id(\"ui-gallery-tooltip-focus-row\")",
    ] {
        assert!(
            keyboard_focus.contains(needle),
            "tooltip keyboard-focus snippet should remain a copyable focus-parity follow-up; missing `{needle}`",
        );
    }

    let combined = [demo, usage, long_content, keyboard_focus].join("\n");
    assert!(
        !combined.contains("compose()"),
        "tooltip snippets should keep the default teaching lane on `Tooltip::new(...)` instead of inventing a compose() surface",
    );
}

#[test]
fn tooltip_docs_diag_scripts_cover_docs_path_and_follow_ups() {
    let docs_script = include_str!(
        "../../../tools/diag-scripts/ui-gallery/overlay/ui-gallery-tooltip-docs-smoke.json"
    );
    let long_content_script = include_str!(
        "../../../tools/diag-scripts/ui-gallery/tooltip/ui-gallery-tooltip-long-content-screenshot-zinc-dark.json"
    );
    let docs_stub = include_str!("../../../tools/diag-scripts/ui-gallery-tooltip-docs-smoke.json");
    let docs_suite =
        include_str!("../../../tools/diag-scripts/suites/ui-gallery-shadcn-conformance/suite.json");

    for needle in [
        "\"ui-gallery-tooltip-demo-content\"",
        "\"ui-gallery-tooltip-usage-content\"",
        "\"ui-gallery-tooltip-side-content\"",
        "\"ui-gallery-tooltip-keyboard-shortcut-content\"",
        "\"ui-gallery-tooltip-disabled-button-content\"",
        "\"ui-gallery-tooltip-rtl-content\"",
        "\"ui-gallery-tooltip-api-reference-title\"",
        "\"ui-gallery-tooltip-api-reference-content\"",
        "\"ui-gallery-tooltip-long-content-content\"",
        "\"ui-gallery-tooltip-keyboard-focus-content\"",
        "\"ui-gallery-tooltip-notes-content\"",
        "\"ui-gallery-tooltip-docs-smoke\"",
    ] {
        assert!(
            docs_script.contains(needle),
            "tooltip docs diag script should cover the docs path and the Fret follow-ups; missing `{needle}`",
        );
    }

    for needle in [
        "\"ui-gallery-tooltip-long-content-trigger\"",
        "\"ui-gallery-tooltip-long-content-panel\"",
        "\"ui-gallery-tooltip-long-content-text\"",
        "\"ui-gallery-tooltip-long-content-zinc-dark\"",
    ] {
        assert!(
            long_content_script.contains(needle),
            "tooltip long-content diag script should keep the existing visual regression coverage; missing `{needle}`",
        );
    }

    assert!(
        docs_stub.contains(
            "\"to\": \"tools/diag-scripts/ui-gallery/overlay/ui-gallery-tooltip-docs-smoke.json\""
        ),
        "tooltip docs smoke top-level redirect stub should point at the canonical overlay script",
    );
    assert!(
        docs_suite.contains("\"tools/diag-scripts/ui-gallery-tooltip-docs-smoke.json\""),
        "tooltip docs smoke script should be promoted into the ui-gallery shadcn conformance suite",
    );
}
