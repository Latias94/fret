fn normalize_ws(source: &str) -> String {
    source.split_whitespace().collect()
}

#[test]
fn popover_page_documents_source_axes_and_children_api_decision() {
    let source = include_str!("../src/ui/pages/popover.rs");

    for needle in [
        "Reference stack: shadcn/base Popover docs and demo, the default registry recipe, Radix Primitives popover semantics, and Base UI popover ownership.",
        "`Popover::new(cx, trigger, content)` remains the default recipe-level entry point and already covers the upstream nested `<Popover><PopoverTrigger /><PopoverContent /></Popover>` composition plus the custom-trigger / `asChild` story, because `trigger` can be any landed or late-landed element.",
        "`PopoverTrigger::build(...)`, `PopoverContent::build(cx, ...)`, and `PopoverContent::new([...])` cover the copyable compound-parts lane plus the landed-children follow-up without adding a separate heterogeneous root `children([...])` / `compose()` API.",
        "`Popover::trigger_element(...)` and `Popover::anchor_element(...)` cover the detached-trigger, anchor-aware follow-up without widening the default shadcn docs lane.",
        "`Popover::open_on_hover(...)`, `hover_open_delay_frames(...)`, and `hover_close_delay_frames(...)` cover the Base UI hover-open follow-up while staying outside the default docs path.",
        "A generic heterogeneous `children([...])` root API is not currently warranted here: unlike Dialog/Drawer, Popover root only needs trigger/content, while managed-open and anchor-aware cases already stay explicit on `from_open(...).into_element_with(...)` / `into_element_with_anchor(...)`.",
        "Preview mirrors the shadcn/base Popover docs path after `Installation`: `Demo`, `Usage`, `Basic`, `Align`, `With Form`, `RTL`, and `API Reference`. `Detached Trigger (Fret)`, `Open on Hover (Base UI/Fret)`, and `Inline Children (Fret)` stay as explicit follow-ups.",
        "Popover dismissal, focus restore, outside-press routing, placement, and inline-child sizing are already covered by the existing Radix/web contract tests and UI Gallery overlay gates; the remaining work here is docs/public-surface alignment rather than a `fret-ui` mechanism bug.",
        ".test_id_prefix(\"ui-gallery-popover-demo\")",
        ".test_id_prefix(\"ui-gallery-popover-usage\")",
        ".test_id_prefix(\"ui-gallery-popover-basic\")",
        ".test_id_prefix(\"ui-gallery-popover-align\")",
        ".test_id_prefix(\"ui-gallery-popover-with-form\")",
        ".test_id_prefix(\"ui-gallery-popover-rtl\")",
        ".test_id_prefix(\"ui-gallery-popover-detached-trigger\")",
        ".test_id_prefix(\"ui-gallery-popover-open-on-hover\")",
        ".test_id_prefix(\"ui-gallery-popover-inline-children\")",
    ] {
        assert!(
            source.contains(needle),
            "popover page should document the source axes and children-api decision; missing `{needle}`"
        );
    }

    let normalized = normalize_ws(source);
    let ordered_sections = normalize_ws(
        r#"
        vec![
            demo,
            usage,
            basic,
            align,
            with_form,
            rtl,
            api_reference,
            detached_trigger,
            open_on_hover,
            inline_children,
            notes,
        ]
        "#,
    );
    assert!(
        normalized.contains(&ordered_sections),
        "popover page should keep the docs-path sections before the Fret follow-ups and notes"
    );
}

#[test]
fn popover_docs_path_snippets_stay_copyable_and_docs_aligned() {
    let usage = include_str!("../src/ui/snippets/popover/usage.rs");
    let demo = include_str!("../src/ui/snippets/popover/demo.rs");
    let rtl = include_str!("../src/ui/snippets/popover/rtl.rs");

    for needle in [
        "use fret::{UiChild, AppComponentCx};",
        "use fret_ui_shadcn::{facade as shadcn, prelude::*};",
        "shadcn::Popover::new(",
        "shadcn::PopoverTrigger::build(",
        "shadcn::PopoverContent::build(cx, |cx| {",
        "shadcn::Button::new(\"Open Popover\").variant(shadcn::ButtonVariant::Outline)",
    ] {
        assert!(
            usage.contains(needle),
            "popover usage snippet should remain a complete copyable docs-path example; missing `{needle}`"
        );
    }

    for needle in [
        ".refine_layout(LayoutRefinement::default().w_px(Px(320.0)))",
        ".test_id(\"ui-gallery-popover-demo-panel\")",
        ".test_id(\"ui-gallery-popover-demo-trigger\")",
        ".test_id(\"ui-gallery-popover-demo-popover\")",
        ".test_id(\"ui-gallery-popover-demo\")",
    ] {
        assert!(
            demo.contains(needle),
            "popover demo snippet should keep the upstream-shaped preview surface and stable selectors; missing `{needle}`"
        );
    }

    for needle in [
        "shadcn::PopoverSide::InlineStart",
        "shadcn::PopoverSide::InlineEnd",
        "\"بداية السطر\"",
        "\"نهاية السطر\"",
        "\"ui-gallery-popover-rtl-inline-start-trigger\"",
        "\"ui-gallery-popover-rtl-inline-end-trigger\"",
    ] {
        assert!(
            rtl.contains(needle),
            "popover rtl snippet should keep logical sides and readable RTL labels; missing `{needle}`"
        );
    }

    let combined = [usage, demo, rtl].join("\n");
    assert!(
        !combined.contains("compose()"),
        "popover docs-path snippets should keep the default teaching lane on `Popover::new(...)` instead of inventing a compose() surface",
    );
}

