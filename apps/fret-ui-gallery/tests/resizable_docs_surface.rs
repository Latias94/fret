fn normalize_ws(source: &str) -> String {
    source.split_whitespace().collect()
}

#[test]
fn resizable_page_documents_source_axes_and_children_api_decision() {
    let source = include_str!("../src/ui/pages/resizable.rs");

    for needle in [
        "Reference stack: shadcn Resizable docs on the Base UI and Radix lanes.",
        "The current visual/chrome baseline comes from the default shadcn registry recipe, with parallel headless baselines in the base and radix registry variants.",
        "Unlike `slider` or `progress`, there is no direct `Resizable` primitive in Radix Primitives or Base UI; those libraries still inform general headless/mechanism decisions, but the concrete source axis here is shadcn plus the runtime panel-group contract.",
        "`resizable_panel_group(cx, model, |cx| ..)` is already the composable children-equivalent lane for Fret",
        "A generic composable children / `compose()` API is not warranted here",
        "The Fret-only follow-up below intentionally keeps one fixed-window splitter proof on the first-party docs surface so `panel width` remains visibly distinct from `viewport width` in review and diagnostics.",
        "Preview mirrors the shadcn/Base UI Resizable docs path after collapsing the top `ComponentPreview` into `Demo` and skipping `Installation`: `Demo`, `About`, `Usage`, `Vertical`, `Handle`, `RTL`, and `API Reference`. `Adaptive Panel Proof` is the explicit Fret follow-up",
        "DocSection::build(cx, \"About\", about)",
        "DocSection::build(cx, \"API Reference\", api_reference)",
        "DocSection::build(cx, \"Adaptive Panel Proof\", adaptive_panel)",
    ] {
        assert!(
            source.contains(needle),
            "resizable page should document source axes and the children-api decision; missing `{needle}`",
        );
    }

    let normalized = normalize_ws(source);
    let ordered_sections = normalize_ws(
        r#"
        vec![demo, about, usage, vertical, handle, rtl, api_reference, adaptive_panel, notes,]
        "#,
    );
    assert!(
        normalized.contains(&ordered_sections),
        "resizable page should keep the docs-path sections before the fixed-window proof and the final `Notes` follow-up",
    );
}

#[test]
fn resizable_snippets_stay_copyable_and_docs_aligned() {
    let usage = include_str!("../src/ui/snippets/resizable/usage.rs");
    let demo = include_str!("../src/ui/snippets/resizable/demo.rs");
    let handle = include_str!("../src/ui/snippets/resizable/handle.rs");
    let rtl = include_str!("../src/ui/snippets/resizable/rtl.rs");
    let adaptive_panel = include_str!("../src/ui/snippets/resizable/adaptive_panel.rs");
    let notes = include_str!("../src/ui/snippets/resizable/notes.rs");

    for needle in [
        "use fret::{AppComponentCx, UiChild};",
        "use fret_ui_shadcn::{facade as shadcn, prelude::*};",
        "let fractions = cx.local_model_keyed(\"ui-gallery-resizable-usage-fractions\", || vec![0.5, 0.5]);",
        "shadcn::resizable_panel_group(cx, fractions, |cx| {",
        "shadcn::ResizablePanel::new([panel(cx, \"One\").into_element(cx)]).into()",
        "shadcn::ResizableHandle::new().into()",
        ".axis(Axis::Horizontal)",
        ".test_id_prefix(\"ui-gallery-resizable-usage\")",
    ] {
        assert!(
            usage.contains(needle),
            "resizable usage snippet should remain a complete copyable minimal example; missing `{needle}`",
        );
    }

    for needle in [
        "let nested_vertical = shadcn::resizable_panel_group(cx, v_fractions, |cx| {",
        "Some(Px(200.0))",
        ".axis(Axis::Vertical)",
        ".test_id_prefix(\"ui-gallery-resizable-demo.nested-vertical\")",
        ".test_id(\"ui-gallery-resizable-panels\")",
    ] {
        assert!(
            demo.contains(needle),
            "resizable demo snippet should keep the nested upstream-style layout demo; missing `{needle}`",
        );
    }

    for needle in [
        "shadcn::ResizableHandle::new().with_handle(true).into()",
        ".test_id_prefix(\"ui-gallery-resizable-handle\")",
        ".max_w(Px(448.0))",
    ] {
        assert!(
            handle.contains(needle),
            "resizable handle snippet should keep the visible grabber example; missing `{needle}`",
        );
    }

    for needle in [
        "with_direction_provider(cx, LayoutDirection::Rtl, |cx| {",
        "\"واحد\"",
        "\"اثنان\"",
        "\"ثلاثة\"",
        ".with_handle(true)",
        ".test_id_prefix(\"ui-gallery-resizable-rtl\")",
    ] {
        assert!(
            rtl.contains(needle),
            "resizable RTL snippet should keep the translated RTL coverage and handle affordance; missing `{needle}`",
        );
    }

    for needle in [
        "ui-gallery-resizable-adaptive-panel-state-wide",
        "ui-gallery-resizable-adaptive-panel-state-compact",
        "FieldOrientation::ContainerAdaptive",
        ".test_id_prefix(\"ui-gallery-resizable-adaptive-panel\")",
        "\"Resize the splitter, not the viewport\"",
        "\"ui-gallery.resizable.adaptive_panel.target\"",
    ] {
        assert!(
            adaptive_panel.contains(needle),
            "resizable adaptive-panel proof snippet should keep the fixed-window container-query teaching surface; missing `{needle}`",
        );
    }

    for needle in [
        "Adaptive Panel Proof",
        "tools/diag-scripts/ui-gallery/resizable/",
        "ui-gallery-resizable-adaptive-panel-proof.json",
        "No extra generic children API is planned unless a real authoring cliff appears",
    ] {
        assert!(
            notes.contains(needle),
            "resizable notes snippet should keep the remaining parity conclusions visible; missing `{needle}`",
        );
    }
}
