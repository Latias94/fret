fn normalize_ws(source: &str) -> String {
    source.split_whitespace().collect()
}

fn canonicalize_rust_fragment(fragment: &str) -> String {
    let mut canonical = fragment.split_whitespace().collect::<String>();
    loop {
        let next = canonical.replace(",)", ")");
        if next == canonical {
            return canonical;
        }
        canonical = next;
    }
}

#[test]
fn scroll_area_page_documents_wrapper_mapping_and_children_api_decision() {
    let source = include_str!("../src/ui/pages/scroll_area.rs");

    for needle in [
        "Reference stack: current shadcn Scroll Area docs/source, new-york-v4 registry chrome, and Radix primitive semantics.",
        "`ScrollArea::new([...])` is the default copyable wrapper lane for the docs surface.",
        "Because the Fret recipe stays layout-only, the upstream root chrome (`rounded-md border` and fixed size) maps to a caller-owned wrapper container around the scroll surface.",
        "`ScrollAreaRoot::new(ScrollAreaViewport::new([...])).scrollbar(ScrollBar::new().orientation(...))` already covers the shadcn/Radix mixed `ScrollArea` + `ScrollBar` examples without widening this family into an untyped arbitrary-children API.",
        "Radix's internal viewport content wrapper and thumb remain mechanism/runtime details in Fret",
        "No generic `children([...])` / `compose()` root API is warranted here because `ScrollArea::new([...])` already covers the upstream children-owned wrapper lane while `ScrollAreaRoot` / `ScrollAreaViewport` / `ScrollBar` keep the explicit parts path typed.",
        "Preview now mirrors the current shadcn docs path first: `Demo`, `Usage`, and `Horizontal`. `RTL`, `API Reference`, and diagnostics remain explicit Fret follow-ups.",
    ] {
        assert!(
            source.contains(needle),
            "scroll area page should document wrapper ownership and the children-api decision; missing `{needle}`",
        );
    }

    let normalized = normalize_ws(source);
    let ordered_sections = normalize_ws(
        r#"
        vec![
            demo,
            usage,
            horizontal,
            rtl,
            api_reference,
            compact_helper,
            nested_scroll_routing,
            drag_baseline,
            expand_at_bottom,
            notes,
        ]
        "#,
    );
    assert!(
        normalized.contains(&ordered_sections),
        "scroll area page should keep the docs-path sections before the Fret-only follow-ups",
    );
}

#[test]
fn scroll_area_docs_surface_covers_last_action_plumbing_and_click_stability_gate() {
    let content = include_str!("../src/ui/content.rs");
    let pages_mod = include_str!("../src/ui/pages/mod.rs");
    let diagnostics = include_str!("../src/ui/diagnostics/scroll_area/drag_baseline.rs");
    let preview_atom = include_str!("../src/ui/previews/gallery/atoms/scroll.rs");
    let script = include_str!(
        "../../../tools/diag-scripts/ui-gallery/scroll-area/ui-gallery-scrollbar-arm-content-growth-click-stability.json"
    );
    let suite =
        include_str!("../../../tools/diag-scripts/suites/ui-gallery-scroll-area/suite.json");

    for needle in [
        "PAGE_SCROLL_AREA => pages::preview_scroll_area(cx, Some(last_action.clone()))",
        "pub(super) fn preview_scroll_area(",
        "last_action: Option<fret_app::Model<std::sync::Arc<str>>>",
        "scroll_area::preview_scroll_area(cx, last_action)",
        "pages::preview_scroll_area(cx, None)",
    ] {
        assert!(
            content.contains(needle) || pages_mod.contains(needle) || preview_atom.contains(needle),
            "scroll area page wiring should keep the last_action plumbing visible; missing `{needle}`",
        );
    }

    for needle in [
        "last_action: Option<Model<Arc<str>>>",
        "ui_gallery.scroll_area.drag_baseline.reset",
        "ui_gallery.scroll_area.drag_baseline.arm_growth",
        "Duration::from_millis(360)",
        "ui-gallery-scroll-area-drag-baseline-arm-grow",
    ] {
        assert!(
            diagnostics.contains(needle) || script.contains(needle),
            "scroll area drag-baseline diagnostics should keep the click-stability gate surface; missing `{needle}`",
        );
    }

    for needle in [
        "\"ui-gallery-scrollbar-arm-content-growth-click-stability\"",
        "tools/diag-scripts/ui-gallery/scroll-area/ui-gallery-scrollbar-arm-content-growth-click-stability.json",
    ] {
        assert!(
            script.contains(needle) || suite.contains(needle),
            "scroll area suite registration should keep the new click-stability gate promoted; missing `{needle}`",
        );
    }
}