#[test]
fn popover_follow_up_snippets_stay_copyable_and_explicit() {
    let detached = include_str!("../src/ui/snippets/popover/detached_trigger.rs");
    let open_on_hover = include_str!("../src/ui/snippets/popover/open_on_hover.rs");
    let inline_children = include_str!("../src/ui/snippets/popover/inline_children.rs");

    for needle in [
        "use fret::{UiChild, AppComponentCx};",
        ".auto_toggle(false)",
        ".toggle_model(open.clone())",
        ".trigger_element(detached_id)",
        ".anchor_element(detached_id)",
        ".into_element_with(",
        "\"ui-gallery-popover-detached-trigger-button\"",
        "\"ui-gallery-popover-detached-trigger-panel\"",
    ] {
        assert!(
            detached.contains(needle),
            "detached-trigger snippet should remain a complete copyable example; missing `{needle}`"
        );
    }

    for needle in [
        "use fret::{UiChild, AppComponentCx};",
        ".open_on_hover(true)",
        ".hover_open_delay_frames(12)",
        ".hover_close_delay_frames(6)",
        "\"ui-gallery-popover-open-on-hover-trigger\"",
        "\"ui-gallery-popover-open-on-hover-panel\"",
    ] {
        assert!(
            open_on_hover.contains(needle),
            "open-on-hover snippet should remain a complete copyable example; missing `{needle}`"
        );
    }

    for needle in [
        "Inline-sized children should keep their intrinsic width by default.",
        ".test_id(\"ui-gallery-popover-inline-children-button\")",
        ".test_id(\"ui-gallery-popover-inline-children-panel\")",
        ".test_id(\"ui-gallery-popover-inline-children-trigger\")",
        ".align(shadcn::PopoverAlign::Start)",
    ] {
        assert!(
            inline_children.contains(needle),
            "inline-children snippet should remain the focused intrinsic-width follow-up; missing `{needle}`"
        );
    }

    let combined = [detached, open_on_hover, inline_children].join("\n");
    assert!(
        !combined.contains("compose()"),
        "popover follow-up snippets should stay on the explicit advanced seams instead of inventing a compose() surface",
    );
    assert!(
        !combined.contains("Popover::children("),
        "popover follow-up snippets should not widen the root surface into a generic children API",
    );
}

#[test]
fn popover_docs_diag_scripts_cover_docs_path_and_existing_regression_gates() {
    let docs_script = include_str!(
        "../../../tools/diag-scripts/ui-gallery/overlay/ui-gallery-popover-doc-page-opens.json"
    );
    let docs_stub = include_str!("../../../tools/diag-scripts/ui-gallery-popover-docs-smoke.json");
    let docs_suite =
        include_str!("../../../tools/diag-scripts/suites/ui-gallery-shadcn-conformance/suite.json");
    let inline_children_script = include_str!(
        "../../../tools/diag-scripts/ui-gallery/popover/ui-gallery-popover-inline-children-button-not-stretched.json"
    );

    for needle in [
        "\"ui-gallery-popover-demo-content\"",
        "\"ui-gallery-popover-usage-content\"",
        "\"ui-gallery-popover-basic-content\"",
        "\"ui-gallery-popover-align-content\"",
        "\"ui-gallery-popover-with-form-content\"",
        "\"ui-gallery-popover-rtl-content\"",
        "\"ui-gallery-popover-api-reference-title\"",
        "\"ui-gallery-popover-api-reference-content\"",
        "\"ui-gallery-popover-detached-trigger-content\"",
        "\"ui-gallery-popover-open-on-hover-content\"",
        "\"ui-gallery-popover-inline-children-content\"",
        "\"ui-gallery-popover-notes-content\"",
        "\"ui-gallery-popover-docs-smoke\"",
    ] {
        assert!(
            docs_script.contains(needle),
            "popover docs diag script should cover the docs path and the Fret follow-ups; missing `{needle}`"
        );
    }

    for needle in [
        "\"ui-gallery-popover-inline-children-trigger\"",
        "\"ui-gallery-popover-inline-children-panel\"",
        "\"ui-gallery-popover-inline-children-button\"",
        "\"ui-gallery-popover-inline-children-button-not-stretched\"",
    ] {
        assert!(
            inline_children_script.contains(needle),
            "popover inline-children diag script should keep the existing intrinsic-width regression gate; missing `{needle}`"
        );
    }

    assert!(
        docs_stub.contains(
            "\"to\": \"tools/diag-scripts/ui-gallery/overlay/ui-gallery-popover-doc-page-opens.json\""
        ),
        "popover docs smoke top-level redirect stub should point at the canonical overlay docs script",
    );
    assert!(
        docs_suite.contains("\"tools/diag-scripts/ui-gallery-popover-docs-smoke.json\""),
        "popover docs smoke script should be promoted into the ui-gallery shadcn conformance suite",
    );
}

#[test]
fn popover_demo_narrow_diag_script_waits_for_stable_overlay_bounds() {
    let script = include_str!(
        "../../../tools/diag-scripts/ui-gallery/overlay/ui-gallery-popover-demo-narrow-open-screenshot.json"
    );

    for needle in [
        "\"ui-gallery-popover-demo-panel\"",
        "\"type\": \"wait_bounds_stable\"",
        "\"stable_frames\": 6",
        "\"max_move_px\": 1.0",
        "\"ui-gallery-popover-demo-open-narrow\"",
    ] {
        assert!(
            script.contains(needle),
            "popover demo narrow diag script should wait for stable overlay bounds before screenshots; missing `{needle}`",
        );
    }
}
