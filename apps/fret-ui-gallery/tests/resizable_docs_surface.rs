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
        "The Fret-only follow-ups below keep fixed-window splitter, viewport-root overlay ownership, and cached-source movement proofs diagnostics opt-in",
        "FRET_UI_GALLERY_RESIZABLE_ADAPTIVE_PANEL",
        "FRET_UI_GALLERY_RESIZABLE_MULTI_VIEWPORT_COMBOBOX",
        "FRET_UI_GALLERY_RESIZABLE_MOVING_CACHED_COMBOBOX",
        "section_focus_filters",
        "should_build_resizable_section",
        "adaptive_panel_enabled",
        "multi_viewport_combobox_enabled",
        "moving_cached_combobox_enabled",
        "Preview mirrors the shadcn/Base UI Resizable docs path after collapsing the top `ComponentPreview` into `Demo` and skipping `Installation`: `Demo`, `About`, `Usage`, `Vertical`, `Handle`, `RTL`, and `API Reference`. `Adaptive Panel Proof`, `Multi-Viewport Combobox`, and `Moving Cached Combobox` are diagnostics opt-in follow-ups",
        "DocSection::build(cx, \"About\", about)",
        "DocSection::build(cx, \"API Reference\", api_reference)",
        "DocSection::build(cx, \"Adaptive Panel Proof\", adaptive_panel)",
        "Multi-Viewport Combobox",
        "Moving Cached Combobox",
    ] {
        assert!(
            source.contains(needle),
            "resizable page should document source axes and the children-api decision; missing `{needle}`",
        );
    }

    let normalized = normalize_ws(source);
    assert!(
        normalized.contains(&normalize_ws(
            "let focus_filters = doc_layout::section_focus_filters();"
        )),
        "resizable page should read doc-section focus before building snippet previews",
    );
    assert!(
        normalized.contains(&normalize_ws(
            r#"should_build_resizable_section( focus_filters.as_deref(), "Demo", "ui-gallery-resizable-demo", )"#
        )),
        "resizable page should keep the default docs-path sections build-gated by focus filters",
    );
    assert!(
        normalized.contains(&normalize_ws(
            r#"if adaptive_panel_enabled && should_build_resizable_section( focus_filters.as_deref(), "Adaptive Panel Proof", "ui-gallery-resizable-adaptive-panel-proof", ) { let adaptive_panel = snippets::adaptive_panel::render(cx);"#
        )),
        "resizable page should keep the fixed-window container-query proof diagnostics opt-in",
    );
    assert!(
        normalized.contains(&normalize_ws("sections.push(adaptive_panel);")),
        "resizable page should append the opt-in adaptive panel proof before Notes",
    );
    assert!(
        normalized.contains(&normalize_ws(
            r#"if multi_viewport_combobox_enabled && should_build_resizable_section( focus_filters.as_deref(), "Multi-Viewport Combobox", "ui-gallery-resizable-multi-viewport-combobox-docsec", ) { let multi_viewport_combobox = snippets::multi_viewport_combobox::render(cx);"#
        )),
        "resizable page should keep the viewport-root overlay ownership surface diagnostics opt-in",
    );
    assert!(
        normalized.contains(&normalize_ws("sections.push(multi_viewport_combobox);")),
        "resizable page should append the opt-in multi-viewport Combobox section before Notes",
    );
    assert!(
        normalized.contains(&normalize_ws(
            r#"if moving_cached_combobox_enabled && should_build_resizable_section( focus_filters.as_deref(), "Moving Cached Combobox", "ui-gallery-resizable-view-cache-moving-combobox-docsec", ) { let moving_cached_combobox = snippets::moving_cached_combobox::render(cx);"#
        )),
        "resizable page should keep the deeper cached-source movement surface diagnostics opt-in",
    );
    assert!(
        normalized.contains(&normalize_ws("sections.push(moving_cached_combobox);")),
        "resizable page should append the opt-in cached-source movement section before Notes",
    );
    assert!(
        normalized.contains(&normalize_ws(
            r#"should_build_resizable_section( focus_filters.as_deref(), "Notes", "ui-gallery-resizable-notes", )"#
        )),
        "resizable page should keep Notes last after default and opt-in follow-up sections",
    );
}

#[test]
fn resizable_snippets_stay_copyable_and_docs_aligned() {
    let usage = include_str!("../src/ui/snippets/resizable/usage.rs");
    let demo = include_str!("../src/ui/snippets/resizable/demo.rs");
    let handle = include_str!("../src/ui/snippets/resizable/handle.rs");
    let rtl = include_str!("../src/ui/snippets/resizable/rtl.rs");
    let adaptive_panel = include_str!("../src/ui/snippets/resizable/adaptive_panel.rs");
    let multi_viewport_combobox =
        include_str!("../src/ui/snippets/resizable/multi_viewport_combobox.rs");
    let moving_cached_combobox =
        include_str!("../src/ui/snippets/resizable/moving_cached_combobox.rs");
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
        "ui-gallery-resizable-multi-viewport-combobox",
        "shadcn::Combobox::new(value.clone(), open.clone())",
        ".side_offset_px(Px(6.0))",
        ".test_id_prefix(TEST_ID_PREFIX)",
    ] {
        assert!(
            multi_viewport_combobox.contains(needle),
            "resizable multi-viewport Combobox snippet should keep the viewport-root placement fixture; missing `{needle}`",
        );
    }

    for needle in [
        "ui-gallery-resizable-view-cache-moving-combobox",
        "cx.cached_subtree_with(",
        "Move source right",
        "The source element keeps one callsite identity while the parent panel changes.",
        "source_panel(cx, \"right\", source)",
    ] {
        assert!(
            moving_cached_combobox.contains(needle),
            "resizable moving cached Combobox snippet should keep the cached-source movement fixture; missing `{needle}`",
        );
    }

    for needle in [
        "Adaptive Panel Proof",
        "tools/diag-scripts/ui-gallery/resizable/",
        "ui-gallery-resizable-adaptive-panel-proof.json",
        "ui-gallery-resizable-multi-viewport-combobox-placement.json",
        "ui-gallery-resizable-view-cache-moving-combobox-root-boundary.json",
        "No extra generic children API is planned unless a real authoring cliff appears",
    ] {
        assert!(
            notes.contains(needle),
            "resizable notes snippet should keep the remaining parity conclusions visible; missing `{needle}`",
        );
    }
}