#[test]
fn scroll_area_usage_snippet_keeps_wrapper_chrome_and_copyable_root_lane() {
    let source = include_str!("../src/ui/snippets/scroll_area/usage.rs");

    for needle in [
        "use fret::{AppComponentCx, UiChild};",
        "use fret_ui_shadcn::{facade as shadcn, prelude::*};",
        "let area = shadcn::ScrollArea::new([content])",
        ".refine_layout(LayoutRefinement::default().w_full().h_full())",
        "ChromeRefinement::default().border_1().rounded(Radius::Md)",
        "LayoutRefinement::default()",
        ".w_px(Px(350.0))",
        ".h_px(Px(200.0))",
        ".overflow_hidden()",
        "cx.container(props, move |_cx| [area])",
        ".test_id(\"ui-gallery-scroll-area-usage\")",
    ] {
        assert!(
            source.contains(needle),
            "scroll area usage snippet should stay copyable and teach the caller-owned wrapper mapping; missing `{needle}`",
        );
    }
}

#[test]
fn scroll_area_snippets_route_visible_text_through_shared_roles() {
    let cases = [
        (
            "demo",
            include_str!("../src/ui/snippets/scroll_area/demo.rs"),
            &[
                "use fret_ui_kit::declarative::text as decl_text;",
                "decl_text::text_section_chrome_label(cx, \"Tags\")",
                "decl_text::text_list_row_label(cx, tag.clone())",
            ][..],
            &[
                "ui::text(\"Tags\")",
                "ui::text(tag.clone())",
                ".text_size_px(Px(14.0))",
            ][..],
        ),
        (
            "usage",
            include_str!("../src/ui/snippets/scroll_area/usage.rs"),
            &[
                "use fret_ui_kit::declarative::text as decl_text;",
                "decl_text::text_paragraph(cx, story)",
            ][..],
            &[
                "use fret_core::TextWrap;",
                "ui::text(story)",
                ".wrap(TextWrap::Word)",
            ][..],
        ),
        (
            "compact_helper",
            include_str!("../src/ui/snippets/scroll_area/compact_helper.rs"),
            &[
                "use fret_ui_kit::declarative::text as decl_text;",
                "decl_text::text_paragraph(",
            ][..],
            &[
                "use fret_core::TextWrap;",
                "ui::text(",
                ".wrap(TextWrap::Word)",
            ][..],
        ),
        (
            "rtl",
            include_str!("../src/ui/snippets/scroll_area/rtl.rs"),
            &[
                "use fret_ui_kit::declarative::text as decl_text;",
                "decl_text::text_section_chrome_label(cx,",
                "decl_text::text_list_row_label(cx, (41 - idx).to_string())",
            ][..],
            &["ui::text((41 - idx).to_string())", ".text_sm()"][..],
        ),
        (
            "horizontal",
            include_str!("../src/ui/snippets/scroll_area/horizontal.rs"),
            &[
                "use fret_ui_kit::declarative::text as decl_text;",
                "decl_text::text_control_readout(cx, format!(\"Photo by {artist}\"))",
            ][..],
            &["shadcn::typography::muted(format!(\"Photo by {artist}\"))"][..],
        ),
        (
            "nested_scroll_routing",
            include_str!("../src/ui/snippets/scroll_area/nested_scroll_routing.rs"),
            &[
                "use fret_ui_kit::declarative::text as decl_text;",
                "decl_text::text_control_readout(cx, format!(\"Item {i}\"))",
            ][..],
            &["shadcn::typography::muted(format!(\"Item {i}\"))"][..],
        ),
    ];

    for (name, source, required, forbidden) in cases {
        let canonical = canonicalize_rust_fragment(source);
        for marker in required {
            let marker = canonicalize_rust_fragment(marker);
            assert!(
                canonical.contains(&marker),
                "{name} should route scroll-area visible text through shared text roles; missing `{marker}`"
            );
        }
        for marker in forbidden {
            let marker = canonicalize_rust_fragment(marker);
            assert!(
                !canonical.contains(&marker),
                "{name} reintroduced bare or locally styled scroll-area text: `{marker}`"
            );
        }
    }
}
